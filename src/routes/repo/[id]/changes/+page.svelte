<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { page } from '$app/stores';
  import { repos } from '$lib/stores/repos.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import DiffView from '$lib/components/primitives/DiffView.svelte';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import FileIcon from '$lib/components/file/FileIcon.svelte';
  import RecentCommitsStack from '$lib/components/changes/RecentCommitsStack.svelte';
  import CommitsModal from '$lib/components/changes/CommitsModal.svelte';
  import ConflictModal from '$lib/components/changes/ConflictModal.svelte';
  import EmptyDiffHints from '$lib/components/changes/EmptyDiffHints.svelte';
  import StashList from '$lib/components/changes/StashList.svelte';
  import CreateStashModal from '$lib/components/dialogs/CreateStashModal.svelte';
  import { isOpInProgress } from '$lib/types';
  import type {
    StatusSnapshot,
    DiffPayload,
    DiffFile,
    FileStatus,
    BranchInfo,
    OpState,
    AppError,
    StashEntry,
    FileChange,
  } from '$lib/types';
  import { gitUrlToWebUrl, fileUrlOnRemote } from '$lib/utils/git-url';

  const id = $derived($page.params.id ?? '');

  const status = createQuery<StatusSnapshot>(
    () => queryKeys.repoStatus(id),
    () => invoke<StatusSnapshot>('repo_status', { id }),
  );

  // Branch + remote URL for the per-file "open on remote" link.
  const branches = createQuery<BranchInfo[] | null>(
    () => queryKeys.repoBranches(id),
    () => invoke<BranchInfo[]>('branch_list', { id }),
  );
  const remoteUrl = createQuery<string | null>(
    () => queryKeys.repoRemoteUrl(id),
    () => invoke<string | null>('repo_remote_url', { id }),
  );
  const headBranch = $derived(branches.data?.find((b) => b.is_head) ?? null);
  const webBase = $derived(gitUrlToWebUrl(remoteUrl.data ?? null));

  function fileHref(file: DiffFile): string | null {
    if (!webBase || !headBranch) return null;
    // Files that don't exist on the remote at this branch:
    if (file.status === 'added' || file.status === 'untracked') return null;
    return fileUrlOnRemote(webBase, headBranch.name, file.path);
  }

  // Conflict handling.
  const opState = createQuery<OpState>(
    () => queryKeys.repoOpState(id),
    () => invoke<OpState>('repo_op_state', { id }),
  );
  const conflictedCount = $derived(status.data?.conflicted.length ?? 0);
  const activeRepoPath = $derived(repos.activeRepo?.path ?? null);
  const opInProgress = $derived(opState.data != null && isOpInProgress(opState.data.kind));

  async function openInEditor(relPath: string) {
    if (!activeRepoPath) return;
    const sep = activeRepoPath.endsWith('/') ? '' : '/';
    const abs = `${activeRepoPath}${sep}${relPath}`;
    try {
      await openPath(abs);
    } catch (err) {
      alert(`Failed to open ${relPath}: ${String(err)}`);
    }
  }

  async function markResolved(paths: string[]) {
    // Staging a conflicted file marks it resolved (the index entry collapses
    // back to a single stage and it leaves the conflicted set).
    await stagePaths(paths);
  }

  async function discardHunk(file: DiffFile, hunkIndex: number) {
    const ok = confirm(
      `Discard this hunk in ${file.path}? This cannot be undone.`,
    );
    if (!ok) return;
    await withBusy(async () => {
      await invoke('discard_hunk', { id, path: file.path, hunkIndex });
      await refresh();
    });
  }

  let selected = $state<string | null>(null);
  let busy = $state(false);
  let committing = $state(false);
  let message = $state('');
  let commitsModalOpen = $state(false);

  // Stash UI state.
  let stashModalOpen = $state(false);
  let selectedStashIndex = $state<number | null>(null);

  const stashFiles = createQuery<FileChange[]>(
    () =>
      selectedStashIndex == null
        ? ['noop']
        : queryKeys.repoStashFiles(id, selectedStashIndex),
    () =>
      selectedStashIndex == null
        ? Promise.resolve([] as FileChange[])
        : invoke<FileChange[]>('stash_show_files', { id, index: selectedStashIndex }),
  );
  const stashDiff = createQuery<string>(
    () =>
      selectedStashIndex == null || selected == null
        ? ['noop']
        : queryKeys.repoStashDiff(id, selectedStashIndex, selected),
    () =>
      selectedStashIndex == null || selected == null
        ? Promise.resolve('')
        : invoke<string>('stash_diff_file', {
            id,
            index: selectedStashIndex,
            path: selected,
          }),
  );

  function selectStash(idx: number | null): void {
    selectedStashIndex = idx;
    selected = null;
  }

  // Whether to show the stash diff or the working-tree diff in the right pane.
  const showingStash = $derived(selectedStashIndex != null);

  const diff = createQuery<DiffPayload>(
    () => queryKeys.repoDiffWorkdir(id, selected ?? ''),
    () => {
      if (selected == null) return Promise.resolve({ files: [] });
      return invoke<DiffPayload>('diff_workdir', { id, paths: [selected] });
    },
  );

  function isStaged(path: string): boolean {
    return status.data?.staged.some((f) => f.path === path) ?? false;
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
  async function discardPaths(paths: string[], label: string) {
    if (paths.length === 0) return;
    const ok = confirm(
      `Discard changes to ${label}? This cannot be undone.\n\n` +
        paths.slice(0, 8).join('\n') +
        (paths.length > 8 ? `\n…and ${paths.length - 8} more` : ''),
    );
    if (!ok) return;
    await withBusy(async () => {
      await invoke('discard_files', { id, paths });
      if (paths.includes(selected ?? '')) selected = null;
      await refresh();
    });
  }

  async function commit() {
    if (!message.trim()) return;
    if ((status.data?.staged.length ?? 0) === 0) return;
    committing = true;
    try {
      await withBusy(async () => {
        await invoke('commit_create', { id, message: message.trim() });
        message = '';
        queryClient.invalidate(['repo', id]);
      });
    } finally {
      committing = false;
    }
  }

  // ---- Unified file list -----------------------------------------------

  type ChangeRow = {
    path: string;
    status: FileStatus;
    staged: boolean;
  };

  const allChanges = $derived.by((): ChangeRow[] => {
    if (showingStash) {
      const files = stashFiles.data ?? [];
      return files
        .map<ChangeRow>((f) => ({ path: f.path, status: f.status, staged: false }))
        .sort((a, b) => a.path.localeCompare(b.path));
    }
    const s = status.data;
    if (!s) return [];
    const seen = new Set<string>();
    const rows: ChangeRow[] = [];
    for (const f of s.staged) {
      seen.add(f.path);
      rows.push({ path: f.path, status: f.status, staged: true });
    }
    for (const f of s.unstaged) {
      if (seen.has(f.path)) continue;
      seen.add(f.path);
      rows.push({ path: f.path, status: f.status, staged: false });
    }
    for (const f of s.untracked) {
      if (seen.has(f.path)) continue;
      seen.add(f.path);
      rows.push({ path: f.path, status: 'untracked', staged: false });
    }
    for (const f of s.conflicted) {
      if (seen.has(f.path)) continue;
      seen.add(f.path);
      rows.push({ path: f.path, status: 'conflicted', staged: false });
    }
    rows.sort((a, b) => a.path.localeCompare(b.path));
    return rows;
  });

  const stagedCount = $derived(status.data?.staged.length ?? 0);
  const allStaged = $derived(
    allChanges.length > 0 && allChanges.every((r) => r.staged),
  );
  const someStaged = $derived(allChanges.some((r) => r.staged) && !allStaged);

  // The header "select-all" checkbox needs the indeterminate property
  // (only settable on the DOM node, not via an HTML attribute).
  let selectAllEl = $state<HTMLInputElement | null>(null);
  $effect(() => {
    if (selectAllEl) selectAllEl.indeterminate = someStaged;
  });

  // If the selected path leaves the changes list (committed, discarded
  // externally, etc.), clear the selection so the diff pane falls back to
  // EmptyDiffHints instead of "No changes." for a phantom file.
  $effect(() => {
    if (
      selected != null &&
      status.data &&
      !allChanges.some((r) => r.path === selected)
    ) {
      selected = null;
    }
  });

  async function toggleStage(row: ChangeRow) {
    if (row.staged) await unstagePaths([row.path]);
    else await stagePaths([row.path]);
  }
  async function toggleAll() {
    // Indeterminate (some staged) and fully checked both unstage everything,
    // matching the convention that clicking the box only "checks all" when
    // it starts empty.
    const anyStaged = allChanges.some((r) => r.staged);
    if (anyStaged)
      await unstagePaths(allChanges.filter((r) => r.staged).map((r) => r.path));
    else await stagePaths(allChanges.map((r) => r.path));
  }
  async function discardAll() {
    await discardPaths(
      allChanges.map((r) => r.path),
      `${allChanges.length} file${allChanges.length === 1 ? '' : 's'}`,
    );
  }

  // ---- Path / icon helpers ---------------------------------------------

  function splitPath(p: string): { name: string; dir: string } {
    const lastSlash = p.lastIndexOf('/');
    return lastSlash < 0
      ? { name: p, dir: '' }
      : { name: p.slice(lastSlash + 1), dir: p.slice(0, lastSlash) };
  }

  type StatusTone = 'add' | 'mod' | 'del' | 'ren' | 'conflict';
  function statusMeta(status: FileStatus): {
    icon: string;
    label: string;
    tone: StatusTone;
  } {
    switch (status) {
      case 'added':
        return { icon: 'SquarePlus', label: 'Added', tone: 'add' };
      case 'untracked':
        return { icon: 'SquarePlus', label: 'New file', tone: 'add' };
      case 'deleted':
        return { icon: 'SquareMinus', label: 'Deleted', tone: 'del' };
      case 'renamed':
        return { icon: 'SquareArrowRight', label: 'Renamed', tone: 'ren' };
      case 'modified':
        return { icon: 'SquareDot', label: 'Modified', tone: 'mod' };
      case 'typechange':
        return { icon: 'RefreshCw', label: 'Type changed', tone: 'mod' };
      case 'conflicted':
        return { icon: 'AlertTriangle', label: 'Conflicted', tone: 'conflict' };
    }
  }
</script>

<div class="layout">
  <aside class="files">
    <div class="files-scroll">
      <StashList
        repoId={id}
        selectedIndex={selectedStashIndex}
        onSelect={selectStash}
        onRequestSelect={selectStash}
        disabled={opInProgress}
      />
      {#if status.data}
        {#if conflictedCount > 0 && !opInProgress}
          <aside class="conflict-banner" role="alert">
            <div class="conflict-head">
              <Icon name="AlertTriangle" size={14} />
              <span>{conflictedCount} conflicted file{conflictedCount === 1 ? '' : 's'}</span>
            </div>
            <p>Resolve each file in your editor, then mark it resolved.</p>
            <div class="conflict-actions">
              <button
                class="conflict-bulk"
                onclick={() => markResolved(status.data!.conflicted.map((f) => f.path))}
                disabled={busy}
              >Mark all resolved</button>
            </div>
          </aside>
        {/if}
        {#if allChanges.length === 0}
          <div class="empty-state">
            <Icon name="Sparkles" size={20} />
            <p>Working tree is clean.</p>
          </div>
        {:else}
          <header class="group-header">
            <input
              type="checkbox"
              class="check check-all"
              bind:this={selectAllEl}
              checked={allStaged}
              onclick={(e) => {
                e.preventDefault();
                toggleAll();
              }}
              aria-label={allStaged || someStaged ? 'Unstage all' : 'Stage all'}
              title={allStaged || someStaged ? 'Unstage all' : 'Stage all'}
            />
            <span class="group-label">Changed files</span>
            <span class="group-count">
              {#key allChanges.length}
                <span class="num">{allChanges.length}</span>
              {/key}
            </span>
            <button
              class="bulk danger"
              onclick={discardAll}
              disabled={busy}
              title="Discard all changes">Discard all</button
            >
          </header>
          <ul>
            {#each allChanges as row}
              {@const meta = statusMeta(row.status)}
              {@const parts = splitPath(row.path)}
              <li class:selected={selected === row.path}>
                <input
                  type="checkbox"
                  class="check"
                  checked={row.staged}
                  onclick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    toggleStage(row);
                  }}
                  disabled={row.status === 'conflicted'}
                  aria-label="{row.staged ? 'Unstage' : 'Stage'} {row.path}"
                />
                <button class="row" onclick={() => (selected = row.path)}>
                  <FileIcon fileName={parts.name} size={14} />
                  <span class="name">
                    <span class="basename">{parts.name}</span>
                    {#if parts.dir}
                      <span class="dir">{parts.dir}</span>
                    {/if}
                  </span>
                </button>
                {#if row.status === 'conflicted'}
                  <button
                    class="action"
                    title="Open in editor"
                    aria-label="Open {row.path} in editor"
                    onclick={() => openInEditor(row.path)}
                    disabled={busy}
                  >
                    <Icon name="ExternalLink" size={12} />
                  </button>
                  <button
                    class="action ok"
                    title="Mark resolved"
                    aria-label="Mark {row.path} resolved"
                    onclick={() => markResolved([row.path])}
                    disabled={busy}
                  >
                    <Icon name="Check" size={12} />
                  </button>
                {:else}
                  <button
                    class="action danger"
                    title="Discard"
                    aria-label="Discard {row.path}"
                    onclick={() => discardPaths([row.path], row.path)}
                    disabled={busy}
                  >
                    <Icon name="Undo2" size={12} />
                  </button>
                {/if}
                <span
                  class="status-pill tone-{meta.tone}"
                  title={meta.label}
                  aria-label={meta.label}
                >
                  <Icon name={meta.icon} size={14} />
                </span>
              </li>
            {/each}
          </ul>
        {/if}
      {:else if status.error}
        <p class="err">Failed: {String(status.error)}</p>
      {:else if status.loading}
        <p class="hint">Loading…</p>
      {/if}
    </div>

    <footer class="composer">
      <div class="toolbar">
        <button
          class="stash-btn"
          onclick={() => (stashModalOpen = true)}
          disabled={
            committing ||
            busy ||
            opInProgress ||
            ((status.data?.staged.length ?? 0) +
              (status.data?.unstaged.length ?? 0) +
              (status.data?.untracked.length ?? 0)) === 0
          }
          title={opInProgress ? 'Operation in progress' : 'Stash all working-tree changes'}
        >
          <Icon name="Archive" size={12} />
          <span>Stash…</span>
        </button>
      </div>

      <RecentCommitsStack {id} onOpen={() => (commitsModalOpen = true)} />

      <textarea
        class="message"
        placeholder={stagedCount > 0
          ? `Commit ${stagedCount} staged file${stagedCount === 1 ? '' : 's'}…`
          : allChanges.length > 0
            ? 'Stage files first to commit.'
            : 'Nothing to commit.'}
        bind:value={message}
        autocomplete="off"
        autocapitalize="none"
        spellcheck="false"
        rows="3"
        disabled={committing}
        onkeydown={(e) => {
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
          label={committing ? 'Committing…' : 'Commit'}
          variant="primary"
          size="sm"
          disabled={committing || stagedCount === 0 || !message.trim()}
          onclick={commit}
        />
      </div>
    </footer>
  </aside>

  <section class="diff">
    {#if showingStash}
      {#if selected == null}
        <div class="hint">
          Select a file from this stash to see its changes. Press <kbd>Esc</kbd> to return to the working tree.
        </div>
      {:else if stashDiff.loading}
        <div class="hint">Loading diff…</div>
      {:else if stashDiff.error}
        <div class="err">{String(stashDiff.error)}</div>
      {:else if (stashDiff.data ?? '').trim() === ''}
        <div class="hint">No changes for this file in the stash.</div>
      {:else}
        <pre class="patch">{stashDiff.data}</pre>
      {/if}
    {:else if selected == null}
      <EmptyDiffHints {id} />
    {:else if diff.data}
      <DiffView payload={diff.data} {fileHref} onDiscardHunk={discardHunk} />
    {:else if diff.error}
      <div class="err">{String(diff.error)}</div>
    {:else if diff.loading}
      <div class="hint">Loading diff…</div>
    {/if}
  </section>
</div>

{#if commitsModalOpen}
  <CommitsModal {id} onClose={() => (commitsModalOpen = false)} />
{/if}

{#if stashModalOpen}
  <CreateStashModal
    repoId={id}
    status={status.data ?? null}
    onClose={() => (stashModalOpen = false)}
  />
{/if}

{#if opInProgress && opState.data}
  <ConflictModal
    {id}
    kind={opState.data.kind}
    conflicted={opState.data.conflicted}
    repoPath={activeRepoPath}
  />
{/if}

<style>
  .layout {
    display: grid;
    grid-template-columns: 340px 1fr;
    height: 100%;
    min-height: 0;
  }

  .files {
    width: 340px;
    height: 100%;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    background: var(--bg-elev-1);
    min-height: 0;
  }
  .files-scroll {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-2) 0;
  }

  .conflict-banner {
    margin: 4px 10px 8px;
    padding: 10px 12px;
    border: 1px solid color-mix(in srgb, var(--removed) 35%, transparent);
    border-radius: var(--r-md);
    background: color-mix(in srgb, var(--removed) 10%, transparent);
    color: var(--fg);
  }
  .conflict-head {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--removed);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
  }
  .conflict-head :global(svg) { color: var(--removed); flex-shrink: 0; }
  .conflict-banner p {
    margin: 4px 0 8px;
    color: var(--fg-muted);
    font-size: var(--fs-xs);
    line-height: 1.4;
  }
  .conflict-actions { display: flex; gap: 6px; }
  .conflict-bulk {
    height: 24px;
    padding: 0 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    cursor: pointer;
    transition: color var(--t-fast), border-color var(--t-fast), background var(--t-fast);
  }
  .conflict-bulk:hover:not(:disabled) {
    color: var(--added);
    background: color-mix(in srgb, var(--added) 14%, transparent);
    border-color: color-mix(in srgb, var(--added) 28%, transparent);
  }
  .conflict-bulk:disabled { opacity: 0.5; cursor: not-allowed; }

  .group-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 var(--sp-3) 0 10px;
    margin-bottom: var(--sp-2);
  }
  .check-all {
    margin-right: 2px;
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
    display: inline-block;
    overflow: hidden;
    line-height: 1;
  }
  /* Each new {#key} mount slides up from below — keeps the eye on the
     count when bulk discard / stage changes it. */
  .group-count .num {
    display: inline-block;
    animation: count-slide 220ms cubic-bezier(0.16, 1, 0.3, 1);
  }
  @keyframes count-slide {
    from { transform: translateY(60%); opacity: 0; }
    to   { transform: translateY(0);    opacity: 1; }
  }
  .bulk {
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    transition: color var(--t-fast);
  }
  .bulk:hover:not(:disabled) {
    color: var(--accent-fg);
  }
  .bulk:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .bulk.danger {
    margin-left: auto;
  }
  .bulk.danger:hover:not(:disabled) {
    color: var(--removed);
  }

  .files ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .files li {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 var(--sp-3) 0 10px;
    min-height: 28px;
  }
  .files li:hover {
    background: var(--bg-elev-2);
  }
  .files li.selected {
    background: var(--accent-bg-medium);
  }
  .files li.selected .basename {
    color: var(--accent-fg);
  }

  .check {
    flex-shrink: 0;
    margin: 0;
    width: 14px;
    height: 14px;
    cursor: pointer;
    accent-color: var(--accent-500);
  }
  .check:disabled {
    cursor: not-allowed;
    opacity: 0.4;
  }

  .files li button.row {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    text-align: left;
    padding: 4px 0;
    cursor: pointer;
    color: inherit;
    overflow: hidden;
  }
  .files li button.row :global(svg) {
    color: var(--fg-subtle);
    flex-shrink: 0;
  }
  .files li.selected button.row :global(svg) {
    color: var(--accent-fg);
  }

  .name {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 6px;
    overflow: hidden;
  }
  .basename {
    color: var(--fg);
    font-size: var(--fs-sm);
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 0;
    max-width: 60%;
  }
  .dir {
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl; /* truncate at the start, keep the leaf dir visible */
    text-align: left;
  }

  .files li button.action {
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    background: transparent;
    border: none;
    opacity: 0;
    transition:
      opacity var(--t-fast),
      color var(--t-fast),
      background var(--t-fast);
  }
  .files li:hover button.action {
    opacity: 1;
  }
  .files li button.action.danger:hover:not(:disabled) {
    color: var(--removed);
    background: color-mix(in srgb, var(--removed) 14%, transparent);
  }
  .files li button.action.ok:hover:not(:disabled) {
    color: var(--added);
    background: color-mix(in srgb, var(--added) 14%, transparent);
  }
  /* Conflicted-row actions stay visible (not hover-revealed) so they're
     impossible to miss while resolving. */
  .files li button.action[title="Open in editor"],
  .files li button.action[title="Mark resolved"] { opacity: 1; }
  .files li button.action:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .status-pill {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
  }
  .status-pill :global(svg) {
    display: block;
  }
  .tone-add :global(svg) {
    color: var(--added);
  }
  .tone-del :global(svg) {
    color: var(--removed);
  }
  .tone-mod :global(svg) {
    color: #f59e0b;
  }
  .tone-ren :global(svg) {
    color: var(--accent-500);
  }
  .tone-conflict :global(svg) {
    color: var(--removed);
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
  .empty-state p {
    margin: 0;
  }

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
  .message::placeholder {
    color: var(--fg-subtle);
  }
  .message:focus {
    border-color: var(--accent-500);
  }
  .message:disabled {
    opacity: 0.6;
  }

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
    width: 6px;
    height: 6px;
    border-radius: var(--r-pill);
    background: var(--fg-faint);
  }
  .dot.on {
    background: var(--accent-500);
    box-shadow: 0 0 8px var(--accent-bg-strong);
  }

  .diff {
    padding: var(--sp-3);
    background: var(--bg);
    min-width: 0;
    height: 100%;
    overflow-y: auto;
  }
  .hint {
    color: var(--fg-subtle);
    padding: var(--sp-3);
    font-size: var(--fs-sm);
  }
  .err {
    color: var(--removed);
    padding: var(--sp-3);
    font-size: var(--fs-sm);
  }
  .toolbar {
    display: flex;
    gap: 6px;
    margin-bottom: var(--sp-2);
  }
  .stash-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    cursor: pointer;
  }
  .stash-btn:hover:not(:disabled) {
    color: var(--fg);
    border-color: var(--border-strong);
  }
  .stash-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .stash-btn :global(svg) { color: var(--fg-subtle); }
  .patch {
    margin: 0;
    padding: var(--sp-3);
    background: var(--bg);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    line-height: 1.45;
    white-space: pre;
    overflow: auto;
  }
</style>
