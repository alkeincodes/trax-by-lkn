mod connection;
mod models;
mod schema;
mod songs;
mod stems;
mod setlists;
mod settings;

#[cfg(test)]
mod tests;

use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub use models::*;

// Database wrapper with thread-safe connection
pub struct Database {
  conn: Arc<Mutex<Connection>>,
}

impl Database {
  // Create a new database instance with file-based storage
  pub fn new() -> Result<Self> {
    let db_path = connection::get_database_path();
    let conn = connection::create_connection(&db_path)?;

    // Initialize schema and run migrations
    schema::initialize_schema(&conn)?;

    Ok(Database {
      conn: Arc::new(Mutex::new(conn)),
    })
  }

  // Create an in-memory database (for testing)
  pub fn new_in_memory() -> Result<Self> {
    let conn = connection::create_in_memory_connection()?;

    // Initialize schema and run migrations
    schema::initialize_schema(&conn)?;

    Ok(Database {
      conn: Arc::new(Mutex::new(conn)),
    })
  }

  // Get a reference to the connection (for internal use)
  pub fn get_connection(&self) -> Result<std::sync::MutexGuard<Connection>> {
    self.conn.lock()
      .map_err(|_| rusqlite::Error::InvalidQuery)
  }

  // Get current schema version
  pub fn get_schema_version(&self) -> Result<i32> {
    let conn = self.get_connection()?;
    let version: i32 = conn.query_row(
      "SELECT MAX(version) FROM schema_migrations",
      [],
      |row| row.get(0),
    )?;
    Ok(version)
  }

  // ========================================
  // SONG OPERATIONS
  // ========================================

  pub fn create_song(&self, song: &Song) -> Result<()> {
    let conn = self.get_connection()?;
    songs::create_song(&conn, song)
  }

  pub fn get_song(&self, id: &str) -> Result<Song> {
    let conn = self.get_connection()?;
    songs::get_song(&conn, id)
  }

  pub fn update_song(&self, song: &Song) -> Result<()> {
    let conn = self.get_connection()?;
    songs::update_song(&conn, song)
  }

  pub fn delete_song(&self, id: &str) -> Result<()> {
    let conn = self.get_connection()?;
    songs::delete_song(&conn, id)
  }

  pub fn list_songs(&self, filter: Option<SongFilter>) -> Result<Vec<Song>> {
    let conn = self.get_connection()?;
    songs::list_songs(&conn, filter)
  }

  // ========================================
  // STEM OPERATIONS
  // ========================================

  pub fn create_stem(&self, stem: &Stem) -> Result<()> {
    let conn = self.get_connection()?;
    stems::create_stem(&conn, stem)
  }

  pub fn get_stem(&self, id: &str) -> Result<Stem> {
    let conn = self.get_connection()?;
    stems::get_stem(&conn, id)
  }

  pub fn get_stems_for_song(&self, song_id: &str) -> Result<Vec<Stem>> {
    let conn = self.get_connection()?;
    stems::get_stems_for_song(&conn, song_id)
  }

  pub fn update_stem(&self, stem: &Stem) -> Result<()> {
    let conn = self.get_connection()?;
    stems::update_stem(&conn, stem)
  }

  pub fn delete_stem(&self, id: &str) -> Result<()> {
    let conn = self.get_connection()?;
    stems::delete_stem(&conn, id)
  }

  // ========================================
  // SETLIST OPERATIONS
  // ========================================

  pub fn create_setlist(&self, setlist: &Setlist) -> Result<()> {
    let conn = self.get_connection()?;
    setlists::create_setlist(&conn, setlist)
  }

  pub fn get_setlist(&self, id: &str) -> Result<Setlist> {
    let conn = self.get_connection()?;
    setlists::get_setlist(&conn, id)
  }

  pub fn update_setlist(&self, setlist: &Setlist) -> Result<()> {
    let conn = self.get_connection()?;
    setlists::update_setlist(&conn, setlist)
  }

  pub fn delete_setlist(&self, id: &str) -> Result<()> {
    let conn = self.get_connection()?;
    setlists::delete_setlist(&conn, id)
  }

  pub fn list_setlists(&self) -> Result<Vec<Setlist>> {
    let conn = self.get_connection()?;
    setlists::list_setlists(&conn)
  }

  // ========================================
  // SETTINGS OPERATIONS
  // ========================================

  pub fn get_settings(&self) -> Result<AppSettings> {
    let conn = self.get_connection()?;
    settings::get_settings(&conn)
  }

  pub fn update_settings(&self, settings: &AppSettings) -> Result<()> {
    let conn = self.get_connection()?;
    settings::update_settings(&conn, settings)
  }
}

// Error type for database operations
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
  #[error("Database error: {0}")]
  Rusqlite(#[from] rusqlite::Error),

  #[error("Serialization error: {0}")]
  Serialization(#[from] serde_json::Error),

  #[error("Item not found: {0}")]
  NotFound(String),
}
