// Song model matching Rust backend
export interface Song {
  id: string
  name: string
  artist: string | null
  duration: number
  tempo: number | null
  key: string | null
  time_signature: string | null
  mixdown_path: string | null
  created_at: number
  updated_at: number
}

// Stem model matching Rust backend
export interface Stem {
  id: string
  song_id: string
  name: string
  file_path: string
  file_size: number
  sample_rate: number
  channels: number
  duration: number
  volume: number
  is_muted: boolean
  display_order: number
  level?: number // Peak audio level (0.0 to 1.0+), updated in real-time
  is_solo?: boolean // Solo state (frontend only, not persisted)
}

// Filter options for library queries
export interface SongFilter {
  search_query?: string
  tempo_min?: number
  tempo_max?: number
  key?: string
  sort_by?: SortBy
}

// Sort options matching backend enum
export enum SortBy {
  Name = 'name',
  Artist = 'artist',
  Tempo = 'tempo',
  Duration = 'duration',
  DateAdded = 'date_added',
}

// Import request payload
export interface ImportRequest {
  file_paths: string[]
  title: string
  artist?: string
  key?: string
  time_signature?: string
}

// Import progress event payload
export interface ImportProgress {
  current_file: string
  total_files: number
  processed_files: number
  status: 'scanning' | 'importing' | 'complete' | 'error'
  error?: string
}

// Setlist model matching Rust backend
export interface Setlist {
  id: string
  name: string
  created_at: number
  updated_at: number
  song_ids: string[]
}

// Audio device model matching Rust backend
export interface AudioDevice {
  name: string
  is_default: boolean
}
