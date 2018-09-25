#![feature(mpsc_select)]

extern crate dbus;
#[macro_use]
extern crate dbus_macros;
extern crate futures;

use std::sync::mpsc;
use std::thread;

extern crate tts;
use tts::{Config, Player, Sentence, Sentences, Voice, DBUS_ID, DBUS_PATH};

pub enum Command {
    Say(Sentence),
    Stop,
}

dbus_class!(DBUS_ID, class DBusService (commands: mpsc::Sender<Command>) {
    fn say(&this, text: &str) {
        let sentences = Sentences::from(text);
        for sentence in sentences.into_iter() {
            this.commands.send(Command::Say(sentence)).expect("success");
        }
    }

    fn flush(&this) {
        this.commands.send(Command::Stop).expect("success");
    }
});

pub fn main() {
    let (channel, commands) = mpsc::channel();

    thread::spawn(move || {
        let config = Config::new();
        let player = Player::new();
        let voice = Voice::from(config);

        while let Ok(command) = commands.recv() {
            match command {
                Command::Say(sentence) => {
                    let item = voice.speak(sentence);
                    player.enqueue(item);
                }
                Command::Stop => player.reset(),
            }
        }
    });

    let connection = dbus::Connection::get_private(dbus::BusType::Session).unwrap();
    let service = DBusService::new(channel);
    service.run(DBUS_ID, &connection, DBUS_PATH);
}
