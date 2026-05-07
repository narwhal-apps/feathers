<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { BranchInfo, AppError } from '$lib/types';

  const active = $derived(repos.activeRepo);

  const branches = createQuery<BranchInfo[] | null>(
    () => active ? queryKeys.repoBranches(active.id) : ['noop'],
    () => active ? invoke<BranchInfo[]>('branch_list', { id: active.id }) : Promise.resolve(null),
  );

  const head = $derived(branches.data?.find((b) => b.is_head) ?? null);
  const localBranches = $derived(
    (branches.data ?? []).filter((b) => !b.is_remote),
  );
  // The repo's default branch — prefer "main" then "master".
  const defaultBranch = $derived(
    localBranches.find((b) => b.name === 'main')
      ?? localBranches.find((b) => b.name === 'master')
      ?? null,
  );
  // Only offer the "default" choice when it's distinct from the current branch.
  const showDefaultOption = $derived(
    !!defaultBranch && !!head && defaultBranch.name !== head.name,
  );

  let open = $state(false);
  let busy = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let filterEl = $state<HTMLInputElement | null>(null);
  let filter = $state('');

  // New-branch modal state.
  let modalOpen = $state(false);
  let modalNameEl = $state<HTMLInputElement | null>(null);
  let newName = $state('');
  type FromKind = 'current' | 'default';
  let fromKind = $state<FromKind>('current');

  // Right-click context menu state.
  let ctxMenu = $state<{ branch: BranchInfo; x: number; y: number } | null>(null);

  // Rename modal state.
  let renameTarget = $state<BranchInfo | null>(null);
  let renameName = $state('');
  let renameNameEl = $state<HTMLInputElement | null>(null);

  function isDefaultBranch(b: BranchInfo): boolean {
    return !!defaultBranch && b.name === defaultBranch.name;
  }

  const filtered = $derived(
    filter.trim() === ''
      ? localBranches
      : localBranches.filter((b) => b.name.toLowerCase().includes(filter.toLowerCase())),
  );

  function close() {
    open = false;
    filter = '';
  }
  function closeModal() {
    modalOpen = false;
    newName = '';
    fromKind = 'current';
  }
  function openModal() {
    close();
    modalOpen = true;
    fromKind = 'current';
  }

  function reportError(prefix: string, err: unknown) {
    const e = err as AppError;
    const msg =
      typeof e === 'object' && e !== null && 'message' in e
        ? (e as { message: string }).message
        : JSON.stringify(err);
    alert(`${prefix}: ${msg}`);
  }

  async function pick(b: BranchInfo) {
    if (!active) return;
    if (b.is_head) { close(); return; }
    busy = true;
    try {
      await invoke('branch_checkout', { id: active.id, name: b.name });
      // Branch + status + log + workdir diff all change after checkout.
      queryClient.invalidate(['repo', active.id]);
      close();
    } catch (err) {
      const e = err as AppError;
      if (e.kind === 'dirty') {
        alert(
          `Cannot switch branches — working tree has uncommitted changes:\n\n` +
            e.paths.slice(0, 10).join('\n') +
            (e.paths.length > 10 ? `\n…and ${e.paths.length - 10} more` : '') +
            `\n\nCommit, stash, or discard them first.`,
        );
      } else if (e.kind === 'git') {
        alert(`Failed to switch: ${e.message}`);
      } else {
        alert(`Failed to switch: ${JSON.stringify(err)}`);
      }
    } finally {
      busy = false;
    }
  }

  async function createBranch() {
    if (!active || !head) return;
    const name = newName.trim();
    if (!name) return;
    const fromName = fromKind === 'default' && defaultBranch
      ? defaultBranch.name
      : head.name;
    busy = true;
    try {
      await invoke('branch_create', {
        id: active.id,
        name,
        from: fromName,
        checkout: true,
      });
      queryClient.invalidate(['repo', active.id]);
      closeModal();
    } catch (err) {
      reportError('Failed to create branch', err);
    } finally {
      busy = false;
    }
  }

  function openCtxMenu(b: BranchInfo, e: MouseEvent) {
    if (isDefaultBranch(b)) return;
    e.preventDefault();
    e.stopPropagation();
    ctxMenu = { branch: b, x: e.clientX, y: e.clientY };
  }
  function closeCtxMenu() {
    ctxMenu = null;
  }

  function startRename(b: BranchInfo) {
    closeCtxMenu();
    close();
    renameTarget = b;
    renameName = b.name;
  }
  function closeRename() {
    renameTarget = null;
    renameName = '';
  }
  async function submitRename() {
    if (!active || !renameTarget) return;
    const next = renameName.trim();
    if (!next || next === renameTarget.name) { closeRename(); return; }
    busy = true;
    try {
      await invoke('branch_rename', {
        id: active.id,
        oldName: renameTarget.name,
        newName: next,
      });
      queryClient.invalidate(['repo', active.id]);
      closeRename();
    } catch (err) {
      reportError('Failed to rename branch', err);
    } finally {
      busy = false;
    }
  }

  async function confirmDelete(b: BranchInfo) {
    closeCtxMenu();
    if (!confirm(`Delete branch "${b.name}"?`)) return;
    await runDelete(b.name, false);
  }
  async function runDelete(name: string, force: boolean) {
    if (!active) return;
    busy = true;
    try {
      await invoke('branch_delete', { id: active.id, name, force });
      queryClient.invalidate(queryKeys.repoBranches(active.id));
    } catch (err) {
      const e = err as AppError;
      if (e.kind === 'unmerged') {
        busy = false;
        const ok = confirm(
          `Branch "${name}" has commits that aren't merged into HEAD.\n\n` +
            `Force delete and lose those commits?`,
        );
        if (ok) await runDelete(name, true);
        return;
      }
      reportError('Failed to delete branch', err);
    } finally {
      busy = false;
    }
  }

  function onDocClick(e: MouseEvent) {
    const t = e.target as Node;
    if (ctxMenu) {
      const cm = document.getElementById('branch-ctx-menu');
      if (!cm || !cm.contains(t)) closeCtxMenu();
    }
    if (!open) return;
    if (triggerEl && (triggerEl === t || triggerEl.contains(t))) return;
    const menu = document.getElementById('branch-switcher-menu');
    if (menu && menu.contains(t)) return;
    close();
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (renameTarget) closeRename();
      else if (ctxMenu) closeCtxMenu();
      else if (modalOpen) closeModal();
      else if (open) close();
    }
  }

  $effect(() => {
    document.addEventListener('click', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('click', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  });

  $effect(() => {
    if (open && filterEl) filterEl.focus();
  });

  $effect(() => {
    if (modalOpen && modalNameEl) modalNameEl.focus();
  });

  $effect(() => {
    if (renameTarget && renameNameEl) {
      renameNameEl.focus();
      renameNameEl.select();
    }
  });
</script>

{#if active && head}
  <div class="wrap">
    <button
      class="trigger"
      bind:this={triggerEl}
      onclick={() => (open = !open)}
      disabled={busy}
      aria-haspopup="menu"
      aria-expanded={open}
      title="Switch branch"
    >
      <Icon name="GitBranch" size={12} />
      <span class="name">{head.name}</span>
      <span class="total" title="{localBranches.length} local branches">{localBranches.length}</span>
      {#if head.ahead || head.behind}
        <span class="counts">↓ {head.behind ?? 0}  ↑ {head.ahead ?? 0}</span>
      {/if}
      <Icon name="ChevronDown" size={12} />
    </button>

    {#if open}
      <div id="branch-switcher-menu" class="menu" role="menu">
        <div class="filter-wrap">
          <input
            class="filter"
            type="text"
            placeholder="Filter branches…"
            bind:value={filter}
            bind:this={filterEl}
          />
        </div>
        <div class="list">
          <ul>
            {#each filtered as b}
              <li>
                <button
                  class="item"
                  class:current={b.is_head}
                  role="menuitem"
                  onclick={() => pick(b)}
                  oncontextmenu={(e) => openCtxMenu(b, e)}
                  disabled={busy}
                >
                  <Icon name="GitBranch" size={12} />
                  <span class="item-name">{b.name}</span>
                  {#if isDefaultBranch(b)}
                    <span class="default-tag">default</span>
                  {/if}
                  {#if b.is_head}
                    <Icon name="Check" size={14} />
                  {/if}
                </button>
              </li>
            {:else}
              <li class="empty">No matching branches.</li>
            {/each}
          </ul>
        </div>

        <div class="footer">
          <button
            class="new"
            onclick={openModal}
            disabled={busy}
          >
            <Icon name="Plus" size={12} />
            <span>New branch…</span>
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}

{#if modalOpen && head}
  <div
    class="modal-backdrop"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
    onkeydown={() => {}}
  >
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="new-branch-title">
      <header class="modal-header">
        <h2 id="new-branch-title">New branch</h2>
        <button class="modal-close" onclick={closeModal} aria-label="Close">
          <Icon name="X" size={14} />
        </button>
      </header>

      <form
        class="modal-body"
        onsubmit={(e) => { e.preventDefault(); createBranch(); }}
      >
        <label class="field">
          <span class="label">Name</span>
          <input
            class="input"
            type="text"
            placeholder="feature/short-description"
            bind:value={newName}
            bind:this={modalNameEl}
            disabled={busy}
          />
        </label>

        <div class="field">
          <span class="label">Branch from</span>
          {#if showDefaultOption}
            <div class="seg" role="radiogroup" aria-label="Branch from">
              <button
                type="button"
                class="seg-btn"
                class:on={fromKind === 'current'}
                role="radio"
                aria-checked={fromKind === 'current'}
                onclick={() => (fromKind = 'current')}
                disabled={busy}
              >
                <Icon name="GitBranch" size={12} />
                <span class="seg-name">{head.name}</span>
                <span class="seg-tag">current</span>
              </button>
              <button
                type="button"
                class="seg-btn"
                class:on={fromKind === 'default'}
                role="radio"
                aria-checked={fromKind === 'default'}
                onclick={() => (fromKind = 'default')}
                disabled={busy}
              >
                <Icon name="GitBranch" size={12} />
                <span class="seg-name">{defaultBranch?.name}</span>
                <span class="seg-tag">default</span>
              </button>
            </div>
          {:else}
            <div class="seg-static">
              <Icon name="GitBranch" size={12} />
              <span class="seg-name">{head.name}</span>
              <span class="seg-tag">{defaultBranch?.name === head.name ? 'current · default' : 'current'}</span>
            </div>
          {/if}
        </div>

        <footer class="modal-footer">
          <button
            type="button"
            class="btn ghost"
            onclick={closeModal}
            disabled={busy}
          >Cancel</button>
          <button
            type="submit"
            class="btn primary"
            disabled={busy || !newName.trim()}
          >{busy ? 'Creating…' : 'Create branch'}</button>
        </footer>
      </form>
    </div>
  </div>
{/if}

{#if ctxMenu}
  <div
    id="branch-ctx-menu"
    class="ctx-menu"
    role="menu"
    style="left: {ctxMenu.x}px; top: {ctxMenu.y}px;"
  >
    <button
      type="button"
      class="ctx-item"
      role="menuitem"
      onclick={() => startRename(ctxMenu!.branch)}
      disabled={busy}
    >
      <Icon name="Pencil" size={12} />
      <span>Rename branch…</span>
    </button>
    <button
      type="button"
      class="ctx-item danger"
      role="menuitem"
      onclick={() => confirmDelete(ctxMenu!.branch)}
      disabled={busy || ctxMenu.branch.is_head}
      title={ctxMenu.branch.is_head ? 'Cannot delete the current branch' : ''}
    >
      <Icon name="Trash2" size={12} />
      <span>Delete branch</span>
    </button>
  </div>
{/if}

{#if renameTarget}
  <div
    class="modal-backdrop"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) closeRename(); }}
    onkeydown={() => {}}
  >
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="rename-branch-title">
      <header class="modal-header">
        <h2 id="rename-branch-title">Rename branch</h2>
        <button class="modal-close" onclick={closeRename} aria-label="Close">
          <Icon name="X" size={14} />
        </button>
      </header>

      <form
        class="modal-body"
        onsubmit={(e) => { e.preventDefault(); submitRename(); }}
      >
        <label class="field">
          <span class="label">Rename "{renameTarget.name}" to</span>
          <input
            class="input"
            type="text"
            bind:value={renameName}
            bind:this={renameNameEl}
            disabled={busy}
          />
        </label>

        <footer class="modal-footer">
          <button
            type="button"
            class="btn ghost"
            onclick={closeRename}
            disabled={busy}
          >Cancel</button>
          <button
            type="submit"
            class="btn primary"
            disabled={busy || !renameName.trim() || renameName.trim() === renameTarget.name}
          >{busy ? 'Renaming…' : 'Rename'}</button>
        </footer>
      </form>
    </div>
  </div>
{/if}

<style>
  .wrap { position: relative; }

  .trigger {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    background: var(--accent-bg-soft);
    border: 1px solid var(--accent-bg-medium);
    border-radius: var(--r-pill);
    color: var(--accent-fg);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast), color var(--t-fast);
  }
  .trigger:hover:not(:disabled) {
    background: var(--accent-bg-medium);
    border-color: var(--accent-bg-strong);
  }
  .trigger:disabled { opacity: 0.6; cursor: progress; }
  .name {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    max-width: 220px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .total {
    background: var(--accent-bg-strong);
    color: var(--accent-fg);
    border-radius: var(--r-pill);
    padding: 1px 7px;
    font-size: 10px;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    line-height: 16px;
    font-weight: var(--weight-bold);
  }
  .counts {
    color: var(--fg-muted);
    font-family: var(--font-mono);
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
    margin-left: 2px;
  }
  .trigger :global(svg:last-of-type) { color: var(--accent-fg); opacity: 0.7; }

  .menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    min-width: 320px;
    max-width: 380px;
    max-height: 420px;
    display: flex;
    flex-direction: column;
    background: var(--bg-elev-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-3);
    padding: 6px;
    z-index: 10;
    overflow: hidden;
  }
  .menu::before {
    content: "";
    position: absolute; inset: 0;
    border-radius: var(--r-lg);
    background-image: var(--grain);
    opacity: 0.4;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .filter-wrap {
    flex-shrink: 0;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border);
    position: relative; z-index: 1;
  }
  .filter {
    display: block;
    width: 100%;
    padding: 8px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-family: var(--font-mono);
    outline: none;
    transition: border-color var(--t-fast);
  }
  .filter::placeholder { color: var(--fg-subtle); }
  .filter:focus { border-color: var(--accent-500); }

  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 6px 0;
    position: relative; z-index: 1;
  }
  ul { list-style: none; margin: 0; padding: 0; }
  li { padding: 0; }
  li.empty {
    padding: var(--sp-4);
    color: var(--fg-subtle);
    text-align: center;
    font-size: var(--fs-sm);
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    font-family: var(--font-mono);
    font-weight: var(--weight-medium);
    text-align: left;
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .item :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
  .item:disabled { opacity: 0.6; cursor: progress; }
  .item:hover:not(:disabled) { background: var(--bg-elev-2); color: var(--fg); }
  .item:hover:not(:disabled) :global(svg) { color: var(--fg-muted); }
  .item.current {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    font-weight: var(--weight-semibold);
  }
  .item.current :global(svg) { color: var(--accent-fg); }
  .item-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .default-tag {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    height: 16px;
    padding: 0 6px;
    border-radius: var(--r-pill);
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    border: 1px solid var(--accent-bg-strong);
    font-family: var(--font-sans);
    font-size: 9px;
    font-weight: var(--weight-bold);
    letter-spacing: var(--tracking-wider);
    text-transform: uppercase;
    line-height: 1;
  }

  .footer {
    flex-shrink: 0;
    padding-top: 6px;
    border-top: 1px solid var(--border);
    position: relative;
    z-index: 1;
  }
  .new {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    font-family: var(--font-sans);
    font-weight: var(--weight-semibold);
    text-align: left;
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .new :global(svg) { color: var(--accent-fg); flex-shrink: 0; }
  .new:hover:not(:disabled) { background: var(--bg-elev-2); color: var(--fg); }
  .new:disabled { opacity: 0.6; cursor: progress; }

  /* Modal */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in srgb, #000 55%, transparent);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 14vh;
    z-index: 100;
  }
  .modal {
    width: min(440px, calc(100vw - 32px));
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
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    position: relative; z-index: 1;
  }
  .modal-header h2 {
    margin: 0;
    font-size: var(--fs-md);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    color: var(--fg);
  }
  .modal-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px; height: 26px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .modal-close:hover { background: var(--bg-elev-3); color: var(--fg); }

  .modal-body {
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    position: relative; z-index: 1;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
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

  .seg {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }
  .seg-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    text-align: left;
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast), color var(--t-fast);
    min-width: 0;
  }
  .seg-btn :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
  .seg-btn:hover:not(:disabled) { background: var(--bg-elev-3); color: var(--fg); border-color: var(--border-strong); }
  .seg-btn.on {
    background: var(--accent-bg-medium);
    border-color: var(--accent-bg-strong);
    color: var(--accent-fg);
  }
  .seg-btn.on :global(svg) { color: var(--accent-fg); }
  .seg-btn:disabled { opacity: 0.45; cursor: not-allowed; }
  .seg-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .seg-tag {
    font-family: var(--font-sans);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
    font-weight: var(--weight-semibold);
  }
  .seg-btn.on .seg-tag { color: var(--accent-fg); opacity: 0.85; }

  .seg-static {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--accent-bg-soft);
    border: 1px solid var(--accent-bg-medium);
    border-radius: var(--r-sm);
    color: var(--accent-fg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    min-width: 0;
  }
  .seg-static :global(svg) { color: var(--accent-fg); flex-shrink: 0; }
  .seg-static .seg-tag { color: var(--accent-fg); opacity: 0.85; }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 4px;
  }
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
  .btn.primary {
    background: var(--accent-500);
    color: var(--accent-on);
  }
  .btn.primary:hover:not(:disabled) { background: var(--accent-400); }
  .btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.ghost {
    background: transparent;
    color: var(--fg-muted);
    border-color: var(--border);
  }
  .btn.ghost:hover:not(:disabled) { color: var(--fg); border-color: var(--border-strong); }

  /* Right-click context menu */
  .ctx-menu {
    position: fixed;
    min-width: 180px;
    padding: 4px;
    background: var(--bg-elev-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-3);
    z-index: 200;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .ctx-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    text-align: left;
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .ctx-item :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
  .ctx-item:hover:not(:disabled) { background: var(--bg-elev-2); color: var(--fg); }
  .ctx-item:hover:not(:disabled) :global(svg) { color: var(--fg-muted); }
  .ctx-item.danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--removed) 14%, transparent);
    color: var(--removed);
  }
  .ctx-item.danger:hover:not(:disabled) :global(svg) { color: var(--removed); }
  .ctx-item:disabled { opacity: 0.45; cursor: not-allowed; }
</style>
