<script setup lang="ts">
import { Power } from 'lucide-vue-next'
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuPortal,
  DropdownMenuGroup,
} from 'radix-vue'
import DropdownMenu from '@/components/ui/DropdownMenu.vue'
import DropdownMenuItem from '@/components/ui/DropdownMenuItem.vue'
import DropdownMenuLabel from '@/components/ui/DropdownMenuLabel.vue'
import { useDronePadStore } from '@/stores/dronePad'
import { cn } from '@/lib/utils'

const dronePadStore = useDronePadStore()

function handleKeySelect(key: typeof dronePadStore.availableKeys[number]) {
  if (dronePadStore.selectedKey === key && dronePadStore.isPlaying) {
    // If clicking the same key while playing, turn off
    dronePadStore.togglePlayback()
  } else {
    dronePadStore.changeKey(key)
  }
}

function handlePresetSelect(presetId: string) {
  const preset = dronePadStore.dronePresets.find(p => p.id === presetId)
  if (preset) {
    dronePadStore.changePreset(preset)
  }
}
</script>

<template>
  <div class="flex flex-col w-full gap-4 rounded-lg h-full">
    <!-- Header with On/Off toggle and preset selector -->
    <div class="flex items-center gap-3">
      <!-- Power Button -->
      <button
        @click="dronePadStore.togglePlayback()"
        :class="cn(
          'flex h-12 w-12 items-center justify-center rounded-full border-2 transition-all duration-300',
          dronePadStore.isPlaying
            ? 'border-primary bg-primary text-primary-foreground shadow-lg shadow-primary/50'
            : 'border-muted-foreground/30 bg-muted text-muted-foreground hover:border-primary hover:bg-primary/10'
        )"
      >
        <Power :size="20" />
      </button>

      <!-- Preset Selector -->
      <div class="flex-1">
        <DropdownMenuRoot>
          <DropdownMenuTrigger as-child>
            <button
              class="w-full flex flex-col items-start rounded-md border border-border bg-background px-3 py-2 text-left hover:bg-accent hover:text-accent-foreground focus:border-transparent focus:outline-none focus:ring-1 focus:ring-transparent transition-colors"
            >
              <span class="text-sm font-medium">
                {{ dronePadStore.selectedPreset?.name || 'Select a drone pad' }}
              </span>
              <span class="text-xs text-muted-foreground">
                {{ dronePadStore.selectedPreset?.description || 'Choose a preset' }}
              </span>
            </button>
          </DropdownMenuTrigger>

          <DropdownMenuPortal>
            <DropdownMenu :side-offset="4" align="start" class="w-[var(--radix-dropdown-menu-trigger-width)]">
              <DropdownMenuGroup>
                <DropdownMenuItem
                  v-for="preset in dronePadStore.dronePresets"
                  :key="preset.id"
                  @select="() => handlePresetSelect(preset.id)"
                  :class="cn(
                    dronePadStore.selectedPreset?.id === preset.id && 'bg-accent'
                  )"
                >
                  <div class="flex flex-col">
                    <span class="font-medium">{{ preset.name }}</span>
                    <span class="text-xs text-muted-foreground">{{ preset.description }}</span>
                  </div>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenu>
          </DropdownMenuPortal>
        </DropdownMenuRoot>
      </div>
    </div>

    <!-- Key Selection Grid -->
    <div class="grid grid-cols-6 gap-2">
      <!-- Top row: Db, Eb, Gb, Ab, Bb, B -->
      <button
        v-for="key in ['Db', 'Eb', 'Gb', 'Ab', 'Bb', 'B', 'C', 'D', 'E', 'F', 'G', 'A']"
        :key="key || 'empty-1'"
        @click="key ? handleKeySelect(key as any) : null"
        :disabled="!key"
        :class="cn(
          'h-16 rounded-md text-lg font-semibold transition-all duration-200',
          !key && 'invisible',
          key && dronePadStore.selectedKey === key
            ? 'bg-primary text-primary-foreground shadow-lg shadow-primary/50'
            : 'border border-border bg-card text-foreground hover:bg-accent hover:text-accent-foreground'
        )"
      >
        {{ key }}
      </button>
    </div>

    <!-- Status indicator -->
    <div v-if="dronePadStore.isFading" class="text-center text-xs text-muted-foreground">
      {{ dronePadStore.fadingDirection === 'in' ? 'Fading in...' : 'Fading out...' }}
    </div>
  </div>
</template>
