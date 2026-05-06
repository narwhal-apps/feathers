<script lang="ts">
  import * as icons from '@lucide/svelte';

  let {
    name,
    size = 16,
    strokeWidth = 1.75,
    class: className = '',
  }: {
    name: string;
    size?: number;
    strokeWidth?: number;
    class?: string;
  } = $props();

  const Component = $derived(
    ((icons as Record<string, unknown>)[name] ??
      (icons as Record<string, unknown>).HelpCircle) as any,
  );
</script>

{#if Component}
  <Component {size} stroke-width={strokeWidth} class={className} />
{/if}
