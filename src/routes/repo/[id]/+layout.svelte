<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { page } from '$app/stores';
  import { repos } from '$lib/stores/repos.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { provideRepoQueries } from '$lib/stores/repo-context';
  import RepoActions from '$lib/components/shell/RepoActions.svelte';
  import type { StatusSnapshot, BranchInfo, OpState } from '$lib/types';

  let { children } = $props();

  // Sync URL → active repo.
  $effect(() => {
    repos.activeRepoId = $page.params.id ?? null;
  });

  const id = $derived($page.params.id ?? '');
  const active = $derived($page.url.pathname.split('/')[3] ?? 'changes');

  // Single subscription for status / branches / op-state, shared with every
  // child route via Svelte context. Without this, each consumer page (changes,
  // history, …) calls createQuery independently and pays the notify cost N
  // times even though the cache entry is identical.
  const status = createQuery<StatusSnapshot | null>(
    () => id ? queryKeys.repoStatus(id) : ['noop'],
    () => id ? invoke<StatusSnapshot>('repo_status', { id }) : Promise.resolve(null),
  );
  const branches = createQuery<BranchInfo[] | null>(
    () => id ? queryKeys.repoBranches(id) : ['noop'],
    () => id ? invoke<BranchInfo[]>('branch_list', { id }) : Promise.resolve(null),
  );
  const opState = createQuery<OpState | null>(
    () => id ? queryKeys.repoOpState(id) : ['noop'],
    () => id ? invoke<OpState>('repo_op_state', { id }) : Promise.resolve(null),
  );

  provideRepoQueries({ status, branches, opState });

  const changesCount = $derived.by(() => {
    const s = status.data;
    return s ? s.staged.length + s.unstaged.length + s.untracked.length + s.conflicted.length : 0;
  });
</script>

<nav class="tabs">
  <a
    class="tab"
    class:active={active === 'changes'}
    href={`/repo/${id}/changes/`}
  >
    <span>Changes</span>
    {#if changesCount > 0}
      <span class="badge">{changesCount}</span>
    {/if}
  </a>
  <a
    class="tab"
    class:active={active === 'history'}
    href={`/repo/${id}/history/`}
  >
    <span>History</span>
  </a>
  <a
    class="tab"
    class:active={active === 'pull-requests'}
    href={`/repo/${id}/pull-requests/`}
  >
    <span>Pull requests</span>
  </a>

  <div class="tab-actions">
    <RepoActions />
  </div>
</nav>

<div class="content">
  {@render children?.()}
</div>

<style>
  .tabs {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 0 var(--sp-3);
    border-bottom: 1px solid var(--border);
    background: var(--bg-elev-1);
  }
  /* Pushes RepoActions to the right edge of the tab strip. */
  .tab-actions { margin-left: auto; padding: 4px 0; }
  .tab {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: var(--sp-2) var(--sp-3);
    color: var(--fg-muted);
    border-bottom: 2px solid transparent;
    font-size: var(--fs-sm);
    font-weight: 600;
    text-decoration: none;
  }
  .tab:hover { color: var(--fg); }
  .tab.active { color: var(--accent-fg); border-bottom-color: var(--accent-500); }
  .badge {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    border-radius: var(--r-pill);
    padding: 1px 8px;
    font-size: var(--fs-2xs);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-semibold);
    line-height: 16px;
  }
  .content {
    flex: 1;
    overflow: hidden;
    min-height: 0;
    min-width: 0;
  }
</style>
