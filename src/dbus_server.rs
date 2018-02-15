
use dbus;

use std::sync::mpsc;
use std::thread;

use speech_server;

dbus_class!("com.accesibility.speech", class DBusService (channel: mpsc::Sender<speech_server::Message>) {
    fn say(&this, text: &str) {
        this.channel.send(speech_server::Message::SpeakText(text.to_string())).unwrap();
    }

    fn flush(&this) {
        this.channel.send(speech_server::Message::Flush).unwrap();
    }
});

pub fn run() {

    let (tx, rx) = mpsc::channel();
    let worker = thread::spawn( move || {
        speech_server::run(rx);
    });

    let service = DBusService::new(tx);
    let connection = dbus::Connection::get_private(dbus::BusType::Session).unwrap();
    service.run("com.accesibility.speech", &connection, "/Speech");
    worker.join().unwrap();
}
