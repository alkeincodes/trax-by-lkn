# Phase 1 MVP Specification - TraX by LKN

**Created**: November 8, 2025  
**Timeline**: 3 months (12 weeks)  
**Status**: Planning Complete

## Overview

This specification defines the foundational MVP for TraX by LKN, a professional backing track management and playback application. Phase 1 establishes the core technical architecture required for all future features while delivering immediate value to users through multi-track audio playback, library management, and setlist organization.

## What's Included

### Core Features
1. **Audio Engine** - Real-time multi-track playback with <10ms latency supporting up to 16 simultaneous stems
2. **File Import** - Bulk import with automatic stem detection and metadata extraction
3. **Library Management** - Local database with search and filtering
4. **Setlist Builder** - Drag-and-drop interface for creating and managing setlists
5. **Playback Controls** - Transport controls, seeking, and per-stem volume/mute/solo

### Performance Requirements
- **Latency**: <10ms audio processing latency
- **Stability**: 99.9% uptime during performances (zero crashes)
- **Track Count**: Support 16 simultaneous audio stems
- **Memory**: <500MB idle, <2GB active
- **Platforms**: macOS 11+ and Windows 10/11 64-bit

## What's NOT Included (Future Phases)

- Click track generation → Phase 2
- Practice tools (looping, speed/pitch control) → Phase 3
- Performance mode UI with stage view → Phase 2
- MIDI control and foot pedal support → Phase 2
- Team collaboration features → Phase 4
- Advanced audio routing → Phase 5
- Recording functionality → Phase 3
- Mobile apps (iOS/Android) → Phase 5

## Documentation Structure

### Main Documents

1. **[spec.md](./spec.md)** - Complete requirements specification
   - User stories
   - Feature scope
   - Expected deliverables

2. **[spec-lite.md](./spec-lite.md)** - Condensed summary for quick reference

3. **[tasks.md](./tasks.md)** - Detailed task breakdown with 10 major tasks and 90+ subtasks
   - Follows TDD approach
   - Organized by component (audio engine, database, UI, etc.)

### Technical Specifications

4. **[sub-specs/technical-spec.md](./sub-specs/technical-spec.md)** - Comprehensive technical requirements
   - Audio engine architecture (cpal + symphonia)
   - File import system with stem detection
   - Tauri backend command structure
   - Vue 3 frontend architecture
   - Performance optimization strategies
   - Development timeline (3 months)

5. **[sub-specs/database-schema.md](./sub-specs/database-schema.md)** - Complete database design
   - SQLite schema (5 tables: songs, stems, setlists, setlist_items, app_settings)
   - Foreign key constraints and indexes
   - Migration system
   - Common query patterns
   - Backup and recovery strategies

6. **[sub-specs/implementation-guide.md](./sub-specs/implementation-guide.md)** - Practical development guidance
   - Recommended directory structure
   - Week-by-week implementation plan
   - Key technical decisions and rationale
   - Common pitfalls to avoid
   - Testing strategy
   - Performance benchmarks

## Technology Stack

### Backend (Rust)
- **Tauri v2** - Desktop app framework
- **cpal** - Cross-platform audio I/O
- **symphonia** - Audio decoding (WAV, MP3, FLAC)
- **rusqlite** - SQLite database interface
- **crossbeam-channel** - Lock-free audio thread communication

### Frontend (TypeScript)
- **Vue 3.5+** - UI framework with Composition API
- **Pinia** - State management
- **TailwindCSS v4** - Styling
- **Radix Vue** - Accessible UI primitives

## Development Timeline

### Month 1: Backend Foundation
- **Week 1-2**: Audio engine core (cpal + symphonia integration)
- **Week 3**: File import system with stem detection
- **Week 4**: Database schema and rusqlite integration

### Month 2: Frontend UI
- **Week 5**: Library view components
- **Week 6**: Playback controls and seek functionality
- **Week 7**: Stem mixer with volume/mute/solo
- **Week 8**: Setlist builder with drag-and-drop

### Month 3: Integration & Polish
- **Week 9**: Multi-track integration and synchronization
- **Week 10**: Error handling and recovery flows
- **Week 11**: Performance optimization
- **Week 12**: Testing, bug fixes, documentation

## Key Technical Decisions

### Audio Engine: cpal + symphonia
- **cpal**: Industry-standard cross-platform audio library for Rust
- **symphonia**: Pure Rust decoder (no C dependencies, better security)
- **Rationale**: Maximum control over latency, proven in production

### Database: SQLite
- **Local-first**: All data stored on user's device
- **Performance**: Fast for expected library sizes (< 100,000 songs)
- **Simplicity**: Single file, no server setup required

### Frontend: Pinia over Vuex
- **Pinia**: Official Vue 3 recommendation, better TypeScript support
- **Simpler API**: Less boilerplate than Vuex 4
- **Future-proof**: Actively maintained by Vue core team

## Success Metrics

### Technical Metrics
- Audio latency consistently < 10ms
- Zero crashes during 1-hour continuous playback
- Database queries < 100ms for 10,000 songs
- Startup time < 2 seconds
- Memory usage within targets (500MB idle, 2GB active)

### User Experience Metrics
- Import 1000 files in < 30 seconds
- Stem detection accuracy > 90% for standard naming patterns
- Search results update within 300ms of typing
- Drag-and-drop operations feel instant (< 50ms feedback)

## Getting Started

### For Developers

1. **Read the specifications in order**:
   - Start with `spec.md` for feature overview
   - Review `technical-spec.md` for architecture
   - Study `database-schema.md` for data model
   - Use `implementation-guide.md` as your week-by-week roadmap

2. **Set up development environment**:
   - Install Rust 1.70+
   - Install Node.js 22 LTS
   - Install Tauri prerequisites: https://tauri.app/v1/guides/getting-started/prerequisites

3. **Begin implementation**:
   - Follow tasks in `tasks.md` sequentially
   - Start with Task 1 (Audio Engine Core)
   - Write tests first (TDD approach)

### For Project Managers

- **Estimated effort**: 3 months with 1-2 full-time developers
- **Risk areas**: Audio latency optimization (Week 2), multi-track sync (Week 9)
- **Milestones**:
  - End of Month 1: Basic audio playback working
  - End of Month 2: Full UI functional
  - End of Month 3: Production-ready MVP

### For Stakeholders

This MVP delivers the core value proposition:
- Musicians can import their backing tracks
- Organize them into setlists
- Play them with professional-grade reliability
- Control individual instrument stems in real-time

Future phases will add practice tools, performance mode, and collaboration features, but Phase 1 establishes the foundation and delivers a usable product.

## Questions or Issues?

For questions about this specification, refer to:
- Technical architecture questions → `technical-spec.md`
- Database design questions → `database-schema.md`
- Implementation order questions → `implementation-guide.md`
- Task breakdown questions → `tasks.md`

---

**Next Steps**: Review all specification documents, then proceed to Task 1 in `tasks.md` to begin development.
