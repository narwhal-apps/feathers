<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    description,
    control,
  }: {
    label: string;
    description?: string;
    control: Snippet;
  } = $props();
</script>

<div class="row">
  <div class="info">
    <div class="label">{label}</div>
    {#if description}<div class="desc">{description}</div>{/if}
  </div>
  <div class="control">{@render control()}</div>
</div>

<style>
  .row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--sp-4);
    padding: var(--sp-3) 0;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child { border-bottom: none; }
  .info { flex: 1; min-width: 0; }
  .label {
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
    color: var(--fg);
  }
  .desc {
    margin-top: 2px;
    font-size: var(--fs-xs);
    color: var(--fg-subtle);
    line-height: 1.4;
  }
  .control { flex-shrink: 0; }
</style>
