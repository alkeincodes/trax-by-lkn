import { onMounted, onUnmounted } from 'vue'
import { usePlaybackStore } from '@/stores/playback'

export function useKeyboardShortcuts() {
  const playbackStore = usePlaybackStore()

  const handleKeyDown = (event: KeyboardEvent) => {
    // Ignore if user is typing in an input or textarea
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return
    }

    switch (event.key) {
      case ' ':
        // Space: Toggle play/pause
        event.preventDefault()
        if (playbackStore.isPlaying) {
          playbackStore.pause()
        } else if (playbackStore.currentSong) {
          // Resume paused song
          playbackStore.resume()
        } else if (playbackStore.selectedSong) {
          // Play selected song
          playbackStore.playSong(playbackStore.selectedSong.id)
        }
        break

      case 'ArrowLeft':
        // Left Arrow: Seek backward 5 seconds
        event.preventDefault()
        if (playbackStore.currentSong) {
          const newPosition = Math.max(0, playbackStore.currentPosition - 5)
          playbackStore.seek(newPosition)
        }
        break

      case 'ArrowRight':
        // Right Arrow: Seek forward 5 seconds
        event.preventDefault()
        if (playbackStore.currentSong) {
          const newPosition = Math.min(
            playbackStore.duration,
            playbackStore.currentPosition + 5
          )
          playbackStore.seek(newPosition)
        }
        break

      case 'm':
      case 'M':
        // M: Mute all stems
        event.preventDefault()
        if (playbackStore.stems.length > 0) {
          playbackStore.stems.forEach(stem => {
            if (!stem.is_muted) {
              playbackStore.toggleStemMute(stem.id)
            }
          })
        }
        break

      case 's':
      case 'S':
        // S: Solo first stem (or toggle solo mode)
        event.preventDefault()
        if (playbackStore.stems.length > 0) {
          playbackStore.toggleStemSolo(playbackStore.stems[0].id)
        }
        break
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown)
  })

  return {
    handleKeyDown,
  }
}
