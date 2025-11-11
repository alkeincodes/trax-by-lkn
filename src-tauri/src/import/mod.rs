mod metadata;
mod stem_detection;
mod duplicate;
mod mixdown;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use rayon::prelude::*;
use crate::database::{Database, Song, Stem};

pub use metadata::{extract_metadata, AudioMetadata};
pub use stem_detection::detect_stem_name;
pub use duplicate::calculate_file_hash;

// ========================================
// ERROR TYPES
// ========================================

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
  #[error("File not found: {0}")]
  FileNotFound(String),

  #[error("Invalid format: {0}")]
  InvalidFormat(String),

  #[error("Metadata extraction failed: {0}")]
  MetadataExtraction(String),

  #[error("Database error: {0}")]
  Database(String),

  #[error("Validation error: {0}")]
  Validation(String),

  #[error("Duplicate file detected: {0}")]
  Duplicate(String),

  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),
}

// ========================================
// DATA STRUCTURES
// ========================================

/// Request to import a multi-track song
#[derive(Debug, Clone)]
pub struct ImportRequest {
  pub file_paths: Vec<PathBuf>,
  pub title: String,
  pub artist: Option<String>,
  pub key: Option<String>,
  pub time_signature: Option<String>,
}

impl ImportRequest {
  /// Validate import request
  pub fn validate(&self) -> Result<(), ImportError> {
    if self.title.trim().is_empty() {
      return Err(ImportError::Validation("Song title is required".to_string()));
    }

    if self.file_paths.is_empty() {
      return Err(ImportError::Validation("At least one audio file is required".to_string()));
    }

    Ok(())
  }
}

/// Progress information for import operation
#[derive(Debug, Clone)]
pub struct ImportProgress {
  pub total_files: usize,
  pub processed_files: usize,
  pub current_file: Option<String>,
  pub status: ImportStatus,
  pub errors: Vec<String>,
}

impl ImportProgress {
  pub fn new(total_files: usize) -> Self {
    ImportProgress {
      total_files,
      processed_files: 0,
      current_file: None,
      status: ImportStatus::Processing,
      errors: Vec::new(),
    }
  }

  pub fn percentage(&self) -> f64 {
    if self.total_files == 0 {
      return 0.0;
    }
    (self.processed_files as f64 / self.total_files as f64) * 100.0
  }

  pub fn add_error(&mut self, error: impl Into<String>) {
    self.errors.push(error.into());
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportStatus {
  Processing,
  Completed,
  Failed,
}

/// Processed file information
#[derive(Debug, Clone)]
struct ProcessedFile {
  file_path: PathBuf,
  metadata: AudioMetadata,
  stem_name: String,
  hash: String,
}

// ========================================
// FILE VALIDATION
// ========================================

/// Validate that file has supported extension
pub fn validate_file_path(file_path: &Path) -> Result<(), ImportError> {
  let extension = file_path
    .extension()
    .and_then(|e| e.to_str())
    .ok_or_else(|| ImportError::Validation("File has no extension".to_string()))?;

  let supported_extensions = ["wav", "mp3", "flac"];
  let ext_lower = extension.to_lowercase();

  if !supported_extensions.contains(&ext_lower.as_str()) {
    return Err(ImportError::InvalidFormat(
      format!("Unsupported file format: {}. Supported formats: WAV, MP3, FLAC", extension)
    ));
  }

  Ok(())
}

// ========================================
// MULTI-THREADED PROCESSING
// ========================================

/// Process multiple files concurrently using rayon
pub fn process_files_concurrently(file_paths: &[PathBuf]) -> Vec<Result<ProcessedFile, ImportError>> {
  file_paths
    .par_iter()
    .map(|file_path| {
      // Validate file extension
      validate_file_path(file_path)?;

      // Extract metadata
      let metadata = extract_metadata(file_path)?;

      // Detect stem name
      let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
      let stem_name = detect_stem_name(filename);

      // Calculate hash
      let hash = calculate_file_hash(file_path)?;

      Ok(ProcessedFile {
        file_path: file_path.clone(),
        metadata,
        stem_name,
        hash,
      })
    })
    .collect()
}

// ========================================
// STEM NAME DEDUPLICATION
// ========================================

/// Deduplicate stem names by appending numbers to duplicates
fn deduplicate_stem_names(processed_files: &mut [ProcessedFile]) {
  use std::collections::HashMap;

  // Track how many times we've seen each stem name
  let mut name_counts: HashMap<String, usize> = HashMap::new();

  // First pass: count occurrences of each stem name
  for file in processed_files.iter() {
    *name_counts.entry(file.stem_name.clone()).or_insert(0) += 1;
  }

  // Second pass: append numbers to duplicates
  let mut current_counts: HashMap<String, usize> = HashMap::new();

  for file in processed_files.iter_mut() {
    let count = name_counts.get(&file.stem_name).unwrap_or(&0);

    // Only add numbers if there are multiple stems with the same name
    if *count > 1 {
      let current_number = current_counts.entry(file.stem_name.clone()).or_insert(0);
      *current_number += 1;
      file.stem_name = format!("{} {}", file.stem_name, current_number);
    }
  }
}

// ========================================
// MAIN IMPORT FUNCTION
// ========================================

/// Import a multi-track song into the database
pub fn import_song(db: &Database, request: ImportRequest) -> Result<String, ImportError> {
  // Validate request
  request.validate()?;

  // Process files concurrently
  let results = process_files_concurrently(&request.file_paths);

  // Separate successful and failed results
  let mut processed_files = Vec::new();
  let mut errors = Vec::new();

  for result in results {
    match result {
      Ok(file) => processed_files.push(file),
      Err(e) => {
        log::warn!("Failed to process file: {}", e);
        errors.push(e.to_string());
      }
    }
  }

  // Check if we have at least one valid file
  if processed_files.is_empty() {
    return Err(ImportError::Validation(
      "No valid audio files could be processed".to_string()
    ));
  }

  // Deduplicate stem names
  deduplicate_stem_names(&mut processed_files);

  // Check for duplicates (we'll implement a simple in-memory check for now)
  // In production, this would check against existing files in database
  let hashes: Vec<String> = processed_files.iter().map(|f| f.hash.clone()).collect();
  for (i, file) in processed_files.iter().enumerate() {
    let other_hashes: Vec<String> = hashes.iter()
      .enumerate()
      .filter(|(j, _)| *j != i)
      .map(|(_, h)| h.clone())
      .collect();

    if duplicate::is_duplicate(&file.hash, &other_hashes) {
      return Err(ImportError::Duplicate(
        format!("Duplicate file detected: {}", file.file_path.display())
      ));
    }
  }

  // Calculate song duration (use longest stem)
  let song_duration = processed_files
    .iter()
    .map(|f| f.metadata.duration)
    .fold(0.0f64, |max, d| if d > max { d } else { max });

  // Create song record
  let song_id = uuid::Uuid::new_v4().to_string();
  let now = chrono::Utc::now().timestamp();

  let song = Song {
    id: song_id.clone(),
    name: request.title.clone(),
    artist: request.artist.clone(),
    duration: song_duration,
    tempo: None,
    key: request.key.clone(),
    time_signature: request.time_signature.clone(),
    mixdown_path: None, // Will be set after mixdown generation
    created_at: now,
    updated_at: now,
  };

  // Start transaction by creating song first
  db.create_song(&song)
    .map_err(|e| ImportError::Database(format!("Failed to create song: {}", e)))?;

  // Store the count and file paths before consuming the vector
  let stems_count = processed_files.len();
  let stem_file_paths: Vec<PathBuf> = processed_files.iter()
    .map(|f| f.file_path.clone())
    .collect();

  // Create stem records
  for processed_file in processed_files {
    let stem_id = uuid::Uuid::new_v4().to_string();

    let stem = Stem {
      id: stem_id,
      song_id: song_id.clone(),
      name: processed_file.stem_name,
      file_path: processed_file.file_path.to_string_lossy().to_string(),
      file_size: processed_file.metadata.file_size,
      sample_rate: processed_file.metadata.sample_rate,
      channels: processed_file.metadata.channels,
      duration: processed_file.metadata.duration,
      volume: 0.8, // Default volume
      is_muted: false,
    };

    db.create_stem(&stem)
      .map_err(|e| {
        // If stem creation fails, we should ideally rollback the song creation
        // For now, log the error
        log::error!("Failed to create stem, song may be incomplete: {}", e);
        ImportError::Database(format!("Failed to create stem: {}", e))
      })?;
  }

  // Generate mixdown from all stems
  log::info!("Generating mixdown for song '{}'...", request.title);
  let mixdown_path = match mixdown::generate_mixdown(&song_id, &stem_file_paths) {
    Ok(path) => {
      log::info!("Mixdown generated successfully: {}", path);
      Some(path)
    }
    Err(e) => {
      log::error!("Failed to generate mixdown: {}. Song will be imported without mixdown.", e);
      // Don't fail the entire import if mixdown generation fails
      None
    }
  };

  // Update song with mixdown path
  if mixdown_path.is_some() {
    let mut updated_song = song.clone();
    updated_song.mixdown_path = mixdown_path;
    db.update_song(&updated_song)
      .map_err(|e| {
        log::error!("Failed to update song with mixdown path: {}", e);
        ImportError::Database(format!("Failed to update song: {}", e))
      })?;
  }

  log::info!(
    "Successfully imported song '{}' with {} stems",
    request.title,
    stems_count
  );

  Ok(song_id)
}

// ========================================
// PROGRESS REPORTING
// ========================================

/// Import multiple songs with progress reporting
/// This function can be used with Tauri events to report progress
pub fn import_songs_with_progress<F>(
  db: &Database,
  requests: Vec<ImportRequest>,
  mut progress_callback: F,
) -> Vec<Result<String, ImportError>>
where
  F: FnMut(&ImportProgress),
{
  let total = requests.len();
  let mut progress = ImportProgress::new(total);

  let results: Vec<Result<String, ImportError>> = requests
    .into_iter()
    .enumerate()
    .map(|(i, request)| {
      progress.current_file = Some(request.title.clone());
      progress_callback(&progress);

      let result = import_song(db, request);

      progress.processed_files = i + 1;

      if let Err(ref e) = result {
        progress.add_error(e.to_string());
      }

      progress_callback(&progress);

      result
    })
    .collect();

  progress.status = if progress.errors.is_empty() {
    ImportStatus::Completed
  } else {
    ImportStatus::Failed
  };

  progress_callback(&progress);

  results
}
