<script lang="ts">
  import Modal from './Modal.svelte';
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';
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
  >
    {#snippet body()}
      <p class="msg">{opts.message}</p>
    {/snippet}
    {#snippet foot()}
      <Button
        label={opts.cancelLabel ?? 'Cancel'}
        variant="ghost"
        size="md"
        onclick={() => _resolveConfirm(active.id, false)}
      />
      <Button
        label={opts.confirmLabel ?? 'Confirm'}
        variant={opts.danger ? 'danger' : 'primary'}
        size="md"
        onclick={() => _resolveConfirm(active.id, true)}
      />
    {/snippet}
  </Modal>
{/if}

{#if state.toasts.length > 0}
  <div class="toasts" role="region" aria-label="Notifications">
    {#each state.toasts as t (t.id)}
      <button
        class="toast {t.kind}"
        type="button"
        onclick={() => _dismissToast(t.id)}
        title={t.durationMs === 0 ? 'Click to dismiss' : ''}
      >
        {#if t.kind === 'success'}
          <Icon name="Check" size={14} />
        {:else if t.kind === 'error'}
          <Icon name="AlertTriangle" size={14} />
        {:else}
          <Icon name="Info" size={14} />
        {/if}
        <span>{t.message}</span>
      </button>
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
    gap: 8px;
    padding: 10px 14px;
    background: var(--bg-elev-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-2);
    color: var(--fg);
    font-size: var(--fs-sm);
    text-align: left;
    cursor: pointer;
    animation: toast-in 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }
  .toast :global(svg) { flex-shrink: 0; }
  .toast.success { border-color: color-mix(in srgb, var(--added) 50%, var(--border-strong)); }
  .toast.success :global(svg) { color: var(--added); }
  .toast.error { border-color: color-mix(in srgb, var(--removed) 50%, var(--border-strong)); }
  .toast.error :global(svg) { color: var(--removed); }
  .toast.info :global(svg) { color: var(--accent-fg); }
  @keyframes toast-in {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
