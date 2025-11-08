import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import StemRow from '../StemRow.vue'
import { usePlaybackStore } from '@/stores/playback'
import type { Stem } from '@/types/library'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

// Stub Button component
vi.mock('@/components/ui/Button.vue', () => ({
  default: { name: 'Button', template: '<button><slot /></button>' },
}))

describe('StemRow', () => {
  let mockStem: Stem

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()

    mockStem = {
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
  })

  it('renders stem name', () => {
    const wrapper = mount(StemRow, {
      props: { stem: mockStem },
    })

    expect(wrapper.text()).toContain('Vocals')
  })

  it('displays volume percentage', () => {
    const wrapper = mount(StemRow, {
      props: { stem: mockStem },
    })

    expect(wrapper.text()).toContain('80%')
  })

  it('renders volume slider with correct value', () => {
    const wrapper = mount(StemRow, {
      props: { stem: mockStem },
    })

    const slider = wrapper.find('input[type="range"]')
    expect(slider.exists()).toBe(true)
    expect((slider.element as HTMLInputElement).value).toBe('80')
  })

  it('renders mute and solo buttons', () => {
    const wrapper = mount(StemRow, {
      props: { stem: mockStem },
    })

    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBe(2)
    expect(wrapper.text()).toContain('M')
    expect(wrapper.text()).toContain('S')
  })

  it('calls setStemVolume when volume slider changes', async () => {
    const store = usePlaybackStore()
    const setStemVolumeSpy = vi.spyOn(store, 'setStemVolume').mockResolvedValue()

    const wrapper = mount(StemRow, {
      props: { stem: mockStem },
    })

    const slider = wrapper.find('input[type="range"]')

    // Manually trigger the input event with the correct structure
    const inputEvent = new Event('input', { bubbles: true })
    Object.defineProperty(inputEvent, 'target', {
      value: { value: '60' },
      writable: false,
    })

    slider.element.dispatchEvent(inputEvent)
    await wrapper.vm.$nextTick()

    expect(setStemVolumeSpy).toHaveBeenCalledWith('stem-1', 0.6)
  })

  it('calls toggleStemMute when mute button clicked', async () => {
    const wrapper = mount(StemRow, {
      props: { stem: mockStem },
    })

    const store = usePlaybackStore()
    store.isPlaying = true // Enable buttons

    await wrapper.vm.$nextTick()

    const toggleMuteSpy = vi.spyOn(store, 'toggleStemMute').mockResolvedValue()

    const buttons = wrapper.findAll('button')
    const muteButton = buttons[0]
    await muteButton.trigger('click')

    expect(toggleMuteSpy).toHaveBeenCalledWith('stem-1')
  })

  it('calls toggleStemSolo when solo button clicked', async () => {
    const wrapper = mount(StemRow, {
      props: { stem: mockStem },
    })

    const store = usePlaybackStore()
    store.isPlaying = true // Enable buttons

    await wrapper.vm.$nextTick()

    const toggleSoloSpy = vi.spyOn(store, 'toggleStemSolo').mockResolvedValue()

    const buttons = wrapper.findAll('button')
    const soloButton = buttons[1]
    await soloButton.trigger('click')

    expect(toggleSoloSpy).toHaveBeenCalledWith('stem-1')
  })

  it('shows visual feedback when muted', async () => {
    const mutedStem = { ...mockStem, is_muted: true }
    const wrapper = mount(StemRow, {
      props: { stem: mutedStem },
    })

    const buttons = wrapper.findAll('button')
    const muteButton = buttons[0]

    expect(muteButton.classes()).toContain('bg-destructive')
  })

  it('disables controls when not playing', async () => {
    const wrapper = mount(StemRow, {
      props: { stem: mockStem },
    })

    const store = usePlaybackStore()
    store.isPlaying = false

    await wrapper.vm.$nextTick()

    const slider = wrapper.find('input[type="range"]')
    const buttons = wrapper.findAll('button')

    expect(slider.attributes('disabled')).toBeDefined()
    buttons.forEach(button => {
      expect(button.attributes('disabled')).toBeDefined()
    })
  })

  it('enables controls when playing', async () => {
    const wrapper = mount(StemRow, {
      props: { stem: mockStem },
    })

    const store = usePlaybackStore()
    store.isPlaying = true

    await wrapper.vm.$nextTick()

    const slider = wrapper.find('input[type="range"]')
    const buttons = wrapper.findAll('button')

    expect(slider.attributes('disabled')).toBeUndefined()
    buttons.forEach(button => {
      expect(button.attributes('disabled')).toBeUndefined()
    })
  })
})
