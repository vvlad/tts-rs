extern crate dbus;
#[macro_use]
extern crate dbus_macros;
mod server;

pub use server::{DBusService, SpeechServer, DBusClient, DBUS_ID, DBUS_PATH};
