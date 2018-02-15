extern crate rusoto_polly;
extern crate rusoto_core;

use self::rusoto_polly::{Polly, PollyClient, SynthesizeSpeechInput};
use self::rusoto_core::Region;

extern crate rodio;

use dbus;

use std::io::BufReader;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::mpsc;


pub enum Message {
    SpeakText(String),
    Flush,
    Quit,
}

pub struct SpeechServer {
    aws: PollyClient,
    input: SynthesizeSpeechInput,
    rate: &'static str,
    sink: rodio::Sink,
    endpoint: rodio::Endpoint,
}

impl SpeechServer {

    pub fn run(channel: mpsc::Receiver<Message>) {
        let mut server = SpeechServer::new();
       
        loop {
            match channel.recv() {
                Ok(Message::SpeakText(text)) => server.speak(text),
                Ok(Message::Flush) => server.flush(),
                Ok(Message::Quit) => { break; }
                Err(e) => {
                    println!("error: {:?}", e)
                }
            }
        }

        server.done();
    }

    fn new() -> SpeechServer {

        let endpoint = rodio::default_endpoint().unwrap();

        let mut input = SynthesizeSpeechInput::default(); 
        input.voice_id = "Salli".to_string(); 
        input.output_format = "ogg_vorbis".to_string();
        input.text_type = Some("ssml".to_string());

        SpeechServer{
            aws: PollyClient::simple(Region::EuCentral1),
            input: input,
            rate: "fast",
            sink: rodio::Sink::new(&endpoint),
            endpoint: endpoint
        }
    }

    pub fn speak(&mut self, text: String) {
        let sink = &self.sink; 
        let mut input = self.input.clone();

        input.text = format!("<speak><prosody rate=\"{}\">{}</prosody></speak>", self.rate, text); 
        let stream = match self.aws.synthesize_speech(&input).sync() {
            Ok(output) => Cursor::new(output.audio_stream.unwrap()),
            Err(e) => { println!("error: {:?}", e); return; }
        };

        sink.append(rodio::Decoder::new(BufReader::new(stream)).unwrap());
    }

    pub fn done(&self) {
       self.sink.sleep_until_end();
    }

    pub fn flush(&mut self) {
        self.sink.stop();
        self.sink = rodio::Sink::new(&self.endpoint);
        println!("Sink flushed");
    }
}

pub const DBUS_ID: &'static str = "com.github.vvlad.tts"; 
pub const DBUS_PATH: &'static str = "/com/github/vvlad/tts"; 

dbus_class!(DBUS_ID, class DBusService (channel: mpsc::Sender<Message>) {
    fn say(&this, text: &str) {
        this.channel.send(Message::SpeakText(text.to_string())).unwrap();
    }

    fn flush(&this) {
        this.channel.send(Message::Flush).unwrap();
    }
});


dbus_interface!(DBUS_ID, interface DBusClient {
    fn say(text: &str); 
    fn flush();
});

