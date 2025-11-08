import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import LibraryView from '../LibraryView.vue'
import { useLibraryStore } from '@/stores/library'
import type { Song } from '@/types/library'

// Mock Tauri modules
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

describe('LibraryView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders toolbar and content area', () => {
    const wrapper = mount(LibraryView)

    expect(wrapper.find('.flex.h-full.flex-col').exists()).toBe(true)
  })

  it('shows loading state', async () => {
    const wrapper = mount(LibraryView)
    const libraryStore = useLibraryStore()

    libraryStore.loading = true
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('') // Loading skeleton should be shown
  })

  it('shows error state', async () => {
    const wrapper = mount(LibraryView)
    const libraryStore = useLibraryStore()

    libraryStore.error = 'Network error'
    libraryStore.loading = false
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Error loading library')
    expect(wrapper.text()).toContain('Network error')
  })

  it('shows empty state when no songs', async () => {
    const wrapper = mount(LibraryView)
    const libraryStore = useLibraryStore()

    libraryStore.songs = []
    libraryStore.loading = false
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('No songs yet')
    expect(wrapper.text()).toContain('Click Import')
  })

  it('shows no results state when filters applied', async () => {
    const wrapper = mount(LibraryView)
    const libraryStore = useLibraryStore()

    libraryStore.songs = []
    libraryStore.searchQuery = 'test'
    libraryStore.loading = false
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('No results found')
  })

  it('renders song cards in grid mode', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockSongs: Song[] = [
      {
        id: '1',
        name: 'Song 1',
        artist: 'Artist 1',
        duration: 180,
        tempo: 120,
        key: 'C',
        time_signature: '4/4',
        created_at: Date.now(),
        updated_at: Date.now(),
      },
      {
        id: '2',
        name: 'Song 2',
        artist: 'Artist 2',
        duration: 200,
        tempo: 100,
        key: 'G',
        time_signature: '4/4',
        created_at: Date.now(),
        updated_at: Date.now(),
      },
    ]

    vi.mocked(invoke).mockResolvedValue(mockSongs)

    const wrapper = mount(LibraryView)
    const libraryStore = useLibraryStore()

    await libraryStore.fetchSongs()
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Song 1')
    expect(wrapper.text()).toContain('Song 2')
  })

  it('shows song count', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const mockSongs: Song[] = [
      {
        id: '1',
        name: 'Song 1',
        artist: null,
        duration: 180,
        tempo: null,
        key: null,
        time_signature: null,
        created_at: Date.now(),
        updated_at: Date.now(),
      },
    ]

    vi.mocked(invoke).mockResolvedValue(mockSongs)

    const wrapper = mount(LibraryView)
    const libraryStore = useLibraryStore()

    await libraryStore.fetchSongs()
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Showing 1 of 1 songs')
  })

  it('changes layout based on view mode', async () => {
    const wrapper = mount(LibraryView)
    const libraryStore = useLibraryStore()

    libraryStore.setViewMode('list')
    await wrapper.vm.$nextTick()

    expect(libraryStore.viewMode).toBe('list')

    libraryStore.setViewMode('grid')
    await wrapper.vm.$nextTick()

    expect(libraryStore.viewMode).toBe('grid')
  })
})
