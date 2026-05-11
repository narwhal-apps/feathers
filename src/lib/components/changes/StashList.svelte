<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { StashEntry } from '$lib/types';
  import { formatError } from '$lib/utils/error';

  let {
    repoId,
    selectedIndex,
    onSelect,
    onRequestSelect,
    disabled = false,
  }: {
    repoId: string;
    /** Currently-selected stash index, or null. Owned by the parent. */
    selectedIndex: number | null;
    /** Parent calls this to swap the selection (null = deselect). */
    onSelect: (index: number | null) => void;
    /** Convenience alias used inline in clicks. */
    onRequestSelect: (index: number | null) => void;
    /** True when an op is in progress; disables apply/pop/drop. */
    disabled?: boolean;
  } = $props();

  const stashes = createQuery<StashEntry[]>(
    () => queryKeys.repoStashes(repoId),
    () => invoke<StashEntry[]>('stash_list', { id: repoId }),
  );

  let collapsed = $state(false);
  let busy = $state<null | string>(null); // a key like 'apply:0' to disable just one row
  let actionError = $state<string | null>(null);

  // Right-click context menu state.
  let ctxMenu = $state<{ stash: StashEntry; x: number; y: number } | null>(null);


  function flashError(msg: string): void {
    actionError = msg;
    setTimeout(() => { if (actionError === msg) actionError = null; }, 5000);
  }

  function relativeTime(unix: number): string {
    const seconds = Math.floor(Date.now() / 1000) - unix;
    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    return `${Math.floor(seconds / 86400)}d ago`;
  }

  function shortMessage(s: StashEntry): string {
    // Strip the "WIP on <branch>: " prefix if present so the row reads cleaner.
    const wip = `WIP on ${s.branch}: `;
    if (s.message.startsWith(wip)) return s.message.slice(wip.length);
    const on = `On ${s.branch}: `;
    if (s.message.startsWith(on)) return s.message.slice(on.length);
    return s.message;
  }

  async function doApply(s: StashEntry): Promise<void> {
    if (disabled || busy) return;
    busy = `apply:${s.index}`;
    try {
      await invoke('stash_apply', { id: repoId, index: s.index });
      queryClient.invalidateMany([
        queryKeys.repoStatus(repoId),
        queryKeys.repoOpState(repoId),
        ['repo', repoId, 'diff'],
      ]);
    } catch (err) {
      flashError(formatError(err));
    } finally {
      busy = null;
    }
  }

  async function doPop(s: StashEntry): Promise<void> {
    if (disabled || busy) return;
    busy = `pop:${s.index}`;
    try {
      await invoke('stash_pop', { id: repoId, index: s.index });
      queryClient.invalidateMany([
        queryKeys.repoStashes(repoId),
        queryKeys.repoStatus(repoId),
        queryKeys.repoOpState(repoId),
        ['repo', repoId, 'diff'],
      ]);
      // The popped stash may no longer exist; clear selection if it was selected.
      if (selectedIndex === s.index) onSelect(null);
    } catch (err) {
      flashError(formatError(err));
    } finally {
      busy = null;
    }
  }

  async function doDrop(s: StashEntry): Promise<void> {
    closeCtxMenu();
    if (disabled || busy) return;
    if (!confirm(`Drop stash "${shortMessage(s)}"?\n\nThis cannot be undone via the UI.`)) return;
    busy = `drop:${s.index}`;
    try {
      await invoke('stash_drop', { id: repoId, index: s.index });
      queryClient.invalidate(queryKeys.repoStashes(repoId));
      if (selectedIndex === s.index) onSelect(null);
    } catch (err) {
      flashError(formatError(err));
    } finally {
      busy = null;
    }
  }

  function openCtxMenu(s: StashEntry, e: MouseEvent): void {
    e.preventDefault();
    e.stopPropagation();
    ctxMenu = { stash: s, x: e.clientX, y: e.clientY };
  }
  function closeCtxMenu(): void {
    ctxMenu = null;
  }

  function onDocClick(e: MouseEvent): void {
    if (!ctxMenu) return;
    const cm = document.getElementById('stash-ctx-menu');
    if (!cm || !cm.contains(e.target as Node)) closeCtxMenu();
  }
  function onKey(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      if (ctxMenu) closeCtxMenu();
      else if (selectedIndex != null) onSelect(null);
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
</script>

{#if (stashes.data?.length ?? 0) > 0}
  <section class="stashes" aria-label="Stashes">
    <header class="head">
      <button class="toggle" onclick={() => (collapsed = !collapsed)} aria-expanded={!collapsed}>
        <Icon name={collapsed ? 'ChevronRight' : 'ChevronDown'} size={12} />
        <span class="title">Stashes</span>
        <span class="count">{stashes.data?.length ?? 0}</span>
      </button>
    </header>

    {#if !collapsed}
      {#if actionError}
        <div class="strip" role="alert">
          {actionError}
          <button class="dismiss" onclick={() => (actionError = null)} aria-label="Dismiss">×</button>
        </div>
      {/if}

      <ul>
        {#each stashes.data ?? [] as s (s.oid)}
            <li
              class:selected={selectedIndex === s.index}
            >
              <button
                class="row"
                onclick={() => onRequestSelect(selectedIndex === s.index ? null : s.index)}
                oncontextmenu={(e) => openCtxMenu(s, e)}
                title={s.message}
              >
                <Icon name="Archive" size={12} />
                <span class="msg">{shortMessage(s)}</span>
                <span class="meta">on {s.branch || '?'} · {relativeTime(s.time)}</span>
              </button>
              <div class="actions">
                <button
                  class="act"
                  onclick={() => doApply(s)}
                  disabled={disabled || busy !== null}
                  title="Apply (keeps stash)"
                >
                  Apply
                </button>
                <button
                  class="act primary"
                  onclick={() => doPop(s)}
                  disabled={disabled || busy !== null}
                  title="Apply and drop"
                >
                  Pop
                </button>
                <button
                  class="act danger"
                  onclick={() => doDrop(s)}
                  disabled={disabled || busy !== null}
                  title="Drop without applying"
                  aria-label="Drop"
                >
                  <Icon name="Trash2" size={12} />
                </button>
              </div>
            </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

{#if ctxMenu}
  <div
    id="stash-ctx-menu"
    class="ctx-menu"
    role="menu"
    style="left: {ctxMenu.x}px; top: {ctxMenu.y}px;"
  >
    <button class="ctx-item" role="menuitem"
      onclick={() => { const s = ctxMenu!.stash; closeCtxMenu(); doApply(s); }}
      disabled={disabled}>
      <Icon name="Download" size={12} />
      <span>Apply</span>
    </button>
    <button class="ctx-item" role="menuitem"
      onclick={() => { const s = ctxMenu!.stash; closeCtxMenu(); doPop(s); }}
      disabled={disabled}>
      <Icon name="Download" size={12} />
      <span>Pop (apply + drop)</span>
    </button>
    <button class="ctx-item" role="menuitem"
      onclick={() => { const s = ctxMenu!.stash; closeCtxMenu(); onRequestSelect(s.index); }}>
      <Icon name="Eye" size={12} />
      <span>Show diff</span>
    </button>
    <div class="ctx-divider"></div>
    <button class="ctx-item danger" role="menuitem"
      onclick={() => doDrop(ctxMenu!.stash)}
      disabled={disabled}>
      <Icon name="Trash2" size={12} />
      <span>Drop</span>
    </button>
  </div>
{/if}

<style>
  .stashes {
    border-bottom: 1px solid var(--border);
    padding: 6px 0 10px;
    margin-bottom: var(--sp-2);
  }
  .head { padding: 0 var(--sp-3) 0 10px; }
  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    background: transparent;
    border: none;
    padding: 4px 0;
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    font-weight: var(--weight-semibold);
    cursor: pointer;
  }
  .toggle :global(svg) { color: var(--fg-subtle); }
  .toggle .title { color: var(--fg-subtle); }
  .toggle .count {
    margin-left: 4px;
    color: var(--fg-muted);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .strip {
    margin: 4px 10px;
    padding: 6px 10px;
    background: color-mix(in srgb, #c00 12%, var(--bg-elev-1));
    border: 1px solid color-mix(in srgb, #c00 30%, var(--border));
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-2xs);
    display: flex;
    justify-content: space-between;
    gap: 6px;
  }
  .strip .dismiss {
    background: transparent; border: none; color: var(--fg-muted);
    font-size: 14px; line-height: 1; cursor: pointer; padding: 0 2px;
  }
  ul { list-style: none; margin: 0; padding: 0; }
  li {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 var(--sp-3) 0 10px;
    min-height: 30px;
    position: relative;
  }
  li:hover { background: var(--bg-elev-2); }
  li.selected { background: var(--accent-bg-medium); }
  li.selected :global(svg) { color: var(--accent-fg); }

  .row {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: none;
    padding: 4px 0;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .row :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
  .msg {
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 60%;
  }
  .meta {
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .actions { display: flex; align-items: center; gap: 4px; opacity: 0; transition: opacity var(--t-fast); }
  li:hover .actions { opacity: 1; }
  .act {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 22px;
    padding: 0 8px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    box-sizing: border-box;
  }
  .act:hover:not(:disabled) { color: var(--fg); border-color: var(--border-strong); }
  .act.primary { background: var(--accent-500); color: var(--accent-on); border-color: var(--accent-500); }
  .act.danger { color: var(--fg-muted); width: 22px; padding: 0; }
  .act.danger:hover:not(:disabled) {
    color: var(--removed);
    background: color-mix(in srgb, var(--removed) 14%, transparent);
    border-color: color-mix(in srgb, var(--removed) 30%, transparent);
  }
  .act:disabled { opacity: 0.4; cursor: not-allowed; }

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
  }
  .ctx-item :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
  .ctx-item:hover:not(:disabled) { background: var(--bg-elev-2); color: var(--fg); }
  .ctx-item.danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--removed) 14%, transparent);
    color: var(--removed);
  }
  .ctx-item:disabled { opacity: 0.45; cursor: not-allowed; }
  .ctx-divider { height: 1px; background: var(--border); margin: 4px 6px; }
</style>
