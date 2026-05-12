<script lang="ts">
  import type { Snippet } from 'svelte';

  let { n, children }: { n: number; children: Snippet } = $props();
</script>

<li class="step">
  <span class="step-num">{n}</span>
  <div>{@render children()}</div>
</li>
