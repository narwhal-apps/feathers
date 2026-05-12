<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { goto } from '$app/navigation';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import type { AppError } from '$lib/types';

  let { onClose }: { onClose: () => void } = $props();

  let url = $state('');
  let parentDir = $state('');
  let folderName = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);
  let urlEl = $state<HTMLInputElement | null>(null);

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
  $effect(() => { urlEl?.focus(); });
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
      <label class="field">
        <span class="label">Repository URL</span>
        <input
          class="input"
          type="text"
          bind:value={url}
          bind:this={urlEl}
          disabled={busy}
          placeholder="git@github.com:owner/repo.git or https://github.com/owner/repo"
          required
        />
      </label>

      <div class="field">
        <span class="label">Parent directory</span>
        <div class="picker">
          <input
            class="input"
            type="text"
            bind:value={parentDir}
            disabled={busy}
            placeholder="/Users/you/Developer"
            required
          />
          <button
            type="button"
            class="picker-btn"
            onclick={pickParent}
            disabled={busy}
            title="Choose folder"
          >
            <Icon name="FolderOpen" size={12} />
          </button>
        </div>
      </div>

      <label class="field">
        <span class="label">Folder name</span>
        <input
          class="input"
          type="text"
          bind:value={folderName}
          disabled={busy}
          oninput={() => { folderManuallyEdited = true; }}
          placeholder="repo"
          required
        />
      </label>

      {#if dest}
        <div class="dest-preview" title={dest}>
          <Icon name="Folder" size={12} />
          <span>{dest}</span>
        </div>
      {/if}

      {#if error}
        <div class="err">
          <Icon name="AlertTriangle" size={14} />
          <span>{error}</span>
        </div>
      {/if}
    </form>
  {/snippet}
</Modal>

<style>
  .form { display: contents; }
  .field { display: flex; flex-direction: column; gap: 6px; }
  .label {
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
    font-weight: var(--weight-semibold);
  }
  .input {
    width: 100%;
    padding: 8px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    outline: none;
    transition: border-color var(--t-fast);
  }
  .input::placeholder { color: var(--fg-subtle); }
  .input:focus { border-color: var(--accent-500); }

  .picker { display: flex; gap: 6px; }
  .picker .input { flex: 1; }
  .picker-btn {
    flex-shrink: 0;
    width: 36px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    transition: color var(--t-fast), border-color var(--t-fast), background var(--t-fast);
  }
  .picker-btn:hover:not(:disabled) {
    color: var(--accent-fg);
    border-color: var(--accent-bg-strong);
    background: var(--accent-bg-soft);
  }

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

  .err {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 10px 12px;
    background: color-mix(in srgb, var(--removed) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--removed) 35%, transparent);
    border-radius: var(--r-md);
    color: var(--fg);
    font-size: var(--fs-xs);
    line-height: 1.4;
  }
  .err :global(svg) { color: var(--removed); flex-shrink: 0; margin-top: 2px; }

</style>
