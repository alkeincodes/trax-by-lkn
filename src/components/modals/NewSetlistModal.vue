<script setup lang="ts">
import { ref, watch } from 'vue'
import Modal from '@/components/ui/Modal.vue'
import Button from '@/components/ui/Button.vue'
import { useModalStore } from '@/stores/modal'
import { useSetlistStore } from '@/stores/setlist'

const modalStore = useModalStore()
const setlistStore = useSetlistStore()

const name = ref('')
const error = ref('')
const isSubmitting = ref(false)

const isOpen = ref(false)

// Watch for modal open state
watch(
  () => modalStore.activeModal,
  (newValue) => {
    isOpen.value = newValue === 'new-setlist'
    if (isOpen.value) {
      // Reset form when opening
      name.value = ''
      error.value = ''
      isSubmitting.value = false
    }
  }
)

function handleClose() {
  modalStore.closeModal()
}

async function handleSubmit() {
  // Validate
  if (!name.value.trim()) {
    error.value = 'Setlist name is required'
    return
  }

  isSubmitting.value = true
  error.value = ''

  try {
    await setlistStore.createSetlist(name.value.trim())
    handleClose()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to create setlist'
  } finally {
    isSubmitting.value = false
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !isSubmitting.value) {
    handleSubmit()
  }
}
</script>

<template>
  <Modal :open="isOpen" title="Create New Setlist" @close="handleClose">
    <div class="space-y-4">
      <!-- Name Input -->
      <div>
        <label for="setlist-name" class="mb-2 block text-sm font-medium text-foreground">
          Setlist Name
        </label>
        <input
          id="setlist-name"
          v-model="name"
          type="text"
          placeholder="e.g., Sunday Service - Nov 10"
          class="w-full rounded-md border border-border bg-background px-3 py-2 text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
          :disabled="isSubmitting"
          @keydown="handleKeydown"
        />
        <p v-if="error" class="mt-1 text-sm text-destructive">{{ error }}</p>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button variant="ghost" :disabled="isSubmitting" @click="handleClose">
          Cancel
        </Button>
        <Button variant="default" :disabled="isSubmitting" @click="handleSubmit">
          {{ isSubmitting ? 'Creating...' : 'Create' }}
        </Button>
      </div>
    </template>
  </Modal>
</template>
