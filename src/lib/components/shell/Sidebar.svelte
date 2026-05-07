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
    <header class="section-header">
      <span class="label">Changes</span>
      {#if changesCount > 0}<span class="count">{changesCount}</span>{/if}
    </header>
    <div class="row">
      <span class="indicator" class:on={changesCount > 0}></span>
      <span class="row-text">
        {activeId
          ? (changesCount === 0 ? 'Working tree clean' : `${changesCount} changed file${changesCount === 1 ? '' : 's'}`)
          : 'No repository selected'}
      </span>
    </div>
  </section>

  <section class="section">
    <header class="section-header">
      <span class="label">Pull Requests</span>
    </header>
    <div class="row">
      <span class="indicator"></span>
      <span class="row-text muted">—</span>
    </div>
  </section>
</aside>

<style>
  .sidebar {
    position: relative;
    width: 240px;
    flex-shrink: 0;
    background: var(--bg-elev-1);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: var(--sp-4) 0 var(--sp-6);
  }
  .sidebar::after {
    content: "";
    position: absolute;
    inset: 0;
    background-image: var(--grain);
    opacity: 0.4;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .section { padding: var(--sp-3) 0; position: relative; z-index: 1; }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--sp-4);
    margin-bottom: var(--sp-2);
  }
  .label {
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    font-weight: var(--weight-semibold);
  }
  .count {
    background: var(--accent-bg-medium);
    color: var(--accent-300);
    border-radius: var(--r-pill);
    padding: 1px 8px;
    font-size: var(--fs-2xs);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-semibold);
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-4);
  }
  .indicator {
    width: 6px;
    height: 6px;
    border-radius: var(--r-pill);
    background: var(--fg-faint);
    flex-shrink: 0;
  }
  .indicator.on { background: var(--accent-500); box-shadow: 0 0 8px var(--accent-bg-strong); }
  .row-text { color: var(--fg-muted); font-size: var(--fs-sm); }
  .row-text.muted { color: var(--fg-subtle); }
</style>
