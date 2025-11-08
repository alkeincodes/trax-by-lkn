import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Song, Stem, SongFilter } from '@/types/library'

export const useLibraryStore = defineStore('library', () => {
  // State
  const songs = ref<Song[]>([])
  const searchQuery = ref('')
  const filters = ref<SongFilter>({})
  const loading = ref(false)
  const error = ref<string | null>(null)
  const viewMode = ref<'grid' | 'list'>('grid')

  // Getters
  const filteredSongs = computed(() => {
    let result = [...songs.value]

    // Apply search query
    if (searchQuery.value.trim()) {
      const query = searchQuery.value.toLowerCase()
      result = result.filter(
        (song) =>
          song.name.toLowerCase().includes(query) ||
          song.artist?.toLowerCase().includes(query)
      )
    }

    // Apply tempo filter
    if (filters.value.tempo_min !== undefined) {
      result = result.filter(
        (song) => song.tempo !== null && song.tempo >= filters.value.tempo_min!
      )
    }
    if (filters.value.tempo_max !== undefined) {
      result = result.filter(
        (song) => song.tempo !== null && song.tempo <= filters.value.tempo_max!
      )
    }

    // Apply key filter
    if (filters.value.key) {
      result = result.filter((song) => song.key === filters.value.key)
    }

    // Apply sorting
    if (filters.value.sort_by) {
      result.sort((a, b) => {
        switch (filters.value.sort_by) {
          case 'name':
            return a.name.localeCompare(b.name)
          case 'artist':
            return (a.artist || '').localeCompare(b.artist || '')
          case 'tempo':
            return (a.tempo || 0) - (b.tempo || 0)
          case 'duration':
            return a.duration - b.duration
          case 'date_added':
            return b.created_at - a.created_at
          default:
            return 0
        }
      })
    }

    return result
  })

  const songCount = computed(() => songs.value.length)
  const hasFilters = computed(
    () =>
      searchQuery.value.trim() !== '' ||
      filters.value.tempo_min !== undefined ||
      filters.value.tempo_max !== undefined ||
      filters.value.key !== undefined ||
      filters.value.sort_by !== undefined
  )

  // Actions
  async function fetchSongs() {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<Song[]>('get_all_songs')
      songs.value = result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to fetch songs:', e)
    } finally {
      loading.value = false
    }
  }

  async function searchSongs(query: string) {
    searchQuery.value = query
    loading.value = true
    error.value = null

    try {
      const result = await invoke<Song[]>('search_songs', { query })
      songs.value = result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to search songs:', e)
    } finally {
      loading.value = false
    }
  }

  async function filterSongs(newFilters: SongFilter) {
    filters.value = newFilters
    loading.value = true
    error.value = null

    try {
      const result = await invoke<Song[]>('filter_songs', {
        searchQuery: newFilters.search_query,
        tempoMin: newFilters.tempo_min,
        tempoMax: newFilters.tempo_max,
        key: newFilters.key,
        sortBy: newFilters.sort_by,
      })
      songs.value = result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to filter songs:', e)
    } finally {
      loading.value = false
    }
  }

  async function importFiles(
    filePaths: string[],
    title: string,
    artist?: string,
    key?: string,
    timeSignature?: string
  ): Promise<string> {
    loading.value = true
    error.value = null

    try {
      const songId = await invoke<string>('import_files', {
        filePaths,
        title,
        artist,
        key,
        timeSignature,
      })

      // Refresh library after import
      await fetchSongs()

      return songId
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to import files:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function getSongStems(songId: string): Promise<Stem[]> {
    try {
      return await invoke<Stem[]>('get_song_stems', { songId })
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to get song stems:', e)
      throw e
    }
  }

  async function deleteSong(songId: string) {
    loading.value = true
    error.value = null

    try {
      await invoke('delete_song', { songId })
      // Remove from local state
      songs.value = songs.value.filter((song) => song.id !== songId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to delete song:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function clearFilters() {
    searchQuery.value = ''
    filters.value = {}
    fetchSongs()
  }

  function setViewMode(mode: 'grid' | 'list') {
    viewMode.value = mode
  }

  return {
    // State
    songs,
    searchQuery,
    filters,
    loading,
    error,
    viewMode,

    // Getters
    filteredSongs,
    songCount,
    hasFilters,

    // Actions
    fetchSongs,
    searchSongs,
    filterSongs,
    importFiles,
    getSongStems,
    deleteSong,
    clearFilters,
    setViewMode,
  }
})
