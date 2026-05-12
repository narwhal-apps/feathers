<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import Spinner from '$lib/components/primitives/Spinner.svelte';
  import Banner from '$lib/components/primitives/Banner.svelte';
  import NumberedSteps from '$lib/components/primitives/NumberedSteps.svelte';
  import NumberedStep from '$lib/components/primitives/NumberedStep.svelte';
  import { github } from '$lib/stores/github.svelte';
  import type { DeviceCodeResponse, AppError } from '$lib/types';

  let { onClose }: { onClose: () => void } = $props();

  type Stage = 'starting' | 'waiting' | 'success' | 'error';
  let stage = $state<Stage>('starting');
  let code = $state<DeviceCodeResponse | null>(null);
  let errorMsg = $state<string | null>(null);
  let copied = $state(false);

  const footerActions = $derived(
    stage === 'error'
      ? { secondary: { label: 'Close', onclick: onClose } }
      : undefined,
  );

  $effect(() => {
    let cancelled = false;
    (async () => {
      try {
        const resp = await invoke<DeviceCodeResponse>('github_start_device_flow');
        if (cancelled) return;
        code = resp;
        stage = 'waiting';
        try { await openUrl(resp.verification_uri); } catch { /* user can click */ }
        try { await navigator.clipboard.writeText(resp.user_code); copied = true; } catch { /* ignore */ }
        await invoke('github_complete_device_flow', {
          deviceCode: resp.device_code,
          interval: resp.interval,
        });
        if (cancelled) return;
        await github.refresh();
        stage = 'success';
        setTimeout(() => { if (!cancelled) onClose(); }, 1200);
      } catch (err) {
        if (cancelled) return;
        const e = err as AppError;
        errorMsg =
          typeof e === 'object' && e !== null && 'message' in e
            ? (e as { message: string }).message
            : JSON.stringify(err);
        stage = 'error';
      }
    })();
    return () => { cancelled = true; };
  });

  async function copyCode() {
    if (!code) return;
    try {
      await navigator.clipboard.writeText(code.user_code);
      copied = true;
    } catch { /* ignore */ }
  }
</script>

<Modal title="Sign in to GitHub" onClose={onClose} width="sm" actions={footerActions}>
  {#snippet body()}
    {#if stage === 'starting'}
      <p class="hint">Requesting device code…</p>
    {:else if stage === 'waiting' && code}
      <NumberedSteps>
        <NumberedStep n={1}>
          <strong>Open the verification page</strong>
          <a
            href={code.verification_uri}
            onclick={(e) => { e.preventDefault(); openUrl(code!.verification_uri); }}
          >
            {code.verification_uri}
            <Icon name="ExternalLink" size={11} />
          </a>
        </NumberedStep>
        <NumberedStep n={2}>
          <strong>Enter this code</strong>
          <button class="code" onclick={copyCode} title="Copy to clipboard">
            <span>{code.user_code}</span>
            <Icon name={copied ? 'Check' : 'Copy'} size={12} />
          </button>
          {#if copied}<span class="copied-tag">Copied to clipboard</span>{/if}
        </NumberedStep>
        <NumberedStep n={3}>
          <strong>Authorize the app</strong>
          <span class="muted">This window will close automatically.</span>
        </NumberedStep>
      </NumberedSteps>
      <div class="waiting">
        <Spinner size="xs" />
        Waiting for authorization…
      </div>
    {:else if stage === 'success'}
      <Banner tone="success" title="Signed in as {github.user?.login ?? 'GitHub user'}">
        You can now see your pull requests.
      </Banner>
    {:else}
      <Banner tone="error" title="Sign-in failed">{errorMsg}</Banner>
    {/if}
  {/snippet}
</Modal>

<style>
  .hint { color: var(--fg-subtle); font-size: var(--fs-sm); margin: 0; }

  /* Numbered-step content slots */
  :global(.steps a) {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--accent-fg);
    font-size: var(--fs-xs);
    text-decoration: none;
    word-break: break-all;
  }
  :global(.steps a:hover) { text-decoration: underline; }
  .muted { color: var(--fg-subtle); font-size: var(--fs-xs); }

  .code {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    align-self: flex-start;
    padding: 8px 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: var(--fs-xl);
    font-weight: var(--weight-bold);
    letter-spacing: 0.15em;
    cursor: pointer;
    transition: border-color var(--t-fast);
  }
  .code:hover { border-color: var(--border-strong); }
  .code :global(svg) { color: var(--fg-subtle); }
  .copied-tag { color: var(--added); font-size: var(--fs-2xs); }

  .waiting {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: var(--sp-3);
    padding: 10px 12px;
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--fg-muted);
    font-size: var(--fs-xs);
  }
</style>
