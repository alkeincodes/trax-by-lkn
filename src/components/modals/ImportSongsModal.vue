<script setup lang="ts">
import { X } from 'lucide-vue-next'
import Modal from '@/components/ui/Modal.vue'
import Button from '@/components/ui/Button.vue'
import LibraryView from '@/components/library/LibraryView.vue'
import { useModalStore } from '@/stores/modal'

const modalStore = useModalStore()

function handleClose() {
  modalStore.closeModal()
}
</script>

<template>
  <Modal
    :open="modalStore.activeModal === 'import-songs'"
    @close="handleClose"
  >
    <div class="flex h-[90vh] w-[90vw] max-w-7xl flex-col rounded-lg bg-card">
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-border p-4">
        <h2 class="text-xl font-semibold text-foreground">Import Songs</h2>
        <button
          class="rounded-md p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          @click="handleClose"
        >
          <X :size="20" />
        </button>
      </div>

      <!-- Library View Content -->
      <div class="flex-1 overflow-hidden">
        <LibraryView />
      </div>

      <!-- Footer -->
      <div class="flex justify-end border-t border-border p-4">
        <Button variant="secondary" @click="handleClose">
          Done
        </Button>
      </div>
    </div>
  </Modal>
</template>
