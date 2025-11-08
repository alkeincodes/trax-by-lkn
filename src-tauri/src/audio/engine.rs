use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, SampleRate};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use super::buffer::AudioBuffer;
use super::decoder::AudioDecoder;
use super::resampler::LinearResampler;
use super::types::{AudioCommand, AudioError, AudioMetadata, AudioResult, PlaybackState};

const TARGET_SAMPLE_RATE: u32 = 48000;
const BUFFER_SIZE: usize = 512;
const RING_BUFFER_SIZE: usize = 48000 * 2;
const CROSSFADE_MS: f64 = 25.0;

pub struct AudioEngine {
  state: Arc<Mutex<EngineState>>,
  stream: Option<Stream>,
  command_tx: Sender<AudioCommand>,
  decoder_thread: Option<thread::JoinHandle<()>>,
  shutdown: Arc<AtomicBool>,
}

struct EngineState {
  playback_state: PlaybackState,
  volume: f32,
  position: Arc<AtomicU64>,
  duration: f64,
  sample_rate: u32,
  channels: u16,
  buffer: Arc<Mutex<AudioBuffer>>,
  decoder: Option<AudioDecoder>,
  resampler: Option<LinearResampler>,
  crossfade_samples: usize,
  fade_position: usize,
  fading_in: bool,
  fading_out: bool,
}

impl AudioEngine {
  pub fn new() -> AudioResult<Self> {
    log::info!("Initializing audio engine...");
    let host = cpal::default_host();
    let device = host
      .default_output_device()
      .ok_or_else(|| AudioError::DeviceInit("No output device available".to_string()))?;

    log::info!("Using audio device: {:?}", device.name());

    let (command_tx, command_rx) = unbounded();

    let state = Arc::new(Mutex::new(EngineState {
      playback_state: PlaybackState::Stopped,
      volume: 1.0,
      position: Arc::new(AtomicU64::new(0)),
      duration: 0.0,
      sample_rate: TARGET_SAMPLE_RATE,
      channels: 2,
      buffer: Arc::new(Mutex::new(AudioBuffer::new(RING_BUFFER_SIZE))),
      decoder: None,
      resampler: None,
      crossfade_samples: ((CROSSFADE_MS / 1000.0) * TARGET_SAMPLE_RATE as f64) as usize,
      fade_position: 0,
      fading_in: false,
      fading_out: false,
    }));

    let shutdown = Arc::new(AtomicBool::new(false));

    let mut engine = Self {
      state: state.clone(),
      stream: None,
      command_tx,
      decoder_thread: None,
      shutdown: shutdown.clone(),
    };

    engine.initialize_stream(&device)?;
    engine.start_decoder_thread(command_rx);

    log::info!("Audio engine initialized successfully");
    Ok(engine)
  }

  fn initialize_stream(&mut self, device: &Device) -> AudioResult<()> {
    let config = StreamConfig {
      channels: 2,
      sample_rate: SampleRate(TARGET_SAMPLE_RATE),
      buffer_size: cpal::BufferSize::Fixed(BUFFER_SIZE as u32),
    };

    let state = self.state.clone();
    let err_fn = |err| log::error!("Audio stream error: {}", err);

    let stream = device
      .build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
          let mut engine_state = state.lock().unwrap();
          engine_state.audio_callback(data);
        },
        err_fn,
        None,
      )
      .map_err(|e| AudioError::DeviceInit(format!("Failed to build stream: {}", e)))?;

    stream
      .play()
      .map_err(|e| AudioError::PlaybackError(format!("Failed to start stream: {}", e)))?;

    self.stream = Some(stream);

    Ok(())
  }

  fn start_decoder_thread(&mut self, command_rx: Receiver<AudioCommand>) {
    let state = self.state.clone();
    let shutdown = self.shutdown.clone();

    let handle = thread::spawn(move || {
      while !shutdown.load(Ordering::Acquire) {
        if let Ok(command) = command_rx.try_recv() {
          let mut engine_state = state.lock().unwrap();
          match command {
            AudioCommand::Play(path) => {
              if let Err(e) = engine_state.handle_play(&path) {
                log::error!("Failed to play file: {}", e);
              }
            }
            AudioCommand::Pause => {
              engine_state.handle_pause();
            }
            AudioCommand::Stop => {
              engine_state.handle_stop();
            }
            AudioCommand::Seek(position) => {
              if let Err(e) = engine_state.handle_seek(position) {
                log::error!("Failed to seek: {}", e);
              }
            }
            AudioCommand::SetVolume(volume) => {
              engine_state.volume = volume.clamp(0.0, 1.0);
            }
          }
        }

        let should_decode = {
          let engine_state = state.lock().unwrap();
          engine_state.playback_state == PlaybackState::Playing
            && engine_state.buffer.lock().unwrap().available_samples() < RING_BUFFER_SIZE / 2
        };

        if should_decode {
          let mut engine_state = state.lock().unwrap();
          if let Err(e) = engine_state.decode_and_buffer() {
            log::error!("Decode error: {}", e);
          }
        }

        thread::sleep(std::time::Duration::from_millis(1));
      }
    });

    self.decoder_thread = Some(handle);
  }

  pub fn state(&self) -> PlaybackState {
    self.state.lock().unwrap().playback_state
  }

  pub fn volume(&self) -> f32 {
    self.state.lock().unwrap().volume
  }

  pub fn set_volume(&mut self, volume: f32) {
    let clamped_volume = volume.clamp(0.0, 1.0);
    self.state.lock().unwrap().volume = clamped_volume;
  }

  pub fn position(&self) -> f64 {
    let state = self.state.lock().unwrap();
    let sample_position = state.position.load(Ordering::Acquire);
    sample_position as f64 / (state.sample_rate as f64 * state.channels as f64)
  }

  pub fn duration(&self) -> f64 {
    self.state.lock().unwrap().duration
  }

  pub fn play(&mut self) -> AudioResult<()> {
    let current_state = self.state();
    if current_state == PlaybackState::Stopped && self.duration() == 0.0 {
      return Err(AudioError::PlaybackError("No file loaded".to_string()));
    }

    let mut state = self.state.lock().unwrap();
    state.playback_state = PlaybackState::Playing;
    state.fading_in = true;
    state.fade_position = 0;
    Ok(())
  }

  pub fn pause(&mut self) -> AudioResult<()> {
    let _ = self.command_tx.try_send(AudioCommand::Pause);
    Ok(())
  }

  pub fn stop(&mut self) -> AudioResult<()> {
    let _ = self.command_tx.try_send(AudioCommand::Stop);
    Ok(())
  }

  pub fn seek(&mut self, position: f64) -> AudioResult<()> {
    if self.duration() == 0.0 {
      return Err(AudioError::PlaybackError("No file loaded".to_string()));
    }

    let clamped_position = position.clamp(0.0, self.duration());
    let _ = self.command_tx.try_send(AudioCommand::Seek(clamped_position));
    Ok(())
  }

  pub fn load_file(&mut self, path: &str) -> AudioResult<AudioMetadata> {
    let decoder = AudioDecoder::new(path)?;
    let metadata = decoder.get_metadata()?;

    let mut state = self.state.lock().unwrap();
    state.duration = metadata.duration;
    state.sample_rate = metadata.sample_rate;
    state.channels = metadata.channels;
    state.position.store(0, Ordering::Release);

    if metadata.sample_rate != TARGET_SAMPLE_RATE {
      state.resampler = Some(LinearResampler::new(
        metadata.sample_rate,
        TARGET_SAMPLE_RATE,
        metadata.channels,
      ));
    } else {
      state.resampler = None;
    }

    state.decoder = Some(decoder);
    state.buffer.lock().unwrap().reset();
    state.buffer.lock().unwrap().set_ready(true);

    Ok(metadata)
  }
}

impl EngineState {
  fn audio_callback(&mut self, output: &mut [f32]) {
    if self.playback_state != PlaybackState::Playing {
      output.fill(0.0);
      return;
    }

    let samples_read = self.buffer.lock().unwrap().read(output);

    for i in 0..output.len() {
      output[i] *= self.volume;

      if self.fading_in && self.fade_position < self.crossfade_samples {
        let fade_gain = self.fade_position as f32 / self.crossfade_samples as f32;
        output[i] *= fade_gain;
        self.fade_position += 1;
      } else if self.fading_in {
        self.fading_in = false;
        self.fade_position = 0;
      }

      if self.fading_out && self.fade_position < self.crossfade_samples {
        let fade_gain = 1.0 - (self.fade_position as f32 / self.crossfade_samples as f32);
        output[i] *= fade_gain;
        self.fade_position += 1;
      } else if self.fading_out && self.fade_position >= self.crossfade_samples {
        self.playback_state = PlaybackState::Stopped;
        self.fading_out = false;
        self.fade_position = 0;
        output.fill(0.0);
        return;
      }
    }

    let new_position = self.position.load(Ordering::Acquire) + samples_read as u64;
    self.position.store(new_position, Ordering::Release);
  }

  fn decode_and_buffer(&mut self) -> AudioResult<()> {
    if let Some(decoder) = &mut self.decoder {
      if let Some(decoded) = decoder.decode_next_packet()? {
        let samples = if let Some(resampler) = &mut self.resampler {
          resampler.process(&decoded.samples)
        } else {
          decoded.samples
        };

        let mut buffer = self.buffer.lock().unwrap();
        buffer.write(&samples);
      } else {
        self.playback_state = PlaybackState::Stopped;
        self.position.store(0, Ordering::Release);
      }
    }
    Ok(())
  }

  fn handle_play(&mut self, path: &str) -> AudioResult<()> {
    let decoder = AudioDecoder::new(path)?;
    let metadata = decoder.get_metadata()?;

    self.duration = metadata.duration;
    self.sample_rate = metadata.sample_rate;
    self.channels = metadata.channels;
    self.position.store(0, Ordering::Release);

    if metadata.sample_rate != TARGET_SAMPLE_RATE {
      self.resampler = Some(LinearResampler::new(
        metadata.sample_rate,
        TARGET_SAMPLE_RATE,
        metadata.channels,
      ));
    } else {
      self.resampler = None;
    }

    self.decoder = Some(decoder);
    self.buffer.lock().unwrap().reset();
    self.buffer.lock().unwrap().set_ready(true);
    self.playback_state = PlaybackState::Playing;
    self.fading_in = true;
    self.fade_position = 0;

    Ok(())
  }

  fn handle_pause(&mut self) {
    if self.playback_state == PlaybackState::Playing {
      self.fading_out = true;
      self.fade_position = 0;
      self.playback_state = PlaybackState::Paused;
    }
  }

  fn handle_stop(&mut self) {
    self.fading_out = true;
    self.fade_position = 0;
    self.playback_state = PlaybackState::Stopped;
    self.position.store(0, Ordering::Release);
    self.buffer.lock().unwrap().reset();
  }

  fn handle_seek(&mut self, position: f64) -> AudioResult<()> {
    if let Some(decoder) = &mut self.decoder {
      decoder.seek(position)?;
      self.buffer.lock().unwrap().reset();
      let sample_pos = (position * self.sample_rate as f64 * self.channels as f64) as u64;
      self.position.store(sample_pos, Ordering::Release);
    }
    Ok(())
  }
}

impl Drop for AudioEngine {
  fn drop(&mut self) {
    self.shutdown.store(true, Ordering::Release);

    if let Some(handle) = self.decoder_thread.take() {
      let _ = handle.join();
    }

    if let Some(stream) = self.stream.take() {
      drop(stream);
    }
  }
}
