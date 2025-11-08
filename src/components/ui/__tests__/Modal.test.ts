import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import Modal from '../Modal.vue'

describe('Modal', () => {
  it('does not render when open is false', () => {
    const wrapper = mount(Modal, {
      props: {
        open: false,
        title: 'Test Modal',
      },
    })

    expect(wrapper.find('.fixed').exists()).toBe(false)
  })

  it('renders when open is true', () => {
    const wrapper = mount(Modal, {
      props: {
        open: true,
        title: 'Test Modal',
      },
    })

    expect(wrapper.find('.fixed').exists()).toBe(true)
    expect(wrapper.text()).toContain('Test Modal')
  })

  it('renders custom header slot', () => {
    const wrapper = mount(Modal, {
      props: {
        open: true,
      },
      slots: {
        header: '<h1>Custom Header</h1>',
      },
    })

    expect(wrapper.text()).toContain('Custom Header')
  })

  it('renders default slot content', () => {
    const wrapper = mount(Modal, {
      props: {
        open: true,
        title: 'Test Modal',
      },
      slots: {
        default: '<p>Modal content here</p>',
      },
    })

    expect(wrapper.text()).toContain('Modal content here')
  })

  it('renders footer slot', () => {
    const wrapper = mount(Modal, {
      props: {
        open: true,
        title: 'Test Modal',
      },
      slots: {
        footer: '<button>Close</button>',
      },
    })

    expect(wrapper.text()).toContain('Close')
  })

  it('emits close event when backdrop is clicked', async () => {
    const wrapper = mount(Modal, {
      props: {
        open: true,
        title: 'Test Modal',
      },
    })

    const backdrop = wrapper.find('.fixed')
    await backdrop.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('does not emit close when modal content is clicked', async () => {
    const wrapper = mount(Modal, {
      props: {
        open: true,
        title: 'Test Modal',
      },
      slots: {
        default: '<p>Content</p>',
      },
    })

    const content = wrapper.find('.relative')
    await content.trigger('click')

    expect(wrapper.emitted('close')).toBeFalsy()
  })

  it('emits close event when Escape key is pressed', async () => {
    const wrapper = mount(Modal, {
      props: {
        open: true,
        title: 'Test Modal',
      },
      attachTo: document.body,
    })

    const event = new KeyboardEvent('keydown', { key: 'Escape' })
    document.dispatchEvent(event)

    await wrapper.vm.$nextTick()

    expect(wrapper.emitted('close')).toBeTruthy()

    wrapper.unmount()
  })

  it('applies custom class', () => {
    const wrapper = mount(Modal, {
      props: {
        open: true,
        title: 'Test Modal',
        class: 'custom-class',
      },
    })

    const modalContent = wrapper.find('.relative')
    expect(modalContent.classes()).toContain('custom-class')
  })
})
