import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useKeyboardShortcuts } from '../useKeyboardShortcuts'
import { usePlaybackStore } from '@/stores/playback'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

describe('useKeyboardShortcuts', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('toggles play/pause on space key', () => {
    const store = usePlaybackStore()
    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
    mixdown_path: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    const playSpy = vi.spyOn(store, 'playSong').mockResolvedValue()
    const pauseSpy = vi.spyOn(store, 'pause').mockResolvedValue()

    const { handleKeyDown } = useKeyboardShortcuts()

    // Test play
    const spaceEvent = new KeyboardEvent('keydown', { key: ' ' })
    handleKeyDown(spaceEvent)
    expect(playSpy).toHaveBeenCalledWith('song-1')

    // Test pause
    store.isPlaying = true
    handleKeyDown(spaceEvent)
    expect(pauseSpy).toHaveBeenCalled()
  })

  it('seeks backward on left arrow key', () => {
    const store = usePlaybackStore()
    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
    mixdown_path: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    store.currentPosition = 30
    const seekSpy = vi.spyOn(store, 'seek').mockResolvedValue()

    const { handleKeyDown } = useKeyboardShortcuts()

    const leftEvent = new KeyboardEvent('keydown', { key: 'ArrowLeft' })
    handleKeyDown(leftEvent)

    expect(seekSpy).toHaveBeenCalledWith(25)
  })

  it('seeks forward on right arrow key', () => {
    const store = usePlaybackStore()
    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
    mixdown_path: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    store.currentPosition = 30
    store.duration = 180
    const seekSpy = vi.spyOn(store, 'seek').mockResolvedValue()

    const { handleKeyDown } = useKeyboardShortcuts()

    const rightEvent = new KeyboardEvent('keydown', { key: 'ArrowRight' })
    handleKeyDown(rightEvent)

    expect(seekSpy).toHaveBeenCalledWith(35)
  })

  it('does not seek before 0', () => {
    const store = usePlaybackStore()
    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
    mixdown_path: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    store.currentPosition = 2
    const seekSpy = vi.spyOn(store, 'seek').mockResolvedValue()

    const { handleKeyDown } = useKeyboardShortcuts()

    const leftEvent = new KeyboardEvent('keydown', { key: 'ArrowLeft' })
    handleKeyDown(leftEvent)

    expect(seekSpy).toHaveBeenCalledWith(0)
  })

  it('does not seek beyond duration', () => {
    const store = usePlaybackStore()
    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 100,
      tempo: null,
      key: null,
      time_signature: null,
    mixdown_path: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    store.currentPosition = 98
    store.duration = 100
    const seekSpy = vi.spyOn(store, 'seek').mockResolvedValue()

    const { handleKeyDown } = useKeyboardShortcuts()

    const rightEvent = new KeyboardEvent('keydown', { key: 'ArrowRight' })
    handleKeyDown(rightEvent)

    expect(seekSpy).toHaveBeenCalledWith(100)
  })

  it('mutes all stems on M key', () => {
    const store = usePlaybackStore()
    store.setStems([
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
    display_order: 0,
      },
      {
        id: 'stem-2',
        song_id: 'song-1',
        name: 'Drums',
        file_path: '/path/to/drums.wav',
        file_size: 2048,
        sample_rate: 44100,
        channels: 2,
        duration: 180,
        volume: 0.7,
        is_muted: false,
    display_order: 0,
      },
    ])

    const toggleMuteSpy = vi.spyOn(store, 'toggleStemMute').mockResolvedValue()

    const { handleKeyDown } = useKeyboardShortcuts()

    const mEvent = new KeyboardEvent('keydown', { key: 'M' })
    handleKeyDown(mEvent)

    expect(toggleMuteSpy).toHaveBeenCalledTimes(2)
    expect(toggleMuteSpy).toHaveBeenCalledWith('stem-1')
    expect(toggleMuteSpy).toHaveBeenCalledWith('stem-2')
  })

  it('solos first stem on S key', () => {
    const store = usePlaybackStore()
    store.setStems([
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
    display_order: 0,
      },
    ])

    const toggleSoloSpy = vi.spyOn(store, 'toggleStemSolo').mockResolvedValue()

    const { handleKeyDown } = useKeyboardShortcuts()

    const sEvent = new KeyboardEvent('keydown', { key: 's' })
    handleKeyDown(sEvent)

    expect(toggleSoloSpy).toHaveBeenCalledWith('stem-1')
  })

  it('ignores keys when typing in input', () => {
    const store = usePlaybackStore()
    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
    mixdown_path: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    const playSpy = vi.spyOn(store, 'playSong').mockResolvedValue()

    const { handleKeyDown } = useKeyboardShortcuts()

    const input = document.createElement('input')
    const spaceEvent = new KeyboardEvent('keydown', {
      key: ' ',
      bubbles: true,
    })
    Object.defineProperty(spaceEvent, 'target', { value: input, enumerable: true })

    handleKeyDown(spaceEvent)

    expect(playSpy).not.toHaveBeenCalled()
  })

  it('ignores keys when typing in textarea', () => {
    const store = usePlaybackStore()
    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
    mixdown_path: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    const playSpy = vi.spyOn(store, 'playSong').mockResolvedValue()

    const { handleKeyDown } = useKeyboardShortcuts()

    const textarea = document.createElement('textarea')
    const spaceEvent = new KeyboardEvent('keydown', {
      key: ' ',
      bubbles: true,
    })
    Object.defineProperty(spaceEvent, 'target', {
      value: textarea,
      enumerable: true,
    })

    handleKeyDown(spaceEvent)

    expect(playSpy).not.toHaveBeenCalled()
  })
})
