/// <reference types="@testing-library/jest-dom" />
import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Default-stub Tauri's invoke so component tests that touch backend-bound
// stores don't throw. Individual tests can still vi.mock(...) to override.
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
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
