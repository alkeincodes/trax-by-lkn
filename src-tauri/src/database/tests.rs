#[cfg(test)]
mod database_tests {
  use super::super::*;
  use uuid::Uuid;

  // Helper function to create an in-memory test database
  fn create_test_db() -> Result<Database> {
    Database::new_in_memory()
  }

  // Helper function to create a test song
  fn create_test_song() -> Song {
    Song {
      id: Uuid::new_v4().to_string(),
      name: "Test Song".to_string(),
      artist: Some("Test Artist".to_string()),
      duration: 180.0,
      tempo: Some(120.0),
      key: Some("C".to_string()),
      created_at: chrono::Utc::now().timestamp(),
      updated_at: chrono::Utc::now().timestamp(),
    }
  }

  // Helper function to create a test stem
  fn create_test_stem(song_id: &str) -> Stem {
    Stem {
      id: Uuid::new_v4().to_string(),
      song_id: song_id.to_string(),
      name: "Vocals".to_string(),
      file_path: "/path/to/vocals.wav".to_string(),
      file_size: 1024000,
      sample_rate: 48000,
      channels: 2,
      duration: 180.0,
      volume: 0.8,
      is_muted: false,
    }
  }

  // Helper function to create a test setlist
  fn create_test_setlist() -> Setlist {
    Setlist {
      id: Uuid::new_v4().to_string(),
      name: "Sunday Service".to_string(),
      created_at: chrono::Utc::now().timestamp(),
      updated_at: chrono::Utc::now().timestamp(),
      song_ids: vec![],
    }
  }

  // ===========================================
  // DATABASE INITIALIZATION AND MIGRATIONS
  // ===========================================

  #[test]
  fn test_database_initialization() {
    let db = create_test_db();
    assert!(db.is_ok(), "Database should initialize successfully");
  }

  #[test]
  fn test_database_migrations_create_tables() {
    let db = create_test_db().unwrap();
    let conn = db.get_connection().unwrap();

    // Check that songs table exists
    let songs_table_exists: bool = conn
      .query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='songs'",
        [],
        |row| row.get(0),
      )
      .unwrap_or(0) > 0;
    assert!(songs_table_exists, "Songs table should exist");

    // Check that stems table exists
    let stems_table_exists: bool = conn
      .query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='stems'",
        [],
        |row| row.get(0),
      )
      .unwrap_or(0) > 0;
    assert!(stems_table_exists, "Stems table should exist");

    // Check that setlists table exists
    let setlists_table_exists: bool = conn
      .query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='setlists'",
        [],
        |row| row.get(0),
      )
      .unwrap_or(0) > 0;
    assert!(setlists_table_exists, "Setlists table should exist");

    // Check that settings table exists
    let settings_table_exists: bool = conn
      .query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='settings'",
        [],
        |row| row.get(0),
      )
      .unwrap_or(0) > 0;
    assert!(settings_table_exists, "Settings table should exist");
  }

  #[test]
  fn test_database_migrations_create_indexes() {
    let db = create_test_db().unwrap();
    let conn = db.get_connection().unwrap();

    // Check for indexes on songs table
    let indexes: Vec<String> = conn
      .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='songs'")
      .unwrap()
      .query_map([], |row| row.get(0))
      .unwrap()
      .filter_map(|r| r.ok())
      .collect();

    assert!(
      indexes.iter().any(|name| name.contains("name")),
      "Should have index on song name"
    );
    assert!(
      indexes.iter().any(|name| name.contains("artist")),
      "Should have index on artist"
    );
  }

  #[test]
  fn test_foreign_keys_enabled() {
    let db = create_test_db().unwrap();
    let conn = db.get_connection().unwrap();

    let fk_enabled: i32 = conn
      .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
      .unwrap();
    assert_eq!(fk_enabled, 1, "Foreign keys should be enabled");
  }

  #[test]
  fn test_migration_versioning() {
    let db = create_test_db().unwrap();
    let version = db.get_schema_version().unwrap();
    assert!(version > 0, "Schema version should be greater than 0");
  }

  // ===========================================
  // SONG CRUD OPERATIONS
  // ===========================================

  #[test]
  fn test_create_song() {
    let db = create_test_db().unwrap();
    let song = create_test_song();
    let result = db.create_song(&song);
    assert!(result.is_ok(), "Should create song successfully");
  }

  #[test]
  fn test_read_song() {
    let db = create_test_db().unwrap();
    let song = create_test_song();
    let song_id = song.id.clone();
    db.create_song(&song).unwrap();

    let retrieved = db.get_song(&song_id).unwrap();
    assert_eq!(retrieved.id, song_id);
    assert_eq!(retrieved.name, song.name);
    assert_eq!(retrieved.artist, song.artist);
  }

  #[test]
  fn test_update_song() {
    let db = create_test_db().unwrap();
    let mut song = create_test_song();
    db.create_song(&song).unwrap();

    song.name = "Updated Song Name".to_string();
    song.tempo = Some(140.0);
    let result = db.update_song(&song);
    assert!(result.is_ok(), "Should update song successfully");

    let updated = db.get_song(&song.id).unwrap();
    assert_eq!(updated.name, "Updated Song Name");
    assert_eq!(updated.tempo, Some(140.0));
  }

  #[test]
  fn test_delete_song() {
    let db = create_test_db().unwrap();
    let song = create_test_song();
    let song_id = song.id.clone();
    db.create_song(&song).unwrap();

    let result = db.delete_song(&song_id);
    assert!(result.is_ok(), "Should delete song successfully");

    let retrieved = db.get_song(&song_id);
    assert!(retrieved.is_err(), "Deleted song should not be found");
  }

  #[test]
  fn test_list_all_songs() {
    let db = create_test_db().unwrap();
    let song1 = create_test_song();
    let song2 = create_test_song();
    db.create_song(&song1).unwrap();
    db.create_song(&song2).unwrap();

    let songs = db.list_songs(None).unwrap();
    assert_eq!(songs.len(), 2, "Should retrieve all songs");
  }

  // ===========================================
  // STEM CRUD OPERATIONS
  // ===========================================

  #[test]
  fn test_create_stem() {
    let db = create_test_db().unwrap();
    let song = create_test_song();
    db.create_song(&song).unwrap();

    let stem = create_test_stem(&song.id);
    let result = db.create_stem(&stem);
    assert!(result.is_ok(), "Should create stem successfully");
  }

  #[test]
  fn test_read_stem() {
    let db = create_test_db().unwrap();
    let song = create_test_song();
    db.create_song(&song).unwrap();

    let stem = create_test_stem(&song.id);
    let stem_id = stem.id.clone();
    db.create_stem(&stem).unwrap();

    let retrieved = db.get_stem(&stem_id).unwrap();
    assert_eq!(retrieved.id, stem_id);
    assert_eq!(retrieved.name, stem.name);
    assert_eq!(retrieved.volume, stem.volume);
  }

  #[test]
  fn test_get_stems_for_song() {
    let db = create_test_db().unwrap();
    let song = create_test_song();
    db.create_song(&song).unwrap();

    let stem1 = create_test_stem(&song.id);
    let mut stem2 = create_test_stem(&song.id);
    stem2.name = "Drums".to_string();

    db.create_stem(&stem1).unwrap();
    db.create_stem(&stem2).unwrap();

    let stems = db.get_stems_for_song(&song.id).unwrap();
    assert_eq!(stems.len(), 2, "Should retrieve all stems for song");
  }

  #[test]
  fn test_update_stem() {
    let db = create_test_db().unwrap();
    let song = create_test_song();
    db.create_song(&song).unwrap();

    let mut stem = create_test_stem(&song.id);
    db.create_stem(&stem).unwrap();

    stem.volume = 0.5;
    stem.is_muted = true;
    let result = db.update_stem(&stem);
    assert!(result.is_ok(), "Should update stem successfully");

    let updated = db.get_stem(&stem.id).unwrap();
    assert_eq!(updated.volume, 0.5);
    assert_eq!(updated.is_muted, true);
  }

  #[test]
  fn test_delete_stem() {
    let db = create_test_db().unwrap();
    let song = create_test_song();
    db.create_song(&song).unwrap();

    let stem = create_test_stem(&song.id);
    let stem_id = stem.id.clone();
    db.create_stem(&stem).unwrap();

    let result = db.delete_stem(&stem_id);
    assert!(result.is_ok(), "Should delete stem successfully");

    let retrieved = db.get_stem(&stem_id);
    assert!(retrieved.is_err(), "Deleted stem should not be found");
  }

  #[test]
  fn test_foreign_key_constraint_on_stem() {
    let db = create_test_db().unwrap();
    let stem = create_test_stem("non-existent-song-id");

    let result = db.create_stem(&stem);
    assert!(
      result.is_err(),
      "Should fail to create stem with invalid song_id"
    );
  }

  #[test]
  fn test_cascade_delete_stems_with_song() {
    let db = create_test_db().unwrap();
    let song = create_test_song();
    db.create_song(&song).unwrap();

    let stem = create_test_stem(&song.id);
    let stem_id = stem.id.clone();
    db.create_stem(&stem).unwrap();

    // Delete the song
    db.delete_song(&song.id).unwrap();

    // Stem should also be deleted
    let retrieved = db.get_stem(&stem_id);
    assert!(
      retrieved.is_err(),
      "Stem should be deleted when song is deleted"
    );
  }

  // ===========================================
  // SETLIST CRUD OPERATIONS
  // ===========================================

  #[test]
  fn test_create_setlist() {
    let db = create_test_db().unwrap();
    let setlist = create_test_setlist();
    let result = db.create_setlist(&setlist);
    assert!(result.is_ok(), "Should create setlist successfully");
  }

  #[test]
  fn test_read_setlist() {
    let db = create_test_db().unwrap();
    let setlist = create_test_setlist();
    let setlist_id = setlist.id.clone();
    db.create_setlist(&setlist).unwrap();

    let retrieved = db.get_setlist(&setlist_id).unwrap();
    assert_eq!(retrieved.id, setlist_id);
    assert_eq!(retrieved.name, setlist.name);
  }

  #[test]
  fn test_update_setlist() {
    let db = create_test_db().unwrap();
    let mut setlist = create_test_setlist();
    db.create_setlist(&setlist).unwrap();

    setlist.name = "Updated Setlist Name".to_string();
    let result = db.update_setlist(&setlist);
    assert!(result.is_ok(), "Should update setlist successfully");

    let updated = db.get_setlist(&setlist.id).unwrap();
    assert_eq!(updated.name, "Updated Setlist Name");
  }

  #[test]
  fn test_delete_setlist() {
    let db = create_test_db().unwrap();
    let setlist = create_test_setlist();
    let setlist_id = setlist.id.clone();
    db.create_setlist(&setlist).unwrap();

    let result = db.delete_setlist(&setlist_id);
    assert!(result.is_ok(), "Should delete setlist successfully");

    let retrieved = db.get_setlist(&setlist_id);
    assert!(retrieved.is_err(), "Deleted setlist should not be found");
  }

  #[test]
  fn test_setlist_with_ordered_songs() {
    let db = create_test_db().unwrap();
    let song1 = create_test_song();
    let song2 = create_test_song();
    db.create_song(&song1).unwrap();
    db.create_song(&song2).unwrap();

    let mut setlist = create_test_setlist();
    setlist.song_ids = vec![song1.id.clone(), song2.id.clone()];
    db.create_setlist(&setlist).unwrap();

    let retrieved = db.get_setlist(&setlist.id).unwrap();
    assert_eq!(retrieved.song_ids.len(), 2);
    assert_eq!(retrieved.song_ids[0], song1.id);
    assert_eq!(retrieved.song_ids[1], song2.id);
  }

  #[test]
  fn test_list_all_setlists() {
    let db = create_test_db().unwrap();
    let setlist1 = create_test_setlist();
    let setlist2 = create_test_setlist();
    db.create_setlist(&setlist1).unwrap();
    db.create_setlist(&setlist2).unwrap();

    let setlists = db.list_setlists().unwrap();
    assert_eq!(setlists.len(), 2, "Should retrieve all setlists");
  }

  // ===========================================
  // APP SETTINGS PERSISTENCE
  // ===========================================

  #[test]
  fn test_create_default_settings() {
    let db = create_test_db().unwrap();
    let settings = db.get_settings().unwrap();

    assert_eq!(settings.audio_buffer_size, 512);
    assert_eq!(settings.sample_rate, 48000);
    assert_eq!(settings.theme, "dark");
  }

  #[test]
  fn test_update_settings() {
    let db = create_test_db().unwrap();
    let mut settings = db.get_settings().unwrap();

    settings.audio_buffer_size = 1024;
    settings.theme = "light".to_string();
    settings.audio_output_device = Some("Built-in Output".to_string());

    let result = db.update_settings(&settings);
    assert!(result.is_ok(), "Should update settings successfully");

    let updated = db.get_settings().unwrap();
    assert_eq!(updated.audio_buffer_size, 1024);
    assert_eq!(updated.theme, "light");
    assert_eq!(
      updated.audio_output_device,
      Some("Built-in Output".to_string())
    );
  }

  #[test]
  fn test_settings_single_row() {
    let db = create_test_db().unwrap();
    let conn = db.get_connection().unwrap();

    let count: i64 = conn
      .query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
      .unwrap();
    assert_eq!(count, 1, "Settings table should have exactly one row");
  }

  // ===========================================
  // SEARCH AND FILTER FUNCTIONALITY
  // ===========================================

  #[test]
  fn test_search_songs_by_name() {
    let db = create_test_db().unwrap();
    let mut song1 = create_test_song();
    song1.name = "Amazing Grace".to_string();
    let mut song2 = create_test_song();
    song2.name = "How Great Thou Art".to_string();

    db.create_song(&song1).unwrap();
    db.create_song(&song2).unwrap();

    let filter = SongFilter {
      search_query: Some("Grace".to_string()),
      tempo_min: None,
      tempo_max: None,
      key: None,
      sort_by: None,
    };
    let results = db.list_songs(Some(filter)).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Amazing Grace");
  }

  #[test]
  fn test_search_songs_by_artist() {
    let db = create_test_db().unwrap();
    let mut song1 = create_test_song();
    song1.artist = Some("Hillsong".to_string());
    let mut song2 = create_test_song();
    song2.artist = Some("Bethel Music".to_string());

    db.create_song(&song1).unwrap();
    db.create_song(&song2).unwrap();

    let filter = SongFilter {
      search_query: Some("Hillsong".to_string()),
      tempo_min: None,
      tempo_max: None,
      key: None,
      sort_by: None,
    };
    let results = db.list_songs(Some(filter)).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].artist, Some("Hillsong".to_string()));
  }

  #[test]
  fn test_filter_by_tempo_range() {
    let db = create_test_db().unwrap();
    let mut song1 = create_test_song();
    song1.tempo = Some(80.0);
    let mut song2 = create_test_song();
    song2.tempo = Some(120.0);
    let mut song3 = create_test_song();
    song3.tempo = Some(150.0);

    db.create_song(&song1).unwrap();
    db.create_song(&song2).unwrap();
    db.create_song(&song3).unwrap();

    let filter = SongFilter {
      search_query: None,
      tempo_min: Some(100.0),
      tempo_max: Some(140.0),
      key: None,
      sort_by: None,
    };
    let results = db.list_songs(Some(filter)).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].tempo, Some(120.0));
  }

  #[test]
  fn test_filter_by_key() {
    let db = create_test_db().unwrap();
    let mut song1 = create_test_song();
    song1.key = Some("C".to_string());
    let mut song2 = create_test_song();
    song2.key = Some("G".to_string());
    let mut song3 = create_test_song();
    song3.key = Some("Am".to_string());

    db.create_song(&song1).unwrap();
    db.create_song(&song2).unwrap();
    db.create_song(&song3).unwrap();

    let filter = SongFilter {
      search_query: None,
      tempo_min: None,
      tempo_max: None,
      key: Some("C".to_string()),
      sort_by: None,
    };
    let results = db.list_songs(Some(filter)).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, Some("C".to_string()));
  }

  #[test]
  fn test_combined_filters() {
    let db = create_test_db().unwrap();
    let mut song1 = create_test_song();
    song1.name = "Song in C".to_string();
    song1.artist = Some("Artist A".to_string());
    song1.tempo = Some(120.0);
    song1.key = Some("C".to_string());

    let mut song2 = create_test_song();
    song2.name = "Another Song".to_string();
    song2.artist = Some("Artist A".to_string());
    song2.tempo = Some(140.0);
    song2.key = Some("G".to_string());

    db.create_song(&song1).unwrap();
    db.create_song(&song2).unwrap();

    let filter = SongFilter {
      search_query: Some("Artist A".to_string()),
      tempo_min: Some(100.0),
      tempo_max: Some(130.0),
      key: Some("C".to_string()),
      sort_by: None,
    };
    let results = db.list_songs(Some(filter)).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Song in C");
  }

  #[test]
  fn test_sort_songs_by_name() {
    let db = create_test_db().unwrap();
    let mut song1 = create_test_song();
    song1.name = "Zebra Song".to_string();
    let mut song2 = create_test_song();
    song2.name = "Apple Song".to_string();
    let mut song3 = create_test_song();
    song3.name = "Mango Song".to_string();

    db.create_song(&song1).unwrap();
    db.create_song(&song2).unwrap();
    db.create_song(&song3).unwrap();

    let filter = SongFilter {
      search_query: None,
      tempo_min: None,
      tempo_max: None,
      key: None,
      sort_by: Some(SortBy::Name),
    };
    let results = db.list_songs(Some(filter)).unwrap();
    assert_eq!(results[0].name, "Apple Song");
    assert_eq!(results[1].name, "Mango Song");
    assert_eq!(results[2].name, "Zebra Song");
  }

  // ===========================================
  // DATA INTEGRITY TESTS
  // ===========================================

  #[test]
  fn test_timestamps_updated_on_modification() {
    let db = create_test_db().unwrap();
    let mut song = create_test_song();
    db.create_song(&song).unwrap();

    let original_updated_at = song.updated_at;

    // Wait a second to ensure timestamp changes
    std::thread::sleep(std::time::Duration::from_secs(1));

    song.name = "Updated Name".to_string();
    db.update_song(&song).unwrap();

    let updated = db.get_song(&song.id).unwrap();
    assert!(
      updated.updated_at > original_updated_at,
      "updated_at should be newer after update"
    );
  }

  #[test]
  fn test_uuid_uniqueness() {
    let db = create_test_db().unwrap();
    let song1 = create_test_song();
    let mut song2 = create_test_song();
    song2.id = song1.id.clone(); // Try to use same ID

    db.create_song(&song1).unwrap();
    let result = db.create_song(&song2);
    assert!(
      result.is_err(),
      "Should not allow duplicate UUIDs"
    );
  }
}
