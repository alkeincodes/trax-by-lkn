import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import SettingsModal from '../SettingsModal.vue'
import { useModalStore } from '@/stores/modal'

describe('SettingsModal', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('does not render when modal is closed', () => {
    const wrapper = mount(SettingsModal, {
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

    expect(wrapper.find('select').exists()).toBe(false)
  })

  it('renders when modal is open', () => {
    const modalStore = useModalStore()
    modalStore.openModal('settings')

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title', 'class'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: true,
        },
      },
    })

    expect(wrapper.findAll('select').length).toBeGreaterThan(0)
  })

  it('renders all settings sections', () => {
    const modalStore = useModalStore()
    modalStore.openModal('settings')

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title', 'class'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: true,
        },
      },
    })

    expect(wrapper.text()).toContain('Audio')
    expect(wrapper.text()).toContain('Output Device')
    expect(wrapper.text()).toContain('Buffer Size')
    expect(wrapper.text()).toContain('Sample Rate')
    expect(wrapper.text()).toContain('Appearance')
    expect(wrapper.text()).toContain('Theme')
  })

  it('has default values', () => {
    const modalStore = useModalStore()
    modalStore.openModal('settings')

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title', 'class'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: true,
        },
      },
    })

    const selects = wrapper.findAll('select')
    expect(selects.length).toBeGreaterThan(0)
  })

  it('closes modal when cancel is clicked', async () => {
    const modalStore = useModalStore()
    modalStore.openModal('settings')

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title', 'class'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: {
            template: '<button @click="$attrs.onClick"><slot /></button>',
          },
        },
      },
    })

    const cancelButton = wrapper.findAll('button').find(btn => btn.text() === 'Cancel')
    await cancelButton?.trigger('click')

    expect(modalStore.activeModal).toBeNull()
  })

  it('allows selecting different buffer sizes', async () => {
    const modalStore = useModalStore()
    modalStore.openModal('settings')

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title', 'class'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: true,
        },
      },
    })

    const bufferSelect = wrapper.find('#buffer-size')
    expect(bufferSelect.exists()).toBe(true)

    await bufferSelect.setValue(256)
    expect((bufferSelect.element as HTMLSelectElement).value).toBe('256')
  })

  it('allows selecting different sample rates', async () => {
    const modalStore = useModalStore()
    modalStore.openModal('settings')

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title', 'class'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: true,
        },
      },
    })

    const sampleRateSelect = wrapper.find('#sample-rate')
    expect(sampleRateSelect.exists()).toBe(true)

    await sampleRateSelect.setValue(44100)
    expect((sampleRateSelect.element as HTMLSelectElement).value).toBe('44100')
  })

  it('allows selecting theme', async () => {
    const modalStore = useModalStore()
    modalStore.openModal('settings')

    const wrapper = mount(SettingsModal, {
      global: {
        stubs: {
          Modal: {
            props: ['open', 'title', 'class'],
            template: '<div v-if="open"><slot /><slot name="footer" /></div>',
          },
          Button: true,
        },
      },
    })

    const themeSelect = wrapper.find('#theme')
    expect(themeSelect.exists()).toBe(true)

    await themeSelect.setValue('light')
    expect((themeSelect.element as HTMLSelectElement).value).toBe('light')
  })
})
