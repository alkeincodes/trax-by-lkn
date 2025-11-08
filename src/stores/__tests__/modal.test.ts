import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useModalStore } from '../modal'

describe('Modal Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initializes with null active modal', () => {
    const store = useModalStore()

    expect(store.activeModal).toBeNull()
    expect(store.modalData).toEqual({})
  })

  it('opens modal with name and data', () => {
    const store = useModalStore()
    const testData = { test: 'value' }

    store.openModal('test-modal', testData)

    expect(store.activeModal).toBe('test-modal')
    expect(store.modalData).toEqual(testData)
  })

  it('closes modal and clears data', () => {
    const store = useModalStore()

    store.openModal('test-modal', { test: 'value' })
    store.closeModal()

    expect(store.activeModal).toBeNull()
    expect(store.modalData).toEqual({})
  })

  it('checks if modal is open correctly', () => {
    const store = useModalStore()

    expect(store.isOpen('test-modal')).toBe(false)

    store.openModal('test-modal')

    expect(store.isOpen('test-modal')).toBe(true)
    expect(store.isOpen('other-modal')).toBe(false)
  })
})
