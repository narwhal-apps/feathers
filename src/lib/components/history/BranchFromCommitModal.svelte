<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { goto } from '$app/navigation';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { CommitInfo, AppError } from '$lib/types';

  let {
    repoId,
    commit,
    onClose,
  }: { repoId: string; commit: CommitInfo; onClose: () => void } = $props();

  let name = $state('');
  let saving = $state(false);
  let errorMsg = $state<string | null>(null);

  function formatError(err: unknown): string {
    if (typeof err === 'string') return err;
    const ae = err as AppError;
    if ('message' in ae) return ae.message;
    return String(err);
  }

  // Validate: non-empty after trim, no spaces, no `..`, no leading `-`,
  // no trailing `/`. These are the invariants `branch_create_at` will
  // also enforce; client-side check just spares a round-trip.
  const isValid = $derived.by(() => {
    const n = name.trim();
    if (!n) return false;
    if (/\s/.test(n)) return false;
    if (n.includes('..')) return false;
    if (n.startsWith('-')) return false;
    if (n.endsWith('/')) return false;
    return true;
  });

  async function submit(): Promise<void> {
    if (!isValid || saving) return;
    saving = true;
    errorMsg = null;
    try {
      await invoke('branch_create_at', { id: repoId, name: name.trim(), oid: commit.oid });
      queryClient.invalidateMany([
        queryKeys.repoBranches(repoId),
        ['repo', repoId, 'log'],
        queryKeys.repoStatus(repoId),
        queryKeys.repoOpState(repoId),
      ]);
      onClose();
      await goto(`/repo/${repoId}/changes/`);
    } catch (err) {
      errorMsg = formatError(err);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title="Create branch from commit" onClose={onClose} width="sm">
  {#snippet body()}
    <form class="form" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <div class="meta">
        <span class="sha">{commit.short_sha}</span>
        <span class="summary">{commit.summary}</span>
      </div>
      <label class="field">
        <span class="label">Branch name</span>
        <input
          class="input"
          type="text"
          autofocus
          placeholder="feat/your-branch"
          bind:value={name}
          disabled={saving}
        />
      </label>
      {#if errorMsg}<div class="err">{errorMsg}</div>{/if}
    </form>
  {/snippet}
  {#snippet foot()}
    <button class="btn ghost" onclick={onClose} disabled={saving}>Cancel</button>
    <button class="btn primary" onclick={submit} disabled={!isValid || saving}>
      {saving ? 'Creating…' : 'Create + checkout'}
    </button>
  {/snippet}
</Modal>

<style>
  .form { display: flex; flex-direction: column; gap: var(--sp-3); }
  .meta { display: flex; align-items: baseline; gap: var(--sp-2); color: var(--fg-subtle); font-size: var(--fs-xs); }
  .sha { font-family: var(--font-mono); font-variant-numeric: tabular-nums; }
  .summary { color: var(--fg); font-size: var(--fs-sm); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .field { display: flex; flex-direction: column; gap: 6px; }
  .label {
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
    font-weight: var(--weight-semibold);
  }
  .input {
    padding: 8px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    outline: none;
  }
  .input:focus { border-color: var(--accent-500); }
  .err { color: #c00; font-size: var(--fs-xs); }
  .btn {
    height: 32px;
    padding: 0 14px;
    border-radius: var(--r-sm);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    border: 1px solid transparent;
  }
  .btn.primary { background: var(--accent-500); color: var(--accent-on); }
  .btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.ghost { background: transparent; color: var(--fg-muted); border-color: var(--border); }
</style>
