<script setup lang="ts">
import { ref, computed } from 'vue'
import { ChevronDown } from 'lucide-vue-next'
import Modal from '@/components/ui/Modal.vue'
import Button from '@/components/ui/Button.vue'
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuPortal,
  DropdownMenuGroup,
} from 'radix-vue'
import DropdownMenu from '@/components/ui/DropdownMenu.vue'
import DropdownMenuItem from '@/components/ui/DropdownMenuItem.vue'
import { useModalStore } from '@/stores/modal'
import { useLibraryStore } from '@/stores/library'

const modalStore = useModalStore()
const libraryStore = useLibraryStore()

const isOpen = computed(() => modalStore.isOpen('import-progress'))
const files = computed(() => (modalStore.modalData.files as string[]) || [])

const title = ref('')
const artist = ref('')
const key = ref('')
const timeSignature = ref('4/4')

const importing = ref(false)
const importError = ref<string | null>(null)
const importSuccess = ref(false)

const keys = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B']
const timeSignatures = ['3/4', '4/4', '5/4', '6/8']

async function handleImport() {
  if (!title.value.trim()) {
    importError.value = 'Title is required'
    return
  }

  importing.value = true
  importError.value = null
  importSuccess.value = false

  try {
    await libraryStore.importFiles(
      files.value,
      title.value,
      artist.value || undefined,
      key.value || undefined,
      timeSignature.value || undefined
    )

    importSuccess.value = true

    // Close modal after short delay
    setTimeout(() => {
      handleClose()
    }, 1500)
  } catch (error) {
    importError.value = error instanceof Error ? error.message : 'Import failed'
  } finally {
    importing.value = false
  }
}

function handleClose() {
  // Reset form
  title.value = ''
  artist.value = ''
  key.value = ''
  timeSignature.value = '4/4'
  importing.value = false
  importError.value = null
  importSuccess.value = false

  modalStore.closeModal()
}
</script>

<template>
  <Modal :open="isOpen" title="Import Audio Files" @close="handleClose">
    <div class="space-y-4">
      <!-- File Count -->
      <div class="rounded-md bg-muted p-3">
        <p class="text-sm text-muted-foreground">
          {{ files.length }} file{{ files.length !== 1 ? 's' : '' }} selected
        </p>
      </div>

      <!-- Success Message -->
      <div
        v-if="importSuccess"
        class="rounded-md border border-primary bg-primary/10 p-3 text-sm text-primary"
      >
        Import successful! Song added to library.
      </div>

      <!-- Error Message -->
      <div
        v-if="importError"
        class="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive"
      >
        {{ importError }}
      </div>

      <!-- Form -->
      <div v-if="!importSuccess" class="space-y-4">
        <!-- Title (Required) -->
        <div>
          <label for="title" class="mb-1 block text-sm font-medium text-foreground">
            Title <span class="text-destructive">*</span>
          </label>
          <input
            id="title"
            v-model="title"
            type="text"
            placeholder="Enter song title"
            class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
            :disabled="importing"
          />
        </div>

        <!-- Artist (Optional) -->
        <div>
          <label for="artist" class="mb-1 block text-sm font-medium text-foreground">
            Artist
          </label>
          <input
            id="artist"
            v-model="artist"
            type="text"
            placeholder="Enter artist name"
            class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
            :disabled="importing"
          />
        </div>

        <!-- Key (Optional) -->
        <div>
          <label for="key" class="mb-1 block text-sm font-medium text-foreground">
            Key
          </label>
          <DropdownMenuRoot>
            <DropdownMenuTrigger as-child>
              <button
                class="w-full flex items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                :disabled="importing"
              >
                <span>{{ key || 'Select key' }}</span>
                <ChevronDown :size="16" class="text-muted-foreground ml-2" />
              </button>
            </DropdownMenuTrigger>

            <DropdownMenuPortal>
              <DropdownMenu :side-offset="4" align="start" class="w-[var(--radix-dropdown-menu-trigger-width)]">
                <DropdownMenuGroup>
                  <DropdownMenuItem @select="() => key = ''">
                    Select key
                  </DropdownMenuItem>
                  <DropdownMenuItem
                      v-for="k in keys"
                      :key="k"
                      @select="() => key = k"
                  >
                    {{ k }}
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenu>
            </DropdownMenuPortal>
          </DropdownMenuRoot>
        </div>

        <!-- Time Signature (Optional) -->
        <div>
          <label for="time-signature" class="mb-1 block text-sm font-medium text-foreground">
            Time Signature
          </label>
          <DropdownMenuRoot>
            <DropdownMenuTrigger as-child>
              <button
                class="w-full flex items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                :disabled="importing"
              >
                <span>{{ timeSignature }}</span>
                <ChevronDown :size="16" class="text-muted-foreground ml-2" />
              </button>
            </DropdownMenuTrigger>

            <DropdownMenuPortal>
              <DropdownMenu :side-offset="4" align="start" class="w-[var(--radix-dropdown-menu-trigger-width)]">
                <DropdownMenuGroup>
                  <DropdownMenuItem
                    v-for="sig in timeSignatures"
                    :key="sig"
                    @select="() => timeSignature = sig"
                  >
                    {{ sig }}
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenu>
            </DropdownMenuPortal>
          </DropdownMenuRoot>
        </div>
      </div>
    </div>

    <!-- Footer Buttons -->
    <template #footer>
      <div class="flex justify-end gap-2">
        <Button variant="outline" @click="handleClose" :disabled="importing">
          Cancel
        </Button>
        <Button
          v-if="!importSuccess"
          @click="handleImport"
          :disabled="importing || !title.trim()"
        >
          {{ importing ? 'Importing...' : 'Import' }}
        </Button>
      </div>
    </template>
  </Modal>
</template>
