<script lang="ts">
  import Icon from './Icon.svelte';
  import type { Snippet } from 'svelte';

  let {
    icon,
    label,
    onclick,
    disabled = false,
    danger = false,
    title,
    children,
  }: {
    icon?: string;
    label?: string;
    onclick?: (e: MouseEvent) => void;
    disabled?: boolean;
    danger?: boolean;
    title?: string;
    children?: Snippet;
  } = $props();
</script>

<button
  type="button"
  class="ctx-item"
  class:danger
  role="menuitem"
  {disabled}
  {title}
  onclick={onclick}
>
  {#if icon}<Icon name={icon} size={12} />{/if}
  {#if children}{@render children()}{:else if label}<span>{label}</span>{/if}
</button>

<style>
  .ctx-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    text-align: left;
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .ctx-item :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
  .ctx-item:hover:not(:disabled) {
    background: var(--bg-elev-2);
    color: var(--fg);
  }
  .ctx-item:hover:not(:disabled) :global(svg) { color: var(--fg); }
  .ctx-item:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: -2px;
  }
  .ctx-item.danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--removed) 14%, transparent);
    color: var(--removed);
  }
  .ctx-item.danger:hover:not(:disabled) :global(svg) { color: var(--removed); }
  .ctx-item:disabled { opacity: 0.45; cursor: not-allowed; }
</style>
