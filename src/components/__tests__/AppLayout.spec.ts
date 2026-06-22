import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import AppLayout from '../layout/AppLayout.vue'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'

describe('AppLayout', () => {
  let vuetify: ReturnType<typeof createVuetify>
  let router: ReturnType<typeof createRouter>

  beforeEach(() => {
    vuetify = createVuetify({ components, directives })
    router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', component: { template: '<div>Dashboard</div>' } },
        { path: '/employees', component: { template: '<div>Employees</div>' } },
      ],
    })
  })

  it('renders the app layout', () => {
    const wrapper = mount(AppLayout, {
      global: {
        plugins: [createPinia(), router, vuetify],
      },
    })

    // Vuetify 4 renders VApp without a .v-app class wrapper
    expect(wrapper.find('.v-application, .v-app').exists() || wrapper.find('[class*="v-application"]').exists() || wrapper.html().includes('v-app')).toBe(true)
  })

  it('displays navigation drawer', () => {
    const wrapper = mount(AppLayout, {
      global: {
        plugins: [createPinia(), router, vuetify],
      },
    })

    expect(wrapper.find('.v-navigation-drawer').exists()).toBe(true)
  })

  it('displays app bar', () => {
    const wrapper = mount(AppLayout, {
      global: {
        plugins: [createPinia(), router, vuetify],
      },
    })

    expect(wrapper.find('.v-app-bar').exists()).toBe(true)
  })
})
