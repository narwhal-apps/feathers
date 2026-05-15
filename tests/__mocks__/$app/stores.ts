/**
 * Minimal $app/stores mock for vitest. Real SvelteKit only exposes
 * `page` / `navigating` / `updated` inside the dev server; under
 * jsdom we hand the consuming components a writable readable that
 * looks like a SvelteKit Page object with sensible defaults.
 */
import { writable, readable } from 'svelte/store';

const defaultPage = {
  url: new URL('http://localhost/'),
  params: {} as Record<string, string>,
  route: { id: null as string | null },
  status: 200,
  error: null as Error | null,
  data: {} as Record<string, unknown>,
  form: null as unknown,
  state: {} as Record<string, unknown>,
};

export const page = writable(defaultPage);
export const navigating = readable(null);
export const updated = readable(false);
