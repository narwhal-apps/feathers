<script lang="ts">
  type Variant = 'primary' | 'ghost' | 'danger';
  type Size = 'sm' | 'md';

  let {
    label,
    variant = 'primary',
    size = 'md',
    disabled = false,
    onclick,
    type = 'button',
  }: {
    label: string;
    variant?: Variant;
    size?: Size;
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
    type?: 'button' | 'submit';
  } = $props();
</script>

<button
  class="btn btn-{variant} btn-{size}"
  {disabled}
  {type}
  {onclick}
>
  {label}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    border-radius: var(--r-md);
    font-weight: 600;
    transition: background var(--t-fast), border-color var(--t-fast), color var(--t-fast);
    border: 1px solid transparent;
  }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-sm { padding: 4px 10px; font-size: var(--fs-sm); }
  .btn-md { padding: 6px 14px; font-size: var(--fs-md); }

  .btn-primary { background: var(--accent-500); color: white; }
  .btn-primary:not(:disabled):hover  { background: var(--accent-600); }
  .btn-primary:not(:disabled):active { background: var(--accent-700); }

  .btn-ghost { background: transparent; color: var(--fg); border-color: var(--border); }
  .btn-ghost:not(:disabled):hover  { background: var(--bg-elev-2); }

  .btn-danger { background: #b3261e; color: white; }
  .btn-danger:not(:disabled):hover { background: #9a1e17; }
</style>
