#![feature(mpsc_select)]
extern crate dbus;
#[macro_use]
extern crate dbus_macros;
//mod server;
mod tts;
mod sound;
mod dbus_server;

//pub use server::{DBusService, SpeechServer, DBusClient, DBUS_ID, DBUS_PATH};
pub use tts::TTSService;
pub use sound::{SoundService, Sound};
pub use dbus_server::{DBUS_ID, DBUS_PATH, DBusClient, DBusService, Controller};
