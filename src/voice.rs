use lewton::inside_ogg::OggStreamReader;
use rusoto_core::{Region, RusotoFuture};
use rusoto_polly::{
  Polly, PollyClient, SynthesizeSpeechError, SynthesizeSpeechInput, SynthesizeSpeechOutput,
};
use std::io::Cursor;
use tokio::prelude::*;
pub type SoundBuffer = Vec<i16>;

use errors::Error;

use config::Config;
use sentence::Sentence;

#[derive(Clone)]
pub struct Voice {
  region: Region,
  name: String,
}

impl From<Config> for Voice {
  fn from(config: Config) -> Voice {
    Voice {
      region: config.aws_region(),
      name: config.voice.clone().unwrap_or("Salli".to_string()),
    }
  }
}

impl Voice {
  pub fn new(region: Region) -> Voice {
    Voice {
      region,
      name: "Salli".to_string(),
    }
  }

  pub fn speak(&self, sentence: Sentence) -> SpeakFuture {
    let polly = PollyClient::new(self.region.clone());
    let mut input = SynthesizeSpeechInput::default();
    input.voice_id = self.name.clone();
    input.output_format = "ogg_vorbis".to_string();
    input.text_type = Some("ssml".to_string());
    input.text = sentence.to_xml();

    SpeakFuture {
      future: Some(polly.synthesize_speech(input)),
    }
  }
}

pub struct SpeakFuture {
  future: Option<RusotoFuture<SynthesizeSpeechOutput, SynthesizeSpeechError>>,
}

impl Future for SpeakFuture {
  type Item = SoundBuffer;
  type Error = Error;

  fn poll(&mut self) -> Result<Async<SoundBuffer>, Self::Error> {
    let sound = self.future.take().unwrap().sync()?;
    let mut ogg = OggStreamReader::new(Cursor::new(sound.audio_stream.unwrap()))?;
    let mut buf = vec![];
    while let Some(mut samples) = ogg.read_dec_packet_itl()? {
      buf.append(&mut samples);
    }

    Ok(Async::Ready(buf))
  }
}
