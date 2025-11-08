use serde::{Deserialize, Serialize};

// Song model matching TypeScript interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
  pub id: String,
  pub name: String,
  pub artist: Option<String>,
  pub duration: f64,
  pub tempo: Option<f64>,
  pub key: Option<String>,
  pub time_signature: Option<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

// Stem model matching TypeScript interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stem {
  pub id: String,
  pub song_id: String,
  pub name: String,
  pub file_path: String,
  pub file_size: i64,
  pub sample_rate: i32,
  pub channels: i32,
  pub duration: f64,
  pub volume: f64,
  pub is_muted: bool,
}

// Setlist model matching TypeScript interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setlist {
  pub id: String,
  pub name: String,
  pub created_at: i64,
  pub updated_at: i64,
  pub song_ids: Vec<String>,
}

// AppSettings model matching TypeScript interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
  pub audio_output_device: Option<String>,
  pub audio_buffer_size: i32,
  pub sample_rate: i32,
  pub theme: String,
}

// Default implementation for AppSettings
impl Default for AppSettings {
  fn default() -> Self {
    AppSettings {
      audio_output_device: None,
      audio_buffer_size: 512,
      sample_rate: 48000,
      theme: "dark".to_string(),
    }
  }
}

// Filter and sorting options for song queries
#[derive(Debug, Clone, Default)]
pub struct SongFilter {
  pub search_query: Option<String>,
  pub tempo_min: Option<f64>,
  pub tempo_max: Option<f64>,
  pub key: Option<String>,
  pub sort_by: Option<SortBy>,
}

#[derive(Debug, Clone)]
pub enum SortBy {
  Name,
  Artist,
  Tempo,
  Duration,
  DateAdded,
}
