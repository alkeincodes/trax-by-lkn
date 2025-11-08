import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface ModalData {
  [key: string]: unknown
}

export const useModalStore = defineStore('modal', () => {
  const activeModal = ref<string | null>(null)
  const modalData = ref<ModalData>({})

  function openModal(name: string, data: ModalData = {}) {
    activeModal.value = name
    modalData.value = data
  }

  function closeModal() {
    activeModal.value = null
    modalData.value = {}
  }

  function isOpen(name: string): boolean {
    return activeModal.value === name
  }

  return {
    activeModal,
    modalData,
    openModal,
    closeModal,
    isOpen,
  }
})
