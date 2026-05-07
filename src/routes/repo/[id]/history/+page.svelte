<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { page } from '$app/stores';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import DiffView from '$lib/components/primitives/DiffView.svelte';
  import type { CommitPage, DiffPayload } from '$lib/types';

  const id = $derived($page.params.id ?? '');

  const log = createQuery<CommitPage>(
    () => queryKeys.repoLog(id),
    () => invoke<CommitPage>('commit_log', { id, opts: { max: 50 } }),
  );

  let selectedOid = $state<string | null>(null);

  const diff = createQuery<DiffPayload>(
    () => queryKeys.repoDiffCommit(id, selectedOid ?? ''),
    () =>
      selectedOid == null
        ? Promise.resolve({ files: [] })
        : invoke<DiffPayload>('diff_commit', { id, oid: selectedOid }),
  );

  function relTime(secs: number): string {
    const ms = Date.now() - secs * 1000;
    const m = Math.round(ms / 60000);
    if (m < 1) return 'just now';
    if (m < 60) return `${m}m`;
    const h = Math.round(m / 60);
    if (h < 24) return `${h}h`;
    const d = Math.round(h / 24);
    return `${d}d`;
  }
</script>

<div class="layout">
  <aside class="commits">
    {#if log.loading}
      <p class="hint">Loading…</p>
    {:else if log.error}
      <p class="err">{String(log.error)}</p>
    {:else if log.data}
      <ul>
        {#each log.data.commits as c}
          <li class:selected={selectedOid === c.oid}>
            <button class="row" onclick={() => (selectedOid = c.oid)}>
              <div class="row1">
                <span class="dot"></span>
                <span class="summary">{c.summary || '(no message)'}</span>
              </div>
              <div class="row2">
                <span class="who">{c.author_name}</span>
                <span class="when">· {relTime(c.author_when)}</span>
                <span class="sha">{c.short_sha}</span>
              </div>
            </button>
          </li>
        {/each}
      </ul>
      {#if log.data.commits.length === 0}
        <p class="hint">No commits yet.</p>
      {/if}
    {/if}
  </aside>

  <section class="diff">
    {#if selectedOid == null}
      <div class="hint">Select a commit to view its diff.</div>
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
  .layout {
    display: grid;
    grid-template-columns: 360px 1fr;
    align-items: start;
  }
  .commits {
    position: sticky;
    top: 33px;
    align-self: start;
    width: 360px;
    height: calc(100vh - 56px - 33px);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: var(--sp-2) 0;
    background: var(--bg-elev-1);
    z-index: 1;
  }
  .commits ul { list-style: none; margin: 0; padding: 0; }
  .commits li { padding: 0; border-bottom: 1px solid var(--border); }
  .commits li button.row {
    display: block;
    width: 100%;
    text-align: left;
    padding: var(--sp-2) var(--sp-3);
    cursor: pointer;
    color: inherit;
  }
  .commits li button.row:hover { background: var(--bg-elev-2); }
  .commits li.selected button.row { background: var(--accent-bg-medium); color: var(--accent-fg); }
  .row1 { display: flex; align-items: center; gap: var(--sp-2); }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--accent-500); flex-shrink: 0; }
  .summary { color: var(--fg); font-size: var(--fs-sm); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .row2 { display: flex; gap: var(--sp-2); padding-left: 16px; color: var(--fg-subtle); font-size: var(--fs-xs); }
  .sha { margin-left: auto; font-family: var(--font-mono); font-variant-numeric: tabular-nums; }
  .diff { padding: var(--sp-3); min-width: 0; }
  .hint { color: var(--fg-subtle); padding: var(--sp-3); font-size: var(--fs-sm); }
  .err { color: var(--removed); padding: var(--sp-3); font-size: var(--fs-sm); }
</style>
