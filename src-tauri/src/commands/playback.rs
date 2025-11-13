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
    if cache.contains(&song_id) {
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
  log::info!("Loading {} stems in PARALLEL...", total_stems);

  // Get device sample rate once before spawning tasks
  let device_sample_rate = {
    let engine = state.audio_engine.lock().map_err(|_| "Failed to lock engine")?;
    engine.device_sample_rate()
  };
  log::info!("Using device sample rate: {}Hz for all stems", device_sample_rate);

  // Spawn parallel decoding tasks for all stems
  let mut decode_tasks = Vec::new();

  for (index, stem) in stems.iter().enumerate() {
    let current_stem = index + 1;
    let song_id = song_id.clone();
    let song_name = song.name.clone();
    let stem_name = stem.name.clone();
    let stem_id = stem.id.clone();
    let stem_file_path = stem.file_path.clone();
    let stem_volume = stem.volume;
    let stem_is_muted = stem.is_muted;
    let app_handle_clone = app_handle.clone();

    // Spawn blocking task for CPU-intensive decoding
    let task = tokio::task::spawn_blocking(move || {
      log::info!("⚙️  PARALLEL: Starting decode for stem {}/{}: {}", current_stem, total_stems, stem_name);

      // Emit progress event to frontend
      let _ = app_handle_clone.emit("stem:loading", serde_json::json!({
        "song_name": song_name,
        "stem_name": stem_name.clone(),
        "current": current_stem,
        "total": total_stems,
      }));

      let source_path = Path::new(&stem_file_path);

      // Decode directly from original file
      let mut decoder = super::super::audio::decoder::AudioDecoder::new(source_path.to_str().unwrap())
        .map_err(|e| format!("Failed to create decoder for '{}': {}", stem_name, e))?;

      let metadata = decoder.get_metadata()
        .map_err(|e| format!("Failed to get metadata for '{}': {}", stem_name, e))?;

      let mut samples = decoder.decode_all()
        .map_err(|e| format!("Failed to decode '{}': {}", stem_name, e))?;

      // Resample if necessary (using device_sample_rate from outer scope)
      let final_sample_rate = if metadata.sample_rate != device_sample_rate {
        log::info!("Resampling {} from {}Hz to {}Hz", stem_name, metadata.sample_rate, device_sample_rate);
        let mut resampler = super::super::audio::resampler::LinearResampler::new(
          metadata.sample_rate,
          device_sample_rate,
          metadata.channels,
        );
        samples = resampler.process(&samples);
        device_sample_rate
      } else {
        metadata.sample_rate
      };

      log::info!("✅ PARALLEL: Completed decode for stem {}/{}: {} at {}Hz", current_stem, total_stems, stem_name, final_sample_rate);

      Ok::<_, String>(super::CachedStem {
        stem_id,
        samples: std::sync::Arc::new(samples), // Wrap in Arc for zero-copy
        sample_rate: final_sample_rate, // Store the sample rate
        volume: stem_volume as f32,
        is_muted: stem_is_muted,
      })
    });

    decode_tasks.push(task);
  }

  // Wait for all parallel decoding tasks to complete
  log::info!("⏳ Waiting for {} parallel decode tasks to complete...", decode_tasks.len());
  let results = futures::future::join_all(decode_tasks).await;

  // Collect results and check for errors
  let mut cached_stems = Vec::new();
  for (index, result) in results.into_iter().enumerate() {
    match result {
      Ok(Ok(cached_stem)) => {
        cached_stems.push(cached_stem);
      }
      Ok(Err(e)) => {
        return Err(format!("Failed to decode stem {}: {}", index + 1, e));
      }
      Err(e) => {
        return Err(format!("Task panic for stem {}: {}", index + 1, e));
      }
    }
  }

  log::info!("✅ All {} stems decoded successfully in parallel!", cached_stems.len());

  // Store in memory cache (LRU will auto-evict if needed)
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

  // Get cached song data (this updates LRU access time)
  let cached_song = {
    let mut cache = state.song_cache.lock().map_err(|_| "Failed to lock cache")?;
    cache.get(&song_id)
      .ok_or_else(|| "Song not in cache".to_string())?
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

  // Load cached stems into the engine (zero-copy via Arc)
  for cached_stem in &cached_song.stems {
    let stem_index = engine
      .load_stem_from_samples(cached_stem.samples.clone()) // Clone the Arc (cheap reference count bump)
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

/// Preload songs with priority based on current playback position
/// Priority: current song (instant) > next 2 > previous 1 > rest (background)
#[tauri::command]
pub async fn preload_setlist_smart(
  setlist_id: String,
  current_song_index: Option<usize>,
  state: State<'_, AppState>,
  app_handle: tauri::AppHandle
) -> Result<(), String> {
  log::info!("Starting smart preload for setlist: {} (current index: {:?})", setlist_id, current_song_index);

  // Get setlist and its songs
  let setlist = state.database
    .get_setlist(&setlist_id)
    .map_err(|e| format!("Failed to get setlist: {}", e))?;

  let songs = state.database
    .get_setlist_songs(&setlist_id)
    .map_err(|e| format!("Failed to get setlist songs: {}", e))?;

  if songs.is_empty() {
    return Ok(());
  }

  let total = songs.len();
  log::info!("Found {} songs in setlist '{}'", total, setlist.name);

  // Determine priority order based on current position
  let current_idx = current_song_index.unwrap_or(0);
  let mut priority_queue: Vec<(usize, &str, &str)> = Vec::new(); // (index, song_id, priority_label)

  // Priority 1: Current song (if specified)
  if current_idx < songs.len() {
    priority_queue.push((current_idx, &songs[current_idx].id, "CURRENT"));
  }

  // Priority 2: Next 2 songs
  for offset in 1..=2 {
    let next_idx = current_idx + offset;
    if next_idx < songs.len() {
      priority_queue.push((next_idx, &songs[next_idx].id, "NEXT"));
    }
  }

  // Priority 3: Previous 1 song
  if current_idx > 0 {
    let prev_idx = current_idx - 1;
    priority_queue.push((prev_idx, &songs[prev_idx].id, "PREVIOUS"));
  }

  // Priority 4: Rest of songs (background)
  for (index, song) in songs.iter().enumerate() {
    // Skip if already in priority queue
    if index == current_idx || (index > current_idx && index <= current_idx + 2) || (index == current_idx.saturating_sub(1)) {
      continue;
    }
    priority_queue.push((index, &song.id, "BACKGROUND"));
  }

  // Load songs in priority order
  for (loaded_count, (song_idx, song_id, priority)) in priority_queue.iter().enumerate() {
    let song_name = &songs[*song_idx].name;

    log::info!(
      "[{}] Preloading song {}/{}: '{}' (position {})",
      priority,
      loaded_count + 1,
      total,
      song_name,
      song_idx + 1
    );

    // Emit progress event
    let _ = app_handle.emit("preload:progress", serde_json::json!({
      "current": loaded_count + 1,
      "total": total,
      "song_name": song_name,
      "priority": priority,
      "position": song_idx + 1,
    }));

    // Load song into cache (LRU will auto-evict if needed)
    if let Err(e) = load_song(song_id.to_string(), state.clone(), app_handle.clone()).await {
      log::warn!("Failed to preload song '{}': {}", song_name, e);
      // Continue with next song even if this one fails
    }
  }

  // Emit completion event
  let _ = app_handle.emit("preload:complete", serde_json::json!({}));

  log::info!("Finished smart preload for setlist '{}'", setlist.name);
  Ok(())
}

/// Preload all songs in a setlist for instant playback during performance
/// (Legacy method - loads all songs sequentially without prioritization)
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

    // Load song into cache (decode all stems in parallel)
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
