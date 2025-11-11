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
  <div class="flex max-h-[100px]">
    <!-- Transport Controls -->
    <div class="flex items-stretch gap-2">
      <div class="panel text-5xl">C</div>
      <div class="panel text-2xl text-center flex flex-col justify-center gap-[10px]">
        <h6>74</h6>
        <div class="divider border-t border-white/20 w-full"></div>
        <h6>6/8</h6>
      </div>
      <div class="panel text-2xl text-center">
        <span class="text-white">{{ playbackStore.formattedPosition }} / {{ playbackStore.formattedDuration }}</span>
      </div>
      <button
        class="panel text-6xl h-full"
        :disabled="isDisabled"
        @click="handlePrevious"
        aria-label="Previous song"
      >
        <svg class="size-[50px]" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M17.6901 20.0902C17.1201 20.0902 16.5601 19.9402 16.0401 19.6402L7.75009 14.8602C6.72009 14.2602 6.1001 13.1902 6.1001 12.0002C6.1001 10.8102 6.72009 9.74019 7.75009 9.14019L16.0401 4.36016C17.0701 3.76016 18.3001 3.76016 19.3401 4.36016C20.3801 4.96016 20.9901 6.02017 20.9901 7.22017V16.7902C20.9901 17.9802 20.3701 19.0502 19.3401 19.6502C18.8201 19.9402 18.2601 20.0902 17.6901 20.0902ZM17.6901 5.41017C17.3801 5.41017 17.0701 5.49016 16.7901 5.65016L8.50009 10.4302C7.94009 10.7602 7.6001 11.3402 7.6001 11.9902C7.6001 12.6402 7.94009 13.2202 8.50009 13.5502L16.7901 18.3302C17.3501 18.6602 18.0301 18.6602 18.5901 18.3302C19.1501 18.0002 19.4901 17.4202 19.4901 16.7702V7.20018C19.4901 6.55018 19.1501 5.97019 18.5901 5.64019C18.3101 5.50019 18.0001 5.41017 17.6901 5.41017Z" fill="currentColor"/>
          <path d="M3.75977 18.9298C3.34977 18.9298 3.00977 18.5898 3.00977 18.1798V5.81982C3.00977 5.40982 3.34977 5.06982 3.75977 5.06982C4.16977 5.06982 4.50977 5.40982 4.50977 5.81982V18.1798C4.50977 18.5898 4.16977 18.9298 3.75977 18.9298Z" fill="currentColor"/>
        </svg>

      </button>

      <button
          class="panel"
        :disabled="isDisabled"
        @click="togglePlayPause"
        aria-label="Play/Pause"
      >
        <svg v-if="!playbackStore.isPlaying" class="size-[50px]" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M7.87 21.28C7.08 21.28 6.33 21.09 5.67 20.71C4.11 19.81 3.25 17.98 3.25 15.57V8.43999C3.25 6.01999 4.11 4.19999 5.67 3.29999C7.23 2.39999 9.24 2.56999 11.34 3.77999L17.51 7.33999C19.6 8.54999 20.76 10.21 20.76 12.01C20.76 13.81 19.61 15.47 17.51 16.68L11.34 20.24C10.13 20.93 8.95 21.28 7.87 21.28ZM7.87 4.21999C7.33 4.21999 6.85 4.33999 6.42 4.58999C5.34 5.20999 4.75 6.57999 4.75 8.43999V15.56C4.75 17.42 5.34 18.78 6.42 19.41C7.5 20.04 8.98 19.86 10.59 18.93L16.76 15.37C18.37 14.44 19.26 13.25 19.26 12C19.26 10.75 18.37 9.55999 16.76 8.62999L10.59 5.06999C9.61 4.50999 8.69 4.21999 7.87 4.21999Z" fill="currentColor"/>
        </svg>
        <svg v-else class="size-[50px]" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M8.64 21.75H5.01C3.15 21.75 2.25 20.89 2.25 19.11V4.89C2.25 3.11 3.15 2.25 5.01 2.25H8.64C10.5 2.25 11.4 3.11 11.4 4.89V19.11C11.4 20.89 10.5 21.75 8.64 21.75ZM5.01 3.75C3.93 3.75 3.75 4.02 3.75 4.89V19.11C3.75 19.98 3.92 20.25 5.01 20.25H8.64C9.72 20.25 9.9 19.98 9.9 19.11V4.89C9.9 4.02 9.73 3.75 8.64 3.75H5.01Z" fill="currentColor"/>
          <path d="M18.9901 21.75H15.3601C13.5001 21.75 12.6001 20.89 12.6001 19.11V4.89C12.6001 3.11 13.5001 2.25 15.3601 2.25H18.9901C20.8501 2.25 21.7501 3.11 21.7501 4.89V19.11C21.7501 20.89 20.8501 21.75 18.9901 21.75ZM15.3601 3.75C14.2801 3.75 14.1001 4.02 14.1001 4.89V19.11C14.1001 19.98 14.2701 20.25 15.3601 20.25H18.9901C20.0701 20.25 20.2501 19.98 20.2501 19.11V4.89C20.2501 4.02 20.0801 3.75 18.9901 3.75H15.3601Z" fill="currentColor"/>
        </svg>
      </button>

      <button
          class="panel"
        :disabled="isDisabled"
        @click="handleStop"
        aria-label="Stop"
      >
        <svg class="size-[50px]" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M14.7 21.75H9.3C4.36 21.75 2.25 19.64 2.25 14.7V9.3C2.25 4.36 4.36 2.25 9.3 2.25H14.7C19.64 2.25 21.75 4.36 21.75 9.3V14.7C21.75 19.64 19.64 21.75 14.7 21.75ZM9.3 3.75C5.2 3.75 3.75 5.2 3.75 9.3V14.7C3.75 18.8 5.2 20.25 9.3 20.25H14.7C18.8 20.25 20.25 18.8 20.25 14.7V9.3C20.25 5.2 18.8 3.75 14.7 3.75H9.3Z" fill="currentColor"/>
        </svg>

      </button>

      <button
        class="panel"
        :disabled="isDisabled"
        @click="handleNext"
        aria-label="Next song"
      >
        <svg class="size-[50px]" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M6.30975 20.0899C5.73975 20.0899 5.17976 19.9399 4.65976 19.6399C3.61976 19.0399 3.00977 17.9799 3.00977 16.7799V7.20989C3.00977 6.01989 3.62976 4.9499 4.65976 4.3499C5.69976 3.7499 6.92975 3.7499 7.95975 4.3499L16.2498 9.1299C17.2798 9.7299 17.8997 10.7999 17.8997 11.9899C17.8997 13.1799 17.2798 14.2499 16.2498 14.8499L7.95975 19.6299C7.43975 19.9399 6.87975 20.0899 6.30975 20.0899ZM6.30975 5.4099C5.99975 5.4099 5.68976 5.48989 5.40976 5.64989C4.84976 5.97989 4.50977 6.55989 4.50977 7.20989V16.7799C4.50977 17.4299 4.84976 18.0099 5.40976 18.3399C5.96976 18.6599 6.64975 18.6699 7.20975 18.3399L15.4998 13.5599C16.0598 13.2299 16.3997 12.6499 16.3997 11.9999C16.3997 11.3499 16.0598 10.7699 15.4998 10.4399L7.20975 5.6599C6.92975 5.4999 6.61975 5.4099 6.30975 5.4099Z" fill="currentColor"/>
          <path d="M20.2402 18.9303C19.8302 18.9303 19.4902 18.5903 19.4902 18.1803V5.82031C19.4902 5.41031 19.8302 5.07031 20.2402 5.07031C20.6502 5.07031 20.9902 5.41031 20.9902 5.82031V18.1803C20.9902 18.5903 20.6602 18.9303 20.2402 18.9303Z" fill="currentColor"/>
        </svg>
      </button>
    </div>

    <!-- Key and Tempo Display -->
    <div
      v-if="playbackStore.selectedSong"
      class="ml-auto flex items-center gap-4 text-sm"
    >
      <div v-if="playbackStore.selectedSong.key" class="flex items-center gap-1">
        <span class="text-muted-foreground">Key:</span>
        <span class="font-medium">{{ playbackStore.selectedSong.key }}</span>
      </div>
      <div v-if="playbackStore.selectedSong.tempo" class="flex items-center gap-1">
        <span class="text-muted-foreground">Tempo:</span>
        <span class="font-medium">{{ playbackStore.selectedSong.tempo }}</span>
      </div>
      <div
        v-if="playbackStore.selectedSong.time_signature"
        class="flex items-center gap-1"
      >
        <span class="text-muted-foreground">Time:</span>
        <span class="font-medium">{{ playbackStore.selectedSong.time_signature }}</span>
      </div>
    </div>
  </div>
</template>
