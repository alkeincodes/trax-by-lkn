<script setup lang="ts">
import { usePlaybackStore } from '@/stores/playback'
import StemRow from './StemRow.vue'

const playbackStore = usePlaybackStore()
</script>

<template>
  <div class="flex h-full flex-col rounded-lg border border-border bg-card p-4">
    <!-- Header -->
    <div class="mb-4 border-b border-border pb-2">
      <h2 class="text-lg font-semibold">Stem Mixer</h2>
    </div>

    <!-- Stem List -->
    <div
      v-if="playbackStore.stems.length > 0"
      class="flex-1 space-y-2 overflow-y-auto"
    >
      <StemRow
        v-for="stem in playbackStore.stems"
        :key="stem.id"
        :stem="stem"
      />
    </div>

    <!-- Empty State -->
    <div
      v-else
      class="flex flex-1 items-center justify-center text-center text-muted-foreground"
    >
      <p>No song loaded</p>
    </div>
  </div>
</template>
