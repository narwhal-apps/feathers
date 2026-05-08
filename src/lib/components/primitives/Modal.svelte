<script lang="ts">
  import Icon from './Icon.svelte';
  import { portal } from '$lib/utils/portal';

  type Width = 'sm' | 'md' | 'lg';
  type Align = 'top' | 'center';

  let {
    title,
    onClose,
    width = 'md',
    align = 'top',
    closeOnBackdrop = true,
    showClose = true,
    titleId = 'modal-title',
    head,
    body,
    foot,
  }: {
    title?: string;
    /** Omit to make the modal sticky — no Esc, no backdrop close, no X. */
    onClose?: () => void;
    width?: Width;
    align?: Align;
    closeOnBackdrop?: boolean;
    showClose?: boolean;
    titleId?: string;
    head?: import('svelte').Snippet;
    body?: import('svelte').Snippet;
    foot?: import('svelte').Snippet;
  } = $props();

  const dismissible = $derived(!!onClose);
  const closeBtnVisible = $derived(showClose && dismissible);

  function onBackdropClick(e: MouseEvent) {
    if (!onClose || !closeOnBackdrop) return;
    if (e.target === e.currentTarget) onClose();
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape' && onClose) onClose();
  }
  $effect(() => {
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  });
</script>

<div
  class="backdrop align-{align}"
  role="presentation"
  use:portal
  onclick={onBackdropClick}
  onkeydown={() => {}}
>
  <div
    class="modal w-{width}"
    role="dialog"
    aria-modal="true"
    aria-labelledby={title ? titleId : undefined}
  >
    {#if head}
      {@render head()}
    {:else if title}
      <header class="head">
        <h2 id={titleId}>{title}</h2>
        {#if closeBtnVisible}
          <button class="close" onclick={onClose} aria-label="Close">
            <Icon name="X" size={14} />
          </button>
        {/if}
      </header>
    {/if}

    {#if body}
      <div class="body">{@render body()}</div>
    {/if}

    {#if foot}
      <footer class="foot">{@render foot()}</footer>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in srgb, #000 55%, transparent);
    backdrop-filter: blur(2px);
    display: flex;
    justify-content: center;
    z-index: 200;
    /* Mount fade — 120ms */
    animation: backdrop-in 120ms cubic-bezier(0.4, 0, 0.2, 1);
  }
  .align-top    { align-items: flex-start; padding-top: 12vh; }
  .align-center { align-items: center; padding: 5vh 4vw; }

  .modal {
    background: var(--bg-elev-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-3);
    overflow: hidden;
    position: relative;
    /* Mount scale + fade — 180ms */
    animation: modal-in 180ms cubic-bezier(0.16, 1, 0.3, 1);
    transform-origin: center top;
  }
  .modal::before {
    content: "";
    position: absolute; inset: 0;
    background-image: var(--grain);
    opacity: 0.35;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .w-sm { width: min(440px, calc(100vw - 32px)); }
  .w-md { width: min(560px, calc(100vw - 32px)); }
  .w-lg { width: min(1100px, calc(100vw - 32px)); height: 90vh; display: flex; flex-direction: column; }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 16px 18px;
    border-bottom: 1px solid var(--border);
    position: relative; z-index: 1;
  }
  .head :global(h2) {
    margin: 0;
    /* "Code-as-headline" inversion: dialog titles set in Geist Mono.
       Reads as "operation name" — fits a developer tool. */
    font-family: var(--font-mono);
    font-size: var(--fs-lg);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    color: var(--fg);
  }
  .close {
    width: 28px; height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .close:hover { background: var(--bg-elev-3); color: var(--fg); }

  .body {
    padding: 16px 18px;
    position: relative; z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  /* For lg (split-pane) modals, body fills remaining space and scrolls
     internally — children manage their own overflow. */
  .w-lg .body {
    flex: 1;
    min-height: 0;
    padding: 0;
    gap: 0;
  }

  .foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 18px;
    border-top: 1px solid var(--border);
    position: relative; z-index: 1;
  }

  @keyframes backdrop-in {
    from { opacity: 0; }
    to   { opacity: 1; }
  }
  @keyframes modal-in {
    from {
      opacity: 0;
      transform: translateY(-8px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
