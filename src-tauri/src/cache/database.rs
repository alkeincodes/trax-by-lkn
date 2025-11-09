use super::types::{CachedAudio, CacheResult, CacheStats};
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct CacheDatabase {
  conn: Connection,
}

impl CacheDatabase {
  pub fn new(db_path: &PathBuf) -> CacheResult<Self> {
    let conn = Connection::open(db_path)?;
    let db = Self { conn };
    db.initialize_schema()?;
    Ok(db)
  }

  fn initialize_schema(&self) -> CacheResult<()> {
    self.conn.execute_batch(
      r#"
      CREATE TABLE IF NOT EXISTS audio_cache (
        song_id TEXT NOT NULL,
        stem_id TEXT NOT NULL,
        source_path TEXT NOT NULL,
        source_hash TEXT NOT NULL,
        cache_path TEXT NOT NULL,
        sample_rate INTEGER NOT NULL,
        channels INTEGER NOT NULL,
        duration_seconds REAL NOT NULL,
        decoded_at INTEGER NOT NULL,
        last_accessed INTEGER NOT NULL,
        file_size_bytes INTEGER NOT NULL,
        PRIMARY KEY (song_id, stem_id)
      );

      CREATE INDEX IF NOT EXISTS idx_last_accessed ON audio_cache(last_accessed);
      CREATE INDEX IF NOT EXISTS idx_song_id ON audio_cache(song_id);

      CREATE TABLE IF NOT EXISTS cache_stats (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        cache_hits INTEGER NOT NULL DEFAULT 0,
        cache_misses INTEGER NOT NULL DEFAULT 0,
        evictions INTEGER NOT NULL DEFAULT 0
      );

      INSERT OR IGNORE INTO cache_stats (id, cache_hits, cache_misses, evictions)
      VALUES (1, 0, 0, 0);
      "#,
    )?;
    Ok(())
  }

  /// Insert or update a cache entry
  pub fn upsert(&self, entry: &CachedAudio) -> CacheResult<()> {
    self.conn.execute(
      r#"
      INSERT INTO audio_cache (
        song_id, stem_id, source_path, source_hash, cache_path,
        sample_rate, channels, duration_seconds, decoded_at,
        last_accessed, file_size_bytes
      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
      ON CONFLICT(song_id, stem_id) DO UPDATE SET
        source_path = ?3,
        source_hash = ?4,
        cache_path = ?5,
        sample_rate = ?6,
        channels = ?7,
        duration_seconds = ?8,
        last_accessed = ?10,
        file_size_bytes = ?11
      "#,
      params![
        entry.song_id,
        entry.stem_id,
        entry.source_path.to_string_lossy().to_string(),
        entry.source_hash,
        entry.cache_path.to_string_lossy().to_string(),
        entry.sample_rate,
        entry.channels,
        entry.duration_seconds,
        entry.decoded_at,
        entry.last_accessed,
        entry.file_size_bytes,
      ],
    )?;
    Ok(())
  }

  /// Get a cache entry by song_id and stem_id
  pub fn get(&self, song_id: &str, stem_id: &str) -> CacheResult<Option<CachedAudio>> {
    let mut stmt = self.conn.prepare(
      r#"
      SELECT song_id, stem_id, source_path, source_hash, cache_path,
             sample_rate, channels, duration_seconds, decoded_at,
             last_accessed, file_size_bytes
      FROM audio_cache
      WHERE song_id = ?1 AND stem_id = ?2
      "#,
    )?;

    let result = stmt.query_row(params![song_id, stem_id], |row| {
      Ok(CachedAudio {
        song_id: row.get(0)?,
        stem_id: row.get(1)?,
        source_path: PathBuf::from(row.get::<_, String>(2)?),
        source_hash: row.get(3)?,
        cache_path: PathBuf::from(row.get::<_, String>(4)?),
        sample_rate: row.get(5)?,
        channels: row.get(6)?,
        duration_seconds: row.get(7)?,
        decoded_at: row.get(8)?,
        last_accessed: row.get(9)?,
        file_size_bytes: row.get(10)?,
      })
    });

    match result {
      Ok(entry) => Ok(Some(entry)),
      Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }

  /// Get all cache entries for a song
  pub fn get_song_entries(&self, song_id: &str) -> CacheResult<Vec<CachedAudio>> {
    let mut stmt = self.conn.prepare(
      r#"
      SELECT song_id, stem_id, source_path, source_hash, cache_path,
             sample_rate, channels, duration_seconds, decoded_at,
             last_accessed, file_size_bytes
      FROM audio_cache
      WHERE song_id = ?1
      "#,
    )?;

    let entries = stmt
      .query_map(params![song_id], |row| {
        Ok(CachedAudio {
          song_id: row.get(0)?,
          stem_id: row.get(1)?,
          source_path: PathBuf::from(row.get::<_, String>(2)?),
          source_hash: row.get(3)?,
          cache_path: PathBuf::from(row.get::<_, String>(4)?),
          sample_rate: row.get(5)?,
          channels: row.get(6)?,
          duration_seconds: row.get(7)?,
          decoded_at: row.get(8)?,
          last_accessed: row.get(9)?,
          file_size_bytes: row.get(10)?,
        })
      })?
      .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
  }

  /// Update last accessed time
  pub fn touch(&self, song_id: &str, stem_id: &str) -> CacheResult<()> {
    let now = chrono::Utc::now().timestamp();
    self.conn.execute(
      "UPDATE audio_cache SET last_accessed = ?1 WHERE song_id = ?2 AND stem_id = ?3",
      params![now, song_id, stem_id],
    )?;
    Ok(())
  }

  /// Remove a cache entry
  pub fn remove(&self, song_id: &str, stem_id: &str) -> CacheResult<()> {
    self.conn.execute(
      "DELETE FROM audio_cache WHERE song_id = ?1 AND stem_id = ?2",
      params![song_id, stem_id],
    )?;
    Ok(())
  }

  /// Get total cache size in bytes
  pub fn get_total_size(&self) -> CacheResult<u64> {
    let size: i64 = self.conn.query_row(
      "SELECT COALESCE(SUM(file_size_bytes), 0) FROM audio_cache",
      [],
      |row| row.get(0),
    )?;
    Ok(size as u64)
  }

  /// Get least recently used entries for eviction
  pub fn get_lru_entries(&self, limit: usize) -> CacheResult<Vec<CachedAudio>> {
    let mut stmt = self.conn.prepare(
      r#"
      SELECT song_id, stem_id, source_path, source_hash, cache_path,
             sample_rate, channels, duration_seconds, decoded_at,
             last_accessed, file_size_bytes
      FROM audio_cache
      ORDER BY last_accessed ASC
      LIMIT ?1
      "#,
    )?;

    let entries = stmt
      .query_map(params![limit], |row| {
        Ok(CachedAudio {
          song_id: row.get(0)?,
          stem_id: row.get(1)?,
          source_path: PathBuf::from(row.get::<_, String>(2)?),
          source_hash: row.get(3)?,
          cache_path: PathBuf::from(row.get::<_, String>(4)?),
          sample_rate: row.get(5)?,
          channels: row.get(6)?,
          duration_seconds: row.get(7)?,
          decoded_at: row.get(8)?,
          last_accessed: row.get(9)?,
          file_size_bytes: row.get(10)?,
        })
      })?
      .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
  }

  /// Get cache statistics
  pub fn get_stats(&self) -> CacheResult<CacheStats> {
    let mut stmt = self.conn.prepare(
      "SELECT COUNT(*), COALESCE(SUM(file_size_bytes), 0) FROM audio_cache",
    )?;

    let (total_entries, total_size_bytes): (usize, i64) = stmt.query_row([], |row| {
      Ok((row.get(0)?, row.get(1)?))
    })?;

    let mut stats_stmt = self.conn.prepare(
      "SELECT cache_hits, cache_misses, evictions FROM cache_stats WHERE id = 1",
    )?;

    let (cache_hits, cache_misses, evictions): (u64, u64, u64) = stats_stmt.query_row([], |row| {
      Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;

    Ok(CacheStats {
      total_entries,
      total_size_bytes: total_size_bytes as u64,
      cache_hits,
      cache_misses,
      evictions,
    })
  }

  /// Increment cache hit counter
  pub fn increment_hits(&self) -> CacheResult<()> {
    self.conn.execute(
      "UPDATE cache_stats SET cache_hits = cache_hits + 1 WHERE id = 1",
      [],
    )?;
    Ok(())
  }

  /// Increment cache miss counter
  pub fn increment_misses(&self) -> CacheResult<()> {
    self.conn.execute(
      "UPDATE cache_stats SET cache_misses = cache_misses + 1 WHERE id = 1",
      [],
    )?;
    Ok(())
  }

  /// Increment eviction counter
  pub fn increment_evictions(&self) -> CacheResult<()> {
    self.conn.execute(
      "UPDATE cache_stats SET evictions = evictions + 1 WHERE id = 1",
      [],
    )?;
    Ok(())
  }

  /// Clear all cache entries (but keep stats)
  pub fn clear(&self) -> CacheResult<()> {
    self.conn.execute("DELETE FROM audio_cache", [])?;
    Ok(())
  }
}
