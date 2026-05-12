<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { goto } from '$app/navigation';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import Field from '$lib/components/primitives/Field.svelte';
  import Input from '$lib/components/primitives/Input.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { CommitInfo } from '$lib/types';
  import { formatError } from '$lib/utils/error';

  let {
    repoId,
    commit,
    onClose,
  }: { repoId: string; commit: CommitInfo; onClose: () => void } = $props();

  let name = $state('');
  let saving = $state(false);
  let errorMsg = $state<string | null>(null);

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

<Modal
  title="Create branch from commit"
  onClose={onClose}
  width="sm"
  actions={{
    secondary: { label: 'Cancel', onclick: onClose, disabled: saving },
    primary: {
      label: saving ? 'Creating…' : 'Create + checkout',
      onclick: submit,
      loading: saving,
      disabled: !isValid || saving,
    },
  }}
>
  {#snippet body()}
    <form class="form" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <div class="meta">
        <span class="sha">{commit.short_sha}</span>
        <span class="summary">{commit.summary}</span>
      </div>
      <Field label="Branch name" error={errorMsg}>
        <Input
          variant="mono"
          bind:value={name}
          placeholder="feat/your-branch"
          disabled={saving}
          autofocus
        />
      </Field>
    </form>
  {/snippet}
</Modal>

<style>
  .form { display: flex; flex-direction: column; gap: var(--sp-3); }
  .meta { display: flex; align-items: baseline; gap: var(--sp-2); color: var(--fg-subtle); font-size: var(--fs-xs); }
  .sha { font-family: var(--font-mono); font-variant-numeric: tabular-nums; }
  .summary { color: var(--fg); font-size: var(--fs-sm); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
