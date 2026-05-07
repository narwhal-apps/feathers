<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { page } from '$app/stores';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import DiffView from '$lib/components/primitives/DiffView.svelte';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import type { StatusSnapshot, DiffPayload, FileChange, FileStatus, AppError } from '$lib/types';

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
  let busy = $state(false);
  let message = $state('');

  const diff = createQuery<DiffPayload>(
    () => selected != null && isStaged(selected)
      ? queryKeys.repoDiffIndex(id, selected)
      : queryKeys.repoDiffWorkdir(id, selected),
    () => {
      if (selected == null) return Promise.resolve({ files: [] });
      const cmd = isStaged(selected) ? 'diff_index' : 'diff_workdir';
      return invoke<DiffPayload>(cmd, { id, paths: [selected] });
    },
  );

  function isStaged(path: string): boolean {
    return status.data?.staged.some((f) => f.path === path) ?? false;
  }

  function pick(f: FileChange) {
    selected = f.path;
  }

  async function refresh() {
    queryClient.invalidate(['repo', id, 'status']);
    queryClient.invalidate(['repo', id, 'diff']);
  }

  async function withBusy<T>(fn: () => Promise<T>): Promise<T | null> {
    busy = true;
    try {
      return await fn();
    } catch (err) {
      const e = err as AppError;
      const msg = 'message' in e ? e.message : JSON.stringify(err);
      alert(`Failed: ${msg}`);
      return null;
    } finally {
      busy = false;
    }
  }

  async function stagePaths(paths: string[]) {
    if (paths.length === 0) return;
    await withBusy(async () => {
      await invoke('stage_files', { id, paths });
      await refresh();
    });
  }
  async function unstagePaths(paths: string[]) {
    if (paths.length === 0) return;
    await withBusy(async () => {
      await invoke('unstage_files', { id, paths });
      await refresh();
    });
  }

  async function commit() {
    if (!message.trim()) return;
    if ((status.data?.staged.length ?? 0) === 0) return;
    await withBusy(async () => {
      await invoke('commit_create', { id, message: message.trim() });
      message = '';
      // Status, log, branches all change after a commit.
      queryClient.invalidate(['repo', id]);
    });
  }

  const stagedCount = $derived(status.data?.staged.length ?? 0);
  const unstagedCount = $derived((status.data?.unstaged.length ?? 0) + (status.data?.untracked.length ?? 0));
</script>

<div class="layout">
  <aside class="files">
    <div class="files-scroll">
      {#if status.loading}
        <p class="hint">Loading…</p>
      {:else if status.error}
        <p class="err">Failed: {String(status.error)}</p>
      {:else if status.data}
        {#if status.data.staged.length > 0}
          <section class="group">
            <header class="group-header">
              <span class="group-label">Staged</span>
              <span class="group-count">{status.data.staged.length}</span>
              <button
                class="bulk"
                onclick={() => unstagePaths(status.data!.staged.map((f) => f.path))}
                disabled={busy}
                title="Unstage all"
              >Unstage all</button>
            </header>
            <ul>
              {#each status.data.staged as f}
                <li class:selected={selected === f.path}>
                  <button class="row" onclick={() => pick(f)}>
                    <span class="status status-{f.status}">{shortStatus(f.status)}</span>
                    <span class="path">{f.path}</span>
                  </button>
                  <button
                    class="action"
                    title="Unstage"
                    onclick={() => unstagePaths([f.path])}
                    disabled={busy}
                  >
                    <Icon name="Minus" size={12} />
                  </button>
                </li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if status.data.unstaged.length > 0}
          <section class="group">
            <header class="group-header">
              <span class="group-label">Unstaged</span>
              <span class="group-count">{status.data.unstaged.length}</span>
              <button
                class="bulk"
                onclick={() => stagePaths(status.data!.unstaged.map((f) => f.path))}
                disabled={busy}
                title="Stage all"
              >Stage all</button>
            </header>
            <ul>
              {#each status.data.unstaged as f}
                <li class:selected={selected === f.path}>
                  <button class="row" onclick={() => pick(f)}>
                    <span class="status status-{f.status}">{shortStatus(f.status)}</span>
                    <span class="path">{f.path}</span>
                  </button>
                  <button
                    class="action"
                    title="Stage"
                    onclick={() => stagePaths([f.path])}
                    disabled={busy}
                  >
                    <Icon name="Plus" size={12} />
                  </button>
                </li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if status.data.untracked.length > 0}
          <section class="group">
            <header class="group-header">
              <span class="group-label">Untracked</span>
              <span class="group-count">{status.data.untracked.length}</span>
              <button
                class="bulk"
                onclick={() => stagePaths(status.data!.untracked.map((f) => f.path))}
                disabled={busy}
                title="Stage all"
              >Stage all</button>
            </header>
            <ul>
              {#each status.data.untracked as f}
                <li class:selected={selected === f.path}>
                  <button class="row" onclick={() => pick(f)}>
                    <span class="status status-untracked">U</span>
                    <span class="path">{f.path}</span>
                  </button>
                  <button
                    class="action"
                    title="Stage"
                    onclick={() => stagePaths([f.path])}
                    disabled={busy}
                  >
                    <Icon name="Plus" size={12} />
                  </button>
                </li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if status.data.conflicted.length > 0}
          <section class="group">
            <header class="group-header">
              <span class="group-label">Conflicted</span>
              <span class="group-count">{status.data.conflicted.length}</span>
            </header>
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
          </section>
        {/if}

        {#if isEmpty(status.data)}
          <div class="empty-state">
            <Icon name="Sparkles" size={20} />
            <p>Working tree is clean.</p>
          </div>
        {/if}
      {/if}
    </div>

    <footer class="composer">
      <textarea
        class="message"
        placeholder={stagedCount > 0
          ? `Commit ${stagedCount} staged file${stagedCount === 1 ? '' : 's'}…`
          : unstagedCount > 0
            ? 'Stage files first to commit.'
            : 'Nothing to commit.'}
        bind:value={message}
        rows="3"
        disabled={busy}
        onkeydown={(e) => {
          // ⌘/Ctrl + Enter commits
          if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
            e.preventDefault();
            commit();
          }
        }}
      ></textarea>
      <div class="commit-row">
        <span class="commit-hint">
          {#if stagedCount > 0}
            <span class="dot on"></span>
            {stagedCount} staged
          {:else}
            <span class="dot"></span>
            no staged changes
          {/if}
        </span>
        <Button
          label={busy ? 'Committing…' : 'Commit'}
          variant="primary"
          size="sm"
          disabled={busy || stagedCount === 0 || !message.trim()}
          onclick={commit}
        />
      </div>
    </footer>
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
    width: 340px;
    flex-shrink: 0;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    background: var(--bg-elev-1);
  }
  .files-scroll { flex: 1; overflow-y: auto; padding: var(--sp-2) 0; }

  .group { padding: var(--sp-2) 0; }
  .group-header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 0 var(--sp-3);
    margin-bottom: var(--sp-1);
  }
  .group-label {
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    font-weight: var(--weight-semibold);
  }
  .group-count {
    color: var(--fg-muted);
    font-size: var(--fs-2xs);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .bulk {
    margin-left: auto;
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    transition: color var(--t-fast);
  }
  .bulk:hover:not(:disabled) { color: var(--accent-fg); }
  .bulk:disabled { opacity: 0.5; cursor: not-allowed; }

  .files ul { list-style: none; margin: 0; padding: 0; }
  .files li {
    position: relative;
    display: flex;
    align-items: stretch;
    padding: 0;
  }
  .files li button.row {
    flex: 1;
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    text-align: left;
    padding: 5px var(--sp-3);
    cursor: pointer;
    font-size: var(--fs-xs);
    font-family: var(--font-mono);
    color: inherit;
    overflow: hidden;
  }
  .files li button.row:hover { background: var(--bg-elev-2); }
  .files li.selected button.row { background: var(--accent-bg-medium); color: var(--accent-fg); }

  .files li button.action {
    flex-shrink: 0;
    width: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-subtle);
    opacity: 0;
    transition: opacity var(--t-fast), color var(--t-fast), background var(--t-fast);
  }
  .files li:hover button.action { opacity: 1; }
  .files li button.action:hover:not(:disabled) {
    color: var(--accent-fg);
    background: var(--accent-bg-medium);
  }
  .files li button.action:disabled { opacity: 0.5; }

  .files .status {
    width: 16px; text-align: center; color: var(--fg-subtle);
    font-weight: 700;
    flex-shrink: 0;
  }
  .files .status-added, .files .status-untracked { color: var(--added); }
  .files .status-deleted { color: var(--removed); }
  .files .status-modified { color: var(--accent-500); }
  .files .status-conflicted { color: var(--removed); }
  .files .path {
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-8) var(--sp-3);
    color: var(--fg-subtle);
    font-size: var(--fs-sm);
  }
  .empty-state p { margin: 0; }

  .composer {
    border-top: 1px solid var(--border);
    padding: var(--sp-2) var(--sp-3) var(--sp-3);
    background: var(--bg-elev-2);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .message {
    width: 100%;
    resize: vertical;
    min-height: 60px;
    max-height: 200px;
    padding: 8px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--fs-sm);
    line-height: 1.4;
    outline: none;
    transition: border-color var(--t-fast);
  }
  .message::placeholder { color: var(--fg-subtle); }
  .message:focus { border-color: var(--accent-500); }
  .message:disabled { opacity: 0.6; }

  .commit-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
  }
  .commit-hint {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--fg-muted);
    font-size: var(--fs-xs);
    font-family: var(--font-mono);
  }
  .dot {
    width: 6px; height: 6px;
    border-radius: var(--r-pill);
    background: var(--fg-faint);
  }
  .dot.on {
    background: var(--accent-500);
    box-shadow: 0 0 8px var(--accent-bg-strong);
  }

  .diff { flex: 1; overflow: auto; padding: var(--sp-3); background: var(--bg); }
  .hint { color: var(--fg-subtle); padding: var(--sp-3); font-size: var(--fs-sm); }
  .err { color: var(--removed); padding: var(--sp-3); font-size: var(--fs-sm); }
</style>
