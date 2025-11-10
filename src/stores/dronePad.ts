import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type Key = 'C' | 'Db' | 'D' | 'Eb' | 'E' | 'F' | 'Gb' | 'G' | 'Ab' | 'A' | 'Bb' | 'B'

export interface DronePreset {
  id: string
  name: string
  description: string
  folder: string
}

export const useDronePadStore = defineStore('dronePad', () => {
  // State
  const isPlaying = ref(false)
  const selectedKey = ref<Key | null>(null)
  const selectedPreset = ref<DronePreset | null>(null)
  const volume = ref(1.0)
  const isFading = ref(false)
  const fadingDirection = ref<'in' | 'out' | null>(null)

  // Audio context and nodes
  const audioContext = ref<AudioContext | null>(null)
  const currentSource = ref<AudioBufferSourceNode | null>(null)
  const currentGainNode = ref<GainNode | null>(null)
  const audioBuffer = ref<AudioBuffer | null>(null)

  // For crossfading
  const previousSource = ref<AudioBufferSourceNode | null>(null)
  const previousGainNode = ref<GainNode | null>(null)

  // Drone presets
  const dronePresets = ref<DronePreset[]>([
    {
      id: 'warm-pads-churchfront',
      name: 'Warm Pads by Church front',
      description: 'Warm and uplifting atmosphere',
      folder: 'warm-pads-by-churchfront'
    },
    {
      id: 'ambient-pads-vishal',
      name: 'Ambient Pads by Vishal Bhojane',
      description: 'Ethereal and calming atmosphere',
      folder: 'ambient-pads-by-vishal-bhojane'
    }
  ])

  // Available keys in order
  const availableKeys: Key[] = ['C', 'Db', 'D', 'Eb', 'E', 'F', 'Gb', 'G', 'Ab', 'A', 'Bb', 'B']

  // Computed
  const currentAudioPath = computed(() => {
    if (!selectedPreset.value || !selectedKey.value) return null
    return `/drone-pads/${selectedPreset.value.folder}/${selectedKey.value}.mp3`
  })

  // Initialize audio context
  function initAudioContext() {
    if (!audioContext.value) {
      audioContext.value = new AudioContext()
    }
  }

  // Load audio file
  async function loadAudio(path: string): Promise<AudioBuffer> {
    initAudioContext()

    const response = await fetch(path)
    const arrayBuffer = await response.arrayBuffer()
    const buffer = await audioContext.value!.decodeAudioData(arrayBuffer)

    return buffer
  }

  // Fade in a specific gain node
  async function fadeInGainNode(gainNode: GainNode, duration: number = 10): Promise<void> {
    if (!audioContext.value) return

    const currentTime = audioContext.value.currentTime
    const currentGain = gainNode.gain.value

    // Cancel any existing automation
    gainNode.gain.cancelScheduledValues(currentTime)
    gainNode.gain.setValueAtTime(currentGain, currentTime)

    // Calculate remaining fade time based on current volume
    const remainingGain = volume.value - currentGain
    const adjustedDuration = remainingGain > 0 ? (remainingGain / volume.value) * duration : 0

    if (adjustedDuration > 0) {
      gainNode.gain.linearRampToValueAtTime(volume.value, currentTime + adjustedDuration)
    }

    return new Promise((resolve) => {
      setTimeout(resolve, adjustedDuration * 1000)
    })
  }

  // Fade out a specific gain node
  async function fadeOutGainNode(gainNode: GainNode, duration: number = 10): Promise<void> {
    if (!audioContext.value) return

    const currentTime = audioContext.value.currentTime
    const currentGain = gainNode.gain.value

    // Cancel any existing automation
    gainNode.gain.cancelScheduledValues(currentTime)
    gainNode.gain.setValueAtTime(currentGain, currentTime)

    // Calculate fade duration based on current volume
    const adjustedDuration = currentGain > 0 ? (currentGain / volume.value) * duration : 0

    if (adjustedDuration > 0) {
      gainNode.gain.linearRampToValueAtTime(0, currentTime + adjustedDuration)
    }

    return new Promise((resolve) => {
      setTimeout(resolve, adjustedDuration * 1000)
    })
  }

  // Stop current audio with fade out
  async function stopCurrentAudio() {
    if (currentSource.value && currentGainNode.value) {
      fadingDirection.value = 'out'
      await fadeOutGainNode(currentGainNode.value, 10)

      currentSource.value.stop()
      currentSource.value.disconnect()
      currentGainNode.value.disconnect()

      currentSource.value = null
      currentGainNode.value = null
      fadingDirection.value = null
    }
  }

  // Play audio with a new source and gain node
  async function playAudio() {
    if (!currentAudioPath.value) return

    try {
      initAudioContext()

      // Load the audio
      audioBuffer.value = await loadAudio(currentAudioPath.value)

      // Create new gain node for this source
      const newGainNode = audioContext.value!.createGain()
      newGainNode.connect(audioContext.value!.destination)
      newGainNode.gain.value = 0

      // Create source
      const newSource = audioContext.value!.createBufferSource()
      newSource.buffer = audioBuffer.value
      newSource.loop = true
      newSource.connect(newGainNode)

      // Start playback
      newSource.start(0)

      // Update refs
      currentSource.value = newSource
      currentGainNode.value = newGainNode
      isPlaying.value = true

      // Fade in over 10 seconds
      fadingDirection.value = 'in'
      await fadeInGainNode(newGainNode, 10)
      fadingDirection.value = null
    } catch (error) {
      console.error('Error playing drone pad:', error)
      isPlaying.value = false
      isFading.value = false
      fadingDirection.value = null
    }
  }

  // Toggle playback
  async function togglePlayback() {
    if (isPlaying.value) {
      // Turn off - update state immediately
      isPlaying.value = false
      isFading.value = true
      await stopCurrentAudio()
      isFading.value = false
    } else {
      // Turn on
      if (!selectedKey.value) {
        // Don't start if no key is selected
        return
      }
      if (!selectedPreset.value) {
        selectedPreset.value = dronePresets.value[0]
      }
      isFading.value = true
      await playAudio()
      isFading.value = false
    }
  }

  // Change key with crossfade
  async function changeKey(newKey: Key) {
    const wasPlaying = isPlaying.value
    selectedKey.value = newKey

    if (wasPlaying) {
      isFading.value = true

      // Move current to previous for fade out
      previousSource.value = currentSource.value
      previousGainNode.value = currentGainNode.value
      currentSource.value = null
      currentGainNode.value = null

      // Start fading out the previous audio (non-blocking)
      const fadeOutPromise = previousGainNode.value
        ? fadeOutGainNode(previousGainNode.value, 10).then(() => {
            // Clean up previous source after fade out
            if (previousSource.value) {
              previousSource.value.stop()
              previousSource.value.disconnect()
              previousSource.value = null
            }
            if (previousGainNode.value) {
              previousGainNode.value.disconnect()
              previousGainNode.value = null
            }
          })
        : Promise.resolve()

      // Start playing new key immediately (will fade in while old fades out)
      try {
        initAudioContext()

        // Load the new audio
        const newPath = `/drone-pads/${selectedPreset.value!.folder}/${newKey}.mp3`
        const newBuffer = await loadAudio(newPath)

        // Create new gain node
        const newGainNode = audioContext.value!.createGain()
        newGainNode.connect(audioContext.value!.destination)
        newGainNode.gain.value = 0

        // Create new source
        const newSource = audioContext.value!.createBufferSource()
        newSource.buffer = newBuffer
        newSource.loop = true
        newSource.connect(newGainNode)

        // Start new playback
        newSource.start(0)

        // Update refs
        currentSource.value = newSource
        currentGainNode.value = newGainNode

        // Fade in new audio (happens simultaneously with fade out)
        fadingDirection.value = 'in'
        await fadeInGainNode(newGainNode, 10)
        fadingDirection.value = null

        // Wait for fade out to complete
        await fadeOutPromise

        isFading.value = false
      } catch (error) {
        console.error('Error changing key:', error)
        isFading.value = false
      }
    }
  }

  // Change preset
  async function changePreset(preset: DronePreset) {
    const wasPlaying = isPlaying.value
    selectedPreset.value = preset

    if (wasPlaying) {
      // Fade out and stop when changing preset
      isFading.value = true
      await stopCurrentAudio()
      isPlaying.value = false
      isFading.value = false
    }
  }

  // Set volume
  function setVolume(newVolume: number) {
    volume.value = Math.max(0, Math.min(1, newVolume))
    if (gainNode.value && isPlaying.value) {
      const currentTime = audioContext.value!.currentTime
      gainNode.value.gain.cancelScheduledValues(currentTime)
      gainNode.value.gain.setValueAtTime(volume.value, currentTime)
    }
  }

  // Cleanup
  function cleanup() {
    if (currentSource.value) {
      currentSource.value.stop()
      currentSource.value.disconnect()
      currentSource.value = null
    }
    if (currentGainNode.value) {
      currentGainNode.value.disconnect()
      currentGainNode.value = null
    }
    if (previousSource.value) {
      previousSource.value.stop()
      previousSource.value.disconnect()
      previousSource.value = null
    }
    if (previousGainNode.value) {
      previousGainNode.value.disconnect()
      previousGainNode.value = null
    }
    if (audioContext.value) {
      audioContext.value.close()
      audioContext.value = null
    }
  }

  return {
    // State
    isPlaying,
    selectedKey,
    selectedPreset,
    volume,
    isFading,
    fadingDirection,
    dronePresets,
    availableKeys,
    currentAudioPath,

    // Actions
    togglePlayback,
    changeKey,
    changePreset,
    setVolume,
    cleanup
  }
})
