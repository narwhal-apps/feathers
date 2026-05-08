<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { goto } from '$app/navigation';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { portal } from '$lib/utils/portal';
  import type { AppError } from '$lib/types';

  let { onClose }: { onClose: () => void } = $props();

  let url = $state('');
  let parentDir = $state('');
  let folderName = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);
  let urlEl = $state<HTMLInputElement | null>(null);

  // Auto-derive the folder name from the URL whenever the user hasn't
  // overridden it. Tracking that with a flag keeps manual edits intact.
  let folderManuallyEdited = false;
  $effect(() => {
    if (folderManuallyEdited) return;
    folderName = deriveName(url);
  });

  function deriveName(raw: string): string {
    const trimmed = raw.trim();
    if (!trimmed) return '';
    // Strip trailing slashes and a trailing .git, then take the last segment.
    const noGit = trimmed.replace(/\.git\/?$/, '').replace(/\/+$/, '');
    // SSH form: git@host:owner/repo
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
  function onKey(e: KeyboardEvent) { if (e.key === 'Escape') close(); }
  $effect(() => {
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  });
  $effect(() => { urlEl?.focus(); });
</script>

<div
  class="backdrop"
  role="presentation"
  use:portal
  onclick={(e) => { if (e.target === e.currentTarget) close(); }}
  onkeydown={() => {}}
>
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="clone-title">
    <header class="head">
      <h2 id="clone-title">Clone repository</h2>
      <button class="close" onclick={close} aria-label="Close">
        <Icon name="X" size={14} />
      </button>
    </header>

    <form class="body" onsubmit={(e) => { e.preventDefault(); submit(); }}>
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

      <footer class="foot">
        <button type="button" class="btn ghost" onclick={close} disabled={busy}>Cancel</button>
        <button type="submit" class="btn primary" disabled={!ready}>
          {busy ? 'Cloning…' : 'Clone'}
        </button>
      </footer>
    </form>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in srgb, #000 55%, transparent);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 12vh;
    z-index: 200;
  }
  .modal {
    width: min(560px, calc(100vw - 32px));
    background: var(--bg-elev-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-3);
    overflow: hidden;
    position: relative;
  }
  .modal::before {
    content: "";
    position: absolute; inset: 0;
    background-image: var(--grain);
    opacity: 0.35;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    position: relative; z-index: 1;
  }
  .head h2 {
    margin: 0;
    font-size: var(--fs-md);
    font-weight: var(--weight-semibold);
    color: var(--fg);
    letter-spacing: var(--tracking-tight);
  }
  .close {
    width: 26px; height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .close:hover { background: var(--bg-elev-3); color: var(--fg); }

  .body {
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    position: relative; z-index: 1;
  }

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

  .foot { display: flex; justify-content: flex-end; gap: 8px; }
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
</style>
