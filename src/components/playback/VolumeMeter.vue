<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  level: number // Peak level from 0.0 to 1.0+
}

const props = withDefaults(defineProps<Props>(), {
  level: 0
})

// Convert linear amplitude to dB scale for more realistic meter behavior
const levelDb = computed(() => {
  if (props.level <= 0) return -Infinity
  return 20 * Math.log10(props.level)
})

// Map dB to percentage (typical range: -60dB to +6dB)
const percentage = computed(() => {
  const db = levelDb.value
  if (db === -Infinity || db < -60) return 0
  if (db >= 6) return 100
  // Map -60dB to 0%, 0dB to ~85%, +6dB to 100%
  return ((db + 60) / 66) * 100
})

// Determine the color gradient position based on level
const gradientHeight = computed(() => {
  return Math.min(100, Math.max(0, percentage.value))
})

// Invert the percentage for clip-path (100% level = 0% clip, 0% level = 100% clip)
const clipPathInset = computed(() => {
  return 100 - gradientHeight.value
})
</script>

<template>
  <div class="volume-meter">

    <!-- Level indicators with animated clip-path -->
    <div class="trax-levels">
      <div
        class="level"
        :style="{ clipPath: `inset(${clipPathInset}% 0 0 0)` }"
      ></div>
    </div>
  </div>
</template>

<style scoped>
.volume-meter {
  position: relative;
  width: 15px;
  height: 100%;
  border-radius: 4px;
  overflow: hidden;
  background: hsl(var(--color-muted));
}

.meter-background {
  position: absolute;
  inset: 0;
  background: #010101;
  opacity: 0.3;
}

.trax-levels {
  position: absolute;
  inset: 0;
  display: flex;
  gap: 2px;
}

.level {
  flex: 1;
  background: linear-gradient(
    to bottom,
    hsl(0, 90%, 55%) 0%,
    hsl(30, 95%, 55%) 15%,
    hsl(45, 100%, 55%) 30%,
    hsl(60, 100%, 50%) 50%,
    hsl(80, 75%, 50%) 70%,
    hsl(120, 65%, 45%) 100%
  );
  transition: clip-path 50ms ease-out;
  box-shadow: 0 0 4px rgba(255, 255, 255, 0.3);
}
</style>
