use rusqlite::{Connection, Result, params};
use super::models::{Song, SongFilter, SortBy};

// Create a new song
pub fn create_song(conn: &Connection, song: &Song) -> Result<()> {
  conn.execute(
    "INSERT INTO songs (id, name, artist, duration, tempo, key, time_signature, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    params![
      song.id,
      song.name,
      song.artist,
      song.duration,
      song.tempo,
      song.key,
      song.time_signature,
      song.created_at,
      song.updated_at,
    ],
  )?;
  Ok(())
}

// Get a song by ID
pub fn get_song(conn: &Connection, id: &str) -> Result<Song> {
  conn.query_row(
    "SELECT id, name, artist, duration, tempo, key, time_signature, created_at, updated_at
     FROM songs WHERE id = ?1",
    [id],
    |row| {
      Ok(Song {
        id: row.get(0)?,
        name: row.get(1)?,
        artist: row.get(2)?,
        duration: row.get(3)?,
        tempo: row.get(4)?,
        key: row.get(5)?,
        time_signature: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
      })
    },
  )
}

// Update a song
pub fn update_song(conn: &Connection, song: &Song) -> Result<()> {
  let updated_at = chrono::Utc::now().timestamp();
  conn.execute(
    "UPDATE songs SET name = ?1, artist = ?2, duration = ?3, tempo = ?4, key = ?5, time_signature = ?6, updated_at = ?7
     WHERE id = ?8",
    params![
      song.name,
      song.artist,
      song.duration,
      song.tempo,
      song.key,
      song.time_signature,
      updated_at,
      song.id,
    ],
  )?;
  Ok(())
}

// Delete a song
pub fn delete_song(conn: &Connection, id: &str) -> Result<()> {
  conn.execute("DELETE FROM songs WHERE id = ?1", [id])?;
  Ok(())
}

// List songs with optional filtering and sorting
pub fn list_songs(conn: &Connection, filter: Option<SongFilter>) -> Result<Vec<Song>> {
  let mut query = String::from(
    "SELECT id, name, artist, duration, tempo, key, time_signature, created_at, updated_at FROM songs WHERE 1=1"
  );
  let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

  // Apply filters
  if let Some(ref f) = filter {
    if let Some(ref search) = f.search_query {
      query.push_str(" AND (name LIKE ?1 OR artist LIKE ?1)");
      params.push(Box::new(format!("%{}%", search)));
    }

    if let Some(tempo_min) = f.tempo_min {
      let param_num = params.len() + 1;
      query.push_str(&format!(" AND tempo >= ?{}", param_num));
      params.push(Box::new(tempo_min));
    }

    if let Some(tempo_max) = f.tempo_max {
      let param_num = params.len() + 1;
      query.push_str(&format!(" AND tempo <= ?{}", param_num));
      params.push(Box::new(tempo_max));
    }

    if let Some(ref key) = f.key {
      let param_num = params.len() + 1;
      query.push_str(&format!(" AND key = ?{}", param_num));
      params.push(Box::new(key.clone()));
    }

    // Apply sorting
    if let Some(ref sort) = f.sort_by {
      query.push_str(" ORDER BY ");
      match sort {
        SortBy::Name => query.push_str("name COLLATE NOCASE"),
        SortBy::Artist => query.push_str("artist COLLATE NOCASE"),
        SortBy::Tempo => query.push_str("tempo"),
        SortBy::Duration => query.push_str("duration"),
        SortBy::DateAdded => query.push_str("created_at DESC"),
      }
    }
  }

  let mut stmt = conn.prepare(&query)?;
  let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

  let songs = stmt.query_map(param_refs.as_slice(), |row| {
    Ok(Song {
      id: row.get(0)?,
      name: row.get(1)?,
      artist: row.get(2)?,
      duration: row.get(3)?,
      tempo: row.get(4)?,
      key: row.get(5)?,
      time_signature: row.get(6)?,
      created_at: row.get(7)?,
      updated_at: row.get(8)?,
    })
  })?;

  songs.collect()
}
