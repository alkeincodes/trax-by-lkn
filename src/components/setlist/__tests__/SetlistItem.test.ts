import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import SetlistItem from '../SetlistItem.vue'
import type { Song } from '@/types/library'

const mockSong: Song = {
  id: 'song1',
  name: 'Amazing Grace',
  artist: 'John Newton',
  duration: 240,
  tempo: 80,
  key: 'G',
  time_signature: '4/4',
    mixdown_path: null,
  created_at: Date.now(),
  updated_at: Date.now(),
}

describe('SetlistItem', () => {
  it('renders song information correctly', () => {
    const wrapper = mount(SetlistItem, {
      props: {
        song: mockSong,
        index: 0,
      },
    })

    expect(wrapper.text()).toContain('Amazing Grace')
    expect(wrapper.text()).toContain('John Newton')
    expect(wrapper.text()).toContain('G')
    expect(wrapper.text()).toContain('80 BPM')
    expect(wrapper.text()).toContain('4:00')
  })

  it('is draggable', () => {
    const wrapper = mount(SetlistItem, {
      props: {
        song: mockSong,
        index: 0,
      },
    })

    const element = wrapper.find('div[draggable]')
    expect(element.exists()).toBe(true)
    expect(element.attributes('draggable')).toBe('true')
  })

  it('emits remove event when delete button is clicked', async () => {
    const wrapper = mount(SetlistItem, {
      props: {
        song: mockSong,
        index: 0,
      },
    })

    const removeButton = wrapper.findAll('button')[1] // Second button is remove
    await removeButton.trigger('click')

    expect(wrapper.emitted('remove')).toBeTruthy()
  })

  it('emits dragstart event with index', async () => {
    const wrapper = mount(SetlistItem, {
      props: {
        song: mockSong,
        index: 2,
      },
    })

    const dragEvent = new DragEvent('dragstart', {
      dataTransfer: new DataTransfer(),
    })

    await wrapper.find('div[draggable]').element.dispatchEvent(dragEvent)

    expect(wrapper.emitted('dragstart')).toBeTruthy()
    expect(wrapper.emitted('dragstart')?.[0]).toEqual([2])
  })

  it('emits dragend event', async () => {
    const wrapper = mount(SetlistItem, {
      props: {
        song: mockSong,
        index: 0,
      },
    })

    await wrapper.find('div[draggable]').trigger('dragend')

    expect(wrapper.emitted('dragend')).toBeTruthy()
  })

  it('emits drop event with target index', async () => {
    const wrapper = mount(SetlistItem, {
      props: {
        song: mockSong,
        index: 1,
      },
    })

    const dropEvent = new DragEvent('drop', {
      dataTransfer: new DataTransfer(),
    })

    await wrapper.find('div[draggable]').element.dispatchEvent(dropEvent)

    expect(wrapper.emitted('drop')).toBeTruthy()
    expect(wrapper.emitted('drop')?.[0]).toEqual([1])
  })

  it('formats duration correctly', () => {
    const song = { ...mockSong, duration: 65 }
    const wrapper = mount(SetlistItem, {
      props: {
        song,
        index: 0,
      },
    })

    expect(wrapper.text()).toContain('1:05')
  })

  it('handles missing optional fields gracefully', () => {
    const song: Song = {
      id: 'song2',
      name: 'Test Song',
      artist: null,
      duration: 180,
      tempo: null,
      key: null,
      time_signature: null,
    mixdown_path: null,
      created_at: Date.now(),
      updated_at: Date.now(),
    }

    const wrapper = mount(SetlistItem, {
      props: {
        song,
        index: 0,
      },
    })

    expect(wrapper.text()).toContain('Test Song')
    expect(wrapper.text()).not.toContain('BPM')
  })
})
