use super::AppState;
use tauri::State;

/// Set the volume for a specific stem (0.0 to 1.0)
#[tauri::command]
pub async fn set_stem_volume(
  stem_id: String,
  volume: f64,
  state: State<'_, AppState>
) -> Result<(), String> {
  log::debug!("Setting stem {} volume to {}", stem_id, volume);

  // Clamp volume to valid range
  let clamped_volume = volume.clamp(0.0, 1.0);

  // Get the engine stem index from the database stem ID
  let stem_map = state.stem_id_map
    .lock()
    .map_err(|_| "Failed to lock stem ID map")?;

  let stem_index = stem_map
    .get(&stem_id)
    .ok_or_else(|| format!("Stem not found in audio engine: {}", stem_id))?;

  // Update the audio engine
  let mut engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  engine.set_stem_volume(*stem_index, clamped_volume as f32);

  // Update the database
  let mut stem = state.database
    .get_stem(&stem_id)
    .map_err(|e| format!("Failed to get stem from database: {}", e))?;

  stem.volume = clamped_volume;

  state.database
    .update_stem(&stem)
    .map_err(|e| format!("Failed to update stem in database: {}", e))?;

  Ok(())
}

/// Toggle mute state for a specific stem
#[tauri::command]
pub async fn toggle_stem_mute(
  stem_id: String,
  state: State<'_, AppState>
) -> Result<(), String> {
  log::debug!("Toggling mute for stem {}", stem_id);

  // Get current stem state from database
  let mut stem = state.database
    .get_stem(&stem_id)
    .map_err(|e| format!("Failed to get stem from database: {}", e))?;

  // Toggle mute state
  stem.is_muted = !stem.is_muted;

  // Get the engine stem index
  let stem_map = state.stem_id_map
    .lock()
    .map_err(|_| "Failed to lock stem ID map")?;

  let stem_index = stem_map
    .get(&stem_id)
    .ok_or_else(|| format!("Stem not found in audio engine: {}", stem_id))?;

  // Update the audio engine
  let mut engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  engine.set_stem_mute(*stem_index, stem.is_muted);

  // Update the database
  state.database
    .update_stem(&stem)
    .map_err(|e| format!("Failed to update stem in database: {}", e))?;

  Ok(())
}

/// Toggle solo state for a specific stem
#[tauri::command]
pub async fn toggle_stem_solo(
  stem_id: String,
  state: State<'_, AppState>
) -> Result<bool, String> {
  log::debug!("Toggling solo for stem {}", stem_id);

  // Get the engine stem index
  let stem_map = state.stem_id_map
    .lock()
    .map_err(|_| "Failed to lock stem ID map")?;

  let stem_index = stem_map
    .get(&stem_id)
    .ok_or_else(|| format!("Stem not found in audio engine: {}", stem_id))?;

  // Get current solo state and toggle it
  let mut engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  let current_solo = engine.is_stem_soloed(*stem_index);
  let new_solo = !current_solo;
  engine.set_stem_solo(*stem_index, new_solo);

  // Note: Solo state is not persisted in database (it's ephemeral)

  Ok(new_solo)
}

/// Set the master volume (0.0 to 1.0)
#[tauri::command]
pub async fn set_master_volume(
  volume: f64,
  state: State<'_, AppState>
) -> Result<(), String> {
  log::debug!("Setting master volume to {}", volume);

  // Clamp volume to valid range
  let clamped_volume = volume.clamp(0.0, 1.0);

  // Update the audio engine
  let mut engine = state.audio_engine
    .lock()
    .map_err(|_| "Failed to lock audio engine")?;

  engine.set_master_volume(clamped_volume as f32);

  Ok(())
}

/// Get all stems for the currently loaded song
#[tauri::command]
pub async fn get_current_stems(
  state: State<'_, AppState>
) -> Result<Vec<crate::database::Stem>, String> {
  // This would need to track the current song ID in app state
  // For now, return an empty list
  Ok(Vec::new())
}
