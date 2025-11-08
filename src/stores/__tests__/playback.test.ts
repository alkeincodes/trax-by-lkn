import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePlaybackStore } from '../playback'
import type { Song, Stem } from '@/types/library'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

const { invoke } = await import('@tauri-apps/api/core')

describe('Playback Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes with default state', () => {
    const store = usePlaybackStore()

    expect(store.currentSong).toBeNull()
    expect(store.isPlaying).toBe(false)
    expect(store.currentPosition).toBe(0)
    expect(store.duration).toBe(0)
    expect(store.stems).toEqual([])
    expect(store.volume).toBe(0.8)
  })

  it('plays song successfully', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined)

    const store = usePlaybackStore()
    await store.playSong('song-123')

    expect(invoke).toHaveBeenCalledWith('play_song', { songId: 'song-123' })
    expect(store.isPlaying).toBe(true)
  })

  it('pauses playback', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined)

    const store = usePlaybackStore()
    store.isPlaying = true

    await store.pause()

    expect(invoke).toHaveBeenCalledWith('pause_playback')
    expect(store.isPlaying).toBe(false)
  })

  it('stops playback and resets position', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined)

    const store = usePlaybackStore()
    store.isPlaying = true
    store.currentPosition = 30

    await store.stop()

    expect(invoke).toHaveBeenCalledWith('stop_playback')
    expect(store.isPlaying).toBe(false)
    expect(store.currentPosition).toBe(0)
  })

  it('seeks to position', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined)

    const store = usePlaybackStore()
    await store.seek(45)

    expect(invoke).toHaveBeenCalledWith('seek_to_position', { position: 45 })
    expect(store.currentPosition).toBe(45)
  })

  it('sets volume within valid range', () => {
    const store = usePlaybackStore()

    store.setVolume(0.5)
    expect(store.volume).toBe(0.5)

    store.setVolume(1.5)
    expect(store.volume).toBe(1) // Clamped to max

    store.setVolume(-0.5)
    expect(store.volume).toBe(0) // Clamped to min
  })

  it('sets stem volume and updates local state', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined)

    const mockStem: Stem = {
      id: 'stem-1',
      song_id: 'song-1',
      name: 'Vocals',
      file_path: '/path/to/vocals.wav',
      file_size: 1024,
      sample_rate: 44100,
      channels: 2,
      duration: 180,
      volume: 0.8,
      is_muted: false,
    }

    const store = usePlaybackStore()
    store.stems = [mockStem]

    await store.setStemVolume('stem-1', 0.6)

    expect(invoke).toHaveBeenCalledWith('set_stem_volume', {
      stemId: 'stem-1',
      volume: 0.6,
    })
    expect(store.stems[0].volume).toBe(0.6)
  })

  it('toggles stem mute', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined)

    const mockStem: Stem = {
      id: 'stem-1',
      song_id: 'song-1',
      name: 'Vocals',
      file_path: '/path/to/vocals.wav',
      file_size: 1024,
      sample_rate: 44100,
      channels: 2,
      duration: 180,
      volume: 0.8,
      is_muted: false,
    }

    const store = usePlaybackStore()
    store.stems = [mockStem]

    await store.toggleStemMute('stem-1')

    expect(invoke).toHaveBeenCalledWith('toggle_stem_mute', { stemId: 'stem-1' })
    expect(store.stems[0].is_muted).toBe(true)
  })

  it('formats position time correctly', () => {
    const store = usePlaybackStore()

    store.currentPosition = 0
    expect(store.formattedPosition).toBe('0:00')

    store.currentPosition = 65
    expect(store.formattedPosition).toBe('1:05')

    store.currentPosition = 125
    expect(store.formattedPosition).toBe('2:05')
  })

  it('formats duration time correctly', () => {
    const store = usePlaybackStore()

    store.duration = 0
    expect(store.formattedDuration).toBe('0:00')

    store.duration = 180
    expect(store.formattedDuration).toBe('3:00')

    store.duration = 195
    expect(store.formattedDuration).toBe('3:15')
  })

  it('calculates progress percentage correctly', () => {
    const store = usePlaybackStore()

    store.duration = 100
    store.currentPosition = 0
    expect(store.progress).toBe(0)

    store.currentPosition = 50
    expect(store.progress).toBe(50)

    store.currentPosition = 100
    expect(store.progress).toBe(100)
  })

  it('returns 0 progress when duration is 0', () => {
    const store = usePlaybackStore()

    store.duration = 0
    store.currentPosition = 50
    expect(store.progress).toBe(0)
  })

  it('updates position from events', () => {
    const store = usePlaybackStore()

    store.updatePosition(42)
    expect(store.currentPosition).toBe(42)
  })

  it('updates playback state from events', () => {
    const store = usePlaybackStore()

    store.updatePlaybackState(true)
    expect(store.isPlaying).toBe(true)

    store.updatePlaybackState(false)
    expect(store.isPlaying).toBe(false)
  })

  it('sets current song and updates duration', () => {
    const mockSong: Song = {
      id: 'song-1',
      name: 'Test Song',
      artist: 'Test Artist',
      duration: 200,
      tempo: 120,
      key: 'C',
      time_signature: '4/4',
      created_at: Date.now(),
      updated_at: Date.now(),
    }

    const store = usePlaybackStore()
    store.setCurrentSong(mockSong)

    expect(store.currentSong).toEqual(mockSong)
    expect(store.duration).toBe(200)
  })

  it('clears state when setting null song', () => {
    const store = usePlaybackStore()
    store.duration = 100
    store.currentPosition = 50
    store.stems = [{} as Stem]

    store.setCurrentSong(null)

    expect(store.currentSong).toBeNull()
    expect(store.duration).toBe(0)
    expect(store.currentPosition).toBe(0)
    expect(store.stems).toEqual([])
  })

  it('sets stems', () => {
    const mockStems: Stem[] = [
      {
        id: 'stem-1',
        song_id: 'song-1',
        name: 'Vocals',
        file_path: '/path/to/vocals.wav',
        file_size: 1024,
        sample_rate: 44100,
        channels: 2,
        duration: 180,
        volume: 0.8,
        is_muted: false,
      },
    ]

    const store = usePlaybackStore()
    store.setStems(mockStems)

    expect(store.stems).toEqual(mockStems)
  })
})
