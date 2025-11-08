# Spec Requirements Document

> Spec: Phase 1 MVP - Core Audio Engine & Setlist Management
> Created: 2025-11-08
> Status: Planning

## Overview

Build the foundational MVP for TraX by LKN that delivers core multi-track audio playback, setlist management, and file organization capabilities. This phase establishes the technical architecture, local-first data model, and performance-critical audio engine that will support all future features while meeting the <10ms latency requirement for live performance use.

## User Stories

### Story 1: Multi-Track Playback for Live Performance

As a worship leader, I want to play backing tracks with multiple stems during Sunday service, so that I can adjust individual instrument volumes in real-time and deliver a professional sound experience.

**Workflow:**
1. User opens TraX to the main single-page interface with library visible
2. Selects a song from their imported tracks
3. Views all available stems (vocals, drums, bass, keys, etc.) in the stem mixer panel
4. Adjusts individual stem volumes before or during playback
5. Plays track with synchronized multi-stem audio
6. Pauses, stops, or seeks to different positions seamlessly
7. Moves to next song in setlist without interruption

### Story 2: Setlist Organization for Sunday Services

As a church music director, I want to create and save setlists for different worship services, so that I can quickly access the right songs in the correct order without manual searching during time-sensitive moments.

**Workflow:**
1. User clicks "New Setlist" button which opens a modal
2. Names the setlist "Sunday Nov 10 - 9am Service" in the modal
3. Drags songs from library into setlist panel in desired order
4. Saves setlist (modal closes automatically)
5. On Sunday morning, selects saved setlist from dropdown in main UI
6. Plays through songs sequentially with one-click navigation
7. Makes live adjustments to order if needed (all within the single-page interface)
8. Setlist persists for future reference

### Story 3: Importing and Organizing Backing Track Library

As a professional musician, I want to import my collection of backing tracks and stems from external sources, so that I can manage all my performance audio in one centralized, searchable library.

**Workflow:**
1. User clicks "Import Files" button in the library toolbar
2. System file picker modal appears for folder selection
3. User selects folder containing backing tracks (WAV, MP3, FLAC files)
4. Progress modal displays showing import status
5. System automatically detects stems for each song (matching file naming patterns)
6. After import completes, files appear in library with indexed metadata (tempo, key, duration)
7. User can search, filter, and organize library by song name, artist, tempo, key (all in main UI)

## Spec Scope

1. **Audio Engine** - Real-time multi-track playback with <10ms latency supporting up to 16 simultaneous stems per song
2. **File Import System** - Bulk import of audio files (WAV, MP3, FLAC) with automatic stem detection and metadata extraction
3. **Library Management** - Local database for organizing tracks, stems, and metadata with search and filtering capabilities
4. **Setlist Builder** - Drag-and-drop interface for creating, saving, and managing multiple setlists
5. **Playback Controls** - Play, pause, stop, seek, volume control, and individual stem muting/soloing
6. **File Organization** - Automatic stem grouping based on file naming conventions and manual override options
7. **Data Persistence** - Local storage of library database, setlists, and user preferences

## Out of Scope

- Click track generation (moved to Phase 2)
- Practice tools (looping, speed control, pitch shifting) - Phase 3
- Performance mode UI with stage view - Phase 2
- MIDI control and foot pedal support - Phase 2
- Team collaboration features - Phase 4
- Audio routing beyond basic stereo output - Phase 5
- Recording functionality - Phase 3
- Mobile apps (iOS/Android) - Phase 5

## Expected Deliverable

1. User can import a folder of backing track files and see them organized in a library with detected stems grouped together
2. User can create a setlist by dragging songs from library, save it, and reload it in future sessions
3. User can play a song with multiple stems, adjust individual stem volumes in real-time, and experience <10ms latency with zero audio glitches
4. Application runs stably on macOS 11+ and Windows 10/11 64-bit without crashes during continuous playback sessions
