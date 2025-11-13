<script setup lang="ts">
import { computed } from 'vue'
import { Plus, Trash2, ChevronDown } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuPortal,
  DropdownMenuGroup,
} from 'radix-vue'
import DropdownMenu from '@/components/ui/DropdownMenu.vue'
import DropdownMenuItem from '@/components/ui/DropdownMenuItem.vue'
import DropdownMenuLabel from '@/components/ui/DropdownMenuLabel.vue'
import DropdownMenuSeparator from '@/components/ui/DropdownMenuSeparator.vue'
import { useSetlistStore } from '@/stores/setlist'
import { useModalStore } from '@/stores/modal'

const setlistStore = useSetlistStore()
const modalStore = useModalStore()

const showRecentSetlists = computed(() => setlistStore.recentSetlists.length > 0)

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

const openAddSongToSetlistModal = () => {
  if(!setlistStore.currentSetlist) return;
  modalStore.openModal('add-song-to-setlist')
}
</script>

<template>
  <div class="flex items-center gap-3 border-b border-border px-4 py-3">
    <!-- Setlist Dropdown -->
    <div class="flex-1">
      <DropdownMenuRoot>
        <DropdownMenuTrigger as-child>
          <button
            class="w-full flex items-center justify-between rounded-md border border-border bg-background px-3 py-2 text-foreground hover:bg-accent hover:text-accent-foreground focus:border-transparent focus:outline-none focus:ring-1 focus:ring-transparent transition-colors"
          >
            <span class="text-sm">
              {{ setlistStore.currentSetlist?.name || 'Select a setlist' }}
            </span>
            <ChevronDown :size="16" class="text-muted-foreground ml-2" />
          </button>
        </DropdownMenuTrigger>

        <DropdownMenuPortal>
          <DropdownMenu :side-offset="4" align="start" class="w-[var(--radix-dropdown-menu-trigger-width)]">
            <template v-if="setlistStore.allSetlists.length > 0">
              <DropdownMenuSeparator v-if="showRecentSetlists" />
              <DropdownMenuLabel class="text-white/20 text-xs font-normal">All Setlists</DropdownMenuLabel>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  v-for="setlist in setlistStore.allSetlists"
                  :key="setlist.id"
                  @select="() => handleSelectSetlist(setlist.id)"
                >
                  {{ setlist.name }}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem
                    @select="modalStore.openModal('new-setlist')"
                >
                  <Plus :size="16" class="mr-2" />
                  Create a Setlist
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </template>
          </DropdownMenu>
        </DropdownMenuPortal>
      </DropdownMenuRoot>
    </div>

    <!-- Action Buttons -->
    <div class="flex items-center gap-2">
      <!-- Save Status -->
<!--      <div class="flex items-center gap-2 text-sm text-muted-foreground">-->
<!--        <Save :size="14" />-->
<!--        <span>{{ saveStatus }}</span>-->
<!--      </div>-->

      <!-- New Setlist Button -->
      <Button
          variant="secondary"
          size="sm"
          :disabled="!setlistStore.currentSetlist"
          @click="openAddSongToSetlistModal"
      >
        <Plus :size="16" />
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
