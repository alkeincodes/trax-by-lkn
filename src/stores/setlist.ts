import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useDebounceFn } from '@vueuse/core'
import type { Setlist } from '@/types/library'
import { useLibraryStore } from './library'

export const useSetlistStore = defineStore('setlist', () => {
  const currentSetlist = ref<Setlist | null>(null)
  const allSetlists = ref<Setlist[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const currentSetlistSongs = computed(() => {
    if (!currentSetlist.value) return []
    const libraryStore = useLibraryStore()
    return currentSetlist.value.song_ids
      .map(id => libraryStore.songs.find(song => song.id === id))
      .filter(song => song !== undefined)
  })

  const recentSetlists = computed(() => {
    return [...allSetlists.value]
      .sort((a, b) => b.updated_at - a.updated_at)
      .slice(0, 5)
  })

  // Actions
  async function fetchSetlists() {
    loading.value = true
    error.value = null
    try {
      const setlists = await invoke<Setlist[]>('get_all_setlists')
      allSetlists.value = setlists
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch setlists'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function createSetlist(name: string) {
    loading.value = true
    error.value = null
    try {
      const id = await invoke<string>('create_setlist', { name })
      const newSetlist: Setlist = {
        id,
        name,
        created_at: Date.now(),
        updated_at: Date.now(),
        song_ids: [],
      }
      allSetlists.value.push(newSetlist)
      currentSetlist.value = newSetlist
      return id
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create setlist'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function loadSetlist(id: string) {
    loading.value = true
    error.value = null
    try {
      const setlist = await invoke<Setlist>('get_setlist', { setlistId: id })
      currentSetlist.value = setlist
      // Update in allSetlists if it exists
      const index = allSetlists.value.findIndex(s => s.id === id)
      if (index !== -1) {
        allSetlists.value[index] = setlist
      } else {
        allSetlists.value.push(setlist)
      }

      // Done with UI update
      loading.value = false

      // Preload all songs in the setlist for instant playback (non-blocking)
      if (setlist.song_ids.length > 0) {
        invoke('preload_setlist', { setlistId: id }).catch(e => {
          console.error('Failed to preload setlist:', e)
        })
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load setlist'
      loading.value = false
      throw e
    }
  }

  async function updateSetlist(setlist: Setlist) {
    loading.value = true
    error.value = null
    try {
      await invoke('update_setlist', {
        setlistId: setlist.id,
        name: setlist.name,
        songIds: setlist.song_ids
      })
      // Update local state
      const index = allSetlists.value.findIndex(s => s.id === setlist.id)
      if (index !== -1) {
        allSetlists.value[index] = { ...setlist, updated_at: Date.now() }
      }
      if (currentSetlist.value?.id === setlist.id) {
        currentSetlist.value = { ...setlist, updated_at: Date.now() }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update setlist'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteSetlist(id: string) {
    loading.value = true
    error.value = null
    try {
      await invoke('delete_setlist', { setlistId: id })
      allSetlists.value = allSetlists.value.filter(s => s.id !== id)
      if (currentSetlist.value?.id === id) {
        currentSetlist.value = null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete setlist'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function addSongToSetlist(songId: string) {
    if (!currentSetlist.value) {
      throw new Error('No setlist loaded')
    }

    loading.value = true
    error.value = null
    try {
      await invoke('add_song_to_setlist', {
        setlistId: currentSetlist.value.id,
        songId: songId,
      })
      // Update local state
      currentSetlist.value.song_ids.push(songId)
      currentSetlist.value.updated_at = Date.now()

      // Preload the newly added song for instant playback
      // This happens in the background without blocking the UI
      invoke('load_song', { songId: songId }).catch(e => {
        console.warn('Failed to preload song:', e)
      })

      // Trigger auto-save
      debouncedSave(currentSetlist.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add song to setlist'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function removeSongFromSetlist(songId: string) {
    if (!currentSetlist.value) {
      throw new Error('No setlist loaded')
    }

    loading.value = true
    error.value = null
    try {
      await invoke('remove_song_from_setlist', {
        setlistId: currentSetlist.value.id,
        songId: songId,
      })
      // Update local state
      currentSetlist.value.song_ids = currentSetlist.value.song_ids.filter(
        id => id !== songId
      )
      currentSetlist.value.updated_at = Date.now()
      // Trigger auto-save
      debouncedSave(currentSetlist.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove song from setlist'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function reorderSongs(oldIndex: number, newIndex: number) {
    if (!currentSetlist.value) {
      throw new Error('No setlist loaded')
    }

    const songIds = [...currentSetlist.value.song_ids]
    const [movedSong] = songIds.splice(oldIndex, 1)
    songIds.splice(newIndex, 0, movedSong)

    loading.value = true
    error.value = null
    try {
      await invoke('reorder_setlist_songs', {
        setlistId: currentSetlist.value.id,
        songIds: songIds,
      })
      // Update local state
      currentSetlist.value.song_ids = songIds
      currentSetlist.value.updated_at = Date.now()
      // Trigger auto-save
      debouncedSave(currentSetlist.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to reorder songs'
      throw e
    } finally {
      loading.value = false
    }
  }

  // Auto-save with 500ms debounce
  const debouncedSave = useDebounceFn(async (setlist: Setlist) => {
    try {
      await updateSetlist(setlist)
    } catch (e) {
      console.error('Auto-save failed:', e)
    }
  }, 500)

  return {
    currentSetlist,
    allSetlists,
    loading,
    error,
    currentSetlistSongs,
    recentSetlists,
    fetchSetlists,
    createSetlist,
    loadSetlist,
    updateSetlist,
    deleteSetlist,
    addSongToSetlist,
    removeSongFromSetlist,
    reorderSongs,
    debouncedSave,
  }
})
