<script setup lang="ts">
import { onMounted } from 'vue'
import { useLibraryStore } from '@/stores/library'
import { usePlaybackStore } from '@/stores/playback'
import LibraryToolbar from './LibraryToolbar.vue'
import SongCard from './SongCard.vue'
import type { Song } from '@/types/library'

const libraryStore = useLibraryStore()
const playbackStore = usePlaybackStore()

onMounted(() => {
  libraryStore.fetchSongs()
})

async function handleSongSelect(song: Song) {
  try {
    await playbackStore.selectSong(song)
  } catch (error) {
    console.error('Failed to select song:', error)
  }
}
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- Toolbar -->
    <LibraryToolbar />

    <!-- Content Area -->
    <div class="flex-1 overflow-y-auto p-6">
      <!-- Loading State -->
      <div v-if="libraryStore.loading" class="grid gap-4" :class="{
        'grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4': libraryStore.viewMode === 'grid',
        'grid-cols-1': libraryStore.viewMode === 'list',
      }">
        <div
          v-for="i in 8"
          :key="i"
          class="animate-pulse rounded-lg border border-border bg-card p-4"
        >
          <div class="mb-3 h-12 w-12 rounded-md bg-muted"></div>
          <div class="mb-2 h-5 w-3/4 rounded bg-muted"></div>
          <div class="mb-2 h-4 w-1/2 rounded bg-muted"></div>
          <div class="h-4 w-full rounded bg-muted"></div>
        </div>
      </div>

      <!-- Error State -->
      <div
        v-else-if="libraryStore.error"
        class="flex flex-col items-center justify-center py-12 text-center"
      >
        <p class="mb-2 text-lg font-medium text-destructive">
          Error loading library
        </p>
        <p class="text-sm text-muted-foreground">
          {{ libraryStore.error }}
        </p>
      </div>

      <!-- Empty State -->
      <div
        v-else-if="libraryStore.filteredSongs.length === 0 && !libraryStore.hasFilters"
        class="flex flex-col items-center justify-center py-12 text-center"
      >
        <div class="mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-muted">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-10 w-10 text-muted-foreground"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
            />
          </svg>
        </div>
        <h3 class="mb-2 text-xl font-semibold text-foreground">
          No songs yet
        </h3>
        <p class="mb-4 text-sm text-muted-foreground">
          Click Import to add backing tracks to your library
        </p>
      </div>

      <!-- No Results State -->
      <div
        v-else-if="libraryStore.filteredSongs.length === 0 && libraryStore.hasFilters"
        class="flex flex-col items-center justify-center py-12 text-center"
      >
        <div class="mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-muted">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-10 w-10 text-muted-foreground"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
        </div>
        <h3 class="mb-2 text-xl font-semibold text-foreground">
          No results found
        </h3>
        <p class="text-sm text-muted-foreground">
          Try adjusting your filters or search query
        </p>
      </div>

      <!-- Song Grid/List -->
      <div
        v-else
        class="grid gap-4"
        :class="{
          'grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4': libraryStore.viewMode === 'grid',
          'grid-cols-1': libraryStore.viewMode === 'list',
        }"
      >
        <SongCard
          v-for="song in libraryStore.filteredSongs"
          :key="song.id"
          :song="song"
          :is-selected="playbackStore.selectedSong?.id === song.id"
          @select="handleSongSelect"
        />
      </div>

      <!-- Song Count -->
      <div
        v-if="!libraryStore.loading && libraryStore.filteredSongs.length > 0"
        class="mt-6 text-center text-sm text-muted-foreground"
      >
        Showing {{ libraryStore.filteredSongs.length }} of {{ libraryStore.songCount }} songs
      </div>
    </div>
  </div>
</template>
