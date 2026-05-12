<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    labelSlot,
    optional,
    description,
    error,
    hint,
    control,
    children,
  }: {
    label?: string;
    /** Snippet for richer label content (inline code, links). Overrides label. */
    labelSlot?: Snippet;
    /** Adds an "(optional)" suffix to the label. */
    optional?: boolean;
    description?: string;
    /** When set, replaces the description with a red error message. */
    error?: string | null;
    hint?: string;
    /** Backwards-compat alias for `children`. */
    control?: Snippet;
    children?: Snippet;
  } = $props();
</script>

<label class="field">
  {#if labelSlot}
    <span class="label">{@render labelSlot()}</span>
  {:else if label}
    <span class="label">
      {label}
      {#if optional}<span class="optional">(optional)</span>{/if}
    </span>
  {/if}
  {#if children}
    {@render children()}
  {:else if control}
    {@render control()}
  {/if}
  {#if error}
    <span class="err">{error}</span>
  {:else if hint}
    <span class="hint">{hint}</span>
  {:else if description}
    <span class="desc">{description}</span>
  {/if}
</label>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .label {
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
    font-weight: var(--weight-semibold);
  }
  .label .optional {
    text-transform: none;
    letter-spacing: 0;
    color: var(--fg-faint);
    font-weight: var(--weight-regular);
    margin-left: 4px;
  }
  /* Inline <code> inside a label keeps the body font color but the
     mono family — used by typed-confirm fields like "Type 84a3b21
     to confirm". */
  .label :global(code) {
    text-transform: none;
    letter-spacing: 0;
    font-family: var(--font-mono);
    color: var(--fg);
  }
  .err { color: var(--removed); font-size: var(--fs-xs); }
  .hint, .desc { color: var(--fg-subtle); font-size: var(--fs-xs); line-height: 1.4; }
</style>
