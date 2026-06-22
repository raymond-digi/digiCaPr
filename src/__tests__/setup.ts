import { vi } from 'vitest'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
  open: vi.fn(),
}))

// Mock CSS imports that Vuetify and other components use
vi.mock('vuetify/styles', () => ({})); // eslint-disable-line
vi.mock('@mdi/font/css/materialdesignicons.css', () => ({})); // eslint-disable-line

// Global test utilities
class MockResizeObserver {
  observe = vi.fn()
  unobserve = vi.fn()
  disconnect = vi.fn()
}
;(globalThis as any).ResizeObserver = MockResizeObserver
