<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import SegmentedControl from '$lib/components/primitives/SegmentedControl.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import { ui } from '$lib/stores/ui.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import Field from '$lib/components/primitives/Field.svelte';
  import Input from '$lib/components/primitives/Input.svelte';
  import ContextMenu from '$lib/components/primitives/ContextMenu.svelte';
  import ContextMenuItem from '$lib/components/primitives/ContextMenuItem.svelte';
  import { confirm, notify } from '$lib/utils/dialog.svelte';
  import { formatError } from '$lib/utils/error';
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

  // Remote-tracking refs that don't have a same-named local branch yet.
  // We strip the first path segment (the remote name, usually "origin") and
  // skip the synthetic HEAD pointer.
  const localNames = $derived(new Set(localBranches.map((b) => b.name)));
  const remoteOnly = $derived.by(() => {
    const list = (branches.data ?? []).filter((b) => b.is_remote);
    return list.filter((b) => {
      const stripped = b.name.split('/').slice(1).join('/');
      return stripped !== '' && !stripped.endsWith('HEAD') && !localNames.has(stripped);
    });
  });

  function matchesFilter(b: BranchInfo): boolean {
    const q = filter.trim().toLowerCase();
    return q === '' || b.name.toLowerCase().includes(q);
  }
  const filteredLocal = $derived(localBranches.filter(matchesFilter));
  const filteredRemote = $derived(remoteOnly.filter(matchesFilter));
  const filteredEmpty = $derived(
    filteredLocal.length === 0 && filteredRemote.length === 0,
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
    notify(`${prefix}: ${formatError(err)}`, { kind: 'error', durationMs: 0 });
  }

  async function pick(b: BranchInfo) {
    if (!active) return;
    if (b.is_head) { close(); return; }
    busy = true;
    try {
      await invoke('branch_checkout', { id: active.id, name: b.name });
      // Branch + status + log + op-state + workdir diff all change after checkout.
      queryClient.invalidateMany([
        queryKeys.repoStatus(active.id),
        queryKeys.repoBranches(active.id),
        ['repo', active.id, 'log'],
        queryKeys.repoOpState(active.id),
        ['repo', active.id, 'diff'],
      ]);
      close();
    } catch (err) {
      const e = err as AppError;
      if (e.kind === 'dirty') {
        const text =
          `Cannot switch branches — working tree has uncommitted changes:\n\n` +
          e.paths.slice(0, 10).join('\n') +
          (e.paths.length > 10 ? `\n…and ${e.paths.length - 10} more` : '') +
          `\n\nCommit, stash, or discard them first.`;
        notify(text, { kind: 'error', durationMs: 0 });
      } else {
        notify(`Failed to switch: ${formatError(err)}`, { kind: 'error', durationMs: 0 });
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
      // Create + checkout: branches list changes, status/op-state shift to
      // the new branch, and log + workdir diff may differ.
      queryClient.invalidateMany([
        queryKeys.repoBranches(active.id),
        queryKeys.repoStatus(active.id),
        ['repo', active.id, 'log'],
        queryKeys.repoOpState(active.id),
        ['repo', active.id, 'diff'],
      ]);
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
      queryClient.invalidate(queryKeys.repoBranches(active.id));
      closeRename();
    } catch (err) {
      reportError('Failed to rename branch', err);
    } finally {
      busy = false;
    }
  }

  async function confirmDelete(b: BranchInfo) {
    closeCtxMenu();
    const ok = await confirm({
      title: 'Delete branch',
      message: `Delete branch "${b.name}"?`,
      confirmLabel: 'Delete',
      danger: true,
    });
    if (!ok) return;
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
        const text =
          `Branch "${name}" has commits that aren't merged into HEAD.\n\n` +
          `Force delete and lose those commits?`;
        const ok = await confirm({
          title: 'Force delete?',
          message: text,
          confirmLabel: 'Force delete',
          danger: true,
        });
        if (ok) await runDelete(name, true);
        return;
      }
      reportError('Failed to delete branch', err);
    } finally {
      busy = false;
    }
  }

  function onDocClick(e: MouseEvent) {
    if (!open) return;
    const t = e.target as Node;
    if (triggerEl && (triggerEl === t || triggerEl.contains(t))) return;
    const menu = document.getElementById('branch-switcher-menu');
    if (menu && menu.contains(t)) return;
    close();
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (renameTarget) closeRename();
      else if (modalOpen) closeModal();
      else if (open) close();
      // ctxMenu Escape is handled by ContextMenu primitive
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

  // External request (⌘B): toggle the dropdown ONLY when the request
  // counter actually advances. Without the lastReq guard, any unrelated
  // reactive change to `active` (e.g. the repo list refetching) would
  // re-run this effect and re-toggle, making the dropdown feel stuck.
  let lastBranchReq: number | null = null;
  $effect(() => {
    const req = ui.branchSwitcherRequest;
    if (req != null && req !== lastBranchReq) {
      lastBranchReq = req;
      if (active) open = !open;
    }
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
      title="Switch branch (⌘B)"
    >
      <Icon name="GitBranch" size={12} />
      <span class="name">{head.name}</span>
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
            {#if filteredLocal.length > 0}
              <li class="section-head">
                <span>Local</span>
                <span class="count" title="{localBranches.length} local branches">{localBranches.length}</span>
              </li>
              {#each filteredLocal as b (b.name)}
                {@const tracked = b.ahead != null || b.behind != null}
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
                    <span
                      class="loc-icon"
                      class:remote={tracked}
                      title={tracked ? 'Tracking a remote branch' : 'Local only — not on any remote yet'}
                      aria-label={tracked ? 'remote' : 'local only'}
                    >
                      <Icon name={tracked ? 'Cloud' : 'HardDrive'} size={11} />
                    </span>
                    {#if isDefaultBranch(b)}
                      <span class="default-tag">default</span>
                    {/if}
                    {#if b.is_head}
                      <Icon name="Check" size={14} />
                    {/if}
                  </button>
                </li>
              {/each}
            {/if}

            {#if filteredRemote.length > 0}
              <li class="section-head">Remote</li>
              {#each filteredRemote as b (b.name)}
                <li>
                  <button
                    class="item"
                    role="menuitem"
                    onclick={() => pick(b)}
                    disabled={busy}
                    title="Check out — creates a local branch tracking {b.name}"
                  >
                    <Icon name="GitBranch" size={12} />
                    <span class="item-name">{b.name}</span>
                    <span class="loc-icon remote" aria-label="remote" title="Remote-only — checking out will create a local tracking branch">
                      <Icon name="Cloud" size={11} />
                    </span>
                  </button>
                </li>
              {/each}
            {/if}

            {#if filteredEmpty}
              <li class="empty">No matching branches.</li>
            {/if}
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
  <Modal
    title="New branch"
    onClose={closeModal}
    width="md"
    actions={{
      secondary: { label: 'Cancel', onclick: closeModal, disabled: busy },
      primary: {
        label: busy ? 'Creating…' : 'Create branch',
        onclick: createBranch,
        loading: busy,
        disabled: busy || !newName.trim(),
      },
    }}
  >
    {#snippet body()}
      <form class="form" onsubmit={(e) => { e.preventDefault(); createBranch(); }}>
        <Field label="Name">
          <Input
            variant="mono"
            bind:value={newName}
            bind:ref={modalNameEl}
            disabled={busy}
            placeholder="feature/short-description"
          />
        </Field>

        <div class="field">
          <span class="from-label">Branch from</span>
          {#if showDefaultOption}
            <SegmentedControl
              options={[
                { value: 'current', label: `${head.name} · current`, icon: 'GitBranch' },
                { value: 'default', label: `${defaultBranch?.name ?? ''} · default`, icon: 'GitBranch' },
              ]}
              bind:value={fromKind}
              ariaLabel="Branch from"
              size="md"
              disabled={busy}
            />
          {:else}
            <div class="seg-static">
              <Icon name="GitBranch" size={12} />
              <span class="seg-name">{head.name}</span>
              <span class="seg-tag">{defaultBranch?.name === head.name ? 'current · default' : 'current'}</span>
            </div>
          {/if}
        </div>
      </form>
    {/snippet}
  </Modal>
{/if}

{#if ctxMenu}
  {@const cm = ctxMenu}
  <ContextMenu open={true} x={cm.x} y={cm.y} onClose={closeCtxMenu}>
    <ContextMenuItem
      icon="Pencil"
      label="Rename branch…"
      onclick={() => startRename(cm.branch)}
      disabled={busy}
    />
    <ContextMenuItem
      icon="Trash2"
      label="Delete branch"
      danger
      onclick={() => confirmDelete(cm.branch)}
      disabled={busy || cm.branch.is_head}
      title={cm.branch.is_head ? 'Cannot delete the current branch' : ''}
    />
  </ContextMenu>
{/if}

{#if renameTarget}
  {@const target = renameTarget}
  <Modal
    title="Rename branch"
    onClose={closeRename}
    width="md"
    actions={{
      secondary: { label: 'Cancel', onclick: closeRename, disabled: busy },
      primary: {
        label: busy ? 'Renaming…' : 'Rename',
        onclick: submitRename,
        loading: busy,
        disabled: busy || !renameName.trim() || renameName.trim() === target.name,
      },
    }}
  >
    {#snippet body()}
      <form class="form" onsubmit={(e) => { e.preventDefault(); submitRename(); }}>
        <Field label={`Rename "${target.name}" to`}>
          <Input
            variant="mono"
            bind:value={renameName}
            bind:ref={renameNameEl}
            disabled={busy}
          />
        </Field>
      </form>
    {/snippet}
  </Modal>
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
  li.section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 10px 4px;
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    font-weight: var(--weight-semibold);
  }
  li.section-head .count {
    background: var(--accent-bg-strong);
    color: var(--accent-fg);
    border-radius: var(--r-pill);
    padding: 1px 7px;
    font-size: 10px;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    line-height: 16px;
    font-weight: var(--weight-bold);
    /* Section head is uppercase via text-transform; numbers shouldn't
       inherit that visual weight. */
    text-transform: none;
    letter-spacing: 0;
  }
  /* Tighten the gap when one section follows another. */
  li.section-head + li { margin-top: 0; }

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

  /* Tiny "is this branch tracked or local-only" indicator. */
  .loc-icon {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    color: var(--fg-faint);
  }
  .loc-icon.remote { color: var(--accent-fg); opacity: 0.85; }
  /* Override the .item :global(svg) catch-all so this stays subtle. */
  .item .loc-icon :global(svg) { color: inherit; }

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

  /* Modal form layout (Field handles its own internals). */
  .form { display: flex; flex-direction: column; gap: var(--sp-3); }
  .field { display: flex; flex-direction: column; gap: 6px; }
  .from-label {
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
    font-weight: var(--weight-semibold);
  }

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
</style>
