<script lang="ts">
  import Button from '$lib/components/primitives/Button.svelte';
  import RepoSwitcher from '$lib/components/shell/RepoSwitcher.svelte';
  import BranchSwitcher from '$lib/components/shell/BranchSwitcher.svelte';
  import { repos } from '$lib/stores/repos.svelte';

  const active = $derived(repos.activeRepo);
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="lights-spacer" data-tauri-drag-region></div>

  <div class="cluster" data-tauri-drag-region>
    <RepoSwitcher />
    {#if active}
      <span class="separator" aria-hidden="true" data-tauri-drag-region></span>
      <BranchSwitcher />
    {/if}
  </div>

  <div class="spacer" data-tauri-drag-region></div>

  <div class="actions">
    <Button label="Fetch" variant="ghost" size="sm" disabled />
    <Button label="Push"  variant="primary" size="sm" disabled />
  </div>
</header>

<style>
  .titlebar {
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    height: 56px;
    padding: 0 var(--sp-4) 0 0;
    background:
      linear-gradient(180deg,
        color-mix(in srgb, var(--bg-elev-1) 100%, transparent) 0%,
        color-mix(in srgb, var(--bg-elev-1) 96%, transparent) 100%),
      var(--bg-elev-1);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }
  .titlebar::after {
    content: "";
    position: absolute;
    inset: 0;
    background-image: var(--grain);
    opacity: 0.5;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .lights-spacer { width: 80px; height: 100%; flex-shrink: 0; position: relative; z-index: 1; }
  .cluster { display: flex; align-items: center; gap: var(--sp-2); position: relative; z-index: 1; }
  .separator {
    width: 4px; height: 4px;
    border-radius: var(--r-pill);
    background: var(--fg-faint);
    margin: 0 var(--sp-1);
  }
  .spacer { flex: 1; position: relative; z-index: 1; }
  .actions { display: flex; gap: var(--sp-2); position: relative; z-index: 1; }
</style>
