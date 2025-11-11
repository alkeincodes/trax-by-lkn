<script setup lang="ts">
import { ref, watch } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import { Search, Upload, Grid3x3, List, X } from 'lucide-vue-next'
import { open } from '@tauri-apps/plugin-dialog'
import Button from '@/components/ui/Button.vue'
import { useLibraryStore } from '@/stores/library'
import { useModalStore } from '@/stores/modal'
import { SortBy } from '@/types/library'

const libraryStore = useLibraryStore()
const modalStore = useModalStore()

const searchInput = ref('')
const tempoMin = ref<number>()
const tempoMax = ref<number>()
const selectedKey = ref<string>()
const sortBy = ref<SortBy>()

// Musical keys
const keys = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']

// Debounced search
const debouncedSearch = useDebounceFn((query: string) => {
  libraryStore.searchQuery = query
}, 300)

watch(searchInput, (newValue) => {
  debouncedSearch(newValue)
})

watch([tempoMin, tempoMax, selectedKey, sortBy], () => {
  libraryStore.filterSongs({
    search_query: searchInput.value,
    tempo_min: tempoMin.value,
    tempo_max: tempoMax.value,
    key: selectedKey.value,
    sort_by: sortBy.value,
  })
})

async function handleImport() {
  try {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: 'Audio Files',
          extensions: ['wav', 'mp3', 'flac'],
        },
      ],
    })


    if (selected && Array.isArray(selected) && selected.length > 0) {
      modalStore.openModal('import-progress', { files: selected })
    }
  } catch (error) {
    console.error('Failed to open file dialog:', error)
  }
}

function clearFilters() {
  searchInput.value = ''
  tempoMin.value = undefined
  tempoMax.value = undefined
  selectedKey.value = undefined
  sortBy.value = undefined
  libraryStore.clearFilters()
}

function toggleViewMode() {
  const newMode = libraryStore.viewMode === 'grid' ? 'list' : 'grid'
  libraryStore.setViewMode(newMode)
}
</script>

<template>
  <div class="space-y-4 border-b border-border py-4">
    <!-- Top Row: Search and Import -->
    <div class="flex items-center gap-3">
      <!-- Search Input -->
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <input
          v-model="searchInput"
          type="text"
          placeholder="Search song library..."
          class="w-full rounded-md border border-input bg-background py-2 pl-10 pr-4 text-sm focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
        />
      </div>

      <!-- View Mode Toggle -->
      <Button
        variant="outline"
        size="icon"
        @click="toggleViewMode"
        :title="libraryStore.viewMode === 'grid' ? 'Switch to List View' : 'Switch to Grid View'"
      >
        <Grid3x3 v-if="libraryStore.viewMode === 'grid'" class="h-4 w-4" />
        <List v-else class="h-4 w-4" />
      </Button>

      <!-- Import Button -->
      <Button @click="handleImport" class="gap-2">
        <Upload class="h-4 w-4" />
        Import
      </Button>
    </div>

    <!-- Bottom Row: Filters -->
    <div v-if="false" class="flex flex-wrap items-center gap-3">
      <!-- Tempo Range -->
      <div class="flex items-center gap-2">
        <label class="text-sm text-muted-foreground">Tempo:</label>
        <input
          v-model.number="tempoMin"
          type="number"
          placeholder="Min"
          class="w-20 rounded-md border border-input bg-background px-2 py-1 text-sm focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
        />
        <span class="text-muted-foreground">-</span>
        <input
          v-model.number="tempoMax"
          type="number"
          placeholder="Max"
          class="w-20 rounded-md border border-input bg-background px-2 py-1 text-sm focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
        />
        <span class="text-sm text-muted-foreground">BPM</span>
      </div>

      <!-- Key Filter -->
      <div class="flex items-center gap-2">
        <label class="text-sm text-muted-foreground">Key:</label>
        <select
          v-model="selectedKey"
          class="rounded-md border border-input bg-background px-3 py-1 text-sm focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
        >
          <option :value="undefined">All Keys</option>
          <option v-for="key in keys" :key="key" :value="key">
            {{ key }}
          </option>
        </select>
      </div>

      <!-- Sort By -->
      <div class="flex items-center gap-2">
        <label class="text-sm text-muted-foreground">Sort:</label>
        <select
          v-model="sortBy"
          class="rounded-md border border-input bg-background px-3 py-1 text-sm focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
        >
          <option :value="undefined">Default</option>
          <option :value="SortBy.Name">Name</option>
          <option :value="SortBy.Artist">Artist</option>
          <option :value="SortBy.Tempo">Tempo</option>
          <option :value="SortBy.Duration">Duration</option>
          <option :value="SortBy.DateAdded">Date Added</option>
        </select>
      </div>

      <!-- Clear Filters -->
      <Button
        v-if="libraryStore.hasFilters"
        variant="ghost"
        size="sm"
        @click="clearFilters"
        class="gap-1"
      >
        <X class="h-3 w-3" />
        Clear
      </Button>
    </div>
  </div>
</template>
