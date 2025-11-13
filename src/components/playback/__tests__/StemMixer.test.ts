import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import StemMixer from '../StemMixer.vue'
import { usePlaybackStore } from '@/stores/playback'
import type { Stem } from '@/types/library'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

// Stub StemRow component
vi.mock('../StemRow.vue', () => ({
  default: { name: 'StemRow', template: '<div class="stem-row" />', props: ['stem'] },
}))

describe('StemMixer', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders header', () => {
    const wrapper = mount(StemMixer)
    expect(wrapper.text()).toContain('Stem Mixer')
  })

  it('shows empty state when no stems', () => {
    const wrapper = mount(StemMixer)
    const store = usePlaybackStore()

    store.stems = []

    expect(wrapper.text()).toContain('No song loaded')
  })

  it('renders stem rows when stems are available', async () => {
    const wrapper = mount(StemMixer)
    const store = usePlaybackStore()

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
    ]

    store.setStems(mockStems)

    await wrapper.vm.$nextTick()

    // Should render stem rows (they will be rendered by StemRow component)
    expect(wrapper.text()).not.toContain('No song loaded')
  })

  it('updates when stems change', async () => {
    const wrapper = mount(StemMixer)
    const store = usePlaybackStore()

    // Initially no stems
    expect(wrapper.text()).toContain('No song loaded')

    // Add stems
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

    await wrapper.vm.$nextTick()

    expect(wrapper.text()).not.toContain('No song loaded')
  })
})
