use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::audio::PlaybackState;

/// Start a background task that emits playback position updates
pub fn start_position_emitter(
  app_handle: AppHandle,
  position: Arc<AtomicU64>,
  playback_state: Arc<Mutex<PlaybackState>>,
) {
  tauri::async_runtime::spawn(async move {
    loop {
      tokio::time::sleep(Duration::from_millis(100)).await;

      // Get current position (sample position)
      let sample_position = position.load(Ordering::Acquire);
      let position_seconds = sample_position as f64 / (48000.0 * 2.0); // TARGET_SAMPLE_RATE * channels

      // Get playback state
      let is_playing = {
        let state = match playback_state.lock() {
          Ok(s) => *s,
          Err(_) => continue,
        };
        matches!(state, PlaybackState::Playing)
      };

      // Emit position event
      if let Err(e) = app_handle.emit("playback:position", serde_json::json!({
        "position": position_seconds
      })) {
        log::error!("Failed to emit position event: {}", e);
      }

      // Emit state event
      if let Err(e) = app_handle.emit("playback:state", serde_json::json!({
        "is_playing": is_playing
      })) {
        log::error!("Failed to emit state event: {}", e);
      }
    }
  });
}
