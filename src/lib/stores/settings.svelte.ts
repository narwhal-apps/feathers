import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, ThemeName } from '$lib/types';
import { theme } from '$lib/stores/theme.svelte';

class SettingsStore {
  loaded = $state(false);
  current = $state<AppSettings>({ theme_override: null });

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
}

export const settings = new SettingsStore();
