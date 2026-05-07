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
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    background: var(--accent-bg-soft);
    border: 1px solid var(--accent-bg-medium);
    border-radius: var(--r-pill);
    color: var(--accent-300);
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
    color: var(--accent-300);
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
  .trigger :global(svg:last-of-type) { color: var(--accent-300); opacity: 0.7; }

  .menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    min-width: 300px;
    max-width: 380px;
    max-height: 380px;
    overflow: auto;
    background: var(--bg-elev-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-3);
    padding: 6px;
    z-index: 10;
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
  .filter {
    display: block;
    width: 100%;
    padding: 8px 10px;
    margin-bottom: 6px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-family: var(--font-mono);
    outline: none;
    transition: border-color var(--t-fast);
    position: relative; z-index: 1;
  }
  .filter::placeholder { color: var(--fg-subtle); }
  .filter:focus { border-color: var(--accent-500); }

  ul { list-style: none; margin: 0; padding: 0; position: relative; z-index: 1; }
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
    color: var(--accent-300);
    font-weight: var(--weight-semibold);
  }
  .item.current :global(svg) { color: var(--accent-300); }
  .item-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
