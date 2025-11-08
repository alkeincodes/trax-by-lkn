use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::fs;

// Get the database file path based on platform
pub fn get_database_path() -> PathBuf {
  #[cfg(target_os = "macos")]
  {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home)
      .join("Library")
      .join("Application Support")
      .join("com.lkn.trax")
      .join("trax.db")
  }

  #[cfg(target_os = "windows")]
  {
    let appdata = std::env::var("APPDATA").expect("APPDATA environment variable not set");
    PathBuf::from(appdata)
      .join("lkn")
      .join("trax")
      .join("trax.db")
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home)
      .join(".local")
      .join("share")
      .join("trax")
      .join("trax.db")
  }
}

// Create database connection with proper configuration
pub fn create_connection(path: &PathBuf) -> Result<Connection> {
  // Ensure parent directory exists
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)
      .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
  }

  let conn = Connection::open(path)?;

  // Enable foreign keys
  conn.execute("PRAGMA foreign_keys = ON", [])?;

  // Set WAL mode for better concurrency
  conn.execute("PRAGMA journal_mode = WAL", [])?;

  Ok(conn)
}

// Create an in-memory database connection for testing
pub fn create_in_memory_connection() -> Result<Connection> {
  let conn = Connection::open_in_memory()?;

  // Enable foreign keys
  conn.execute("PRAGMA foreign_keys = ON", [])?;

  Ok(conn)
}
