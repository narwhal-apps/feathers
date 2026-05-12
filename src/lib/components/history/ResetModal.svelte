<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { CommitInfo, ResetMode, StatusSnapshot } from '$lib/types';
  import { formatError } from '$lib/utils/error';

  let {
    repoId,
    commit,
    status,
    onClose,
  }: {
    repoId: string;
    commit: CommitInfo;
    status: StatusSnapshot | null;
    onClose: () => void;
  } = $props();

  let mode = $state<ResetMode>('mixed');
  let typedConfirm = $state('');
  let working = $state(false);
  let errorMsg = $state<string | null>(null);

  // Files at risk if hard-resetting now (the working tree's modified set).
  const lossyCount = $derived.by(() => {
    if (!status) return 0;
    return status.staged.length + status.unstaged.length + status.untracked.length;
  });

  const canConfirm = $derived(
    mode !== 'hard' || typedConfirm.trim() === commit.short_sha,
  );

  const resetActions = $derived.by(() => {
    const btn = {
      label: working ? 'Resetting…' : 'Reset',
      onclick: go,
      loading: working,
      disabled: !canConfirm || working,
    };
    return mode === 'hard'
      ? { secondary: { label: 'Cancel', onclick: onClose, disabled: working }, danger: btn }
      : { secondary: { label: 'Cancel', onclick: onClose, disabled: working }, primary: btn };
  });

  async function go(): Promise<void> {
    if (working || !canConfirm) return;
    working = true;
    errorMsg = null;
    try {
      await invoke('commit_reset', { id: repoId, oid: commit.oid, mode });
      queryClient.invalidateMany([
        queryKeys.repoStatus(repoId),
        ['repo', repoId, 'log'],
        queryKeys.repoLogUnpushed(repoId),
        queryKeys.repoBranches(repoId),
        queryKeys.repoOpState(repoId),
        ['repo', repoId, 'diff'],
      ]);
      onClose();
    } catch (err) {
      errorMsg = formatError(err);
    } finally {
      working = false;
    }
  }
</script>

<Modal title="Reset to commit" onClose={onClose} width="sm" actions={resetActions}>
  {#snippet body()}
    <div class="card">
      <span class="sha">{commit.short_sha}</span>
      <span class="summary">{commit.summary}</span>
    </div>
    <fieldset class="modes">
      <label class="mode">
        <input type="radio" bind:group={mode} value="soft" disabled={working} />
        <div>
          <div class="mode-label">Soft</div>
          <div class="mode-desc">Move HEAD only. All changes (committed → staged) are kept.</div>
        </div>
      </label>
      <label class="mode">
        <input type="radio" bind:group={mode} value="mixed" disabled={working} />
        <div>
          <div class="mode-label">Mixed</div>
          <div class="mode-desc">Move HEAD. Committed changes become unstaged in the working tree.</div>
        </div>
      </label>
      <label class="mode">
        <input type="radio" bind:group={mode} value="hard" disabled={working} />
        <div>
          <div class="mode-label danger">Hard</div>
          <div class="mode-desc">Move HEAD. Discard all working-tree and staged changes.</div>
        </div>
      </label>
    </fieldset>

    {#if mode === 'hard'}
      <div class="confirm">
        <p class="warn">
          {#if lossyCount > 0}
            <strong>{lossyCount} working-tree file{lossyCount === 1 ? '' : 's'} will be permanently lost.</strong>
          {:else}
            No working-tree changes will be lost — but commits between HEAD and {commit.short_sha} will be unreachable.
          {/if}
        </p>
        <label class="field">
          <span class="label">Type <code>{commit.short_sha}</code> to confirm</span>
          <input
            class="input"
            type="text"
            bind:value={typedConfirm}
            disabled={working}
            placeholder={commit.short_sha}
          />
        </label>
      </div>
    {/if}

    {#if errorMsg}<div class="err">{errorMsg}</div>{/if}
  {/snippet}
</Modal>

<style>
  .card {
    display: flex;
    align-items: baseline;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
  }
  .sha { font-family: var(--font-mono); color: var(--fg-subtle); font-size: var(--fs-xs); }
  .summary { color: var(--fg); font-size: var(--fs-sm); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .modes { border: none; padding: 0; margin: var(--sp-3) 0 0; display: flex; flex-direction: column; gap: var(--sp-3); }
  .mode { display: flex; align-items: flex-start; gap: var(--sp-2); cursor: pointer; }
  .mode input { margin-top: 3px; }
  .mode-label { font-size: var(--fs-sm); font-weight: var(--weight-semibold); }
  .mode-label.danger { color: var(--removed); }
  .mode-desc { font-size: var(--fs-xs); color: var(--fg-subtle); margin-top: 2px; line-height: 1.4; }
  .confirm { margin-top: var(--sp-3); padding-top: var(--sp-3); border-top: 1px solid var(--border); }
  .warn { margin: 0 0 var(--sp-2); font-size: var(--fs-xs); color: var(--fg); }
  .field { display: flex; flex-direction: column; gap: 4px; }
  .label { font-size: var(--fs-2xs); color: var(--fg-subtle); }
  .label code { font-family: var(--font-mono); color: var(--fg); }
  .input {
    padding: 6px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    outline: none;
  }
  .input:focus { border-color: var(--accent-500); }
  .err { margin-top: var(--sp-2); color: var(--removed); font-size: var(--fs-xs); }
</style>
