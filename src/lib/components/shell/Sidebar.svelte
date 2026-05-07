<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import type { StatusSnapshot } from '$lib/types';

  onMount(() => { repos.refresh(); });

  const activeId = $derived(repos.activeRepoId);

  const status = createQuery<StatusSnapshot | null>(
    () => activeId ? queryKeys.repoStatus(activeId) : ['noop'],
    () => activeId ? invoke<StatusSnapshot>('repo_status', { id: activeId }) : Promise.resolve(null),
  );

  const changesCount = $derived.by(() => {
    const s = status.data;
    return s ? s.staged.length + s.unstaged.length + s.untracked.length + s.conflicted.length : 0;
  });
</script>

<aside class="sidebar">
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

  .empty.subtle { color: var(--fg-subtle); padding: var(--sp-1) var(--sp-3); }
</style>
