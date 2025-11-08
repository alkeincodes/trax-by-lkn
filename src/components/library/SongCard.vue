<script setup lang="ts">
import { computed } from 'vue'
import type { Song } from '@/types/library'
import { Music2 } from 'lucide-vue-next'

interface SongCardProps {
  song: Song
  stemCount?: number
  isSelected?: boolean
}

const props = withDefaults(defineProps<SongCardProps>(), {
  isSelected: false
})
const emit = defineEmits<{
  select: [song: Song]
}>()

const formattedDuration = computed(() => {
  const minutes = Math.floor(props.song.duration / 60)
  const seconds = Math.floor(props.song.duration % 60)
  return `${minutes}:${seconds.toString().padStart(2, '0')}`
})

const displayArtist = computed(() => props.song.artist || 'Unknown Artist')

function handleClick() {
  emit('select', props.song)
}

function handleDragStart(e: DragEvent) {
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'copy'
    e.dataTransfer.setData('text/plain', props.song.id)
  }
}
</script>

<template>
  <div
    :class="[
      'group relative flex flex-col rounded-lg border p-4 transition-all hover:scale-[1.02] cursor-pointer',
      isSelected
        ? 'border-primary bg-primary/10 shadow-lg'
        : 'border-border bg-card hover:border-primary/50 hover:shadow-lg'
    ]"
    draggable="true"
    @click="handleClick"
    @dragstart="handleDragStart"
  >
    <!-- Icon -->
    <div class="mb-3 flex h-12 w-12 items-center justify-center rounded-md bg-primary/10">
      <Music2 class="h-6 w-6 text-primary" />
    </div>

    <!-- Song Info -->
    <div class="flex-1">
      <h3 class="mb-1 text-lg font-semibold text-card-foreground line-clamp-1">
        {{ song.name }}
      </h3>
      <p class="mb-2 text-sm text-muted-foreground line-clamp-1">
        {{ displayArtist }}
      </p>
    </div>

    <!-- Metadata -->
    <div class="flex items-center justify-between text-xs text-muted-foreground">
      <span>{{ formattedDuration }}</span>
      <div class="flex items-center gap-2">
        <span v-if="song.key" class="rounded bg-primary/10 px-2 py-0.5 font-medium text-primary">
          {{ song.key }}
        </span>
        <span v-if="song.tempo" class="rounded bg-accent px-2 py-0.5">
          {{ Math.round(song.tempo) }} BPM
        </span>
      </div>
    </div>

    <!-- Stem Count Badge -->
    <div v-if="stemCount !== undefined" class="absolute right-2 top-2">
      <span class="rounded-full bg-primary/20 px-2 py-1 text-xs font-medium text-primary">
        {{ stemCount }} stems
      </span>
    </div>
  </div>
</template>
