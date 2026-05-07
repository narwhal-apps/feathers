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

  let open = $state(false);
  let busy = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let filterEl = $state<HTMLInputElement | null>(null);
  let filter = $state('');

  const filtered = $derived(
    filter.trim() === ''
      ? localBranches
      : localBranches.filter((b) => b.name.toLowerCase().includes(filter.toLowerCase())),
  );

  function close() {
    open = false;
    filter = '';
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

  function onDocClick(e: MouseEvent) {
    if (!open) return;
    const t = e.target as Node;
    if (triggerEl && (triggerEl === t || triggerEl.contains(t))) return;
    const menu = document.getElementById('branch-switcher-menu');
    if (menu && menu.contains(t)) return;
    close();
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
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
        <input
          class="filter"
          type="text"
          placeholder="Filter branches…"
          bind:value={filter}
          bind:this={filterEl}
        />
        <ul>
          {#each filtered as b}
            <li>
              <button
                class="item"
                class:current={b.is_head}
                role="menuitem"
                onclick={() => pick(b)}
                disabled={busy}
              >
                <Icon name="GitBranch" size={12} />
                <span class="item-name">{b.name}</span>
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
    {/if}
  </div>
{/if}

<style>
  .wrap { position: relative; }

  .trigger {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    height: 24px;
    padding: 0 8px;
    background: rgba(20, 184, 166, 0.12);
    border: 1px solid transparent;
    border-radius: 999px;
    color: var(--accent-300);
    font-size: var(--fs-xs);
    font-weight: 600;
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast);
  }
  .trigger:hover:not(:disabled) {
    background: rgba(20, 184, 166, 0.2);
    border-color: rgba(20, 184, 166, 0.3);
  }
  .trigger:disabled { opacity: 0.6; cursor: progress; }
  .name { font-variant-numeric: tabular-nums; }
  .total {
    background: rgba(20, 184, 166, 0.18);
    color: var(--accent-300);
    border-radius: 999px;
    padding: 0 6px;
    font-size: 10px;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    line-height: 16px;
    margin-left: 2px;
  }
  .counts {
    color: var(--fg-muted);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    margin-left: 4px;
  }

  .menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    min-width: 280px;
    max-width: 360px;
    max-height: 360px;
    overflow: auto;
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-2);
    padding: 4px;
    z-index: 10;
  }
  .filter {
    display: block;
    width: 100%;
    padding: 6px 8px;
    margin-bottom: 4px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    outline: none;
  }
  .filter:focus { border-color: var(--accent-500); }

  ul { list-style: none; margin: 0; padding: 0; }
  li { padding: 0; }
  li.empty {
    padding: var(--sp-3);
    color: var(--fg-subtle);
    text-align: center;
    font-size: var(--fs-sm);
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    padding: 6px 8px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: 500;
    text-align: left;
    cursor: pointer;
  }
  .item:disabled { opacity: 0.6; cursor: progress; }
  .item:hover:not(:disabled) { background: var(--bg-elev-2); }
  .item.current { background: rgba(20, 184, 166, 0.10); color: var(--accent-300); font-weight: 600; }
  .item-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
