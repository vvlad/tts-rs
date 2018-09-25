use punkt::params::Standard;
use punkt::{SentenceTokenizer, TrainingData};
use xml::writer::{EmitterConfig, XmlEvent};

use std::slice;
use std::vec::IntoIter;

const MAX_LENGTH: usize = 1500;
const SPEAK_TAG: &'static str = "speak";
const PROSODY_TAG: &'static str = "prosody";

#[derive(Clone)]
pub struct Sentence(String);

impl Sentence {
  pub fn to_xml(&self) -> String {
    let mut bytes = Vec::new();
    {
      let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut bytes);

      writer.write(XmlEvent::start_element(SPEAK_TAG)).is_ok();
      writer
        .write(XmlEvent::start_element(PROSODY_TAG).attr("rate", "fast"))
        .is_ok();
      writer.write(XmlEvent::characters(&self.0)).is_ok();
      writer.write(XmlEvent::end_element()).is_ok();
      writer.write(XmlEvent::end_element()).is_ok();
    }
    String::from_utf8(bytes).unwrap_or_default()
  }
}

pub struct Sentences(Vec<Sentence>);

lazy_static! {
  static ref BRAIN: TrainingData = { TrainingData::english() };
}

impl<T> From<T> for Sentences
where
  T: AsRef<str>,
{
  fn from(text: T) -> Sentences {
    let mut parts: Vec<Sentence> = vec![];

    for sentence in SentenceTokenizer::<Standard>::new(text.as_ref(), &BRAIN) {
      let mut sentence = str::replace(sentence.trim(), "\n\n", "\n");
      while sentence.len() > MAX_LENGTH {
        parts.push(Sentence(sentence[0..MAX_LENGTH].to_owned()));
        sentence = sentence[MAX_LENGTH..].to_owned();
      }
      parts.push(Sentence(sentence));
    }

    Sentences(parts)
  }
}

impl Sentences {
  pub fn iter(&self) -> slice::Iter<Sentence> {
    self.0.iter()
  }
  pub fn into_iter(&self) -> IntoIter<Sentence> {
    self.0.clone().into_iter()
  }
}
