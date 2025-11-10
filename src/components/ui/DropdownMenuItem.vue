<script setup lang="ts">
import { DropdownMenuItem, type DropdownMenuItemProps } from 'radix-vue'
import { type ClassValue } from 'clsx'
import { cn } from '@/lib/utils'

interface MenuItemProps extends DropdownMenuItemProps {
  class?: ClassValue
  inset?: boolean
}

const props = withDefaults(defineProps<MenuItemProps>(), {
  inset: false,
})

const emit = defineEmits<{
  select: [event: Event]
}>()
</script>

<template>
  <DropdownMenuItem
    :class="cn(
      'relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors',
      'focus:bg-accent focus:text-accent-foreground',
      'data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
      props.inset && 'pl-8',
      props.class
    )"
    v-bind="props"
    @select="emit('select', $event)"
  >
    <slot />
  </DropdownMenuItem>
</template>
