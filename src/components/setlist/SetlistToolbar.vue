<script setup lang="ts">
import { computed } from 'vue'
import { Plus, Trash2, Save, ChevronDown } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import { useSetlistStore } from '@/stores/setlist'
import { useModalStore } from '@/stores/modal'

const setlistStore = useSetlistStore()
const modalStore = useModalStore()

const showRecentSetlists = computed(() => setlistStore.recentSetlists.length > 0)
const saveStatus = computed(() => {
  if (setlistStore.loading) return 'Saving...'
  return 'Saved'
})

function handleNewSetlist() {
  modalStore.openModal('new-setlist')
}

function handleDeleteSetlist() {
  if (!setlistStore.currentSetlist) return

  const confirmed = confirm(
    `Are you sure you want to delete "${setlistStore.currentSetlist.name}"?`
  )

  if (confirmed) {
    setlistStore.deleteSetlist(setlistStore.currentSetlist.id)
  }
}

async function handleSelectSetlist(id: string) {
  if (!id || id === '') {
    return
  }
  try {
    await setlistStore.loadSetlist(id)
  } catch (error) {
    console.error('Failed to load setlist:', error)
    alert(`Failed to load setlist: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}
</script>

<template>
  <div class="flex items-center gap-3 border-b border-border px-4 py-3">
    <!-- Setlist Dropdown -->
    <div class="flex-1">
      <div class="relative">
        <select
          :value="setlistStore.currentSetlist?.id || ''"
          class="w-full appearance-none rounded-md border border-border bg-background px-3 py-2 pr-8 text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
          @change="(e) => handleSelectSetlist((e.target as HTMLSelectElement).value)"
        >
          <option value="" disabled>Select a setlist</option>
          <optgroup v-if="showRecentSetlists" label="Recent">
            <option
              v-for="setlist in setlistStore.recentSetlists"
              :key="setlist.id"
              :value="setlist.id"
            >
              {{ setlist.name }}
            </option>
          </optgroup>
          <optgroup v-if="setlistStore.allSetlists.length > 0" label="All Setlists">
            <option
              v-for="setlist in setlistStore.allSetlists"
              :key="setlist.id"
              :value="setlist.id"
            >
              {{ setlist.name }}
            </option>
          </optgroup>
        </select>
        <div class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2">
          <ChevronDown :size="16" class="text-muted-foreground" />
        </div>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="flex items-center gap-2">
      <!-- Save Status -->
      <div class="flex items-center gap-2 text-sm text-muted-foreground">
        <Save :size="14" />
        <span>{{ saveStatus }}</span>
      </div>

      <!-- New Setlist Button -->
      <Button variant="default" size="sm" @click="handleNewSetlist">
        <Plus :size="16" class="mr-1" />
        New
      </Button>

      <!-- Delete Setlist Button -->
      <Button
        variant="destructive"
        size="sm"
        :disabled="!setlistStore.currentSetlist"
        @click="handleDeleteSetlist"
      >
        <Trash2 :size="16" />
      </Button>
    </div>
  </div>
</template>
