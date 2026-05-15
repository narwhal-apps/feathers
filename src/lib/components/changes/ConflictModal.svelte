<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openPath } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import FileIcon from '$lib/components/file/FileIcon.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import Banner from '$lib/components/primitives/Banner.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import { isStashApply } from '$lib/types';
  import { confirm, notify } from '$lib/utils/dialog.svelte';
  import { formatError } from '$lib/utils/error';
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

  let busy = $state<null | 'continue' | 'abort'>(null);

  /**
   * Cumulative set of every path that's been conflicted at any point during
   * this modal session. Auto-resolution drops paths from `conflicted` as
   * the user cleans them up; we keep them here so the list stays stable
   * and we can render the resolved checkmark in their old slot.
   *
   * Reset only when the modal unmounts (the parent wraps us in {#if
   * opInProgress}, so a fresh op gets a fresh tracker).
   */
  let seen = $state<string[]>([]);
  $effect(() => {
    for (const path of conflicted) {
      if (!seen.includes(path)) seen.push(path);
    }
  });

  type Row = { path: string; resolved: boolean };
  const displayed = $derived.by<Row[]>(() => {
    const current = new Set(conflicted);
    return seen
      .map((path) => ({ path, resolved: !current.has(path) }))
      // Unresolved first, then alphabetical inside each bucket.
      .sort((a, b) => {
        if (a.resolved !== b.resolved) return a.resolved ? 1 : -1;
        return a.path.localeCompare(b.path);
      });
  });
  const unresolvedCount = $derived(conflicted.length);

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

  const conflictActions = $derived(
    isStashRecovery
      ? {
          primary: {
            label: busy === 'continue' ? 'Finishing…' : 'Finish stash apply',
            onclick: doContinue,
            loading: busy === 'continue',
            disabled: busy !== null,
          },
        }
      : {
          danger: {
            label: busy === 'abort' ? 'Aborting…' : `Abort ${label}`,
            onclick: doAbort,
            loading: busy === 'abort',
            disabled: busy !== null,
          },
          primary: {
            label: busy === 'continue' ? 'Continuing…' : `Continue ${label}`,
            onclick: doContinue,
            loading: busy === 'continue',
            disabled: busy !== null || !allResolved,
          },
        },
  );

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
      notify(formatError(err), { kind: 'error', durationMs: 0 });
    }
  }

  function reportError(prefix: string, err: unknown) {
    const e = err as AppError;
    if (e?.kind === 'merge_conflict') {
      const text =
        `${prefix}: ${e.paths.length} file${e.paths.length === 1 ? '' : 's'} still conflicted.\n\n` +
        e.paths.slice(0, 10).join('\n');
      notify(text, { kind: 'error', durationMs: 0 });
      return;
    }
    notify(`${prefix}: ${formatError(err)}`, { kind: 'error', durationMs: 0 });
  }

  async function doContinue() {
    if (busy) return;
    busy = 'continue';
    try {
      await invoke('repo_op_continue', { id });
      queryClient.invalidateMany([
        queryKeys.repoStatus(id),
        queryKeys.repoOpState(id),
        queryKeys.repoBranches(id),
        ['repo', id, 'log'],
        queryKeys.repoStashes(id),
        ['repo', id, 'diff'],
      ]);
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
    const ok = await confirm({
      title: 'Abort',
      message: confirmMsg,
      confirmLabel: 'Abort',
      danger: true,
    });
    if (!ok) return;
    busy = 'abort';
    try {
      await invoke('repo_op_abort', { id });
      queryClient.invalidateMany([
        queryKeys.repoStatus(id),
        queryKeys.repoOpState(id),
        queryKeys.repoBranches(id),
        ['repo', id, 'log'],
        queryKeys.repoStashes(id),
        ['repo', id, 'diff'],
      ]);
    } catch (err) {
      reportError(`Failed to abort ${label}`, err);
    } finally {
      busy = null;
    }
  }
</script>

<Modal
  title={isStashRecovery ? 'Finish stash apply' : `Resolve ${label} conflicts`}
  width="md"
  actions={conflictActions}
>
  {#snippet body()}
    {#if isStashRecovery}
      <Banner tone="info" title="A previous stash apply was interrupted">
        {stashSubtitle}
      </Banner>
    {:else}
      {#if allResolved}
        <Banner tone="success" title="All conflicts resolved">
          {#if isStash}{stashSubtitle}{:else}Continue the {label} to wrap things up.{/if}
        </Banner>
      {/if}

      {#if displayed.length > 0}
        <h3 class="files-title">
          {unresolvedCount} conflicted file{unresolvedCount === 1 ? '' : 's'}
        </h3>
        {#if isStash && !allResolved}
          <p class="hint hint-top">{stashSubtitle}</p>
        {/if}
        <ul class="files">
          {#each displayed as item (item.path)}
            {@const name = basename(item.path)}
            {@const dir = dirname(item.path)}
            <li class:resolved={item.resolved}>
              <FileIcon fileName={name} size={16} />
              <div class="file-text">
                <div class="file-name">
                  <span class="basename">{name}</span>
                  {#if dir}<span class="dir">{dir}</span>{/if}
                </div>
                <div class="file-sub" class:resolved={item.resolved}>
                  {item.resolved ? 'No conflicts remaining' : 'Needs resolution'}
                </div>
              </div>
              <Button
                variant="secondary"
                size="sm"
                iconLeft="ExternalLink"
                label="Open"
                onclick={() => openOne(item.path)}
                disabled={!repoPath || busy !== null}
                title={repoPath ? `Open ${name} in your default editor` : 'Repo path unavailable'}
              />
              {#if item.resolved}
                <span class="check" aria-label="Resolved">
                  <Icon name="Check" size={14} />
                </span>
              {:else}
                <span class="check-spacer" aria-hidden="true"></span>
              {/if}
            </li>
          {/each}
        </ul>
        {#if !allResolved}
          <p class="hint">
            Open each file and fix the conflict markers
            (<code>{'<<<<<<<'}</code>, <code>{'======='}</code>,
            <code>{'>>>>>>>'}</code>). Saving the file marks it resolved
            automatically.
          </p>
        {/if}
      {/if}
    {/if}
  {/snippet}
</Modal>

<style>
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
  .file-sub.resolved {
    color: var(--added);
  }

  /* Resolved row: file icon and name fade so the eye lands on the
     unresolved files first. The green checkmark badge on the right
     mirrors GitHub Desktop. */
  .files li.resolved {
    opacity: 0.78;
  }
  .check {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: var(--r-pill);
    background: var(--added);
    color: var(--bg);
    flex-shrink: 0;
  }
  /* Same width as .check so the row layout is identical for both
     resolved and unresolved rows — the Open button doesn't shift. */
  .check-spacer {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
  }
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

  .hint-top {
    margin-top: -4px;
    margin-bottom: 8px;
    color: var(--fg-muted);
    font-size: var(--fs-xs);
  }
</style>
