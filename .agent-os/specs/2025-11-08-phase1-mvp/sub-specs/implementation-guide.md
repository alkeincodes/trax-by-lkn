# Implementation Guide

This document provides additional architectural guidance and implementation patterns for Phase 1 MVP development.

## Project Structure

### Recommended Directory Layout

```
trax/
├── src/                          # Vue 3 frontend
│   ├── assets/
│   │   └── index.css             # TailwindCSS v4 theme
│   ├── components/
│   │   ├── library/
│   │   │   ├── LibraryView.vue
│   │   │   ├── SongCard.vue
│   │   │   └── LibraryToolbar.vue
│   │   ├── playback/
│   │   │   ├── PlaybackControls.vue
│   │   │   ├── SeekBar.vue
│   │   │   ├── StemMixer.vue
│   │   │   └── StemRow.vue
│   │   ├── setlist/
│   │   │   ├── SetlistView.vue
│   │   │   ├── SetlistItem.vue
│   │   │   └── SetlistToolbar.vue
│   │   └── ui/
│   │       ├── Button.vue
│   │       ├── Input.vue
│   │       ├── Slider.vue
│   │       └── Toast.vue
│   ├── composables/
│   │   ├── usePlaybackEvents.ts
│   │   ├── useKeyboardShortcuts.ts
│   │   └── useTauriCommands.ts
│   ├── stores/
│   │   ├── library.ts
│   │   ├── playback.ts
│   │   └── setlist.ts
│   ├── types/
│   │   ├── song.ts
│   │   ├── stem.ts
│   │   └── setlist.ts
│   ├── lib/
│   │   └── utils.ts
│   ├── App.vue
│   └── main.ts
├── src-tauri/
│   ├── src/
│   │   ├── audio/
│   │   │   ├── mod.rs             # Audio engine module
│   │   │   ├── engine.rs          # Core playback engine
│   │   │   ├── decoder.rs         # Symphonia decoder wrapper
│   │   │   ├── buffer.rs          # Ring buffer implementation
│   │   │   └── output.rs          # cpal output stream
│   │   ├── db/
│   │   │   ├── mod.rs             # Database module
│   │   │   ├── models.rs          # Struct definitions for DB entities
│   │   │   ├── queries.rs         # SQL queries and helpers
│   │   │   ├── migrations.rs      # Migration system
│   │   │   └── migrations/
│   │   │       └── 001_initial_schema.sql
│   │   ├── import/
│   │   │   ├── mod.rs             # Import module
│   │   │   ├── scanner.rs         # File system scanning
│   │   │   ├── metadata.rs        # Audio metadata extraction
│   │   │   └── stem_detector.rs   # Stem grouping logic
│   │   ├── commands/
│   │   │   ├── mod.rs             # Tauri commands module
│   │   │   ├── playback.rs        # Playback commands
│   │   │   ├── library.rs         # Library commands
│   │   │   └── setlist.rs         # Setlist commands
│   │   ├── lib.rs                 # Tauri app builder
│   │   └── main.rs                # Entry point
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── vite.config.ts
├── tsconfig.json
└── postcss.config.js
```

## Development Workflow

### Phase 1: Backend Foundation (Weeks 1-4)

#### Week 1-2: Audio Engine Core

**Objective**: Build reliable, low-latency audio playback

**Key Files to Create**:
- `src-tauri/src/audio/mod.rs`
- `src-tauri/src/audio/engine.rs`
- `src-tauri/src/audio/decoder.rs`
- `src-tauri/src/audio/output.rs`

**Implementation Steps**:
1. Set up cpal audio output stream with device enumeration
2. Integrate symphonia for audio decoding (start with WAV only)
3. Implement basic playback loop with buffer management
4. Add pause/stop/seek functionality
5. Expand format support to MP3 and FLAC
6. Optimize for <10ms latency

**Testing Strategy**:
- Unit tests for decoder (use sample audio files in `test_data/`)
- Integration tests for playback engine
- Manual latency testing with stopwatch comparison

**Acceptance Criteria**:
- WAV, MP3, FLAC files play without distortion
- Seek to arbitrary position works within 100ms
- Measured latency < 10ms on macOS and Windows test machines

#### Week 3: File Import and Stem Detection

**Objective**: Enable bulk import with intelligent stem grouping

**Key Files to Create**:
- `src-tauri/src/import/mod.rs`
- `src-tauri/src/import/scanner.rs`
- `src-tauri/src/import/metadata.rs`
- `src-tauri/src/import/stem_detector.rs`

**Implementation Steps**:
1. Implement recursive folder scanning with walkdir crate
2. Add metadata extraction using symphonia (duration, sample rate, channels)
3. Build stem detection using regex patterns on filenames
4. Implement duplicate detection with SHA-256 hashing
5. Add progress reporting via Tauri events

**Testing Strategy**:
- Unit tests with mock file system (use tempdir crate)
- Test stem detection with variety of naming patterns
- Test duplicate detection with identical files

**Acceptance Criteria**:
- Imports 1000 files in < 30 seconds
- Correctly groups 90%+ of standard stem naming patterns
- Duplicate detection prevents re-import of same files

#### Week 4: Database Layer

**Objective**: Persistent storage for songs, stems, setlists

**Key Files to Create**:
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/db/models.rs`
- `src-tauri/src/db/queries.rs`
- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/migrations/001_initial_schema.sql`

**Implementation Steps**:
1. Create SQLite schema with migrations system
2. Implement CRUD operations for songs and stems
3. Add setlist management (create, update, delete)
4. Implement search queries with full-text search
5. Add filter queries (tempo, key, date)

**Testing Strategy**:
- Unit tests for all CRUD operations
- Test foreign key constraints and cascading deletes
- Test migration system (apply, rollback)

**Acceptance Criteria**:
- All tables created with proper indexes
- Search query returns results in < 100ms for 10,000 songs
- Migration system successfully upgrades from empty DB to current schema

### Phase 2: Frontend UI (Weeks 5-8)

#### Week 5: Library View

**Objective**: Display and search song library

**Key Files to Create**:
- `src/components/library/LibraryView.vue`
- `src/components/library/SongCard.vue`
- `src/components/library/LibraryToolbar.vue`
- `src/stores/library.ts`

**Implementation Steps**:
1. Create Pinia store for library state
2. Build SongCard component with TailwindCSS styling
3. Implement LibraryView with grid layout
4. Add search input with debouncing
5. Implement filter UI for tempo, key
6. Connect to Tauri backend via invoke()

**Testing Strategy**:
- Vitest component tests for each component
- Test search debouncing behavior
- Test filter state management

**Acceptance Criteria**:
- Library displays all songs from database
- Search updates within 300ms of typing
- Filters work correctly and update results

#### Week 6: Playback Controls

**Objective**: Transport controls and seek functionality

**Key Files to Create**:
- `src/components/playback/PlaybackControls.vue`
- `src/components/playback/SeekBar.vue`
- `src/stores/playback.ts`
- `src/composables/usePlaybackEvents.ts`

**Implementation Steps**:
1. Create Pinia store for playback state
2. Build PlaybackControls with play/pause/stop buttons
3. Implement SeekBar with draggable progress
4. Add Tauri event listeners for position updates
5. Implement keyboard shortcuts
6. Add time display (current / total)

**Testing Strategy**:
- Test play/pause state transitions
- Test seek interaction
- Test keyboard shortcuts

**Acceptance Criteria**:
- Play/pause toggles correctly
- Seek bar updates smoothly (100ms intervals)
- Keyboard shortcuts work reliably

#### Week 7: Stem Mixer

**Objective**: Per-stem volume control and mute/solo

**Key Files to Create**:
- `src/components/playback/StemMixer.vue`
- `src/components/playback/StemRow.vue`
- `src/components/ui/Slider.vue`

**Implementation Steps**:
1. Build StemRow component with volume slider
2. Add mute and solo buttons
3. Implement real-time volume updates to backend
4. Add visual feedback for muted/soloed stems
5. Support up to 16 stems with scrolling

**Testing Strategy**:
- Test volume changes propagate to backend
- Test mute/solo logic (solo mutes others)
- Test keyboard shortcuts for mute/solo

**Acceptance Criteria**:
- Volume changes are instant with no lag
- Mute/solo states update correctly
- UI handles 16+ stems gracefully

#### Week 8: Setlist Builder

**Objective**: Drag-and-drop setlist creation

**Key Files to Create**:
- `src/components/setlist/SetlistView.vue`
- `src/components/setlist/SetlistItem.vue`
- `src/components/setlist/SetlistToolbar.vue`
- `src/stores/setlist.ts`

**Implementation Steps**:
1. Create Pinia store for setlist state
2. Implement drag-and-drop from library to setlist
3. Add reordering within setlist
4. Implement save/load functionality
5. Add auto-save with debouncing
6. Build recent setlists dropdown

**Testing Strategy**:
- Test drag-and-drop interactions
- Test setlist persistence
- Test auto-save debouncing

**Acceptance Criteria**:
- Drag-and-drop works smoothly
- Setlists persist across app restarts
- Auto-save doesn't spam backend

### Phase 3: Integration & Polish (Weeks 9-12)

#### Week 9: Multi-Track Integration

**Objective**: Connect stem mixer to audio engine

**Implementation Steps**:
1. Implement multi-track playback in audio engine
2. Add stem synchronization logic
3. Connect frontend stem controls to backend
4. Test with real multi-stem songs

**Acceptance Criteria**:
- All stems play in perfect sync
- Volume changes affect correct stem
- No audio dropouts with 16 stems

#### Week 10: Error Handling

**Objective**: Graceful error handling and recovery

**Implementation Steps**:
1. Add error toast notification system
2. Handle missing file paths gracefully
3. Implement audio device disconnection recovery
4. Add database corruption detection
5. Improve error messages throughout app

**Acceptance Criteria**:
- All errors show user-friendly messages
- App doesn't crash on missing files
- Database corruption triggers recovery flow

#### Week 11: Performance Optimization

**Objective**: Meet performance requirements

**Implementation Steps**:
1. Profile audio engine for latency bottlenecks
2. Optimize database queries
3. Implement virtualization for large libraries
4. Add lazy loading for setlists
5. Optimize memory usage

**Acceptance Criteria**:
- Latency consistently < 10ms
- Memory usage < 500MB idle, < 2GB active
- UI responsive with 10,000+ songs

#### Week 12: Testing & Bug Fixes

**Objective**: Prepare for release

**Implementation Steps**:
1. Run full integration test suite
2. Manual testing on macOS and Windows
3. Fix all critical and high-priority bugs
4. Write user documentation
5. Prepare release build

**Acceptance Criteria**:
- All tests pass
- Zero known critical bugs
- App runs for 1+ hours without crashes

## Key Technical Decisions

### Audio Engine: Why cpal + symphonia?

**cpal**: De facto standard for cross-platform audio in Rust. Provides low-level access to Core Audio (macOS) and WASAPI (Windows) with consistent API.

**symphonia**: Pure Rust audio decoder eliminates C library dependencies (e.g., FFmpeg, libsndfile). Better security, easier cross-compilation.

**Alternative considered**: rodio (higher-level audio library)
- **Rejected because**: Less control over latency, harder to implement multi-track sync

### Database: Why SQLite?

**Local-first architecture**: All data on user's device, no server required.

**Performance**: Fast for < 100,000 songs (well beyond expected use case).

**Simplicity**: Single file database, no server setup.

**Alternative considered**: IndexedDB (browser storage)
- **Rejected because**: Less powerful queries, Tauri backend is Rust-native

### Frontend: Why Pinia over Vuex?

**Pinia**: Official recommendation for Vue 3, simpler API, better TypeScript support.

**Composition API**: Natural fit with Vue 3 `<script setup>` syntax.

**Alternative considered**: Vuex 4
- **Rejected because**: More boilerplate, worse TypeScript inference

## Common Pitfalls to Avoid

### Audio Engine
- **Don't** perform file I/O on audio callback thread (causes dropouts)
- **Don't** use blocking locks in audio thread (use lock-free channels)
- **Don't** allocate memory in audio callback (pre-allocate buffers)

### Database
- **Don't** forget to enable foreign keys (`PRAGMA foreign_keys = ON`)
- **Don't** perform long queries on UI thread (use async/await)
- **Don't** ignore migration failures (user loses data)

### Frontend
- **Don't** call Tauri commands in tight loops (debounce/throttle)
- **Don't** store large objects in Pinia (causes reactivity overhead)
- **Don't** forget to unsubscribe from Tauri events (memory leak)

## Testing Strategy

### Unit Tests
- Rust: Use `cargo test` with mock audio devices
- TypeScript: Use Vitest for component and store tests

### Integration Tests
- Test full flow: import → library → setlist → playback
- Use real audio files in `test_data/` directory

### Manual Testing
- Test on macOS and Windows
- Test with large libraries (1000+ songs)
- Test with 16-stem songs
- Stress test: 1-hour continuous playback

## Performance Benchmarks

### Latency Target
- **Goal**: < 10ms from button click to audio output
- **Measure**: Use audio loopback with oscilloscope or audio analysis software

### Memory Target
- **Goal**: < 500MB idle, < 2GB with 16 stems loaded
- **Measure**: Use macOS Activity Monitor or Windows Task Manager

### Startup Target
- **Goal**: < 2 seconds from launch to usable UI
- **Measure**: Time from app start to first render

## Security Considerations

### File Path Validation
- Validate all file paths before passing to Rust
- Prevent directory traversal attacks (check for `..` in paths)
- Use absolute paths, not relative

### Database Injection
- Always use parameterized queries (rusqlite handles this)
- Never concatenate user input into SQL strings

### Audio File Parsing
- Symphonia handles malformed files safely
- Implement timeout for metadata extraction (prevent infinite loops)

## Deployment Checklist

- [ ] Code signing certificate configured (macOS/Windows)
- [ ] Release build optimized (`cargo build --release`)
- [ ] Version number updated in `Cargo.toml` and `package.json`
- [ ] CHANGELOG.md updated
- [ ] User documentation complete
- [ ] Installer tested on clean macOS and Windows machines
- [ ] All third-party licenses included
