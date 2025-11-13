<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Music, Plus } from 'lucide-vue-next'
import SetlistToolbar from './SetlistToolbar.vue'
import SetlistItem from './SetlistItem.vue'
import Button from '@/components/ui/Button.vue'
import { useSetlistStore } from '@/stores/setlist'
import { useModalStore } from '@/stores/modal'
import { usePlaybackStore } from '@/stores/playback'
import type { Song } from '@/types/library'

const setlistStore = useSetlistStore()
const modalStore = useModalStore()
const playbackStore = usePlaybackStore()

const draggedIndex = ref<number | null>(null)

onMounted(() => {
  setlistStore.fetchSetlists()
})

function handleDragStart(index: number) {
  draggedIndex.value = index
}

function handleDragEnd() {
  draggedIndex.value = null
}

function handleDrop(targetIndex: number) {
  if (draggedIndex.value === null) return
  if (draggedIndex.value === targetIndex) return

  setlistStore.reorderSongs(draggedIndex.value, targetIndex)
  draggedIndex.value = null
}

function handleRemoveSong(songId: string) {
  setlistStore.removeSongFromSetlist(songId)
}

async function handleSongSelect(song: Song) {


  try {
    await playbackStore.selectSong(song)
  } catch (error) {
    console.error('Failed to select song:', error)
  }
}

function handleNewSetlist() {
  modalStore.openModal('new-setlist')
}
</script>

<template>
  <div class="flex h-full flex-col min-w-[500px]">
    <!-- Toolbar -->
    <SetlistToolbar v-if="setlistStore.allSetlists.length > 0" />

    <!-- Setlist Content -->
    <div class="flex-1 overflow-y-auto p-4">
      <!-- No Setlist Loaded State -->
      <div
        v-if="!setlistStore.currentSetlist"
        class="flex h-full flex-col items-center justify-center text-center"
      >
        <Music :size="64" class="mb-4 text-muted-foreground" />
        <h3 class="mb-2 text-lg font-semibold text-foreground">No Setlist Loaded</h3>
        <p class="mb-4 text-sm text-muted-foreground">
          Create a new setlist or select an existing one to get started
        </p>
        <Button variant="default" @click="handleNewSetlist">
          <Plus :size="16" class="mr-2" />
          Create Setlist
        </Button>
      </div>

      <!-- Setlist Loaded -->
      <div v-else>
        <!-- Empty Setlist State -->
        <div
          v-if="setlistStore.currentSetlistSongs.length === 0"
          class="rounded-lg border-2 border-dashed border-border p-8 text-center"
        >
          <Music :size="48" class="mx-auto mb-3 text-muted-foreground" />
          <h4 class="mb-1 font-semibold text-foreground">Setlist is empty</h4>
          <p class="mb-4 text-sm text-muted-foreground">
            Add songs to your setlist to get started
          </p>
          <Button variant="default" @click="modalStore.openModal('add-song-to-setlist')">
            <Plus :size="16" class="mr-2" />
            Add Songs
          </Button>
        </div>

        <!-- Song List -->
        <div v-else>
          <!-- Songs -->
          <div class="space-y-2">
            <SetlistItem
              v-for="(song, index) in setlistStore.currentSetlistSongs"
              :key="song.id"
              :song="song"
              :index="index"
              :is-selected="playbackStore.selectedSong?.id === song.id"
              @select="handleSongSelect"
              @remove="handleRemoveSong(song.id)"
              @dragstart="handleDragStart"
              @dragend="handleDragEnd"
              @drop="handleDrop"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
