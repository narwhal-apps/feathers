<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import { github } from '$lib/stores/github.svelte';
  import type { DeviceCodeResponse, AppError } from '$lib/types';

  let { onClose }: { onClose: () => void } = $props();

  type Stage = 'starting' | 'waiting' | 'success' | 'error';
  let stage = $state<Stage>('starting');
  let code = $state<DeviceCodeResponse | null>(null);
  let errorMsg = $state<string | null>(null);
  let copied = $state(false);

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

<Modal title="Sign in to GitHub" onClose={onClose} width="sm">
  {#snippet body()}
    {#if stage === 'starting'}
      <p class="hint">Requesting device code…</p>
    {:else if stage === 'waiting' && code}
      <ol class="steps">
        <li>
          <span class="step-num">1</span>
          <div>
            <strong>Open the verification page</strong>
            <a href={code.verification_uri} onclick={(e) => { e.preventDefault(); openUrl(code!.verification_uri); }}>
              {code.verification_uri}
              <Icon name="ExternalLink" size={11} />
            </a>
          </div>
        </li>
        <li>
          <span class="step-num">2</span>
          <div>
            <strong>Enter this code</strong>
            <button class="code" onclick={copyCode} title="Copy to clipboard">
              <span>{code.user_code}</span>
              <Icon name={copied ? 'Check' : 'Copy'} size={12} />
            </button>
            {#if copied}<span class="copied-tag">Copied to clipboard</span>{/if}
          </div>
        </li>
        <li>
          <span class="step-num">3</span>
          <div>
            <strong>Authorize the app</strong>
            <span class="muted">This window will close automatically.</span>
          </div>
        </li>
      </ol>
      <div class="waiting">
        <span class="spinner"></span>
        Waiting for authorization…
      </div>
    {:else if stage === 'success'}
      <div class="success">
        <span class="ok-pill"><Icon name="Check" size={14} /></span>
        <div>
          <strong>Signed in as {github.user?.login ?? 'GitHub user'}</strong>
          <span class="muted">You can now see your pull requests.</span>
        </div>
      </div>
    {:else}
      <div class="error">
        <Icon name="AlertTriangle" size={16} />
        <div>
          <strong>Sign-in failed</strong>
          <span>{errorMsg}</span>
        </div>
      </div>
    {/if}
  {/snippet}

  {#snippet foot()}
    {#if stage === 'error'}
      <Button variant="ghost" label="Close" onclick={onClose} />
    {/if}
  {/snippet}
</Modal>

<style>
  .hint { color: var(--fg-subtle); font-size: var(--fs-sm); margin: 0; }

  .steps { list-style: none; margin: 0 0 14px; padding: 0; display: flex; flex-direction: column; gap: 12px; }
  .steps li { display: flex; gap: 12px; }
  .steps li > div { display: flex; flex-direction: column; gap: 4px; }
  .step-num {
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-pill);
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-bold);
  }
  .steps li strong { font-size: var(--fs-sm); font-weight: var(--weight-semibold); color: var(--fg); }
  .steps li a {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--accent-fg);
    font-size: var(--fs-xs);
    text-decoration: none;
    word-break: break-all;
  }
  .steps li a:hover { text-decoration: underline; }
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
    font-size: 18px;
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
    padding: 10px 12px;
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--fg-muted);
    font-size: var(--fs-xs);
  }
  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid color-mix(in srgb, var(--accent-500) 30%, transparent);
    border-top-color: var(--accent-500);
    border-radius: var(--r-pill);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .success {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    background: color-mix(in srgb, var(--added) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--added) 30%, transparent);
    border-radius: var(--r-md);
  }
  .success > div { display: flex; flex-direction: column; gap: 2px; }
  .success strong { color: var(--fg); font-size: var(--fs-sm); font-weight: var(--weight-semibold); }
  .ok-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: var(--r-pill);
    background: var(--added);
    color: #fff;
  }

  .error {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    background: color-mix(in srgb, var(--removed) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--removed) 35%, transparent);
    border-radius: var(--r-md);
    color: var(--fg);
  }
  .error :global(svg) { color: var(--removed); flex-shrink: 0; margin-top: 2px; }
  .error > div { display: flex; flex-direction: column; gap: 2px; }
  .error strong { font-size: var(--fs-sm); font-weight: var(--weight-semibold); }
  .error span { color: var(--fg-muted); font-size: var(--fs-xs); }

</style>
