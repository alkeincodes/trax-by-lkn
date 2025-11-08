# Persistent Cache Implementation Plan

## Overview

Implement a persistent disk-based cache for decoded audio to eliminate re-decoding on app restarts. This will significantly improve cold-start performance for frequently used setlists.

## Current System

### In-Memory Cache (Current Implementation)
- **Location**: Rust `HashMap<String, Arc<DecodedAudio>>` in memory
- **Lifetime**: Exists only while app is running
- **Behavior**:
  - First load: Decode and cache
  - Subsequent loads (same session): Use cached version
  - App restart: Cache cleared, must re-decode
- **Performance**: Instant playback after first decode

## Proposed System

### Persistent Disk Cache
Store pre-decoded audio on disk to survive app restarts.

### Architecture

#### 1. Cache Storage Structure
```
~/.trax/cache/
├── metadata.db          # SQLite database for cache index
└── audio/
    ├── {song_id}.wav    # Pre-decoded audio files
    └── {song_id}.meta   # Metadata (hash, decode date, format)
```

#### 2. Cache Metadata Schema
```sql
CREATE TABLE audio_cache (
  song_id TEXT PRIMARY KEY,
  source_path TEXT NOT NULL,
  source_hash TEXT NOT NULL,      -- SHA256 of source file
  cache_path TEXT NOT NULL,       -- Path to cached .wav file
  sample_rate INTEGER NOT NULL,
  channels INTEGER NOT NULL,
  duration_seconds REAL NOT NULL,
  decoded_at INTEGER NOT NULL,    -- Unix timestamp
  last_accessed INTEGER NOT NULL, -- Unix timestamp
  file_size_bytes INTEGER NOT NULL
);

CREATE INDEX idx_last_accessed ON audio_cache(last_accessed);
```

#### 3. Cache Invalidation Strategy

**Invalidate cache when:**
- Source file hash changes (file modified)
- Source file no longer exists
- Cached file corrupted/missing
- Cache format version changes

**Cache eviction policy:**
- LRU (Least Recently Used) when cache size exceeds limit
- Default limit: 10GB (configurable in settings)
- Never evict songs in currently loaded setlist

#### 4. Implementation Flow

##### Preload Setlist (with persistent cache)
```rust
async fn preload_setlist(setlist_id: String) {
  let songs = db.get_setlist_songs(setlist_id)?;

  for song in songs {
    // 1. Check memory cache (fastest)
    if memory_cache.contains(&song.id) {
      continue; // Already in memory
    }

    // 2. Check disk cache (fast)
    if let Some(cached) = disk_cache.get(&song.id).await? {
      // Validate cache
      if cached.is_valid(&song.path)? {
        // Load from disk cache into memory
        let decoded = load_from_cache(&cached.path).await?;
        memory_cache.insert(song.id, decoded);
        emit_progress(current, total);
        continue;
      } else {
        // Invalid cache, remove it
        disk_cache.remove(&song.id).await?;
      }
    }

    // 3. Cache miss - decode from source (slow)
    let decoded = decode_audio(&song.path).await?;

    // Save to memory cache
    memory_cache.insert(song.id.clone(), decoded.clone());

    // Save to disk cache (async, don't block)
    spawn_async(async move {
      disk_cache.save(&song.id, &decoded).await?;
    });

    emit_progress(current, total);
  }
}
```

##### Cache Validation
```rust
impl CachedAudio {
  fn is_valid(&self, source_path: &str) -> Result<bool> {
    // Check if cached file exists
    if !self.cache_path.exists() {
      return Ok(false);
    }

    // Check if source file still exists
    if !Path::new(source_path).exists() {
      return Ok(false);
    }

    // Verify source file hasn't changed
    let current_hash = calculate_file_hash(source_path)?;
    if current_hash != self.source_hash {
      return Ok(false);
    }

    Ok(true)
  }
}
```

#### 5. File Format for Cached Audio

**Option A: WAV (Recommended)**
- **Pros**:
  - Simple, no encoding overhead
  - Standard format, widely supported
  - Fast read/write
- **Cons**:
  - Larger file size (~10MB per minute of stereo audio)
- **Use case**: Best for TraX (fast load is priority)

**Option B: FLAC**
- **Pros**:
  - Lossless compression (~50% size of WAV)
  - Good read performance
- **Cons**:
  - Encoding overhead when writing cache
  - Slightly slower load than WAV
- **Use case**: If disk space is a major concern

**Recommendation: Use WAV for cache format**

#### 6. Settings Integration

Add to app settings:
```rust
pub struct CacheSettings {
  pub enabled: bool,              // Default: true
  pub max_size_gb: u64,          // Default: 10GB
  pub auto_clear_days: u32,      // Default: 30 days
  pub cache_location: PathBuf,   // Default: ~/.trax/cache/
}
```

UI settings panel:
- Enable/Disable persistent cache
- Set maximum cache size
- View current cache size
- "Clear Cache" button
- "Preload All Setlists" button (for pre-show prep)

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1)
- [ ] Create cache directory structure
- [ ] Implement SQLite metadata database
- [ ] Add file hashing utilities
- [ ] Create cache validation logic

### Phase 2: Cache Read/Write (Week 1-2)
- [ ] Implement cache lookup on preload
- [ ] Implement cache write after decode
- [ ] Add cache invalidation checks
- [ ] Handle cache corruption gracefully

### Phase 3: Cache Management (Week 2)
- [ ] Implement LRU eviction policy
- [ ] Add cache size limits
- [ ] Create cache statistics tracking
- [ ] Add cache cleanup on startup

### Phase 4: UI Integration (Week 2-3)
- [ ] Add settings panel for cache config
- [ ] Show cache status in UI
- [ ] Add "Clear Cache" button
- [ ] Add cache hit/miss indicators

### Phase 5: Testing & Optimization (Week 3)
- [ ] Test with large setlists (50+ songs)
- [ ] Benchmark cache performance
- [ ] Test cache invalidation scenarios
- [ ] Test across app restarts

## Performance Expectations

### Cold Start (First Load)
- **Current**: Decode all songs (~2-5s per song)
- **With Cache**: Decode all songs, write to cache (~2-5s per song + 0.5s write)
- **Impact**: Slightly slower first load due to cache write

### Warm Start (App Restart)
- **Current**: Re-decode all songs (~2-5s per song)
- **With Cache**: Load from disk (~0.1-0.2s per song)
- **Impact**: 10-50x faster! (Huge win)

### Hot Start (Same Session)
- **Current**: Instant (memory cache hit)
- **With Cache**: Instant (memory cache hit)
- **Impact**: No change

## Disk Space Considerations

### Example: Typical Setlist
- 15 songs × 4 minutes × 48kHz stereo
- WAV format: ~15 songs × 40MB = 600MB per setlist
- 10GB limit = ~16 setlists cached
- FLAC format: ~15 songs × 20MB = 300MB per setlist
- 10GB limit = ~33 setlists cached

**Recommendation**: 10GB default limit is reasonable for worship team use case

## Edge Cases to Handle

1. **Partial setlist in cache**
   - Some songs cached, some not
   - Show combined progress for both loads

2. **Cache corruption**
   - Detect corrupted files
   - Fall back to re-decode
   - Remove bad entries

3. **Disk full**
   - Gracefully handle write failures
   - Fall back to memory-only mode
   - Warn user to clear cache

4. **Concurrent access**
   - Multiple setlists loading simultaneously
   - Lock cache database during writes
   - Queue cache writes to avoid conflicts

5. **Source file moved/renamed**
   - Hash-based validation will detect this
   - Re-decode from new location
   - Update cache metadata

## Future Enhancements

### Cloud Sync (Phase 2 - Future)
- Sync cache across devices
- Team members share pre-decoded setlists
- Reduces decode time for entire team

### Smart Preloading (Phase 2 - Future)
- Predict likely next setlists
- Pre-cache in background when idle
- ML-based usage patterns

### Cache Compression (Phase 2 - Future)
- Compress older cache entries
- Keep recent in WAV, compress old to FLAC
- Balance space vs speed

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Cache takes too much disk space | High | Implement size limits + LRU eviction |
| Corrupted cache causes crashes | High | Validate before load, graceful fallback |
| Cache invalidation too aggressive | Medium | Test thoroughly, log invalidations |
| Write performance degrades | Medium | Async writes, don't block UI |
| Hash calculation is slow | Low | Use fast hash (xxHash), cache hashes |

## Success Metrics

- [ ] 90%+ cache hit rate for repeat setlists
- [ ] <0.5s per song load time from disk cache
- [ ] <10GB disk usage for typical user (16 setlists)
- [ ] Zero crashes due to cache issues
- [ ] No noticeable UI lag during cache operations

## References

- Rust crate: `symphonia` (already using for decode)
- Rust crate: `hound` (WAV read/write)
- Rust crate: `rusqlite` (SQLite for metadata)
- Rust crate: `blake3` or `xxhash-rust` (fast hashing)

---

**Status**: Planning Phase
**Priority**: Medium (Quality of Life improvement)
**Effort**: 2-3 weeks
**Owner**: TBD
**Created**: 2025-11-09
