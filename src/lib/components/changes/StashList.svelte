<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import ContextMenu from '$lib/components/primitives/ContextMenu.svelte';
  import ContextMenuItem from '$lib/components/primitives/ContextMenuItem.svelte';
  import ContextMenuDivider from '$lib/components/primitives/ContextMenuDivider.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import { relTime } from '$lib/utils/time';
  import type { StashEntry } from '$lib/types';
  import { formatError } from '$lib/utils/error';
  import { confirm, notify } from '$lib/utils/dialog.svelte';

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

  let collapsed = $state(true);
  let busy = $state<null | string>(null); // a key like 'apply:0' to disable just one row

  // Right-click context menu state.
  let ctxMenu = $state<{ stash: StashEntry; x: number; y: number } | null>(null);

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
      notify(formatError(err), { kind: 'error', durationMs: 5000 });
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
      notify(formatError(err), { kind: 'error', durationMs: 5000 });
    } finally {
      busy = null;
    }
  }

  async function doDrop(s: StashEntry): Promise<void> {
    closeCtxMenu();
    if (disabled || busy) return;
    const ok = await confirm({
      title: 'Drop stash',
      message: `Drop stash "${shortMessage(s)}"?\n\nThis cannot be undone via the UI.`,
      confirmLabel: 'Drop',
      danger: true,
    });
    if (!ok) return;
    busy = `drop:${s.index}`;
    try {
      await invoke('stash_drop', { id: repoId, index: s.index });
      queryClient.invalidate(queryKeys.repoStashes(repoId));
      if (selectedIndex === s.index) onSelect(null);
    } catch (err) {
      notify(formatError(err), { kind: 'error', durationMs: 5000 });
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

  // ContextMenu primitive handles its own outside-click + Escape close;
  // we only need to handle Escape-to-deselect when no ctx menu is up.
  function onKey(e: KeyboardEvent): void {
    if (e.key === 'Escape' && !ctxMenu && selectedIndex != null) onSelect(null);
  }
  $effect(() => {
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
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
                <span class="meta">on {s.branch || '?'} · {relTime(s.time)}</span>
              </button>
              <div class="actions">
                <Button
                  variant="ghost"
                  size="sm"
                  label="Apply"
                  onclick={() => doApply(s)}
                  disabled={disabled || busy !== null}
                  title="Apply (keeps stash)"
                />
                <Button
                  variant="primary"
                  size="sm"
                  label="Pop"
                  onclick={() => doPop(s)}
                  disabled={disabled || busy !== null}
                  title="Apply and drop"
                />
                <Button
                  variant="ghost"
                  size="sm"
                  iconOnly="Trash2"
                  label="Drop"
                  onclick={() => doDrop(s)}
                  disabled={disabled || busy !== null}
                  title="Drop without applying"
                />
              </div>
            </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}

{#if ctxMenu}
  {@const cm = ctxMenu}
  <ContextMenu open={true} x={cm.x} y={cm.y} onClose={closeCtxMenu}>
    <ContextMenuItem
      icon="Download"
      label="Apply"
      onclick={() => { closeCtxMenu(); doApply(cm.stash); }}
      disabled={disabled}
    />
    <ContextMenuItem
      icon="Download"
      label="Pop (apply + drop)"
      onclick={() => { closeCtxMenu(); doPop(cm.stash); }}
      disabled={disabled}
    />
    <ContextMenuItem
      icon="Eye"
      label="Show diff"
      onclick={() => { closeCtxMenu(); onRequestSelect(cm.stash.index); }}
    />
    <ContextMenuDivider />
    <ContextMenuItem
      icon="Trash2"
      label="Drop"
      danger
      onclick={() => doDrop(cm.stash)}
      disabled={disabled}
    />
  </ContextMenu>
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

  .actions {
    display: flex;
    align-items: center;
    gap: 4px;
    opacity: 0;
    transition: opacity var(--t-fast);
  }
  /* Reveal action buttons on hover OR when keyboard focus enters the row,
     so they're reachable via Tab. */
  li:hover .actions,
  li:focus-within .actions { opacity: 1; }
</style>
