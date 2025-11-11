<script setup lang="ts">
import { usePlaybackStore } from '@/stores/playback'
import StemRow from './StemRow.vue'
import MasterChannel from './MasterChannel.vue'

const playbackStore = usePlaybackStore()
</script>

<template>
  <div class="h-full rounded-lg panel overflow-y-auto relative pr-[100px]">
    <!-- Master Channel (always visible when there are stems) -->
    <MasterChannel v-if="playbackStore.stems.length > 0" />
    <div class="flex flex-wrap gap-4 rounded-lg h-full relative overflow-x-auto w-full">

      <!-- Stem List -->
      <template
          v-if="playbackStore.stems.length > 0"
          class="flex-1 "
      >
        <StemRow
            v-for="stem in playbackStore.stems"
            :key="stem.id"
            :stem="stem"
        />
      </template>

      <!-- Empty State -->
      <div
          v-else
          class="flex flex-1 items-center justify-center text-center text-muted-foreground"
      >
        <p>No song loaded</p>
      </div>
    </div>
  </div>
</template>
