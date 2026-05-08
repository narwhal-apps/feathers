<script lang="ts">
  type Tone =
    | 'neutral'
    | 'accent'
    | 'add'
    | 'del'
    | 'warn'
    | 'rename'
    | 'count';
  type Variant = 'soft' | 'solid' | 'outline';
  type Size = 'xs' | 'sm';

  let {
    children,
    tone = 'neutral',
    variant = 'soft',
    size = 'sm',
    uppercase = false,
    mono = false,
    title,
  }: {
    children?: import('svelte').Snippet;
    tone?: Tone;
    variant?: Variant;
    size?: Size;
    uppercase?: boolean;
    mono?: boolean;
    title?: string;
  } = $props();
</script>

<span
  class="tag t-{tone} v-{variant} s-{size}"
  class:uppercase
  class:mono
  {title}
>
  {#if children}{@render children()}{/if}
</span>

<style>
  .tag {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid transparent;
    line-height: 1;
    font-weight: var(--weight-bold);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .uppercase {
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
  }
  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  /* Sizes */
  .s-xs {
    height: 16px;
    padding: 0 6px;
    font-size: 9.5px;
    border-radius: var(--r-pill);
  }
  .s-sm {
    height: 20px;
    padding: 0 8px;
    font-size: var(--fs-2xs);
    border-radius: var(--r-pill);
  }

  /* Tone × variant matrix. Soft = tinted bg + tinted border + colored fg. */
  .v-soft.t-neutral { background: var(--bg-elev-2); color: var(--fg-muted); border-color: var(--border); }
  .v-soft.t-accent  { background: var(--accent-bg-medium); color: var(--accent-fg); border-color: var(--accent-bg-strong); }
  .v-soft.t-add     { background: color-mix(in srgb, var(--added) 14%, transparent);   color: var(--added);   border-color: color-mix(in srgb, var(--added) 28%, transparent); }
  .v-soft.t-del     { background: color-mix(in srgb, var(--removed) 14%, transparent); color: var(--removed); border-color: color-mix(in srgb, var(--removed) 28%, transparent); }
  .v-soft.t-warn    { background: var(--warn-bg);   color: var(--warn);   border-color: var(--warn-border); }
  .v-soft.t-rename  { background: var(--rename-bg); color: var(--rename); border-color: var(--rename-border); }

  /* Count = solid neutral pill with tabular nums. */
  .v-soft.t-count {
    background: var(--bg-elev-2);
    color: var(--fg-muted);
    border-color: var(--border);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-bold);
  }

  .v-solid.t-accent { background: var(--accent-500); color: var(--accent-on); }
  .v-solid.t-warn   { background: var(--warn);   color: #1a0e00; }
  .v-solid.t-del    { background: var(--removed); color: #1a0303; }
  .v-solid.t-add    { background: var(--added);  color: #03130d; }

  .v-outline {
    background: transparent;
    color: var(--fg-muted);
    border-color: var(--border);
  }
</style>
