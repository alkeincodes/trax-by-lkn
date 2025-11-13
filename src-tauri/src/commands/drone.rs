use tauri::State;
use std::path::PathBuf;
use super::AppState;

/// Play a drone pad with specified key and preset
#[tauri::command]
pub fn drone_play(
  state: State<'_, AppState>,
  preset_folder: String,
  key: String,
) -> Result<(), String> {
  log::info!("Drone command: play {} - {}", preset_folder, key);

  // Get current device from audio engine
  let device_name = {
    let engine = state.audio_engine.lock()
      .map_err(|_| "Failed to lock audio engine".to_string())?;
    engine.current_device_name()
  };

  // Construct path to drone pad audio file
  // Assumes files are in the app's resources at: drone-pads/{preset_folder}/{key}.mp3
  let app_path = std::env::current_exe()
    .map_err(|e| format!("Failed to get app path: {}", e))?;
  let app_dir = app_path.parent()
    .ok_or("Failed to get app directory")?;

  // For dev mode, files are in public/drone-pads
  // For production, they'll be in resources
  let file_path = if cfg!(debug_assertions) {
    // Dev mode - look in workspace public folder
    let workspace_root = app_dir.parent()
      .and_then(|p| p.parent())
      .and_then(|p| p.parent())
      .ok_or("Failed to find workspace root")?;
    workspace_root.join("public").join("drone-pads").join(&preset_folder).join(format!("{}.mp3", key))
  } else {
    // Production - look in app resources
    app_dir.join("resources").join("drone-pads").join(&preset_folder).join(format!("{}.mp3", key))
  };

  log::info!("Loading drone pad from: {:?}", file_path);

  if !file_path.exists() {
    return Err(format!("Drone pad file not found: {:?}", file_path));
  }

  // Load and play
  let mut player = state.drone_player.lock()
    .map_err(|_| "Failed to lock drone player".to_string())?;

  player.load(file_path)
    .map_err(|e| format!("Failed to load drone pad: {}", e))?;

  player.play(device_name)
    .map_err(|e| format!("Failed to play drone pad: {}", e))?;

  Ok(())
}

/// Stop drone pad playback
#[tauri::command]
pub fn drone_stop(state: State<'_, AppState>) -> Result<(), String> {
  log::info!("Drone command: stop");

  let mut player = state.drone_player.lock()
    .map_err(|_| "Failed to lock drone player".to_string())?;

  player.stop();
  Ok(())
}

/// Set drone pad volume (0.0 to 1.0)
#[tauri::command]
pub fn drone_set_volume(
  state: State<'_, AppState>,
  volume: f32,
) -> Result<(), String> {
  log::info!("Drone command: set volume to {}", volume);

  let mut player = state.drone_player.lock()
    .map_err(|_| "Failed to lock drone player".to_string())?;

  player.set_volume(volume);
  Ok(())
}

/// Check if drone pad is currently playing
#[tauri::command]
pub fn drone_is_playing(state: State<'_, AppState>) -> Result<bool, String> {
  let player = state.drone_player.lock()
    .map_err(|_| "Failed to lock drone player".to_string())?;

  Ok(player.is_playing())
}

/// Switch drone pad to a different audio device
#[tauri::command]
pub fn drone_switch_device(
  state: State<'_, AppState>,
  device_name: String,
) -> Result<(), String> {
  log::info!("Drone command: switch device to {}", device_name);

  let mut player = state.drone_player.lock()
    .map_err(|_| "Failed to lock drone player".to_string())?;

  player.switch_device(device_name)
    .map_err(|e| format!("Failed to switch device: {}", e))
}
