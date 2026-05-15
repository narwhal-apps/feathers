/// <reference types="@testing-library/jest-dom" />
import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// ── Tauri runtime stubs ────────────────────────────────────────────
// Tauri ships a host-injected window.__TAURI_INTERNALS__ at runtime;
// jsdom has none. Stub the surface area touched by stores + components
// so render() doesn't throw. Per-test files can still vi.mock(...) for
// finer control (e.g. asserting specific invoke args).

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
  once: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: vi.fn().mockResolvedValue(undefined),
  openPath: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
  save: vi.fn().mockResolvedValue(null),
  message: vi.fn().mockResolvedValue(undefined),
  confirm: vi.fn().mockResolvedValue(false),
}));

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn().mockResolvedValue(null),
}));

vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn().mockResolvedValue(undefined),
  exit: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn().mockResolvedValue({
    get: vi.fn().mockResolvedValue(undefined),
    set: vi.fn().mockResolvedValue(undefined),
    save: vi.fn().mockResolvedValue(undefined),
    delete: vi.fn().mockResolvedValue(undefined),
  }),
}));

if (typeof window !== 'undefined' && !window.matchMedia) {
  // jsdom polyfill for prefers-color-scheme
  // @ts-expect-error - minimal polyfill
  window.matchMedia = (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  });
}
