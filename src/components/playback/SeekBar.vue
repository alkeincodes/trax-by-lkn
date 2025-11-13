<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted, computed } from 'vue'
import { usePlaybackStore } from '@/stores/playback'
import WaveSurfer from 'wavesurfer.js'
import { convertFileSrc } from '@tauri-apps/api/core'

const playbackStore = usePlaybackStore()

const waveformContainer = ref<HTMLDivElement | null>(null)
let wavesurfer: WaveSurfer | null = null
let isUserSeeking = ref(false)
const isLoading = ref(false)

const hasSong = computed(() => {
  return playbackStore.selectedSong !== null || playbackStore.currentSong !== null
})

onMounted(() => {
  if (!waveformContainer.value) return

  // Initialize WaveSurfer
  wavesurfer = WaveSurfer.create({
    container: waveformContainer.value,
    waveColor: '#4b5563',
    progressColor: '#3b82f6',
    cursorColor: '#3b82f6',
    height: 130,
    barWidth: 2,
    barGap: 1,
    barRadius: 2,
    normalize: true,
    interact: true,
    hideScrollbar: true,
    // Prevent WaveSurfer from playing audio - we only use it for visualization
    // The actual audio playback is handled by the Rust audio engine with stems
    backend: 'WebAudio',
    // Don't use media element to avoid any audio playback
    media: undefined,
  })

  // Handle seeking - when user interacts with waveform
  wavesurfer.on('interaction', () => {
    isUserSeeking.value = true
    const currentTime = wavesurfer?.getCurrentTime() || 0

    // Seek to the new position
    playbackStore.seek(currentTime)

    // Reset seeking flag after a short delay
    setTimeout(() => {
      isUserSeeking.value = false
    }, 100)
  })

  // Update position display during seeking
  wavesurfer.on('seeking', (currentTime) => {
    playbackStore.updatePosition(currentTime)
  })

  // Load waveform when a song is selected or playing
  watch(
    () => playbackStore.selectedSong || playbackStore.currentSong,
    async (song) => {
      if (song && wavesurfer) {
        try {
          isLoading.value = true

          // Use mixdown_path if available, otherwise fall back to first stem
          let audioPath = song.mixdown_path

          if (!audioPath) {
            // Fallback: use the first stem if no mixdown is available
            const stems = playbackStore.stems
            if (stems.length > 0) {
              audioPath = stems[0].file_path
              console.warn('No mixdown available for song, using first stem for waveform')
            }
          }

          if (audioPath) {
            // Normalize path for Windows (replace backslashes with forward slashes)
            const normalizedPath = audioPath.replace(/\\/g, '/')

            // Convert the file path to a URL that Tauri can serve
            const audioUrl = convertFileSrc(normalizedPath)
            console.log('Loading waveform from:', audioUrl)
            console.log('Original path:', audioPath)
            console.log('Normalized path:', normalizedPath)

            // Fetch the audio file and create a blob URL
            try {
              const response = await fetch(audioUrl)
              if (!response.ok) {
                throw new Error(`Failed to fetch audio: ${response.status} ${response.statusText}`)
              }
              const blob = await response.blob()
              console.log('Blob created:', blob.size, 'bytes, type:', blob.type)
              const blobUrl = URL.createObjectURL(blob)
              console.log('Blob URL created:', blobUrl)

              // Set a timeout to detect if loading gets stuck
              const loadTimeout = setTimeout(() => {
                console.error('Waveform loading timeout after 10 seconds')
                isLoading.value = false
                URL.revokeObjectURL(blobUrl)
              }, 10000)

              // Set duration once waveform is loaded
              wavesurfer.once('ready', () => {
                clearTimeout(loadTimeout)
                isLoading.value = false
                console.log('Waveform loaded successfully')
                const waveDuration = wavesurfer?.getDuration() || 0
                // Update the selected song's duration if not set
                if (waveDuration > 0 && playbackStore.selectedSong && playbackStore.selectedSong.duration === 0) {
                  playbackStore.selectedSong.duration = waveDuration
                }
                // Reset waveform position to match store position (should be 0 for new songs)
                if (wavesurfer) {
                  const storePosition = playbackStore.currentPosition
                  const seekPosition = storePosition / waveDuration
                  wavesurfer.seekTo(seekPosition)
                }
                // Clean up blob URL after loading
                URL.revokeObjectURL(blobUrl)
              })

              // Handle loading errors
              wavesurfer.once('error', (error) => {
                clearTimeout(loadTimeout)
                console.error('WaveSurfer error:', error)
                isLoading.value = false
                URL.revokeObjectURL(blobUrl)
              })

              // Load the waveform from blob URL
              console.log('Starting wavesurfer.load()...')
              await wavesurfer.load(blobUrl)
              console.log('wavesurfer.load() completed')
            } catch (fetchError) {
              console.error('Failed to fetch audio file:', fetchError)
              isLoading.value = false
            }
          } else {
            console.error('No audio file available for waveform')
            isLoading.value = false
          }
        } catch (error) {
          console.error('Failed to load waveform:', error)
          isLoading.value = false
        }
      } else if (!song && wavesurfer) {
        // Clear waveform when no song is selected
        wavesurfer.empty()
        isLoading.value = false
      }
    },
    { immediate: true }
  )

  // Update waveform position when store position changes (smooth client-side animation)
  // Note: We don't sync play/pause state because WaveSurfer is only used for visualization
  // The actual audio playback is handled by the Rust audio engine
  watch(
    () => playbackStore.currentPosition,
    (position) => {
      if (!wavesurfer || isUserSeeking.value) return

      const duration = wavesurfer.getDuration()
      if (duration > 0) {
        const seekPosition = position / duration
        // Update on every frame for smooth animation
        wavesurfer.seekTo(seekPosition)
      }
    }
  )
})

onUnmounted(() => {
  if (wavesurfer) {
    wavesurfer.destroy()
  }
})
</script>

<template>
  <div class="relative w-full">
    <div
      v-if="!hasSong"
      class="flex min-h-[130px] w-full items-center justify-center rounded-lg bg-secondary/50 text-sm text-muted-foreground"
    >
      Select a song to see waveform
    </div>
    <div v-else-if="isLoading" class="flex min-h-[130px] w-full items-center justify-center">
      <span class="text-sm text-muted-foreground">Loading waveform...</span>
    </div>
    <div v-show="hasSong && !isLoading" ref="waveformContainer" class="w-full cursor-pointer" />
  </div>
</template>
