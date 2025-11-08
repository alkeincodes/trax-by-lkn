use super::AppState;
use crate::database::Setlist;
use tauri::State;

/// Create a new empty setlist
#[tauri::command]
pub async fn create_setlist(
  name: String,
  state: State<'_, AppState>
) -> Result<String, String> {
  log::info!("Creating new setlist: {}", name);

  let setlist_id = uuid::Uuid::new_v4().to_string();
  let now = chrono::Utc::now().timestamp();

  let setlist = Setlist {
    id: setlist_id.clone(),
    name,
    created_at: now,
    updated_at: now,
    song_ids: Vec::new(),
  };

  state.database
    .create_setlist(&setlist)
    .map_err(|e| format!("Failed to create setlist: {}", e))?;

  log::info!("Created setlist with ID: {}", setlist_id);

  Ok(setlist_id)
}

/// Get a specific setlist by ID
#[tauri::command]
pub async fn get_setlist(
  setlist_id: String,
  state: State<'_, AppState>
) -> Result<Setlist, String> {
  log::debug!("Getting setlist: {}", setlist_id);

  let setlist = state.database
    .get_setlist(&setlist_id)
    .map_err(|e| format!("Failed to get setlist: {}", e))?;

  Ok(setlist)
}

/// Update a setlist (name and/or song order)
#[tauri::command]
pub async fn update_setlist(
  setlist_id: String,
  name: Option<String>,
  song_ids: Option<Vec<String>>,
  state: State<'_, AppState>
) -> Result<(), String> {
  log::info!("Updating setlist: {}", setlist_id);

  // Get current setlist
  let mut setlist = state.database
    .get_setlist(&setlist_id)
    .map_err(|e| format!("Failed to get setlist: {}", e))?;

  // Update fields if provided
  if let Some(new_name) = name {
    setlist.name = new_name;
  }

  if let Some(new_song_ids) = song_ids {
    setlist.song_ids = new_song_ids;
  }

  // Update timestamp
  setlist.updated_at = chrono::Utc::now().timestamp();

  // Save to database
  state.database
    .update_setlist(&setlist)
    .map_err(|e| format!("Failed to update setlist: {}", e))?;

  Ok(())
}

/// Delete a setlist
#[tauri::command]
pub async fn delete_setlist(
  setlist_id: String,
  state: State<'_, AppState>
) -> Result<(), String> {
  log::info!("Deleting setlist: {}", setlist_id);

  state.database
    .delete_setlist(&setlist_id)
    .map_err(|e| format!("Failed to delete setlist: {}", e))?;

  Ok(())
}

/// Get all setlists
#[tauri::command]
pub async fn get_all_setlists(
  state: State<'_, AppState>
) -> Result<Vec<Setlist>, String> {
  log::debug!("Getting all setlists");

  let setlists = state.database
    .list_setlists()
    .map_err(|e| format!("Failed to get setlists: {}", e))?;

  Ok(setlists)
}

/// Add a song to a setlist
#[tauri::command]
pub async fn add_song_to_setlist(
  setlist_id: String,
  song_id: String,
  state: State<'_, AppState>
) -> Result<(), String> {
  log::info!("Adding song {} to setlist {}", song_id, setlist_id);

  // Get current setlist
  let mut setlist = state.database
    .get_setlist(&setlist_id)
    .map_err(|e| format!("Failed to get setlist: {}", e))?;

  // Add song if not already in setlist
  if !setlist.song_ids.contains(&song_id) {
    setlist.song_ids.push(song_id);
    setlist.updated_at = chrono::Utc::now().timestamp();

    state.database
      .update_setlist(&setlist)
      .map_err(|e| format!("Failed to update setlist: {}", e))?;
  }

  Ok(())
}

/// Remove a song from a setlist
#[tauri::command]
pub async fn remove_song_from_setlist(
  setlist_id: String,
  song_id: String,
  state: State<'_, AppState>
) -> Result<(), String> {
  log::info!("Removing song {} from setlist {}", song_id, setlist_id);

  // Get current setlist
  let mut setlist = state.database
    .get_setlist(&setlist_id)
    .map_err(|e| format!("Failed to get setlist: {}", e))?;

  // Remove song
  setlist.song_ids.retain(|id| id != &song_id);
  setlist.updated_at = chrono::Utc::now().timestamp();

  state.database
    .update_setlist(&setlist)
    .map_err(|e| format!("Failed to update setlist: {}", e))?;

  Ok(())
}

/// Reorder songs in a setlist
#[tauri::command]
pub async fn reorder_setlist_songs(
  setlist_id: String,
  song_ids: Vec<String>,
  state: State<'_, AppState>
) -> Result<(), String> {
  log::info!("Reordering songs in setlist {}", setlist_id);

  // Get current setlist
  let mut setlist = state.database
    .get_setlist(&setlist_id)
    .map_err(|e| format!("Failed to get setlist: {}", e))?;

  // Update song order
  setlist.song_ids = song_ids;
  setlist.updated_at = chrono::Utc::now().timestamp();

  state.database
    .update_setlist(&setlist)
    .map_err(|e| format!("Failed to update setlist: {}", e))?;

  Ok(())
}
