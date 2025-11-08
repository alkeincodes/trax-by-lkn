import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import NewSetlistModal from '../NewSetlistModal.vue'
import { useModalStore } from '@/stores/modal'

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock VueUse
vi.mock('@vueuse/core', () => ({
  useDebounceFn: vi.fn((fn) => fn),
}))

import { invoke } from '@tauri-apps/api/core'

describe('NewSetlistModal', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('does not render when modal is closed', () => {
    const wrapper = mount(NewSetlistModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open'],
            template: '<div v-if="open"><slot /></div>',
          },
          Button: true,
        },
      },
    })

    expect(wrapper.find('input').exists()).toBe(false)
  })

  it('renders when modal is open', async () => {
    const modalStore = useModalStore()

    const wrapper = mount(NewSetlistModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: true,
        },
      },
    })

    modalStore.openModal('new-setlist')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('input').exists()).toBe(true)
  })

  it('shows validation error when name is empty', async () => {
    const modalStore = useModalStore()

    const wrapper = mount(NewSetlistModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: {
            template: '<button @click="$attrs.onClick"><slot /></button>',
          },
        },
      },
    })

    modalStore.openModal('new-setlist')
    await wrapper.vm.$nextTick()

    const createButton = wrapper.findAll('button').find(btn => btn.text() === 'Create')
    await createButton?.trigger('click')

    expect(wrapper.text()).toContain('required')
  })

  it('creates setlist and closes modal on success', async () => {
    const modalStore = useModalStore()
    vi.mocked(invoke).mockResolvedValue('new-id')

    const wrapper = mount(NewSetlistModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: {
            template: '<button @click="$attrs.onClick"><slot /></button>',
          },
        },
      },
    })

    modalStore.openModal('new-setlist')
    await wrapper.vm.$nextTick()

    const input = wrapper.find('input')
    await input.setValue('Sunday Service')

    const createButton = wrapper.findAll('button').find(btn => btn.text() === 'Create')
    await createButton?.trigger('click')

    await wrapper.vm.$nextTick()

    expect(invoke).toHaveBeenCalledWith('create_setlist', { name: 'Sunday Service' })
    expect(modalStore.activeModal).toBeNull()
  })

  it('handles Enter key to submit', async () => {
    const modalStore = useModalStore()
    vi.mocked(invoke).mockResolvedValue('new-id')

    const wrapper = mount(NewSetlistModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: true,
        },
      },
    })

    modalStore.openModal('new-setlist')
    await wrapper.vm.$nextTick()

    const input = wrapper.find('input')
    await input.setValue('Sunday Service')
    await input.trigger('keydown', { key: 'Enter' })

    await wrapper.vm.$nextTick()

    expect(invoke).toHaveBeenCalledWith('create_setlist', { name: 'Sunday Service' })
  })

  it('closes modal when cancel is clicked', async () => {
    const modalStore = useModalStore()

    const wrapper = mount(NewSetlistModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: {
            template: '<button @click="$attrs.onClick"><slot /></button>',
          },
        },
      },
    })

    modalStore.openModal('new-setlist')
    await wrapper.vm.$nextTick()

    const cancelButton = wrapper.findAll('button').find(btn => btn.text() === 'Cancel')
    await cancelButton?.trigger('click')

    expect(modalStore.activeModal).toBeNull()
  })
})
