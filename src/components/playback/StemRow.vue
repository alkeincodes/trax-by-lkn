<script setup lang="ts">
import { computed } from 'vue'
import { usePlaybackStore } from '@/stores/playback'
import type { Stem } from '@/types/library'
import VolumeMeter from './VolumeMeter.vue'

interface Props {
  stem: Stem
}

const props = defineProps<Props>()
const playbackStore = usePlaybackStore()

const volumePercentage = computed(() => props.stem.volume * 100)

const handleVolumeChange = (event: Event) => {
  const target = event.target as HTMLInputElement
  const newVolume = parseFloat(target.value) / 100
  playbackStore.setStemVolume(props.stem.id, newVolume)
}

const handleMuteToggle = () => {
  playbackStore.toggleStemMute(props.stem.id)
}

const handleSoloToggle = () => {
  playbackStore.toggleStemSolo(props.stem.id)
}

const isDisabled = computed(() => !playbackStore.isPlaying)

const currentLevel = computed(() => props.stem.level || 0)
</script>

<template>
  <div class="flex min-h-[410px] items-center gap-3 rounded-lg border border-border bg-inner-panel p-3">
    <!-- Volume Meter -->
    <div class="h-full bg-panel/50">
      <VolumeMeter :level="currentLevel" />
    </div>

    <!-- Stem Name -->
    <div class="h-full text-center flex flex-col justify-center">
      <div class="truncate text-sm font-medium mb-4 h-[25px] max-w-[85px]">
        {{ stem.name }}
      </div>

      <!-- Volume Slider -->
      <div class="mixer-fader">
        <input
            type="range"
            min="0"
            max="100"
            :class="{'down-six': playbackStore.stems.length <= 6}"
            :value="volumePercentage"
            @input="handleVolumeChange"
            aria-label="Volume"
        />
      </div>

      <!-- Mute/Solo Buttons -->
      <div class="flex justify-center gap-1 py-4">
        <button
            class="text-xs py-1 px-2 bg-accent rounded-md"
            :disabled="isDisabled"
            :class="stem.is_muted ? 'bg-destructive text-destructive-foreground' : ''"
            @click="handleMuteToggle"
            aria-label="Mute"
        >
          M
        </button>
        <button
            class="text-xs py-1 px-2 bg-accent rounded-md"
            :disabled="isDisabled"
            :class="stem.is_solo ? 'bg-yellow-500 text-destructive-foreground' : ''"
            @click="handleSoloToggle"
            aria-label="Solo"
        >
          S
        </button>
      </div>

      <!-- Volume Display -->
      <div class="mx-auto text-right text-xs text-muted-foreground">
        {{ Math.round(volumePercentage) }}%
      </div>
    </div>
  </div>
</template>
