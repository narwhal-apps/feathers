<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { goto } from '$app/navigation';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import Field from '$lib/components/primitives/Field.svelte';
  import Input from '$lib/components/primitives/Input.svelte';
  import Banner from '$lib/components/primitives/Banner.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import type { AppError } from '$lib/types';

  let { onClose }: { onClose: () => void } = $props();

  let url = $state('');
  let parentDir = $state('');
  let folderName = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);

  let folderManuallyEdited = false;
  $effect(() => {
    if (folderManuallyEdited) return;
    folderName = deriveName(url);
  });

  function deriveName(raw: string): string {
    const trimmed = raw.trim();
    if (!trimmed) return '';
    const noGit = trimmed.replace(/\.git\/?$/, '').replace(/\/+$/, '');
    const sshMatch = noGit.match(/^[^@\s]+@[^:\s]+:(.+)$/);
    const path = sshMatch ? sshMatch[1] : noGit;
    const segs = path.split('/').filter(Boolean);
    return segs[segs.length - 1] ?? '';
  }

  function joinPath(parent: string, name: string): string {
    if (!parent) return name;
    return parent.endsWith('/') ? `${parent}${name}` : `${parent}/${name}`;
  }
  const dest = $derived(joinPath(parentDir, folderName));
  const ready = $derived(!busy && !!url.trim() && !!parentDir && !!folderName);

  async function pickParent() {
    const sel = await open({
      multiple: false,
      directory: true,
      title: 'Choose parent directory',
      defaultPath: parentDir || undefined,
    });
    if (typeof sel === 'string') parentDir = sel;
  }

  async function submit() {
    if (!ready) return;
    busy = true;
    error = null;
    try {
      const result = await repos.clone(url.trim(), dest);
      repos.activeRepoId = result.id;
      onClose();
      await goto(`/repo/${result.id}/changes/`);
    } catch (err) {
      const e = err as AppError;
      error =
        typeof e === 'object' && e !== null && 'message' in e
          ? (e as { message: string }).message
          : JSON.stringify(err);
    } finally {
      busy = false;
    }
  }

  function close() { if (!busy) onClose(); }
</script>

<Modal
  title="Clone repository"
  onClose={close}
  width="md"
  actions={{
    secondary: { label: 'Cancel', onclick: close, disabled: busy },
    primary: {
      label: busy ? 'Cloning…' : 'Clone',
      onclick: submit,
      loading: busy,
      disabled: !ready,
    },
  }}
>
  {#snippet body()}
    <form class="form" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <Field label="Repository URL">
        <Input
          variant="mono"
          bind:value={url}
          disabled={busy}
          placeholder="git@github.com:owner/repo.git or https://github.com/owner/repo"
          required
          autofocus
        />
      </Field>

      <Field label="Parent directory">
        <div class="picker">
          <Input
            variant="mono"
            bind:value={parentDir}
            disabled={busy}
            placeholder="/Users/you/Developer"
            required
          />
          <Button
            variant="secondary"
            size="md"
            iconOnly="FolderOpen"
            label="Choose folder"
            title="Choose folder"
            onclick={pickParent}
            disabled={busy}
          />
        </div>
      </Field>

      <Field label="Folder name">
        <Input
          variant="mono"
          bind:value={folderName}
          disabled={busy}
          oninput={() => { folderManuallyEdited = true; }}
          placeholder="repo"
          required
        />
      </Field>

      {#if dest}
        <div class="dest-preview" title={dest}>
          <Icon name="Folder" size={12} />
          <span>{dest}</span>
        </div>
      {/if}

      {#if error}
        <Banner tone="error">{error}</Banner>
      {/if}
    </form>
  {/snippet}
</Modal>

<style>
  .form { display: flex; flex-direction: column; gap: var(--sp-3); }
  .picker { display: flex; gap: 6px; align-items: stretch; }
  .picker > :global(:first-child) { flex: 1; min-width: 0; }

  .dest-preview {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    background: var(--bg-elev-1);
    border: 1px dashed var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    overflow: hidden;
  }
  .dest-preview span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .dest-preview :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
</style>
