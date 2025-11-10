use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::audio::PlaybackState;

/// Start a background task that emits playback position updates
pub fn start_position_emitter(
  app_handle: AppHandle,
  position: Arc<AtomicU64>,
  playback_state: Arc<Mutex<PlaybackState>>,
  stem_levels: Vec<Arc<AtomicU32>>,
  master_level: Arc<AtomicU32>,
) {
  tauri::async_runtime::spawn(async move {
    loop {
      tokio::time::sleep(Duration::from_millis(50)).await; // 20 FPS for smooth meters

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

      // Get stem levels (convert from atomic bits to f32)
      let levels: Vec<f32> = stem_levels
        .iter()
        .map(|level| f32::from_bits(level.load(Ordering::Acquire)))
        .collect();

      // Get master level
      let master = f32::from_bits(master_level.load(Ordering::Acquire));

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

      // Emit stem levels event with master level
      if let Err(e) = app_handle.emit("playback:levels", serde_json::json!({
        "levels": levels,
        "master": master
      })) {
        log::error!("Failed to emit levels event: {}", e);
      }
    }
  });
}
