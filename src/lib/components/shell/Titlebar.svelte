<script lang="ts">
  import Button from '$lib/components/primitives/Button.svelte';
  import RepoSwitcher from '$lib/components/shell/RepoSwitcher.svelte';
  import BranchSwitcher from '$lib/components/shell/BranchSwitcher.svelte';
  import { repos } from '$lib/stores/repos.svelte';

  const active = $derived(repos.activeRepo);
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="lights-spacer" data-tauri-drag-region></div>

  <RepoSwitcher />

  {#if active}
    <span class="sep" data-tauri-drag-region>/</span>
    <BranchSwitcher />
  {/if}

  <div class="spacer" data-tauri-drag-region></div>

  <div class="actions">
    <Button label="Fetch" variant="ghost" size="sm" disabled />
    <Button label="Push"  variant="primary" size="sm" disabled />
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    height: 56px;
    padding: 0 var(--sp-4) 0 0;
    background: var(--bg-elev-1);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }
  .lights-spacer { width: 80px; height: 100%; flex-shrink: 0; }
  .sep   { color: var(--fg-subtle); }
  .spacer { flex: 1; }
  .actions { display: flex; gap: var(--sp-2); }
</style>
