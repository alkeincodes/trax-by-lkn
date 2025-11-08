use rusqlite::{Connection, Result};

// Current schema version
pub const SCHEMA_VERSION: i32 = 1;

// Initialize the database schema
pub fn initialize_schema(conn: &Connection) -> Result<()> {
  // Enable foreign keys
  conn.execute("PRAGMA foreign_keys = ON", [])?;

  // Create migrations table if it doesn't exist
  conn.execute(
    "CREATE TABLE IF NOT EXISTS schema_migrations (
      version INTEGER PRIMARY KEY,
      applied_at INTEGER NOT NULL
    )",
    [],
  )?;

  // Check current schema version
  let current_version = get_current_version(conn)?;

  // Run migrations
  if current_version < 1 {
    run_migration_v1(conn)?;
  }

  Ok(())
}

// Get the current schema version
fn get_current_version(conn: &Connection) -> Result<i32> {
  let version: Result<i32> = conn.query_row(
    "SELECT MAX(version) FROM schema_migrations",
    [],
    |row| row.get(0),
  );

  match version {
    Ok(v) => Ok(v),
    Err(_) => Ok(0),
  }
}

// Record a migration as applied
fn record_migration(conn: &Connection, version: i32) -> Result<()> {
  conn.execute(
    "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
    [version, chrono::Utc::now().timestamp() as i32],
  )?;
  Ok(())
}

// Migration V1: Initial schema
fn run_migration_v1(conn: &Connection) -> Result<()> {
  // Create songs table
  conn.execute(
    "CREATE TABLE IF NOT EXISTS songs (
      id TEXT PRIMARY KEY NOT NULL,
      name TEXT NOT NULL,
      artist TEXT,
      duration REAL NOT NULL,
      tempo REAL,
      key TEXT,
      time_signature TEXT,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    )",
    [],
  )?;

  // Create indexes on songs table for fast search
  conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_songs_name ON songs(name COLLATE NOCASE)",
    [],
  )?;
  conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_songs_artist ON songs(artist COLLATE NOCASE)",
    [],
  )?;
  conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_songs_tempo ON songs(tempo)",
    [],
  )?;
  conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_songs_key ON songs(key)",
    [],
  )?;

  // Create stems table
  conn.execute(
    "CREATE TABLE IF NOT EXISTS stems (
      id TEXT PRIMARY KEY NOT NULL,
      song_id TEXT NOT NULL,
      name TEXT NOT NULL,
      file_path TEXT NOT NULL,
      file_size INTEGER NOT NULL,
      sample_rate INTEGER NOT NULL,
      channels INTEGER NOT NULL,
      duration REAL NOT NULL,
      volume REAL NOT NULL DEFAULT 0.8,
      is_muted INTEGER NOT NULL DEFAULT 0,
      FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
    )",
    [],
  )?;

  // Create index on stems for fast lookup by song
  conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_stems_song_id ON stems(song_id)",
    [],
  )?;

  // Create setlists table
  conn.execute(
    "CREATE TABLE IF NOT EXISTS setlists (
      id TEXT PRIMARY KEY NOT NULL,
      name TEXT NOT NULL,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL,
      song_ids TEXT NOT NULL
    )",
    [],
  )?;

  // Create settings table (single row)
  conn.execute(
    "CREATE TABLE IF NOT EXISTS settings (
      id INTEGER PRIMARY KEY CHECK (id = 1),
      audio_output_device TEXT,
      audio_buffer_size INTEGER NOT NULL DEFAULT 512,
      sample_rate INTEGER NOT NULL DEFAULT 48000,
      theme TEXT NOT NULL DEFAULT 'dark'
    )",
    [],
  )?;

  // Insert default settings row
  conn.execute(
    "INSERT OR IGNORE INTO settings (id, audio_buffer_size, sample_rate, theme)
     VALUES (1, 512, 48000, 'dark')",
    [],
  )?;

  // Record migration
  record_migration(conn, 1)?;

  Ok(())
}
