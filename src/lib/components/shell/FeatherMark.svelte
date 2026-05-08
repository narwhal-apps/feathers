<script lang="ts">
  let { size = 22 }: { size?: number } = $props();
</script>

<!--
  Custom feather mark — a single quill stroke with diagonal vanes.
  Hand-tuned (not lucide) so the app has its own glyph in moments
  that need branding.
-->
<svg
  class="feather"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.6"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <!-- Quill spine -->
  <path d="M5 20 L19 6" />
  <!-- Vanes (top → bottom) -->
  <path d="M9 16 L15 16" />
  <path d="M11 14 L17 14" />
  <path d="M13 12 L18.5 12" />
  <path d="M15 10 L19 10" />
  <path d="M16.5 8.5 L19 8.5" />
  <!-- Tip flourish -->
  <path d="M19 6 q1.6 -1 2.4 -2.4" stroke-width="1.3" />
</svg>

<style>
  .feather {
    color: var(--accent-300);
    filter: drop-shadow(0 0 6px color-mix(in srgb, var(--accent-500) 28%, transparent));
    transition: color var(--t-med), transform var(--t-med);
  }
  .feather:hover { color: var(--accent-400); transform: rotate(-4deg); }
</style>
