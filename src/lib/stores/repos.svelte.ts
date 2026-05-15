import { invoke } from '@tauri-apps/api/core';
import type { RepoId, RepoSummary, RepoOpenResult } from '$lib/types';
import { settings } from '$lib/stores/settings.svelte';

class ReposStore {
  /** All known repos (loaded by repo_list_known and pushed to by repo_open). */
  knownRepos = $state<RepoSummary[]>([]);

  /** Set true once `refresh()` resolves at least once. Lets the welcome
   *  screen distinguish "no known repos" from "still hydrating". */
  hydrated = $state(false);

  /** The repo whose URL we're on (set by routes/repo/[id]/+layout.svelte). */
  activeRepoId = $state<RepoId | null>(null);

  /** Per-repo "last fetched" wall-clock timestamps (ms since epoch).
   *  Set when the user runs Fetch / Pull / Push from the titlebar so the
   *  UI can show "Last fetched 5m ago". Lives in-memory only. */
  lastFetched = $state<Record<RepoId, number>>({});

  markFetched(id: RepoId): void {
    this.lastFetched = { ...this.lastFetched, [id]: Date.now() };
  }

  activeRepo = $derived(
    this.activeRepoId == null
      ? null
      : this.knownRepos.find((r) => r.id === this.activeRepoId) ?? null,
  );

  async refresh(): Promise<void> {
    this.knownRepos = await invoke<RepoSummary[]>('repo_list_known');
    this.hydrated = true;
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
    // Capture the path before close drops the registry entry — we need
    // it to know whether to clear the persisted pointer below.
    const closedPath = this.knownRepos.find((r) => r.id === id)?.path ?? null;
    await invoke('repo_close', { id });
    await this.refresh();
    if (this.activeRepoId === id) this.activeRepoId = null;
    // Forget the persisted last-active pointer if it was this repo,
    // otherwise the next launch would try to redirect to a repo that
    // no longer exists in the registry.
    if (closedPath != null && settings.current.last_active_repo_path === closedPath) {
      void settings.setLastActiveRepoPath(null);
    }
  }
}

export const repos = new ReposStore();
