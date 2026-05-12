<script lang="ts">
  import { portal } from '$lib/utils/portal';
  import type { Snippet } from 'svelte';

  let {
    open,
    x,
    y,
    onClose,
    children,
  }: {
    open: boolean;
    x: number;
    y: number;
    onClose: () => void;
    children?: Snippet;
  } = $props();

  let menuEl = $state<HTMLDivElement | null>(null);

  /** Adjusted position so the menu stays inside the viewport. */
  let adjusted = $state<{ x: number; y: number } | null>(null);
  $effect(() => {
    if (!open || !menuEl) { adjusted = null; return; }
    // Measure once after mount and shift left/up if we'd overflow.
    const rect = menuEl.getBoundingClientRect();
    let nx = x;
    let ny = y;
    const gap = 8;
    if (nx + rect.width > window.innerWidth - gap) nx = window.innerWidth - rect.width - gap;
    if (ny + rect.height > window.innerHeight - gap) ny = window.innerHeight - rect.height - gap;
    if (nx < gap) nx = gap;
    if (ny < gap) ny = gap;
    adjusted = { x: nx, y: ny };
  });

  $effect(() => {
    if (!open) return;
    function onDocClick(e: MouseEvent) {
      if (menuEl && !menuEl.contains(e.target as Node)) onClose();
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose();
    }
    document.addEventListener('mousedown', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  });

  const pos = $derived(adjusted ?? { x, y });
</script>

{#if open}
  <div
    class="ctx-menu"
    role="menu"
    bind:this={menuEl}
    use:portal
    style="left: {pos.x}px; top: {pos.y}px"
  >
    {@render children?.()}
  </div>
{/if}

<style>
  .ctx-menu {
    position: fixed;
    min-width: 180px;
    max-width: 280px;
    padding: 4px;
    background: var(--bg-elev-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-3);
    z-index: 200;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
