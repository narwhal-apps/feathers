<script lang="ts">
  let {
    width = $bindable(),
    min = 180,
    max = Infinity,
    onResize,
  }: {
    width: number;
    min?: number;
    max?: number;
    onResize?: (w: number) => void;
  } = $props();

  let dragging = $state(false);
  let handleEl = $state<HTMLDivElement | null>(null);

  function clamp(v: number): number {
    return Math.min(Math.max(v, min), max);
  }

  function onPointerDown(e: PointerEvent) {
    if (!handleEl) return;
    dragging = true;
    handleEl.setPointerCapture(e.pointerId);
    e.preventDefault();
  }
  function onPointerMove(e: PointerEvent) {
    if (!dragging || !handleEl) return;
    // Compute width relative to the handle's parent (the grid container).
    const parent = handleEl.parentElement;
    if (!parent) return;
    const left = parent.getBoundingClientRect().left;
    const next = clamp(e.clientX - left);
    if (next !== width) {
      width = next;
      onResize?.(next);
    }
  }
  function onPointerUp(e: PointerEvent) {
    if (!handleEl) return;
    dragging = false;
    try { handleEl.releasePointerCapture(e.pointerId); } catch {}
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="resizer"
  class:dragging
  bind:this={handleEl}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  role="separator"
  aria-orientation="vertical"
  aria-valuenow={width}
  aria-valuemin={min}
></div>

<style>
  .resizer {
    width: 4px;
    height: 100%;
    cursor: col-resize;
    background: transparent;
    transition: background var(--t-fast);
    position: relative;
    z-index: 5;
    flex-shrink: 0;
  }
  .resizer:hover, .resizer.dragging {
    background: color-mix(in srgb, var(--accent-500) 40%, transparent);
  }
</style>
