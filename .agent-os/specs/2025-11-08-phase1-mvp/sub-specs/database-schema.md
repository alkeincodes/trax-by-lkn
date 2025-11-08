# Database Schema

This is the database schema implementation for the spec detailed in /Users/alkein/Developments/trax/.agent-os/specs/2025-11-08-phase1-mvp/spec.md

## Overview

TraX uses SQLite for local-first data persistence. All audio files remain on the user's device in their original locations, with only metadata and file paths stored in the database.

## Schema Design

### Songs Table

Stores metadata for each song in the library.

```sql
CREATE TABLE songs (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  artist TEXT,
  duration REAL NOT NULL, -- seconds with decimal precision
  tempo INTEGER, -- BPM (beats per minute)
  key TEXT, -- musical key, e.g., "C", "Am", "F#"
  created_at INTEGER NOT NULL, -- Unix timestamp
  updated_at INTEGER NOT NULL, -- Unix timestamp
  CHECK (duration > 0),
  CHECK (tempo IS NULL OR (tempo >= 20 AND tempo <= 300))
);

CREATE INDEX idx_songs_name ON songs(name COLLATE NOCASE);
CREATE INDEX idx_songs_artist ON songs(artist COLLATE NOCASE);
CREATE INDEX idx_songs_tempo ON songs(tempo);
CREATE INDEX idx_songs_key ON songs(key);
CREATE INDEX idx_songs_created_at ON songs(created_at DESC);
```

**Rationale:**
- `id` is TEXT UUID for cross-platform compatibility and offline generation
- `duration` as REAL allows precise seeking (e.g., 243.567 seconds)
- `tempo` and `key` are nullable to support tracks without detected metadata
- Indexes on name/artist use COLLATE NOCASE for case-insensitive search
- created_at index DESC optimizes "recently added" queries

### Stems Table

Stores individual audio stem files associated with songs.

```sql
CREATE TABLE stems (
  id TEXT PRIMARY KEY NOT NULL,
  song_id TEXT NOT NULL,
  name TEXT NOT NULL, -- e.g., "Vocals", "Drums", "Bass"
  file_path TEXT NOT NULL, -- absolute path to audio file
  file_size INTEGER NOT NULL, -- bytes
  sample_rate INTEGER NOT NULL, -- Hz, e.g., 44100, 48000
  channels INTEGER NOT NULL, -- 1=mono, 2=stereo
  duration REAL NOT NULL, -- seconds
  volume REAL NOT NULL DEFAULT 0.8, -- 0.0 to 1.0
  is_muted INTEGER NOT NULL DEFAULT 0, -- boolean: 0=false, 1=true
  created_at INTEGER NOT NULL,
  FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE,
  CHECK (file_size > 0),
  CHECK (sample_rate >= 8000 AND sample_rate <= 192000),
  CHECK (channels >= 1 AND channels <= 2),
  CHECK (duration > 0),
  CHECK (volume >= 0.0 AND volume <= 1.0),
  CHECK (is_muted IN (0, 1))
);

CREATE INDEX idx_stems_song_id ON stems(song_id);
CREATE INDEX idx_stems_name ON stems(name);
CREATE UNIQUE INDEX idx_stems_file_path ON stems(file_path);
```

**Rationale:**
- `song_id` foreign key with CASCADE delete ensures orphaned stems are removed
- `file_path` is unique to prevent duplicate imports of same file
- `volume` defaults to 0.8 (80%) as a safe starting point below clipping threshold
- `is_muted` stored as INTEGER (SQLite boolean convention)
- Index on song_id for fast stem lookups when loading a song
- Unique index on file_path prevents duplicate file imports

### Setlists Table

Stores saved setlists with ordered song references.

```sql
CREATE TABLE setlists (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE(name) -- Prevent duplicate setlist names
);

CREATE INDEX idx_setlists_updated_at ON setlists(updated_at DESC);
```

**Rationale:**
- `name` is unique to prevent confusion (user can rename if needed)
- `updated_at` index DESC enables "recently used" setlist queries

### Setlist Items Table

Stores ordered song references for each setlist (join table with ordering).

```sql
CREATE TABLE setlist_items (
  id TEXT PRIMARY KEY NOT NULL,
  setlist_id TEXT NOT NULL,
  song_id TEXT NOT NULL,
  position INTEGER NOT NULL, -- 0-indexed position in setlist
  FOREIGN KEY (setlist_id) REFERENCES setlists(id) ON DELETE CASCADE,
  FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE,
  UNIQUE(setlist_id, position), -- Prevent duplicate positions
  UNIQUE(setlist_id, song_id), -- Prevent duplicate songs in same setlist
  CHECK (position >= 0)
);

CREATE INDEX idx_setlist_items_setlist_id ON setlist_items(setlist_id, position);
```

**Rationale:**
- Separate table allows flexible ordering without JSON arrays
- `position` integer provides explicit sort order
- Unique constraints prevent duplicates and position conflicts
- CASCADE delete removes items when setlist or song is deleted
- Composite index (setlist_id, position) optimizes ordered retrieval

### App Settings Table

Stores application configuration and user preferences.

```sql
CREATE TABLE app_settings (
  key TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL, -- JSON-encoded value
  updated_at INTEGER NOT NULL
);

-- Pre-populate default settings
INSERT INTO app_settings (key, value, updated_at) VALUES
  ('audio_output_device', 'null', strftime('%s', 'now')),
  ('audio_buffer_size', '1024', strftime('%s', 'now')),
  ('sample_rate', '48000', strftime('%s', 'now')),
  ('theme', '"dark"', strftime('%s', 'now'));
```

**Rationale:**
- Key-value store provides flexibility for adding settings without migrations
- `value` as TEXT supports JSON encoding for complex types (arrays, objects)
- Pre-populated defaults ensure app works immediately after first launch

## Database Migrations

### Initial Schema (Migration 001)

```rust
// migrations/001_initial_schema.sql
-- Enable foreign key constraints (must be set per connection)
PRAGMA foreign_keys = ON;

-- Create all tables as defined above
-- (Full SQL included in implementation)
```

### Migration System

Use `rusqlite` migration pattern:

```rust
// src-tauri/src/db/migrations.rs
use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &Connection) -> Result<()> {
  // Create migrations table
  conn.execute(
    "CREATE TABLE IF NOT EXISTS migrations (
      id INTEGER PRIMARY KEY,
      name TEXT NOT NULL,
      applied_at INTEGER NOT NULL
    )",
    [],
  )?;

  // Check current version
  let current_version: i32 = conn
    .query_row("SELECT COALESCE(MAX(id), 0) FROM migrations", [], |row| row.get(0))
    .unwrap_or(0);

  // Apply migrations in sequence
  if current_version < 1 {
    apply_migration_001(conn)?;
  }

  Ok(())
}

fn apply_migration_001(conn: &Connection) -> Result<()> {
  conn.execute_batch(include_str!("migrations/001_initial_schema.sql"))?;
  conn.execute(
    "INSERT INTO migrations (id, name, applied_at) VALUES (1, 'initial_schema', ?1)",
    [chrono::Utc::now().timestamp()],
  )?;
  Ok(())
}
```

## Query Patterns

### Common Queries

#### Get all songs with stem count
```sql
SELECT 
  s.id,
  s.name,
  s.artist,
  s.duration,
  s.tempo,
  s.key,
  COUNT(st.id) as stem_count
FROM songs s
LEFT JOIN stems st ON s.id = st.song_id
GROUP BY s.id
ORDER BY s.name COLLATE NOCASE;
```

#### Get song with all stems
```sql
SELECT 
  s.*,
  json_group_array(
    json_object(
      'id', st.id,
      'name', st.name,
      'filePath', st.file_path,
      'volume', st.volume,
      'isMuted', st.is_muted
    )
  ) as stems
FROM songs s
LEFT JOIN stems st ON s.id = st.song_id
WHERE s.id = ?1
GROUP BY s.id;
```

#### Get setlist with ordered songs
```sql
SELECT 
  sl.id as setlist_id,
  sl.name as setlist_name,
  s.id as song_id,
  s.name as song_name,
  s.duration,
  si.position
FROM setlists sl
JOIN setlist_items si ON sl.id = si.setlist_id
JOIN songs s ON si.song_id = s.id
WHERE sl.id = ?1
ORDER BY si.position ASC;
```

#### Search songs by name or artist
```sql
SELECT s.*, COUNT(st.id) as stem_count
FROM songs s
LEFT JOIN stems st ON s.id = st.song_id
WHERE s.name LIKE ?1 OR s.artist LIKE ?1
GROUP BY s.id
ORDER BY s.name COLLATE NOCASE
LIMIT 100;
```

#### Filter songs by tempo range
```sql
SELECT s.*, COUNT(st.id) as stem_count
FROM songs s
LEFT JOIN stems st ON s.id = st.song_id
WHERE s.tempo BETWEEN ?1 AND ?2
GROUP BY s.id
ORDER BY s.tempo ASC;
```

## Data Integrity Rules

### Foreign Key Constraints
- All foreign keys use `ON DELETE CASCADE` to maintain referential integrity
- Enable `PRAGMA foreign_keys = ON` on every connection

### Validation Checks
- `CHECK` constraints enforce valid ranges for tempo, volume, sample rate
- Application layer validates UUIDs before insertion
- File path existence validated before database insert

### Transaction Management
- Wrap bulk imports in transactions for atomicity
- Use `SAVEPOINT` for nested transactions during import error recovery
- Implement retry logic for `SQLITE_BUSY` errors (database locked)

## Performance Considerations

### Indexing Strategy
- Indexes on all foreign keys for join performance
- Indexes on searchable columns (name, artist, tempo, key)
- Composite index on (setlist_id, position) for ordered setlist retrieval
- Avoid over-indexing: no indexes on low-cardinality columns (is_muted)

### Query Optimization
- Use `EXPLAIN QUERY PLAN` to verify index usage
- Limit result sets for large libraries (pagination)
- Use prepared statements for repeated queries
- Cache frequently accessed data (e.g., current setlist) in memory

### Database Maintenance
- Run `VACUUM` periodically to reclaim space after deletions
- Use `ANALYZE` to update query optimizer statistics
- Set `PRAGMA journal_mode = WAL` for better concurrency
- Set `PRAGMA synchronous = NORMAL` for acceptable durability/performance balance

## Backup and Recovery

### Backup Strategy
- User-initiated manual backup: Copy `.db` file to external location
- Backup includes only metadata (audio files backed up separately by user)
- Future: Automated backup to user-chosen directory on app close

### Recovery Process
- Detect corrupted database on startup using `PRAGMA integrity_check`
- Attempt repair: `PRAGMA quick_check`, re-index if needed
- Last resort: Delete database, restart with fresh schema (user loses metadata, not audio files)
- Log all database errors to `logs/app.log` for troubleshooting
