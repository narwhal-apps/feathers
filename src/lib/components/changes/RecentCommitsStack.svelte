<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Avatar from '$lib/components/primitives/Avatar.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import { relTime } from '$lib/utils/time';
  import { confirm, notify } from '$lib/utils/dialog.svelte';
  import { formatError } from '$lib/utils/error';
  import type { CommitPage } from '$lib/types';

  let { id, onOpen }: { id: string; onOpen: () => void } = $props();

  // Backend computes "reachable from HEAD but not from any remote-tracking
  // ref" — the right definition regardless of upstream config.
  const log = createQuery<CommitPage>(
    () => queryKeys.repoLogUnpushed(id),
    () => invoke<CommitPage>('commit_log_unpushed', { id, max: 50 }),
  );

  const top = $derived(log.data?.commits[0] ?? null);
  const count = $derived(log.data?.commits.length ?? 0);

  let undoing = $state(false);

  async function undo(e: MouseEvent) {
    e.stopPropagation();
    if (!top) return;
    const ok = await confirm({
      title: 'Undo last commit',
      message:
        `Undo last commit?\n\n"${top.summary}"\n\n` +
        `The commit will be removed from history and its changes will reappear as staged.`,
      confirmLabel: 'Undo',
      danger: true,
    });
    if (!ok) return;
    undoing = true;
    try {
      await invoke('commit_undo', { id });
      queryClient.invalidateMany([
        queryKeys.repoStatus(id),
        ['repo', id, 'log'],
        queryKeys.repoLogUnpushed(id),
        queryKeys.repoBranches(id),
      ]);
    } catch (err) {
      notify(`Failed to undo: ${formatError(err)}`, { kind: 'error', durationMs: 0 });
    } finally {
      undoing = false;
    }
  }
</script>

{#if top}
  <button
    class="card"
    onclick={onOpen}
    title="Show {count} unpushed commit{count === 1 ? '' : 's'}"
    aria-label="Show {count} unpushed commit{count === 1 ? '' : 's'}"
  >
    {#if count > 1}
      <span class="count" aria-label="{count} unpushed commits">{count}</span>
    {/if}

    <header class="head">
      <span class="message">{top.summary || '(no message)'}</span>
      <span
        class="undo"
        role="button"
        tabindex="0"
        aria-label="Undo last commit"
        title="Undo last commit"
        onclick={undo}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); undo(e as unknown as MouseEvent); } }}
      >
        {undoing ? 'Undoing…' : 'Undo'}
        {#if !undoing}<Icon name="Undo2" size={12} />{/if}
      </span>
    </header>

    <div class="meta">
      <Avatar name={top.author_name} email={top.author_email} size={18} />
      <span class="who">{top.author_name}</span>
      <span class="when">{relTime(top.author_when)}</span>
    </div>
  </button>
{/if}

<style>
  .card {
    position: relative;
    display: block;
    width: 100%;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: 10px 12px 8px;
    margin: 0;
    cursor: pointer;
    text-align: left;
    color: inherit;
    transition: border-color var(--t-fast), background var(--t-fast);
  }
  .card:hover {
    border-color: var(--border-strong);
    background: var(--bg-elev-3);
  }

  /* Floating count chip at the top-left corner — slightly overlapping the border. */
  .count {
    position: absolute;
    top: -8px;
    left: -8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    background: var(--accent-500);
    color: var(--accent-on);
    border-radius: var(--r-pill);
    border: 2px solid var(--bg-elev-1);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 11px;
    font-weight: var(--weight-bold);
    line-height: 1;
    box-shadow: var(--shadow-2, 0 1px 3px rgba(0,0,0,0.3));
    pointer-events: none;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }
  .message {
    flex: 1;
    min-width: 0;
    font-family: var(--font-sans);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Inline pseudo-button — the parent `.card` is itself a button so we use
     a role="button" span here to keep markup valid. */
  .undo {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 8px;
    border-radius: var(--r-sm);
    background: var(--bg);
    border: 1px solid var(--border);
    color: var(--fg-muted);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast), border-color var(--t-fast);
  }
  .undo:hover {
    color: var(--fg);
    background: var(--bg-elev-1);
    border-color: var(--border-strong);
  }
  .undo :global(svg) { color: var(--fg-subtle); }
  .undo:hover :global(svg) { color: var(--fg); }

  .meta {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--fg-muted);
    font-size: var(--fs-xs);
  }
  .who { color: var(--fg); }
  .when { margin-left: auto; color: var(--fg-subtle); font-variant-numeric: tabular-nums; }
</style>
