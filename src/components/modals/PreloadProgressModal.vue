<script setup lang="ts">
import { ref, computed } from 'vue'
import { Loader2 } from 'lucide-vue-next'
import Modal from '@/components/ui/Modal.vue'
import { listen } from '@tauri-apps/api/event'

const isOpen = ref(false)
const currentSong = ref('')
const currentStem = ref('')
const currentStemIndex = ref(0)
const totalStems = ref(0)
const currentSongIndex = ref(0)
const totalSongs = ref(0)

const stemProgress = computed(() => {
  if (totalStems.value > 0) {
    return Math.round((currentStemIndex.value / totalStems.value) * 100)
  }
  return 0
})

const displayText = computed(() => {
  if (totalStems.value > 0) {
    return `Stem ${currentStemIndex.value} of ${totalStems.value}`
  }
  return ''
})

// Listen for stem loading events (when loading a single song)
listen('stem:loading', (event: any) => {
  isOpen.value = true
  currentSong.value = event.payload.song_name
  currentStem.value = event.payload.stem_name
  currentStemIndex.value = event.payload.current
  totalStems.value = event.payload.total
})

listen('stem:complete', () => {
  // Only close modal if we're loading a single song (not a setlist)
  // When preloading a setlist, we wait for 'preload:complete' instead
  if (totalSongs.value === 0) {
    // Keep modal open for a brief moment to show completion
    setTimeout(() => {
      isOpen.value = false
      currentSong.value = ''
      currentStem.value = ''
      currentStemIndex.value = 0
      totalStems.value = 0
    }, 500)
  }
})

// Listen for setlist preload events
listen('preload:progress', (event: any) => {
  isOpen.value = true
  currentSong.value = event.payload.song_name
  currentSongIndex.value = event.payload.current
  totalSongs.value = event.payload.total
  // Reset stem progress for new song
  currentStem.value = ''
  currentStemIndex.value = 0
  totalStems.value = 0
})

listen('preload:complete', () => {
  // Keep modal open for a brief moment to show completion
  setTimeout(() => {
    isOpen.value = false
    currentSong.value = ''
    currentStem.value = ''
    currentStemIndex.value = 0
    totalStems.value = 0
    currentSongIndex.value = 0
    totalSongs.value = 0
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

        <!-- Setlist Loading: Show current song progress -->
        <div v-if="totalSongs > 0" class="mb-4 w-full">
          <p class="mb-2 text-sm font-medium text-foreground">
            Song {{ currentSongIndex }} of {{ totalSongs }}
          </p>
          <p class="mb-4 text-xs text-muted-foreground">
            {{ currentSong }}
          </p>
          <p class="text-xs text-muted-foreground">
            Loading stems...
          </p>
        </div>

        <!-- Single Song Loading: Show stem progress -->
        <div v-else class="mb-4 w-full">
          <p class="mb-1 text-sm font-medium text-foreground">
            {{ currentSong }}
          </p>

          <!-- Current Stem -->
          <p v-if="currentStem" class="mb-4 text-xs text-muted-foreground">
            Loading: {{ currentStem }}
          </p>
          <p v-else class="mb-4 text-xs text-muted-foreground">
            &nbsp;
          </p>

          <!-- Progress Bar (only for single song) -->
          <div class="w-full">
            <div class="mb-2 flex items-center justify-between text-sm text-muted-foreground">
              <span>{{ displayText }}</span>
              <span>{{ stemProgress }}%</span>
            </div>
            <div class="h-2 w-full overflow-hidden rounded-full bg-muted">
              <div
                class="h-full bg-primary transition-all duration-300"
                :style="{ width: `${stemProgress}%` }"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </Modal>
</template>
