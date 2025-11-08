import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import SetlistView from '../SetlistView.vue'
import { useSetlistStore } from '@/stores/setlist'
import { useLibraryStore } from '@/stores/library'

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock VueUse
vi.mock('@vueuse/core', () => ({
  useDebounceFn: vi.fn((fn) => fn),
}))

describe('SetlistView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders toolbar', () => {
    const wrapper = mount(SetlistView, {
      global: {
        stubs: {
          SetlistToolbar: true,
          SetlistItem: true,
          Button: true,
        },
      },
    })

    expect(wrapper.findComponent({ name: 'SetlistToolbar' }).exists()).toBe(true)
  })

  it('shows empty state when no setlist is loaded', () => {
    const wrapper = mount(SetlistView, {
      global: {
        stubs: {
          SetlistToolbar: true,
          Button: true,
        },
      },
    })

    expect(wrapper.text()).toContain('No Setlist Loaded')
    expect(wrapper.text()).toContain('Create a new setlist')
  })

  it('shows empty setlist message when setlist has no songs', () => {
    const store = useSetlistStore()
    store.currentSetlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: [],
    }

    const wrapper = mount(SetlistView, {
      global: {
        stubs: {
          SetlistToolbar: true,
          Button: true,
        },
      },
    })

    expect(wrapper.text()).toContain('Setlist is empty')
    expect(wrapper.text()).toContain('Drag songs from the library')
  })

  it('renders setlist items when songs are present', () => {
    const store = useSetlistStore()
    const libraryStore = useLibraryStore()

    store.currentSetlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: ['song1', 'song2'],
    }

    libraryStore.songs = [
      {
        id: 'song1',
        name: 'Amazing Grace',
        artist: 'John Newton',
        duration: 240,
        tempo: 80,
        key: 'G',
        time_signature: '4/4',
        created_at: Date.now(),
        updated_at: Date.now(),
      },
      {
        id: 'song2',
        name: 'How Great Thou Art',
        artist: 'Carl Boberg',
        duration: 300,
        tempo: 75,
        key: 'C',
        time_signature: '4/4',
        created_at: Date.now(),
        updated_at: Date.now(),
      },
    ]

    const wrapper = mount(SetlistView, {
      global: {
        stubs: {
          SetlistToolbar: true,
          SetlistItem: {
            template: '<div class="setlist-item">{{ song.name }}</div>',
            props: ['song', 'index'],
          },
        },
      },
    })

    const items = wrapper.findAll('.setlist-item')
    expect(items).toHaveLength(2)
    expect(wrapper.text()).toContain('Amazing Grace')
    expect(wrapper.text()).toContain('How Great Thou Art')
  })

  it('calls fetchSetlists on mount', () => {
    const store = useSetlistStore()
    vi.spyOn(store, 'fetchSetlists')

    mount(SetlistView, {
      global: {
        stubs: {
          SetlistToolbar: true,
          Button: true,
        },
      },
    })

    expect(store.fetchSetlists).toHaveBeenCalled()
  })
})
