<script setup lang="ts">
import { ref } from 'vue'
import { GripVertical, X } from 'lucide-vue-next'
import type { Song } from '@/types/library'
import { cn } from '@/lib/utils'

interface SetlistItemProps {
  song: Song
  index: number
  isSelected?: boolean
}

interface SetlistItemEmits {
  select: [song: Song]
  remove: []
  dragstart: [index: number]
  dragend: []
  drop: [targetIndex: number]
}

const props = defineProps<SetlistItemProps>()
const emit = defineEmits<SetlistItemEmits>()

const isDragging = ref(false)
const isDropTarget = ref(false)

function handleDragStart(e: DragEvent) {
  isDragging.value = true
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', props.index.toString())
  }
  emit('dragstart', props.index)
}

function handleDragEnd() {
  isDragging.value = false
  emit('dragend')
}

function handleDragOver(e: DragEvent) {
  e.preventDefault()
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'move'
  }
  isDropTarget.value = true
}

function handleDragLeave() {
  isDropTarget.value = false
}

function handleDrop(e: DragEvent) {
  e.preventDefault()
  isDropTarget.value = false
  emit('drop', props.index)
}

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}
</script>

<template>
  <div
    :class="cn(
      'group flex items-center gap-3 rounded-md border border-border bg-card p-3 transition-all',
      isDragging && 'opacity-50',
      isDropTarget && 'border-primary bg-primary/10',
      isSelected && 'border-primary bg-primary/5'
    )"
    draggable="true"
    @dragstart="handleDragStart"
    @dragend="handleDragEnd"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <!-- Drag Handle -->
    <button
      class="cursor-grab text-muted-foreground opacity-0 transition-opacity hover:text-foreground group-hover:opacity-100 active:cursor-grabbing"
      type="button"
    >
      <GripVertical :size="20" />
    </button>

    <!-- Song Info (Clickable) -->
    <div
      class="flex-1 cursor-pointer"
      @click="emit('select', song)"
    >
      <div class="flex items-baseline gap-2">
        <span class="font-semibold text-card-foreground">{{ song.name }}</span>
        <span v-if="song.artist" class="text-sm text-muted-foreground">
          by {{ song.artist }}
        </span>
      </div>
      <div class="mt-1 flex gap-4 text-xs text-muted-foreground">
        <span v-if="song.key">{{ song.key }}</span>
        <span v-if="song.tempo">{{ song.tempo }} BPM</span>
        <span>{{ formatDuration(song.duration) }}</span>
      </div>
    </div>

    <!-- Remove Button -->
    <button
      class="rounded-md p-1 text-muted-foreground opacity-0 transition-all hover:bg-destructive/20 hover:text-destructive group-hover:opacity-100"
      type="button"
      @click="emit('remove')"
    >
      <X :size="16" />
    </button>
  </div>
</template>
