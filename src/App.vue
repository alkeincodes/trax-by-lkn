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
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'
import { usePlaybackStore } from '@/stores/playback'

// Initialize keyboard shortcuts
useKeyboardShortcuts()

// Initialize playback store event listeners
const playbackStore = usePlaybackStore()
onMounted(() => {
  playbackStore.initializeEventListeners()
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
    <main class="flex flex-1 overflow-hidden">
      <!-- Setlist View -->
      <div class="w-96 border-r border-border">
        <SetlistView />
      </div>

      <!-- Library View -->
      <div class="flex-1 overflow-hidden">
        <LibraryView />
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