<script lang="ts">
  import Icon from './Icon.svelte';
  import type { Snippet } from 'svelte';

  type Variant = 'primary' | 'secondary' | 'ghost' | 'danger';
  type Size = 'sm' | 'md' | 'lg';

  let {
    label,
    variant = 'secondary',
    size = 'md',
    iconLeft,
    iconRight,
    iconOnly,
    loading = false,
    disabled = false,
    type = 'button',
    onclick,
    title,
    badge,
    children,
  }: {
    label?: string;
    variant?: Variant;
    size?: Size;
    iconLeft?: string;
    iconRight?: string;
    iconOnly?: string;
    loading?: boolean;
    disabled?: boolean;
    type?: 'button' | 'submit';
    onclick?: (e: MouseEvent) => void;
    title?: string;
    badge?: string | number;
    children?: Snippet;
  } = $props();

  const isIconOnly = $derived(!!iconOnly);
  const iconSize = $derived(size === 'sm' ? 12 : size === 'md' ? 14 : 16);
  const effectiveDisabled = $derived(disabled || loading);
</script>

<button
  class="btn btn-{variant} btn-{size}"
  class:icon-only={isIconOnly}
  class:loading
  disabled={effectiveDisabled}
  {type}
  {onclick}
  title={title ?? (isIconOnly ? label : undefined)}
  aria-label={isIconOnly ? label : undefined}
>
  {#if loading}
    <span class="spinner" aria-hidden="true"></span>
  {:else if iconOnly}
    <Icon name={iconOnly} size={iconSize} />
  {:else if iconLeft}
    <Icon name={iconLeft} size={iconSize} />
  {/if}

  {#if !isIconOnly}
    {#if children}
      {@render children()}
    {:else if label}
      <span class="label">{label}</span>
    {/if}
  {/if}

  {#if !isIconOnly && iconRight && !loading}
    <Icon name={iconRight} size={iconSize} />
  {/if}

  {#if badge != null && badge !== '' && !isIconOnly}
    <span class="badge">{badge}</span>
  {/if}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-radius: var(--r-md);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    border: 1px solid transparent;
    line-height: 1;
    box-sizing: border-box;
    cursor: pointer;
    white-space: nowrap;
    user-select: none;
    transition:
      background var(--t-fast),
      border-color var(--t-fast),
      color var(--t-fast),
      box-shadow var(--t-fast),
      opacity var(--t-fast);
  }
  .btn:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn.loading { cursor: progress; }

  /* Sizes — height / padding-x / font-size */
  .btn-sm { height: 26px; padding: 0 10px; font-size: var(--fs-2xs); }
  .btn-md { height: 32px; padding: 0 14px; font-size: var(--fs-sm); }
  .btn-lg { height: 40px; padding: 0 18px; font-size: var(--fs-md); }

  /* Icon-only buttons are square */
  .btn-sm.icon-only { width: 26px; padding: 0; }
  .btn-md.icon-only { width: 32px; padding: 0; }
  .btn-lg.icon-only { width: 40px; padding: 0; }

  /* Variants */
  .btn-primary {
    background: var(--accent-500);
    color: var(--accent-on);
    border-color: var(--accent-500);
    box-shadow: var(--inset-top), 0 1px 0 rgba(0, 0, 0, 0.15);
  }
  .btn-primary :global(svg) { color: var(--accent-on); }
  .btn-primary:not(:disabled):hover  { background: var(--accent-400); border-color: var(--accent-400); }
  .btn-primary:not(:disabled):active { background: var(--accent-600); border-color: var(--accent-600); }

  .btn-secondary {
    background: var(--bg);
    color: var(--fg-muted);
    border-color: var(--border);
  }
  .btn-secondary :global(svg) { color: var(--fg-subtle); }
  .btn-secondary:not(:disabled):hover {
    color: var(--fg);
    border-color: var(--border-strong);
  }
  .btn-secondary:not(:disabled):hover :global(svg) { color: var(--fg); }

  .btn-ghost {
    background: transparent;
    color: var(--fg-muted);
    border-color: transparent;
  }
  .btn-ghost :global(svg) { color: var(--fg-subtle); }
  .btn-ghost:not(:disabled):hover {
    background: var(--bg-elev-2);
    color: var(--fg);
  }
  .btn-ghost:not(:disabled):hover :global(svg) { color: var(--fg); }

  .btn-danger {
    background: #f87171;
    color: #2a0a0a;
    border-color: #f87171;
    box-shadow: var(--inset-top), 0 1px 0 rgba(0, 0, 0, 0.15);
  }
  .btn-danger :global(svg) { color: #2a0a0a; }
  .btn-danger:not(:disabled):hover  { background: #fca5a5; border-color: #fca5a5; }
  .btn-danger:not(:disabled):active { background: #ef4444; border-color: #ef4444; }

  .label { display: inline-block; }

  /* Inline count chip — tints itself against parent variant via currentColor */
  .badge {
    display: inline-flex;
    align-items: center;
    height: 16px;
    padding: 0 6px;
    border-radius: var(--r-pill);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 10px;
    font-weight: var(--weight-bold);
    line-height: 1;
    background: color-mix(in srgb, currentColor 18%, transparent);
    color: inherit;
    margin-left: 2px;
  }
  .btn-primary .badge { background: color-mix(in srgb, var(--accent-on) 22%, transparent); }
  .btn-danger  .badge { background: color-mix(in srgb, #2a0a0a 22%, transparent); }

  /* Loading spinner */
  .spinner {
    width: 12px;
    height: 12px;
    border: 1.5px solid currentColor;
    border-right-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    opacity: 0.85;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
