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

/// Preset configurations for maximum stem count
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemCapacity {
  /// Standard configuration - suitable for most backing tracks (16 stems)
  Standard,
  /// Extended configuration - for complex arrangements (32 stems)
  Extended,
  /// Professional configuration - for orchestral/large productions (64 stems)
  Professional,
  /// Custom configuration - user-defined stem count
  Custom(usize),
}

impl StemCapacity {
  pub fn as_usize(&self) -> usize {
    match self {
      StemCapacity::Standard => 16,
      StemCapacity::Extended => 32,
      StemCapacity::Professional => 64,
      StemCapacity::Custom(n) => *n,
    }
  }

  pub fn from_usize(count: usize) -> Self {
    match count {
      16 => StemCapacity::Standard,
      32 => StemCapacity::Extended,
      64 => StemCapacity::Professional,
      n => StemCapacity::Custom(n),
    }
  }
}

pub struct MultiTrackEngine {
  max_stems: usize,
  stems: Arc<Mutex<Vec<Option<Stem>>>>,
  stem_volumes: Vec<Arc<std::sync::atomic::AtomicU32>>,
  stem_mutes: Vec<Arc<AtomicBool>>,
  stem_solos: Vec<Arc<AtomicBool>>,
  playback_state: Arc<Mutex<PlaybackState>>,
  position: Arc<AtomicU64>,
  stream: Option<Stream>,
}

struct Stem {
  id: usize,
  // Pre-decoded audio samples (shared via Arc - no copying!)
  samples: Arc<Vec<f32>>,
  sample_rate: u32,
  channels: u16,
  duration: f64,
}

impl MultiTrackEngine {
  /// Create a new multi-track engine with the specified capacity preset
  pub fn with_capacity(capacity: StemCapacity) -> AudioResult<Self> {
    Self::new(capacity.as_usize())
  }

  /// Create a new multi-track engine with standard capacity (16 stems)
  pub fn new_standard() -> AudioResult<Self> {
    Self::with_capacity(StemCapacity::Standard)
  }

  /// Create a new multi-track engine with extended capacity (32 stems)
  pub fn new_extended() -> AudioResult<Self> {
    Self::with_capacity(StemCapacity::Extended)
  }

  /// Create a new multi-track engine with professional capacity (64 stems)
  pub fn new_professional() -> AudioResult<Self> {
    Self::with_capacity(StemCapacity::Professional)
  }

  /// Create a new multi-track engine with a custom stem count
  pub fn new(max_stems: usize) -> AudioResult<Self> {
    // Validate reasonable limits (prevent excessive memory allocation)
    if max_stems == 0 {
      return Err(AudioError::DeviceInit(
        "Maximum stems must be at least 1".to_string()
      ));
    }
    if max_stems > 256 {
      return Err(AudioError::DeviceInit(format!(
        "Maximum 256 stems supported for stability, requested {}",
        max_stems
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

    let mut engine = Self {
      max_stems,
      stems: stems.clone(),
      stem_volumes,
      stem_mutes,
      stem_solos,
      playback_state: playback_state.clone(),
      position: position.clone(),
      stream: None,
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

    let current_position = position.load(Ordering::Acquire) as usize;

    for (idx, stem_opt) in stems_guard.iter().enumerate() {
      if let Some(stem) = stem_opt {
        let is_muted = stem_mutes[idx].load(Ordering::Acquire);
        let is_soloed = stem_solos[idx].load(Ordering::Acquire);

        let should_output = if any_soloed {
          is_soloed
        } else {
          !is_muted
        };

        if should_output {
          let volume_bits = stem_volumes[idx].load(Ordering::Acquire);
          let volume = f32::from_bits(volume_bits);

          // Read directly from pre-decoded samples
          let samples_to_copy = output.len().min(stem.samples.len().saturating_sub(current_position));

          for i in 0..samples_to_copy {
            output[i] += stem.samples[current_position + i] * volume;
          }
        }
      }
    }

    drop(stems_guard);

    // Advance position by the number of samples we output
    let new_position = current_position + output.len();
    position.store(new_position as u64, Ordering::Release);
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
    log::info!("Loading stem from: {}", path);

    let mut decoder = AudioDecoder::new(path)?;
    let metadata = decoder.get_metadata()?;

    log::info!("Decoding entire audio file...");
    let mut decoded_samples = decoder.decode_all()?;

    // Resample if necessary
    if metadata.sample_rate != TARGET_SAMPLE_RATE {
      log::info!("Resampling from {}Hz to {}Hz", metadata.sample_rate, TARGET_SAMPLE_RATE);
      let mut resampler = LinearResampler::new(
        metadata.sample_rate,
        TARGET_SAMPLE_RATE,
        metadata.channels,
      );
      decoded_samples = resampler.process(&decoded_samples);
    }

    // Wrap in Arc for zero-copy loading
    self.load_stem_from_samples(Arc::new(decoded_samples))
  }

  /// Load pre-decoded samples directly into the engine (from cache)
  pub fn load_stem_from_samples(&mut self, samples: Arc<Vec<f32>>) -> AudioResult<usize> {
    let mut stems = self.stems.lock().unwrap();

    let stem_id = stems
      .iter()
      .position(|s| s.is_none())
      .ok_or_else(|| AudioError::PlaybackError("No available stem slots".to_string()))?;

    let duration = samples.len() as f64 / (TARGET_SAMPLE_RATE as f64 * 2.0);

    let stem = Stem {
      id: stem_id,
      samples, // No copying - just share the Arc!
      sample_rate: TARGET_SAMPLE_RATE,
      channels: 2, // Assuming stereo
      duration,
    };

    stems[stem_id] = Some(stem);
    drop(stems);

    log::info!("Successfully loaded stem from samples at index {} (zero-copy)", stem_id);

    Ok(stem_id)
  }


  pub fn clear_stems(&mut self) {
    // Clear all stem slots
    let mut stems = self.stems.lock().unwrap();
    for stem_slot in stems.iter_mut() {
      *stem_slot = None;
    }
    drop(stems);

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

  pub fn seek(&mut self, position_seconds: f64) -> AudioResult<()> {
    // Convert seconds to sample position (stereo, so multiply by 2)
    let sample_position = (position_seconds * TARGET_SAMPLE_RATE as f64 * 2.0) as u64;

    // Update the position - no need to clear buffers since we read directly from pre-decoded samples
    self.position.store(sample_position, Ordering::Release);

    log::info!("Seeked to position: {} seconds ({} samples)", position_seconds, sample_position);

    Ok(())
  }

  pub fn position(&self) -> f64 {
    let sample_position = self.position.load(Ordering::Acquire);
    sample_position as f64 / (TARGET_SAMPLE_RATE as f64 * 2.0)
  }

  pub fn state(&self) -> PlaybackState {
    *self.playback_state.lock().unwrap()
  }

  /// Get a clone of the position Arc for cross-thread access
  pub fn position_arc(&self) -> Arc<AtomicU64> {
    self.position.clone()
  }

  /// Get a clone of the playback state Arc for cross-thread access
  pub fn playback_state_arc(&self) -> Arc<Mutex<PlaybackState>> {
    self.playback_state.clone()
  }
}

impl Drop for MultiTrackEngine {
  fn drop(&mut self) {
    if let Some(stream) = self.stream.take() {
      drop(stream);
    }
  }
}
