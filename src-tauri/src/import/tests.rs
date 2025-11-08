use super::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

// ========================================
// HELPER FUNCTIONS FOR TESTS
// ========================================

fn create_test_directory() -> PathBuf {
  let test_dir = std::env::temp_dir().join(format!("trax_import_test_{}", uuid::Uuid::new_v4()));
  fs::create_dir_all(&test_dir).unwrap();
  test_dir
}

fn cleanup_test_directory(path: &PathBuf) {
  let _ = fs::remove_dir_all(path);
}

fn create_test_audio_file(dir: &PathBuf, filename: &str, content: &[u8]) -> PathBuf {
  let file_path = dir.join(filename);
  let mut file = File::create(&file_path).unwrap();
  file.write_all(content).unwrap();
  file_path
}

// Create a minimal valid WAV file for testing with unique content
fn create_minimal_wav_file(dir: &PathBuf, filename: &str) -> PathBuf {
  // Use filename hash to create unique sample data
  let mut sample_data = vec![0u8; 8];
  for (i, byte) in filename.bytes().enumerate() {
    sample_data[i % 8] ^= byte;
  }

  // Minimal WAV file with unique sample data
  let mut wav_data = vec![
    // RIFF header
    0x52, 0x49, 0x46, 0x46, // "RIFF"
    0x2C, 0x00, 0x00, 0x00, // file size - 8 (44 + 8 - 8 = 44)
    0x57, 0x41, 0x56, 0x45, // "WAVE"
    // fmt chunk
    0x66, 0x6D, 0x74, 0x20, // "fmt "
    0x10, 0x00, 0x00, 0x00, // chunk size
    0x01, 0x00, // audio format (PCM)
    0x02, 0x00, // num channels (stereo)
    0x44, 0xAC, 0x00, 0x00, // sample rate (44100)
    0x10, 0xB1, 0x02, 0x00, // byte rate
    0x04, 0x00, // block align
    0x10, 0x00, // bits per sample
    // data chunk
    0x64, 0x61, 0x74, 0x61, // "data"
    0x08, 0x00, 0x00, 0x00, // data size (8 bytes)
  ];
  wav_data.extend_from_slice(&sample_data);
  create_test_audio_file(dir, filename, &wav_data)
}

// ========================================
// METADATA EXTRACTION TESTS
// ========================================

#[test]
fn test_extract_metadata_valid_wav() {
  let test_dir = create_test_directory();
  let file_path = create_minimal_wav_file(&test_dir, "test.wav");

  let result = extract_metadata(&file_path);
  assert!(result.is_ok(), "Should successfully extract metadata from valid WAV");

  let metadata = result.unwrap();
  assert_eq!(metadata.sample_rate, 44100);
  assert_eq!(metadata.channels, 2);
  assert!(metadata.duration > 0.0);
  assert!(metadata.file_size > 0);

  cleanup_test_directory(&test_dir);
}

#[test]
fn test_extract_metadata_missing_file() {
  let result = extract_metadata(&PathBuf::from("/nonexistent/file.wav"));
  assert!(result.is_err(), "Should fail for missing file");
}

#[test]
fn test_extract_metadata_corrupted_file() {
  let test_dir = create_test_directory();
  let corrupted_file = create_test_audio_file(&test_dir, "corrupted.wav", b"not a valid wav file");

  let result = extract_metadata(&corrupted_file);
  assert!(result.is_err(), "Should fail for corrupted file");

  cleanup_test_directory(&test_dir);
}

#[test]
fn test_extract_metadata_empty_file() {
  let test_dir = create_test_directory();
  let empty_file = create_test_audio_file(&test_dir, "empty.wav", &[]);

  let result = extract_metadata(&empty_file);
  assert!(result.is_err(), "Should fail for empty file");

  cleanup_test_directory(&test_dir);
}

// ========================================
// STEM NAME DETECTION TESTS
// ========================================

#[test]
fn test_detect_stem_name_with_dash_separator() {
  assert_eq!(detect_stem_name("Song Name - Vocals.wav"), "Vocals");
  assert_eq!(detect_stem_name("Amazing Song - Drums.mp3"), "Drums");
  assert_eq!(detect_stem_name("Track 01 - Bass.flac"), "Bass");
}

#[test]
fn test_detect_stem_name_with_underscore_separator() {
  assert_eq!(detect_stem_name("Song_Vocals.wav"), "Vocals");
  assert_eq!(detect_stem_name("Track_Drums.mp3"), "Drums");
  assert_eq!(detect_stem_name("Music_Bass.flac"), "Bass");
}

#[test]
fn test_detect_stem_name_with_parentheses() {
  assert_eq!(detect_stem_name("Song Name (Vocals).wav"), "Vocals");
  assert_eq!(detect_stem_name("Track (Drums).mp3"), "Drums");
  assert_eq!(detect_stem_name("Music (Bass).flac"), "Bass");
}

#[test]
fn test_detect_stem_name_simple_filename() {
  assert_eq!(detect_stem_name("vocals.wav"), "Vocals");
  assert_eq!(detect_stem_name("drums.mp3"), "Drums");
  assert_eq!(detect_stem_name("bass.flac"), "Bass");
}

#[test]
fn test_detect_stem_name_all_keywords() {
  let keywords = vec![
    ("vocals.wav", "Vocals"),
    ("vox.wav", "Vox"),
    ("drums.wav", "Drums"),
    ("bass.wav", "Bass"),
    ("keys.wav", "Keys"),
    ("keyboard.wav", "Keyboard"),
    ("piano.wav", "Piano"),
    ("guitar.wav", "Guitar"),
    ("synth.wav", "Synth"),
    ("pad.wav", "Pad"),
    ("strings.wav", "Strings"),
    ("orchestra.wav", "Orchestra"),
    ("click.wav", "Click"),
    ("guide.wav", "Guide"),
  ];

  for (filename, expected) in keywords {
    assert_eq!(detect_stem_name(filename), expected);
  }
}

#[test]
fn test_detect_stem_name_fallback_to_filename() {
  assert_eq!(detect_stem_name("unknown_stem.wav"), "Unknown_stem");
  assert_eq!(detect_stem_name("my_custom_name.mp3"), "My_custom_name");
  // Numbers at the end get trimmed by clean_filename
  assert_eq!(detect_stem_name("weird123.flac"), "Weird");
}

#[test]
fn test_detect_stem_name_case_insensitive() {
  assert_eq!(detect_stem_name("VOCALS.wav"), "Vocals");
  assert_eq!(detect_stem_name("DrUmS.mp3"), "Drums");
  assert_eq!(detect_stem_name("BaSs.flac"), "Bass");
}

#[test]
fn test_detect_stem_name_with_numbers() {
  assert_eq!(detect_stem_name("vocals_01.wav"), "Vocals");
  assert_eq!(detect_stem_name("drums_02.mp3"), "Drums");
  assert_eq!(detect_stem_name("Song - Vocals 1.wav"), "Vocals");
}

// ========================================
// DUPLICATE DETECTION TESTS
// ========================================

#[test]
fn test_calculate_file_hash_identical_files() {
  let test_dir = create_test_directory();
  let file1 = create_minimal_wav_file(&test_dir, "file1.wav");
  // Create a true copy to ensure identical content
  let file2 = test_dir.join("file1_copy.wav");
  std::fs::copy(&file1, &file2).unwrap();

  let hash1 = calculate_file_hash(&file1).unwrap();
  let hash2 = calculate_file_hash(&file2).unwrap();

  assert_eq!(hash1, hash2, "Identical files should have same hash");

  cleanup_test_directory(&test_dir);
}

#[test]
fn test_calculate_file_hash_different_files() {
  let test_dir = create_test_directory();
  let file1 = create_test_audio_file(&test_dir, "file1.wav", b"content1");
  let file2 = create_test_audio_file(&test_dir, "file2.wav", b"content2");

  let hash1 = calculate_file_hash(&file1).unwrap();
  let hash2 = calculate_file_hash(&file2).unwrap();

  assert_ne!(hash1, hash2, "Different files should have different hashes");

  cleanup_test_directory(&test_dir);
}

#[test]
fn test_calculate_file_hash_missing_file() {
  let result = calculate_file_hash(&PathBuf::from("/nonexistent/file.wav"));
  assert!(result.is_err(), "Should fail for missing file");
}

#[test]
fn test_calculate_file_hash_large_file() {
  let test_dir = create_test_directory();
  // Create a file larger than 1MB to test partial hashing
  let large_data = vec![0u8; 2_000_000]; // 2MB
  let file_path = create_test_audio_file(&test_dir, "large.wav", &large_data);

  let result = calculate_file_hash(&file_path);
  assert!(result.is_ok(), "Should successfully hash large file");

  cleanup_test_directory(&test_dir);
}

// ========================================
// IMPORT DATA STRUCTURE TESTS
// ========================================

#[test]
fn test_import_request_validation_valid() {
  let request = ImportRequest {
    file_paths: vec![PathBuf::from("song1.wav"), PathBuf::from("song2.wav")],
    title: "Test Song".to_string(),
    artist: Some("Test Artist".to_string()),
    key: Some("C".to_string()),
    time_signature: Some("4/4".to_string()),
  };

  let result = request.validate();
  assert!(result.is_ok(), "Valid request should pass validation");
}

#[test]
fn test_import_request_validation_missing_title() {
  let request = ImportRequest {
    file_paths: vec![PathBuf::from("song.wav")],
    title: "".to_string(),
    artist: None,
    key: None,
    time_signature: None,
  };

  let result = request.validate();
  assert!(result.is_err(), "Should fail validation with empty title");
  assert!(result.unwrap_err().to_string().contains("title"));
}

#[test]
fn test_import_request_validation_no_files() {
  let request = ImportRequest {
    file_paths: vec![],
    title: "Test Song".to_string(),
    artist: None,
    key: None,
    time_signature: None,
  };

  let result = request.validate();
  assert!(result.is_err(), "Should fail validation with no files");
  assert!(result.unwrap_err().to_string().contains("file"));
}

#[test]
fn test_import_request_optional_fields() {
  let request = ImportRequest {
    file_paths: vec![PathBuf::from("song.wav")],
    title: "Test Song".to_string(),
    artist: None,
    key: None,
    time_signature: None,
  };

  let result = request.validate();
  assert!(result.is_ok(), "Optional fields should not be required");
}

// ========================================
// PROGRESS REPORTING TESTS
// ========================================

#[test]
fn test_import_progress_creation() {
  let progress = ImportProgress {
    total_files: 10,
    processed_files: 5,
    current_file: Some("vocals.wav".to_string()),
    status: ImportStatus::Processing,
    errors: vec![],
  };

  assert_eq!(progress.total_files, 10);
  assert_eq!(progress.processed_files, 5);
  assert_eq!(progress.percentage(), 50.0);
}

#[test]
fn test_import_progress_percentage_calculation() {
  let mut progress = ImportProgress::new(100);

  assert_eq!(progress.percentage(), 0.0);

  progress.processed_files = 25;
  assert_eq!(progress.percentage(), 25.0);

  progress.processed_files = 50;
  assert_eq!(progress.percentage(), 50.0);

  progress.processed_files = 100;
  assert_eq!(progress.percentage(), 100.0);
}

#[test]
fn test_import_progress_error_tracking() {
  let mut progress = ImportProgress::new(5);

  progress.add_error("File 1 corrupted");
  progress.add_error("File 2 not found");

  assert_eq!(progress.errors.len(), 2);
  assert!(progress.errors[0].contains("File 1"));
  assert!(progress.errors[1].contains("File 2"));
}

// ========================================
// ERROR HANDLING TESTS
// ========================================

#[test]
fn test_import_error_metadata_extraction() {
  let error = ImportError::MetadataExtraction("Failed to read file".to_string());
  assert!(error.to_string().contains("Metadata") || error.to_string().contains("Failed"));
}

#[test]
fn test_import_error_invalid_format() {
  let error = ImportError::InvalidFormat("Not a WAV file".to_string());
  assert!(error.to_string().contains("Invalid format"));
}

#[test]
fn test_import_error_database() {
  let error = ImportError::Database("Insert failed".to_string());
  assert!(error.to_string().contains("Database"));
}

// ========================================
// FILE VALIDATION TESTS
// ========================================

#[test]
fn test_validate_file_path_valid_extensions() {
  let valid_files = vec![
    "song.wav",
    "track.mp3",
    "audio.flac",
    "VOCALS.WAV",
    "drums.MP3",
    "bass.FlAc",
  ];

  for filename in valid_files {
    let result = validate_file_path(&PathBuf::from(filename));
    assert!(result.is_ok(), "Should accept {} as valid extension", filename);
  }
}

#[test]
fn test_validate_file_path_invalid_extensions() {
  let invalid_files = vec![
    "song.ogg",
    "track.aac",
    "audio.m4a",
    "vocals.txt",
    "drums.pdf",
  ];

  for filename in invalid_files {
    let result = validate_file_path(&PathBuf::from(filename));
    assert!(result.is_err(), "Should reject {} as invalid extension", filename);
  }
}

#[test]
fn test_validate_file_path_no_extension() {
  let result = validate_file_path(&PathBuf::from("noextension"));
  assert!(result.is_err(), "Should reject file without extension");
}

// ========================================
// MULTI-THREADED PROCESSING TESTS
// ========================================

#[test]
fn test_process_files_concurrently() {
  let test_dir = create_test_directory();
  let files: Vec<PathBuf> = (0..5)
    .map(|i| create_minimal_wav_file(&test_dir, &format!("song_{}.wav", i)))
    .collect();

  let results = process_files_concurrently(&files);

  assert_eq!(results.len(), 5);
  for result in results {
    assert!(result.is_ok(), "Should successfully process all files");
  }

  cleanup_test_directory(&test_dir);
}

#[test]
fn test_process_files_with_errors() {
  let test_dir = create_test_directory();
  let mut files = vec![
    create_minimal_wav_file(&test_dir, "valid.wav"),
    create_test_audio_file(&test_dir, "corrupted.wav", b"invalid"),
    create_minimal_wav_file(&test_dir, "valid2.wav"),
  ];
  files.push(PathBuf::from("/nonexistent/file.wav"));

  let results = process_files_concurrently(&files);

  assert_eq!(results.len(), 4);
  let successes = results.iter().filter(|r| r.is_ok()).count();
  let failures = results.iter().filter(|r| r.is_err()).count();

  assert_eq!(successes, 2, "Should have 2 successful processes");
  assert_eq!(failures, 2, "Should have 2 failures");

  cleanup_test_directory(&test_dir);
}

// ========================================
// INTEGRATION TESTS
// ========================================

#[test]
fn test_import_song_end_to_end() {
  let test_dir = create_test_directory();
  let db = crate::database::Database::new_in_memory().unwrap();

  let files = vec![
    create_minimal_wav_file(&test_dir, "Test Song - Vocals.wav"),
    create_minimal_wav_file(&test_dir, "Test Song - Drums.wav"),
    create_minimal_wav_file(&test_dir, "Test Song - Bass.wav"),
  ];

  let request = ImportRequest {
    file_paths: files,
    title: "Test Song".to_string(),
    artist: Some("Test Artist".to_string()),
    key: Some("C".to_string()),
    time_signature: Some("4/4".to_string()),
  };

  let result = import_song(&db, request);
  if let Err(ref e) = result {
    eprintln!("Import failed: {:?}", e);
  }
  assert!(result.is_ok(), "Should successfully import song: {:?}", result.as_ref().err());

  let song_id = result.unwrap();
  let song = db.get_song(&song_id).unwrap();

  assert_eq!(song.name, "Test Song");
  assert_eq!(song.artist, Some("Test Artist".to_string()));
  assert_eq!(song.key, Some("C".to_string()));
  assert_eq!(song.time_signature, Some("4/4".to_string()));

  let stems = db.get_stems_for_song(&song_id).unwrap();
  assert_eq!(stems.len(), 3);

  let stem_names: Vec<String> = stems.iter().map(|s| s.name.clone()).collect();
  assert!(stem_names.contains(&"Vocals".to_string()));
  assert!(stem_names.contains(&"Drums".to_string()));
  assert!(stem_names.contains(&"Bass".to_string()));

  cleanup_test_directory(&test_dir);
}

#[test]
fn test_import_duplicate_detection() {
  let test_dir = create_test_directory();
  let db = crate::database::Database::new_in_memory().unwrap();

  // Create two identical files with same filename to ensure same content
  let file1 = create_minimal_wav_file(&test_dir, "identical.wav");
  // Copy file1 to create a true duplicate
  let file2 = test_dir.join("identical_copy.wav");
  std::fs::copy(&file1, &file2).unwrap();

  // Try to import both identical files in the same batch
  let request = ImportRequest {
    file_paths: vec![file1, file2],
    title: "Song with Duplicates".to_string(),
    artist: None,
    key: None,
    time_signature: None,
  };
  let result = import_song(&db, request);
  assert!(result.is_err(), "Should detect duplicate file in same batch");
  let error_msg = result.unwrap_err().to_string();
  assert!(error_msg.contains("Duplicate") || error_msg.contains("duplicate"));

  cleanup_test_directory(&test_dir);
}

#[test]
fn test_import_with_mixed_valid_invalid_files() {
  let test_dir = create_test_directory();
  let db = crate::database::Database::new_in_memory().unwrap();

  let files = vec![
    create_minimal_wav_file(&test_dir, "valid1.wav"),
    create_test_audio_file(&test_dir, "corrupted.wav", b"invalid"),
    create_minimal_wav_file(&test_dir, "valid2.wav"),
  ];

  let request = ImportRequest {
    file_paths: files,
    title: "Mixed Song".to_string(),
    artist: None,
    key: None,
    time_signature: None,
  };

  let result = import_song(&db, request);
  // Should succeed but skip corrupted file
  assert!(result.is_ok(), "Should import valid files and skip corrupted ones");

  let song_id = result.unwrap();
  let stems = db.get_stems_for_song(&song_id).unwrap();

  // Only 2 valid files should be imported
  assert_eq!(stems.len(), 2);

  cleanup_test_directory(&test_dir);
}

#[test]
fn test_import_transaction_rollback_on_error() {
  let test_dir = create_test_directory();
  let db = crate::database::Database::new_in_memory().unwrap();

  // Create request with invalid title (empty) to trigger validation error
  let files = vec![create_minimal_wav_file(&test_dir, "song.wav")];

  let request = ImportRequest {
    file_paths: files,
    title: "".to_string(), // Invalid
    artist: None,
    key: None,
    time_signature: None,
  };

  let result = import_song(&db, request);
  assert!(result.is_err(), "Should fail with invalid title");

  // Verify no song was created in database
  let songs = db.list_songs(None).unwrap();
  assert_eq!(songs.len(), 0, "Database should be unchanged after error");

  cleanup_test_directory(&test_dir);
}
