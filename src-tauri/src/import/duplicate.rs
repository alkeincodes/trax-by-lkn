use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use super::ImportError;

const HASH_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

/// Calculate SHA-256 hash of first 1MB of file + file size
/// This provides fast duplicate detection without reading entire file
pub fn calculate_file_hash(file_path: &Path) -> Result<String, ImportError> {
  // Check if file exists
  if !file_path.exists() {
    return Err(ImportError::FileNotFound(file_path.to_string_lossy().to_string()));
  }

  // Get file size
  let file_size = std::fs::metadata(file_path)
    .map_err(|e| ImportError::MetadataExtraction(format!("Failed to read file metadata: {}", e)))?
    .len();

  // Open file
  let file = File::open(file_path)
    .map_err(|e| ImportError::MetadataExtraction(format!("Failed to open file: {}", e)))?;

  let mut reader = BufReader::new(file);
  let mut hasher = Sha256::new();

  // Read first 1MB (or entire file if smaller)
  let mut buffer = vec![0u8; HASH_BUFFER_SIZE];
  let bytes_read = reader.read(&mut buffer)
    .map_err(|e| ImportError::MetadataExtraction(format!("Failed to read file: {}", e)))?;

  // Hash the data
  hasher.update(&buffer[..bytes_read]);

  // Also include file size in hash to differentiate files with same start
  hasher.update(file_size.to_le_bytes());

  // Get final hash as hex string
  let hash_result = hasher.finalize();
  Ok(format!("{:x}", hash_result))
}

/// Check if a file with this hash already exists in the database
pub fn is_duplicate(hash: &str, existing_hashes: &[String]) -> bool {
  existing_hashes.contains(&hash.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;
  use std::path::PathBuf;

  fn create_temp_file(content: &[u8]) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("test_file_{}.tmp", uuid::Uuid::new_v4()));
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content).unwrap();
    file_path
  }

  #[test]
  fn test_calculate_file_hash_identical_files() {
    let content = b"test content for hashing";
    let file1 = create_temp_file(content);
    let file2 = create_temp_file(content);

    let hash1 = calculate_file_hash(&file1).unwrap();
    let hash2 = calculate_file_hash(&file2).unwrap();

    assert_eq!(hash1, hash2);

    std::fs::remove_file(file1).ok();
    std::fs::remove_file(file2).ok();
  }

  #[test]
  fn test_calculate_file_hash_different_files() {
    let file1 = create_temp_file(b"content1");
    let file2 = create_temp_file(b"content2");

    let hash1 = calculate_file_hash(&file1).unwrap();
    let hash2 = calculate_file_hash(&file2).unwrap();

    assert_ne!(hash1, hash2);

    std::fs::remove_file(file1).ok();
    std::fs::remove_file(file2).ok();
  }

  #[test]
  fn test_calculate_file_hash_nonexistent() {
    let result = calculate_file_hash(&PathBuf::from("/nonexistent/file.txt"));
    assert!(result.is_err());
  }

  #[test]
  fn test_calculate_file_hash_large_file() {
    // Create a 2MB file
    let large_content = vec![0u8; 2 * 1024 * 1024];
    let file_path = create_temp_file(&large_content);

    let result = calculate_file_hash(&file_path);
    assert!(result.is_ok());

    std::fs::remove_file(file_path).ok();
  }

  #[test]
  fn test_is_duplicate_found() {
    let hash = "abc123".to_string();
    let existing = vec!["def456".to_string(), "abc123".to_string(), "ghi789".to_string()];

    assert!(is_duplicate(&hash, &existing));
  }

  #[test]
  fn test_is_duplicate_not_found() {
    let hash = "xyz999".to_string();
    let existing = vec!["def456".to_string(), "abc123".to_string(), "ghi789".to_string()];

    assert!(!is_duplicate(&hash, &existing));
  }

  #[test]
  fn test_is_duplicate_empty_list() {
    let hash = "abc123".to_string();
    let existing: Vec<String> = vec![];

    assert!(!is_duplicate(&hash, &existing));
  }
}
