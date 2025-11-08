import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import SeekBar from '../SeekBar.vue'
import { usePlaybackStore } from '@/stores/playback'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

describe('SeekBar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders seek bar', () => {
    const wrapper = mount(SeekBar)
    expect(wrapper.find('.bg-secondary').exists()).toBe(true)
  })

  it('displays progress percentage correctly', async () => {
    const wrapper = mount(SeekBar)
    const store = usePlaybackStore()

    store.duration = 100
    store.currentPosition = 50

    await wrapper.vm.$nextTick()

    const progressBar = wrapper.find('.bg-primary')
    expect(progressBar.attributes('style')).toContain('width: 50%')
  })

  it('updates progress as position changes', async () => {
    const wrapper = mount(SeekBar)
    const store = usePlaybackStore()

    store.duration = 200
    store.currentPosition = 50

    await wrapper.vm.$nextTick()
    let progressBar = wrapper.find('.bg-primary')
    expect(progressBar.attributes('style')).toContain('width: 25%')

    store.currentPosition = 100
    await wrapper.vm.$nextTick()
    progressBar = wrapper.find('.bg-primary')
    expect(progressBar.attributes('style')).toContain('width: 50%')
  })

  it('calls seek on click', async () => {
    const wrapper = mount(SeekBar)
    const store = usePlaybackStore()

    store.duration = 100
    const seekSpy = vi.spyOn(store, 'seek').mockResolvedValue()

    // Find seek bar element
    const seekBar = wrapper.find('.bg-secondary')
    const element = seekBar.element

    // Mock getBoundingClientRect
    vi.spyOn(element, 'getBoundingClientRect').mockReturnValue({
      left: 0,
      width: 100,
      top: 0,
      right: 100,
      bottom: 10,
      height: 10,
      x: 0,
      y: 0,
      toJSON: () => ({}),
    })

    // Trigger click with clientX
    await seekBar.trigger('click', { clientX: 50 })

    expect(seekSpy).toHaveBeenCalled()
  })

  it('handles mouse down to start dragging', async () => {
    const wrapper = mount(SeekBar)
    const store = usePlaybackStore()

    store.duration = 100
    const seekSpy = vi.spyOn(store, 'seek').mockResolvedValue()

    const seekBar = wrapper.find('.bg-secondary')
    const element = seekBar.element

    // Mock getBoundingClientRect
    vi.spyOn(element, 'getBoundingClientRect').mockReturnValue({
      left: 0,
      width: 100,
      top: 0,
      right: 100,
      bottom: 10,
      height: 10,
      x: 0,
      y: 0,
      toJSON: () => ({}),
    })

    await seekBar.trigger('mousedown', { clientX: 25 })

    expect(seekSpy).toHaveBeenCalled()
  })

  it('shows indicator when dragging', async () => {
    const wrapper = mount(SeekBar)
    const store = usePlaybackStore()

    store.duration = 100

    // Initially indicator should be hidden
    let indicator = wrapper.find('.rounded-full.bg-primary.shadow-md')
    expect(indicator.classes()).toContain('opacity-0')

    // Simulate mousedown to start dragging
    const seekBar = wrapper.find('.bg-secondary')
    const element = seekBar.element

    // Mock getBoundingClientRect
    vi.spyOn(element, 'getBoundingClientRect').mockReturnValue({
      left: 0,
      width: 100,
      top: 0,
      right: 100,
      bottom: 10,
      height: 10,
      x: 0,
      y: 0,
      toJSON: () => ({}),
    })

    await seekBar.trigger('mousedown', { clientX: 50 })

    await wrapper.vm.$nextTick()

    // Indicator should be visible when dragging
    indicator = wrapper.find('.rounded-full.bg-primary.shadow-md')
    expect(indicator.classes()).toContain('opacity-100')
  })

  it('handles zero duration gracefully', async () => {
    const wrapper = mount(SeekBar)
    const store = usePlaybackStore()

    store.duration = 0
    store.currentPosition = 0

    await wrapper.vm.$nextTick()

    const progressBar = wrapper.find('.bg-primary')
    // Should not crash, progress should be 0
    expect(progressBar.exists()).toBe(true)
  })
})
