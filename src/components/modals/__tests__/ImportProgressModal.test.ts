import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ImportProgressModal from '../ImportProgressModal.vue'
import { useModalStore } from '@/stores/modal'

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('ImportProgressModal', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders when modal is open', async () => {
    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav'] })
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Import Audio Files')
  })

  it('shows file count', async () => {
    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav', 'file2.wav'] })
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('2 files selected')
  })

  it('requires title field', async () => {
    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav'] })
    await wrapper.vm.$nextTick()

    const titleInput = wrapper.find('input#title')
    expect(titleInput.exists()).toBe(true)
    expect(wrapper.text()).toContain('*') // Required indicator
  })

  it('allows optional artist field', async () => {
    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav'] })
    await wrapper.vm.$nextTick()

    const artistInput = wrapper.find('input#artist')
    expect(artistInput.exists()).toBe(true)
  })

  it('provides key selection dropdown', async () => {
    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav'] })
    await wrapper.vm.$nextTick()

    const keySelect = wrapper.find('select#key')
    expect(keySelect.exists()).toBe(true)
  })

  it('provides time signature selection', async () => {
    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav'] })
    await wrapper.vm.$nextTick()

    const timeSignatureSelect = wrapper.find('select#time-signature')
    expect(timeSignatureSelect.exists()).toBe(true)
  })

  it('calls importFiles when import button clicked', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValue('song-id-123')

    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav'] })
    await wrapper.vm.$nextTick()

    const titleInput = wrapper.find('input#title')
    await titleInput.setValue('Test Song')

    const importButton = wrapper.findAll('button').find(btn =>
      btn.text().includes('Import')
    )

    if (importButton) {
      await importButton.trigger('click')
      await flushPromises()

      expect(invoke).toHaveBeenCalledWith('import_files', expect.objectContaining({
        filePaths: ['file1.wav'],
        title: 'Test Song',
      }))
    }
  })

  it('shows success message after import', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockResolvedValue('song-id-123')

    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav'] })
    await wrapper.vm.$nextTick()

    const titleInput = wrapper.find('input#title')
    await titleInput.setValue('Test Song')

    const importButton = wrapper.findAll('button').find(btn =>
      btn.text().includes('Import')
    )

    if (importButton) {
      await importButton.trigger('click')
      await flushPromises()

      expect(wrapper.text()).toContain('Import successful')
    }
  })

  it('shows error message on import failure', async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    vi.mocked(invoke).mockRejectedValue(new Error('Import failed'))

    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav'] })
    await wrapper.vm.$nextTick()

    const titleInput = wrapper.find('input#title')
    await titleInput.setValue('Test Song')

    const importButton = wrapper.findAll('button').find(btn =>
      btn.text().includes('Import')
    )

    if (importButton) {
      await importButton.trigger('click')
      await flushPromises()

      expect(wrapper.text()).toContain('Import failed')
    }
  })

  it('closes modal when cancel button clicked', async () => {
    const wrapper = mount(ImportProgressModal)
    const modalStore = useModalStore()

    modalStore.openModal('import-progress', { files: ['file1.wav'] })
    await wrapper.vm.$nextTick()

    const cancelButton = wrapper.findAll('button').find(btn =>
      btn.text().includes('Cancel')
    )

    if (cancelButton) {
      await cancelButton.trigger('click')
      expect(modalStore.activeModal).toBeNull()
    }
  })
})
