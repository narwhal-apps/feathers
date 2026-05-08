<script lang="ts">
  import { fade } from 'svelte/transition';

  let { message, onDone }: { message: string; onDone: () => void } = $props();

  $effect(() => {
    const t = setTimeout(onDone, 1500);
    return () => clearTimeout(t);
  });
</script>

<div class="toast" role="status" transition:fade={{ duration: 200 }}>
  {message}
</div>

<style>
  .toast {
    position: absolute;
    bottom: var(--sp-4);
    left: 50%;
    transform: translateX(-50%);
    padding: 6px 14px;
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    border-radius: 999px;
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    box-shadow: 0 4px 14px -6px rgba(0, 0, 0, 0.3);
    pointer-events: none;
    z-index: 30;
  }
</style>
