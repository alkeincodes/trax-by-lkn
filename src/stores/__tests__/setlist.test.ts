import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSetlistStore } from '../setlist'
import { useLibraryStore } from '../library'
import type { Setlist } from '@/types/library'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock VueUse
vi.mock('@vueuse/core', () => ({
  useDebounceFn: vi.fn((fn) => fn),
}))

import { invoke } from '@tauri-apps/api/core'

describe('Setlist Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes with empty state', () => {
    const store = useSetlistStore()

    expect(store.currentSetlist).toBeNull()
    expect(store.allSetlists).toEqual([])
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
  })

  it('fetches all setlists', async () => {
    const mockSetlists: Setlist[] = [
      {
        id: '1',
        name: 'Sunday Service',
        created_at: Date.now(),
        updated_at: Date.now(),
        song_ids: [],
      },
      {
        id: '2',
        name: 'Wednesday Practice',
        created_at: Date.now(),
        updated_at: Date.now(),
        song_ids: [],
      },
    ]

    vi.mocked(invoke).mockResolvedValue(mockSetlists)

    const store = useSetlistStore()
    await store.fetchSetlists()

    expect(invoke).toHaveBeenCalledWith('get_all_setlists')
    expect(store.allSetlists).toEqual(mockSetlists)
    expect(store.loading).toBe(false)
  })

  it('handles fetch error', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Network error'))

    const store = useSetlistStore()

    await expect(store.fetchSetlists()).rejects.toThrow('Network error')
    expect(store.error).toBe('Network error')
    expect(store.loading).toBe(false)
  })

  it('creates a new setlist', async () => {
    const mockId = 'new-setlist-id'
    vi.mocked(invoke).mockResolvedValue(mockId)

    const store = useSetlistStore()
    const id = await store.createSetlist('Sunday Service')

    expect(invoke).toHaveBeenCalledWith('create_setlist', { name: 'Sunday Service' })
    expect(id).toBe(mockId)
    expect(store.currentSetlist).toBeTruthy()
    expect(store.currentSetlist?.name).toBe('Sunday Service')
    expect(store.allSetlists).toHaveLength(1)
  })

  it('loads a setlist', async () => {
    const mockSetlist: Setlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: ['song1', 'song2'],
    }

    vi.mocked(invoke).mockResolvedValue(mockSetlist)

    const store = useSetlistStore()
    await store.loadSetlist('1')

    expect(invoke).toHaveBeenCalledWith('get_setlist', { id: '1' })
    expect(store.currentSetlist).toEqual(mockSetlist)
  })

  it('updates a setlist', async () => {
    const mockSetlist: Setlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: ['song1'],
    }

    vi.mocked(invoke).mockResolvedValue(undefined)

    const store = useSetlistStore()
    store.currentSetlist = mockSetlist
    store.allSetlists = [mockSetlist]

    await store.updateSetlist(mockSetlist)

    expect(invoke).toHaveBeenCalledWith('update_setlist', { setlist: mockSetlist })
  })

  it('deletes a setlist', async () => {
    const mockSetlist: Setlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: [],
    }

    vi.mocked(invoke).mockResolvedValue(undefined)

    const store = useSetlistStore()
    store.currentSetlist = mockSetlist
    store.allSetlists = [mockSetlist]

    await store.deleteSetlist('1')

    expect(invoke).toHaveBeenCalledWith('delete_setlist', { id: '1' })
    expect(store.currentSetlist).toBeNull()
    expect(store.allSetlists).toHaveLength(0)
  })

  it('adds song to setlist', async () => {
    const mockSetlist: Setlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: [],
    }

    vi.mocked(invoke).mockResolvedValue(undefined)

    const store = useSetlistStore()
    store.currentSetlist = mockSetlist

    await store.addSongToSetlist('song1')

    expect(invoke).toHaveBeenCalledWith('add_song_to_setlist', {
      setlistId: '1',
      songId: 'song1',
    })
    expect(store.currentSetlist.song_ids).toContain('song1')
  })

  it('removes song from setlist', async () => {
    const mockSetlist: Setlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: ['song1', 'song2'],
    }

    vi.mocked(invoke).mockResolvedValue(undefined)

    const store = useSetlistStore()
    store.currentSetlist = mockSetlist

    await store.removeSongFromSetlist('song1')

    expect(invoke).toHaveBeenCalledWith('remove_song_from_setlist', {
      setlistId: '1',
      songId: 'song1',
    })
    expect(store.currentSetlist.song_ids).not.toContain('song1')
    expect(store.currentSetlist.song_ids).toContain('song2')
  })

  it('reorders songs in setlist', async () => {
    const mockSetlist: Setlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: ['song1', 'song2', 'song3'],
    }

    vi.mocked(invoke).mockResolvedValue(undefined)

    const store = useSetlistStore()
    store.currentSetlist = mockSetlist

    await store.reorderSongs(0, 2)

    expect(invoke).toHaveBeenCalledWith('reorder_setlist_songs', {
      setlistId: '1',
      songIds: ['song2', 'song3', 'song1'],
    })
    expect(store.currentSetlist.song_ids).toEqual(['song2', 'song3', 'song1'])
  })

  it('returns current setlist songs', () => {
    const mockSetlist: Setlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: ['song1', 'song2'],
    }

    const setlistStore = useSetlistStore()
    const libraryStore = useLibraryStore()

    setlistStore.currentSetlist = mockSetlist
    libraryStore.songs = [
      {
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
      },
      {
        id: 'song2',
        name: 'How Great Thou Art',
        artist: 'Carl Boberg',
        duration: 300,
        tempo: 75,
        key: 'C',
        time_signature: '4/4',
    mixdown_path: null,
        created_at: Date.now(),
        updated_at: Date.now(),
      },
    ]

    const songs = setlistStore.currentSetlistSongs

    expect(songs).toHaveLength(2)
    expect(songs[0].name).toBe('Amazing Grace')
    expect(songs[1].name).toBe('How Great Thou Art')
  })

  it('returns recent setlists', () => {
    const now = Date.now()
    const mockSetlists: Setlist[] = [
      {
        id: '1',
        name: 'Old',
        created_at: now - 10000,
        updated_at: now - 10000,
        song_ids: [],
      },
      {
        id: '2',
        name: 'Recent',
        created_at: now - 1000,
        updated_at: now - 1000,
        song_ids: [],
      },
      {
        id: '3',
        name: 'Newest',
        created_at: now,
        updated_at: now,
        song_ids: [],
      },
    ]

    const store = useSetlistStore()
    store.allSetlists = mockSetlists

    const recent = store.recentSetlists

    expect(recent).toHaveLength(3)
    expect(recent[0].name).toBe('Newest')
    expect(recent[1].name).toBe('Recent')
    expect(recent[2].name).toBe('Old')
  })

  it('limits recent setlists to 5', () => {
    const now = Date.now()
    const mockSetlists: Setlist[] = Array.from({ length: 10 }, (_, i) => ({
      id: `${i}`,
      name: `Setlist ${i}`,
      created_at: now - (10 - i) * 1000,
      updated_at: now - (10 - i) * 1000,
      song_ids: [],
    }))

    const store = useSetlistStore()
    store.allSetlists = mockSetlists

    const recent = store.recentSetlists

    expect(recent).toHaveLength(5)
  })
})
