extern crate dbus;
extern crate tts;

use tts::{DBusService, SpeechServer, DBUS_ID, DBUS_PATH};

use std::sync::mpsc;
use std::thread;


fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn( move || {
        SpeechServer::run(rx);
    });

    let service = DBusService::new(tx);
    let connection = dbus::Connection::get_private(dbus::BusType::Session).unwrap();
    service.run(DBUS_ID, &connection, DBUS_PATH);
}
