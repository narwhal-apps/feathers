<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Avatar from '$lib/components/primitives/Avatar.svelte';
  import DiffView from '$lib/components/primitives/DiffView.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { relTime } from '$lib/utils/time';
  import { gitUrlToWebUrl, fileUrlOnRemote } from '$lib/utils/git-url';
  import type { CommitPage, DiffFile, DiffPayload } from '$lib/types';

  let { id, onClose }: { id: string; onClose: () => void } = $props();

  const log = createQuery<CommitPage>(
    () => queryKeys.repoLogUnpushed(id),
    () => invoke<CommitPage>('commit_log_unpushed', { id, max: 50 }),
  );

  const commits = $derived(log.data?.commits ?? []);

  let selectedOid = $state<string | null>(null);

  // Default-select the latest unpushed commit when the list resolves; if the
  // current selection scrolled out of the unpushed window (e.g. after a push),
  // reset to the new top.
  $effect(() => {
    if (commits.length === 0) {
      selectedOid = null;
      return;
    }
    if (!selectedOid || !commits.some((c) => c.oid === selectedOid)) {
      selectedOid = commits[0].oid;
    }
  });

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

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }

  $effect(() => {
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  });
</script>

<div
  class="backdrop"
  role="presentation"
  onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
  onkeydown={() => {}}
>
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="commits-title">
    <header class="head">
      <h2 id="commits-title">
        Unpushed commits
        {#if commits.length > 0}<span class="count">{commits.length}</span>{/if}
      </h2>
      <button class="close" onclick={onClose} aria-label="Close">
        <Icon name="X" size={14} />
      </button>
    </header>

    <div class="body">
      <aside class="commits">
        {#if log.data}
          {#if commits.length === 0}
            <p class="hint">No unpushed commits.</p>
          {:else}
            <ul>
              {#each commits as c}
                <li class:selected={selectedOid === c.oid}>
                  <button class="row" onclick={() => (selectedOid = c.oid)}>
                    <Avatar name={c.author_name} email={c.author_email} size={20} />
                    <div class="row-text">
                      <div class="row-1">
                        <span class="message">{c.summary || '(no message)'}</span>
                        <span class="sha">{c.short_sha}</span>
                      </div>
                      <div class="row-2">
                        <span class="who">{c.author_name}</span>
                        <span class="when">· {relTime(c.author_when)}</span>
                      </div>
                    </div>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        {:else if log.error}
          <p class="err">{String(log.error)}</p>
        {:else}
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
        {:else}
          <div class="hint">Loading diff…</div>
        {/if}
      </section>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in srgb, #000 55%, transparent);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 5vh 4vw;
  }
  .modal {
    width: min(1100px, 100%);
    height: 100%;
    background: var(--bg-elev-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-3);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    position: relative;
  }
  .modal::before {
    content: "";
    position: absolute; inset: 0;
    background-image: var(--grain);
    opacity: 0.35;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    position: relative; z-index: 1;
    flex-shrink: 0;
  }
  .head h2 {
    margin: 0;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: var(--fs-md);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    color: var(--fg);
  }
  .count {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    border-radius: var(--r-pill);
    padding: 1px 8px;
    font-size: var(--fs-2xs);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-bold);
  }
  .close {
    width: 26px; height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .close:hover { background: var(--bg-elev-3); color: var(--fg); }

  .body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 360px 1fr;
    position: relative; z-index: 1;
  }
  .commits {
    border-right: 1px solid var(--border);
    overflow-y: auto;
    background: var(--bg-elev-1);
  }
  .commits ul { list-style: none; margin: 0; padding: 6px 0; }
  .commits li { padding: 0; }
  .commits li.selected { background: var(--accent-bg-medium); }
  .commits li.selected .message { color: var(--accent-fg); }
  .commits li button.row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
    color: inherit;
  }
  .commits li button.row:hover { background: var(--bg-elev-2); }
  .commits li.selected button.row:hover { background: var(--accent-bg-medium); }
  .row-text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .row-1 { display: flex; align-items: baseline; gap: 8px; min-width: 0; }
  .message {
    flex: 1;
    min-width: 0;
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sha {
    color: var(--fg-subtle);
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }
  .row-2 {
    display: flex;
    gap: 4px;
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
  }
  .who { color: var(--fg-muted); }

  .diff {
    overflow-y: auto;
    padding: var(--sp-3);
    background: var(--bg);
    min-width: 0;
  }
  .hint { color: var(--fg-subtle); padding: var(--sp-3); font-size: var(--fs-sm); }
  .err { color: var(--removed); padding: var(--sp-3); font-size: var(--fs-sm); }
</style>
