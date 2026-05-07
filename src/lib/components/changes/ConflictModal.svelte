<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openPath } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import FileIcon from '$lib/components/file/FileIcon.svelte';
  import { queryClient } from '$lib/query/client';
  import type { OpKind, AppError } from '$lib/types';

  let {
    id,
    kind,
    conflicted,
    repoPath,
  }: {
    id: string;
    kind: OpKind;
    conflicted: string[];
    repoPath: string | null;
  } = $props();

  let busy = $state<null | 'continue' | 'abort' | string>(null);

  const KIND_LABEL: Record<Exclude<OpKind, 'clean'>, string> = {
    merge: 'Merge',
    rebase: 'Rebase',
    cherry_pick: 'Cherry-pick',
    revert: 'Revert',
    bisect: 'Bisect',
    apply_mailbox: 'Mailbox',
  };
  const label = $derived(kind === 'clean' ? '' : KIND_LABEL[kind]);

  const allResolved = $derived(conflicted.length === 0);

  function basename(p: string): string {
    const i = p.lastIndexOf('/');
    return i < 0 ? p : p.slice(i + 1);
  }
  function dirname(p: string): string {
    const i = p.lastIndexOf('/');
    return i < 0 ? '' : p.slice(0, i);
  }
  function absPath(rel: string): string | null {
    if (!repoPath) return null;
    return repoPath.endsWith('/') ? `${repoPath}${rel}` : `${repoPath}/${rel}`;
  }

  async function openOne(rel: string) {
    const abs = absPath(rel);
    if (!abs) return;
    try {
      await openPath(abs);
    } catch (err) {
      alert(`Failed to open ${rel}: ${String(err)}`);
    }
  }

  async function resolveOne(path: string) {
    if (busy) return;
    busy = `resolve:${path}`;
    try {
      await invoke('stage_files', { id, paths: [path] });
      queryClient.invalidate(['repo', id]);
    } catch (err) {
      reportError(`Failed to mark ${path} resolved`, err);
    } finally {
      busy = null;
    }
  }

  function reportError(prefix: string, err: unknown) {
    const e = err as AppError;
    if (e?.kind === 'merge_conflict') {
      alert(
        `${prefix}: ${e.paths.length} file${e.paths.length === 1 ? '' : 's'} still conflicted.\n\n` +
          e.paths.slice(0, 10).join('\n'),
      );
      return;
    }
    const msg =
      typeof e === 'object' && e !== null && 'message' in e
        ? (e as { message: string }).message
        : JSON.stringify(err);
    alert(`${prefix}: ${msg}`);
  }

  async function doContinue() {
    if (busy) return;
    busy = 'continue';
    try {
      await invoke('repo_op_continue', { id });
      queryClient.invalidate(['repo', id]);
    } catch (err) {
      reportError(`Failed to continue ${label.toLowerCase()}`, err);
    } finally {
      busy = null;
    }
  }

  async function doAbort() {
    if (busy) return;
    const ok = confirm(`Abort ${label.toLowerCase()}? Working tree will be reset.`);
    if (!ok) return;
    busy = 'abort';
    try {
      await invoke('repo_op_abort', { id });
      queryClient.invalidate(['repo', id]);
    } catch (err) {
      reportError(`Failed to abort ${label.toLowerCase()}`, err);
    } finally {
      busy = null;
    }
  }
</script>

<div class="backdrop" role="presentation">
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="conflict-title">
    <header class="head">
      <h2 id="conflict-title">Resolve conflicts before {label}</h2>
    </header>

    <div class="body">
      {#if allResolved}
        <div class="status ok">
          <span class="icon-wrap"><Icon name="Check" size={14} /></span>
          <div class="status-text">
            <strong>All conflicts resolved</strong>
            <span>Continue {label.toLowerCase()} to wrap things up.</span>
          </div>
        </div>
      {:else}
        <h3 class="files-title">
          {conflicted.length} conflicted file{conflicted.length === 1 ? '' : 's'}
        </h3>
        <ul class="files">
          {#each conflicted as path}
            {@const name = basename(path)}
            {@const dir = dirname(path)}
            <li>
              <FileIcon fileName={name} size={16} />
              <div class="file-text">
                <div class="file-name">
                  <span class="basename">{name}</span>
                  {#if dir}<span class="dir">{dir}</span>{/if}
                </div>
                <div class="file-sub">Conflicted</div>
              </div>
              <button
                class="row-btn"
                onclick={() => openOne(path)}
                disabled={!repoPath || busy !== null}
                title={repoPath ? `Open ${name} in your default editor` : 'Repo path unavailable'}
              >
                <Icon name="ExternalLink" size={12} />
                <span>Open</span>
              </button>
              <button
                class="row-btn ok"
                onclick={() => resolveOne(path)}
                disabled={busy !== null}
                title="Mark this file resolved"
              >
                <Icon name="Check" size={12} />
                <span>{busy === `resolve:${path}` ? 'Resolving…' : 'Resolved'}</span>
              </button>
            </li>
          {/each}
        </ul>
        <p class="hint">
          Open each file in your editor, fix the conflict markers (<code>{'<<<<<<<'}</code>, <code>{'======='}</code>, <code>{'>>>>>>>'}</code>), save, then mark it resolved.
        </p>
      {/if}
    </div>

    <footer class="foot">
      <button class="btn ghost" onclick={doAbort} disabled={busy !== null}>
        {busy === 'abort' ? 'Aborting…' : `Abort ${label}`}
      </button>
      <button class="btn primary" onclick={doContinue} disabled={busy !== null || !allResolved}>
        {busy === 'continue' ? 'Continuing…' : `Continue ${label}`}
      </button>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in srgb, #000 55%, transparent);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 18vh;
    z-index: 200;
  }
  .modal {
    width: min(560px, calc(100vw - 32px));
    background: var(--bg-elev-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-3);
    overflow: hidden;
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
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    position: relative; z-index: 1;
  }
  .head h2 {
    margin: 0;
    font-size: var(--fs-md);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    color: var(--fg);
  }
  .body {
    padding: 14px 16px;
    position: relative; z-index: 1;
  }
  .status {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border: 1px solid color-mix(in srgb, var(--removed) 35%, transparent);
    background: color-mix(in srgb, var(--removed) 12%, transparent);
    border-radius: var(--r-md);
  }
  .status.ok {
    border-color: color-mix(in srgb, var(--added) 40%, transparent);
    background: color-mix(in srgb, var(--added) 14%, transparent);
  }
  .icon-wrap {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: var(--r-pill);
    background: color-mix(in srgb, var(--removed) 40%, transparent);
    color: #fff;
  }
  .status.ok .icon-wrap {
    background: var(--added);
  }
  .status-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: var(--fs-sm);
    color: var(--fg);
    line-height: 1.4;
  }
  .status-text strong { font-weight: var(--weight-semibold); }
  .status-text span { color: var(--fg-muted); font-size: var(--fs-xs); }

  .files-title {
    margin: 0 0 10px;
    font-size: var(--fs-md);
    font-weight: var(--weight-semibold);
    color: var(--fg);
    letter-spacing: var(--tracking-tight);
  }
  .files {
    list-style: none;
    margin: 0 0 12px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    background: var(--bg-elev-1);
    max-height: 320px;
    overflow-y: auto;
  }
  .files li {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }
  .files li:last-child { border-bottom: none; }
  .file-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .file-name {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
    overflow: hidden;
  }
  .file-name .basename {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    color: var(--fg);
  }
  .file-name .dir {
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    color: var(--fg-subtle);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-sub {
    color: #f59e0b;
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
  }
  .row-btn {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 9px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    transition: color var(--t-fast), border-color var(--t-fast), background var(--t-fast);
  }
  .row-btn :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
  .row-btn:hover:not(:disabled) {
    color: var(--fg);
    border-color: var(--border-strong);
  }
  .row-btn:hover:not(:disabled) :global(svg) { color: var(--fg); }
  .row-btn.ok:hover:not(:disabled) {
    color: var(--added);
    background: color-mix(in srgb, var(--added) 14%, transparent);
    border-color: color-mix(in srgb, var(--added) 30%, transparent);
  }
  .row-btn.ok:hover:not(:disabled) :global(svg) { color: var(--added); }
  .row-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .hint {
    margin: 0;
    color: var(--fg-subtle);
    font-size: var(--fs-xs);
    line-height: 1.5;
  }
  .hint code {
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    padding: 1px 5px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-muted);
  }

  .foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    position: relative; z-index: 1;
  }
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
  .btn.primary {
    background: var(--accent-500);
    color: var(--accent-on);
  }
  .btn.primary:hover:not(:disabled) { background: var(--accent-400); }
  .btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.ghost {
    background: transparent;
    color: var(--fg-muted);
    border-color: var(--border);
  }
  .btn.ghost:hover:not(:disabled) { color: var(--fg); border-color: var(--border-strong); }
</style>
