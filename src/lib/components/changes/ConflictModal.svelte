<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openPath } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import FileIcon from '$lib/components/file/FileIcon.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import { queryClient } from '$lib/query/client';
  import { isStashApply } from '$lib/types';
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

  const STRING_KIND_LABEL: Record<
    'merge' | 'rebase' | 'cherry_pick' | 'revert' | 'bisect' | 'apply_mailbox',
    string
  > = {
    merge: 'merge',
    rebase: 'rebase',
    cherry_pick: 'cherry-pick',
    revert: 'revert',
    bisect: 'bisect',
    apply_mailbox: 'mailbox',
  };

  const stashApply = $derived(isStashApply(kind) ? kind.stash_apply : null);
  const isStash = $derived(stashApply !== null);
  const isStashRecovery = $derived(stashApply !== null && !stashApply.conflicts_present);

  const label = $derived.by(() => {
    if (isStash) return 'stash apply';
    if (kind === 'clean') return '';
    return STRING_KIND_LABEL[kind as keyof typeof STRING_KIND_LABEL] ?? '';
  });

  const stashSubtitle = $derived(
    stashApply?.was_pop
      ? 'Stash will be dropped on continue.'
      : 'Stash will be kept on continue.',
  );

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
      reportError(`Failed to continue ${label}`, err);
    } finally {
      busy = null;
    }
  }

  async function doAbort() {
    if (busy) return;
    const confirmMsg = isStash
      ? 'Aborting will discard your in-progress resolution. The stash itself remains.'
      : `Abort ${label}? Working tree will be reset.`;
    const ok = confirm(confirmMsg);
    if (!ok) return;
    busy = 'abort';
    try {
      await invoke('repo_op_abort', { id });
      queryClient.invalidate(['repo', id]);
    } catch (err) {
      reportError(`Failed to abort ${label}`, err);
    } finally {
      busy = null;
    }
  }
</script>

<Modal title={isStashRecovery ? 'Finish stash apply' : `Resolve ${label} conflicts`} width="md">
  {#snippet body()}
    {#if isStashRecovery}
      <div class="status">
        <span class="icon-wrap"><Icon name="Info" size={14} /></span>
        <div class="status-text">
          <strong>A previous stash apply was interrupted</strong>
          <span>{stashSubtitle}</span>
        </div>
      </div>
    {:else if allResolved}
      <div class="status ok">
        <span class="icon-wrap"><Icon name="Check" size={14} /></span>
        <div class="status-text">
          <strong>All conflicts resolved</strong>
          <span>
            {#if isStash}
              {stashSubtitle}
            {:else}
              Continue the {label} to wrap things up.
            {/if}
          </span>
        </div>
      </div>
    {:else}
      <h3 class="files-title">
        {conflicted.length} conflicted file{conflicted.length === 1 ? '' : 's'}
      </h3>
      {#if isStash}
        <p class="hint hint-top">{stashSubtitle}</p>
      {/if}
      <ul class="files">
        {#each conflicted as path (path)}
          {@const name = basename(path)}
          {@const dir = dirname(path)}
          <li>
            <FileIcon fileName={name} size={16} />
            <div class="file-text">
              <div class="file-name">
                <span class="basename">{name}</span>
                {#if dir}<span class="dir">{dir}</span>{/if}
              </div>
              <div class="file-sub">Needs resolution</div>
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
        Open each file, fix the conflict markers (<code>{'<<<<<<<'}</code>, <code>{'======='}</code>, <code>{'>>>>>>>'}</code>), save, then mark it resolved.
      </p>
    {/if}
  {/snippet}

  {#snippet foot()}
    {#if isStashRecovery}
      <button class="btn primary" onclick={doContinue} disabled={busy !== null}>
        {busy === 'continue' ? 'Finishing…' : 'Finish stash apply'}
      </button>
    {:else}
      <button class="btn ghost" onclick={doAbort} disabled={busy !== null}>
        {busy === 'abort' ? 'Aborting…' : `Abort ${label}`}
      </button>
      <button class="btn primary" onclick={doContinue} disabled={busy !== null || !allResolved}>
        {busy === 'continue' ? 'Continuing…' : `Continue ${label}`}
      </button>
    {/if}
  {/snippet}
</Modal>

<style>
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
  .status.ok .icon-wrap { background: var(--added); }
  .status-text { display: flex; flex-direction: column; gap: 2px; font-size: var(--fs-sm); color: var(--fg); line-height: 1.4; }
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
  .file-text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .file-name { display: flex; align-items: baseline; gap: 6px; min-width: 0; overflow: hidden; }
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
    color: var(--removed);
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
  .row-btn:hover:not(:disabled) { color: var(--fg); border-color: var(--border-strong); }
  .row-btn:hover:not(:disabled) :global(svg) { color: var(--fg); }
  .row-btn.ok:hover:not(:disabled) {
    color: var(--added);
    background: color-mix(in srgb, var(--added) 14%, transparent);
    border-color: color-mix(in srgb, var(--added) 30%, transparent);
  }
  .row-btn.ok:hover:not(:disabled) :global(svg) { color: var(--added); }
  .row-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .hint { margin: 0; color: var(--fg-subtle); font-size: var(--fs-xs); line-height: 1.5; }
  .hint code {
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    padding: 1px 5px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-muted);
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
  .btn.primary { background: var(--accent-500); color: var(--accent-on); }
  .btn.primary:hover:not(:disabled) { background: var(--accent-400); }
  .btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.ghost {
    background: transparent;
    color: var(--fg-muted);
    border-color: var(--border);
  }
  .btn.ghost:hover:not(:disabled) { color: var(--fg); border-color: var(--border-strong); }

  .hint-top {
    margin-top: -4px;
    margin-bottom: 8px;
    color: var(--fg-muted);
    font-size: var(--fs-xs);
  }
  .status:not(.ok) .icon-wrap {
    background: color-mix(in srgb, var(--accent-500) 60%, transparent);
  }
</style>
