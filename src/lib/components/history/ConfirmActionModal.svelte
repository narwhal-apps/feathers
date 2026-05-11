<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { CommitInfo, AppError } from '$lib/types';

  type Kind = 'cherrypick' | 'revert';

  let {
    repoId,
    commit,
    currentBranch,
    kind,
    onClose,
  }: {
    repoId: string;
    commit: CommitInfo;
    currentBranch: string;
    kind: Kind;
    onClose: () => void;
  } = $props();

  let working = $state(false);
  let errorMsg = $state<string | null>(null);

  function formatError(err: unknown): string {
    if (typeof err === 'string') return err;
    const ae = err as AppError;
    if ('message' in ae) return ae.message;
    return String(err);
  }

  const cmd = $derived(kind === 'cherrypick' ? 'commit_cherrypick' : 'commit_revert');
  const verb = $derived(kind === 'cherrypick' ? 'Apply' : 'Revert');
  const titleText = $derived(kind === 'cherrypick' ? 'Cherry-pick commit' : 'Revert commit');
  const lede = $derived(
    kind === 'cherrypick'
      ? `Apply this commit on top of ${currentBranch}.`
      : `Create a new commit on ${currentBranch} that undoes this one.`,
  );

  async function go(): Promise<void> {
    if (working) return;
    working = true;
    errorMsg = null;
    try {
      await invoke(cmd, { id: repoId, oid: commit.oid });
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

<Modal title={titleText} onClose={onClose} width="sm">
  {#snippet body()}
    <p class="lede">{lede}</p>
    <div class="card">
      <span class="sha">{commit.short_sha}</span>
      <span class="summary">{commit.summary}</span>
    </div>
    <p class="hint">Working tree must be clean.</p>
    {#if errorMsg}<div class="err">{errorMsg}</div>{/if}
  {/snippet}
  {#snippet foot()}
    <button class="btn ghost" onclick={onClose} disabled={working}>Cancel</button>
    <button class="btn primary" onclick={go} disabled={working}>
      {working ? `${verb}ing…` : verb}
    </button>
  {/snippet}
</Modal>

<style>
  .lede { margin: 0 0 var(--sp-2); color: var(--fg-muted); font-size: var(--fs-sm); }
  .card {
    display: flex;
    align-items: baseline;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    margin: var(--sp-2) 0;
  }
  .sha {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    color: var(--fg-subtle);
    font-size: var(--fs-xs);
  }
  .summary { color: var(--fg); font-size: var(--fs-sm); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .hint { margin: 0; color: var(--fg-subtle); font-size: var(--fs-xs); }
  .err { margin-top: var(--sp-2); color: #c00; font-size: var(--fs-xs); }
  .btn { height: 32px; padding: 0 14px; border-radius: var(--r-sm); font-size: var(--fs-sm); font-weight: var(--weight-semibold); cursor: pointer; border: 1px solid transparent; }
  .btn.primary { background: var(--accent-500); color: var(--accent-on); }
  .btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.ghost { background: transparent; color: var(--fg-muted); border-color: var(--border); }
</style>
