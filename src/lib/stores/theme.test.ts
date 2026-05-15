import { describe, it, expect } from 'vitest';
import { theme } from './theme.svelte';

describe('theme store', () => {
  it('effective falls back to systemPref when no override is set', () => {
    theme.setOverride(null);
    theme.setSystemPref('light');
    expect(theme.effective).toBe('light');
    theme.setSystemPref('dark');
    expect(theme.effective).toBe('dark');
  });

  it('override pins effective regardless of systemPref', () => {
    theme.setSystemPref('dark');
    theme.setOverride('light');
    expect(theme.effective).toBe('light');
    theme.setSystemPref('light');
    expect(theme.effective).toBe('light');
  });

  it('clearing the override returns to systemPref', () => {
    theme.setSystemPref('light');
    theme.setOverride('dark');
    expect(theme.effective).toBe('dark');
    theme.setOverride(null);
    expect(theme.effective).toBe('light');
  });
});
