<script setup lang="ts">
import { ref, computed } from 'vue'
import { Loader2 } from 'lucide-vue-next'
import Modal from '@/components/ui/Modal.vue'
import { listen } from '@tauri-apps/api/event'

const isOpen = ref(false)
const currentSong = ref('')
const currentIndex = ref(0)
const totalSongs = ref(0)

const progress = computed(() => {
  if (totalSongs.value === 0) return 0
  return Math.round((currentIndex.value / totalSongs.value) * 100)
})

// Listen for preload events
listen('preload:progress', (event: any) => {
  isOpen.value = true
  currentSong.value = event.payload.song_name
  currentIndex.value = event.payload.current
  totalSongs.value = event.payload.total
})

listen('preload:complete', () => {
  // Keep modal open for a brief moment to show completion
  setTimeout(() => {
    isOpen.value = false
    currentSong.value = ''
    currentIndex.value = 0
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
          Preloading Setlist
        </h2>

        <!-- Current Song -->
        <p class="mb-4 text-sm text-muted-foreground">
          {{ currentSong }}
        </p>

        <!-- Progress Bar -->
        <div class="w-full">
          <div class="mb-2 flex items-center justify-between text-sm text-muted-foreground">
            <span>Song {{ currentIndex }} of {{ totalSongs }}</span>
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
