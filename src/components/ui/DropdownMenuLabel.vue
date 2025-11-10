<script setup lang="ts">
import { DropdownMenuLabel, type DropdownMenuLabelProps } from 'radix-vue'
import { type ClassValue } from 'clsx'
import { cn } from '@/lib/utils'

interface LabelProps extends DropdownMenuLabelProps {
  class?: ClassValue
  inset?: boolean
}

const props = withDefaults(defineProps<LabelProps>(), {
  inset: false,
})
</script>

<template>
  <DropdownMenuLabel
    :class="cn(
      'px-2 py-1.5 text-sm font-semibold',
      props.inset && 'pl-8',
      props.class
    )"
    v-bind="props"
  >
    <slot />
  </DropdownMenuLabel>
</template>
