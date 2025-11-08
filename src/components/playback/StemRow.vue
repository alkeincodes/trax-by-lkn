<script setup lang="ts">
import { computed } from 'vue'
import { usePlaybackStore } from '@/stores/playback'
import type { Stem } from '@/types/library'
import Button from '@/components/ui/Button.vue'

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
</script>

<template>
  <div class="flex items-center gap-3 rounded-lg border border-border bg-card p-3">
    <!-- Stem Name -->
    <div class="flex-1 truncate text-sm font-medium">
      {{ stem.name }}
    </div>

    <!-- Volume Slider -->
    <div class="relative flex h-24 w-8 items-center justify-center">
      <input
        type="range"
        min="0"
        max="100"
        :value="volumePercentage"
        :disabled="isDisabled"
        class="slider-vertical h-full w-2 cursor-pointer appearance-none rounded-full bg-secondary accent-primary disabled:opacity-50"
        orient="vertical"
        @input="handleVolumeChange"
        aria-label="Volume"
      />
    </div>

    <!-- Mute/Solo Buttons -->
    <div class="flex flex-col gap-1">
      <Button
        variant="ghost"
        size="sm"
        :disabled="isDisabled"
        :class="stem.is_muted ? 'bg-destructive text-destructive-foreground' : ''"
        @click="handleMuteToggle"
        aria-label="Mute"
      >
        M
      </Button>
      <Button
        variant="ghost"
        size="sm"
        :disabled="isDisabled"
        @click="handleSoloToggle"
        aria-label="Solo"
      >
        S
      </Button>
    </div>

    <!-- Volume Display -->
    <div class="w-10 text-right text-xs text-muted-foreground">
      {{ Math.round(volumePercentage) }}%
    </div>
  </div>
</template>

<style scoped>
/* Vertical slider styling */
.slider-vertical {
  writing-mode: bt-lr;
  -webkit-appearance: slider-vertical;
}

.slider-vertical::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: hsl(var(--color-primary));
  cursor: pointer;
}

.slider-vertical::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: hsl(var(--color-primary));
  cursor: pointer;
  border: none;
}

.slider-vertical:disabled::-webkit-slider-thumb {
  cursor: not-allowed;
}

.slider-vertical:disabled::-moz-range-thumb {
  cursor: not-allowed;
}
</style>
