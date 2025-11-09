use super::database::CacheDatabase;
use super::hash::calculate_file_hash;
use super::types::{CachedAudio, CacheError, CacheResult, CacheSettings, CacheStats};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct CacheManager {
  db: Arc<Mutex<CacheDatabase>>,
  settings: CacheSettings,
  audio_dir: PathBuf,
}

impl CacheManager {
  /// Create a new cache manager
  pub fn new(settings: CacheSettings) -> CacheResult<Self> {
    // The cache_location already points to the audio_cache directory
    // No need for subdirectories - just create the main cache directory
    let audio_dir = settings.cache_location.clone();
    log::info!("Creating cache directory at: {:?}", audio_dir);
    fs::create_dir_all(&audio_dir)?;
    log::info!("Cache directory created successfully");

    // Create database in parent directory (.cache/trax/)
    let cache_root = audio_dir.parent()
      .ok_or_else(|| CacheError::IoError(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Cache location has no parent directory"
      )))?;
    let db_path = cache_root.join("metadata.db");
    let db = CacheDatabase::new(&db_path)?;

    Ok(Self {
      db: Arc::new(Mutex::new(db)),
      settings,
      audio_dir,
    })
  }

  /// Check if cache is enabled
  pub fn is_enabled(&self) -> bool {
    self.settings.enabled
  }

  /// Get a cached audio file path if it exists and is valid
  /// Returns the path to the cached file (still compressed, ready to decode)
  pub fn get(&self, song_id: &str, stem_id: &str, source_path: &Path) -> CacheResult<Option<PathBuf>> {
    if !self.settings.enabled {
      return Err(CacheError::Disabled);
    }

    log::info!("Cache lookup: {}/{}", song_id, stem_id);

    let db = self.db.lock().map_err(|_| {
      CacheError::DatabaseError(rusqlite::Error::InvalidQuery)
    })?;

    // Try to get from database
    let entry = match db.get(song_id, stem_id)? {
      Some(entry) => entry,
      None => {
        log::info!("Cache MISS (not in database): {}/{}", song_id, stem_id);
        db.increment_misses()?;
        return Ok(None);
      }
    };

    log::info!("Found cache entry, validating...");

    // Validate the cache entry
    if !self.validate_entry(&entry, source_path)? {
      // Invalid, remove it
      log::warn!("Cache entry invalid, removing: {}/{}", song_id, stem_id);
      db.remove(song_id, stem_id)?;
      db.increment_misses()?;
      return Ok(None);
    }

    log::info!("Cache entry valid, returning cached file path");

    // Return cached file path (caller will decode it)
    db.touch(song_id, stem_id)?;
    db.increment_hits()?;
    log::info!("Cache HIT: {}/{} -> {:?}", song_id, stem_id, entry.cache_path);
    Ok(Some(entry.cache_path))
  }

  /// Store original audio file in cache (copy as-is, no conversion)
  pub fn put(
    &self,
    song_id: &str,
    stem_id: &str,
    source_path: &Path,
    duration_seconds: f64,
  ) -> CacheResult<()> {
    if !self.settings.enabled {
      return Err(CacheError::Disabled);
    }

    // Get source file size
    let source_size = fs::metadata(source_path)?.len();

    // Check size limits before copying
    self.check_size_limits(source_size as usize)?;

    // Calculate source file hash
    let source_hash = calculate_file_hash(source_path)?;

    // Get file extension from source
    let extension = source_path
      .extension()
      .and_then(|e| e.to_str())
      .unwrap_or("audio");

    // Generate cache file path (preserve original extension)
    let cache_filename = format!("{}_{}.{}", song_id, stem_id, extension);
    let cache_path = self.audio_dir.join(&cache_filename);

    // Copy original file to cache (no conversion)
    fs::copy(source_path, &cache_path)?;
    log::info!("Cached original file: {} -> {:?}", source_path.display(), cache_path);

    // Get file size
    let file_size_bytes = fs::metadata(&cache_path)?.len();

    // Store metadata in database (sample_rate and channels not needed anymore)
    let now = chrono::Utc::now().timestamp();
    let entry = CachedAudio {
      song_id: song_id.to_string(),
      stem_id: stem_id.to_string(),
      source_path: source_path.to_path_buf(),
      source_hash,
      cache_path,
      sample_rate: 0, // Not applicable - will decode on demand
      channels: 0,    // Not applicable - will decode on demand
      duration_seconds,
      decoded_at: now,
      last_accessed: now,
      file_size_bytes,
    };

    let db = self.db.lock().map_err(|_| {
      CacheError::DatabaseError(rusqlite::Error::InvalidQuery)
    })?;
    db.upsert(&entry)?;

    log::info!("Cached: {}/{} ({:.2} MB)", song_id, stem_id, file_size_bytes as f64 / 1_000_000.0);
    Ok(())
  }

  /// Validate a cache entry
  fn validate_entry(&self, entry: &CachedAudio, source_path: &Path) -> CacheResult<bool> {
    // Check if cached file exists
    if !entry.cache_path.exists() {
      log::warn!("Cache file missing: {:?}", entry.cache_path);
      return Ok(false);
    }

    // Check if source file still exists
    if !source_path.exists() {
      log::warn!("Source file missing: {:?}", source_path);
      return Ok(false);
    }

    // PERFORMANCE: Skip hash validation for now - it's too slow
    // Hash validation requires reading the entire source file which defeats the purpose of caching
    // TODO: Only validate hash if file modification time has changed

    // Verify source file hasn't changed
    // let current_hash = calculate_file_hash(source_path)?;
    // if current_hash != entry.source_hash {
    //   log::warn!("Source file changed (hash mismatch): {:?}", source_path);
    //   return Ok(false);
    // }

    Ok(true)
  }


  /// Check if adding new data would exceed size limits
  fn check_size_limits(&self, new_bytes: usize) -> CacheResult<()> {
    let db = self.db.lock().map_err(|_| {
      CacheError::DatabaseError(rusqlite::Error::InvalidQuery)
    })?;

    let current_size = db.get_total_size()?;
    let max_size = self.settings.max_size_gb * 1_000_000_000; // Convert GB to bytes

    let new_size = current_size + new_bytes as u64;

    if new_size > max_size {
      // Need to evict some entries
      let bytes_to_free = new_size - max_size + (max_size / 10); // Free 10% extra
      self.evict_lru(bytes_to_free)?;
    }

    Ok(())
  }

  /// Evict least recently used entries until enough space is freed
  fn evict_lru(&self, bytes_to_free: u64) -> CacheResult<()> {
    let db = self.db.lock().map_err(|_| {
      CacheError::DatabaseError(rusqlite::Error::InvalidQuery)
    })?;

    let mut freed_bytes = 0u64;
    let mut entries_to_check = 100; // Check 100 entries at a time

    while freed_bytes < bytes_to_free {
      let entries = db.get_lru_entries(entries_to_check)?;
      if entries.is_empty() {
        break; // No more entries to evict
      }

      for entry in entries {
        // Delete the cached file
        if entry.cache_path.exists() {
          if let Err(e) = fs::remove_file(&entry.cache_path) {
            log::warn!("Failed to delete cached file: {}", e);
          } else {
            freed_bytes += entry.file_size_bytes;
            log::info!("Evicted: {}/{}", entry.song_id, entry.stem_id);
          }
        }

        // Remove from database
        db.remove(&entry.song_id, &entry.stem_id)?;
        db.increment_evictions()?;

        if freed_bytes >= bytes_to_free {
          break;
        }
      }
    }

    log::info!("Freed {:.2} MB from cache", freed_bytes as f64 / 1_000_000.0);
    Ok(())
  }

  /// Get cache statistics
  pub fn get_stats(&self) -> CacheResult<CacheStats> {
    let db = self.db.lock().map_err(|_| {
      CacheError::DatabaseError(rusqlite::Error::InvalidQuery)
    })?;
    db.get_stats()
  }

  /// Clear all cached files and database entries
  pub fn clear(&self) -> CacheResult<()> {
    let db = self.db.lock().map_err(|_| {
      CacheError::DatabaseError(rusqlite::Error::InvalidQuery)
    })?;

    // Delete all WAV files in audio directory
    if self.audio_dir.exists() {
      for entry in fs::read_dir(&self.audio_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("wav") {
          fs::remove_file(path)?;
        }
      }
    }

    // Clear database
    db.clear()?;

    log::info!("Cache cleared");
    Ok(())
  }

  /// Remove a specific song's cache entries
  pub fn remove_song(&self, song_id: &str) -> CacheResult<()> {
    let db = self.db.lock().map_err(|_| {
      CacheError::DatabaseError(rusqlite::Error::InvalidQuery)
    })?;

    let entries = db.get_song_entries(song_id)?;

    for entry in entries {
      // Delete cached file
      if entry.cache_path.exists() {
        fs::remove_file(&entry.cache_path)?;
      }

      // Remove from database
      db.remove(&entry.song_id, &entry.stem_id)?;
    }

    log::info!("Removed song from cache: {}", song_id);
    Ok(())
  }
}
