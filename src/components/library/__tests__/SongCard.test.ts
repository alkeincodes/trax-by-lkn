import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import SongCard from '../SongCard.vue'
import type { Song } from '@/types/library'

describe('SongCard', () => {
  const mockSong: Song = {
    id: '1',
    name: 'Amazing Grace',
    artist: 'John Newton',
    duration: 185,
    tempo: 120,
    key: 'C',
    time_signature: '4/4',
    mixdown_path: null,
    created_at: Date.now(),
    updated_at: Date.now(),
  }

  it('renders song information correctly', () => {
    const wrapper = mount(SongCard, {
      props: { song: mockSong },
    })

    expect(wrapper.text()).toContain('Amazing Grace')
    expect(wrapper.text()).toContain('John Newton')
    expect(wrapper.text()).toContain('C')
    expect(wrapper.text()).toContain('120 BPM')
  })

  it('formats duration correctly', () => {
    const wrapper = mount(SongCard, {
      props: { song: mockSong },
    })

    expect(wrapper.text()).toContain('3:05')
  })

  it('displays "Unknown Artist" when artist is null', () => {
    const songWithoutArtist = { ...mockSong, artist: null }
    const wrapper = mount(SongCard, {
      props: { song: songWithoutArtist },
    })

    expect(wrapper.text()).toContain('Unknown Artist')
  })

  it('displays stem count when provided', () => {
    const wrapper = mount(SongCard, {
      props: {
        song: mockSong,
        stemCount: 8,
      },
    })

    expect(wrapper.text()).toContain('8 stems')
  })

  it('emits select event when clicked', async () => {
    const wrapper = mount(SongCard, {
      props: { song: mockSong },
    })

    await wrapper.trigger('click')

    expect(wrapper.emitted('select')).toBeTruthy()
    expect(wrapper.emitted('select')?.[0]).toEqual([mockSong])
  })

  it('does not display key badge when key is null', () => {
    const songWithoutKey = { ...mockSong, key: null }
    const wrapper = mount(SongCard, {
      props: { song: songWithoutKey },
    })

    expect(wrapper.text()).not.toContain('C')
  })

  it('does not display tempo badge when tempo is null', () => {
    const songWithoutTempo = { ...mockSong, tempo: null }
    const wrapper = mount(SongCard, {
      props: { song: songWithoutTempo },
    })

    expect(wrapper.text()).not.toContain('BPM')
  })
})
