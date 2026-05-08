<script lang="ts">
  import Icon from '$lib/components/primitives/Icon.svelte';

  type Pane = 'general' | 'account' | 'git' | 'repos' | 'about';

  let { active, onSelect }: { active: Pane; onSelect: (p: Pane) => void } = $props();

  const items: { id: Pane; label: string; icon: string }[] = [
    { id: 'general', label: 'General', icon: 'Palette' },
    { id: 'account', label: 'Account', icon: 'Github' },
    { id: 'git', label: 'Git identity', icon: 'UserCircle' },
    { id: 'repos', label: 'Repositories', icon: 'Folder' },
    { id: 'about', label: 'About', icon: 'Info' },
  ];
</script>

<nav class="sidebar">
  <ul>
    {#each items as item}
      <li>
        <button
          class="item"
          class:active={item.id === active}
          onclick={() => onSelect(item.id)}
        >
          <Icon name={item.icon} size={14} />
          <span>{item.label}</span>
        </button>
      </li>
    {/each}
  </ul>
</nav>

<style>
  .sidebar {
    width: 180px;
    flex-shrink: 0;
    border-right: 1px solid var(--border);
    padding: var(--sp-3) var(--sp-2);
    background: var(--bg-elev-1);
  }
  .sidebar ul { list-style: none; margin: 0; padding: 0; }
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
</style>
