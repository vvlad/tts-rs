extern crate alto;
extern crate lewton;
extern crate byteorder;

use self::lewton::inside_ogg::OggStreamReader;
use self::alto::{Alto, Mono, Stereo, Source, Context, StreamingSource, SourceState};
use std::io::Cursor;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::sync::{Arc, RwLock};

type SoundBuffer = Vec<u8>;

pub enum Sound {
    Play(SoundBuffer),
    Stop,
    ExitWhenDone
}

pub struct SoundService {
}

enum WorkerCommand {
    Play(SoundSample),
    Stop
}

struct SoundSample {
	bytes: Vec<i16>,
	sample_rate: i32,
	channels: i32
}

#[derive(PartialEq)]
enum WorkerState {
    Initializing,
    Running,
    Dead
}

struct Player {
    worker_commands: Option<mpsc::Sender<WorkerCommand>>,
    state: Arc<RwLock<WorkerState>>
}

impl Player {
    pub fn new() -> Self {
        Player{
            worker_commands: None,
            state: Arc::new(RwLock::new(WorkerState::Dead))
        }
    }

    pub fn enqueue(&mut self, sound: SoundSample){
        self.send(WorkerCommand::Play(sound));
    }

    fn send(&mut self, command: WorkerCommand){
        if ! self.is_playing() {
            self.worker_commands = None
        }
        let channel = match self.worker_commands.take() {
            Some(channel) => { channel }
            None => {
                let (tx, rx) = mpsc::channel();
                *self.state.write().unwrap() = WorkerState::Initializing;
                let state = self.state.clone();
                thread::spawn(||{Self::play(rx, state)});
                self.worker_commands = Some(tx.clone());
                while !self.is_playing() {
                    thread::sleep(time::Duration::from_millis(10));
                }
                tx
            }
        };
        channel.send(command).is_ok();
        self.worker_commands = Some(channel);
    }

    pub fn stop(&mut self) {
        self.send(WorkerCommand::Stop);
    }
    pub fn is_playing(&self) -> bool { *self.state.read().unwrap() == WorkerState::Running }

    fn play(channel: mpsc::Receiver<WorkerCommand>, state: Arc<RwLock<WorkerState>>) {
        *state.write().unwrap() = WorkerState::Running;
        let al = Alto::load_default().expect("Could not load alto");
        let device = al.open(None).expect("Could not open device");
        let context = device.new_context(None).expect("Could not create context");
        let mut stream = context.new_streaming_source().expect("cloud not create streaming src");
        let mut play_time = 0.0;
        let mut start_play_time = None;
        loop {
            for command in channel.try_iter(){
                match command {
                    WorkerCommand::Play(sample) => {
                        let buf = match sample.channels {
                            1 => context.new_buffer::<Mono<i16>,_>(&sample.bytes, sample.sample_rate),
                            2 => context.new_buffer::<Stereo<i16>,_>(&sample.bytes, sample.sample_rate),
                            n => panic!("unsupported number of channels: {}", n),
                        }.unwrap();
                        stream.queue_buffer(buf).ok();
                        play_time += sample.bytes.len() as f32 / ( sample.channels * sample.sample_rate) as f32;
                    },
                    WorkerCommand::Stop => { 
                        *state.write().unwrap() = WorkerState::Dead;
                        return;
                    },
                }
            }

            if start_play_time == None && play_time > 0.0 as f32 {
                stream.play();
                start_play_time = Some(time::Instant::now());
            }

            let total_duration = time::Duration::from_millis((play_time * 1000.0) as u64);
            let played_time = time::Instant::now() - start_play_time.unwrap_or(time::Instant::now());
            if played_time > total_duration && play_time > 0.0 as f32 {
                println!("Done playing {}.{} second(s)", played_time.as_secs() , played_time.subsec_millis());
                *state.write().unwrap() = WorkerState::Dead;
                return;
            }else{
                thread::sleep(time::Duration::from_millis(50));
            }
        }
    }
}


impl SoundService {

    pub fn run(channel: mpsc::Receiver<Sound>) {
		let mut player = Player::new();
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
                        println!("Stream error: {} channels are too many!", ogg.ident_hdr.audio_channels);
                        continue;
                    }

                    while let Ok(pck_samples) = ogg.read_dec_packet_itl() {
                        if let Some(samples) = pck_samples {
							player.enqueue(SoundSample{
                                bytes: samples,
                                channels: channels,
                                sample_rate: sample_rate
							});
                        }else{
                            break;
                        }
                    }


                },
                Ok(Sound::Stop) => { 
                    player.stop();
                },
                Ok(Sound::ExitWhenDone) => { 
                    while player.is_playing() {
                        thread::sleep(time::Duration::from_secs(1));
                    }
                }
                Err(err) => { panic!(err) }
            }
        }
    }
}

