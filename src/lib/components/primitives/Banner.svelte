<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  type Tone = 'info' | 'success' | 'warn' | 'error';

  let {
    tone = 'info',
    icon,
    title,
    children,
    actions,
  }: {
    tone?: Tone;
    /** Override the default icon for the chosen tone. Pass `null` to hide. */
    icon?: string | null;
    title?: string;
    /** Body content. */
    children?: Snippet;
    /** Trailing action area, typically one or two Buttons. */
    actions?: Snippet;
  } = $props();

  const defaultIcon = $derived(
    tone === 'success' ? 'Check'
      : tone === 'warn' ? 'AlertTriangle'
      : tone === 'error' ? 'AlertTriangle'
      : 'Info',
  );
  const showIcon = $derived(icon !== null);
  const iconName = $derived(icon ?? defaultIcon);
</script>

<div class="banner tone-{tone}" role={tone === 'error' ? 'alert' : 'status'}>
  {#if showIcon}
    <span class="icon" aria-hidden="true">
      <Icon name={iconName} size={14} />
    </span>
  {/if}
  <div class="body">
    {#if title}<strong class="title">{title}</strong>{/if}
    {#if children}<div class="text">{@render children()}</div>{/if}
  </div>
  {#if actions}
    <div class="actions">{@render actions()}</div>
  {/if}
</div>

<style>
  .banner {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    background: var(--bg-elev-1);
    color: var(--fg);
    font-size: var(--fs-xs);
    line-height: 1.5;
  }
  .icon {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: var(--r-pill);
    color: var(--fg);
  }
  .body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .title {
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    color: var(--fg);
  }
  .text { color: var(--fg-muted); }
  .actions {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
    margin-left: 4px;
  }

  .tone-info {
    background: var(--bg-elev-1);
    border-color: var(--border);
  }
  .tone-info .icon {
    background: color-mix(in srgb, var(--accent-500) 18%, transparent);
    color: var(--accent-fg);
  }

  .tone-success {
    background: color-mix(in srgb, var(--added) 14%, transparent);
    border-color: color-mix(in srgb, var(--added) 30%, transparent);
  }
  .tone-success .icon {
    background: var(--added);
    color: var(--bg);
  }

  .tone-warn {
    background: color-mix(in srgb, var(--warn) 14%, transparent);
    border-color: color-mix(in srgb, var(--warn) 30%, transparent);
  }
  .tone-warn .icon {
    background: var(--warn);
    color: var(--bg);
  }

  .tone-error {
    background: color-mix(in srgb, var(--removed) 12%, transparent);
    border-color: color-mix(in srgb, var(--removed) 35%, transparent);
  }
  .tone-error .icon {
    background: color-mix(in srgb, var(--removed) 40%, transparent);
    color: var(--fg);
  }
</style>
