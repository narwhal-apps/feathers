<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { page } from '$app/stores';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import DiffView from '$lib/components/primitives/DiffView.svelte';
  import type { StatusSnapshot, DiffPayload, FileChange, FileStatus } from '$lib/types';

  function shortStatus(s: FileStatus): string {
    switch (s) {
      case 'added': return 'A';
      case 'modified': return 'M';
      case 'deleted': return 'D';
      case 'renamed': return 'R';
      case 'typechange': return 'T';
      case 'untracked': return 'U';
      case 'conflicted': return 'C';
    }
  }
  function isEmpty(s: StatusSnapshot): boolean {
    return s.staged.length + s.unstaged.length + s.untracked.length + s.conflicted.length === 0;
  }

  const id = $derived($page.params.id ?? '');

  const status = createQuery<StatusSnapshot>(
    () => queryKeys.repoStatus(id),
    () => invoke<StatusSnapshot>('repo_status', { id }),
  );

  let selected = $state<string | null>(null);

  const diff = createQuery<DiffPayload>(
    () => queryKeys.repoDiffWorkdir(id, selected),
    () =>
      selected == null
        ? Promise.resolve({ files: [] })
        : invoke<DiffPayload>('diff_workdir', { id, paths: [selected] }),
  );

  function pick(f: FileChange) {
    selected = f.path;
  }
</script>

<div class="layout">
  <aside class="files">
    {#if status.loading}
      <p class="hint">Loading…</p>
    {:else if status.error}
      <p class="err">Failed: {String(status.error)}</p>
    {:else if status.data}
      {#if status.data.staged.length > 0}
        <h3>Staged</h3>
        <ul>
          {#each status.data.staged as f}
            <li class:selected={selected === f.path}>
              <button class="row" onclick={() => pick(f)}>
              <span class="status status-{f.status}">{shortStatus(f.status)}</span>
              <span class="path">{f.path}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
      {#if status.data.unstaged.length > 0}
        <h3>Unstaged</h3>
        <ul>
          {#each status.data.unstaged as f}
            <li class:selected={selected === f.path}>
              <button class="row" onclick={() => pick(f)}>
              <span class="status status-{f.status}">{shortStatus(f.status)}</span>
              <span class="path">{f.path}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
      {#if status.data.untracked.length > 0}
        <h3>Untracked</h3>
        <ul>
          {#each status.data.untracked as f}
            <li class:selected={selected === f.path}>
              <button class="row" onclick={() => pick(f)}>
              <span class="status status-untracked">U</span>
              <span class="path">{f.path}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
      {#if status.data.conflicted.length > 0}
        <h3>Conflicted</h3>
        <ul>
          {#each status.data.conflicted as f}
            <li class:selected={selected === f.path}>
              <button class="row" onclick={() => pick(f)}>
              <span class="status status-conflicted">C</span>
              <span class="path">{f.path}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
      {#if isEmpty(status.data)}
        <p class="hint">Working tree is clean.</p>
      {/if}
    {/if}
  </aside>

  <section class="diff">
    {#if selected == null}
      <div class="hint">Select a file to view its diff.</div>
    {:else if diff.loading}
      <div class="hint">Loading diff…</div>
    {:else if diff.error}
      <div class="err">{String(diff.error)}</div>
    {:else}
      <DiffView payload={diff.data ?? null} />
    {/if}
  </section>
</div>

<style>
  .layout { display: flex; height: 100%; }
  .files {
    width: 320px;
    flex-shrink: 0;
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: var(--sp-2) 0;
  }
  .files h3 {
    font-size: var(--fs-xs);
    color: var(--fg-subtle);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin: var(--sp-3) var(--sp-3) var(--sp-1);
  }
  .files ul { list-style: none; margin: 0; padding: 0; }
  .files li { padding: 0; }
  .files li button.row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    text-align: left;
    padding: 4px var(--sp-3);
    cursor: pointer;
    font-size: var(--fs-xs);
    font-family: var(--font-mono);
    color: inherit;
  }
  .files li button.row:hover { background: var(--bg-elev-2); }
  .files li.selected button.row { background: var(--accent-bg-medium); color: var(--accent-fg); }
  .files .status {
    width: 16px; text-align: center; color: var(--fg-subtle);
    font-weight: 700;
  }
  .files .status-added, .files .status-untracked { color: var(--added); }
  .files .status-deleted { color: var(--removed); }
  .files .status-modified { color: var(--accent-500); }
  .files .status-conflicted { color: var(--removed); }
  .files .path { color: var(--fg); }

  .diff { flex: 1; overflow: auto; padding: var(--sp-3); }
  .hint { color: var(--fg-subtle); padding: var(--sp-3); font-size: var(--fs-sm); }
  .err { color: var(--removed); padding: var(--sp-3); font-size: var(--fs-sm); }
</style>
