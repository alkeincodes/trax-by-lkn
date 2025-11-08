<script setup lang="ts">
import { ref, watch } from 'vue'
import Modal from '@/components/ui/Modal.vue'
import Button from '@/components/ui/Button.vue'
import { useModalStore } from '@/stores/modal'

const modalStore = useModalStore()

const isOpen = ref(false)
const isSubmitting = ref(false)

// Settings state
const audioDevice = ref('default')
const bufferSize = ref(512)
const sampleRate = ref(48000)
const theme = ref<'dark' | 'light'>('dark')

// Watch for modal open state
watch(
  () => modalStore.activeModal,
  (newValue) => {
    isOpen.value = newValue === 'settings'
    if (isOpen.value) {
      // Load current settings
      loadSettings()
    }
  }
)

function loadSettings() {
  // TODO: Load settings from backend
  // For now, use defaults
  audioDevice.value = 'default'
  bufferSize.value = 512
  sampleRate.value = 48000
  theme.value = 'dark'
}

function handleClose() {
  modalStore.closeModal()
}

async function handleSave() {
  isSubmitting.value = true

  try {
    // TODO: Call backend to save settings
    // await invoke('save_settings', {
    //   audioDevice: audioDevice.value,
    //   bufferSize: bufferSize.value,
    //   sampleRate: sampleRate.value,
    //   theme: theme.value,
    // })

    handleClose()
  } catch (e) {
    console.error('Failed to save settings:', e)
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <Modal :open="isOpen" title="Settings" class="max-w-2xl" @close="handleClose">
    <div class="space-y-6">
      <!-- Audio Settings -->
      <div>
        <h3 class="mb-3 font-semibold text-foreground">Audio</h3>
        <div class="space-y-4">
          <!-- Audio Output Device -->
          <div>
            <label for="audio-device" class="mb-2 block text-sm font-medium text-foreground">
              Output Device
            </label>
            <select
              id="audio-device"
              v-model="audioDevice"
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
              :disabled="isSubmitting"
            >
              <option value="default">Default Output</option>
              <option value="system">System Audio</option>
            </select>
          </div>

          <!-- Buffer Size -->
          <div>
            <label for="buffer-size" class="mb-2 block text-sm font-medium text-foreground">
              Buffer Size
            </label>
            <select
              id="buffer-size"
              v-model.number="bufferSize"
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
              :disabled="isSubmitting"
            >
              <option :value="128">128 samples (low latency)</option>
              <option :value="256">256 samples (balanced)</option>
              <option :value="512">512 samples (recommended)</option>
              <option :value="1024">1024 samples (stable)</option>
              <option :value="2048">2048 samples (high stability)</option>
            </select>
            <p class="mt-1 text-xs text-muted-foreground">
              Lower values reduce latency but may cause audio dropouts
            </p>
          </div>

          <!-- Sample Rate -->
          <div>
            <label for="sample-rate" class="mb-2 block text-sm font-medium text-foreground">
              Sample Rate
            </label>
            <select
              id="sample-rate"
              v-model.number="sampleRate"
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
              :disabled="isSubmitting"
            >
              <option :value="44100">44.1 kHz (CD quality)</option>
              <option :value="48000">48 kHz (recommended)</option>
              <option :value="88200">88.2 kHz</option>
              <option :value="96000">96 kHz</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Appearance Settings -->
      <div>
        <h3 class="mb-3 font-semibold text-foreground">Appearance</h3>
        <div class="space-y-4">
          <!-- Theme -->
          <div>
            <label for="theme" class="mb-2 block text-sm font-medium text-foreground">
              Theme
            </label>
            <select
              id="theme"
              v-model="theme"
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
              :disabled="isSubmitting"
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
            </select>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button variant="ghost" :disabled="isSubmitting" @click="handleClose">
          Cancel
        </Button>
        <Button variant="default" :disabled="isSubmitting" @click="handleSave">
          {{ isSubmitting ? 'Saving...' : 'Save' }}
        </Button>
      </div>
    </template>
  </Modal>
</template>
