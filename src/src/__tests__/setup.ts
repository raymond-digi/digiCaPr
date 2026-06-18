import { vi } from 'vitest'

// Mock Tauri API
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/dialog', () => ({
  save: vi.fn(),
  open: vi.fn(),
}))

// Mock @mdi/font
vi.mock('@mdi/font/css/materialdesignicons.css', () => ({})); // eslint-disable-line

// Global test utilities
(globalThis as any).ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))
