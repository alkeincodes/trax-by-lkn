use super::types::CacheResult;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Calculate SHA-256 hash of a file
/// Uses buffered reading to handle large files efficiently
pub fn calculate_file_hash(path: &Path) -> CacheResult<String> {
  let file = File::open(path)?;
  let mut reader = BufReader::new(file);
  let mut hasher = Sha256::new();
  let mut buffer = [0u8; 8192]; // 8KB buffer

  loop {
    let bytes_read = reader.read(&mut buffer)?;
    if bytes_read == 0 {
      break;
    }
    hasher.update(&buffer[..bytes_read]);
  }

  let result = hasher.finalize();
  Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;
  use tempfile::NamedTempFile;

  #[test]
  fn test_file_hash_consistency() {
    // Create a temporary file with known content
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"test content").unwrap();
    temp_file.flush().unwrap();

    let path = temp_file.path();
    let hash1 = calculate_file_hash(path).unwrap();
    let hash2 = calculate_file_hash(path).unwrap();

    // Same file should produce same hash
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex characters
  }

  #[test]
  fn test_different_content_different_hash() {
    let mut temp1 = NamedTempFile::new().unwrap();
    temp1.write_all(b"content 1").unwrap();
    temp1.flush().unwrap();

    let mut temp2 = NamedTempFile::new().unwrap();
    temp2.write_all(b"content 2").unwrap();
    temp2.flush().unwrap();

    let hash1 = calculate_file_hash(temp1.path()).unwrap();
    let hash2 = calculate_file_hash(temp2.path()).unwrap();

    // Different content should produce different hashes
    assert_ne!(hash1, hash2);
  }
}
