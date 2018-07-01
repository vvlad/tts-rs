#![feature(mpsc_select)]
#![feature(duration_extras)]
#![feature(iterator_flatten)]

extern crate dbus;
#[macro_use]
extern crate dbus_macros;
extern crate toml;

#[macro_use]
extern crate serde_derive;
extern crate ini;

mod config;
mod sound;
mod tts;

pub use config::Config;
pub use sound::{Sound, SoundService};
pub use tts::TTSService;

use std::rc::Rc;
pub const DBUS_ID: &'static str = "com.github.vvlad.tts";
pub const DBUS_PATH: &'static str = "/com/github/vvlad/tts";

dbus_interface!(DBUS_ID, interface DBusClient {
    fn say(text: &str);
    fn flush();
});
