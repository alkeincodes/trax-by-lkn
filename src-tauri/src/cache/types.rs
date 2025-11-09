use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Represents a cached audio file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAudio {
  pub song_id: String,
  pub stem_id: String,
  pub source_path: PathBuf,
  pub source_hash: String,
  pub cache_path: PathBuf,
  pub sample_rate: u32,
  pub channels: u16,
  pub duration_seconds: f64,
  pub decoded_at: i64,
  pub last_accessed: i64,
  pub file_size_bytes: u64,
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
  pub total_entries: usize,
  pub total_size_bytes: u64,
  pub cache_hits: u64,
  pub cache_misses: u64,
  pub evictions: u64,
}

/// Cache settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
  pub enabled: bool,
  pub max_size_gb: u64,
  pub cache_location: PathBuf,
}

impl Default for CacheSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      max_size_gb: 10,
      cache_location: get_default_cache_path(),
    }
  }
}

/// Get the default cache directory path
pub fn get_default_cache_path() -> PathBuf {
  let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
  let path = base.join("trax").join("audio_cache");
  log::info!("Default cache path: {:?}", path);
  path
}

/// Error types for cache operations
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
  #[error("Database error: {0}")]
  DatabaseError(#[from] rusqlite::Error),

  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),

  #[error("WAV error: {0}")]
  WavError(#[from] hound::Error),

  #[error("Cache entry not found: {0}")]
  NotFound(String),

  #[error("Cache validation failed: {0}")]
  ValidationFailed(String),

  #[error("Cache is disabled")]
  Disabled,

  #[error("Cache size limit exceeded")]
  SizeLimitExceeded,
}

pub type CacheResult<T> = Result<T, CacheError>;
