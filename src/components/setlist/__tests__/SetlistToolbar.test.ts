import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import SetlistToolbar from '../SetlistToolbar.vue'
import { useSetlistStore } from '@/stores/setlist'
import { useModalStore } from '@/stores/modal'
import type { Setlist } from '@/types/library'

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock VueUse
vi.mock('@vueuse/core', () => ({
  useDebounceFn: vi.fn((fn) => fn),
}))

describe('SetlistToolbar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders dropdown with no setlist selected by default', () => {
    const wrapper = mount(SetlistToolbar, {
      global: {
        stubs: {
          Button: true,
        },
      },
    })

    const select = wrapper.find('select')
    expect(select.exists()).toBe(true)
    expect(select.element.value).toBe('')
  })

  it('displays all setlists in dropdown', () => {
    const store = useSetlistStore()
    store.allSetlists = [
      {
        id: '1',
        name: 'Sunday Service',
        created_at: Date.now(),
        updated_at: Date.now(),
        song_ids: [],
      },
      {
        id: '2',
        name: 'Wednesday Practice',
        created_at: Date.now(),
        updated_at: Date.now(),
        song_ids: [],
      },
    ]

    const wrapper = mount(SetlistToolbar, {
      global: {
        stubs: {
          Button: true,
        },
      },
    })

    const options = wrapper.findAll('option')
    expect(options.length).toBeGreaterThan(0)
    expect(wrapper.text()).toContain('Sunday Service')
    expect(wrapper.text()).toContain('Wednesday Practice')
  })

  it('opens new setlist modal when New button is clicked', async () => {
    const modalStore = useModalStore()

    const wrapper = mount(SetlistToolbar, {
      global: {
        stubs: {
          Button: {
            template: '<button @click="$attrs.onClick"><slot /></button>',
          },
        },
      },
    })

    const newButton = wrapper.findAll('button').find(btn => btn.text().includes('New'))
    expect(newButton).toBeTruthy()

    await newButton?.trigger('click')

    expect(modalStore.activeModal).toBe('new-setlist')
  })

  it('shows save status', () => {
    const wrapper = mount(SetlistToolbar, {
      global: {
        stubs: {
          Button: true,
        },
      },
    })

    expect(wrapper.text()).toContain('Saved')
  })

  it('disables delete button when no setlist is loaded', () => {
    const wrapper = mount(SetlistToolbar, {
      global: {
        stubs: {
          Button: {
            props: ['disabled'],
            template: '<button :disabled="disabled"><slot /></button>',
          },
        },
      },
    })

    const buttons = wrapper.findAll('button')
    const deleteButton = buttons.find(btn => btn.attributes('disabled') !== undefined)
    expect(deleteButton).toBeTruthy()
  })

  it('calls loadSetlist when selecting from dropdown', async () => {
    const store = useSetlistStore()
    const mockSetlist: Setlist = {
      id: '1',
      name: 'Sunday Service',
      created_at: Date.now(),
      updated_at: Date.now(),
      song_ids: [],
    }
    store.allSetlists = [mockSetlist]

    vi.spyOn(store, 'loadSetlist')

    const wrapper = mount(SetlistToolbar, {
      global: {
        stubs: {
          Button: true,
        },
      },
    })

    const select = wrapper.find('select')
    await select.setValue('1')

    expect(store.loadSetlist).toHaveBeenCalledWith('1')
  })
})
