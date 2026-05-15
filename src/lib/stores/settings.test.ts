import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { settings } from './settings.svelte';
import { theme } from './theme.svelte';

const mockInvoke = vi.mocked(invoke);

describe('settings store', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    settings.loaded = false;
    settings.current = { theme_override: null, last_active_repo_path: null };
    theme.setOverride(null);
  });

  it('refresh() pulls AppSettings from settings_get and propagates to theme', async () => {
    mockInvoke.mockResolvedValueOnce({ theme_override: 'dark', last_active_repo_path: null });
    await settings.refresh();
    expect(mockInvoke).toHaveBeenCalledWith('settings_get');
    expect(settings.current.theme_override).toBe('dark');
    expect(settings.loaded).toBe(true);
    expect(theme.override).toBe('dark');
  });

  it('refresh() with null theme_override clears the theme override', async () => {
    theme.setOverride('light');
    mockInvoke.mockResolvedValueOnce({ theme_override: null, last_active_repo_path: null });
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
    settings.current = { theme_override: 'dark', last_active_repo_path: null };
    theme.setOverride('dark');
    mockInvoke.mockResolvedValueOnce(undefined);
    await settings.setTheme(null);
    expect(settings.current.theme_override).toBeNull();
    expect(theme.override).toBeNull();
  });

  describe('setLastActiveRepoPath()', () => {
    it('persists the path and updates the local cache', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      await settings.setLastActiveRepoPath('/Users/me/code/proj');
      expect(mockInvoke).toHaveBeenCalledWith('settings_set_last_active_repo_path', {
        path: '/Users/me/code/proj',
      });
      expect(settings.current.last_active_repo_path).toBe('/Users/me/code/proj');
    });

    it('is a no-op when the path is unchanged', async () => {
      settings.current = { theme_override: null, last_active_repo_path: '/Users/me/code/proj' };
      await settings.setLastActiveRepoPath('/Users/me/code/proj');
      expect(mockInvoke).not.toHaveBeenCalled();
    });

    it('null clears the value', async () => {
      settings.current = { theme_override: null, last_active_repo_path: '/Users/me/code/proj' };
      mockInvoke.mockResolvedValueOnce(undefined);
      await settings.setLastActiveRepoPath(null);
      expect(mockInvoke).toHaveBeenCalledWith('settings_set_last_active_repo_path', {
        path: null,
      });
      expect(settings.current.last_active_repo_path).toBeNull();
    });

    it('swallows IPC failures so the next launch just falls back to welcome', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('disk full'));
      await expect(settings.setLastActiveRepoPath('/p')).resolves.toBeUndefined();
      // Local cache still updates so the in-session UI is consistent.
      expect(settings.current.last_active_repo_path).toBe('/p');
    });
  });
});
