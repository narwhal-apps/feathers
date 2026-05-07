<script lang="ts">
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { openRepoFlow } from '$lib/components/dialogs/openRepo';
  import { goto } from '$app/navigation';
  import type { RepoSummary } from '$lib/types';

  let open = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);

  const active = $derived(repos.activeRepo);
  const list = $derived(repos.knownRepos);

  function close() { open = false; }

  function pick(r: RepoSummary) {
    repos.activeRepoId = r.id;
    close();
    goto(`/repo/${r.id}/changes/`);
  }

  async function add() {
    close();
    await openRepoFlow();
  }

  function onDocClick(e: MouseEvent) {
    if (!open) return;
    const t = e.target as Node;
    if (triggerEl && (triggerEl === t || triggerEl.contains(t))) return;
    const menu = document.getElementById('repo-switcher-menu');
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

  function avatarLetter(name: string): string {
    return (name.trim()[0] ?? '?').toUpperCase();
  }
</script>

{#if list.length === 0}
  <button class="empty-trigger" onclick={add}>
    <Icon name="FolderOpen" size={14} />
    <span>Open repository</span>
  </button>
{:else}
  <div class="wrap">
    <button
      class="trigger"
      bind:this={triggerEl}
      onclick={() => (open = !open)}
      aria-haspopup="menu"
      aria-expanded={open}
    >
      {#if active}
        <span class="avatar">{avatarLetter(active.name)}</span>
        <span class="name">{active.name}</span>
      {:else}
        <span class="avatar muted">·</span>
        <span class="name muted">Select repository</span>
      {/if}
      <Icon name="ChevronsUpDown" size={14} />
    </button>

    {#if open}
      <div id="repo-switcher-menu" class="menu" role="menu">
        <ul>
          {#each list as r}
            <li>
              <button
                class="item"
                class:active={r.id === active?.id}
                role="menuitem"
                onclick={() => pick(r)}
              >
                <span class="item-name">{r.name}</span>
                {#if r.id === active?.id}
                  <Icon name="Check" size={14} />
                {/if}
              </button>
            </li>
          {/each}
        </ul>
        <div class="divider"></div>
        <button class="add" role="menuitem" onclick={add}>
          <span>Add new repository</span>
          <Icon name="Plus" size={14} />
        </button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .wrap { position: relative; }

  .trigger,
  .empty-trigger {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: 34px;
    padding: 0 12px 0 6px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast);
    box-shadow: var(--inset-top);
  }
  .trigger:hover,
  .empty-trigger:hover {
    background: var(--bg-elev-3);
    border-color: var(--border-strong);
  }
  .empty-trigger { padding: 0 12px; gap: var(--sp-2); }

  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--r-sm);
    background: linear-gradient(135deg, var(--accent-600), var(--accent-800));
    color: var(--accent-50);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: var(--weight-bold);
    letter-spacing: 0;
    flex-shrink: 0;
    box-shadow: var(--inset-top), 0 1px 2px rgba(0, 0, 0, 0.3);
  }
  .avatar.muted {
    background: var(--bg-elev-1);
    color: var(--fg-subtle);
    box-shadow: none;
  }
  .name { color: var(--fg); max-width: 220px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .name.muted { color: var(--fg-subtle); font-weight: var(--weight-medium); }
  .trigger :global(svg:last-of-type) { color: var(--fg-subtle); margin-left: 2px; }

  .menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    min-width: 280px;
    max-width: 380px;
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
  .menu ul { list-style: none; margin: 0; padding: 0; position: relative; z-index: 1; }
  .menu li { padding: 0; }

  .item {
    display: flex;
    align-items: center;
    width: 100%;
    gap: var(--sp-2);
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
    cursor: pointer;
    text-align: left;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .item:hover { background: var(--bg-elev-2); color: var(--fg); }
  .item.active {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    font-weight: var(--weight-semibold);
  }
  .item-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 6px 4px;
    position: relative; z-index: 1;
  }

  .add {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    width: 100%;
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    transition: background var(--t-fast);
    position: relative; z-index: 1;
  }
  .add:hover { background: var(--accent-bg-soft); color: var(--accent-fg); }
</style>
