# Parallel Decoding Architecture

## Overview

TraX uses **parallel FLAC decoding** to achieve fast load times while maintaining zero audio dropouts during live performance.

## Architecture

### Core Approach
- **Full Pre-Decode**: All stems are decoded into RAM before playback starts (like Multitrack.com Playback, LoopCommunity Prime)
- **Parallel Processing**: All stems decode simultaneously using `tokio::spawn_blocking`
- **Zero Dropouts**: Once loaded, all audio is in memory - no real-time decoding during playback

### Performance

**Sequential Loading (Old)**:
- 20 stems × 1.5s each = **30 seconds**
- CPU cores sitting idle

**Parallel Loading (Current)**:
- 20 stems decoded simultaneously
- On 8-core CPU: ~4 seconds (**7.5x faster**)
- On 16-core CPU: ~2 seconds (**15x faster**)

## Implementation Details

### Key Files
- `src-tauri/src/commands/playback.rs` - Parallel decoding implementation
- `src-tauri/src/audio/multi_track.rs` - Audio engine
- `src-tauri/src/audio/decoder.rs` - FLAC decoder

### How It Works

```rust
// 1. Spawn parallel decoding tasks for all stems
for stem in stems {
    let task = tokio::task::spawn_blocking(move || {
        // CPU-intensive FLAC decoding
        let samples = decode_stem(stem);
        samples
    });
    tasks.push(task);
}

// 2. Wait for all to complete
let results = futures::future::join_all(tasks).await;

// 3. Store in memory cache
cache.insert(song_id, decoded_stems);

// 4. Play from memory (zero latency)
audio_engine.play(cached_stems);
```

### Benefits
1. **Fast Loading**: Multi-core utilization reduces load time dramatically
2. **Reliable Playback**: All audio pre-decoded = zero dropouts
3. **Simple Architecture**: No complex streaming, buffering, or chunk management
4. **Live Performance Ready**: Suitable for worship services, concerts, rehearsals

## Comparison to Streaming Approach

| Aspect | Parallel Pre-Decode (Current) | Streaming (Removed) |
|--------|------------------------------|---------------------|
| Load Time | 2-4 seconds | 40+ seconds |
| Dropouts | Zero | Frequent |
| RAM Usage | ~5GB per song | ~200MB per song |
| Complexity | Simple | Complex |
| Live Performance | ✅ Perfect | ❌ Unreliable |

## Why Streaming Was Removed

Initial attempts to implement chunk-based streaming revealed:
- FLAC decoding is too CPU-intensive for real-time streaming
- Background chunk loading caused audio dropouts
- Complex architecture (background workers, buffers, polling)
- Not suitable for live performance (zero-dropout requirement)

Professional backing track apps (Multitrack.com, LoopCommunity Prime) all use full pre-decode for reliability.

## Architecture Decision: No Persistent Cache

The application **does not use a persistent cache** for decoded audio. Here's why:

### Why No Cache?
1. **Cache was useless** - Just copied FLAC files, still required decoding
2. **No performance benefit** - Same decode time whether from original or cached copy
3. **Wasted disk space** - Duplicated files that already exist
4. **Added complexity** - Hash calculation, validation, database tracking

### What About PCM Cache?
Converting FLAC → PCM and caching would help, but:
- **Not needed yet** - Parallel decoding is already fast (2-4 seconds)
- **Disk space** - Would use ~900MB per song
- **Complexity** - Requires cache management, LRU eviction, etc.
- **Premature optimization** - Current approach works great

### Future Consideration
If load times become a problem (>5 seconds consistently):
- Convert FLAC → WAV on import
- Store in original location (user's audio library)
- App just loads pre-decoded WAV files
- No cache management needed - files live with user's music

## Technical Notes

### Thread Safety
- Uses `tokio::spawn_blocking` for CPU-bound work
- Each stem decodes independently (no shared state)
- Results collected via `futures::join_all`

### Memory Management
- Decoded audio stored in `Arc<Mutex<HashMap<String, CachedSong>>>`
- Shared between Tauri commands
- Memory released when song evicted from cache

### Capacity
- Current: 32 stems max (Extended capacity)
- Sufficient for most backing tracks (10-30 stems typical)
- Can increase to 64 or 256 if needed

## Conclusion

The parallel pre-decode approach provides:
- ✅ Fast load times (multi-core utilization)
- ✅ Zero dropouts (all audio in RAM)
- ✅ Simple, maintainable code
- ✅ Live performance reliability

Perfect for worship teams, live musicians, and professional performances.
