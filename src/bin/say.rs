extern crate dbus;

extern crate atty;
use atty::Stream;

extern crate tts;

use std::io::{self, Read};
use std::rc::Rc;

use std::env;

fn main() {
    let mut text = env::args_os()
        .skip(1)
        .map(|s| s.to_str().unwrap().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    if text.is_empty() && !atty::is(Stream::Stdin) {
        io::stdin().read_to_string(&mut text).unwrap();
    }

    let session_connection =
        Rc::new(dbus::Connection::get_private(dbus::BusType::Session).unwrap());
    let hello = tts::DBusClient::new(tts::DBUS_ID, tts::DBUS_PATH, session_connection);

    if text.is_empty() {
        hello.flush().unwrap();
    } else {
        hello.say(&text).unwrap();
    }
}
