
extern crate rusoto_core;
extern crate rusoto_polly;
extern crate quick_xml;
extern crate punkt;

use self::rusoto_polly::{Polly, PollyClient, SynthesizeSpeechInput};
use self::rusoto_core::Region;
use self::punkt::{TrainingData, SentenceTokenizer};
use self::punkt::params::Standard;

use self::quick_xml::writer::Writer;
use self::quick_xml::events::{Event, BytesEnd, BytesStart, BytesText};
use std::io::Cursor;
use std::sync::mpsc;


pub struct TTSService {
   aws: PollyClient,
   rate: &'static str,
   brain: TrainingData,
   channel: mpsc::Sender<Vec<u8>>
}

const MAX_LENGTH: usize = 1500;

impl TTSService {
    pub fn new(channel: mpsc::Sender<Vec<u8>>) -> TTSService {
        TTSService{
           aws:     PollyClient::simple(Region::EuCentral1),
           rate:    "fast",
           brain:   TrainingData::english(),
           channel: channel
        }
    }

    pub fn run(channel: mpsc::Receiver<String>, broadcast: mpsc::Sender<Vec<u8>>) {
        let service = TTSService::new(broadcast);

        loop {
            match channel.recv() {
                Ok(text) => service.speak(&text),
                Err(_) => { return }
            };
        }
    }

    pub fn speak(&self, text: &str) {
        let mut input = SynthesizeSpeechInput::default();
        input.voice_id = "Salli".to_string();
        input.output_format = "ogg_vorbis".to_string();
        input.text_type = Some("ssml".to_string());
        let mut parts: Vec<String> = vec![];

        for sentence in SentenceTokenizer::<Standard>::new(text, &self.brain) {
            let mut sentence = str::replace(sentence.trim(), "\n\n", "\n");
            while sentence.len() > MAX_LENGTH {
                parts.push(sentence[0.. MAX_LENGTH].to_owned());
                sentence = sentence[MAX_LENGTH..].to_owned();
            }
            parts.push(sentence);
        }

        for part in parts {
            input.text = self.to_xml(part); 
            match self.aws.synthesize_speech(&input).sync() {
                Ok(output) => { 
                    self.channel.send(output.audio_stream.unwrap()).ok();
                }
                Err(e) => { 
                    println!("error: {:?}", e) 
                }
            };
        }
    }

    fn to_xml(&self, text: String) -> String {
        let bytes = Vec::new();
        let mut writer = Writer::new(Cursor::new(bytes));


        let speak_tag = SPEAK_TAG.to_owned().into_bytes();
        let speak_start = BytesStart::owned(speak_tag.clone(), speak_tag.len());
        let speak_end = BytesEnd::borrowed(speak_tag.as_slice());

        let prosody_tag = PROSODY_TAG.to_owned().into_bytes();
        let mut prosody_start = BytesStart::owned(prosody_tag.clone(), prosody_tag.len());
        let prosody_end = BytesEnd::borrowed(prosody_tag.as_slice()); 

        prosody_start.push_attribute(("rate", self.rate));

        writer.write_event(Event::Start(speak_start)).is_ok();

        writer.write_event(Event::Start(prosody_start)).is_ok();
        writer.write_event(Event::Text(BytesText::from_str(text))).is_ok(); 
        writer.write_event(Event::End(prosody_end)).is_ok();

        writer.write_event(Event::End(speak_end)).is_ok();

        let result = writer.into_inner().into_inner();
        return String::from_utf8(result).unwrap();
    }
}

const SPEAK_TAG : &'static str = "speak";
const PROSODY_TAG : &'static str = "prosody";

