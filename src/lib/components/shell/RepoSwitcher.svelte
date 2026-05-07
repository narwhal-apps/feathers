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
    height: 32px;
    padding: 0 10px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: 600;
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast);
  }
  .trigger:hover,
  .empty-trigger:hover {
    background: var(--bg-elev-1);
    border-color: var(--border-strong);
  }

  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: var(--accent-700);
    color: white;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    flex-shrink: 0;
  }
  .avatar.muted { background: var(--bg-elev-1); color: var(--fg-subtle); }
  .name { color: var(--fg); }
  .name.muted { color: var(--fg-subtle); font-weight: 500; }

  .menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    min-width: 240px;
    max-width: 360px;
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-2);
    padding: 4px;
    z-index: 10;
  }
  .menu ul { list-style: none; margin: 0; padding: 0; }
  .menu li { padding: 0; }

  .item {
    display: flex;
    align-items: center;
    width: 100%;
    gap: var(--sp-2);
    padding: 6px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: 500;
    cursor: pointer;
    text-align: left;
  }
  .item:hover { background: var(--bg-elev-2); }
  .item.active { background: rgba(20, 184, 166, 0.10); color: var(--accent-300); font-weight: 600; }
  .item-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .divider { height: 1px; background: var(--border); margin: 4px 0; }

  .add {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    width: 100%;
    padding: 6px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: 600;
    cursor: pointer;
  }
  .add:hover { background: var(--bg-elev-2); }

  .empty-trigger { font-weight: 600; }
</style>
