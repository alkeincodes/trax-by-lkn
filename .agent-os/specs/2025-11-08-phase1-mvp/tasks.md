# Spec Tasks

## Tasks

- [ ] 1. Audio Engine Core Implementation
  - [ ] 1.1 Write tests for audio playback initialization and basic playback control
  - [ ] 1.2 Set up cpal audio output stream with platform-specific configuration (Core Audio for macOS, WASAPI for Windows)
  - [ ] 1.3 Implement symphonia-based audio decoder for WAV, MP3, and FLAC formats
  - [ ] 1.4 Create lock-free ring buffer for audio thread communication using crossbeam-channel
  - [ ] 1.5 Implement audio playback loop with real-time thread priority and buffer management
  - [ ] 1.6 Add sample rate conversion to normalize all sources to 48kHz output
  - [ ] 1.7 Implement pause, stop, and seek functionality with sample-accurate positioning
  - [ ] 1.8 Add crossfade on start/stop to prevent audio clicks (10-50ms)
  - [ ] 1.9 Verify all audio engine tests pass with <10ms latency

- [ ] 2. Multi-Track Stem Playback
  - [ ] 2.1 Write tests for simultaneous playback of up to 16 audio streams
  - [ ] 2.2 Implement stem synchronization to ensure all tracks play in perfect alignment
  - [ ] 2.3 Add per-stem volume control with linear-to-dB conversion
  - [ ] 2.4 Implement per-stem mute and solo functionality
  - [ ] 2.5 Create audio buffer pool for pre-loading multiple stems into memory
  - [ ] 2.6 Add buffer management to release unused stems after song changes
  - [ ] 2.7 Implement real-time volume updates without audio glitches
  - [ ] 2.8 Verify all multi-track tests pass with synchronized playback

- [ ] 3. Database Schema and Migrations
  - [ ] 3.1 Write tests for database schema creation and migration system
  - [ ] 3.2 Create SQLite database schema (songs, stems, setlists, setlist_items, app_settings tables)
  - [ ] 3.3 Implement rusqlite integration in Rust with connection pooling
  - [ ] 3.4 Add foreign key constraints and CHECK constraints for data integrity
  - [ ] 3.5 Create database migration system with version tracking
  - [ ] 3.6 Implement UUID generation for primary keys using uuid crate
  - [ ] 3.7 Add database indexes for performance (name, artist, tempo, key, foreign keys)
  - [ ] 3.8 Create pre-populated default settings in app_settings table
  - [ ] 3.9 Verify all database tests pass including constraint validation

- [ ] 4. Multi-Track Song Import System
  - [ ] 4.1 Write tests for multi-file import, metadata extraction, and stem name detection
  - [ ] 4.2 Implement file selection handling for multiple audio files (WAV, MP3, FLAC)
  - [ ] 4.3 Add multi-threaded metadata extraction using symphonia (duration, sample rate, channels, file size)
  - [ ] 4.4 Implement stem name detection from filenames (extract keywords: vocals, drums, bass, etc.)
  - [ ] 4.5 Create import data structure to hold user inputs (title, artist, key, time signature) and file metadata
  - [ ] 4.6 Add duplicate detection using SHA-256 hash of first 1MB + file size
  - [ ] 4.7 Create progress reporting system using Tauri events (import:progress)
  - [ ] 4.8 Implement error handling for corrupted or unsupported files (log and skip)
  - [ ] 4.9 Add database insertion for imported songs and stems with transaction management
  - [ ] 4.10 Verify all import tests pass including error cases and edge cases (invalid files, missing required fields)

- [ ] 5. Tauri Backend Commands
  - [ ] 5.1 Write tests for all Tauri command handlers
  - [ ] 5.2 Implement playback commands (play_song, pause_playback, stop_playback, seek_to_position)
  - [ ] 5.3 Add stem control commands (set_stem_volume, toggle_stem_mute, toggle_stem_solo)
  - [ ] 5.4 Implement library commands (import_files, get_all_songs, search_songs, filter_songs)
  - [ ] 5.5 Add setlist commands (create_setlist, get_setlist, update_setlist, delete_setlist, get_all_setlists)
  - [ ] 5.6 Create event emitter system for real-time updates (playback:position, playback:state, import:progress, audio:error)
  - [ ] 5.7 Implement shared state management using Arc<Mutex<AudioEngine>> for thread-safe access
  - [ ] 5.8 Add error handling and Result types for all commands
  - [ ] 5.9 Verify all backend command tests pass

- [ ] 6. Library UI Components (Single-Page Architecture)
  - [ ] 6.1 Write tests for library view components and interactions
  - [ ] 6.2 Create LibraryView.vue component with grid/list layout (always visible in main UI)
  - [ ] 6.3 Implement SongCard.vue component showing song name, artist, duration, stem count
  - [ ] 6.4 Add LibraryToolbar.vue with search input, filter controls, and import button
  - [ ] 6.5 Implement search functionality with debounced input (300ms)
  - [ ] 6.6 Add filter controls for tempo range, key, and date added
  - [ ] 6.7 Create Pinia store for library state management (songs, search, filters)
  - [ ] 6.8 Create ImportProgressModal.vue to show file scanning and import progress
  - [ ] 6.9 Implement file import dialog using Tauri file picker (opens modal)
  - [ ] 6.10 Add loading states and progress indicators during import (in modal)
  - [ ] 6.11 Verify all library UI tests pass

- [ ] 7. Playback UI Components
  - [ ] 7.1 Write tests for playback controls and stem mixer components
  - [ ] 7.2 Create PlaybackControls.vue with play/pause, stop, previous/next buttons
  - [ ] 7.3 Implement SeekBar.vue with draggable progress indicator and time display
  - [ ] 7.4 Add StemMixer.vue container for displaying all stems
  - [ ] 7.5 Create StemRow.vue component with name, volume slider, mute/solo buttons
  - [ ] 7.6 Implement real-time position updates using Tauri events (playback:position)
  - [ ] 7.7 Add keyboard shortcuts for playback control (space, arrows, M, S)
  - [ ] 7.8 Create Pinia store for playback state (currentSong, isPlaying, currentPosition, stems)
  - [ ] 7.9 Implement volume slider with real-time updates to backend
  - [ ] 7.10 Verify all playback UI tests pass

- [ ] 8. Setlist Builder UI (Single-Page with Modals)
  - [ ] 8.1 Write tests for setlist builder drag-and-drop and persistence
  - [ ] 8.2 Create SetlistView.vue with split-panel layout (library left, setlist right, always visible)
  - [ ] 8.3 Implement SetlistItem.vue as draggable song component with reorder capability
  - [ ] 8.4 Add SetlistToolbar.vue with new, save, load, delete buttons and setlist dropdown
  - [ ] 8.5 Create NewSetlistModal.vue for naming and creating new setlists (no page navigation)
  - [ ] 8.6 Implement HTML5 drag-and-drop for adding songs from library to setlist
  - [ ] 8.7 Add reordering functionality within setlist by dragging
  - [ ] 8.8 Implement remove song from setlist (drag to trash or delete button)
  - [ ] 8.9 Create Pinia store for setlist state (currentSetlist, allSetlists)
  - [ ] 8.10 Create Pinia store for modal state management (activeModal, modalData)
  - [ ] 8.11 Add auto-save functionality with 500ms debounce
  - [ ] 8.12 Implement recent setlists dropdown for quick access (in main UI, no navigation)
  - [ ] 8.13 Create base Modal.vue and Dialog.vue components with overlay and focus trap
  - [ ] 8.14 Create SettingsModal.vue for app configuration (audio device, buffer size, theme)
  - [ ] 8.15 Verify all setlist UI tests pass

- [ ] 9. Error Handling and Logging (Modal-Based)
  - [ ] 9.1 Write tests for error handling scenarios
  - [ ] 9.2 Create ErrorModal.vue for displaying user-friendly error messages
  - [ ] 9.3 Implement file-not-found handling (show error modal, skip to next song option)
  - [ ] 9.4 Add audio device disconnection detection and recovery UI (modal-based)
  - [ ] 9.5 Implement database corruption detection using PRAGMA integrity_check
  - [ ] 9.6 Add logging system using log and env_logger crates
  - [ ] 9.7 Create error toast notification system in Vue frontend for non-blocking errors
  - [ ] 9.8 Implement graceful degradation for missing audio files (show in UI, allow re-linking via modal)
  - [ ] 9.9 Add user-friendly error messages for all failure scenarios
  - [ ] 9.10 Verify all error handling tests pass

- [ ] 10. Performance Optimization and Polish
  - [ ] 10.1 Write tests for performance benchmarks (startup time, memory usage, latency)
  - [ ] 10.2 Optimize database queries using EXPLAIN QUERY PLAN
  - [ ] 10.3 Implement virtualization for song list (vue-virtual-scroller) for libraries > 100 songs
  - [ ] 10.4 Add lazy loading for large setlists
  - [ ] 10.5 Optimize audio buffer pool size based on available system memory
  - [ ] 10.6 Implement PRAGMA journal_mode = WAL and PRAGMA synchronous = NORMAL for SQLite performance
  - [ ] 10.7 Add startup performance improvements (defer audio engine init until first playback)
  - [ ] 10.8 Measure and optimize memory usage to stay under 500MB idle, 2GB active
  - [ ] 10.9 Run full integration tests and verify <10ms latency, zero crashes during 1-hour playback session
  - [ ] 10.10 Verify all performance tests pass
