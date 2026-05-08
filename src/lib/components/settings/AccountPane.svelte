<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { github } from '$lib/stores/github.svelte';
  import type { DeviceCodeResponse, AppError } from '$lib/types';

  type Stage = 'idle' | 'starting' | 'waiting' | 'error';
  let stage = $state<Stage>('idle');
  let code = $state<DeviceCodeResponse | null>(null);
  let errorMsg = $state<string | null>(null);
  let copied = $state(false);

  $effect(() => { github.refresh(); });

  async function startSignIn(): Promise<void> {
    stage = 'starting';
    errorMsg = null;
    try {
      const resp = await invoke<DeviceCodeResponse>('github_start_device_flow');
      code = resp;
      stage = 'waiting';
      try { await openUrl(resp.verification_uri); } catch { /* user can click */ }
      try { await navigator.clipboard.writeText(resp.user_code); copied = true; } catch { /* ignore */ }
      await invoke('github_complete_device_flow', {
        deviceCode: resp.device_code,
        interval: resp.interval,
      });
      await github.refresh();
      stage = 'idle';
      code = null;
    } catch (err) {
      const e = err as AppError;
      errorMsg =
        typeof e === 'object' && e !== null && 'message' in e
          ? (e as { message: string }).message
          : JSON.stringify(err);
      stage = 'error';
    }
  }

  async function signOut(): Promise<void> {
    await github.signOut();
  }

  async function copyCode() {
    if (!code) return;
    try { await navigator.clipboard.writeText(code.user_code); copied = true; } catch { /* ignore */ }
  }
</script>

{#if github.user}
  <div class="card">
    <img class="avatar" src={github.user.avatar_url} alt="" />
    <div class="who">
      <div class="login">@{github.user.login}</div>
      {#if github.user.name}<div class="name">{github.user.name}</div>{/if}
    </div>
    <button class="btn ghost" onclick={signOut}>
      <Icon name="LogOut" size={13} />
      Sign out
    </button>
  </div>
{:else if stage === 'waiting' && code}
  <div class="flow">
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
          <button class="code" onclick={copyCode}>
            <span>{code.user_code}</span>
            <Icon name={copied ? 'Check' : 'Copy'} size={12} />
          </button>
        </div>
      </li>
      <li>
        <span class="step-num">3</span>
        <div>
          <strong>Authorize Feathers</strong>
          <span class="muted">This card will update once you approve.</span>
        </div>
      </li>
    </ol>
    <div class="waiting"><span class="spinner"></span> Waiting for authorization…</div>
  </div>
{:else if stage === 'error'}
  <div class="error">
    <Icon name="AlertTriangle" size={16} />
    <div>
      <strong>Sign-in failed</strong>
      <span>{errorMsg}</span>
    </div>
    <button class="btn" onclick={startSignIn}>Try again</button>
  </div>
{:else}
  <div class="signed-out">
    <Icon name="Github" size={32} />
    <p>Sign in to see your pull requests and create new ones.</p>
    <button class="btn primary" onclick={startSignIn}>Sign in with GitHub</button>
  </div>
{/if}

<style>
  .card {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-3);
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
  }
  .avatar { width: 40px; height: 40px; border-radius: 50%; }
  .who { flex: 1; min-width: 0; }
  .login { font-family: var(--font-mono); font-size: var(--fs-sm); font-weight: var(--weight-medium); }
  .name { color: var(--fg-subtle); font-size: var(--fs-xs); }
  .btn {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    background: var(--bg-elev-2);
    color: var(--fg);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    cursor: pointer;
  }
  .btn:hover { background: var(--bg-elev-3); }
  .btn.ghost { background: transparent; }
  .btn.primary { background: var(--accent-bg-medium); color: var(--accent-fg); border-color: transparent; }
  .signed-out {
    display: flex; flex-direction: column; align-items: center; gap: var(--sp-3);
    padding: var(--sp-5);
    text-align: center;
    color: var(--fg-subtle);
  }
  .signed-out p { margin: 0; max-width: 280px; }
  .flow { padding: var(--sp-3) 0; }
  .steps { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: var(--sp-3); }
  .steps li { display: flex; gap: var(--sp-3); }
  .step-num {
    width: 22px; height: 22px;
    display: inline-flex; align-items: center; justify-content: center;
    background: var(--accent-bg-medium); color: var(--accent-fg);
    border-radius: 50%;
    font-size: var(--fs-xs); font-weight: var(--weight-bold);
    flex-shrink: 0;
  }
  .code {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 4px 8px;
    margin-top: 4px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    cursor: pointer;
  }
  .waiting { margin-top: var(--sp-3); display: flex; align-items: center; gap: 8px; color: var(--fg-subtle); font-size: var(--fs-xs); }
  .spinner {
    width: 12px; height: 12px;
    border-radius: 50%;
    border: 2px solid var(--border);
    border-top-color: var(--accent-fg);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .muted { color: var(--fg-subtle); font-size: var(--fs-xs); }
  .error {
    display: flex; align-items: center; gap: var(--sp-3);
    padding: var(--sp-3);
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--fg);
    font-size: var(--fs-sm);
  }
  .error > div { flex: 1; display: flex; flex-direction: column; }
  .error span { color: var(--fg-subtle); font-size: var(--fs-xs); }
</style>
