
extern crate dbus;
extern crate tts;

use std::thread;
use std::sync::mpsc;
use tts::{DBUS_ID, DBUS_PATH, Controller, DBusService};

pub fn main() {

    let (tx, rx) = mpsc::channel();

    thread::spawn( move || {
        Controller::run(rx);
    });

    let controller = Controller::new(tx);
    let connection = dbus::Connection::get_private(dbus::BusType::Session).unwrap();
    let service = DBusService::new(controller);
    service.run(DBUS_ID, &connection, DBUS_PATH);
}
