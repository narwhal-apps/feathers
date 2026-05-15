import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, ThemeName } from '$lib/types';
import { theme } from '$lib/stores/theme.svelte';

class SettingsStore {
  loaded = $state(false);
  current = $state<AppSettings>({ theme_override: null, last_active_repo_path: null });

  async refresh(): Promise<void> {
    this.current = await invoke<AppSettings>('settings_get');
    theme.setOverride(this.current.theme_override);
    this.loaded = true;
  }

  async setTheme(value: ThemeName | null): Promise<void> {
    await invoke('settings_set_theme', { theme: value });
    this.current = { ...this.current, theme_override: value };
    theme.setOverride(value);
  }

  /** Persist the canonical path of the last-active repo (or null to
   *  clear). Stores by PATH, not registry id — ids aren't stable across
   *  restarts. Local cache updates synchronously so subsequent reads
   *  see the new value without waiting for the IPC roundtrip. */
  async setLastActiveRepoPath(path: string | null): Promise<void> {
    if (this.current.last_active_repo_path === path) return;
    this.current = { ...this.current, last_active_repo_path: path };
    try {
      await invoke('settings_set_last_active_repo_path', { path });
    } catch {
      // Best-effort; the next launch just won't auto-open.
    }
  }
}

export const settings = new SettingsStore();
