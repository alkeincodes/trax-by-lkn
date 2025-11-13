mod playback;
mod stems;
mod library;
mod setlists;
mod cache;
mod settings;

#[cfg(test)]
mod tests;

pub use playback::*;
pub use stems::*;
pub use library::*;
pub use setlists::*;
pub use cache::*;
pub use settings::*;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::audio::MultiTrackEngine;
use crate::database::Database;

// Cached song data - all stems pre-decoded and ready to play (in-memory only)
#[derive(Clone)]
pub struct CachedSong {
  pub song_id: String,
  pub stems: Vec<CachedStem>,
}

#[derive(Clone)]
pub struct CachedStem {
  pub stem_id: String,
  pub samples: Arc<Vec<f32>>, // Zero-copy sharing via Arc!
  pub sample_rate: u32, // Sample rate these samples were encoded at
  pub volume: f32,
  pub is_muted: bool,
}

// LRU Cache Entry with access tracking
#[derive(Clone)]
pub struct CacheEntry {
  pub song: CachedSong,
  pub last_accessed: u64, // Unix timestamp in seconds
  pub size_bytes: usize,  // Approximate size in bytes
}

// LRU Song Cache with size limit
pub struct SongCache {
  entries: HashMap<String, CacheEntry>,
  max_size_bytes: usize,
  current_size_bytes: usize,
}

impl SongCache {
  pub fn new(max_size_bytes: usize) -> Self {
    SongCache {
      entries: HashMap::new(),
      max_size_bytes,
      current_size_bytes: 0,
    }
  }

  pub fn get(&mut self, song_id: &str) -> Option<CachedSong> {
    if let Some(entry) = self.entries.get_mut(song_id) {
      // Update access time
      entry.last_accessed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
      Some(entry.song.clone())
    } else {
      None
    }
  }

  pub fn insert(&mut self, song_id: String, song: CachedSong) {
    // Calculate approximate size (samples * 4 bytes per f32)
    let size_bytes: usize = song.stems.iter()
      .map(|stem| stem.samples.len() * 4)
      .sum();

    // Evict entries if needed to make space
    while self.current_size_bytes + size_bytes > self.max_size_bytes && !self.entries.is_empty() {
      self.evict_lru();
    }

    let entry = CacheEntry {
      song: song.clone(),
      last_accessed: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs(),
      size_bytes,
    };

    // Remove old entry if exists (to update size)
    if let Some(old_entry) = self.entries.remove(&song_id) {
      self.current_size_bytes -= old_entry.size_bytes;
    }

    self.current_size_bytes += size_bytes;
    self.entries.insert(song_id, entry);

    log::info!(
      "Cache: {} songs, {:.1} MB / {:.1} MB",
      self.entries.len(),
      self.current_size_bytes as f64 / 1_048_576.0,
      self.max_size_bytes as f64 / 1_048_576.0
    );
  }

  pub fn contains(&self, song_id: &str) -> bool {
    self.entries.contains_key(song_id)
  }

  pub fn remove(&mut self, song_id: &str) {
    if let Some(entry) = self.entries.remove(song_id) {
      self.current_size_bytes -= entry.size_bytes;
      log::info!("Cache: Removed song {} ({:.1} MB)", song_id, entry.size_bytes as f64 / 1_048_576.0);
    }
  }

  pub fn clear(&mut self) {
    self.entries.clear();
    self.current_size_bytes = 0;
    log::info!("Cache: Cleared all songs");
  }

  fn evict_lru(&mut self) {
    // Find the least recently used entry
    if let Some(lru_id) = self.entries
      .iter()
      .min_by_key(|(_, entry)| entry.last_accessed)
      .map(|(id, _)| id.clone())
    {
      log::info!("Cache: Evicting LRU song {}", lru_id);
      self.remove(&lru_id);
    }
  }

  pub fn stats(&self) -> (usize, usize, usize) {
    // Returns (num_songs, current_bytes, max_bytes)
    (self.entries.len(), self.current_size_bytes, self.max_size_bytes)
  }

  pub fn set_max_size(&mut self, new_max_bytes: usize) {
    self.max_size_bytes = new_max_bytes;
    log::info!("Cache: Max size updated to {:.1} GB", new_max_bytes as f64 / 1_073_741_824.0);

    // Evict entries if current usage exceeds new limit
    while self.current_size_bytes > self.max_size_bytes && !self.entries.is_empty() {
      self.evict_lru();
    }
  }
}

// Shared application state for all Tauri commands
pub struct AppState {
  pub audio_engine: Arc<Mutex<MultiTrackEngine>>,
  pub database: Arc<Database>,
  pub stem_id_map: Arc<Mutex<HashMap<String, usize>>>,
  pub song_cache: Arc<Mutex<SongCache>>,
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
  pub fn new(
    database: Database,
    audio_engine: MultiTrackEngine,
  ) -> Self {
    // Default cache size: 3GB (allows ~5 songs with 20 stems each)
    const DEFAULT_CACHE_SIZE_BYTES: usize = 3 * 1024 * 1024 * 1024; // 3 GB

    AppState {
      audio_engine: Arc::new(Mutex::new(audio_engine)),
      database: Arc::new(database),
      stem_id_map: Arc::new(Mutex::new(HashMap::new())),
      song_cache: Arc::new(Mutex::new(SongCache::new(DEFAULT_CACHE_SIZE_BYTES))),
    }
  }
}
