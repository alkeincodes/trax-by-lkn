use super::*;

#[test]
fn test_audio_engine_initialization() {
  let result = AudioEngine::new();
  assert!(result.is_ok(), "Audio engine should initialize successfully");

  let engine = result.unwrap();
  assert_eq!(engine.state(), PlaybackState::Stopped);
}

#[test]
fn test_playback_state_transitions() {
  let engine = AudioEngine::new().expect("Failed to create audio engine");

  assert_eq!(engine.state(), PlaybackState::Stopped);

  // Test that state is Stopped initially
  assert_eq!(engine.state(), PlaybackState::Stopped);
}

#[test]
fn test_default_volume() {
  let engine = AudioEngine::new().expect("Failed to create audio engine");
  assert_eq!(engine.volume(), 1.0, "Default volume should be 1.0");
}

#[test]
fn test_set_volume() {
  let mut engine = AudioEngine::new().expect("Failed to create audio engine");

  engine.set_volume(0.5);
  assert_eq!(engine.volume(), 0.5);

  engine.set_volume(0.0);
  assert_eq!(engine.volume(), 0.0);

  engine.set_volume(1.0);
  assert_eq!(engine.volume(), 1.0);
}

#[test]
fn test_volume_clamping() {
  let mut engine = AudioEngine::new().expect("Failed to create audio engine");

  // Test volume clamping to [0.0, 1.0]
  engine.set_volume(1.5);
  assert_eq!(engine.volume(), 1.0, "Volume should be clamped to 1.0");

  engine.set_volume(-0.5);
  assert_eq!(engine.volume(), 0.0, "Volume should be clamped to 0.0");
}

#[test]
fn test_position_tracking() {
  let engine = AudioEngine::new().expect("Failed to create audio engine");
  assert_eq!(engine.position(), 0.0, "Initial position should be 0.0");
}

#[test]
fn test_duration_before_load() {
  let engine = AudioEngine::new().expect("Failed to create audio engine");
  assert_eq!(engine.duration(), 0.0, "Duration should be 0.0 before loading a file");
}

// Mock test for audio device availability
#[test]
fn test_audio_device_available() {
  let result = AudioEngine::new();
  // This test verifies that cpal can enumerate audio devices
  assert!(result.is_ok(), "Should be able to access audio devices");
}

#[test]
fn test_multiple_engine_instances() {
  let engine1 = AudioEngine::new().expect("First engine should initialize");
  let engine2 = AudioEngine::new().expect("Second engine should initialize");

  // Both engines should be independent
  assert_eq!(engine1.state(), PlaybackState::Stopped);
  assert_eq!(engine2.state(), PlaybackState::Stopped);
}

#[test]
fn test_playback_without_loaded_file() {
  let mut engine = AudioEngine::new().expect("Failed to create audio engine");

  let result = engine.play();
  assert!(result.is_err(), "Play should fail without a loaded file");
}

#[test]
fn test_pause_without_playing() {
  let mut engine = AudioEngine::new().expect("Failed to create audio engine");

  let result = engine.pause();
  // Pausing while stopped should either succeed (no-op) or fail gracefully
  // We'll design it to succeed as a no-op
  assert!(result.is_ok(), "Pause should succeed as no-op when stopped");
  assert_eq!(engine.state(), PlaybackState::Stopped);
}

#[test]
fn test_stop_while_stopped() {
  let mut engine = AudioEngine::new().expect("Failed to create audio engine");

  let result = engine.stop();
  assert!(result.is_ok(), "Stop should succeed when already stopped");
  assert_eq!(engine.state(), PlaybackState::Stopped);
}

#[test]
fn test_seek_without_loaded_file() {
  let mut engine = AudioEngine::new().expect("Failed to create audio engine");

  let result = engine.seek(5.0);
  assert!(result.is_err(), "Seek should fail without a loaded file");
}
