<script lang="ts" generics="T extends string">
  import Icon from './Icon.svelte';

  type Size = 'sm' | 'md';
  interface Option {
    value: T;
    label: string;
    icon?: string;
  }

  let {
    options,
    value = $bindable(),
    onChange,
    size = 'md',
    ariaLabel,
    disabled = false,
  }: {
    options: Option[];
    value: T;
    onChange?: (v: T) => void;
    size?: Size;
    ariaLabel?: string;
    disabled?: boolean;
  } = $props();

  const iconSize = $derived(size === 'sm' ? 12 : 14);
</script>

<div role="radiogroup" aria-label={ariaLabel} class="seg seg-{size}" class:disabled>
  {#each options as opt (opt.value)}
    <button
      type="button"
      class="seg-btn"
      class:on={value === opt.value}
      role="radio"
      aria-checked={value === opt.value}
      {disabled}
      onclick={() => {
        if (value === opt.value) return;
        value = opt.value;
        onChange?.(opt.value);
      }}
    >
      {#if opt.icon}<Icon name={opt.icon} size={iconSize} />{/if}
      <span>{opt.label}</span>
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    align-items: stretch;
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: 2px;
    gap: 2px;
    box-sizing: border-box;
  }
  .seg.disabled { opacity: 0.5; cursor: not-allowed; }

  .seg-sm { height: 26px; }
  .seg-md { height: 32px; }

  .seg-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 0 10px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    line-height: 1;
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast), border-color var(--t-fast);
  }
  .seg-sm .seg-btn { font-size: var(--fs-2xs); padding: 0 8px; }

  .seg-btn :global(svg) { color: var(--fg-subtle); flex-shrink: 0; }
  .seg-btn:not(:disabled):hover {
    background: var(--bg-elev-2);
    color: var(--fg);
  }
  .seg-btn:not(:disabled):hover :global(svg) { color: var(--fg); }

  .seg-btn.on {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    border-color: var(--accent-bg-strong);
  }
  .seg-btn.on :global(svg) { color: var(--accent-fg); }

  .seg-btn:disabled { cursor: not-allowed; }
</style>
