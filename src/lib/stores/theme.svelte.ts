import { browser } from '$app/environment';

export type ThemeName = 'dark' | 'light';

// Reactive global theme. Mirrored to <html data-theme="..."> by the root layout.
export const theme = $state<{ value: ThemeName }>({ value: 'dark' });

if (browser) {
  const mql = window.matchMedia('(prefers-color-scheme: dark)');
  theme.value = mql.matches ? 'dark' : 'light';
  mql.addEventListener('change', (e) => {
    theme.value = e.matches ? 'dark' : 'light';
  });
}
