use super::AppState;
use crate::database::{Song, SongFilter, SortBy};
use crate::import::{import_song, ImportRequest};
use std::path::PathBuf;
use tauri::State;

/// Import audio files as a new song with stems
#[tauri::command]
pub async fn import_files(
  file_paths: Vec<String>,
  title: String,
  artist: Option<String>,
  key: Option<String>,
  time_signature: Option<String>,
  state: State<'_, AppState>,
  app_handle: tauri::AppHandle,
) -> Result<String, String> {
  log::info!("Importing {} files for song '{}'", file_paths.len(), title);

  // Convert string paths to PathBuf
  let paths: Vec<PathBuf> = file_paths.iter().map(PathBuf::from).collect();

  // Create import request
  let request = ImportRequest {
    file_paths: paths,
    title,
    artist,
    key,
    time_signature,
  };

  // Perform the import
  let song_id = import_song(&*state.database, request)
    .map_err(|e| format!("Import failed: {}", e))?;

  log::info!("Successfully imported song with ID: {}", song_id);

  // TODO: Emit import:progress events using app_handle.emit()
  // This will be implemented in the event emitter task

  Ok(song_id)
}

/// Get all songs from the library
#[tauri::command]
pub async fn get_all_songs(state: State<'_, AppState>) -> Result<Vec<Song>, String> {
  log::debug!("Getting all songs");

  let songs = state.database
    .list_songs(None)
    .map_err(|e| format!("Failed to get songs: {}", e))?;

  Ok(songs)
}

/// Search songs by query string (searches name and artist)
#[tauri::command]
pub async fn search_songs(
  query: String,
  state: State<'_, AppState>
) -> Result<Vec<Song>, String> {
  log::debug!("Searching songs with query: {}", query);

  let filter = SongFilter {
    search_query: Some(query),
    tempo_min: None,
    tempo_max: None,
    key: None,
    sort_by: None,
  };

  let songs = state.database
    .list_songs(Some(filter))
    .map_err(|e| format!("Failed to search songs: {}", e))?;

  Ok(songs)
}

/// Filter songs with multiple criteria
#[tauri::command]
pub async fn filter_songs(
  search_query: Option<String>,
  tempo_min: Option<f64>,
  tempo_max: Option<f64>,
  key: Option<String>,
  sort_by: Option<String>,
  state: State<'_, AppState>
) -> Result<Vec<Song>, String> {
  log::debug!("Filtering songs with criteria");

  // Convert sort_by string to enum
  let sort_option = match sort_by.as_deref() {
    Some("name") => Some(SortBy::Name),
    Some("artist") => Some(SortBy::Artist),
    Some("tempo") => Some(SortBy::Tempo),
    Some("duration") => Some(SortBy::Duration),
    Some("date_added") => Some(SortBy::DateAdded),
    _ => None,
  };

  let filter = SongFilter {
    search_query,
    tempo_min,
    tempo_max,
    key,
    sort_by: sort_option,
  };

  let songs = state.database
    .list_songs(Some(filter))
    .map_err(|e| format!("Failed to filter songs: {}", e))?;

  Ok(songs)
}

/// Get a specific song by ID
#[tauri::command]
pub async fn get_song(
  song_id: String,
  state: State<'_, AppState>
) -> Result<Song, String> {
  log::debug!("Getting song: {}", song_id);

  let song = state.database
    .get_song(&song_id)
    .map_err(|e| format!("Failed to get song: {}", e))?;

  Ok(song)
}

/// Delete a song and all its stems
#[tauri::command]
pub async fn delete_song(
  song_id: String,
  state: State<'_, AppState>
) -> Result<(), String> {
  log::info!("Deleting song: {}", song_id);

  // Delete all stems first
  let stems = state.database
    .get_stems_for_song(&song_id)
    .map_err(|e| format!("Failed to get stems: {}", e))?;

  for stem in stems {
    state.database
      .delete_stem(&stem.id)
      .map_err(|e| format!("Failed to delete stem: {}", e))?;
  }

  // Delete the song
  state.database
    .delete_song(&song_id)
    .map_err(|e| format!("Failed to delete song: {}", e))?;

  Ok(())
}

/// Get all stems for a specific song
#[tauri::command]
pub async fn get_song_stems(
  song_id: String,
  state: State<'_, AppState>
) -> Result<Vec<crate::database::Stem>, String> {
  log::debug!("Getting stems for song: {}", song_id);

  let stems = state.database
    .get_stems_for_song(&song_id)
    .map_err(|e| format!("Failed to get stems: {}", e))?;

  Ok(stems)
}
