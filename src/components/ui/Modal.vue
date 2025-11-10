<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { cn } from '@/lib/utils'
import type { ClassValue } from 'clsx'

interface ModalProps {
  open: boolean
  title?: string
  class?: ClassValue
}

const props = defineProps<ModalProps>()
const emit = defineEmits<{
  close: []
}>()

function handleBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    emit('close')
  }
}

function handleEscapeKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.open) {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleEscapeKey)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleEscapeKey)
})
</script>

<template>
  <Transition
    enter-active-class="transition-opacity duration-200"
    enter-from-class="opacity-0"
    enter-to-class="opacity-100"
    leave-active-class="transition-opacity duration-200"
    leave-from-class="opacity-100"
    leave-to-class="opacity-0"
  >
    <div
      v-if="open"
      class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/80"
      @click="handleBackdropClick"
    >
      <div
        :class="cn(
          'relative w-full max-w-lg rounded-lg border border-border bg-card p-6 shadow-lg',
          props.class
        )"
      >
        <!-- Header -->
        <div v-if="title || $slots.header" class="mb-4">
          <slot name="header">
            <h2 class="text-xl font-semibold text-card-foreground">
              {{ title }}
            </h2>
          </slot>
        </div>

        <!-- Content -->
        <div class="text-card-foreground">
          <slot />
        </div>

        <!-- Footer -->
        <div v-if="$slots.footer" class="mt-6">
          <slot name="footer" />
        </div>
      </div>
    </div>
  </Transition>
</template>
