use super::AppState;
use tauri::{State, Emitter};
use std::path::Path;

/// Preload a song's stems into cache (decode and store in memory)
#[tauri::command]
pub async fn load_song(song_id: String, state: State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
  log::info!("Loading song stems: {}", song_id);

  // Check if already in memory cache
  {
    let cache = state.song_cache.lock().map_err(|_| "Failed to lock cache")?;
    if cache.contains_key(&song_id) {
      log::info!("Song {} already in memory, skipping load", song_id);
      return Ok(());
    }
  }

  // Get song from database
  let song = state.database
    .get_song(&song_id)
    .map_err(|e| format!("Failed to get song from database: {}", e))?;

  // Get all stems for this song
  let stems = state.database
    .get_stems_for_song(&song_id)
    .map_err(|e| format!("Failed to get stems for song: {}", e))?;

  if stems.is_empty() {
    return Err("Song has no stems".to_string());
  }

  let total_stems = stems.len();

  // Decode all stems and cache them
  let mut cached_stems = Vec::new();

  for (index, stem) in stems.iter().enumerate() {
    let current_stem = index + 1;
    log::info!("Loading stem {}/{}: {}", current_stem, total_stems, stem.name);

    // Emit progress event to frontend
    let _ = app_handle.emit("stem:loading", serde_json::json!({
      "song_name": song.name.clone(),
      "stem_name": stem.name.clone(),
      "current": current_stem,
      "total": total_stems,
    }));

    let source_path = Path::new(&stem.file_path);
    let stem_id = stem.id.clone();

    // Debug logging for cache investigation
    log::info!("=== CACHE DEBUG for stem '{}' ===", stem.name);
    log::info!("Song ID: {}", song_id);
    log::info!("Stem ID: {}", stem_id);
    log::info!("Source path: {:?}", source_path);
    log::info!("Cache enabled: {}", state.cache_manager.is_enabled());

    // Try to get cached file path first
    let decode_path = if state.cache_manager.is_enabled() {
      log::info!("Attempting cache lookup...");
      match state.cache_manager.get(&song_id, &stem_id, source_path) {
        Ok(Some(cached_path)) => {
          log::info!("✓ CACHE HIT for {} - using cached file: {:?}", stem.name, cached_path);
          cached_path
        }
        Ok(None) => {
          log::info!("✗ CACHE MISS for {} - will decode from source and cache", stem.name);
          source_path.to_path_buf()
        }
        Err(e) => {
          log::error!("⚠ CACHE ERROR for {}: {} - decoding from source", stem.name, e);
          source_path.to_path_buf()
        }
      }
    } else {
      log::info!("Cache is DISABLED - decoding directly from source");
      source_path.to_path_buf()
    };

    // Decode from the chosen path (either cached or original)
    let mut decoder = super::super::audio::decoder::AudioDecoder::new(decode_path.to_str().unwrap())
      .map_err(|e| format!("Failed to create decoder for '{}': {}", stem.name, e))?;

    let metadata = decoder.get_metadata()
      .map_err(|e| format!("Failed to get metadata for '{}': {}", stem.name, e))?;

    let mut samples = decoder.decode_all()
      .map_err(|e| format!("Failed to decode '{}': {}", stem.name, e))?;

    // Resample if necessary
    let final_sample_rate = if metadata.sample_rate != 48000 {
      log::info!("Resampling {} from {}Hz to 48000Hz", stem.name, metadata.sample_rate);
      let mut resampler = super::super::audio::resampler::LinearResampler::new(
        metadata.sample_rate,
        48000,
        metadata.channels,
      );
      samples = resampler.process(&samples);
      48000
    } else {
      metadata.sample_rate
    };

    // If this was a cache miss, save the decoded audio to cache for next time
    if decode_path == source_path && state.cache_manager.is_enabled() {
      let duration = samples.len() as f64 / (final_sample_rate as f64 * metadata.channels as f64);
      if let Err(e) = state.cache_manager.put(
        &song_id,
        &stem_id,
        source_path,
        duration,
      ) {
        log::warn!("Failed to write to persistent cache: {}", e);
        // Continue anyway - we have the decoded samples
      } else {
        log::info!("Cached original file for future use");
      }
    }


    cached_stems.push(super::CachedStem {
      stem_id: stem.id.clone(),
      samples,
      volume: stem.volume as f32,
      is_muted: stem.is_muted,
    });

    log::debug!("Cached stem '{}' in memory", stem.name);
  }

  // Store in memory cache
  let mut cache = state.song_cache.lock().map_err(|_| "Failed to lock cache")?;
  cache.insert(song_id.clone(), super::CachedSong {
    song_id: song_id.clone(),
    stems: cached_stems,
  });

  log::info!("Successfully loaded song '{}' into memory", song.name);

  // Emit completion event
  let _ = app_handle.emit("stem:complete", serde_json::json!({}));

  Ok(())
}

/// Play a song from cache (load into audio engine and start playback)
#[tauri::command]
pub async fn play_song(song_id: String, state: State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
  log::info!("Playing song: {}", song_id);

  // Ensure song is cached (decode if needed)
  load_song(song_id.clone(), state.clone(), app_handle).await?;

  // Get cached song data
  let cached_song = {
    let cache = state.song_cache.lock().map_err(|_| "Failed to lock cache")?;
    cache.get(&song_id)
      .ok_or_else(|| "Song not in cache".to_string())?
      .clone() // Clone the CachedSong so we can use it outside the lock
  };

  // Lock the audio engine
  let mut engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  // Clear any previously loaded stems
  engine.clear_stems();

  // Clear the stem ID map
  let mut stem_map = state.stem_id_map
    .lock()
    .map_err(|_| "Failed to lock stem ID map")?;
  stem_map.clear();

  // Load cached stems into the engine
  for cached_stem in &cached_song.stems {
    let stem_index = engine
      .load_stem_from_samples(&cached_stem.samples)
      .map_err(|e| format!("Failed to load cached stem: {}", e))?;

    // Map the database stem ID to the engine stem index
    stem_map.insert(cached_stem.stem_id.clone(), stem_index);

    // Set volume and mute state
    engine.set_stem_volume(stem_index, cached_stem.volume);
    engine.set_stem_mute(stem_index, cached_stem.is_muted);
  }

  // Start playback
  engine
    .play()
    .map_err(|e| format!("Failed to start playback: {}", e))?;

  log::info!("Successfully started playback from cache");

  Ok(())
}

/// Resume current playback (after pause)
#[tauri::command]
pub async fn resume_playback(state: State<'_, AppState>) -> Result<(), String> {
  log::info!("Resuming playback");

  let mut engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  engine
    .play()
    .map_err(|e| format!("Failed to resume playback: {}", e))?;

  Ok(())
}

/// Pause current playback
#[tauri::command]
pub async fn pause_playback(state: State<'_, AppState>) -> Result<(), String> {
  log::info!("Pausing playback");

  let mut engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  engine
    .pause()
    .map_err(|e| format!("Failed to pause playback: {}", e))?;

  Ok(())
}

/// Stop current playback and reset position to start
#[tauri::command]
pub async fn stop_playback(state: State<'_, AppState>) -> Result<(), String> {
  log::info!("Stopping playback");

  let mut engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  engine
    .stop()
    .map_err(|e| format!("Failed to stop playback: {}", e))?;

  Ok(())
}

/// Seek to a specific position in the current song (in seconds)
#[tauri::command]
pub async fn seek_to_position(position: f64, state: State<'_, AppState>) -> Result<(), String> {
  log::info!("Seeking to position: {}", position);

  let mut engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  engine
    .seek(position)
    .map_err(|e| format!("Failed to seek: {}", e))?;

  Ok(())
}

/// Get current playback position in seconds
#[tauri::command]
pub async fn get_playback_position(state: State<'_, AppState>) -> Result<f64, String> {
  let engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  Ok(engine.position())
}

/// Preload all songs in a setlist for instant playback during performance
#[tauri::command]
pub async fn preload_setlist(setlist_id: String, state: State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
  log::info!("Starting to preload setlist: {}", setlist_id);

  // Get setlist and its songs
  let setlist = state.database
    .get_setlist(&setlist_id)
    .map_err(|e| format!("Failed to get setlist: {}", e))?;

  let songs = state.database
    .get_setlist_songs(&setlist_id)
    .map_err(|e| format!("Failed to get setlist songs: {}", e))?;

  let total = songs.len();
  log::info!("Found {} songs in setlist '{}'", total, setlist.name);

  for (index, song) in songs.iter().enumerate() {
    let current = index + 1;

    // Emit progress event
    let _ = app_handle.emit("preload:progress", serde_json::json!({
      "current": current,
      "total": total,
      "song_name": song.name.clone(),
    }));

    log::info!("Preloading song {}/{}: {}", current, total, song.name);

    // Load song into cache (decode all stems)
    if let Err(e) = load_song(song.id.clone(), state.clone(), app_handle.clone()).await {
      log::warn!("Failed to preload song '{}': {}", song.name, e);
      // Continue with next song even if this one fails
    }
  }

  // Emit completion event
  let _ = app_handle.emit("preload:complete", serde_json::json!({}));

  log::info!("Finished preloading setlist '{}'", setlist.name);
  Ok(())
}
