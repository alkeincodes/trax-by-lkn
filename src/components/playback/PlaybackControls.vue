<script setup lang="ts">
import { computed } from 'vue'
import { usePlaybackStore } from '@/stores/playback'
import Button from '@/components/ui/Button.vue'
import { SkipBack, Play, Pause, Square, SkipForward } from 'lucide-vue-next'

const playbackStore = usePlaybackStore()

const isDisabled = computed(() => {
  // Disable if no song is selected
  if (!playbackStore.selectedSong && !playbackStore.currentSong) {
    return true
  }

  // Disable if stems are still loading
  if (playbackStore.isLoadingStems) {
    return true
  }

  // Disable if selected song has no stems loaded
  if (playbackStore.selectedSong && playbackStore.stems.length === 0) {
    return true
  }

  return false
})

const togglePlayPause = async () => {
  try {
    console.log('togglePlayPause called')
    console.log('isPlaying:', playbackStore.isPlaying)
    console.log('currentSong:', playbackStore.currentSong)
    console.log('selectedSong:', playbackStore.selectedSong)
    console.log('stems:', playbackStore.stems)

    if (playbackStore.isPlaying) {
      // Currently playing, so pause it
      console.log('Calling pause()')
      await playbackStore.pause()
    } else if (playbackStore.currentSong && !playbackStore.isPlaying) {
      // Song is loaded in audio engine but paused - resume it
      console.log('Calling resume()')
      await playbackStore.resume()
    } else if (playbackStore.selectedSong) {
      // No current song loaded in audio engine - play the selected song
      console.log('Calling playSong() with ID:', playbackStore.selectedSong.id)
      await playbackStore.playSong(playbackStore.selectedSong.id)
    }
    console.log('togglePlayPause completed')
  } catch (error) {
    console.error('Failed to toggle play/pause:', error)
  }
}

const handleStop = () => {
  playbackStore.stop()
}

const handlePrevious = () => {
  // TODO: Implement previous song in setlist
  console.log('Previous song')
}

const handleNext = () => {
  // TODO: Implement next song in setlist
  console.log('Next song')
}
</script>

<template>
  <div class="flex items-center gap-4 rounded-lg border border-border bg-card p-4">
    <!-- Transport Controls -->
    <div class="flex items-center gap-2">
      <Button
        variant="ghost"
        size="icon"
        :disabled="isDisabled"
        @click="handlePrevious"
        aria-label="Previous song"
      >
        <SkipBack :size="20" />
      </Button>

      <Button
        variant="default"
        size="icon"
        :disabled="isDisabled"
        @click="togglePlayPause"
        aria-label="Play/Pause"
      >
        <Play v-if="!playbackStore.isPlaying" :size="20" />
        <Pause v-else :size="20" />
      </Button>

      <Button
        variant="ghost"
        size="icon"
        :disabled="isDisabled"
        @click="handleStop"
        aria-label="Stop"
      >
        <Square :size="20" />
      </Button>

      <Button
        variant="ghost"
        size="icon"
        :disabled="isDisabled"
        @click="handleNext"
        aria-label="Next song"
      >
        <SkipForward :size="20" />
      </Button>
    </div>

    <!-- Time Display -->
    <div class="flex items-center gap-2 text-sm text-muted-foreground">
      <span>{{ playbackStore.formattedPosition }}</span>
      <span>/</span>
      <span>{{ playbackStore.formattedDuration }}</span>
    </div>

    <!-- Key and Tempo Display -->
    <div
      v-if="playbackStore.currentSong"
      class="ml-auto flex items-center gap-4 text-sm"
    >
      <div v-if="playbackStore.currentSong.key" class="flex items-center gap-1">
        <span class="text-muted-foreground">Key:</span>
        <span class="font-medium">{{ playbackStore.currentSong.key }}</span>
      </div>
      <div v-if="playbackStore.currentSong.tempo" class="flex items-center gap-1">
        <span class="text-muted-foreground">Tempo:</span>
        <span class="font-medium">{{ playbackStore.currentSong.tempo }}</span>
      </div>
      <div
        v-if="playbackStore.currentSong.time_signature"
        class="flex items-center gap-1"
      >
        <span class="text-muted-foreground">Time:</span>
        <span class="font-medium">{{ playbackStore.currentSong.time_signature }}</span>
      </div>
    </div>
  </div>
</template>
