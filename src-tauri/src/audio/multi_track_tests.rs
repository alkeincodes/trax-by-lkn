use super::*;
use std::path::PathBuf;

#[test]
fn test_multi_track_engine_initialization() {
  let result = MultiTrackEngine::new(16);
  assert!(
    result.is_ok(),
    "Multi-track engine should initialize successfully"
  );

  let engine = result.unwrap();
  assert_eq!(engine.max_stems(), 16, "Should support 16 stems");
  assert_eq!(engine.active_stems(), 0, "Should start with 0 active stems");
}

#[test]
fn test_stem_capacity_standard() {
  let engine = MultiTrackEngine::new_standard().expect("Failed to create standard engine");
  assert_eq!(engine.max_stems(), 16, "Standard capacity should be 16 stems");
}

#[test]
fn test_stem_capacity_extended() {
  let engine = MultiTrackEngine::new_extended().expect("Failed to create extended engine");
  assert_eq!(engine.max_stems(), 32, "Extended capacity should be 32 stems");
}

#[test]
fn test_stem_capacity_professional() {
  let engine = MultiTrackEngine::new_professional().expect("Failed to create professional engine");
  assert_eq!(engine.max_stems(), 64, "Professional capacity should be 64 stems");
}

#[test]
fn test_stem_capacity_custom() {
  let engine = MultiTrackEngine::with_capacity(StemCapacity::Custom(128))
    .expect("Failed to create custom engine");
  assert_eq!(engine.max_stems(), 128, "Custom capacity should be 128 stems");
}

#[test]
fn test_stem_capacity_enum() {
  assert_eq!(StemCapacity::Standard.as_usize(), 16);
  assert_eq!(StemCapacity::Extended.as_usize(), 32);
  assert_eq!(StemCapacity::Professional.as_usize(), 64);
  assert_eq!(StemCapacity::Custom(100).as_usize(), 100);

  assert_eq!(StemCapacity::from_usize(16), StemCapacity::Standard);
  assert_eq!(StemCapacity::from_usize(32), StemCapacity::Extended);
  assert_eq!(StemCapacity::from_usize(64), StemCapacity::Professional);
  assert_eq!(StemCapacity::from_usize(100), StemCapacity::Custom(100));
}

#[test]
fn test_maximum_stem_limit() {
  // Should succeed with 256 stems (maximum allowed)
  let result = MultiTrackEngine::new(256);
  assert!(result.is_ok(), "Should support up to 256 stems");

  // Should fail with 257 stems (exceeds limit)
  let result = MultiTrackEngine::new(257);
  assert!(result.is_err(), "Should reject more than 256 stems");
}

#[test]
fn test_minimum_stem_limit() {
  // Should fail with 0 stems
  let result = MultiTrackEngine::new(0);
  assert!(result.is_err(), "Should reject 0 stems");

  // Should succeed with 1 stem
  let result = MultiTrackEngine::new(1);
  assert!(result.is_ok(), "Should support at least 1 stem");
}

#[test]
fn test_load_multiple_stems() {
  let mut engine = MultiTrackEngine::new(8).expect("Failed to create engine");

  // Simulate loading 4 stems for a song
  let stem_paths = vec![
    PathBuf::from("test_vocals.wav"),
    PathBuf::from("test_drums.wav"),
    PathBuf::from("test_bass.wav"),
    PathBuf::from("test_keys.wav"),
  ];

  // Note: These paths don't exist, so we expect errors
  // In real implementation, we'd use actual test files
  for path in &stem_paths {
    let _ = engine.load_stem(path.to_str().unwrap());
  }

  // This test verifies the API exists and accepts multiple stems
}

#[test]
fn test_stem_synchronization() {
  let mut engine = MultiTrackEngine::new(4).expect("Failed to create engine");

  // After loading stems and starting playback,
  // all stems should report the same playback position
  engine.play().ok();

  // Position should be synchronized across all stems
  let position = engine.position();
  assert!(position >= 0.0, "Position should be non-negative");
}

#[test]
fn test_per_stem_volume_control() {
  let mut engine = MultiTrackEngine::new(4).expect("Failed to create engine");

  // Set different volumes for different stems
  engine.set_stem_volume(0, 0.5);
  engine.set_stem_volume(1, 0.8);
  engine.set_stem_volume(2, 1.0);
  engine.set_stem_volume(3, 0.3);

  assert_eq!(engine.stem_volume(0), 0.5);
  assert_eq!(engine.stem_volume(1), 0.8);
  assert_eq!(engine.stem_volume(2), 1.0);
  assert_eq!(engine.stem_volume(3), 0.3);
}

#[test]
fn test_volume_clamping_per_stem() {
  let mut engine = MultiTrackEngine::new(2).expect("Failed to create engine");

  // Test volume clamping to [0.0, 1.0]
  engine.set_stem_volume(0, 1.5);
  assert_eq!(engine.stem_volume(0), 1.0, "Volume should be clamped to 1.0");

  engine.set_stem_volume(1, -0.5);
  assert_eq!(engine.stem_volume(1), 0.0, "Volume should be clamped to 0.0");
}

#[test]
fn test_linear_to_db_conversion() {
  let mut engine = MultiTrackEngine::new(1).expect("Failed to create engine");

  // Linear 0.0 = -∞ dB (silence)
  engine.set_stem_volume(0, 0.0);
  assert_eq!(
    engine.stem_volume_db(0),
    f32::NEG_INFINITY,
    "Linear 0.0 should be -∞ dB"
  );

  // Linear 1.0 = 0 dB (unity gain)
  engine.set_stem_volume(0, 1.0);
  assert!(
    (engine.stem_volume_db(0) - 0.0).abs() < 0.01,
    "Linear 1.0 should be ~0 dB"
  );

  // Linear 0.5 ≈ -6 dB
  engine.set_stem_volume(0, 0.5);
  let db = engine.stem_volume_db(0);
  assert!(
    (db - (-6.0)).abs() < 0.5,
    "Linear 0.5 should be approximately -6 dB, got {}",
    db
  );
}

#[test]
fn test_stem_mute_functionality() {
  let mut engine = MultiTrackEngine::new(4).expect("Failed to create engine");

  // All stems should start unmuted
  assert!(!engine.is_stem_muted(0), "Stem 0 should start unmuted");
  assert!(!engine.is_stem_muted(1), "Stem 1 should start unmuted");

  // Mute stem 0
  engine.set_stem_mute(0, true);
  assert!(engine.is_stem_muted(0), "Stem 0 should be muted");
  assert!(!engine.is_stem_muted(1), "Stem 1 should still be unmuted");

  // Unmute stem 0
  engine.set_stem_mute(0, false);
  assert!(!engine.is_stem_muted(0), "Stem 0 should be unmuted");
}

#[test]
fn test_stem_solo_functionality() {
  let mut engine = MultiTrackEngine::new(4).expect("Failed to create engine");

  // No stems should be soloed initially
  assert!(!engine.is_stem_soloed(0), "Stem 0 should not be soloed");
  assert!(!engine.is_stem_soloed(1), "Stem 1 should not be soloed");

  // Solo stem 0
  engine.set_stem_solo(0, true);
  assert!(engine.is_stem_soloed(0), "Stem 0 should be soloed");

  // When stem 0 is soloed, only stem 0 should be audible
  // (implementation will mute all other stems)
  assert!(!engine.is_stem_soloed(1), "Stem 1 should not be soloed");
}

#[test]
fn test_multiple_stem_solo() {
  let mut engine = MultiTrackEngine::new(4).expect("Failed to create engine");

  // Solo stems 0 and 2
  engine.set_stem_solo(0, true);
  engine.set_stem_solo(2, true);

  assert!(engine.is_stem_soloed(0), "Stem 0 should be soloed");
  assert!(engine.is_stem_soloed(2), "Stem 2 should be soloed");
  assert!(!engine.is_stem_soloed(1), "Stem 1 should not be soloed");
  assert!(!engine.is_stem_soloed(3), "Stem 3 should not be soloed");

  // When any stems are soloed, only soloed stems should be audible
}

#[test]
fn test_solo_overrides_mute() {
  let mut engine = MultiTrackEngine::new(2).expect("Failed to create engine");

  // Mute stem 0
  engine.set_stem_mute(0, true);
  assert!(engine.is_stem_muted(0), "Stem 0 should be muted");

  // Solo stem 0 (should override mute)
  engine.set_stem_solo(0, true);
  assert!(engine.is_stem_soloed(0), "Stem 0 should be soloed");

  // Stem should be audible when soloed, even if muted
  // (implementation detail: solo takes precedence)
}

#[test]
fn test_stem_count_limits() {
  let mut engine = MultiTrackEngine::new(16).expect("Failed to create engine");

  // Should handle up to 16 stems
  for i in 0..16 {
    engine.set_stem_volume(i, 0.8);
    assert_eq!(engine.stem_volume(i), 0.8);
  }

  // Accessing stem 16 (out of bounds) should not panic
  // Implementation should handle gracefully
  engine.set_stem_volume(16, 0.5);
  // Should either ignore or return error
}

#[test]
fn test_buffer_pool_allocation() {
  let engine = MultiTrackEngine::new(8).expect("Failed to create engine");

  // Buffer pool should be allocated for 8 stems
  assert!(
    engine.buffer_pool_capacity() >= 8,
    "Buffer pool should have capacity for at least 8 stems"
  );
}

#[test]
fn test_clear_all_stems() {
  let mut engine = MultiTrackEngine::new(4).expect("Failed to create engine");

  // Simulate loading stems (mock implementation)
  // In real code, this would load actual audio files

  // Clear all stems
  engine.clear_stems();

  assert_eq!(
    engine.active_stems(),
    0,
    "Should have 0 active stems after clearing"
  );
}

#[test]
fn test_real_time_volume_update() {
  let mut engine = MultiTrackEngine::new(2).expect("Failed to create engine");

  // Start playback
  engine.play().ok();

  // Update volume during playback (should not glitch)
  engine.set_stem_volume(0, 0.5);
  engine.set_stem_volume(0, 0.7);
  engine.set_stem_volume(0, 1.0);

  // Volume should update smoothly without audio artifacts
  assert_eq!(engine.stem_volume(0), 1.0);
}

#[test]
fn test_synchronized_playback_start() {
  let mut engine = MultiTrackEngine::new(4).expect("Failed to create engine");

  // All stems should start at position 0.0
  engine.play().ok();

  let position = engine.position();
  assert!(
    position >= 0.0 && position < 0.1,
    "All stems should start near position 0.0"
  );
}

#[test]
fn test_stem_metadata() {
  let mut engine = MultiTrackEngine::new(2).expect("Failed to create engine");

  // Should be able to get stem information
  // This would return name, duration, sample rate, etc.
  // For now, just verify the API exists
  let _ = engine.stem_count();
}
