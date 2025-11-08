use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError, TrySendError};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct AudioBuffer {
  samples: Vec<f32>,
  capacity: usize,
  write_pos: Arc<AtomicUsize>,
  read_pos: Arc<AtomicUsize>,
  ready: Arc<AtomicBool>,
}

impl AudioBuffer {
  pub fn new(capacity: usize) -> Self {
    Self {
      samples: vec![0.0; capacity],
      capacity,
      write_pos: Arc::new(AtomicUsize::new(0)),
      read_pos: Arc::new(AtomicUsize::new(0)),
      ready: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn write(&mut self, data: &[f32]) -> usize {
    if !self.ready.load(Ordering::Acquire) {
      return 0;
    }

    let write_idx = self.write_pos.load(Ordering::Acquire);
    let read_idx = self.read_pos.load(Ordering::Acquire);

    let available = if write_idx >= read_idx {
      self.capacity - (write_idx - read_idx) - 1
    } else {
      read_idx - write_idx - 1
    };

    let to_write = data.len().min(available);

    for i in 0..to_write {
      let idx = (write_idx + i) % self.capacity;
      self.samples[idx] = data[i];
    }

    self
      .write_pos
      .store((write_idx + to_write) % self.capacity, Ordering::Release);

    to_write
  }

  pub fn read(&self, output: &mut [f32]) -> usize {
    if !self.ready.load(Ordering::Acquire) {
      output.fill(0.0);
      return 0;
    }

    let read_idx = self.read_pos.load(Ordering::Acquire);
    let write_idx = self.write_pos.load(Ordering::Acquire);

    let available = if write_idx >= read_idx {
      write_idx - read_idx
    } else {
      self.capacity - read_idx + write_idx
    };

    let to_read = output.len().min(available);

    for i in 0..to_read {
      let idx = (read_idx + i) % self.capacity;
      output[i] = self.samples[idx];
    }

    for i in to_read..output.len() {
      output[i] = 0.0;
    }

    self
      .read_pos
      .store((read_idx + to_read) % self.capacity, Ordering::Release);

    to_read
  }

  pub fn available_samples(&self) -> usize {
    let read_idx = self.read_pos.load(Ordering::Acquire);
    let write_idx = self.write_pos.load(Ordering::Acquire);

    if write_idx >= read_idx {
      write_idx - read_idx
    } else {
      self.capacity - read_idx + write_idx
    }
  }

  pub fn reset(&mut self) {
    self.write_pos.store(0, Ordering::Release);
    self.read_pos.store(0, Ordering::Release);
    self.samples.fill(0.0);
  }

  pub fn set_ready(&self, ready: bool) {
    self.ready.store(ready, Ordering::Release);
  }

  pub fn is_ready(&self) -> bool {
    self.ready.load(Ordering::Acquire)
  }
}

pub struct AudioCommandChannel<T> {
  sender: Sender<T>,
  receiver: Receiver<T>,
}

impl<T> AudioCommandChannel<T> {
  pub fn new(capacity: usize) -> Self {
    let (sender, receiver) = bounded(capacity);
    Self { sender, receiver }
  }

  pub fn sender(&self) -> Sender<T> {
    self.sender.clone()
  }

  pub fn receiver(&self) -> Receiver<T> {
    self.receiver.clone()
  }

  pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
    self.sender.try_send(msg)
  }

  pub fn try_recv(&self) -> Result<T, TryRecvError> {
    self.receiver.try_recv()
  }
}
