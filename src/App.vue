<script setup lang="ts">
import { onMounted } from 'vue'
import LibraryView from '@/components/library/LibraryView.vue'
import SetlistView from '@/components/setlist/SetlistView.vue'
import ImportProgressModal from '@/components/modals/ImportProgressModal.vue'
import NewSetlistModal from '@/components/modals/NewSetlistModal.vue'
import AddSongToSetlistModal from '@/components/modals/AddSongToSetlistModal.vue'
import PreloadProgressModal from '@/components/modals/PreloadProgressModal.vue'
import SettingsModal from '@/components/modals/SettingsModal.vue'
import PlaybackControls from '@/components/playback/PlaybackControls.vue'
import SeekBar from '@/components/playback/SeekBar.vue'
import StemMixer from '@/components/playback/StemMixer.vue'
import DronePad from '@/components/playback/DronePad.vue'
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'
import { usePlaybackStore } from '@/stores/playback'
import { useLibraryStore } from '@/stores/library'

// Initialize keyboard shortcuts
useKeyboardShortcuts()

// Initialize stores
const playbackStore = usePlaybackStore()
const libraryStore = useLibraryStore()

onMounted(() => {
  playbackStore.initializeEventListeners()
  libraryStore.fetchSongs()
})
</script>

<template>
  <div class="flex h-screen flex-col bg-background text-foreground">
    <!-- Playback Controls -->
    <div class="p-6">
      <PlaybackControls />
      <div class="mt-3">
        <SeekBar />
      </div>
    </div>

    <!-- Main Content -->
    <main class="flex flex-1 overflow-hidden p-6">
      <!-- Setlist View -->
      <div class="flex flex-col gap-4">
        <div class="panel p-0">
          <SetlistView />
        </div>
        <div class="border-border panel p-4">
          <DronePad />
        </div>
      </div>

      <!-- Library View -->
      <div class="flex-1 overflow-hidden">
<!--        <LibraryView />-->
      </div>

      <!-- Stem Mixer Sidebar -->
      <aside class="w-80 border-l border-border p-4">
        <StemMixer />
      </aside>
    </main>

    <!-- Modals -->
    <ImportProgressModal />
    <NewSetlistModal />
    <AddSongToSetlistModal />
    <PreloadProgressModal />
    <SettingsModal />
  </div>
</template>