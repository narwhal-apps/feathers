<script lang="ts">
  type Size = 'xs' | 'sm' | 'md';

  let {
    size = 'sm',
    label,
  }: {
    size?: Size;
    /** Aria label — defaults to "Loading". Set to '' to hide. */
    label?: string;
  } = $props();

  const dim = $derived(size === 'xs' ? 10 : size === 'sm' ? 12 : 16);
  const stroke = $derived(size === 'md' ? 2 : 1.5);
</script>

<span
  class="spinner"
  style:width="{dim}px"
  style:height="{dim}px"
  style:border-width="{stroke}px"
  role="status"
  aria-label={label ?? 'Loading'}
></span>

<style>
  .spinner {
    display: inline-block;
    border-style: solid;
    border-color: currentColor;
    border-right-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    opacity: 0.85;
    flex-shrink: 0;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
