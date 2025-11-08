import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useLibraryStore } from '../library'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const { invoke } = await import('@tauri-apps/api/core')

describe('Library Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes with empty state', () => {
    const store = useLibraryStore()

    expect(store.songs).toEqual([])
    expect(store.searchQuery).toBe('')
    expect(store.filters).toEqual({})
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
    expect(store.viewMode).toBe('grid')
  })

  it('fetches songs successfully', async () => {
    const mockSongs = [
      {
        id: '1',
        name: 'Test Song',
        artist: 'Test Artist',
        duration: 180,
        tempo: 120,
        key: 'C',
        time_signature: '4/4',
        created_at: Date.now(),
        updated_at: Date.now(),
      },
    ]

    vi.mocked(invoke).mockResolvedValueOnce(mockSongs)

    const store = useLibraryStore()
    await store.fetchSongs()

    expect(store.songs).toEqual(mockSongs)
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
  })

  it('handles fetch errors', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'))

    const store = useLibraryStore()
    await store.fetchSongs()

    expect(store.songs).toEqual([])
    expect(store.error).toBe('Network error')
    expect(store.loading).toBe(false)
  })

  it('filters songs by search query', () => {
    const store = useLibraryStore()
    store.songs = [
      {
        id: '1',
        name: 'Amazing Grace',
        artist: 'John Newton',
        duration: 180,
        tempo: 120,
        key: 'C',
        time_signature: '4/4',
        created_at: Date.now(),
        updated_at: Date.now(),
      },
      {
        id: '2',
        name: 'How Great Thou Art',
        artist: 'Carl Boberg',
        duration: 200,
        tempo: 100,
        key: 'G',
        time_signature: '4/4',
        created_at: Date.now(),
        updated_at: Date.now(),
      },
    ]

    store.searchQuery = 'amazing'

    expect(store.filteredSongs.length).toBe(1)
    expect(store.filteredSongs[0].name).toBe('Amazing Grace')
  })

  it('filters songs by tempo range', () => {
    const store = useLibraryStore()
    store.songs = [
      {
        id: '1',
        name: 'Fast Song',
        artist: null,
        duration: 180,
        tempo: 140,
        key: null,
        time_signature: null,
        created_at: Date.now(),
        updated_at: Date.now(),
      },
      {
        id: '2',
        name: 'Slow Song',
        artist: null,
        duration: 200,
        tempo: 80,
        key: null,
        time_signature: null,
        created_at: Date.now(),
        updated_at: Date.now(),
      },
    ]

    store.filters = { tempo_min: 100, tempo_max: 150 }

    expect(store.filteredSongs.length).toBe(1)
    expect(store.filteredSongs[0].name).toBe('Fast Song')
  })

  it('toggles view mode', () => {
    const store = useLibraryStore()

    expect(store.viewMode).toBe('grid')

    store.setViewMode('list')
    expect(store.viewMode).toBe('list')

    store.setViewMode('grid')
    expect(store.viewMode).toBe('grid')
  })

  it('clears filters correctly', async () => {
    const store = useLibraryStore()
    store.searchQuery = 'test'
    store.filters = { tempo_min: 100 }

    vi.mocked(invoke).mockResolvedValueOnce([])

    await store.clearFilters()

    expect(store.searchQuery).toBe('')
    expect(store.filters).toEqual({})
  })

  it('computes hasFilters correctly', () => {
    const store = useLibraryStore()

    expect(store.hasFilters).toBe(false)

    store.searchQuery = 'test'
    expect(store.hasFilters).toBe(true)

    store.searchQuery = ''
    store.filters = { tempo_min: 100 }
    expect(store.hasFilters).toBe(true)
  })
})
