<script lang="ts">
  import { page } from '$app/stores';
  import { repos } from '$lib/stores/repos.svelte';

  let { children } = $props();

  // Sync URL → active repo.
  $effect(() => {
    repos.activeRepoId = $page.params.id ?? null;
  });

  const tabs = [
    { href: 'changes', label: 'Changes' },
    { href: 'history', label: 'History' },
  ];
  const id = $derived($page.params.id ?? '');
  const active = $derived($page.url.pathname.split('/')[3] ?? 'changes');
</script>

<nav class="tabs">
  {#each tabs as tab}
    <a
      class="tab"
      class:active={active === tab.href}
      href={`/repo/${id}/${tab.href}/`}
    >{tab.label}</a>
  {/each}
</nav>

<div class="content">
  {@render children?.()}
</div>

<style>
  .tabs {
    display: flex;
    gap: var(--sp-1);
    padding: 0 var(--sp-3);
    border-bottom: 1px solid var(--border);
    background: var(--bg-elev-1);
  }
  .tab {
    padding: var(--sp-2) var(--sp-3);
    color: var(--fg-muted);
    border-bottom: 2px solid transparent;
    font-size: var(--fs-sm);
    font-weight: 600;
    text-decoration: none;
  }
  .tab:hover { color: var(--fg); }
  .tab.active { color: var(--accent-fg); border-bottom-color: var(--accent-500); }
  .content { height: calc(100% - 33px); overflow: hidden; }
</style>
