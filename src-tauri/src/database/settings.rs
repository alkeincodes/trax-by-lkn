use rusqlite::{Connection, Result, params};
use super::models::AppSettings;

// Get app settings (always returns the single row)
pub fn get_settings(conn: &Connection) -> Result<AppSettings> {
  conn.query_row(
    "SELECT audio_output_device, audio_buffer_size, sample_rate, theme
     FROM settings WHERE id = 1",
    [],
    |row| {
      Ok(AppSettings {
        audio_output_device: row.get(0)?,
        audio_buffer_size: row.get(1)?,
        sample_rate: row.get(2)?,
        theme: row.get(3)?,
      })
    },
  )
}

// Update app settings
pub fn update_settings(conn: &Connection, settings: &AppSettings) -> Result<()> {
  conn.execute(
    "UPDATE settings SET audio_output_device = ?1, audio_buffer_size = ?2,
     sample_rate = ?3, theme = ?4 WHERE id = 1",
    params![
      settings.audio_output_device,
      settings.audio_buffer_size,
      settings.sample_rate,
      settings.theme,
    ],
  )?;
  Ok(())
}
