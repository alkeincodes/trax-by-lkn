<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Button from '@/components/ui/Button.vue'

const greetMsg = ref('')
const name = ref('')

async function greet() {
  greetMsg.value = await invoke('greet', { name: name.value })
}
</script>

<template>
  <main class="min-h-screen bg-background text-foreground p-8">
    <div class="max-w-4xl mx-auto">
      <h1 class="text-4xl font-bold text-center mb-8">TraX by LKN</h1>

      <div class="flex justify-center gap-4 mb-8">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" class="h-24 hover:drop-shadow-[0_0_2em_#747bff]" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="h-24 hover:drop-shadow-[0_0_2em_#24c8db]" alt="Tauri logo" />
        </a>
        <a href="https://vuejs.org/" target="_blank">
          <img src="./assets/vue.svg" class="h-24 hover:drop-shadow-[0_0_2em_#249b73]" alt="Vue logo" />
        </a>
      </div>

      <p class="text-center mb-8 text-muted-foreground">
        Click on the Tauri, Vite, and Vue logos to learn more.
      </p>

      <form class="flex justify-center gap-2 mb-4" @submit.prevent="greet">
        <input
          id="greet-input"
          v-model="name"
          placeholder="Enter a name..."
          class="px-4 py-2 rounded-md border border-input bg-background"
        />
        <Button type="submit">Greet</Button>
      </form>

      <p v-if="greetMsg" class="text-center">{{ greetMsg }}</p>

      <div class="mt-12 space-y-4">
        <h2 class="text-2xl font-semibold mb-4">Component Examples</h2>
        <div class="flex flex-wrap gap-2">
          <Button>Default Button</Button>
          <Button variant="secondary">Secondary</Button>
          <Button variant="destructive">Destructive</Button>
          <Button variant="outline">Outline</Button>
          <Button variant="ghost">Ghost</Button>
          <Button variant="link">Link</Button>
        </div>
        <div class="flex flex-wrap gap-2">
          <Button size="sm">Small</Button>
          <Button>Default</Button>
          <Button size="lg">Large</Button>
        </div>
      </div>
    </div>
  </main>
</template>