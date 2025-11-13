use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::path::PathBuf;

use super::decoder::AudioDecoder;
use super::resampler::LinearResampler;
use super::types::{AudioResult, AudioError, PlaybackState};

#[cfg(target_os = "macos")]
use super::macos_backend::MacOSAudioStream;

/// Simple audio player for looping drone pads
pub struct DronePlayer {
  // Audio buffer (pre-decoded and resampled)
  buffer: Arc<Mutex<Option<Vec<f32>>>>,
  channels: u16,
  sample_rate: u32,

  // Playback state (for MacOSAudioStream)
  playback_state: Arc<Mutex<PlaybackState>>,
  position: Arc<AtomicU64>, // Current position in samples (u64 for MacOSAudioStream)

  // Drone-specific state
  is_playing: Arc<AtomicBool>,
  volume: Arc<AtomicU32>,   // Volume as f32 bits

  // Audio backend
  #[cfg(target_os = "macos")]
  backend: Option<MacOSAudioStream>,

  // Device info
  current_device_name: Option<String>,
}

impl DronePlayer {
  pub fn new() -> AudioResult<Self> {
    Ok(Self {
      buffer: Arc::new(Mutex::new(None)),
      channels: 2,
      sample_rate: 44100,
      playback_state: Arc::new(Mutex::new(PlaybackState::Stopped)),
      position: Arc::new(AtomicU64::new(0)),
      is_playing: Arc::new(AtomicBool::new(false)),
      volume: Arc::new(AtomicU32::new(f32::to_bits(1.0))),
      backend: None,
      current_device_name: None,
    })
  }

  /// Load an audio file for playback
  pub fn load(&mut self, file_path: PathBuf) -> AudioResult<()> {
    log::info!("DronePad: Loading audio file: {:?}", file_path);

    // Decode the audio file
    let path_str = file_path.to_str()
      .ok_or_else(|| AudioError::FileError("Invalid file path".to_string()))?;

    let mut decoder = AudioDecoder::new(path_str)?;
    let metadata = decoder.get_metadata()?;

    log::info!(
      "DronePad: Decoding audio - channels: {}, sample_rate: {}",
      metadata.channels,
      metadata.sample_rate
    );

    let mut samples = decoder.decode_all()?;

    // Resample to 44100 if needed
    if metadata.sample_rate != 44100 {
      log::info!("DronePad: Resampling from {} to 44100", metadata.sample_rate);
      let mut resampler = LinearResampler::new(metadata.sample_rate, 44100, metadata.channels);
      samples = resampler.process(&samples);
    }

    log::info!("DronePad: Decoded {} samples", samples.len());

    // Store the buffer
    let mut buffer = self.buffer.lock().unwrap();
    *buffer = Some(samples);
    self.channels = metadata.channels;
    self.sample_rate = 44100;

    log::info!("DronePad: Audio loaded successfully");
    Ok(())
  }

  /// Start playback
  #[cfg(target_os = "macos")]
  pub fn play(&mut self, device_name: Option<String>) -> AudioResult<()> {
    // Check if we have audio loaded
    {
      let buffer = self.buffer.lock().unwrap();
      if buffer.is_none() {
        return Err(AudioError::PlaybackError("No audio loaded".to_string()));
      }
    }

    let device = device_name.as_deref().unwrap_or("default");
    log::info!("DronePad: Starting playback on device: {}", device);

    // Initialize audio backend if needed
    if self.backend.is_none() {
      // Create MacOSAudioStream
      let mut stream = MacOSAudioStream::new(
        device,
        Arc::clone(&self.playback_state),
        Arc::clone(&self.position),
      )?;

      // Set up render callback
      let buffer_clone = Arc::clone(&self.buffer);
      let is_playing_clone = Arc::clone(&self.is_playing);
      let position_clone = Arc::clone(&self.position);
      let volume_clone = Arc::clone(&self.volume);
      let channels = self.channels;

      stream.set_render_callback(move |output| {
        if !is_playing_clone.load(Ordering::Acquire) {
          // Not playing - output silence
          for sample in output.iter_mut() {
            *sample = 0.0;
          }
          return;
        }

        let buffer_lock = buffer_clone.lock().unwrap();
        if let Some(buffer) = buffer_lock.as_ref() {
          let buffer_len = buffer.len();
          let volume = f32::from_bits(volume_clone.load(Ordering::Acquire));

          for chunk in output.chunks_mut(channels as usize) {
            let pos = position_clone.load(Ordering::Acquire) as usize;

            // Copy samples from buffer
            for (i, sample) in chunk.iter_mut().enumerate() {
              let buffer_idx = pos + i;
              if buffer_idx < buffer_len {
                *sample = buffer[buffer_idx] * volume;
              } else {
                *sample = 0.0;
              }
            }

            // Advance position and loop
            let new_pos = ((pos + channels as usize) % buffer_len) as u64;
            position_clone.store(new_pos, Ordering::Release);
          }
        } else {
          // No buffer - output silence
          for sample in output.iter_mut() {
            *sample = 0.0;
          }
        }
      })?;

      // Initialize and start the stream
      stream.initialize()?;
      stream.start()?;

      self.backend = Some(stream);
      self.current_device_name = Some(device.to_string());
    }

    // Reset position and start playing
    self.position.store(0, Ordering::Release);
    self.is_playing.store(true, Ordering::Release);
    *self.playback_state.lock().unwrap() = PlaybackState::Playing;

    log::info!("DronePad: Playback started");
    Ok(())
  }

  /// Stop playback
  pub fn stop(&mut self) {
    self.is_playing.store(false, Ordering::Release);
    self.position.store(0, Ordering::Release);
    *self.playback_state.lock().unwrap() = PlaybackState::Stopped;
    log::info!("DronePad: Playback stopped");
  }

  /// Set volume (0.0 to 1.0)
  pub fn set_volume(&mut self, volume: f32) {
    let clamped = volume.clamp(0.0, 1.0);
    self.volume.store(f32::to_bits(clamped), Ordering::Release);
    log::info!("DronePad: Volume set to {}", clamped);
  }

  /// Check if currently playing
  pub fn is_playing(&self) -> bool {
    self.is_playing.load(Ordering::Acquire)
  }

  /// Switch to a different audio device
  #[cfg(target_os = "macos")]
  pub fn switch_device(&mut self, device_name: String) -> AudioResult<()> {
    log::info!("DronePad: Switching to device: {}", device_name);

    let was_playing = self.is_playing();
    let current_position = self.position.load(Ordering::Acquire);
    let current_volume = f32::from_bits(self.volume.load(Ordering::Acquire));

    // Stop and drop current backend
    self.stop();
    self.backend = None;

    // If we were playing, restart on new device
    if was_playing {
      self.play(Some(device_name))?;
      self.position.store(current_position, Ordering::Release);
      self.set_volume(current_volume);
    } else {
      self.current_device_name = Some(device_name);
    }

    Ok(())
  }

  /// Get current device name
  pub fn current_device_name(&self) -> Option<String> {
    self.current_device_name.clone()
  }
}

impl Drop for DronePlayer {
  fn drop(&mut self) {
    self.stop();
  }
}
