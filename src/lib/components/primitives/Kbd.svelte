<script lang="ts">
  type Tone = 'default' | 'on-accent';
  let { keys, tone = 'default' }: { keys: string[]; tone?: Tone } = $props();
</script>

<span class="grp tone-{tone}">
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
    border-radius: var(--r-sm);
    font-family: var(--font-sans);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    line-height: 1;
  }
  .tone-default kbd {
    background: var(--bg);
    border: 1px solid var(--border);
    border-bottom-width: 2px;
    color: var(--fg-muted);
  }
  /* Reads on the --accent-500 background of a primary Button — uses a
     translucent darken built off --accent-on. */
  .tone-on-accent kbd {
    background: var(--kbd-on-accent);
    color: var(--accent-on);
  }
</style>
