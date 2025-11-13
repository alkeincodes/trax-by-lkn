use super::AppState;
use tauri::State;

/// Get cache statistics (num_songs, current_bytes, max_bytes)
#[tauri::command]
pub async fn get_cache_stats(state: State<'_, AppState>) -> Result<(usize, usize, usize), String> {
  let cache = state.song_cache.lock()
    .map_err(|_| "Failed to lock cache".to_string())?;

  Ok(cache.stats())
}

/// Set cache size limit in bytes
#[tauri::command]
pub async fn set_cache_size(size_bytes: usize, state: State<'_, AppState>) -> Result<(), String> {
  log::info!("Setting cache size to {} bytes ({:.1} GB)", size_bytes, size_bytes as f64 / 1_073_741_824.0);

  let mut cache = state.song_cache.lock()
    .map_err(|_| "Failed to lock cache".to_string())?;

  cache.set_max_size(size_bytes);

  Ok(())
}

/// Clear all cached songs
#[tauri::command]
pub async fn clear_cache(state: State<'_, AppState>) -> Result<(), String> {
  log::info!("Clearing cache");

  let mut cache = state.song_cache.lock()
    .map_err(|_| "Failed to lock cache".to_string())?;

  cache.clear();

  Ok(())
}
