# Streaming Audio Architecture Plan

## Executive Summary

Implement chunk-based streaming decoder to enable **instant playback** (500ms first load, 100-200ms cached loads) by loading audio in small segments instead of decoding entire files upfront.

**Key Metrics:**
- **Current:** 6-7 seconds to start playback
- **Target (Phase 1):** 500ms to start playback (first time)
- **Target (Phase 2):** 100-200ms to start playback (cached)
- **RAM Usage:** 231MB per song (vs 5GB currently)
- **Storage:** 900MB per song (cached chunks)

---

## Problem Statement

**Current System Issues:**
- 30 stems per song × 6-7 seconds decode time = slow loading
- Pre-loading entire songs requires 5GB RAM per song (50GB for 10-song setlist)
- Users experience 6-7 second delay when clicking play

**User Requirements:**
- 5-10 songs per setlist
- ~30 stems per song (average)
- Random song jumping (not just sequential)
- Instant playback feel (<1 second)

---

## Solution: Chunk-Based Streaming

### Core Concept

Instead of decoding entire songs:
1. Decode only **first 5 seconds** immediately
2. Start playback instantly with this buffer
3. Continue decoding in background while playing
4. Keep only 20-second buffer in RAM (rolling window)

```
Song Timeline (360 seconds):
┌─────────────────────────────────────────────────┐
│ [Chunk 0][Chunk 1][Chunk 2]...[Chunk 71]       │
│    5s      5s      5s          5s               │
└─────────────────────────────────────────────────┘
     ↑       ↑                    ↑
  Playing  Buffered           Decoding in BG

Active Memory:
[Chunk N][Chunk N+1][Chunk N+2][Chunk N+3]
  ←────────── 20 seconds ────────────→
```

**Memory Usage:**
- 20 seconds @ 48kHz stereo = ~7.7MB per stem
- 30 stems × 7.7MB = **231MB** (vs 5GB full song)

---

## Architecture Components

### Component 1: ChunkDecoder

**Location:** `src-tauri/src/audio/chunk_decoder.rs`

**Purpose:** Decode FLAC files in 5-second chunks instead of all at once.

```rust
pub struct ChunkDecoder {
  file_path: PathBuf,
  sample_rate: u32,
  channels: u16,
  total_samples: usize,
  chunk_size_samples: usize, // 5 seconds = 480,000 samples @ 48kHz
}

impl ChunkDecoder {
  /// Open file, read metadata only (fast)
  pub fn new(path: &str, chunk_duration_secs: f64) -> Result<Self>

  /// Decode a specific 5-second chunk
  pub fn decode_chunk(&mut self, chunk_index: usize) -> Result<Vec<f32>>

  /// Get total number of chunks in file
  pub fn total_chunks(&self) -> usize

  /// Calculate which chunk contains a time position
  pub fn chunk_at_position(&self, position_secs: f64) -> usize
}
```

**Implementation Note:**
- Symphonia doesn't support efficient seeking
- First decode: Decode from start, discard chunks until target
- Cached: Pre-decoded chunks stored on disk (see StreamingCache)

---

### Component 2: StreamingCache

**Location:** `src-tauri/src/cache/streaming_cache.rs`

**Purpose:** Store pre-decoded chunks on disk for instant loading.

**Directory Structure:**
```
~/Library/Caches/trax/streaming_cache/
├── {song_id}_{stem_id}/
│   ├── metadata.json        # Sample rate, channels, chunk count
│   ├── chunk_000.flac       # 5s of audio, FLAC compressed
│   ├── chunk_001.flac
│   ├── chunk_002.flac
│   └── ...
```

**Chunk Storage Format:**
- **FLAC-compressed** f32 samples (not raw)
- Each chunk: ~7.7MB raw → ~2-3MB FLAC (~70% compression)
- Fast to load: 50-100ms per chunk

**Cache Strategy:**
- **Only cache first 60 seconds (12 chunks)** per stem
- Total per song: 12 chunks × 30 stems × 2.5MB = **~900MB**
- Rest decodes on-demand during playback

**LRU Eviction:**
- Keep last 10-15 songs in cache
- Auto-evict oldest when cache exceeds size limit
- Default limit: 10GB (allows ~11 songs fully cached)

```rust
pub struct StreamingCache {
  cache_dir: PathBuf,
  max_size_gb: u64,
}

impl StreamingCache {
  /// Check if song has cached chunks
  pub fn has_cache(&self, song_id: &str, stem_id: &str) -> bool

  /// Write a FLAC-compressed chunk to disk
  pub fn write_chunk(
    &self,
    song_id: &str,
    stem_id: &str,
    chunk_index: usize,
    samples: &[f32]
  ) -> Result<()>

  /// Load and decompress a chunk (50-100ms)
  pub fn read_chunk(
    &self,
    song_id: &str,
    stem_id: &str,
    chunk_index: usize
  ) -> Result<Vec<f32>>

  /// Write metadata file
  pub fn write_metadata(
    &self,
    song_id: &str,
    stem_id: &str,
    meta: ChunkMetadata
  ) -> Result<()>

  /// Evict least recently used songs
  pub fn evict_lru(&mut self, target_size_gb: u64) -> Result<()>
}
```

---

### Component 3: StreamingBuffer

**Location:** `src-tauri/src/audio/streaming_buffer.rs`

**Purpose:** Manage 4-chunk rolling buffer in RAM per stem.

```rust
pub struct StreamingBuffer {
  chunks: VecDeque<Chunk>,      // Max 4 chunks (20 seconds)
  current_chunk_index: usize,
  total_chunks: usize,
}

struct Chunk {
  index: usize,
  samples: Vec<f32>,
  sample_offset: usize,  // Global sample position in song
}

impl StreamingBuffer {
  /// Get samples for current playback position
  pub fn get_samples(&self, position: usize, count: usize) -> &[f32]

  /// Check if buffer needs refill (playing near edge)
  pub fn needs_refill(&self, current_position: usize) -> bool

  /// Add next chunk, drop oldest if buffer full
  pub fn push_chunk(&mut self, chunk: Chunk)

  /// Clear and reload buffer at new position (for seeking)
  pub fn seek_to_chunk(&mut self, chunk_index: usize)

  /// Check if a chunk is already in buffer
  pub fn has_chunk(&self, chunk_index: usize) -> bool
}
```

**Buffer Management:**
- Keep chunks [N, N+1, N+2, N+3] where N = current playing chunk
- When playback enters N+1, push N+4 and drop N
- Circular buffer prevents unbounded memory growth

---

### Component 4: BackgroundChunkLoader

**Location:** `src-tauri/src/audio/background_loader.rs`

**Purpose:** Worker thread that loads chunks ahead of playback.

```rust
pub struct BackgroundChunkLoader {
  urgent_queue: VecDeque<ChunkLoadRequest>,  // Seek requests
  normal_queue: VecDeque<ChunkLoadRequest>,  // Buffering ahead
  tx: Sender<LoadedChunk>,
  handle: JoinHandle<()>,
}

pub enum ChunkPriority {
  Urgent,   // User seek - load immediately, interrupt normal queue
  Normal,   // Buffering ahead - load when convenient
}

struct ChunkLoadRequest {
  song_id: String,
  stem_id: String,
  chunk_index: usize,
  priority: ChunkPriority,
}

struct LoadedChunk {
  song_id: String,
  stem_id: String,
  chunk_index: usize,
  samples: Vec<f32>,
}

impl BackgroundChunkLoader {
  /// Start background worker thread
  pub fn start(cache: Arc<StreamingCache>) -> Self

  /// Request a chunk to be loaded
  pub fn request_chunk(
    &self,
    song_id: &str,
    stem_id: &str,
    chunk_index: usize,
    priority: ChunkPriority
  )

  /// Stop background thread
  pub fn stop(&self)
}
```

**Thread Loop:**
```rust
loop {
  // 1. Process urgent requests first (user seeks)
  while let Some(req) = urgent_queue.pop() {
    let chunk = load_chunk_from_cache_or_decode(req);
    send_to_main_thread(chunk);
  }

  // 2. Process normal buffering requests
  while let Some(req) = normal_queue.pop() {
    let chunk = load_chunk_from_cache_or_decode(req);
    send_to_main_thread(chunk);
  }

  // 3. Sleep briefly if no work
  sleep(10ms);
}
```

**Parallel Decoding:**
- Use `rayon` to decode multiple stems simultaneously
- Decode 30 stems in parallel → 30x speedup
- First chunk load: 500ms / 30 = ~17ms per stem in parallel

---

## Playback Flows

### Flow 1: First Time Playing a Song

```
User clicks "Play" on Song #1 (never played before)
  ↓
[Main Thread - T+0ms]
├─ Create ChunkDecoder for all 30 stems
├─ Request chunk #0 for all stems (Priority: Urgent)
└─ Show "Loading..." modal
  ↓
[Background Thread - T+0ms]
├─ Receive 30 chunk requests
├─ Check StreamingCache → No chunks cached
├─ Decode chunk #0 from FLAC for all 30 stems (parallel)
│  └─ Time: ~500ms (with parallel decoding)
├─ Save chunks #0-#11 to StreamingCache (first 60s)
└─ Send chunks #0 to main thread
  ↓
[Main Thread - T+500ms]
├─ Receive all chunk #0 samples
├─ Populate StreamingBuffer with chunk #0 for each stem
├─ Start playback ✓
├─ Hide loading modal
└─ Request chunks #1-#3 (Priority: Normal)
  ↓
[Background Thread - Ongoing]
├─ Decode chunks #1-#71 while song plays
├─ Save chunks #1-#11 to cache (60s total cached)
├─ Feed chunks #1-#3 to StreamingBuffer as needed
└─ Discard chunks #12+ after playback (not cached)
```

**Timeline:**
- **T+0ms:** User clicks play
- **T+50ms:** ChunkDecoders initialized
- **T+500ms:** First chunks decoded, **playback starts** ✓
- **T+500ms - T+360s:** Background decodes remaining chunks while playing

---

### Flow 2: Playing a Cached Song

```
User clicks "Play" on Song #1 (played before, cached)
  ↓
[Main Thread - T+0ms]
├─ Create ChunkDecoder for all 30 stems
├─ Request chunk #0 for all stems (Priority: Urgent)
└─ Show "Loading..." indicator
  ↓
[Background Thread - T+0ms]
├─ Receive 30 chunk requests
├─ Check StreamingCache → Chunks #0-#11 exist!
├─ Load chunk #0 from disk (FLAC decompress)
│  └─ Time: ~100ms for all 30 stems (parallel read)
└─ Send chunks #0 to main thread
  ↓
[Main Thread - T+100ms]
├─ Receive all chunk #0 samples
├─ Populate StreamingBuffer
├─ Start playback ✓ (nearly instant!)
└─ Request chunks #1-#3 (Priority: Normal)
  ↓
[Background Thread - Ongoing]
├─ Load chunks #1-#11 from cache (fast)
├─ Decode chunks #12+ as needed (not cached)
└─ Feed to StreamingBuffer
```

**Timeline:**
- **T+0ms:** User clicks play
- **T+100-200ms:** Cached chunks loaded, **playback starts** ✓ (instant!)

---

### Flow 3: Seeking (Corrected - No Pause!)

```
User drags seek bar to 3:45 (225s) while song is playing
  ↓
[Main Thread - T+0ms]
├─ Calculate target chunk: 225s ÷ 5s = chunk #45
├─ Check StreamingBuffer.has_chunk(45)
│  ├─ YES → Jump immediately (0ms delay) ✓
│  └─ NO → Continue to background load ↓
├─ Continue playing current chunk (don't pause!)
├─ Request chunk #45 (Priority: Urgent)
└─ Show "Seeking..." indicator
  ↓
[Background Thread - T+0ms]
├─ Receive urgent seek request
├─ Interrupt normal queue processing
├─ Check cache for chunk #45
│  ├─ Cached → Load from disk (~50-100ms)
│  └─ Not cached → Decode from FLAC (~200-300ms)
└─ Send chunk #45 to main thread
  ↓
[Main Thread - T+50-300ms]
├─ Continue playing old position during load
├─ Receive chunk #45
├─ [Optional] Crossfade from current position to 3:45
│  └─ Fade out old position 100%→0% over 50ms
│  └─ Fade in new position 0%→100% over 50ms
├─ Swap StreamingBuffer to chunks #45-#48
├─ Jump playback position to 3:45
└─ Hide "Seeking..." indicator
  ↓
[Background Thread - Ongoing]
├─ Load chunks #46-#48 (Priority: Normal)
└─ Resume normal buffering
```

**User Experience:**

| Scenario | Delay | Behavior |
|----------|-------|----------|
| Seek within buffered range | 0ms | Instant jump ✓ |
| Seek to cached chunk | 50-100ms | Audio continues at old position, then smooth jump |
| Seek to uncached chunk | 200-300ms | Audio continues, show "Loading...", then jump |

**Key Principle:** **Never pause audio**. Keep playing current buffer while loading new chunks.

---

### Flow 4: Preloading Next Song (Smart Buffering)

```
User is playing Song #1, Song #2 is next in setlist
  ↓
[Background Thread - Opportunistic]
├─ Detect: Song #1 is at 80% completion
├─ Check: Song #2 chunks cached?
│  └─ No → Begin decoding Song #2 chunk #0
├─ Low priority: Only decode during idle CPU time
└─ Cache Song #2 chunks #0-#11 before Song #1 ends
  ↓
[Result]
└─ When Song #1 ends, Song #2 starts instantly (cached!)
```

**Timeline:**
- Song #1 at 4:48 (80% of 6:00) → Start preloading Song #2
- Song #1 ends at 6:00 → Song #2 chunk #0 ready → Instant transition ✓

---

## Integration with Existing Audio Engine

### MultiTrackEngine Modifications

**Current Structure:**
```rust
pub struct MultiTrackEngine {
  stems: Vec<StemTrack>,
  playback_state: Arc<AtomicU8>,
  position: Arc<AtomicUsize>,
  // ...
}
```

**New Structure:**
```rust
pub struct MultiTrackEngine {
  stems: Vec<StreamingStem>,  // Changed from StemTrack
  playback_state: Arc<AtomicU8>,
  position: Arc<AtomicUsize>,
  chunk_loader: Arc<BackgroundChunkLoader>,  // New
  // ...
}

struct StreamingStem {
  stem_id: String,
  buffer: StreamingBuffer,  // New: chunk-based buffer
  volume: f32,
  is_muted: bool,
  is_solo: bool,
}
```

### Audio Callback Changes

**Current Callback:**
```rust
// Reads from large pre-loaded buffer
fn audio_callback(&mut self, output: &mut [f32]) {
  for stem in &self.stems {
    let samples = stem.buffer[position..position+output.len()];
    mix_into(output, samples, stem.volume);
  }
}
```

**New Callback:**
```rust
fn audio_callback(&mut self, output: &mut [f32]) {
  // 1. Check if buffer needs refill
  if self.needs_refill() {
    self.request_next_chunks();
  }

  // 2. Read from streaming buffer
  for stem in &self.stems {
    let samples = stem.buffer.get_samples(self.position, output.len());
    mix_into(output, samples, stem.volume);
  }

  // 3. Update position
  self.position += output.len();
}
```

**Buffer Refill Logic:**
```rust
fn needs_refill(&self) -> bool {
  let current_chunk = self.position / CHUNK_SIZE_SAMPLES;

  // If we're playing chunk N, we should have chunks [N, N+1, N+2, N+3]
  // Request refill when entering chunk N+2 (2 chunks ahead)
  for stem in &self.stems {
    if !stem.buffer.has_chunk(current_chunk + 3) {
      return true;
    }
  }
  false
}
```

---

## Performance Metrics

### Latency Targets

| Operation | Target | Worst Case | Current |
|-----------|--------|------------|---------|
| First playback start | 500ms | 1000ms | 6-7s |
| Cached playback start | 100ms | 200ms | 6-7s |
| Seek (buffered) | 0ms | 0ms | 6-7s |
| Seek (cached) | 50ms | 100ms | 6-7s |
| Seek (uncached) | 200ms | 300ms | 6-7s |
| Random song jump | 100-200ms | 500ms | 6-7s |

### Resource Usage

| Resource | Current | Phase 1 | Phase 2 (Cached) |
|----------|---------|---------|------------------|
| RAM per song | 5GB | 231MB | 231MB |
| Disk per song | 0MB | 0MB | 900MB |
| CPU (playback) | Low | Low | Low |
| CPU (loading) | Spike | Spread over time | Spread over time |

### Cache Efficiency

**Cache Hit Rate (Expected):**
- First 60 seconds of song: 100% hit rate
- Beyond 60 seconds: 0% hit rate (decode on-demand)
- Overall: ~90% hit rate (most playback starts from beginning)

**Cache Size Management:**
- Default limit: 10GB
- Average song: 900MB
- Capacity: ~11 songs
- LRU eviction: Automatic when limit exceeded

---

## Implementation Phases

### Phase 1: Core Streaming (No Cache)

**Goal:** Prove streaming works, achieve 500ms start time

**Tasks:**
1. ✅ Create `ChunkDecoder` module
   - Implement chunk-based FLAC decoding
   - Support seeking to arbitrary chunks
   - Handle sample rate conversion per chunk

2. ✅ Create `StreamingBuffer` module
   - Implement 4-chunk ring buffer
   - Refill logic and buffer management
   - Thread-safe sample access

3. ✅ Create `BackgroundChunkLoader` module
   - Worker thread with priority queue
   - Parallel chunk decoding (rayon)
   - Communication with main thread

4. ✅ Modify `MultiTrackEngine`
   - Replace static buffers with `StreamingBuffer`
   - Integrate `BackgroundChunkLoader`
   - Update audio callback for streaming

5. ✅ Update `load_song` command
   - Initialize streaming components
   - Load first chunks only
   - Start playback quickly

**Test Criteria:**
- ✅ First playback starts in <500ms
- ✅ No audio glitches during streaming
- ✅ RAM usage <500MB per song
- ✅ Seeking works (200-300ms delay acceptable)

**Deliverable:** Working streaming playback, no cache yet

---

### Phase 2: Add Streaming Cache

**Goal:** Reduce cached loads to 100-200ms

**Tasks:**
1. ✅ Create `StreamingCache` module
   - FLAC-compressed chunk storage
   - Metadata management
   - Read/write operations

2. ✅ Implement caching in `BackgroundChunkLoader`
   - Save first 60s of chunks during decode
   - Load from cache when available
   - Fallback to decode if cache miss

3. ✅ Implement LRU eviction
   - Track cache size and access times
   - Auto-evict when limit exceeded
   - User setting for cache size limit

4. ✅ Add cache management UI
   - Show cache size in settings
   - "Clear cache" button
   - Cache statistics

**Test Criteria:**
- ✅ Cached songs load in <200ms
- ✅ Cache size stays under limit
- ✅ LRU eviction works correctly
- ✅ Cache survives app restart

**Deliverable:** Full streaming system with persistent cache

---

### Phase 3: Optimizations

**Goal:** Production-ready performance and UX

**Tasks:**
1. ✅ Parallel chunk decoding
   - Decode multiple stems simultaneously
   - Utilize all CPU cores
   - Target: 500ms / 30 = ~17ms per stem

2. ✅ Smart preloading
   - Detect next song in setlist
   - Preload during idle time
   - Seamless song transitions

3. ✅ Seek improvements
   - Implement crossfade on seek
   - Optimize chunk priority queue
   - Reduce seek latency

4. ✅ Progress indicators
   - Show chunk loading status
   - "Seeking..." indicator
   - Buffer fill level

5. ✅ Error handling
   - Graceful degradation on decode errors
   - Retry logic for failed loads
   - User notifications

**Test Criteria:**
- ✅ Parallel decode achieves <200ms first load
- ✅ Preloading works without affecting current playback
- ✅ Seek feels instant (<100ms)
- ✅ All error cases handled

**Deliverable:** Polished, production-ready streaming

---

## Risk Mitigation

### Risk 1: Symphonia Can't Seek Efficiently

**Problem:** FLAC decoders typically must decode from start to reach arbitrary positions

**Impact:** Chunk decoding might be slow (need to decode all previous chunks)

**Mitigation:**
- ✅ Accept slow first decode (~6s for full song)
- ✅ Cache decoded chunks to disk
- ✅ Subsequent loads read from cache (fast)
- ✅ Only pay decode cost once per song

**Fallback:** If chunking impossible, keep current system

---

### Risk 2: Disk Cache Too Large

**Problem:** 10 songs × 900MB = 9GB cache

**Impact:** Users with limited disk space may run out

**Mitigation:**
- ✅ LRU eviction (auto-delete old songs)
- ✅ User-configurable cache size limit
- ✅ "Clear cache" button in settings
- ✅ Show cache size prominently

**Alternative:** Reduce cache to 30s (450MB per song)

---

### Risk 3: Background Thread Can't Keep Up

**Problem:** Playback reaches buffer edge before next chunk loads

**Impact:** Audio glitches or gaps

**Mitigation:**
- ✅ 4-chunk buffer = 20 seconds lookahead
- ✅ Average decode: 6s for 360s = 60x realtime
- ✅ Even with 30 stems, 5s chunk decodes in ~300ms
- ✅ 20s buffer = 66x safety margin
- ✅ Priority queue for urgent seeks

**Monitoring:** Log buffer fill level, alert if <2 chunks

---

### Risk 4: Implementation Complexity

**Problem:** Streaming architecture is complex

**Impact:** Bugs, maintenance burden

**Mitigation:**
- ✅ Phased rollout (Phase 1 proves concept)
- ✅ Comprehensive testing per phase
- ✅ Can revert to current system if needed
- ✅ Clear documentation (this file!)

**Fallback:** Feature flag to disable streaming

---

## Testing Strategy

### Unit Tests

**ChunkDecoder:**
- ✅ Decodes chunk #0 correctly
- ✅ Decodes arbitrary chunk correctly
- ✅ Handles edge cases (last chunk, short files)
- ✅ Resamples correctly

**StreamingBuffer:**
- ✅ Returns correct samples for position
- ✅ Refill logic triggers at right time
- ✅ Seeking clears and reloads correctly
- ✅ Chunk eviction works

**StreamingCache:**
- ✅ Writes and reads chunks correctly
- ✅ FLAC compression/decompression works
- ✅ LRU eviction removes oldest songs
- ✅ Handles corrupt cache gracefully

---

### Integration Tests

**Full Playback Flow:**
- ✅ Load song, play, verify audio output
- ✅ Seek to various positions, verify correctness
- ✅ Play multiple songs sequentially
- ✅ Random song jumps work

**Cache Behavior:**
- ✅ First load caches chunks
- ✅ Second load reads from cache
- ✅ Cache survives app restart
- ✅ Eviction doesn't break playback

---

### Performance Tests

**Latency Benchmarks:**
- ✅ Measure first load time (target: <500ms)
- ✅ Measure cached load time (target: <200ms)
- ✅ Measure seek latency (target: <100ms)
- ✅ Measure transition time (song to song)

**Resource Tests:**
- ✅ Monitor RAM usage during playback
- ✅ Monitor disk I/O patterns
- ✅ Monitor CPU usage (should be low)
- ✅ Verify no memory leaks

---

### User Acceptance Tests

**Real-world Scenarios:**
1. ✅ User opens setlist, plays song #1
   - Expected: Starts in <500ms

2. ✅ User seeks around during playback
   - Expected: No glitches, smooth seeking

3. ✅ User plays same song again
   - Expected: Starts in <200ms (cached)

4. ✅ User plays 10 songs in a row
   - Expected: Seamless transitions, cache works

5. ✅ User jumps randomly between songs
   - Expected: Quick loads, no crashes

---

## Configuration & Settings

### User Settings

**Cache Settings (in Settings modal):**
```
Audio Cache
├─ Enable Cache: [✓] (toggle)
├─ Max Cache Size: [10] GB (slider: 1-50GB)
├─ Current Usage: 4.2 GB / 10 GB (progress bar)
└─ [Clear Cache] button
```

**Developer Settings (debug mode):**
```
Streaming Debug
├─ Chunk Size: [5] seconds
├─ Buffer Size: [4] chunks
├─ Enable Parallel Decode: [✓]
├─ Log Chunk Loads: [✓]
└─ Cache First N Chunks: [12] (60 seconds)
```

---

### Internal Constants

**File:** `src-tauri/src/audio/streaming/constants.rs`

```rust
// Chunk configuration
pub const CHUNK_DURATION_SECS: f64 = 5.0;
pub const BUFFER_CHUNK_COUNT: usize = 4;  // 20 seconds
pub const CACHE_CHUNK_COUNT: usize = 12;  // 60 seconds

// Cache configuration
pub const DEFAULT_CACHE_SIZE_GB: u64 = 10;
pub const MIN_CACHE_SIZE_GB: u64 = 1;
pub const MAX_CACHE_SIZE_GB: u64 = 50;

// Performance tuning
pub const PARALLEL_DECODE_THREADS: usize = 8;
pub const REFILL_THRESHOLD_CHUNKS: usize = 2;  // Request refill when 2 chunks ahead
```

---

## Monitoring & Debugging

### Logging

**Log Levels:**
- `INFO`: Key events (song load, chunk cache hit/miss)
- `DEBUG`: Detailed flow (chunk requests, buffer state)
- `TRACE`: Per-sample debugging (usually disabled)

**Example Logs:**
```
[INFO] StreamingCache: Cache HIT for song_123/stem_456 chunk #0 (loaded in 85ms)
[INFO] BackgroundChunkLoader: Loaded chunk #1 for stem_456 (from cache)
[DEBUG] StreamingBuffer: Refill needed, requesting chunk #5
[DEBUG] BackgroundChunkLoader: Urgent seek request for chunk #45
[WARN] StreamingCache: Cache size exceeded limit, evicting song_789
```

---

### Metrics to Track

**Performance Metrics:**
- Average first load time
- Average cached load time
- Average seek latency
- Buffer underruns (should be 0)

**Cache Metrics:**
- Cache hit rate
- Cache size growth
- Eviction frequency
- Disk I/O throughput

**Playback Metrics:**
- Audio glitches (should be 0)
- Decode errors
- Memory usage
- CPU usage

---

## Rollout Plan

### Development Timeline

**Week 1-2: Phase 1 Implementation**
- Build ChunkDecoder
- Build StreamingBuffer
- Build BackgroundChunkLoader
- Integrate with MultiTrackEngine

**Week 3: Phase 1 Testing**
- Unit tests for all components
- Integration tests
- Performance benchmarks
- Bug fixes

**Week 4: Phase 2 Implementation**
- Build StreamingCache
- Implement LRU eviction
- Add cache management UI

**Week 5: Phase 2 Testing**
- Cache persistence tests
- Eviction tests
- Performance with cache

**Week 6: Phase 3 Optimization**
- Parallel decoding
- Smart preloading
- Seek improvements
- Polish

**Week 7: Final Testing & Release**
- User acceptance testing
- Documentation
- Release v1.0 with streaming

---

### Feature Flag

**Enable gradual rollout:**

```rust
pub struct AppConfig {
  pub streaming_enabled: bool,  // Default: true in dev, false in prod initially
  pub streaming_cache_enabled: bool,
  pub fallback_to_legacy: bool,  // If streaming fails, use old system
}
```

**Settings UI:**
```
Advanced Settings
└─ Use Streaming Playback (Beta): [✓]
   └─ If disabled, uses legacy full-decode system
```

---

## Success Criteria

### Phase 1 Success:
- ✅ First playback starts in <500ms (90th percentile)
- ✅ No audio glitches during streaming
- ✅ RAM usage <500MB per active song
- ✅ Seeking works with <300ms delay

### Phase 2 Success:
- ✅ Cached playback starts in <200ms (90th percentile)
- ✅ Cache persists across app restarts
- ✅ LRU eviction maintains cache size limit
- ✅ Cache hit rate >80% for typical usage

### Phase 3 Success:
- ✅ Parallel decode achieves <200ms first load
- ✅ Song transitions are seamless (<100ms)
- ✅ Seek feels instant (<100ms)
- ✅ Zero user-reported audio glitches

---

## Future Enhancements

### Post-Launch Improvements:

1. **Adaptive Chunk Size**
   - Larger chunks for high-bandwidth systems
   - Smaller chunks for low-memory devices

2. **Network Streaming**
   - Stream chunks from cloud storage
   - Collaborative setlists across devices

3. **Compression Options**
   - User choice: FLAC (quality) vs Opus (size)
   - Trade-off quality for storage

4. **Smart Cache Prediction**
   - ML model predicts which songs user will play
   - Pre-cache predicted songs

5. **Visualization**
   - Real-time waveform rendering from chunks
   - Buffer fill level indicator

---

## Conclusion

This streaming architecture solves the core problem: **instant playback** without requiring 50GB of RAM.

**Key Benefits:**
- ✅ 500ms → 100ms playback start (12x faster)
- ✅ 5GB → 231MB RAM per song (22x less)
- ✅ Supports random song jumping
- ✅ Scales to large setlists
- ✅ Professional-grade performance

**Implementation is phased and reversible**, reducing risk while delivering immediate value in Phase 1.

Ready to proceed with implementation!
