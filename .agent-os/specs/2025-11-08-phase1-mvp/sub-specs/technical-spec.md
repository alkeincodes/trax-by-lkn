# Technical Specification

This is the technical specification for the spec detailed in /Users/alkein/Developments/trax/.agent-os/specs/2025-11-08-phase1-mvp/spec.md

## Technical Requirements

### 1. Audio Engine Architecture

#### Core Audio Processing
- Use native platform audio APIs for minimal latency:
  - **macOS**: Core Audio (AudioQueue or AVAudioEngine)
  - **Windows**: WASAPI (Windows Audio Session API) in exclusive mode
- Implement lock-free ring buffer for audio thread communication
- Separate audio processing thread with real-time priority scheduling
- Pre-load and cache audio buffers in memory to eliminate disk I/O during playback
- Support simultaneous playback of 16 audio streams with independent volume control
- Implement sample rate conversion for mixed-rate sources (target: 48kHz output)

#### Audio File Decoding
- Support formats: WAV (PCM), MP3 (via minimp3 or similar), FLAC (via libflac)
- Decode audio in background worker threads, not on audio callback thread
- Stream large files from disk using buffered reading (4096-8192 byte chunks)
- Pre-decode stems into PCM buffers for instant playback start
- Handle sample rates: 44.1kHz, 48kHz, 96kHz, 192kHz

#### Playback Controls
- Accurate seek to arbitrary positions in song (sample-accurate)
- Synchronize playback position across all stems in a song
- Support pause/resume without audio artifacts
- Implement crossfade on start/stop (10-50ms fade to prevent clicks)
- Per-stem volume control with linear-to-dB conversion for natural fader feel
- Per-stem mute and solo functionality

#### Performance Requirements
- Audio callback latency: <10ms from trigger to speaker output
- CPU usage: <20% on modern processors (Intel i5/AMD Ryzen 5 or better)
- Memory footprint: <500MB for application, up to 2GB for audio buffers (configurable)
- Zero buffer underruns during normal operation
- Graceful handling of disk I/O delays (pre-buffering)

### 2. File Import System

#### Import Process
- Recursive folder scanning for supported audio files
- Multi-threaded import for parallel metadata extraction
- Progress reporting UI with file count and completion percentage
- Error handling for corrupted or unsupported files (log and skip)
- Duplicate detection based on file hash (SHA-256 of first 1MB + file size)

#### Stem Detection Algorithm
Automatically group files as stems using these heuristics:
1. **File naming patterns**:
   - `Song Name - Vocals.wav`, `Song Name - Drums.wav`
   - `Song Name_Vocals.wav`, `Song Name_Drums.wav`
   - `Song Name (Vocals).wav`, `Song Name (Drums).wav`
2. **Folder structure**:
   - Files in same folder with matching base name
   - Subfolder per song: `/Song Name/vocals.wav`, `/Song Name/drums.wav`
3. **Common stem keywords**: vocals, vox, drums, bass, keys, keyboard, piano, guitar, synth, pad, strings, orchestra, click, guide
4. **Manual override**: Allow user to manually group/ungroup stems in UI

#### Metadata Extraction
- Song duration (in seconds, accurate to 0.01s)
- Sample rate and bit depth
- File size
- Audio channels (mono/stereo)
- Optional: BPM detection using onset detection algorithm (aubio library)
- Optional: Key detection using chromagram analysis
- Store extracted metadata in local database

### 3. Library Management

#### Data Model
```typescript
interface Song {
  id: string // UUID
  name: string
  artist?: string
  duration: number // seconds
  tempo?: number // BPM
  key?: string // e.g., "C", "Am"
  createdAt: Date
  updatedAt: Date
  stems: Stem[]
}

interface Stem {
  id: string // UUID
  songId: string // FK to Song
  name: string // e.g., "Vocals", "Drums"
  filePath: string // absolute path to audio file
  fileSize: number // bytes
  sampleRate: number // Hz
  channels: number // 1=mono, 2=stereo
  duration: number // seconds
  volume: number // 0.0 to 1.0, default 0.8
  isMuted: boolean // default false
}

interface Setlist {
  id: string // UUID
  name: string
  createdAt: Date
  updatedAt: Date
  songIds: string[] // ordered array of Song IDs
}

interface AppSettings {
  audioOutputDevice?: string
  audioBufferSize: number // samples, e.g., 512, 1024
  sampleRate: number // Hz, e.g., 48000
  theme: 'light' | 'dark'
}
```

#### Database Technology
- Use **SQLite** via Rust (rusqlite crate) for local storage
- Store database file in user data directory:
  - macOS: `~/Library/Application Support/com.lkn.trax/trax.db`
  - Windows: `%APPDATA%\lkn\trax\trax.db`
- Implement database migrations for schema versioning
- Use foreign key constraints for referential integrity
- Index on song name, artist, tempo, key for fast search

#### Search and Filter
- Full-text search on song name and artist (SQLite FTS5)
- Filter by tempo range (e.g., 60-120 BPM)
- Filter by key (e.g., "C", "Am")
- Filter by date added
- Sort by name, artist, tempo, duration, date added

### 4. Setlist Builder UI

#### User Interface Components
- **Library Panel** (left): Scrollable list/grid of songs
- **Setlist Panel** (right): Ordered list of songs in current setlist
- **Drag Handle**: Visual indicator for drag-and-drop
- **Controls**: New setlist, save, load, delete buttons

#### Drag and Drop Implementation
- Use HTML5 Drag and Drop API in Vue 3 frontend
- Visual feedback: highlight drop zone, show insertion indicator
- Allow reordering within setlist by dragging
- Allow removing song by dragging to trash icon
- Persist order in database (songIds array maintains order)

#### Setlist Persistence
- Auto-save setlist on every change (debounced 500ms)
- Store setlist name, creation date, update date
- Quick access to recent setlists (last 5)
- Setlist selection dropdown in top toolbar

### 5. Playback UI Controls

#### Transport Controls
- Large play/pause button (space bar shortcut)
- Stop button (resets position to 00:00)
- Previous/next song buttons (arrow keys)
- Seek bar with time display (current / total)
- Volume master fader (controls output volume, not individual stems)

#### Stem Mixer
- One fader row per stem showing:
  - Stem name
  - Volume slider (0-100%, default 80%)
  - Mute button (M key + number)
  - Solo button (S key + number)
  - Peak level meter (visual feedback only)
- Auto-scroll to show all stems (max 16 visible)

#### Keyboard Shortcuts
- Space: Play/Pause
- Arrow Left/Right: Previous/Next song
- M + 1-9: Mute stem 1-9
- S + 1-9: Solo stem 1-9
- Escape: Stop playback

### 6. Tauri Backend Architecture

#### Rust Command Structure
```rust
// Core playback commands
#[tauri::command]
async fn play_song(song_id: String) -> Result<(), String>

#[tauri::command]
async fn pause_playback() -> Result<(), String>

#[tauri::command]
async fn stop_playback() -> Result<(), String>

#[tauri::command]
async fn seek_to_position(seconds: f64) -> Result<(), String>

#[tauri::command]
async fn set_stem_volume(stem_id: String, volume: f32) -> Result<(), String>

#[tauri::command]
async fn toggle_stem_mute(stem_id: String) -> Result<(), String>

// Library management commands
#[tauri::command]
async fn import_files(folder_path: String) -> Result<Vec<Song>, String>

#[tauri::command]
async fn get_all_songs() -> Result<Vec<Song>, String>

#[tauri::command]
async fn search_songs(query: String) -> Result<Vec<Song>, String>

// Setlist commands
#[tauri::command]
async fn create_setlist(name: String, song_ids: Vec<String>) -> Result<Setlist, String>

#[tauri::command]
async fn get_setlist(id: String) -> Result<Setlist, String>

#[tauri::command]
async fn update_setlist(id: String, song_ids: Vec<String>) -> Result<(), String>
```

#### Event System
Use Tauri event emitter for real-time updates:
- `playback:position` - Emit current playback position every 100ms
- `playback:state` - Emit on play/pause/stop state changes
- `import:progress` - Emit import progress (percentage, file count)
- `audio:error` - Emit on audio engine errors

#### Audio Thread Management
- Spawn dedicated thread for audio processing in `lib.rs`
- Use `Arc<Mutex<AudioEngine>>` for shared state between Tauri commands and audio thread
- Use lock-free channels (crossbeam-channel) for audio thread communication
- Never block audio thread with database queries or file I/O

### 7. Vue 3 Frontend Architecture

#### Component Structure
```
src/components/
├── library/
│   ├── LibraryView.vue        # Main library container
│   ├── SongCard.vue           # Individual song card
│   ├── LibraryToolbar.vue     # Search, filter, import buttons
├── setlist/
│   ├── SetlistView.vue        # Main setlist container
│   ├── SetlistItem.vue        # Draggable song item
│   ├── SetlistToolbar.vue     # New, save, load controls
├── playback/
│   ├── PlaybackControls.vue   # Transport controls
│   ├── StemMixer.vue          # Stem volume faders
│   ├── StemRow.vue            # Individual stem control
│   ├── SeekBar.vue            # Progress bar with seeking
├── ui/
│   └── Button.vue             # Shared UI components
```

#### State Management
Use Vue 3 Composition API with Pinia for state management:
```typescript
// stores/library.ts
export const useLibraryStore = defineStore('library', {
  state: () => ({
    songs: [] as Song[],
    searchQuery: '',
    filterOptions: {},
  }),
  actions: {
    async loadSongs() {
      this.songs = await invoke('get_all_songs')
    },
    async importFiles(folderPath: string) {
      const newSongs = await invoke('import_files', { folderPath })
      this.songs.push(...newSongs)
    },
  },
})

// stores/playback.ts
export const usePlaybackStore = defineStore('playback', {
  state: () => ({
    currentSong: null as Song | null,
    isPlaying: false,
    currentPosition: 0,
    stems: [] as Stem[],
  }),
  actions: {
    async playSong(songId: string) {
      await invoke('play_song', { songId })
      this.isPlaying = true
    },
    async setStemVolume(stemId: string, volume: number) {
      await invoke('set_stem_volume', { stemId, volume })
      const stem = this.stems.find(s => s.id === stemId)
      if (stem) stem.volume = volume
    },
  },
})

// stores/setlist.ts
export const useSetlistStore = defineStore('setlist', {
  state: () => ({
    currentSetlist: null as Setlist | null,
    allSetlists: [] as Setlist[],
  }),
  actions: {
    async createSetlist(name: string, songIds: string[]) {
      const setlist = await invoke('create_setlist', { name, songIds })
      this.currentSetlist = setlist
    },
  },
})
```

#### Event Listeners
Set up Tauri event listeners in Vue composables:
```typescript
// composables/usePlaybackEvents.ts
import { listen } from '@tauri-apps/api/event'

export function usePlaybackEvents() {
  const playbackStore = usePlaybackStore()

  onMounted(async () => {
    await listen('playback:position', (event) => {
      playbackStore.currentPosition = event.payload as number
    })
    
    await listen('playback:state', (event) => {
      const state = event.payload as { isPlaying: boolean }
      playbackStore.isPlaying = state.isPlaying
    })
  })
}
```

### 8. File System Organization

#### User Data Directory Structure
```
~/Library/Application Support/com.lkn.trax/  (macOS)
%APPDATA%\lkn\trax\                          (Windows)
├── trax.db                    # SQLite database
├── logs/
│   └── app.log                # Application logs
└── cache/
    └── waveforms/             # Pre-generated waveform images (future)
```

#### Audio Files
- Audio files remain in user's chosen locations (not copied into app directory)
- Store absolute file paths in database
- Handle missing files gracefully (show error, allow re-linking)
- Support external drives (detect if drive unmounted)

### 9. Error Handling

#### Audio Engine Errors
- File not found: Show error toast, skip to next song
- Unsupported format: Log warning, exclude from import
- Audio device disconnected: Pause playback, show reconnection UI
- Buffer underrun: Log warning, attempt to recover by increasing buffer size

#### Database Errors
- Corruption: Attempt repair using SQLite PRAGMA integrity_check
- Migration failure: Rollback to previous schema, log error
- Disk full: Show error dialog with storage usage breakdown

#### UI Error Handling
- Network-agnostic: All operations local, no network errors to handle
- Invalid file paths: Validate before passing to Rust
- Concurrent modifications: Use optimistic locking or last-write-wins

### 10. Performance Optimization

#### Startup Performance
- Load database schema on app init (< 100ms)
- Lazy load song list (paginate if > 1000 songs)
- Defer audio engine initialization until first playback

#### Memory Management
- Pre-load only currently playing song's stems into RAM
- Release previous song buffers after 5 seconds of new song playback
- Configurable audio buffer pool size (default: 4 songs worth of buffers)

#### UI Responsiveness
- Use Web Workers for heavy computations (waveform generation)
- Debounce search input (300ms)
- Virtualize song list for libraries > 100 songs (vue-virtual-scroller)

## External Dependencies

### Rust (Backend)
- **tauri** (v2.x) - Desktop app framework
- **rusqlite** (v0.31+) - SQLite database interface
- **serde** / **serde_json** - Serialization for Tauri commands
- **uuid** - UUID generation for database IDs
- **crossbeam-channel** - Lock-free channels for audio thread
- **symphonia** - Pure Rust audio decoding (WAV, FLAC, MP3 support)
- **cpal** - Cross-platform audio library (abstracts Core Audio/WASAPI)
- **log** / **env_logger** - Logging framework

**Justification**: Symphonia provides a pure-Rust audio decoding solution that eliminates C library dependencies and security concerns. cpal is the de facto standard for cross-platform audio in Rust, with proven low-latency performance.

### TypeScript (Frontend)
- **vue** (v3.5+) - UI framework
- **pinia** (v2.x) - State management
- **@tauri-apps/api** - Tauri frontend bindings
- **radix-vue** - Accessible UI primitives
- **class-variance-authority** - Component variant management
- **tailwind-merge** + **clsx** - className utilities

All frontend dependencies already defined in project's package.json.

## Development Timeline (3 Months)

### Month 1: Audio Engine & File Import
- Week 1-2: Audio engine core (symphonia integration, cpal setup, playback loop)
- Week 3: File import system (folder scanning, metadata extraction, stem detection)
- Week 4: Database schema, rusqlite integration, basic CRUD operations

### Month 2: UI & Playback Controls
- Week 1: Library view (song list, search, filter UI)
- Week 2: Playback controls (transport, seek bar, volume)
- Week 3: Stem mixer UI (faders, mute/solo, real-time updates)
- Week 4: Integration testing, bug fixes

### Month 3: Setlist Management & Polish
- Week 1: Setlist builder (drag-drop, save/load)
- Week 2: Keyboard shortcuts, accessibility improvements
- Week 3: Performance optimization (memory usage, startup time)
- Week 4: Beta testing, bug fixes, documentation
