import { invoke } from '@tauri-apps/api/core';
import type { GitHubUser } from '$lib/types';

class GitHubStore {
  user = $state<GitHubUser | null>(null);
  loading = $state(false);
  /** Set after the first refresh resolves so the FE can distinguish
   *  "haven't checked yet" from "checked, no user". */
  hydrated = $state(false);

  async refresh(): Promise<void> {
    this.loading = true;
    try {
      this.user = await invoke<GitHubUser | null>('github_user');
    } catch {
      this.user = null;
    } finally {
      this.hydrated = true;
      this.loading = false;
    }
  }

  async signOut(): Promise<void> {
    await invoke('github_signout');
    this.user = null;
  }
}

export const github = new GitHubStore();
