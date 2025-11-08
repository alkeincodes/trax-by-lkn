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
use crate::audio::MultiTrackEngine;
use crate::database::Database;

// Shared application state for all Tauri commands
pub struct AppState {
  pub audio_engine: Arc<Mutex<MultiTrackEngine>>,
  pub database: Arc<Database>,
  pub stem_id_map: Arc<Mutex<std::collections::HashMap<String, usize>>>,
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
  pub fn new(database: Database, audio_engine: MultiTrackEngine) -> Self {
    AppState {
      audio_engine: Arc::new(Mutex::new(audio_engine)),
      database: Arc::new(database),
      stem_id_map: Arc::new(Mutex::new(std::collections::HashMap::new())),
    }
  }
}
