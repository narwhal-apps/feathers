import { invoke } from '@tauri-apps/api/core';
import type { RepoId, RepoSummary, RepoOpenResult } from '$lib/types';

class ReposStore {
  /** All known repos (loaded by repo_list_known and pushed to by repo_open). */
  knownRepos = $state<RepoSummary[]>([]);

  /** The repo whose URL we're on (set by routes/repo/[id]/+layout.svelte). */
  activeRepoId = $state<RepoId | null>(null);

  activeRepo = $derived(
    this.activeRepoId == null
      ? null
      : this.knownRepos.find((r) => r.id === this.activeRepoId) ?? null,
  );

  async refresh(): Promise<void> {
    this.knownRepos = await invoke<RepoSummary[]>('repo_list_known');
  }

  async open(path: string): Promise<RepoOpenResult> {
    const result = await invoke<RepoOpenResult>('repo_open', { path });
    // Reuse semantics: result.id may already exist in knownRepos. Refresh.
    await this.refresh();
    return result;
  }

  async clone(url: string, dest: string): Promise<RepoOpenResult> {
    const result = await invoke<RepoOpenResult>('repo_clone', { url, dest });
    await this.refresh();
    return result;
  }

  async close(id: RepoId): Promise<void> {
    await invoke('repo_close', { id });
    await this.refresh();
    if (this.activeRepoId === id) this.activeRepoId = null;
  }
}

export const repos = new ReposStore();
