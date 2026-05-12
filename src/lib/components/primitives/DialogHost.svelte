<script lang="ts">
  import Modal from './Modal.svelte';
  import Icon from './Icon.svelte';
  import Button from './Button.svelte';
  import { _readState, _resolveConfirm, _dismissToast } from '$lib/utils/dialog.svelte';

  const state = $derived(_readState());
  // The active confirm is the head of the queue (we render one at a time).
  const active = $derived(state.confirms[0] ?? null);
</script>

{#if active}
  {@const opts = active.opts}
  <Modal
    title={opts.title}
    onClose={() => _resolveConfirm(active.id, false)}
    width="sm"
    actions={{
      secondary: {
        label: opts.cancelLabel ?? 'Cancel',
        onclick: () => _resolveConfirm(active.id, false),
      },
      ...(opts.danger
        ? {
            danger: {
              label: opts.confirmLabel ?? 'Confirm',
              onclick: () => _resolveConfirm(active.id, true),
            },
          }
        : {
            primary: {
              label: opts.confirmLabel ?? 'Confirm',
              onclick: () => _resolveConfirm(active.id, true),
            },
          }),
    }}
  >
    {#snippet body()}
      <p class="msg">{opts.message}</p>
    {/snippet}
  </Modal>
{/if}

{#if state.toasts.length > 0}
  <div class="toasts" role="region" aria-label="Notifications">
    {#each state.toasts as t (t.id)}
      <div class="toast {t.kind}" role="status">
        {#if t.kind === 'success'}
          <Icon name="Check" size={14} />
        {:else if t.kind === 'error'}
          <Icon name="AlertTriangle" size={14} />
        {:else}
          <Icon name="Info" size={14} />
        {/if}
        <span class="msg">{t.message}</span>
        {#if t.action}
          <Button
            variant="ghost"
            size="sm"
            label={t.action.label}
            onclick={() => { t.action!.onclick(); _dismissToast(t.id); }}
          />
        {/if}
        <button
          type="button"
          class="dismiss"
          onclick={() => _dismissToast(t.id)}
          aria-label="Dismiss"
          title="Dismiss"
        >
          <Icon name="X" size={12} />
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .msg {
    margin: 0;
    color: var(--fg);
    font-size: var(--fs-sm);
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .toasts {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 300;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
    max-width: min(420px, calc(100vw - 32px));
  }
  .toast {
    pointer-events: auto;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 8px 8px 8px 14px;
    background: var(--bg-elev-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-2);
    color: var(--fg);
    font-size: var(--fs-sm);
    text-align: left;
    animation: toast-in 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }
  .toast .msg { flex: 1; min-width: 0; }
  .toast > :global(svg) { flex-shrink: 0; }
  .toast.success { border-color: color-mix(in srgb, var(--added) 50%, var(--border-strong)); }
  .toast.success > :global(svg) { color: var(--added); }
  .toast.error { border-color: color-mix(in srgb, var(--removed) 50%, var(--border-strong)); }
  .toast.error > :global(svg) { color: var(--removed); }
  .toast.info > :global(svg) { color: var(--accent-fg); }
  .dismiss {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .dismiss:hover { background: var(--bg-elev-2); color: var(--fg); }
  @keyframes toast-in {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
