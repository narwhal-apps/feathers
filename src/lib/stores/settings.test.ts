import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { settings } from './settings.svelte';
import { theme } from './theme.svelte';

const mockInvoke = vi.mocked(invoke);

describe('settings store', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    settings.loaded = false;
    settings.current = { theme_override: null };
    theme.setOverride(null);
  });

  it('refresh() pulls AppSettings from settings_get and propagates to theme', async () => {
    mockInvoke.mockResolvedValueOnce({ theme_override: 'dark' });
    await settings.refresh();
    expect(mockInvoke).toHaveBeenCalledWith('settings_get');
    expect(settings.current.theme_override).toBe('dark');
    expect(settings.loaded).toBe(true);
    expect(theme.override).toBe('dark');
  });

  it('refresh() with null theme_override clears the theme override', async () => {
    theme.setOverride('light');
    mockInvoke.mockResolvedValueOnce({ theme_override: null });
    await settings.refresh();
    expect(theme.override).toBeNull();
  });

  it('setTheme() invokes settings_set_theme + updates current + propagates to theme', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await settings.setTheme('light');
    expect(mockInvoke).toHaveBeenCalledWith('settings_set_theme', { theme: 'light' });
    expect(settings.current.theme_override).toBe('light');
    expect(theme.override).toBe('light');
  });

  it('setTheme(null) clears the override', async () => {
    settings.current = { theme_override: 'dark' };
    theme.setOverride('dark');
    mockInvoke.mockResolvedValueOnce(undefined);
    await settings.setTheme(null);
    expect(settings.current.theme_override).toBeNull();
    expect(theme.override).toBeNull();
  });
});
