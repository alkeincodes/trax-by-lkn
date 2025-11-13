<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import Modal from '@/components/ui/Modal.vue'
import Button from '@/components/ui/Button.vue'
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuPortal,
} from 'radix-vue'
import DropdownMenu from '@/components/ui/DropdownMenu.vue'
import DropdownMenuItem from '@/components/ui/DropdownMenuItem.vue'
import { ChevronDown } from 'lucide-vue-next'
import { useModalStore } from '@/stores/modal'
import { invoke } from '@tauri-apps/api/core'
import type { AudioDevice } from '@/types/library'

const modalStore = useModalStore()

const isOpen = ref(false)
const isInitialLoad = ref(true)

// Settings state
const audioDevice = ref('')
const bufferSize = ref(512)
const sampleRate = ref(48000)

// Cache settings
const cacheSizeGB = ref(3)
const cacheStats = ref({
  numSongs: 0,
  currentBytes: 0,
  maxBytes: 0,
})

// Cache size options
const cacheSizeOptions = [
  { value: 1, label: '1 GB', description: '~2 songs' },
  { value: 2, label: '2 GB', description: '~3-4 songs' },
  { value: 3, label: '3 GB', description: '~5 songs, recommended' },
  { value: 4, label: '4 GB', description: '~6-7 songs' },
  { value: 6, label: '6 GB', description: '~10 songs' },
  { value: 8, label: '8 GB', description: '~13 songs' },
  { value: 12, label: '12 GB', description: '~20 songs' },
  { value: 16, label: '16 GB', description: '~26 songs' },
]

// Audio settings options
const audioDeviceOptions = ref<Array<{ value: string; label: string }>>([])

const bufferSizeOptions = [
  { value: 128, label: '128 samples', description: 'low latency' },
  { value: 256, label: '256 samples', description: 'balanced' },
  { value: 512, label: '512 samples', description: 'recommended' },
  { value: 1024, label: '1024 samples', description: 'stable' },
  { value: 2048, label: '2048 samples', description: 'high stability' },
]

const sampleRateOptions = [
  { value: 44100, label: '44.1 kHz', description: 'CD quality' },
  { value: 48000, label: '48 kHz', description: 'recommended' },
]

// Computed values
const cacheUsagePercent = computed(() => {
  if (cacheStats.value.maxBytes === 0) return 0
  return (cacheStats.value.currentBytes / cacheStats.value.maxBytes) * 100
})

const cacheUsageMB = computed(() => {
  return (cacheStats.value.currentBytes / 1_048_576).toFixed(1)
})

const cacheMaxMB = computed(() => {
  return (cacheStats.value.maxBytes / 1_048_576).toFixed(0)
})

const selectedAudioDeviceLabel = computed(() => {
  const option = audioDeviceOptions.value.find(opt => opt.value === audioDevice.value)
  return option ? option.label : audioDevice.value || 'Loading...'
})

const selectedCacheSizeLabel = computed(() => {
  const option = cacheSizeOptions.find(opt => opt.value === cacheSizeGB.value)
  return option ? `${option.label} (${option.description})` : `${cacheSizeGB.value} GB`
})

const selectedBufferSizeLabel = computed(() => {
  const option = bufferSizeOptions.find(opt => opt.value === bufferSize.value)
  return option ? `${option.label} (${option.description})` : `${bufferSize.value} samples`
})

const selectedSampleRateLabel = computed(() => {
  const option = sampleRateOptions.find(opt => opt.value === sampleRate.value)
  return option ? `${option.label}${option.description ? ' (' + option.description + ')' : ''}` : `${sampleRate.value} Hz`
})

// Watch for modal open state
watch(
  () => modalStore.activeModal,
  (newValue) => {
    isOpen.value = newValue === 'settings'
    if (isOpen.value) {
      // Reset initial load flag and load current settings
      isInitialLoad.value = true
      loadSettings()
    }
  }
)

// Watch for settings changes and apply immediately
watch(cacheSizeGB, async (newValue) => {
  if (!isOpen.value) return
  try {
    const cacheSizeBytes = newValue * 1024 * 1024 * 1024
    await invoke('set_cache_size', { sizeBytes: cacheSizeBytes })
    // Reload stats to show updated values
    const stats = await invoke<[number, number, number]>('get_cache_stats')
    cacheStats.value = {
      numSongs: stats[0],
      currentBytes: stats[1],
      maxBytes: stats[2],
    }
  } catch (e) {
    console.error('Failed to update cache size:', e)
  }
})

// Watch for audio settings changes and apply immediately
watch(audioDevice, async (newValue) => {
  if (!isOpen.value || !newValue || isInitialLoad.value) return
  try {
    await invoke('switch_audio_device', { deviceName: newValue })
    console.log('Audio device switched to:', newValue)

    // Emit event for Web Audio API components (like DronePad)
    window.dispatchEvent(new CustomEvent('audio-device-changed', {
      detail: { deviceName: newValue }
    }))
  } catch (e) {
    console.error('Failed to switch audio device:', e)
  }
})

watch(bufferSize, async (newValue) => {
  if (!isOpen.value || isInitialLoad.value) return
  try {
    await invoke('set_buffer_size', { bufferSize: newValue })
    console.log('Buffer size saved:', newValue)
  } catch (e) {
    console.error('Failed to save buffer size:', e)
  }
})

watch(sampleRate, async (newValue) => {
  if (!isOpen.value || isInitialLoad.value) return
  try {
    await invoke('set_sample_rate', { sampleRate: newValue })
    console.log('Sample rate saved:', newValue)
  } catch (e) {
    console.error('Failed to save sample rate:', e)
  }
})

async function loadSettings() {
  try {
    // Load cache stats from backend
    const stats = await invoke<[number, number, number]>('get_cache_stats')
    cacheStats.value = {
      numSongs: stats[0],
      currentBytes: stats[1],
      maxBytes: stats[2],
    }

    // Convert max bytes to GB for display
    cacheSizeGB.value = Math.round(stats[2] / (1024 * 1024 * 1024))

    // Load audio devices from backend
    const devices = await invoke<AudioDevice[]>('get_audio_devices')
    audioDeviceOptions.value = devices.map(device => ({
      value: device.name,
      label: device.name,
    }))

    // Load saved audio settings from database
    const audioSettings = await invoke<{
      audio_output_device: string | null
      audio_buffer_size: number
      sample_rate: number
      theme: string
    }>('get_audio_settings')

    // Set audio device (use saved setting or default)
    if (audioSettings.audio_output_device) {
      audioDevice.value = audioSettings.audio_output_device
    } else {
      const defaultDevice = devices.find(d => d.is_default)
      if (defaultDevice) {
        audioDevice.value = defaultDevice.name
      } else if (devices.length > 0) {
        audioDevice.value = devices[0].name
      }
    }

    // Load other audio settings
    bufferSize.value = audioSettings.audio_buffer_size
    sampleRate.value = audioSettings.sample_rate

    // Allow watchers to fire for user changes
    isInitialLoad.value = false
  } catch (e) {
    console.error('Failed to load settings:', e)
    isInitialLoad.value = false
  }
}

function handleClose() {
  modalStore.closeModal()
}

async function handleClearCache() {
  if (!confirm('Are you sure you want to clear all cached songs? They will need to be reloaded before playback.')) {
    return
  }

  try {
    await invoke('clear_cache')
    await loadSettings() // Reload stats
  } catch (e) {
    console.error('Failed to clear cache:', e)
  }
}
</script>

<template>
  <Modal :open="isOpen" title="Settings" class="max-w-2xl" @close="handleClose">
    <div class="space-y-6">
      <!-- Audio Settings -->
      <div>
        <h3 class="mb-3 font-semibold text-foreground">Audio</h3>
        <p class="mb-3 text-xs text-muted-foreground">
          Audio device can be switched during playback without interruption
        </p>
        <div class="space-y-4">
          <!-- Audio Output Device -->
          <div>
            <label class="mb-2 block text-sm font-medium text-foreground">
              Output Device
            </label>
            <DropdownMenuRoot>
              <DropdownMenuTrigger as-child>
                <button
                  class="flex w-full items-center justify-between rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground hover:bg-accent hover:text-accent-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                >
                  <span>{{ selectedAudioDeviceLabel }}</span>
                  <ChevronDown :size="16" class="ml-2 opacity-50" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuPortal>
                <DropdownMenu class="w-[var(--radix-dropdown-menu-trigger-width)]">
                  <DropdownMenuItem
                    v-for="option in audioDeviceOptions"
                    :key="option.value"
                    @select="audioDevice = option.value"
                  >
                    {{ option.label }}
                  </DropdownMenuItem>
                </DropdownMenu>
              </DropdownMenuPortal>
            </DropdownMenuRoot>
          </div>

          <!-- Buffer Size -->
          <div>
            <label class="mb-2 block text-sm font-medium text-foreground">
              Buffer Size
            </label>
            <DropdownMenuRoot>
              <DropdownMenuTrigger as-child>
                <button
                  class="flex w-full items-center justify-between rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground hover:bg-accent hover:text-accent-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                >
                  <span>{{ selectedBufferSizeLabel }}</span>
                  <ChevronDown :size="16" class="ml-2 opacity-50" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuPortal>
                <DropdownMenu class="w-[var(--radix-dropdown-menu-trigger-width)]">
                  <DropdownMenuItem
                    v-for="option in bufferSizeOptions"
                    :key="option.value"
                    @select="bufferSize = option.value"
                  >
                    {{ option.label }} ({{ option.description }})
                  </DropdownMenuItem>
                </DropdownMenu>
              </DropdownMenuPortal>
            </DropdownMenuRoot>
            <p class="mt-1 text-xs text-muted-foreground">
              Lower values reduce latency but may cause audio dropouts
            </p>
          </div>

          <!-- Sample Rate -->
          <div>
            <label class="mb-2 block text-sm font-medium text-foreground">
              Sample Rate
            </label>
            <DropdownMenuRoot>
              <DropdownMenuTrigger as-child>
                <button
                  class="flex w-full items-center justify-between rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground hover:bg-accent hover:text-accent-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                >
                  <span>{{ selectedSampleRateLabel }}</span>
                  <ChevronDown :size="16" class="ml-2 opacity-50" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuPortal>
                <DropdownMenu class="w-[var(--radix-dropdown-menu-trigger-width)]">
                  <DropdownMenuItem
                    v-for="option in sampleRateOptions"
                    :key="option.value"
                    @select="sampleRate = option.value"
                  >
                    {{ option.label }}{{ option.description ? ' (' + option.description + ')' : '' }}
                  </DropdownMenuItem>
                </DropdownMenu>
              </DropdownMenuPortal>
            </DropdownMenuRoot>
          </div>
        </div>
      </div>

      <!-- Cache Settings -->
      <div>
        <h3 class="mb-3 font-semibold text-foreground">Memory Cache</h3>
        <div class="space-y-4">
          <!-- Cache Size -->
          <div>
            <label class="mb-2 block text-sm font-medium text-foreground">
              Cache Size
            </label>
            <DropdownMenuRoot>
              <DropdownMenuTrigger as-child>
                <button
                  class="flex w-full items-center justify-between rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground hover:bg-accent hover:text-accent-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                >
                  <span>{{ selectedCacheSizeLabel }}</span>
                  <ChevronDown :size="16" class="ml-2 opacity-50" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuPortal>
                <DropdownMenu class="w-[var(--radix-dropdown-menu-trigger-width)]">
                  <DropdownMenuItem
                    v-for="option in cacheSizeOptions"
                    :key="option.value"
                    @select="cacheSizeGB = option.value"
                  >
                    {{ option.label }} ({{ option.description }})
                  </DropdownMenuItem>
                </DropdownMenu>
              </DropdownMenuPortal>
            </DropdownMenuRoot>
            <p class="mt-1 text-xs text-muted-foreground">
              Larger cache allows more songs to stay in memory for instant playback
            </p>
          </div>

          <!-- Cache Stats -->
          <div class="rounded-md border border-border bg-muted/30 p-4">
            <div class="mb-2 flex items-center justify-between text-sm">
              <span class="text-muted-foreground">Current Usage</span>
              <span class="font-mono text-foreground">
                {{ cacheUsageMB }} MB / {{ cacheMaxMB }} MB
              </span>
            </div>

            <!-- Progress Bar -->
            <div class="mb-2 h-2 w-full overflow-hidden rounded-full bg-muted">
              <div
                class="h-full bg-primary transition-all duration-300"
                :style="{ width: `${cacheUsagePercent}%` }"
              />
            </div>

            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground">
                {{ cacheStats.numSongs }} {{ cacheStats.numSongs === 1 ? 'song' : 'songs' }} cached
              </span>
              <Button
                variant="ghost"
                size="sm"
                class="h-auto px-2 py-1 text-xs"
                @click="handleClearCache"
              >
                Clear Cache
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Modal>
</template>
