<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import Pill from '$lib/components/primitives/Pill.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { openRepoFlow } from '$lib/components/dialogs/openRepo';
  import type { StatusSnapshot, BranchInfo } from '$lib/types';

  onMount(() => { repos.refresh(); });

  const activeId = $derived(repos.activeRepoId);

  const status = createQuery<StatusSnapshot | null>(
    () => activeId ? queryKeys.repoStatus(activeId) : ['noop'],
    () => activeId ? invoke<StatusSnapshot>('repo_status', { id: activeId }) : Promise.resolve(null),
  );

  const branches = createQuery<BranchInfo[] | null>(
    () => activeId ? queryKeys.repoBranches(activeId) : ['noop'],
    () => activeId ? invoke<BranchInfo[]>('branch_list', { id: activeId }) : Promise.resolve(null),
  );

  const changesCount = $derived.by(() => {
    const s = status.data;
    return s ? s.staged.length + s.unstaged.length + s.untracked.length + s.conflicted.length : 0;
  });

  const headBranch = $derived(branches.data?.find((b) => b.is_head) ?? null);
  const branchCount = $derived(branches.data?.length ?? 0);

  function activate(id: string) {
    repos.activeRepoId = id;
    goto(`/repo/${id}/changes/`);
  }
</script>

<aside class="sidebar">
  <section class="section">
    <header class="section-header">
      <span>Repos</span>
      <Button label="Open" variant="ghost" size="sm" onclick={openRepoFlow} />
    </header>
    {#if repos.knownRepos.length === 0}
      <div class="empty">
        <Icon name="FolderOpen" size={18} />
        <p>No repositories yet</p>
        <p class="hint">Click Open to add one.</p>
      </div>
    {:else}
      <ul class="repos">
        {#each repos.knownRepos as r}
          <li
            class:active={r.id === activeId}
            onclick={() => activate(r.id)}
          >{r.name}</li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="section">
    <header class="section-header">Branches{#if branchCount > 0}<span class="count">{branchCount}</span>{/if}</header>
    {#if !activeId}
      <div class="empty subtle">—</div>
    {:else if headBranch}
      <div class="branch-row">
        <Pill label={headBranch.name} tone="accent" />
        {#if headBranch.ahead || headBranch.behind}
          <span class="counts">↓ {headBranch.behind ?? 0}  ↑ {headBranch.ahead ?? 0}</span>
        {/if}
      </div>
    {:else}
      <div class="empty subtle">No branches</div>
    {/if}
  </section>

  <section class="section">
    <header class="section-header">Changes{#if changesCount > 0}<span class="count">{changesCount}</span>{/if}</header>
    <div class="empty subtle">{activeId ? (changesCount === 0 ? 'Working tree clean' : `${changesCount} changed`) : '—'}</div>
  </section>

  <section class="section">
    <header class="section-header">Pull Requests</header>
    <div class="empty subtle">—</div>
  </section>
</aside>

<style>
  .sidebar {
    width: 240px;
    flex-shrink: 0;
    background: var(--bg-elev-1);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: var(--sp-3) 0;
  }
  .section { padding: var(--sp-2) 0; }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--sp-3);
    margin-bottom: var(--sp-2);
    color: var(--fg-subtle);
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 600;
  }
  .count { background: var(--bg-elev-2); border-radius: 999px; padding: 0 6px; color: var(--fg-muted); font-size: var(--fs-xs); }

  .repos { list-style: none; margin: 0; padding: 0; }
  .repos li {
    padding: 4px var(--sp-3);
    cursor: pointer;
    color: var(--fg);
    font-size: var(--fs-sm);
  }
  .repos li:hover { background: var(--bg-elev-2); }
  .repos li.active { color: var(--accent-300); background: rgba(20, 184, 166, 0.10); }

  .branch-row { display: flex; align-items: center; gap: var(--sp-2); padding: 0 var(--sp-3); }
  .counts { color: var(--fg-muted); font-family: var(--font-mono); font-size: var(--fs-xs); font-variant-numeric: tabular-nums; }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-1);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    padding: var(--sp-3);
    text-align: center;
  }
  .empty p { margin: 0; }
  .empty .hint { color: var(--fg-subtle); font-size: var(--fs-xs); }
  .empty.subtle { color: var(--fg-subtle); padding: var(--sp-1) var(--sp-3); }
</style>
