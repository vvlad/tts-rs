extern crate punkt;
extern crate rusoto_core;
extern crate rusoto_polly;
extern crate xml;

use self::punkt::params::Standard;
use self::punkt::{SentenceTokenizer, TrainingData};
use self::rusoto_core::Region;
use self::rusoto_polly::{Polly, PollyClient, SynthesizeSpeechInput};
use self::xml::writer::{EmitterConfig, XmlEvent};

use config::Config;
use std::env;
use std::str::FromStr;
use std::sync::mpsc;

pub struct TTSService {
    aws: PollyClient,
    brain: TrainingData,
    channel: mpsc::Sender<Vec<u8>>,
    config: Config,
    voice: String,
}

const MAX_LENGTH: usize = 1500;

impl TTSService {
    pub fn new(config: Config, channel: mpsc::Sender<Vec<u8>>) -> TTSService {
        let cfg = config.clone();
        let region = Region::from_str(&config.aws_region.clone().unwrap()).unwrap();

        env::set_var("AWS_ACCESS_KEY", config.aws_access_key.unwrap());
        env::set_var("AWS_SECRET_ACCESS_KEY", config.aws_secret_key.unwrap());
        let aws = PollyClient::simple(region);
        let voice_name = config.voice.unwrap_or_default();

        TTSService {
            aws: aws,
            brain: TrainingData::english(),
            channel: channel,
            voice: voice_name,
            config: cfg,
        }
    }

    pub fn run(config: Config, channel: mpsc::Receiver<String>, broadcast: mpsc::Sender<Vec<u8>>) {
        let service = TTSService::new(config, broadcast);

        loop {
            match channel.recv() {
                Ok(text) => service.speak(&text),
                Err(_) => return,
            };
        }
    }

    pub fn speak(&self, text: &str) {
        let mut input = SynthesizeSpeechInput::default();
        input.voice_id = self.voice.clone(); //"Salli".to_string();
        input.output_format = "ogg_vorbis".to_string();
        input.text_type = Some("ssml".to_string());
        let mut parts: Vec<String> = vec![];

        for sentence in SentenceTokenizer::<Standard>::new(text, &self.brain) {
            let mut sentence = str::replace(sentence.trim(), "\n\n", "\n");
            while sentence.len() > MAX_LENGTH {
                parts.push(sentence[0..MAX_LENGTH].to_owned());
                sentence = sentence[MAX_LENGTH..].to_owned();
            }
            parts.push(sentence);
        }

        for part in parts {
            input.text = self.to_xml(part);
            match self.aws.synthesize_speech(&input.clone()).sync() {
                Ok(output) => {
                    self.channel.send(output.audio_stream.unwrap()).ok();
                }
                Err(e) => println!("error: {:?}", e),
            };
        }
    }

    fn to_xml(&self, text: String) -> String {
        let mut bytes = Vec::new();
        {
            let mut writer = EmitterConfig::new()
                .perform_indent(true)
                .create_writer(&mut bytes);

            writer.write(XmlEvent::start_element(SPEAK_TAG)).is_ok();
            writer
                .write(
                    XmlEvent::start_element(PROSODY_TAG)
                        .attr("rate", &self.config.speak_rate.clone().unwrap()),
                )
                .is_ok();
            writer.write(XmlEvent::characters(&text)).is_ok();
            writer.write(XmlEvent::end_element()).is_ok();
            writer.write(XmlEvent::end_element()).is_ok();
        }
        String::from_utf8(bytes).unwrap_or_default()
    }
}

const SPEAK_TAG: &'static str = "speak";
const PROSODY_TAG: &'static str = "prosody";
