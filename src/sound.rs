extern crate alto;
extern crate byteorder;
extern crate lewton;

use self::alto::{Alto, Mono, Source, Stereo};
use self::lewton::inside_ogg::OggStreamReader;
use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time;

type SoundBuffer = Vec<u8>;

pub enum Sound {
    Play(SoundBuffer),
    Stop,
}

struct SoundSample {
    bytes: Vec<i16>,
    sample_rate: i32,
    channels: i32,
}

struct Player {
    queue: Arc<Mutex<VecDeque<SoundSample>>>,
    playing: Weak<AtomicBool>,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            playing: Weak::new(),
        }
    }
}

impl Player {
    fn play(&mut self, sound_samples: Vec<SoundSample>) {
        match self.queue.lock() {
            Ok(mut queue) => {
                queue.append(&mut sound_samples.into_iter().collect());
            }
            Err(error) => panic!("Player::play(): {}", error),
        };
        self.maybe_spawn_worker();
    }

    fn stop(&self) {
        if let Some(is_playing) = self.playing.upgrade() {
            (*is_playing).store(false, Ordering::Relaxed);
        }
        if let Ok(mut queue) = self.queue.lock() {
            queue.clear();
        }
    }

    fn maybe_spawn_worker(&mut self) {
        if self.playing.upgrade().is_none() {
            let keep_playing = Arc::new(AtomicBool::new(true));
            self.playing = Arc::downgrade(&keep_playing);

            let queue = self.queue.clone();
            thread::spawn(move || {
                Self::work(queue, keep_playing);
            });
        }
    }

    fn work(queue: Arc<Mutex<VecDeque<SoundSample>>>, keep_playing: Arc<AtomicBool>) {
        let al = Alto::load_default().expect("Could not load alto");
        let device = al.open(None).expect("Could not open device");
        let context = device.new_context(None).expect("Could not create context");
        let mut stream = context
            .new_streaming_source()
            .expect("cloud not create streaming src");
        let mut play_time = 0.0;
        let mut start_play_time = None;

        loop {
            let mut samples: Vec<SoundSample> = vec![];
            if let Ok(mut queue) = queue.lock() {
                samples = queue.drain(..).collect();
            }

            for sample in samples {
                let buf = match sample.channels {
                    1 => context.new_buffer::<Mono<i16>, _>(&sample.bytes, sample.sample_rate),
                    2 => context.new_buffer::<Stereo<i16>, _>(&sample.bytes, sample.sample_rate),
                    n => panic!("unsupported number of channels: {}", n),
                }.unwrap();
                stream.queue_buffer(buf).ok();
                play_time +=
                    sample.bytes.len() as f32 / (sample.channels * sample.sample_rate) as f32;
            }
            if start_play_time == None && play_time > 0.0 as f32 {
                stream.play();
                start_play_time = Some(time::Instant::now());
            }

            let total_duration = time::Duration::from_millis((play_time * 1000.0) as u64);
            let played_time =
                time::Instant::now() - start_play_time.unwrap_or(time::Instant::now());
            let should_stop = !(*keep_playing).load(Ordering::Relaxed);
            if played_time > total_duration && play_time > 0.0 as f32 || should_stop {
                return;
            } else {
                thread::sleep(time::Duration::from_millis(500));
            }
        }
    }
}

pub struct SoundService {}

impl SoundService {
    pub fn run(channel: mpsc::Receiver<Sound>) {
        let mut player = Player::default();

        loop {
            match channel.recv() {
                Ok(Sound::Play(buffer)) => {
                    let mut ogg = match OggStreamReader::new(Cursor::new(buffer)) {
                        Ok(stream) => stream,
                        Err(err) => {
                            println!("error: {:?}", err);
                            return;
                        }
                    };

                    let sample_rate = ogg.ident_hdr.audio_sample_rate as i32;
                    let channels = ogg.ident_hdr.audio_channels as i32;
                    if channels > 2 {
                        println!(
                            "Stream error: {} channels are too many!",
                            ogg.ident_hdr.audio_channels
                        );
                        continue;
                    }

                    let mut sound_samples: Vec<SoundSample> = vec![];

                    while let Ok(pck_samples) = ogg.read_dec_packet_itl() {
                        if let Some(samples) = pck_samples {
                            sound_samples.push(SoundSample {
                                bytes: samples,
                                channels: channels,
                                sample_rate: sample_rate,
                            });
                        } else {
                            break;
                        }
                    }
                    player.play(sound_samples);
                }
                Ok(Sound::Stop) => {
                    player.stop();
                }
                Err(err) => {
                    println!("err: {}", err);
                    panic!(err)
                }
            }
        }
    }
}
