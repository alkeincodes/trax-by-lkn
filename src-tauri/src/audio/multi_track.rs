use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, SampleRate};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use super::buffer::AudioBuffer;
use super::decoder::AudioDecoder;
use super::resampler::LinearResampler;
use super::types::{AudioError, AudioResult, PlaybackState};

const TARGET_SAMPLE_RATE: u32 = 48000;
const BUFFER_SIZE: usize = 512;
const RING_BUFFER_SIZE: usize = 48000 * 2;
const MAX_STEMS: usize = 16;

pub struct MultiTrackEngine {
  max_stems: usize,
  stems: Arc<Mutex<Vec<Option<Stem>>>>,
  stem_volumes: Vec<Arc<std::sync::atomic::AtomicU32>>,
  stem_mutes: Vec<Arc<AtomicBool>>,
  stem_solos: Vec<Arc<AtomicBool>>,
  playback_state: Arc<Mutex<PlaybackState>>,
  position: Arc<AtomicU64>,
  stream: Option<Stream>,
  shutdown: Arc<AtomicBool>,
  decoder_threads: Vec<thread::JoinHandle<()>>,
}

struct Stem {
  id: usize,
  decoder: AudioDecoder,
  buffer: Arc<Mutex<AudioBuffer>>,
  resampler: Option<LinearResampler>,
  sample_rate: u32,
  channels: u16,
  duration: f64,
  shutdown: Arc<AtomicBool>,
}

impl MultiTrackEngine {
  pub fn new(max_stems: usize) -> AudioResult<Self> {
    if max_stems > MAX_STEMS {
      return Err(AudioError::DeviceInit(format!(
        "Maximum {} stems supported, requested {}",
        MAX_STEMS, max_stems
      )));
    }

    log::info!("Initializing multi-track engine with {} stems...", max_stems);

    let host = cpal::default_host();
    let device = host
      .default_output_device()
      .ok_or_else(|| AudioError::DeviceInit("No output device available".to_string()))?;

    log::info!("Using audio device: {:?}", device.name());

    let mut stems_vec = Vec::with_capacity(max_stems);
    let mut stem_volumes = Vec::with_capacity(max_stems);
    let mut stem_mutes = Vec::with_capacity(max_stems);
    let mut stem_solos = Vec::with_capacity(max_stems);

    for _ in 0..max_stems {
      stems_vec.push(None);
      stem_volumes.push(Arc::new(std::sync::atomic::AtomicU32::new(f32::to_bits(1.0))));
      stem_mutes.push(Arc::new(AtomicBool::new(false)));
      stem_solos.push(Arc::new(AtomicBool::new(false)));
    }

    let stems = Arc::new(Mutex::new(stems_vec));
    let playback_state = Arc::new(Mutex::new(PlaybackState::Stopped));
    let position = Arc::new(AtomicU64::new(0));
    let shutdown = Arc::new(AtomicBool::new(false));

    let mut engine = Self {
      max_stems,
      stems: stems.clone(),
      stem_volumes,
      stem_mutes,
      stem_solos,
      playback_state: playback_state.clone(),
      position: position.clone(),
      stream: None,
      shutdown: shutdown.clone(),
      decoder_threads: Vec::new(),
    };

    engine.initialize_stream(&device)?;

    log::info!("Multi-track engine initialized successfully");
    Ok(engine)
  }

  fn initialize_stream(&mut self, device: &Device) -> AudioResult<()> {
    let config = StreamConfig {
      channels: 2,
      sample_rate: SampleRate(TARGET_SAMPLE_RATE),
      buffer_size: cpal::BufferSize::Fixed(BUFFER_SIZE as u32),
    };

    let stems = self.stems.clone();
    let playback_state = self.playback_state.clone();
    let position = self.position.clone();
    let stem_volumes: Vec<_> = self.stem_volumes.iter().cloned().collect();
    let stem_mutes: Vec<_> = self.stem_mutes.iter().cloned().collect();
    let stem_solos: Vec<_> = self.stem_solos.iter().cloned().collect();

    let err_fn = |err| log::error!("Audio stream error: {}", err);

    let stream = device
      .build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
          Self::audio_callback(data, &stems, &playback_state, &position, &stem_volumes, &stem_mutes, &stem_solos);
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

  fn audio_callback(
    output: &mut [f32],
    stems: &Arc<Mutex<Vec<Option<Stem>>>>,
    playback_state: &Arc<Mutex<PlaybackState>>,
    position: &Arc<AtomicU64>,
    stem_volumes: &[Arc<std::sync::atomic::AtomicU32>],
    stem_mutes: &[Arc<AtomicBool>],
    stem_solos: &[Arc<AtomicBool>],
  ) {
    let state = playback_state.lock().unwrap();
    if *state != PlaybackState::Playing {
      output.fill(0.0);
      return;
    }
    drop(state);

    output.fill(0.0);

    let stems_guard = stems.lock().unwrap();

    let any_soloed = stem_solos
      .iter()
      .any(|s| s.load(Ordering::Acquire));

    let mut samples_read = 0;
    let mut temp_buffer = vec![0.0f32; output.len()];

    for (idx, stem_opt) in stems_guard.iter().enumerate() {
      if let Some(stem) = stem_opt {
        let is_muted = stem_mutes[idx].load(Ordering::Acquire);
        let is_soloed = stem_solos[idx].load(Ordering::Acquire);

        let should_play = if any_soloed {
          is_soloed
        } else {
          !is_muted
        };

        if !should_play {
          continue;
        }

        temp_buffer.fill(0.0);
        let read = stem.buffer.lock().unwrap().read(&mut temp_buffer);
        samples_read = samples_read.max(read);

        let volume_bits = stem_volumes[idx].load(Ordering::Acquire);
        let volume = f32::from_bits(volume_bits);

        for i in 0..output.len() {
          output[i] += temp_buffer[i] * volume;
        }
      }
    }

    drop(stems_guard);

    let new_position = position.load(Ordering::Acquire) + samples_read as u64;
    position.store(new_position, Ordering::Release);
  }

  pub fn max_stems(&self) -> usize {
    self.max_stems
  }

  pub fn active_stems(&self) -> usize {
    let stems = self.stems.lock().unwrap();
    stems.iter().filter(|s| s.is_some()).count()
  }

  pub fn stem_count(&self) -> usize {
    self.active_stems()
  }

  pub fn buffer_pool_capacity(&self) -> usize {
    self.max_stems
  }

  pub fn load_stem(&mut self, path: &str) -> AudioResult<usize> {
    let decoder = AudioDecoder::new(path)?;
    let metadata = decoder.get_metadata()?;

    let mut stems = self.stems.lock().unwrap();

    let stem_id = stems
      .iter()
      .position(|s| s.is_none())
      .ok_or_else(|| AudioError::PlaybackError("No available stem slots".to_string()))?;

    let buffer = Arc::new(Mutex::new(AudioBuffer::new(RING_BUFFER_SIZE)));
    buffer.lock().unwrap().set_ready(true);

    let resampler = if metadata.sample_rate != TARGET_SAMPLE_RATE {
      Some(LinearResampler::new(
        metadata.sample_rate,
        TARGET_SAMPLE_RATE,
        metadata.channels,
      ))
    } else {
      None
    };

    let shutdown = Arc::new(AtomicBool::new(false));

    let stem = Stem {
      id: stem_id,
      decoder,
      buffer: buffer.clone(),
      resampler,
      sample_rate: metadata.sample_rate,
      channels: metadata.channels,
      duration: metadata.duration,
      shutdown: shutdown.clone(),
    };

    stems[stem_id] = Some(stem);
    drop(stems);

    self.start_decoder_thread(stem_id);

    Ok(stem_id)
  }

  fn start_decoder_thread(&mut self, stem_id: usize) {
    let stems = self.stems.clone();
    let playback_state = self.playback_state.clone();
    let engine_shutdown = self.shutdown.clone();

    let handle = thread::spawn(move || {
      loop {
        let should_exit = engine_shutdown.load(Ordering::Acquire);
        if should_exit {
          break;
        }

        let should_decode = {
          let state = playback_state.lock().unwrap();
          *state == PlaybackState::Playing
        };

        if should_decode {
          let mut stems_guard = stems.lock().unwrap();
          if let Some(Some(stem)) = stems_guard.get_mut(stem_id) {
            let buffer_space = {
              let buffer = stem.buffer.lock().unwrap();
              buffer.available_samples() < RING_BUFFER_SIZE / 2
            };

            if buffer_space {
              if let Ok(Some(decoded)) = stem.decoder.decode_next_packet() {
                let samples = if let Some(resampler) = &mut stem.resampler {
                  resampler.process(&decoded.samples)
                } else {
                  decoded.samples
                };

                stem.buffer.lock().unwrap().write(&samples);
              }
            }
          }
          drop(stems_guard);
        }

        thread::sleep(std::time::Duration::from_millis(1));
      }
    });

    self.decoder_threads.push(handle);
  }

  pub fn clear_stems(&mut self) {
    self.shutdown.store(true, Ordering::Release);

    for handle in self.decoder_threads.drain(..) {
      let _ = handle.join();
    }

    self.shutdown.store(false, Ordering::Release);

    let mut stems = self.stems.lock().unwrap();
    for stem_slot in stems.iter_mut() {
      *stem_slot = None;
    }

    self.position.store(0, Ordering::Release);
  }

  pub fn set_stem_volume(&mut self, stem_id: usize, volume: f32) {
    if stem_id >= self.max_stems {
      return;
    }

    let clamped_volume = volume.clamp(0.0, 1.0);
    self.stem_volumes[stem_id].store(f32::to_bits(clamped_volume), Ordering::Release);
  }

  pub fn stem_volume(&self, stem_id: usize) -> f32 {
    if stem_id >= self.max_stems {
      return 0.0;
    }

    let bits = self.stem_volumes[stem_id].load(Ordering::Acquire);
    f32::from_bits(bits)
  }

  pub fn stem_volume_db(&self, stem_id: usize) -> f32 {
    let linear = self.stem_volume(stem_id);

    if linear <= 0.0 {
      f32::NEG_INFINITY
    } else {
      20.0 * linear.log10()
    }
  }

  pub fn set_stem_mute(&mut self, stem_id: usize, muted: bool) {
    if stem_id >= self.max_stems {
      return;
    }

    self.stem_mutes[stem_id].store(muted, Ordering::Release);
  }

  pub fn is_stem_muted(&self, stem_id: usize) -> bool {
    if stem_id >= self.max_stems {
      return false;
    }

    self.stem_mutes[stem_id].load(Ordering::Acquire)
  }

  pub fn set_stem_solo(&mut self, stem_id: usize, soloed: bool) {
    if stem_id >= self.max_stems {
      return;
    }

    self.stem_solos[stem_id].store(soloed, Ordering::Release);
  }

  pub fn is_stem_soloed(&self, stem_id: usize) -> bool {
    if stem_id >= self.max_stems {
      return false;
    }

    self.stem_solos[stem_id].load(Ordering::Acquire)
  }

  pub fn play(&mut self) -> AudioResult<()> {
    let mut state = self.playback_state.lock().unwrap();
    *state = PlaybackState::Playing;
    Ok(())
  }

  pub fn pause(&mut self) -> AudioResult<()> {
    let mut state = self.playback_state.lock().unwrap();
    *state = PlaybackState::Paused;
    Ok(())
  }

  pub fn stop(&mut self) -> AudioResult<()> {
    let mut state = self.playback_state.lock().unwrap();
    *state = PlaybackState::Stopped;
    self.position.store(0, Ordering::Release);
    Ok(())
  }

  pub fn position(&self) -> f64 {
    let sample_position = self.position.load(Ordering::Acquire);
    sample_position as f64 / (TARGET_SAMPLE_RATE as f64 * 2.0)
  }
}

impl Drop for MultiTrackEngine {
  fn drop(&mut self) {
    self.shutdown.store(true, Ordering::Release);

    for handle in self.decoder_threads.drain(..) {
      let _ = handle.join();
    }

    if let Some(stream) = self.stream.take() {
      drop(stream);
    }
  }
}
