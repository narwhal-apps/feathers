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
    padding: 1px 8px;
    border-radius: 999px;
    font-size: var(--fs-xs);
    font-weight: 600;
    line-height: 1.5;
    font-variant-numeric: tabular-nums;
  }
  .pill-accent  { background: rgba(20, 184, 166, 0.15); color: var(--accent-300); }
  .pill-neutral { background: var(--bg-elev-2);         color: var(--fg-muted); }
  .pill-success { background: rgba(94, 234, 212, 0.15); color: var(--added); }
  .pill-warn    { background: rgba(251, 191, 36, 0.15); color: #fcd34d; }
  .pill-danger  { background: rgba(248, 113, 113, 0.15); color: var(--removed); }
</style>
