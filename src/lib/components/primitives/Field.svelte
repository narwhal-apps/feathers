<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    optional,
    description,
    error,
    hint,
    control,
    children,
  }: {
    label?: string;
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
  {#if label}
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
  .err { color: var(--removed); font-size: var(--fs-xs); }
  .hint, .desc { color: var(--fg-subtle); font-size: var(--fs-xs); line-height: 1.4; }
</style>
