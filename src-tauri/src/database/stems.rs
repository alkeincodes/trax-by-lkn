use rusqlite::{Connection, Result, params};
use super::models::Stem;

// Create a new stem
pub fn create_stem(conn: &Connection, stem: &Stem) -> Result<()> {
  conn.execute(
    "INSERT INTO stems (id, song_id, name, file_path, file_size, sample_rate, channels, duration, volume, is_muted)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    params![
      stem.id,
      stem.song_id,
      stem.name,
      stem.file_path,
      stem.file_size,
      stem.sample_rate,
      stem.channels,
      stem.duration,
      stem.volume,
      stem.is_muted as i32,
    ],
  )?;
  Ok(())
}

// Get a stem by ID
pub fn get_stem(conn: &Connection, id: &str) -> Result<Stem> {
  conn.query_row(
    "SELECT id, song_id, name, file_path, file_size, sample_rate, channels, duration, volume, is_muted
     FROM stems WHERE id = ?1",
    [id],
    |row| {
      Ok(Stem {
        id: row.get(0)?,
        song_id: row.get(1)?,
        name: row.get(2)?,
        file_path: row.get(3)?,
        file_size: row.get(4)?,
        sample_rate: row.get(5)?,
        channels: row.get(6)?,
        duration: row.get(7)?,
        volume: row.get(8)?,
        is_muted: row.get::<_, i32>(9)? != 0,
      })
    },
  )
}

// Get all stems for a song
pub fn get_stems_for_song(conn: &Connection, song_id: &str) -> Result<Vec<Stem>> {
  let mut stmt = conn.prepare(
    "SELECT id, song_id, name, file_path, file_size, sample_rate, channels, duration, volume, is_muted
     FROM stems WHERE song_id = ?1 ORDER BY name COLLATE NOCASE ASC"
  )?;

  let stems = stmt.query_map([song_id], |row| {
    Ok(Stem {
      id: row.get(0)?,
      song_id: row.get(1)?,
      name: row.get(2)?,
      file_path: row.get(3)?,
      file_size: row.get(4)?,
      sample_rate: row.get(5)?,
      channels: row.get(6)?,
      duration: row.get(7)?,
      volume: row.get(8)?,
      is_muted: row.get::<_, i32>(9)? != 0,
    })
  })?;

  stems.collect()
}

// Update a stem
pub fn update_stem(conn: &Connection, stem: &Stem) -> Result<()> {
  conn.execute(
    "UPDATE stems SET name = ?1, file_path = ?2, file_size = ?3, sample_rate = ?4,
     channels = ?5, duration = ?6, volume = ?7, is_muted = ?8
     WHERE id = ?9",
    params![
      stem.name,
      stem.file_path,
      stem.file_size,
      stem.sample_rate,
      stem.channels,
      stem.duration,
      stem.volume,
      stem.is_muted as i32,
      stem.id,
    ],
  )?;
  Ok(())
}

// Delete a stem
pub fn delete_stem(conn: &Connection, id: &str) -> Result<()> {
  conn.execute("DELETE FROM stems WHERE id = ?1", [id])?;
  Ok(())
}
