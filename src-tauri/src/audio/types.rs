use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
  Stopped,
  Playing,
  Paused,
}

#[derive(Debug, Clone)]
pub enum AudioCommand {
  Play(String),
  Pause,
  Stop,
  Seek(f64),
  SetVolume(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
  pub duration: f64,
  pub sample_rate: u32,
  pub channels: u16,
  pub format: String,
}

pub type AudioResult<T> = Result<T, AudioError>;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
  #[error("Failed to initialize audio device: {0}")]
  DeviceInit(String),

  #[error("Failed to decode audio file: {0}")]
  DecodeError(String),

  #[error("Failed to open audio file: {0}")]
  FileError(String),

  #[error("Audio playback error: {0}")]
  PlaybackError(String),

  #[error("Invalid audio format: {0}")]
  InvalidFormat(String),
}
