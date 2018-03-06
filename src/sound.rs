extern crate alto;
extern crate lewton;
extern crate byteorder;

use self::lewton::inside_ogg::OggStreamReader;
use self::alto::{Alto, Mono, Stereo, Source, Context, StreamingSource, SourceState};
use std::io::Cursor;
use std::sync::mpsc;


pub enum Sound {
    Play(Vec<u8>),
    Stop,
    ExitWhenDone
}

pub struct SoundService {
    context: Context,
}

impl SoundService {

    pub fn new() -> SoundService {
        let al = Alto::load_default().expect("Could not load alto");
        let device = al.open(None).expect("Could not open device");
        SoundService{
            context: device.new_context(None).expect("Could not create context"),
        }
    }

    pub fn run(channel: mpsc::Receiver<Sound>) {
        let service = SoundService::new();
        let mut current_stream = None;
        
        loop {
            match channel.recv() {
                Ok(Sound::Play(message)) => { current_stream = Some(service.play(message, current_stream)) },
                Ok(Sound::Stop) => { if let Some(mut stream) = current_stream.take() { stream.stop() } },
                Ok(Sound::ExitWhenDone) => { if let Some(mut stream) = current_stream.take() {
                    if stream.state() != SourceState::Playing {
                        stream.stop();
                    }else{
                        current_stream = Some(stream);
                    }
                }}
                Err(err) => { panic!(err) }
            }
        }
    }

    pub fn play(&self, buffer: Vec<u8>, stream: Option<StreamingSource>) -> StreamingSource {
        let mut stream = self.reflect_on_stream(stream); 
        let mut ogg = match OggStreamReader::new(Cursor::new(buffer)) {
            Ok(stream) => stream,
            Err(err) => { 
                println!("error: {:?}", err);
                return stream; 
            }
        };
        let sample_rate = ogg.ident_hdr.audio_sample_rate as i32;

        if ogg.ident_hdr.audio_channels > 2 {
            println!("Stream error: {} channels are too many!", ogg.ident_hdr.audio_channels);
        }

        while let Ok(pck_samples) = ogg.read_dec_packet_itl() {
            if let Some(samples) = pck_samples {
                let buf = match ogg.ident_hdr.audio_channels {
                    1 => self.context.new_buffer::<Mono<i16>,_>(&samples, sample_rate),
                    2 => self.context.new_buffer::<Stereo<i16>,_>(&samples, sample_rate),
                    n => panic!("unsupported number of channels: {}", n),
                }.unwrap();
                stream.queue_buffer(buf).ok();
            }else{
                break;
            }
        }

        if stream.state() != SourceState::Playing {
            stream.play();
        }
        return stream;
    }

    fn reflect_on_stream(&self, stream: Option<StreamingSource>) -> StreamingSource {
        let new_stream = self.context.new_streaming_source().expect("cloud not create streaming src");

        match stream {
            None => new_stream,
            Some(stream) => {
                match stream.state() {
                    SourceState::Initial => stream,
                    SourceState::Playing => stream, 
                    _ => new_stream
                }
            }
        }
    }

}

