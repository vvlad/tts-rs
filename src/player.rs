use cpal;
use cpal::{StreamData, UnknownTypeOutputBuffer};
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use tokio::prelude::*;
use voice::SpeakFuture;

pub struct Player {
  sounds: Sounds,
  thread: thread::JoinHandle<()>,
}

impl Player {
  pub fn new() -> Player {
    let sounds = Sounds::new();
    let thread = {
      let sounds = sounds.clone();
      thread::spawn(move || play(sounds))
    };

    Player {
      sounds: sounds,
      thread: thread,
    }
  }

  pub fn enqueue(&self, future: SpeakFuture) {
    let mut buffer = future.wait().expect("success");
    self.sounds.append(&mut buffer);
    self.thread.thread().unpark();
  }

  pub fn reset(&self) {
    self.sounds.clear()
  }
}

#[derive(Clone)]
struct Sounds {
  buffer: Arc<Mutex<Vec<i16>>>,
  index: Arc<AtomicUsize>,
  len: Arc<AtomicUsize>,
}
impl Sounds {
  fn new() -> Sounds {
    Sounds {
      buffer: Arc::new(Mutex::new(vec![])),
      index: Arc::new(ATOMIC_USIZE_INIT),
      len: Arc::new(ATOMIC_USIZE_INIT),
    }
  }

  fn append(&self, buf: &mut Vec<i16>) {
    let mut buffer = self.buffer.lock().unwrap();
    self.len.fetch_add(buf.len(), Ordering::SeqCst);
    buffer.append(buf);
  }

  fn clear(&self) {
    self.buffer.lock().unwrap().clear();
    self._clear(&mut self.buffer.lock().unwrap());
  }

  fn _clear(&self, buffer: &mut Vec<i16>) {
    buffer.clear();
    self.index.store(0, Ordering::SeqCst);
    self.len.store(0, Ordering::SeqCst);
  }

  fn peek(&self) -> i16 {
    loop {
      {
        let mut buffer = self.buffer.lock().unwrap();
        let len = self.len.load(Ordering::SeqCst);
        let index = self.index.load(Ordering::SeqCst);

        if index < len && len > 0 {
          self.index.fetch_add(1, Ordering::SeqCst);
          return buffer[index];
        } else if len > 0 {
          self._clear(&mut buffer);
        }
      }
      thread::park();
    }
  }
}

fn play(sounds: Sounds) {
  let device = cpal::default_output_device().unwrap();
  let format = cpal::Format {
    channels: 1,
    sample_rate: cpal::SampleRate(22050),
    data_type: cpal::SampleFormat::I16,
  };
  let event_loop = cpal::EventLoop::new();
  let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
  event_loop.play_stream(stream_id);

  event_loop.run(move |_, data| match data {
    StreamData::Output {
      buffer: UnknownTypeOutputBuffer::I16(mut buffer),
    } => {
      for elem in buffer.iter_mut() {
        *elem = sounds.peek();
      }
    }
    _ => unimplemented!(),
  });
}
