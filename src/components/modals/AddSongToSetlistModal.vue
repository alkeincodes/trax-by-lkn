<script setup lang="ts">
import { ref, computed } from 'vue'
import { X, Music, Search } from 'lucide-vue-next'
import Modal from '@/components/ui/Modal.vue'
import Button from '@/components/ui/Button.vue'
import { useModalStore } from '@/stores/modal'
import { useLibraryStore } from '@/stores/library'
import { useSetlistStore } from '@/stores/setlist'

const modalStore = useModalStore()
const libraryStore = useLibraryStore()
const setlistStore = useSetlistStore()

const searchQuery = ref('')

const filteredSongs = computed(() => {
  if (!searchQuery.value) {
    return libraryStore.songs
  }

  const query = searchQuery.value.toLowerCase()
  return libraryStore.songs.filter(
    song =>
      song.name.toLowerCase().includes(query) ||
      song.artist?.toLowerCase().includes(query)
  )
})

const currentSetlistSongIds = computed(() => {
  return new Set(setlistStore.currentSetlist?.song_ids || [])
})

const isSongInSetlist = (songId: string) => {
  return currentSetlistSongIds.value.has(songId)
}

async function handleAddSong(songId: string) {
  try {
    await setlistStore.addSongToSetlist(songId)
  } catch (e) {
    console.error('Failed to add song to setlist:', e)
  }
}

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

function handleClose() {
  searchQuery.value = ''
  modalStore.closeModal()
}

function handleImportSongs() {
  modalStore.openModal('import-songs')
}
</script>

<template>
  <Modal
    :open="modalStore.activeModal === 'add-song-to-setlist'"
    @close="handleClose"
  >
    <div class="w-full max-w-2xl rounded-lg bg-card p-6">
      <!-- Header -->
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-xl font-semibold text-foreground">Add Songs to Setlist</h2>
        <button
          class="rounded-md p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          @click="handleClose"
        >
          <X :size="20" />
        </button>
      </div>

      <!-- Search -->
      <div class="mb-4 relative">
        <div class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">
          <Search :size="16" />
        </div>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search songs..."
          class="w-full rounded-md border border-border bg-background px-3 py-2 pl-10 text-foreground placeholder-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
        />
      </div>

      <!-- Song List -->
      <div class="max-h-96 overflow-y-auto rounded-md border border-border">
        <!-- Empty State -->
        <div
          v-if="filteredSongs.length === 0"
          class="flex flex-col items-center justify-center p-8 text-center"
        >
          <Music :size="48" class="mb-3 text-muted-foreground" />
          <p class="text-sm text-muted-foreground">
            {{ searchQuery ? 'No songs found' : 'No songs in library' }}
          </p>
        </div>

        <!-- Song Items -->
        <div v-else class="divide-y divide-border">
          <div
            v-for="song in filteredSongs"
            :key="song.id"
            class="flex items-center justify-between p-3 transition-colors hover:bg-muted/50"
          >
            <div class="flex-1 min-w-0">
              <div class="font-medium text-foreground truncate">{{ song.name }}</div>
              <div class="text-sm text-muted-foreground truncate">{{ song.artist }}</div>
            </div>
            <div class="flex items-center gap-3 ml-4">
              <span class="text-sm text-muted-foreground whitespace-nowrap">
                {{ formatDuration(song.duration) }}
              </span>
              <Button
                v-if="!isSongInSetlist(song.id)"
                variant="default"
                size="sm"
                @click="handleAddSong(song.id)"
              >
                Add
              </Button>
              <span
                v-else
                class="text-sm text-muted-foreground whitespace-nowrap"
              >
                In setlist
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="mt-4 flex justify-between">
        <Button variant="outline" @click="handleImportSongs">
          Import Songs
        </Button>
        <Button variant="secondary" @click="handleClose">
          Done
        </Button>
      </div>
    </div>
  </Modal>
</template>
