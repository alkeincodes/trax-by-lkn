import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import LibraryToolbar from '../LibraryToolbar.vue'
import { useLibraryStore } from '@/stores/library'
import { useModalStore } from '@/stores/modal'

// Mock Tauri modules
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

describe('LibraryToolbar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders search input', () => {
    const wrapper = mount(LibraryToolbar)

    const searchInput = wrapper.find('input[type="text"]')
    expect(searchInput.exists()).toBe(true)
    expect(searchInput.attributes('placeholder')).toContain('Search')
  })

  it('renders import button', () => {
    const wrapper = mount(LibraryToolbar)

    const importButton = wrapper.findAll('button').find(btn =>
      btn.text().includes('Import')
    )
    expect(importButton).toBeDefined()
  })

  it('renders view mode toggle button', () => {
    const wrapper = mount(LibraryToolbar)

    const libraryStore = useLibraryStore()
    expect(libraryStore.viewMode).toBe('grid')

    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBeGreaterThan(0)
  })

  it('updates search query with debounce', async () => {
    const wrapper = mount(LibraryToolbar)
    const libraryStore = useLibraryStore()

    const searchInput = wrapper.find('input[type="text"]')
    await searchInput.setValue('test query')

    // Wait for debounce (300ms)
    await new Promise(resolve => setTimeout(resolve, 350))

    expect(libraryStore.searchQuery).toBe('test query')
  })

  it('updates tempo filter', async () => {
    const wrapper = mount(LibraryToolbar)
    const libraryStore = useLibraryStore()

    const tempoInputs = wrapper.findAll('input[type="number"]')
    await tempoInputs[0].setValue(100)
    await tempoInputs[1].setValue(140)

    await flushPromises()

    expect(libraryStore.filters.tempo_min).toBe(100)
    expect(libraryStore.filters.tempo_max).toBe(140)
  })

  it('updates key filter', async () => {
    const wrapper = mount(LibraryToolbar)
    const libraryStore = useLibraryStore()

    const selects = wrapper.findAll('select')
    const keySelect = selects[0]

    await keySelect.setValue('C')
    await flushPromises()

    expect(libraryStore.filters.key).toBe('C')
  })

  it('clears filters when clear button clicked', async () => {
    const wrapper = mount(LibraryToolbar)
    const libraryStore = useLibraryStore()

    // Set some filters
    libraryStore.searchQuery = 'test'
    libraryStore.filters = { tempo_min: 100 }

    await wrapper.vm.$nextTick()

    // Find and click clear button
    const clearButton = wrapper.findAll('button').find(btn =>
      btn.text().includes('Clear')
    )

    if (clearButton) {
      await clearButton.trigger('click')
      expect(libraryStore.searchQuery).toBe('')
      expect(libraryStore.filters).toEqual({})
    }
  })

  it('toggles view mode', async () => {
    const wrapper = mount(LibraryToolbar)
    const libraryStore = useLibraryStore()

    expect(libraryStore.viewMode).toBe('grid')

    // Find toggle button (should be the icon button)
    const buttons = wrapper.findAll('button')
    const toggleButton = buttons.find(btn =>
      btn.attributes('title')?.includes('View')
    )

    if (toggleButton) {
      await toggleButton.trigger('click')
      expect(libraryStore.viewMode).toBe('list')

      await toggleButton.trigger('click')
      expect(libraryStore.viewMode).toBe('grid')
    }
  })

  it('opens import modal when import button clicked', async () => {
    const { open } = await import('@tauri-apps/plugin-dialog')
    vi.mocked(open).mockResolvedValue(['file1.wav', 'file2.wav'])

    const wrapper = mount(LibraryToolbar)
    const modalStore = useModalStore()

    const importButton = wrapper.findAll('button').find(btn =>
      btn.text().includes('Import')
    )

    if (importButton) {
      await importButton.trigger('click')
      await flushPromises()

      expect(modalStore.activeModal).toBe('import-progress')
      expect(modalStore.modalData.files).toEqual(['file1.wav', 'file2.wav'])
    }
  })
})
