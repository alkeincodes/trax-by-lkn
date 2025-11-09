<script setup lang="ts">
import { ref, computed } from 'vue'
import { Loader2 } from 'lucide-vue-next'
import Modal from '@/components/ui/Modal.vue'
import { listen } from '@tauri-apps/api/event'

const isOpen = ref(false)
const currentSong = ref('')
const currentStem = ref('')
const currentIndex = ref(0)
const totalSongs = ref(0)
const totalStems = ref(0)

const progress = computed(() => {
  // For setlist preloading, use song progress
  if (totalSongs.value > 0) {
    return Math.round((currentIndex.value / totalSongs.value) * 100)
  }
  // For single song loading, use stem progress
  if (totalStems.value > 0) {
    return Math.round((currentIndex.value / totalStems.value) * 100)
  }
  return 0
})

const displayText = computed(() => {
  if (totalSongs.value > 0) {
    return `Song ${currentIndex.value} of ${totalSongs.value}`
  }
  if (totalStems.value > 0) {
    return `Stem ${currentIndex.value} of ${totalStems.value}`
  }
  return ''
})

// Listen for stem loading events (when loading a single song)
listen('stem:loading', (event: any) => {
  isOpen.value = true
  currentSong.value = event.payload.song_name
  currentStem.value = event.payload.stem_name
  currentIndex.value = event.payload.current
  totalStems.value = event.payload.total
  totalSongs.value = 0 // Clear song count
})

listen('stem:complete', () => {
  // Keep modal open for a brief moment to show completion
  setTimeout(() => {
    isOpen.value = false
    currentSong.value = ''
    currentStem.value = ''
    currentIndex.value = 0
    totalStems.value = 0
  }, 500)
})

// Listen for setlist preload events
listen('preload:progress', (event: any) => {
  isOpen.value = true
  currentSong.value = event.payload.song_name
  currentIndex.value = event.payload.current
  totalSongs.value = event.payload.total
  totalStems.value = 0 // Clear stem count
  currentStem.value = ''
})

listen('preload:complete', () => {
  // Keep modal open for a brief moment to show completion
  setTimeout(() => {
    isOpen.value = false
    currentSong.value = ''
    currentStem.value = ''
    currentIndex.value = 0
    totalSongs.value = 0
    totalStems.value = 0
  }, 500)
})
</script>

<template>
  <Modal :open="isOpen">
    <div class="w-full max-w-md rounded-lg bg-card p-6">
      <div class="flex flex-col items-center text-center">
        <!-- Spinner -->
        <div class="mb-4 text-primary">
          <Loader2 :size="48" class="animate-spin" />
        </div>

        <!-- Title -->
        <h2 class="mb-2 text-xl font-semibold text-foreground">
          {{ totalSongs > 0 ? 'Preloading Setlist' : 'Loading Song' }}
        </h2>

        <!-- Current Song -->
        <p class="mb-1 text-sm font-medium text-foreground">
          {{ currentSong }}
        </p>

        <!-- Current Stem (if loading stems) -->
        <p v-if="currentStem" class="mb-4 text-xs text-muted-foreground">
          Loading: {{ currentStem }}
        </p>
        <p v-else class="mb-4 text-xs text-muted-foreground">
          &nbsp;
        </p>

        <!-- Progress Bar -->
        <div class="w-full">
          <div class="mb-2 flex items-center justify-between text-sm text-muted-foreground">
            <span>{{ displayText }}</span>
            <span>{{ progress }}%</span>
          </div>
          <div class="h-2 w-full overflow-hidden rounded-full bg-muted">
            <div
              class="h-full bg-primary transition-all duration-300"
              :style="{ width: `${progress}%` }"
            />
          </div>
        </div>

        <!-- Info -->
        <p class="mt-4 text-xs text-muted-foreground">
          Please wait while we prepare your setlist for instant playback
        </p>
      </div>
    </div>
  </Modal>
</template>
