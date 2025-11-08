use rusqlite::{Connection, Result, params};
use super::models::Setlist;

// Create a new setlist
pub fn create_setlist(conn: &Connection, setlist: &Setlist) -> Result<()> {
  let song_ids_json = serde_json::to_string(&setlist.song_ids)
    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

  conn.execute(
    "INSERT INTO setlists (id, name, created_at, updated_at, song_ids)
     VALUES (?1, ?2, ?3, ?4, ?5)",
    params![
      setlist.id,
      setlist.name,
      setlist.created_at,
      setlist.updated_at,
      song_ids_json,
    ],
  )?;
  Ok(())
}

// Get a setlist by ID
pub fn get_setlist(conn: &Connection, id: &str) -> Result<Setlist> {
  conn.query_row(
    "SELECT id, name, created_at, updated_at, song_ids
     FROM setlists WHERE id = ?1",
    [id],
    |row| {
      let song_ids_json: String = row.get(4)?;
      let song_ids: Vec<String> = serde_json::from_str(&song_ids_json)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

      Ok(Setlist {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        song_ids,
      })
    },
  )
}

// Update a setlist
pub fn update_setlist(conn: &Connection, setlist: &Setlist) -> Result<()> {
  let updated_at = chrono::Utc::now().timestamp();
  let song_ids_json = serde_json::to_string(&setlist.song_ids)
    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

  conn.execute(
    "UPDATE setlists SET name = ?1, updated_at = ?2, song_ids = ?3
     WHERE id = ?4",
    params![
      setlist.name,
      updated_at,
      song_ids_json,
      setlist.id,
    ],
  )?;
  Ok(())
}

// Delete a setlist
pub fn delete_setlist(conn: &Connection, id: &str) -> Result<()> {
  conn.execute("DELETE FROM setlists WHERE id = ?1", [id])?;
  Ok(())
}

// List all setlists
pub fn list_setlists(conn: &Connection) -> Result<Vec<Setlist>> {
  let mut stmt = conn.prepare(
    "SELECT id, name, created_at, updated_at, song_ids
     FROM setlists ORDER BY created_at DESC"
  )?;

  let setlists = stmt.query_map([], |row| {
    let song_ids_json: String = row.get(4)?;
    let song_ids: Vec<String> = serde_json::from_str(&song_ids_json)
      .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    Ok(Setlist {
      id: row.get(0)?,
      name: row.get(1)?,
      created_at: row.get(2)?,
      updated_at: row.get(3)?,
      song_ids,
    })
  })?;

  setlists.collect()
}
