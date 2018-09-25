extern crate punkt;
#[macro_use]
extern crate lazy_static;
extern crate lewton;
extern crate rusoto_core;
extern crate rusoto_polly;
extern crate tokio;
extern crate xml;
#[macro_use]
extern crate error_chain;
extern crate cpal;
extern crate dbus;
#[macro_use]
extern crate dbus_macros;
extern crate ini;
#[macro_use]
extern crate serde_derive;
extern crate dirs;
extern crate toml;

mod errors;

pub use self::errors::Error;

mod voice;
pub use self::voice::Voice;
mod sentence;
pub use self::sentence::{Sentence, Sentences};
mod player;
pub use self::player::Player;
mod dbus_interface;
pub use dbus_interface::{DBusClient, DBUS_ID, DBUS_PATH};
mod config;
pub use config::Config;
