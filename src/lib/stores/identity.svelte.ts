import { invoke } from '@tauri-apps/api/core';
import type { GitIdentity } from '$lib/types';

/** Local git config identity (`user.name` / `user.email`). Loaded once on
 *  app start; consumed by anything that needs to recognise commits as
 *  authored by the current user — e.g. so we can hand the avatar a real
 *  GitHub photo for our own commits even when the email isn't a no-reply. */
class IdentityStore {
  email = $state<string | null>(null);
  name = $state<string | null>(null);
  hydrated = $state(false);

  async refresh(): Promise<void> {
    try {
      const id = await invoke<GitIdentity>('settings_get_git_identity');
      this.email = id.email;
      this.name = id.name;
    } catch {
      this.email = null;
      this.name = null;
    } finally {
      this.hydrated = true;
    }
  }
}

export const identity = new IdentityStore();
