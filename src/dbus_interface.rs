use dbus;

use std::rc::Rc;
pub const DBUS_ID: &'static str = "com.github.vvlad.tts";
pub const DBUS_PATH: &'static str = "/com/github/vvlad/tts";


dbus_interface!(DBUS_ID, interface DBusClient {
    fn say(text: &str);
    fn flush();
});

