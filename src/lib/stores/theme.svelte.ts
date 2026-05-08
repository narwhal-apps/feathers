import { browser } from '$app/environment';
import type { ThemeName } from '$lib/types';

class ThemeStore {
  /** User-pinned theme. `null` = follow OS. */
  override = $state<ThemeName | null>(null);
  /** Last-known OS preference. */
  systemPref = $state<ThemeName>('dark');

  /** What the UI actually renders. */
  effective = $derived<ThemeName>(this.override ?? this.systemPref);

  setOverride(value: ThemeName | null): void {
    this.override = value;
  }

  setSystemPref(value: ThemeName): void {
    this.systemPref = value;
  }
}

export const theme = new ThemeStore();

if (browser) {
  const mql = window.matchMedia('(prefers-color-scheme: dark)');
  theme.setSystemPref(mql.matches ? 'dark' : 'light');
  mql.addEventListener('change', (e) => {
    theme.setSystemPref(e.matches ? 'dark' : 'light');
  });
}
