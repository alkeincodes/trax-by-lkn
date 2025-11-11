use super::*;
use crate::audio::{MultiTrackEngine, StemCapacity};
use crate::database::{Database, Song, Stem, Setlist};

// Helper function to create test database
fn create_test_database() -> Database {
  Database::new_in_memory().expect("Failed to create test database")
}

// Helper function to create a test song in database
fn create_test_song(db: &Database, name: &str) -> Song {
  let song = Song {
    id: uuid::Uuid::new_v4().to_string(),
    name: name.to_string(),
    artist: Some("Test Artist".to_string()),
    duration: 180.0,
    tempo: Some(120.0),
    key: Some("C".to_string()),
    time_signature: Some("4/4".to_string()),
    mixdown_path: None,
    created_at: chrono::Utc::now().timestamp(),
    updated_at: chrono::Utc::now().timestamp(),
  };

  db.create_song(&song).expect("Failed to create test song");
  song
}

// Helper function to create a test stem in database
fn create_test_stem(db: &Database, song_id: &str, name: &str) -> Stem {
  let stem = Stem {
    id: uuid::Uuid::new_v4().to_string(),
    song_id: song_id.to_string(),
    name: name.to_string(),
    file_path: "/path/to/test.wav".to_string(),
    file_size: 1024000,
    sample_rate: 48000,
    channels: 2,
    duration: 180.0,
    volume: 0.8,
    is_muted: false,
  };

  db.create_stem(&stem).expect("Failed to create test stem");
  stem
}

#[cfg(test)]
mod database_integration_tests {
  use super::*;

  #[test]
  fn test_database_song_operations() {
    let db = create_test_database();

    // Create songs
    let song1 = create_test_song(&db, "Song 1");
    let song2 = create_test_song(&db, "Song 2");

    // Get all songs
    let songs = db.list_songs(None).expect("Failed to list songs");
    assert_eq!(songs.len(), 2);

    // Get specific song
    let retrieved = db.get_song(&song1.id).expect("Failed to get song");
    assert_eq!(retrieved.name, "Song 1");

    // Delete song
    db.delete_song(&song2.id).expect("Failed to delete song");
    let songs = db.list_songs(None).expect("Failed to list songs");
    assert_eq!(songs.len(), 1);
  }

  #[test]
  fn test_database_stem_operations() {
    let db = create_test_database();
    let song = create_test_song(&db, "Test Song");

    // Create stems
    let stem1 = create_test_stem(&db, &song.id, "Vocals");
    let stem2 = create_test_stem(&db, &song.id, "Drums");

    // Get stems for song
    let stems = db.get_stems_for_song(&song.id).expect("Failed to get stems");
    assert_eq!(stems.len(), 2);

    // Update stem
    let mut updated_stem = stem1.clone();
    updated_stem.volume = 0.5;
    db.update_stem(&updated_stem).expect("Failed to update stem");

    let retrieved = db.get_stem(&stem1.id).expect("Failed to get stem");
    assert_eq!(retrieved.volume, 0.5);
  }

  #[test]
  fn test_database_setlist_operations() {
    let db = create_test_database();
    let song1 = create_test_song(&db, "Song 1");
    let song2 = create_test_song(&db, "Song 2");

    // Create setlist
    let setlist_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let setlist = Setlist {
      id: setlist_id.clone(),
      name: "Sunday Service".to_string(),
      created_at: now,
      updated_at: now,
      song_ids: vec![song1.id.clone(), song2.id.clone()],
    };

    db.create_setlist(&setlist).expect("Failed to create setlist");

    // Get setlist
    let retrieved = db.get_setlist(&setlist_id).expect("Failed to get setlist");
    assert_eq!(retrieved.name, "Sunday Service");
    assert_eq!(retrieved.song_ids.len(), 2);

    // Update setlist
    let mut updated = retrieved.clone();
    updated.name = "Updated Setlist".to_string();
    updated.song_ids = vec![song1.id.clone()];
    db.update_setlist(&updated).expect("Failed to update setlist");

    let retrieved = db.get_setlist(&setlist_id).expect("Failed to get setlist");
    assert_eq!(retrieved.name, "Updated Setlist");
    assert_eq!(retrieved.song_ids.len(), 1);

    // Delete setlist
    db.delete_setlist(&setlist_id).expect("Failed to delete setlist");
    let setlists = db.list_setlists().expect("Failed to list setlists");
    assert_eq!(setlists.len(), 0);
  }
}

#[cfg(test)]
mod audio_engine_tests {
  use super::*;

  #[test]
  fn test_multi_track_engine_creation() {
    let engine = MultiTrackEngine::with_capacity(StemCapacity::Standard);
    assert!(engine.is_ok());

    let engine = engine.unwrap();
    assert_eq!(engine.max_stems(), 16);
    assert_eq!(engine.active_stems(), 0);
  }

  #[test]
  fn test_multi_track_engine_stem_controls() {
    let mut engine = MultiTrackEngine::with_capacity(StemCapacity::Standard)
      .expect("Failed to create engine");

    // Test volume control
    engine.set_stem_volume(0, 0.5);
    assert_eq!(engine.stem_volume(0), 0.5);

    // Test mute control
    engine.set_stem_mute(0, true);
    assert!(engine.is_stem_muted(0));

    engine.set_stem_mute(0, false);
    assert!(!engine.is_stem_muted(0));

    // Test solo control
    engine.set_stem_solo(0, true);
    assert!(engine.is_stem_soloed(0));

    engine.set_stem_solo(0, false);
    assert!(!engine.is_stem_soloed(0));
  }

  #[test]
  fn test_multi_track_engine_playback_controls() {
    let mut engine = MultiTrackEngine::with_capacity(StemCapacity::Standard)
      .expect("Failed to create engine");

    // Test play/pause/stop
    assert!(engine.play().is_ok());
    assert!(engine.pause().is_ok());
    assert!(engine.stop().is_ok());
  }
}

#[cfg(test)]
mod app_state_tests {
  use super::*;

  #[test]
  fn test_app_state_creation() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let engine = MultiTrackEngine::with_capacity(StemCapacity::Standard)
      .expect("Failed to create engine");

    let state = AppState::new(db, engine);

    // Verify state is accessible
    assert!(state.audio_engine.lock().is_ok());
    assert!(state.stem_id_map.lock().is_ok());
  }

  #[test]
  fn test_app_state_stem_mapping() {
    let db = Database::new_in_memory().expect("Failed to create database");
    let engine = MultiTrackEngine::with_capacity(StemCapacity::Standard)
      .expect("Failed to create engine");

    let state = AppState::new(db, engine);

    // Test stem ID mapping
    let mut map = state.stem_id_map.lock().expect("Failed to lock stem map");
    map.insert("test-stem-id".to_string(), 0);

    assert_eq!(map.get("test-stem-id"), Some(&0));
  }
}

#[cfg(test)]
mod command_logic_tests {
  use super::*;

  #[test]
  fn test_song_import_workflow() {
    let db = create_test_database();

    // Create song directly
    let song = create_test_song(&db, "Test Song");
    create_test_stem(&db, &song.id, "Vocals");
    create_test_stem(&db, &song.id, "Drums");

    // Verify song was created with stems
    let retrieved_song = db.get_song(&song.id).expect("Failed to get song");
    assert_eq!(retrieved_song.name, "Test Song");

    let stems = db.get_stems_for_song(&song.id).expect("Failed to get stems");
    assert_eq!(stems.len(), 2);
  }

  #[test]
  fn test_setlist_workflow() {
    let db = create_test_database();

    // Create songs
    let song1 = create_test_song(&db, "Song 1");
    let song2 = create_test_song(&db, "Song 2");
    let song3 = create_test_song(&db, "Song 3");

    // Create setlist
    let setlist_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let mut setlist = Setlist {
      id: setlist_id.clone(),
      name: "Sunday Service".to_string(),
      created_at: now,
      updated_at: now,
      song_ids: vec![],
    };

    db.create_setlist(&setlist).expect("Failed to create setlist");

    // Add songs to setlist
    setlist.song_ids.push(song1.id.clone());
    setlist.song_ids.push(song2.id.clone());
    db.update_setlist(&setlist).expect("Failed to update setlist");

    let retrieved = db.get_setlist(&setlist_id).expect("Failed to get setlist");
    assert_eq!(retrieved.song_ids.len(), 2);

    // Reorder songs
    setlist.song_ids = vec![song2.id.clone(), song1.id.clone(), song3.id.clone()];
    db.update_setlist(&setlist).expect("Failed to update setlist");

    let retrieved = db.get_setlist(&setlist_id).expect("Failed to get setlist");
    assert_eq!(retrieved.song_ids.len(), 3);
    assert_eq!(retrieved.song_ids[0], song2.id);
    assert_eq!(retrieved.song_ids[1], song1.id);
    assert_eq!(retrieved.song_ids[2], song3.id);
  }
}
