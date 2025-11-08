import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import PlaybackControls from '../PlaybackControls.vue'
import { usePlaybackStore } from '@/stores/playback'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

// Mock Lucide icons
vi.mock('lucide-vue-next', () => ({
  SkipBack: { name: 'SkipBack', template: '<div />' },
  Play: { name: 'Play', template: '<div />' },
  Pause: { name: 'Pause', template: '<div />' },
  Square: { name: 'Square', template: '<div />' },
  SkipForward: { name: 'SkipForward', template: '<div />' },
}))

describe('PlaybackControls', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders all transport buttons', () => {
    const wrapper = mount(PlaybackControls)

    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBeGreaterThanOrEqual(4) // Previous, Play, Stop, Next
  })

  it('disables buttons when no song is loaded', () => {
    const wrapper = mount(PlaybackControls)
    const store = usePlaybackStore()

    store.currentSong = null

    const buttons = wrapper.findAll('button')
    buttons.forEach(button => {
      expect(button.attributes('disabled')).toBeDefined()
    })
  })

  it('enables buttons when song is loaded', async () => {
    const wrapper = mount(PlaybackControls)
    const store = usePlaybackStore()

    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: 'Test Artist',
      duration: 180,
      tempo: 120,
      key: 'C',
      time_signature: '4/4',
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    await wrapper.vm.$nextTick()

    const buttons = wrapper.findAll('button')
    buttons.forEach(button => {
      expect(button.attributes('disabled')).toBeUndefined()
    })
  })

  it('displays formatted time correctly', async () => {
    const wrapper = mount(PlaybackControls)
    const store = usePlaybackStore()

    store.currentPosition = 65
    store.duration = 200

    await wrapper.vm.$nextTick()

    const text = wrapper.text()
    expect(text).toContain('1:05')
    expect(text).toContain('3:20')
  })

  it('toggles play/pause on button click', async () => {
    const wrapper = mount(PlaybackControls)
    const store = usePlaybackStore()

    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    const playSpy = vi.spyOn(store, 'playSong').mockResolvedValue()
    const pauseSpy = vi.spyOn(store, 'pause').mockResolvedValue()

    await wrapper.vm.$nextTick()

    // Find play button and click it
    const playButton = wrapper.findAll('button')[1] // Second button is play/pause
    await playButton.trigger('click')

    expect(playSpy).toHaveBeenCalledWith('song-1')

    // Simulate playing state
    store.isPlaying = true
    await wrapper.vm.$nextTick()

    // Click again to pause
    await playButton.trigger('click')
    expect(pauseSpy).toHaveBeenCalled()
  })

  it('calls stop on stop button click', async () => {
    const wrapper = mount(PlaybackControls)
    const store = usePlaybackStore()

    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    const stopSpy = vi.spyOn(store, 'stop').mockResolvedValue()

    await wrapper.vm.$nextTick()

    // Find stop button and click it
    const stopButton = wrapper.findAll('button')[2] // Third button is stop
    await stopButton.trigger('click')

    expect(stopSpy).toHaveBeenCalled()
  })

  it('displays key and tempo when available', async () => {
    const wrapper = mount(PlaybackControls)
    const store = usePlaybackStore()

    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: 'Test Artist',
      duration: 180,
      tempo: 120,
      key: 'C',
      time_signature: '4/4',
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    await wrapper.vm.$nextTick()

    const text = wrapper.text()
    expect(text).toContain('C')
    expect(text).toContain('120')
    expect(text).toContain('4/4')
  })

  it('hides key and tempo when not available', async () => {
    const wrapper = mount(PlaybackControls)
    const store = usePlaybackStore()

    store.setCurrentSong({
      id: 'song-1',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    })

    await wrapper.vm.$nextTick()

    const text = wrapper.text()
    expect(text).not.toContain('Key:')
    expect(text).not.toContain('Tempo:')
  })
})
