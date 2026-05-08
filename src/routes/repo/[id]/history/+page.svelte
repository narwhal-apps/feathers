<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { page } from '$app/stores';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import DiffView from '$lib/components/primitives/DiffView.svelte';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { gitUrlToWebUrl, fileUrlOnRemote } from '$lib/utils/git-url';
  import { relTime } from '$lib/utils/time';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import type { CommitInfo, CommitPage, DiffFile, DiffPayload, AppError } from '$lib/types';

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
      queryClient.invalidate(['repo', id]);
      closeAmend();
    } catch (err) {
      const e = err as AppError;
      const msg =
        typeof e === 'object' && e !== null && 'message' in e
          ? (e as { message: string }).message
          : JSON.stringify(err);
      alert(`Failed to amend: ${msg}`);
    } finally {
      amending = false;
    }
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

<div class="layout">
  <aside class="commits">
    {#if log.data}
      <ul>
        {#each log.data.commits as c, idx}
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

  <section class="diff">
    {#if selectedOid == null}
      <div class="hint">Select a commit to view its diff.</div>
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
    <button
      type="button"
      class="ctx-item"
      role="menuitem"
      onclick={() => startAmend(ctxMenu!.commit)}
      disabled={!ctxMenu.isHead}
      title={ctxMenu.isHead ? '' : 'Only the most recent commit can be amended'}
    >
      <Icon name="Pencil" size={12} />
      <span>Amend commit…</span>
    </button>
  </div>
{/if}

{#if amendTarget}
  {@const target = amendTarget}
  <Modal title="Amend commit" onClose={closeAmend} width="md">
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

    {#snippet foot()}
      <button type="button" class="btn ghost" onclick={closeAmend} disabled={amending}>Cancel</button>
      <button
        type="button"
        class="btn primary"
        onclick={submitAmend}
        disabled={amending || !amendMessage.trim() || amendMessage.trim() === target.summary}
      >{amending ? 'Amending…' : 'Amend'}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .layout {
    display: grid;
    grid-template-columns: 360px 1fr;
    height: 100%;
    min-height: 0;
  }
  .commits {
    width: 360px;
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
  .btn {
    height: 32px;
    padding: 0 14px;
    border-radius: var(--r-sm);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    border: 1px solid transparent;
    transition: background var(--t-fast), color var(--t-fast), border-color var(--t-fast);
  }
  .btn.primary { background: var(--accent-500); color: var(--accent-on); }
  .btn.primary:hover:not(:disabled) { background: var(--accent-400); }
  .btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.ghost {
    background: transparent;
    color: var(--fg-muted);
    border-color: var(--border);
  }
  .btn.ghost:hover:not(:disabled) { color: var(--fg); border-color: var(--border-strong); }
</style>
