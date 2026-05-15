<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { page } from '$app/stores';
  import { repos } from '$lib/stores/repos.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import {
    useRepoStatus,
    useRepoBranches,
    useRepoOpState,
  } from '$lib/stores/repo-context';
  import DiffView from '$lib/components/primitives/DiffView.svelte';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import Banner from '$lib/components/primitives/Banner.svelte';
  import Kbd from '$lib/components/primitives/Kbd.svelte';
  import PaneResizer from '$lib/components/primitives/PaneResizer.svelte';
  import EmptyState from '$lib/components/primitives/EmptyState.svelte';
  import { loadStorageInt } from '$lib/utils/storage';
  import FileIcon from '$lib/components/file/FileIcon.svelte';
  import RecentCommitsStack from '$lib/components/changes/RecentCommitsStack.svelte';
  import CommitsModal from '$lib/components/changes/CommitsModal.svelte';
  import ConflictModal from '$lib/components/changes/ConflictModal.svelte';
  import EmptyDiffHints from '$lib/components/changes/EmptyDiffHints.svelte';
  import StashList from '$lib/components/changes/StashList.svelte';
  import CreateStashModal from '$lib/components/dialogs/CreateStashModal.svelte';
  import { isOpInProgress } from '$lib/types';
  import type {
    DiffPayload,
    DiffFile,
    FileStatus,
    FileChange,
  } from '$lib/types';
  import { gitUrlToWebUrl, fileUrlOnRemote } from '$lib/utils/git-url';
  import { formatError } from '$lib/utils/error';
  import { confirm, notify } from '$lib/utils/dialog.svelte';
  import { SvelteSet } from 'svelte/reactivity';

  const id = $derived($page.params.id ?? '');

  // Status / branches / op-state come from the repo layout context — one
  // subscription shared across the layout + every child route.
  const status = useRepoStatus();
  const branches = useRepoBranches();
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
  const opState = useRepoOpState();
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
      notify(formatError(err), { kind: 'error', durationMs: 0 });
    }
  }

  async function markResolved(paths: string[]) {
    // Staging a conflicted file marks it resolved (the index entry collapses
    // back to a single stage and it leaves the conflicted set).
    await stagePaths(paths);
  }

  async function discardHunk(file: DiffFile, hunkIndex: number) {
    const ok = await confirm({
      title: 'Discard hunk',
      message: `Discard this hunk in ${file.path}? This cannot be undone.`,
      confirmLabel: 'Discard',
      danger: true,
    });
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

  // GitHub-Desktop-style "include this in the next commit" selection.
  // We invert the storage — track the paths the user has explicitly
  // UNCHECKED — so newly-detected files (FS event added a file mid-
  // session) are checked by default with zero extra bookkeeping.
  // Cleared after every successful commit. Pruned when files leave the
  // changes list (committed elsewhere, discarded, etc.) so it doesn't
  // grow forever.
  const excluded = new SvelteSet<string>();
  function isSelected(path: string): boolean {
    return !excluded.has(path);
  }

  // Stash UI state.
  let stashModalOpen = $state(false);
  let selectedStashIndex = $state<number | null>(null);

  let paneWidth = $state(loadStorageInt('feathers:changes-pane-w', 340, 240, 560));

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
  const stashDiff = createQuery<DiffPayload>(
    () =>
      selectedStashIndex == null || selected == null
        ? ['noop']
        : queryKeys.repoStashDiff(id, selectedStashIndex, selected),
    () =>
      selectedStashIndex == null || selected == null
        ? Promise.resolve({ files: [] } as DiffPayload)
        : invoke<DiffPayload>('stash_diff_file', {
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

  // One workdir diff for the whole repo, computed once per `repo_changed`
  // event and shared across every file in the changes list. Switching
  // between files becomes an O(1) JS lookup instead of triggering a fresh
  // `diff_workdir` IPC + libgit2 worktree walk per click — that walk
  // includes recursing untracked directories, which dominates clicks on
  // big or untracked-heavy repos. This is how GitHub Desktop stays smooth.
  const diff = createQuery<DiffPayload>(
    () => queryKeys.repoDiffWorkdir(id, null),
    () => invoke<DiffPayload>('diff_workdir', { id, paths: null }),
  );

  /** Per-file slice of the cached whole-repo diff. Recomputed cheaply on
   *  selection change; the underlying payload only refetches when the
   *  watcher fires `repo_changed`. */
  const selectedFileDiff = $derived.by<DiffPayload | null>(() => {
    if (selected == null) return null;
    if (!diff.data) return null;
    const file = diff.data.files.find((f) => f.path === selected);
    return file ? { files: [file] } : { files: [] };
  });

  async function refresh() {
    queryClient.invalidate(['repo', id, 'status']);
    queryClient.invalidate(['repo', id, 'diff']);
  }

  async function withBusy<T>(fn: () => Promise<T>): Promise<T | null> {
    busy = true;
    try {
      return await fn();
    } catch (err) {
      notify(formatError(err), { kind: 'error', durationMs: 0 });
      return null;
    } finally {
      busy = false;
    }
  }

  async function stagePaths(paths: string[]) {
    if (paths.length === 0) return;
    try {
      await invoke('stage_files', { id, paths });
      refresh();
    } catch (err) {
      notify(formatError(err), { kind: 'error', durationMs: 0 });
    }
  }
  async function unstagePaths(paths: string[]) {
    if (paths.length === 0) return;
    try {
      await invoke('unstage_files', { id, paths });
      refresh();
    } catch (err) {
      notify(formatError(err), { kind: 'error', durationMs: 0 });
    }
  }
  async function discardPaths(paths: string[], label: string) {
    if (paths.length === 0) return;
    const ok = await confirm({
      title: 'Discard changes',
      message:
        `Discard changes to ${label}? This cannot be undone.\n\n` +
        paths.slice(0, 8).join('\n') +
        (paths.length > 8 ? `\n…and ${paths.length - 8} more` : ''),
      confirmLabel: 'Discard',
      danger: true,
    });
    if (!ok) return;
    await withBusy(async () => {
      await invoke('discard_files', { id, paths });
      if (paths.includes(selected ?? '')) selected = null;
      await refresh();
    });
  }

  async function commit() {
    if (!message.trim()) return;
    // Build the path set to commit from the user's current selection.
    // Conflicted files can never be committed even if technically selected.
    const candidates = allChanges.filter(
      (r) => r.selected && r.status !== 'conflicted',
    );
    if (candidates.length === 0) return;
    const selectedPaths = candidates.map((r) => r.path);

    // Diff the selection against what's currently staged in the index so
    // we issue the smallest possible set of stage / unstage calls. Then
    // commit. The index is treated as an implementation detail — the
    // checkbox state in the FE is the source of truth.
    const stagedNow = new Set(status.data?.staged.map((f) => f.path) ?? []);
    const selectedSet = new Set(selectedPaths);
    const toStage = selectedPaths.filter((p) => !stagedNow.has(p));
    const toUnstage = [...stagedNow].filter((p) => !selectedSet.has(p));

    committing = true;
    try {
      await withBusy(async () => {
        if (toStage.length > 0) {
          await invoke('stage_files', { id, paths: toStage });
        }
        if (toUnstage.length > 0) {
          await invoke('unstage_files', { id, paths: toUnstage });
        }
        await invoke('commit_create', { id, message: message.trim() });
        message = '';
        // Selection resets so the next round of changes starts checked.
        excluded.clear();
        queryClient.invalidateMany([
          queryKeys.repoStatus(id),
          ['repo', id, 'log'],
          queryKeys.repoLogUnpushed(id),
          queryKeys.repoBranches(id),
        ]);
      });
    } finally {
      committing = false;
    }
  }

  // ---- Unified file list -----------------------------------------------

  type ChangeRow = {
    path: string;
    status: FileStatus;
    /** "Include this file in the next commit" — FE-side selection. */
    selected: boolean;
  };

  const allChanges = $derived.by((): ChangeRow[] => {
    if (showingStash) {
      const files = stashFiles.data ?? [];
      return files
        .map<ChangeRow>((f) => ({ path: f.path, status: f.status, selected: false }))
        .sort((a, b) => a.path.localeCompare(b.path));
    }
    const s = status.data;
    if (!s) return [];
    const seen = new Set<string>();
    const rows: ChangeRow[] = [];
    const push = (path: string, status: FileStatus, allowSelect: boolean) => {
      if (seen.has(path)) return;
      seen.add(path);
      rows.push({
        path,
        status,
        selected: allowSelect && isSelected(path),
      });
    };
    for (const f of s.staged) push(f.path, f.status, true);
    for (const f of s.unstaged) push(f.path, f.status, true);
    for (const f of s.untracked) push(f.path, 'untracked', true);
    // Conflicted files can't be committed until resolved — never selectable.
    for (const f of s.conflicted) push(f.path, 'conflicted', false);
    rows.sort((a, b) => a.path.localeCompare(b.path));
    return rows;
  });

  // Selectable rows = everything except conflicted entries.
  const selectableRows = $derived(allChanges.filter((r) => r.status !== 'conflicted'));
  const selectedCount = $derived(selectableRows.filter((r) => r.selected).length);
  const allSelected = $derived(
    selectableRows.length > 0 && selectableRows.every((r) => r.selected),
  );
  const someSelected = $derived(selectedCount > 0 && !allSelected);

  // Prune `excluded` of paths that no longer appear in the changes list
  // (committed elsewhere, discarded, FS-watcher dropped them) so the set
  // can't grow unbounded across a long session.
  $effect(() => {
    if (!status.data) return;
    const live = new Set(allChanges.map((r) => r.path));
    for (const path of excluded) {
      if (!live.has(path)) excluded.delete(path);
    }
  });

  // The header "select-all" checkbox needs the indeterminate property
  // (only settable on the DOM node, not via an HTML attribute).
  let selectAllEl = $state<HTMLInputElement | null>(null);
  $effect(() => {
    if (selectAllEl) selectAllEl.indeterminate = someSelected;
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

  function toggleSelect(row: ChangeRow) {
    if (row.status === 'conflicted') return;
    if (excluded.has(row.path)) excluded.delete(row.path);
    else excluded.add(row.path);
  }
  function toggleAll() {
    // Indeterminate (some selected) and fully checked both clear the
    // selection — matches the convention that clicking the box only
    // "checks all" when it starts empty.
    if (selectedCount > 0) {
      for (const r of selectableRows) excluded.add(r.path);
    } else {
      excluded.clear();
    }
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

<div class="layout" style="--pane-w: {paneWidth}px">
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
          <div class="conflict-wrap">
            <Banner
              tone="error"
              title="{conflictedCount} conflicted file{conflictedCount === 1 ? '' : 's'}"
            >
              Resolve each file in your editor, then mark it resolved.
              {#snippet actions()}
                <Button
                  variant="ghost"
                  size="sm"
                  label="Mark all resolved"
                  onclick={() => markResolved(status.data!.conflicted.map((f) => f.path))}
                  disabled={busy}
                />
              {/snippet}
            </Banner>
          </div>
        {/if}
        {#if allChanges.length === 0}
          <EmptyState
            illustration="astronaut-helmet"
            title="Working tree is clean"
            description="Nothing to stage. You're all caught up."
            size="sm"
          />
        {:else}
          <header class="group-header" class:stash-mode={showingStash}>
            {#if !showingStash}
              <input
                type="checkbox"
                class="check check-all"
                bind:this={selectAllEl}
                checked={allSelected}
                disabled={selectableRows.length === 0}
                onchange={() => toggleAll()}
                aria-label={selectedCount > 0 ? 'Deselect all' : 'Select all'}
                title={selectedCount > 0 ? 'Deselect all' : 'Select all'}
              />
            {/if}
            <span class="group-label">{showingStash ? 'Files in stash' : 'Changed files'}</span>
            <span class="group-count">
              {#key allChanges.length}
                <span class="num">{allChanges.length}</span>
              {/key}
            </span>
            {#if !showingStash}
              <span class="discard-all">
                <Button
                  variant="ghost"
                  size="sm"
                  iconLeft="Undo2"
                  label="Discard all"
                  onclick={discardAll}
                  title="Discard all changes"
                />
              </span>
            {/if}
          </header>
          <ul>
            {#each allChanges as row (row.path)}
              {@const meta = statusMeta(row.status)}
              {@const parts = splitPath(row.path)}
              <li class:selected={selected === row.path} class:stash-mode={showingStash}>
                {#if !showingStash}
                  <input
                    type="checkbox"
                    class="check"
                    checked={row.selected}
                    onclick={(e) => e.stopPropagation()}
                    onchange={() => toggleSelect(row)}
                    disabled={row.status === 'conflicted' || committing}
                    aria-label="{row.selected ? 'Deselect' : 'Select'} {row.path}"
                  />
                {/if}
                <button class="row" onclick={() => (selected = row.path)}>
                  <FileIcon fileName={parts.name} size={14} />
                  <span class="name">
                    <span class="basename">{parts.name}</span>
                    {#if parts.dir}
                      <span class="dir">{parts.dir}</span>
                    {/if}
                  </span>
                </button>
                {#if !showingStash}
                  {#if row.status === 'conflicted'}
                    <span class="row-actions always">
                      <Button
                        variant="ghost"
                        size="sm"
                        iconOnly="ExternalLink"
                        label="Open {row.path} in editor"
                        title="Open in editor"
                        onclick={() => openInEditor(row.path)}
                      />
                      <Button
                        variant="ghost"
                        size="sm"
                        iconOnly="Check"
                        label="Mark {row.path} resolved"
                        title="Mark resolved"
                        onclick={() => markResolved([row.path])}
                      />
                    </span>
                  {:else}
                    <span class="row-actions">
                      <Button
                        variant="ghost"
                        size="sm"
                        iconOnly="Undo2"
                        label="Discard {row.path}"
                        title="Discard"
                        onclick={() => discardPaths([row.path], row.path)}
                      />
                    </span>
                  {/if}
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
        <Button
          variant="ghost"
          size="sm"
          iconLeft="Archive"
          label="Stash…"
          onclick={() => (stashModalOpen = true)}
          disabled={
            opInProgress ||
            ((status.data?.staged.length ?? 0) +
              (status.data?.unstaged.length ?? 0) +
              (status.data?.untracked.length ?? 0)) === 0
          }
          title={opInProgress ? 'Operation in progress' : 'Stash all working-tree changes'}
        />
      </div>

      <RecentCommitsStack {id} onOpen={() => (commitsModalOpen = true)} />

      <textarea
        class="message"
        placeholder={selectedCount > 0
          ? `Commit ${selectedCount} file${selectedCount === 1 ? '' : 's'}…`
          : allChanges.length > 0
            ? 'Select files to commit.'
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
          {#if selectedCount > 0}
            <span class="dot on"></span>
            {selectedCount} selected
          {:else}
            <span class="dot"></span>
            no files selected
          {/if}
        </span>
        <Button
          label={committing ? 'Committing…' : 'Commit'}
          variant="primary"
          size="sm"
          disabled={committing || selectedCount === 0 || !message.trim()}
          onclick={commit}
        />
      </div>
    </footer>
  </aside>

  <PaneResizer bind:width={paneWidth} min={240} max={560} onResize={(w) => localStorage.setItem('feathers:changes-pane-w', String(w))} />

  <section class="diff">
    {#if showingStash}
      {#if selected == null}
        <div class="hint">
          Select a file from this stash to see its changes. Press <Kbd keys={['Esc']} /> to return to the working tree.
        </div>
      {:else if stashDiff.loading}
        <div class="hint">Loading diff…</div>
      {:else if stashDiff.error}
        <div class="err">{String(stashDiff.error)}</div>
      {:else if stashDiff.data && stashDiff.data.files.length === 0}
        <div class="hint">No changes for this file in the stash.</div>
      {:else if stashDiff.data}
        <DiffView payload={stashDiff.data} />
      {/if}
    {:else if selected == null}
      <EmptyDiffHints {id} />
    {:else if selectedFileDiff}
      <DiffView payload={selectedFileDiff} {fileHref} onDiscardHunk={discardHunk} />
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
    grid-template-columns: var(--pane-w) auto 1fr;
    grid-template-rows: minmax(0, 1fr);
    height: 100%;
    min-height: 0;
  }

  .files {
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
    padding-bottom: var(--sp-2);
  }

  .conflict-wrap { margin: 4px 10px 8px; }

  .group-header {
    position: sticky;
    top: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: var(--sp-2) var(--sp-3) var(--sp-2) 10px;
    background: var(--bg-elev-1);
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
  .discard-all {
    margin-left: auto;
    display: inline-flex;
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

  /* Per-row action chips — hidden until hover/focus, except .always
     (used on conflicted rows so resolution affordances are obvious). */
  .row-actions {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity var(--t-fast);
  }
  .row-actions.always { opacity: 1; }
  .files li:hover .row-actions,
  .files li:focus-within .row-actions { opacity: 1; }

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
    /* No padding — DiffView's sticky file headers need to land flush
       with the tab-nav border when scrolled. Spacing lives on DiffView's
       children. */
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
</style>
