mod playback;
mod stems;
mod library;
mod setlists;

#[cfg(test)]
mod tests;

pub use playback::*;
pub use stems::*;
pub use library::*;
pub use setlists::*;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::audio::MultiTrackEngine;
use crate::cache::CacheManager;
use crate::database::Database;

// Cached song data - all stems pre-decoded and ready to play
#[derive(Clone)]
pub struct CachedSong {
  pub song_id: String,
  pub stems: Vec<CachedStem>,
}

#[derive(Clone)]
pub struct CachedStem {
  pub stem_id: String,
  pub samples: Vec<f32>,
  pub volume: f32,
  pub is_muted: bool,
}

// Shared application state for all Tauri commands
pub struct AppState {
  pub audio_engine: Arc<Mutex<MultiTrackEngine>>,
  pub database: Arc<Database>,
  pub cache_manager: Arc<CacheManager>,
  pub stem_id_map: Arc<Mutex<HashMap<String, usize>>>,
  pub song_cache: Arc<Mutex<HashMap<String, CachedSong>>>,
}

// SAFETY: AppState uses Arc<Mutex<>> for interior mutability which provides thread safety.
// The audio engine's Stream is only accessed from the audio callback thread once initialized,
// and all command operations go through the mutex lock. This is safe because:
// 1. All mutable state is protected by Mutex
// 2. The Stream is not directly accessed from multiple threads
// 3. All cross-thread communication uses thread-safe channels
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl AppState {
  pub fn new(database: Database, audio_engine: MultiTrackEngine, cache_manager: CacheManager) -> Self {
    AppState {
      audio_engine: Arc::new(Mutex::new(audio_engine)),
      database: Arc::new(database),
      cache_manager: Arc::new(cache_manager),
      stem_id_map: Arc::new(Mutex::new(HashMap::new())),
      song_cache: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}
