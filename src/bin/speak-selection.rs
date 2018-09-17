extern crate clipboard;
extern crate dbus;
extern crate md5;
extern crate tts;

use clipboard::x11_clipboard::{Primary, X11ClipboardContext};
use clipboard::ClipboardProvider;
use std::fs::{remove_file, File};
use std::io::prelude::*;
use std::rc::Rc;

const STATE_FILE: &'static str = "/tmp/speak-selection.md5";

fn main() {
    let session_connection =
        Rc::new(dbus::Connection::get_private(dbus::BusType::Session).unwrap());
    let engine = tts::DBusClient::new(tts::DBUS_ID, tts::DBUS_PATH, session_connection);
    let mut context: X11ClipboardContext<Primary> = ClipboardProvider::new().unwrap();

    if let Ok(text) = context.get_contents() {
        let digest = format!("{:x}", md5::compute(text.clone().into_bytes()));
        if same_text(digest.clone()) {
            engine.flush().ok();
            remove_file(STATE_FILE).ok();
            return;
        }

        File::create(STATE_FILE)
            .expect("can't create state file")
            .write_all(&digest.clone().into_bytes())
            .ok();
        engine.say(&text).ok();
    } else {
        engine.flush().ok();
    }
}

fn same_text(digest: String) -> bool {
    if let Ok(mut file) = File::open(STATE_FILE) {
        let mut content = String::new();
        file.read_to_string(&mut content).ok();
        return content == digest;
    }
    return false;
}
