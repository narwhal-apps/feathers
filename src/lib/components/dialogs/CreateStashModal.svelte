<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import Field from '$lib/components/primitives/Field.svelte';
  import Input from '$lib/components/primitives/Input.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { StatusSnapshot } from '$lib/types';
  import { formatError } from '$lib/utils/error';

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

<Modal
  title="Stash changes"
  onClose={onClose}
  width="md"
  actions={{
    secondary: { label: 'Cancel', onclick: onClose, disabled: saving },
    primary: {
      label: saving ? 'Stashing…' : 'Stash',
      onclick: submit,
      loading: saving,
      disabled: !canSave,
    },
  }}
>
  {#snippet body()}
    <form class="form" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <Field label="Message" optional error={errorMsg}>
        <Input
          bind:value={message}
          placeholder="WIP on current branch"
          disabled={saving}
          autofocus
        />
      </Field>

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
    </form>
  {/snippet}
</Modal>

<style>
  .form { display: flex; flex-direction: column; gap: var(--sp-3); }
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
</style>
