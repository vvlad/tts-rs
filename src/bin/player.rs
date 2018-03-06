extern crate tts;

use tts::{SoundService, Sound};

use std::thread;
use std::sync::mpsc;

pub fn main(){
    let (tx, rx) = mpsc::channel();
    let th = thread::spawn( move || { SoundService::run(rx) } );

    tx.send(Sound::ExitWhenDone).is_ok();

    th.join().is_ok();
}
