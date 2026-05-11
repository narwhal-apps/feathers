<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { AppError, StatusSnapshot } from '$lib/types';

  let {
    repoId,
    status,
    onClose,
  }: {
    repoId: string;
    status: StatusSnapshot | null;
    onClose: () => void;
  } = $props();

  let message = $state('');
  let includeUntracked = $state(true);
  let keepIndex = $state(false);
  let saving = $state(false);
  let errorMsg = $state<string | null>(null);

  const stagedCount = $derived(status?.staged.length ?? 0);
  const totalChanges = $derived(
    (status?.staged.length ?? 0) +
      (status?.unstaged.length ?? 0) +
      (includeUntracked ? status?.untracked.length ?? 0 : 0),
  );
  const canSave = $derived(totalChanges > 0 && !saving);

  function formatError(err: unknown): string {
    if (typeof err === 'string') return err;
    const ae = err as AppError;
    if ('message' in ae) return ae.message;
    return String(err);
  }

  async function submit(): Promise<void> {
    if (!canSave) return;
    saving = true;
    errorMsg = null;
    try {
      await invoke('stash_create', {
        id: repoId,
        message: message.trim() === '' ? null : message.trim(),
        includeUntracked,
        keepIndex,
      });
      queryClient.invalidateMany([
        queryKeys.repoStashes(repoId),
        queryKeys.repoStatus(repoId),
        ['repo', repoId, 'diff'],
      ]);
      onClose();
    } catch (err) {
      errorMsg = formatError(err);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title="Stash changes" onClose={onClose} width="md">
  {#snippet body()}
    <form class="form" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <label class="field">
        <span class="label">Message <span class="optional">(optional)</span></span>
        <input
          class="input"
          type="text"
          autofocus
          placeholder="WIP on current branch"
          bind:value={message}
          disabled={saving}
        />
      </label>

      <label class="checkbox">
        <input
          type="checkbox"
          bind:checked={includeUntracked}
          disabled={saving}
        />
        <span>Include untracked files</span>
      </label>

      {#if stagedCount > 0}
        <label class="checkbox">
          <input
            type="checkbox"
            bind:checked={keepIndex}
            disabled={saving}
          />
          <span>Keep changes staged after stashing</span>
        </label>
      {/if}

      <div class="summary">
        Will stash {totalChanges} file{totalChanges === 1 ? '' : 's'}.
      </div>

      {#if errorMsg}<div class="err">{errorMsg}</div>{/if}
    </form>
  {/snippet}
  {#snippet foot()}
    <button class="btn ghost" onclick={onClose} disabled={saving}>Cancel</button>
    <button class="btn primary" onclick={submit} disabled={!canSave}>
      {saving ? 'Stashing…' : 'Stash'}
    </button>
  {/snippet}
</Modal>

<style>
  .form { display: flex; flex-direction: column; gap: var(--sp-3); }
  .field { display: flex; flex-direction: column; gap: 6px; }
  .label {
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
    font-weight: var(--weight-semibold);
  }
  .label .optional {
    text-transform: none;
    letter-spacing: 0;
    color: var(--fg-faint);
    font-weight: var(--weight-regular);
    margin-left: 4px;
  }
  .input {
    padding: 8px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--fs-sm);
    outline: none;
  }
  .input:focus { border-color: var(--accent-500); }
  .checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--fg);
    font-size: var(--fs-sm);
    cursor: pointer;
  }
  .summary {
    color: var(--fg-subtle);
    font-size: var(--fs-xs);
    padding-top: 4px;
  }
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
