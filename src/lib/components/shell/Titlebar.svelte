<script lang="ts">
  import Pill from '$lib/components/primitives/Pill.svelte';
  import Button from '$lib/components/primitives/Button.svelte';

  let {
    repoName = null,
    branch = null,
    ahead = 0,
    behind = 0,
  }: {
    repoName?: string | null;
    branch?: string | null;
    ahead?: number;
    behind?: number;
  } = $props();
</script>

<header class="titlebar" data-tauri-drag-region>
  <!-- 80px reserved for the macOS traffic lights (inset) -->
  <div class="lights-spacer" data-tauri-drag-region></div>

  {#if repoName}
    <span class="repo" data-tauri-drag-region>{repoName}</span>
    <span class="sep" data-tauri-drag-region>/</span>
    {#if branch}<Pill label={branch} tone="accent" />{/if}
    {#if ahead > 0 || behind > 0}
      <span class="counts" data-tauri-drag-region>↓ {behind}  ↑ {ahead}</span>
    {/if}
  {:else}
    <span class="placeholder" data-tauri-drag-region>No repository</span>
  {/if}

  <div class="spacer" data-tauri-drag-region></div>

  <div class="actions">
    <Button label="Fetch" variant="ghost" size="sm" disabled={!repoName} />
    <Button label="Push"  variant="primary" size="sm" disabled={!repoName} />
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
  .repo  { color: var(--fg);        font-weight: 600; }
  .sep   { color: var(--fg-subtle); }
  .counts { color: var(--fg-muted); font-family: var(--font-mono); font-size: var(--fs-xs); font-variant-numeric: tabular-nums; }
  .placeholder { color: var(--fg-subtle); font-style: italic; }
  .spacer { flex: 1; }
  .actions { display: flex; gap: var(--sp-2); }
</style>
