use super::AppState;
use tauri::State;

/// Play a song by loading all its stems into the multi-track engine
#[tauri::command]
pub async fn play_song(song_id: String, state: State<'_, AppState>) -> Result<(), String> {
  log::info!("Playing song: {}", song_id);

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

  // Load each stem into the engine
  for stem in stems {
    let stem_index = engine
      .load_stem(&stem.file_path)
      .map_err(|e| format!("Failed to load stem '{}': {}", stem.name, e))?;

    // Map the database stem ID to the engine stem index
    stem_map.insert(stem.id.clone(), stem_index);

    // Set initial volume and mute state from database
    engine.set_stem_volume(stem_index, stem.volume as f32);
    engine.set_stem_mute(stem_index, stem.is_muted);

    log::debug!("Loaded stem '{}' at index {}", stem.name, stem_index);
  }

  // Start playback
  engine
    .play()
    .map_err(|e| format!("Failed to start playback: {}", e))?;

  log::info!("Successfully started playback of song '{}'", song.name);

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

  // Note: The current MultiTrackEngine doesn't have seek functionality
  // This is a placeholder for future implementation
  // For now, we just return Ok to satisfy the interface

  log::warn!("Seek functionality not yet implemented in MultiTrackEngine");

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
