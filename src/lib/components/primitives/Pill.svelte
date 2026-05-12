<script lang="ts">
  type Tone = 'accent' | 'neutral' | 'success' | 'warn' | 'danger';

  let {
    label,
    tone = 'accent',
  }: { label: string; tone?: Tone } = $props();
</script>

<span class="pill pill-{tone}">{label}</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    padding: 2px 9px;
    border-radius: var(--r-pill);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    line-height: 1.5;
    font-variant-numeric: tabular-nums;
    border: 1px solid transparent;
  }
  .pill-accent  {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    border-color: var(--accent-bg-strong);
  }
  .pill-neutral {
    background: var(--bg-elev-2);
    color: var(--fg-muted);
    border-color: var(--border);
  }
  .pill-success {
    background: color-mix(in srgb, var(--added) 14%, transparent);
    color: var(--added-text);
    border-color: color-mix(in srgb, var(--added) 28%, transparent);
  }
  .pill-warn    {
    background: var(--warn-bg);
    color: var(--warn);
    border-color: var(--warn-border);
  }
  .pill-danger  {
    background: color-mix(in srgb, var(--removed) 14%, transparent);
    color: var(--removed-text);
    border-color: color-mix(in srgb, var(--removed) 28%, transparent);
  }
</style>
