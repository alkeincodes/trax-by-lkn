<script setup lang="ts">
import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlaybackStore } from '@/stores/playback'
import VolumeMeter from './VolumeMeter.vue'

const playbackStore = usePlaybackStore()

const volumePercentage = computed(() => playbackStore.volume * 100)

// Use the actual master output level from the backend
// This is calculated from the final mixed audio output after all stems are summed
const masterLevel = computed(() => playbackStore.masterLevel)

const handleVolumeChange = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const newVolume = parseFloat(target.value) / 100

  // Update local state
  playbackStore.setVolume(newVolume)

  // Update backend audio engine
  try {
    await invoke('set_master_volume', { volume: newVolume })
  } catch (e) {
    console.error('Failed to set master volume:', e)
  }
}
</script>

<template>
  <div class="absolute z-[500] right-0 bottom-0 flex h-full min-h-[410px] items-center gap-3 bg-inner-panel p-3">
    <!-- Volume Meter -->
    <div class="h-full bg-panel/50">
      <VolumeMeter :level="masterLevel" />
    </div>

    <!-- Master Controls -->
    <div class="h-full text-center flex flex-col justify-center">
      <div class="truncate text-sm font-bold text-primary mb-4">
        MASTER
      </div>

      <!-- Volume Slider -->
      <div class="mixer-fader">
        <input
          type="range"
          min="0"
          max="100"
          :value="volumePercentage"
          @input="handleVolumeChange"
          aria-label="Master Volume"
        />
      </div>

      <!-- Mute/Solo Buttons Placeholder -->
      <div class="flex justify-center gap-1 py-4">
        <button
          class="text-xs py-1 px-2 bg-accent rounded-md"
          aria-label="Mute"
        >
          M
        </button>
        <button
          class="text-xs py-1 px-2 bg-accent rounded-md"
          aria-label="Solo"
        >
          S
        </button>
      </div>

      <!-- Volume Display -->
      <div class="mx-auto text-right text-xs text-primary font-semibold">
        {{ Math.round(volumePercentage) }}%
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Remove scoped override - use global .mixer-fader styles from index.css */
</style>