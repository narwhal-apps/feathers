<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { page } from '$app/stores';
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
  import PaneResizer from '$lib/components/primitives/PaneResizer.svelte';
  import EmptyState from '$lib/components/primitives/EmptyState.svelte';
  import { loadStorageInt } from '$lib/utils/storage';
  import { gitUrlToWebUrl, fileUrlOnRemote } from '$lib/utils/git-url';
  import { relTime } from '$lib/utils/time';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import type { CommitInfo, CommitPage, DiffFile, DiffPayload } from '$lib/types';
  import { formatError } from '$lib/utils/error';
  import { notify } from '$lib/utils/dialog.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import BranchFromCommitModal from '$lib/components/history/BranchFromCommitModal.svelte';
  import ConfirmActionModal from '$lib/components/history/ConfirmActionModal.svelte';
  import ResetModal from '$lib/components/history/ResetModal.svelte';
  import { opKindLabel, type OpKind } from '$lib/types';

  const id = $derived($page.params.id ?? '');

  const log = createQuery<CommitPage>(
    () => queryKeys.repoLog(id),
    () => invoke<CommitPage>('commit_log', { id, opts: { max: 50 } }),
  );

  let selectedOid = $state<string | null>(null);

  // Branches / status / op-state come from the repo layout context — single
  // shared subscription instead of one per consumer page.
  const branches = useRepoBranches();
  const status = useRepoStatus();
  const opState = useRepoOpState();

  const currentBranch = $derived(branches.data?.find((b) => b.is_head)?.name ?? 'HEAD');
  const opKind = $derived<OpKind>(opState.data?.kind ?? 'clean');

  // Modal mounts.
  let branchTarget = $state<CommitInfo | null>(null);
  let confirmTarget = $state<{ commit: CommitInfo; kind: 'cherrypick' | 'revert' } | null>(null);
  let resetTarget = $state<CommitInfo | null>(null);

  // Inline feedback at the top of the tab.
  let actionError = $state<string | null>(null);

  function flashError(msg: string): void {
    actionError = msg;
    setTimeout(() => { if (actionError === msg) actionError = null; }, 5000);
  }

  function flashToast(msg: string): void {
    notify(msg, { kind: 'success' });
  }

  const diff = createQuery<DiffPayload>(
    () => queryKeys.repoDiffCommit(id, selectedOid ?? ''),
    () =>
      selectedOid == null
        ? Promise.resolve({ files: [] })
        : invoke<DiffPayload>('diff_commit', { id, oid: selectedOid }),
  );

  const remoteUrl = createQuery<string | null>(
    () => queryKeys.repoRemoteUrl(id),
    () => invoke<string | null>('repo_remote_url', { id }),
  );
  const webBase = $derived(gitUrlToWebUrl(remoteUrl.data ?? null));

  function fileHref(file: DiffFile): string | null {
    if (!webBase || !selectedOid) return null;
    if (file.status === 'deleted') return null;
    return fileUrlOnRemote(webBase, selectedOid, file.path);
  }

  // Right-click context menu
  let ctxMenu = $state<{ commit: CommitInfo; isHead: boolean; x: number; y: number } | null>(null);
  function openCtxMenu(e: MouseEvent, commit: CommitInfo, isHead: boolean) {
    e.preventDefault();
    e.stopPropagation();
    ctxMenu = { commit, isHead, x: e.clientX, y: e.clientY };
  }
  function closeCtxMenu() { ctxMenu = null; }

  // Amend modal
  let amendTarget = $state<CommitInfo | null>(null);
  let amendMessage = $state('');
  let amendMessageEl = $state<HTMLTextAreaElement | null>(null);
  let amending = $state(false);

  let paneWidth = $state(loadStorageInt('feathers:history-pane-w', 360, 240, 560));

  function startAmend(commit: CommitInfo) {
    closeCtxMenu();
    amendTarget = commit;
    amendMessage = commit.summary;
  }
  function closeAmend() {
    amendTarget = null;
    amendMessage = '';
  }
  async function submitAmend() {
    if (!amendTarget) return;
    const next = amendMessage.trim();
    if (!next) return;
    amending = true;
    try {
      await invoke('commit_create', {
        id,
        message: next,
        opts: { amend: true },
      });
      queryClient.invalidateMany([
        queryKeys.repoStatus(id),
        ['repo', id, 'log'],
        queryKeys.repoLogUnpushed(id),
        queryKeys.repoBranches(id),
      ]);
      closeAmend();
    } catch (err) {
      notify(`Failed to amend: ${formatError(err)}`, { kind: 'error', durationMs: 0 });
    } finally {
      amending = false;
    }
  }

  // ── Action handlers ────────────────────────────────────────────────────

  async function copySha(commit: CommitInfo): Promise<void> {
    closeCtxMenu();
    try {
      await navigator.clipboard.writeText(commit.oid);
      flashToast(`Copied ${commit.short_sha}`);
    } catch {
      try {
        const ta = document.createElement('textarea');
        ta.value = commit.oid;
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
        flashToast(`Copied ${commit.short_sha}`);
      } catch {
        flashError("Couldn't copy to clipboard");
      }
    }
  }

  function openOnGitHub(commit: CommitInfo): void {
    closeCtxMenu();
    if (!webBase) return;
    openUrl(`${webBase}/commit/${commit.oid}`).catch(() => {});
  }

  function startBranchFrom(commit: CommitInfo): void {
    closeCtxMenu();
    branchTarget = commit;
  }

  function startCherrypick(commit: CommitInfo): void {
    closeCtxMenu();
    confirmTarget = { commit, kind: 'cherrypick' };
  }

  function startRevert(commit: CommitInfo): void {
    closeCtxMenu();
    confirmTarget = { commit, kind: 'revert' };
  }

  function startReset(commit: CommitInfo): void {
    closeCtxMenu();
    resetTarget = commit;
  }

  function onDocClick(e: MouseEvent) {
    if (!ctxMenu) return;
    const t = e.target as Node;
    const cm = document.getElementById('history-ctx-menu');
    if (cm && cm.contains(t)) return;
    closeCtxMenu();
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (amendTarget) closeAmend();
      else if (ctxMenu) closeCtxMenu();
    }
  }
  $effect(() => {
    document.addEventListener('click', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('click', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  });
  $effect(() => {
    if (amendTarget && amendMessageEl) {
      amendMessageEl.focus();
      amendMessageEl.setSelectionRange(amendMessage.length, amendMessage.length);
    }
  });
</script>

<div class="layout" style="--pane-w: {paneWidth}px">
  {#if actionError}
    <div class="action-error" role="alert">
      {actionError}
      <button class="dismiss" onclick={() => (actionError = null)} aria-label="Dismiss">×</button>
    </div>
  {/if}
  <aside class="commits">
    {#if log.data}
      <ul>
        {#each log.data.commits as c, idx (c.oid)}
          <li class:selected={selectedOid === c.oid}>
            <button
              class="row"
              onclick={() => (selectedOid = c.oid)}
              oncontextmenu={(e) => openCtxMenu(e, c, idx === 0)}
            >
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
    {:else if log.error}
      <p class="err">{String(log.error)}</p>
    {:else if log.loading}
      <p class="hint">Loading…</p>
    {/if}
  </aside>

  <PaneResizer bind:width={paneWidth} min={240} max={560} onResize={(w) => localStorage.setItem('feathers:history-pane-w', String(w))} />

  <section class="diff">
    {#if selectedOid == null}
      <EmptyState
        illustration="space-cockpit"
        title="Select a commit to view its diff"
        description="Click any commit in the timeline. Right-click for actions."
      />
    {:else if diff.data}
      <DiffView payload={diff.data} {fileHref} />
    {:else if diff.error}
      <div class="err">{String(diff.error)}</div>
    {:else if diff.loading}
      <div class="hint">Loading diff…</div>
    {/if}
  </section>
</div>

{#if ctxMenu}
  <div
    id="history-ctx-menu"
    class="ctx-menu"
    role="menu"
    style="left: {ctxMenu.x}px; top: {ctxMenu.y}px;"
  >
    <!-- info group -->
    <button type="button" class="ctx-item" role="menuitem"
      onclick={() => copySha(ctxMenu!.commit)}>
      <Icon name="Copy" size={12} />
      <span>Copy SHA</span>
    </button>
    <button type="button" class="ctx-item" role="menuitem"
      onclick={() => openOnGitHub(ctxMenu!.commit)}
      disabled={!webBase}
      title={webBase ? '' : 'No GitHub remote configured'}>
      <Icon name="ExternalLink" size={12} />
      <span>Open on GitHub</span>
    </button>

    <div class="ctx-divider"></div>

    <!-- branch group -->
    <button type="button" class="ctx-item" role="menuitem"
      onclick={() => startBranchFrom(ctxMenu!.commit)}>
      <Icon name="GitBranch" size={12} />
      <span>Create branch from here…</span>
    </button>

    <div class="ctx-divider"></div>

    <!-- apply group -->
    <button type="button" class="ctx-item" role="menuitem"
      onclick={() => startCherrypick(ctxMenu!.commit)}
      disabled={opKind !== 'clean'}
      title={opKind === 'clean' ? '' : `${opKindLabel(opKind)} in progress`}>
      <Icon name="GitCommitHorizontal" size={12} />
      <span>Cherry-pick</span>
    </button>
    <button type="button" class="ctx-item" role="menuitem"
      onclick={() => startRevert(ctxMenu!.commit)}
      disabled={opKind !== 'clean'}
      title={opKind === 'clean' ? '' : `${opKindLabel(opKind)} in progress`}>
      <Icon name="Undo2" size={12} />
      <span>Revert</span>
    </button>

    <div class="ctx-divider"></div>

    <!-- rewind group -->
    <button type="button" class="ctx-item" role="menuitem"
      onclick={() => startReset(ctxMenu!.commit)}
      disabled={opKind !== 'clean' || ctxMenu.isHead}
      title={ctxMenu.isHead ? 'Already at this commit' : (opKind === 'clean' ? '' : `${opKindLabel(opKind)} in progress`)}>
      <Icon name="History" size={12} />
      <span>Reset to here…</span>
    </button>

    <div class="ctx-divider"></div>

    <!-- edit group (existing) -->
    <button type="button" class="ctx-item" role="menuitem"
      onclick={() => startAmend(ctxMenu!.commit)}
      disabled={!ctxMenu.isHead}
      title={ctxMenu.isHead ? '' : 'Only the most recent commit can be amended'}>
      <Icon name="Pencil" size={12} />
      <span>Amend commit…</span>
    </button>
  </div>
{/if}

{#if amendTarget}
  {@const target = amendTarget}
  <Modal
    title="Amend commit"
    onClose={closeAmend}
    width="md"
    actions={{
      secondary: { label: 'Cancel', onclick: closeAmend, disabled: amending },
      primary: {
        label: amending ? 'Amending…' : 'Amend',
        onclick: submitAmend,
        loading: amending,
        disabled: amending || !amendMessage.trim() || amendMessage.trim() === target.summary,
      },
    }}
  >
    {#snippet body()}
      <form class="form" onsubmit={(e) => { e.preventDefault(); submitAmend(); }}>
        <div class="meta">
          <span class="sha">{target.short_sha}</span>
          <span class="when">{relTime(target.author_when)}</span>
        </div>
        <label class="field">
          <span class="label">Message</span>
          <textarea
            class="input message"
            bind:value={amendMessage}
            bind:this={amendMessageEl}
            disabled={amending}
            rows="4"
            onkeydown={(e) => {
              if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
                e.preventDefault();
                submitAmend();
              }
            }}
          ></textarea>
        </label>
      </form>
    {/snippet}
  </Modal>
{/if}

{#if branchTarget}
  <BranchFromCommitModal
    repoId={id}
    commit={branchTarget}
    onClose={() => (branchTarget = null)}
  />
{/if}

{#if confirmTarget}
  <ConfirmActionModal
    repoId={id}
    commit={confirmTarget.commit}
    kind={confirmTarget.kind}
    {currentBranch}
    onClose={() => (confirmTarget = null)}
  />
{/if}

{#if resetTarget}
  <ResetModal
    repoId={id}
    commit={resetTarget}
    status={status.data ?? null}
    onClose={() => (resetTarget = null)}
  />
{/if}

<style>
  .layout {
    position: relative;
    display: grid;
    grid-template-columns: var(--pane-w) auto 1fr;
    height: 100%;
    min-height: 0;
  }
  .commits {
    height: 100%;
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: var(--sp-2) 0;
    background: var(--bg-elev-1);
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
  .diff { padding: var(--sp-3); min-width: 0; height: 100%; overflow-y: auto; }
  .hint { color: var(--fg-subtle); padding: var(--sp-3); font-size: var(--fs-sm); }
  .err { color: var(--removed); padding: var(--sp-3); font-size: var(--fs-sm); }

  /* Right-click context menu */
  .ctx-menu {
    position: fixed;
    min-width: 200px;
    padding: 4px;
    background: var(--bg-elev-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-3);
    z-index: 200;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .ctx-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    text-align: left;
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .ctx-item :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
  .ctx-item:hover:not(:disabled) { background: var(--bg-elev-2); color: var(--fg); }
  .ctx-item:hover:not(:disabled) :global(svg) { color: var(--fg-muted); }
  .ctx-item:disabled { opacity: 0.45; cursor: not-allowed; }

  /* Amend modal — shell provided by Modal primitive. */
  .form { display: contents; }
  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--fg-subtle);
    font-size: var(--fs-xs);
  }
  .meta .sha { margin-left: 0; font-family: var(--font-mono); font-variant-numeric: tabular-nums; }
  .field { display: flex; flex-direction: column; gap: 6px; }
  .label {
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
    font-weight: var(--weight-semibold);
  }
  .input.message {
    width: 100%;
    resize: vertical;
    min-height: 80px;
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
  .input.message:focus { border-color: var(--accent-500); }

  .ctx-divider {
    height: 1px;
    background: var(--border);
    margin: 4px 6px;
  }
  .action-error {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: 6px 12px;
    background: color-mix(in srgb, #c00 12%, var(--bg-elev-1));
    border-bottom: 1px solid color-mix(in srgb, #c00 30%, var(--border));
    color: var(--fg);
    font-size: var(--fs-xs);
  }
  .action-error .dismiss {
    background: transparent;
    border: none;
    color: var(--fg-muted);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
    padding: 0 4px;
  }
</style>
