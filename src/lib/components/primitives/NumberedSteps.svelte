<script lang="ts">
  import type { Snippet } from 'svelte';

  let { children }: { children?: Snippet } = $props();
</script>

<ol class="steps">
  {@render children?.()}
</ol>

<style>
  .steps {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  /* Children are <NumberedStep> instances; expose the bullet styling via
     :global so the step component itself can stay simple. */
  .steps :global(.step) { display: flex; gap: var(--sp-3); }
  .steps :global(.step > div) { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
  .steps :global(.step strong) {
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    color: var(--fg);
  }
  .steps :global(.step .step-num) {
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-pill);
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-bold);
  }
</style>
