import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { Song, Stem } from '@/types/library'

export const usePlaybackStore = defineStore('playback', () => {
  // State
  const selectedSong = ref<Song | null>(null)
  const currentSong = ref<Song | null>(null)
  const isPlaying = ref(false)
  const currentPosition = ref(0)
  const duration = ref(0)
  const stems = ref<Stem[]>([])
  const volume = ref(0.8)
  const isLoadingStems = ref(false)

  // Client-side position interpolation
  let animationFrameId: number | null = null
  let lastUpdateTime = 0
  let backendPosition = 0

  // Getters
  const formattedPosition = computed(() => {
    return formatTime(currentPosition.value)
  })

  const formattedDuration = computed(() => {
    return formatTime(duration.value)
  })

  const progress = computed(() => {
    if (duration.value === 0) return 0
    return (currentPosition.value / duration.value) * 100
  })

  // Helper function to format time as MM:SS
  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  // Actions
  async function selectSong(song: Song) {
    console.log('selectedSong.value: ', selectedSong.value)
    if (selectedSong.value && currentSong.value?.id === selectedSong.value?.id) return;
    // If a different song is currently playing or loaded, stop it first
    if (currentSong.value && currentSong.value.id !== song.id) {
      if (isPlaying.value) {
        await stop()
      } else {
        // Song was paused - stop the backend playback to reset its position
        try {
          await invoke('stop_playback')
        } catch (e) {
          console.error('Failed to stop backend playback:', e)
        }
        // Clear the current song so play button loads the new one
        setCurrentSong(null)
        stopPositionAnimation()
      }
    }

    selectedSong.value = song

    // Always reset position when selecting a new song
    currentPosition.value = 0

    // Fetch stems for the selected song
    isLoadingStems.value = true
    try {
      const songStems = await invoke<Stem[]>('get_song_stems', { songId: song.id })
      setStems(songStems)
    } catch (e) {
      console.error('Failed to fetch stems for selected song:', e)
    } finally {
      isLoadingStems.value = false
    }
  }

  async function playSong(songId: string) {
    isLoadingStems.value = true
    try {
      // Use selectedSong if it matches, otherwise fetch from database
      let song = selectedSong.value
      if (!song || song.id !== songId) {
        song = await invoke<Song>('get_song', { songId })
      }
      setCurrentSong(song)

      // Use existing stems if available, otherwise fetch
      if (stems.value.length === 0 || stems.value[0]?.song_id !== songId) {
        const songStems = await invoke<Stem[]>('get_song_stems', { songId })
        setStems(songStems)
      }

      // Start playback - backend uses cached decoded audio
      console.log('🎵 Starting playback for:', song.name)
      await invoke('play_song', { songId })
      isPlaying.value = true
      startPositionAnimation()
      console.log('✅ Playback started successfully')
    } catch (e) {
      console.error('Failed to play song:', e)
      throw e
    } finally {
      isLoadingStems.value = false
    }
  }

  async function resume() {
    try {
      await invoke('resume_playback')
      isPlaying.value = true
      startPositionAnimation()
    } catch (e) {
      console.error('Failed to resume playback:', e)
      throw e
    }
  }

  async function pause() {
    try {
      await invoke('pause_playback')
      isPlaying.value = false
      stopPositionAnimation()
    } catch (e) {
      console.error('Failed to pause playback:', e)
      throw e
    }
  }

  async function stop() {
    try {
      await invoke('stop_playback')
      isPlaying.value = false
      stopPositionAnimation()
      currentPosition.value = 0
      // Clear current song so Play button will play selected song instead of resuming
      setCurrentSong(null)
    } catch (e) {
      console.error('Failed to stop playback:', e)
      throw e
    }
  }

  async function seek(position: number) {
    try {
      await invoke('seek_to_position', { position })
      currentPosition.value = position
    } catch (e) {
      console.error('Failed to seek:', e)
      throw e
    }
  }

  function setVolume(newVolume: number) {
    volume.value = Math.max(0, Math.min(1, newVolume))
  }

  async function setStemVolume(stemId: string, stemVolume: number) {
    try {
      const clampedVolume = Math.max(0, Math.min(1, stemVolume))
      await invoke('set_stem_volume', { stemId, volume: clampedVolume })

      // Update local state
      const stem = stems.value.find(s => s.id === stemId)
      if (stem) {
        stem.volume = clampedVolume
      }
    } catch (e) {
      console.error('Failed to set stem volume:', e)
      throw e
    }
  }

  async function toggleStemMute(stemId: string) {
    try {
      await invoke('toggle_stem_mute', { stemId })

      // Update local state
      const stem = stems.value.find(s => s.id === stemId)
      if (stem) {
        stem.is_muted = !stem.is_muted
      }
    } catch (e) {
      console.error('Failed to toggle stem mute:', e)
      throw e
    }
  }

  async function toggleStemSolo(stemId: string) {
    try {
      await invoke('toggle_stem_solo', { stemId })

      // Note: Solo state is not stored in Stem interface
      // Backend will handle solo logic (muting all other stems)
    } catch (e) {
      console.error('Failed to toggle stem solo:', e)
      throw e
    }
  }

  // Animation loop for smooth position updates
  function startPositionAnimation() {
    if (animationFrameId !== null) {
      console.log('Animation already running')
      return // Already running
    }

    console.log('Starting position animation')

    const animate = () => {
      if (isPlaying.value) {
        const now = performance.now()
        const deltaTime = (now - lastUpdateTime) / 1000 // Convert to seconds

        if (lastUpdateTime > 0) {
          // Increment position based on time elapsed
          currentPosition.value += deltaTime

          // Clamp to duration
          if (currentPosition.value > duration.value) {
            currentPosition.value = duration.value
          }
        }

        lastUpdateTime = now
      }

      animationFrameId = requestAnimationFrame(animate)
    }

    lastUpdateTime = performance.now()
    animationFrameId = requestAnimationFrame(animate)
  }

  function stopPositionAnimation() {
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId)
      animationFrameId = null
    }
    lastUpdateTime = 0
  }

  function updatePosition(position: number) {
    // Backend position update - sync our local position
    backendPosition = position
    currentPosition.value = position
    lastUpdateTime = performance.now()
  }

  function updatePlaybackState(playing: boolean) {
    const wasPlaying = isPlaying.value
    isPlaying.value = playing

    if (playing && !wasPlaying) {
      // Started playing
      startPositionAnimation()
    } else if (!playing && wasPlaying) {
      // Stopped playing
      stopPositionAnimation()
    }
  }

  function setCurrentSong(song: Song | null) {
    currentSong.value = song
    if (song) {
      duration.value = song.duration
    } else {
      duration.value = 0
      currentPosition.value = 0
      stems.value = []
    }
  }

  function setStems(newStems: Stem[]) {
    stems.value = newStems
  }

  // Initialize event listeners
  function initializeEventListeners() {
    // Listen for playback position updates
    listen('playback:position', (event: any) => {
      updatePosition(event.payload.position)
    })

    // Listen for playback state updates
    listen('playback:state', (event: any) => {
      updatePlaybackState(event.payload.is_playing)
    })
  }

  return {
    // State
    selectedSong,
    currentSong,
    isPlaying,
    currentPosition,
    duration,
    stems,
    volume,
    isLoadingStems,

    // Getters
    formattedPosition,
    formattedDuration,
    progress,

    // Actions
    selectSong,
    playSong,
    resume,
    pause,
    stop,
    seek,
    setVolume,
    setStemVolume,
    toggleStemMute,
    toggleStemSolo,
    updatePosition,
    updatePlaybackState,
    setCurrentSong,
    setStems,
    initializeEventListeners,
  }
})
