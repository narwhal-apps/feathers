<script lang="ts">
  let { keys }: { keys: string[] } = $props();
</script>

<span class="grp">
  {#each keys as k}
    <kbd>{k}</kbd>
  {/each}
</span>

<style>
  .grp { display: inline-flex; gap: 3px; vertical-align: middle; }
  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-bottom-width: 2px;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: var(--weight-semibold);
    line-height: 1;
  }
</style>
