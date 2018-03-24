use dbus;

use std::rc::Rc;

use std::sync::mpsc;
use std::thread;
use std::time;
use tts::TTSService;
use sound::{SoundService, Sound};

pub const DBUS_ID: &'static str = "com.github.vvlad.tts";
pub const DBUS_PATH: &'static str = "/com/github/vvlad/tts";

pub enum Message {
    Speak(String),
    Stop
}

#[derive(Clone)]
pub struct Controller{
    channel: mpsc::Sender<Message>
}

impl Controller {
    pub fn run(channel: mpsc::Receiver<Message>) {
        let (sound, sound_channel) = mpsc::channel();
        let (tts, tts_channel) = mpsc::channel();
        let (sound_packets_channel, sound_packets) = mpsc::channel();

        thread::spawn(move||{ SoundService::run(sound_channel) });
        thread::spawn(move||{ TTSService::run(tts_channel, sound_packets_channel); });

        loop {
            select! {
                command = channel.recv() => match command {
                    Ok(Message::Speak(text)) => { 
                        sound.send(Sound::Stop).ok();
                        thread::sleep(time::Duration::from_millis(100));
                        tts.send(text).ok(); 
                    }
                    Ok(Message::Stop) => { sound.send(Sound::Stop).ok(); }
                    Err(err) => { panic!(err) }
                },
                packet = sound_packets.recv() => match packet {
                    Ok(packet) => { sound.send(Sound::Play(packet)).ok(); },
                    Err(err) => { panic!(err) }
                }
            }
        }
    }

    pub fn new(channel: mpsc::Sender<Message>)-> Controller {
        Controller{
            channel: channel
        }
    }

    pub fn speak(&self, text: &str) {
        self.channel.send(Message::Speak(text.to_owned())).ok();
    }

    pub fn stop(&self) {
        self.channel.send(Message::Stop).ok();
    }
}

dbus_class!(DBUS_ID, class DBusService (controller: Controller) {
    fn say(&this, text: &str) {
        this.controller.speak(text);
    }

    fn flush(&this) {
        this.controller.stop();
    }
});


dbus_interface!(DBUS_ID, interface DBusClient {
    fn say(text: &str);
    fn flush();
});

