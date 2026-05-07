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
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    border: 1px solid transparent;
    transition: background var(--t-fast),
                border-color var(--t-fast),
                color var(--t-fast),
                transform var(--t-fast),
                box-shadow var(--t-fast);
    white-space: nowrap;
    user-select: none;
  }
  .btn:not(:disabled):active { transform: translateY(0.5px); }
  .btn:disabled { opacity: 0.45; cursor: not-allowed; }

  .btn-sm { height: 26px; padding: 0 11px; font-size: var(--fs-sm); }
  .btn-md { height: 32px; padding: 0 14px; font-size: var(--fs-md); }

  /* Primary — solid teal with DARK text (passes WCAG AA) */
  .btn-primary {
    background: var(--accent-500);
    color: var(--accent-on);
    box-shadow:
      var(--inset-top),
      0 1px 0 rgba(0, 0, 0, 0.15);
  }
  .btn-primary:not(:disabled):hover  { background: var(--accent-400); }
  .btn-primary:not(:disabled):active { background: var(--accent-600); }

  /* Ghost — surface tint with proper border */
  .btn-ghost {
    background: var(--bg-elev-2);
    color: var(--fg);
    border-color: var(--border);
  }
  .btn-ghost:not(:disabled):hover {
    background: var(--bg-elev-3);
    border-color: var(--border-strong);
  }
  .btn-ghost:not(:disabled):active { background: var(--bg-elev-2); }

  /* Danger — desaturated rose, dark text for contrast */
  .btn-danger {
    background: #f87171;
    color: #2a0a0a;
    box-shadow: var(--inset-top), 0 1px 0 rgba(0, 0, 0, 0.15);
  }
  .btn-danger:not(:disabled):hover  { background: #fca5a5; }
  .btn-danger:not(:disabled):active { background: #ef4444; }
</style>
