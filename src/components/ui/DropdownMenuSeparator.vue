<script setup lang="ts">
import { DropdownMenuSeparator, type DropdownMenuSeparatorProps } from 'radix-vue'
import { type ClassValue } from 'clsx'
import { cn } from '@/lib/utils'

interface SeparatorProps extends DropdownMenuSeparatorProps {
  class?: ClassValue
}

const props = defineProps<SeparatorProps>()
</script>

<template>
  <DropdownMenuSeparator
    :class="cn('-mx-1 my-1 h-px bg-muted', props.class)"
    v-bind="props"
  />
</template>
