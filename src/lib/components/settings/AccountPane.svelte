<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import Avatar from '$lib/components/primitives/Avatar.svelte';
  import Spinner from '$lib/components/primitives/Spinner.svelte';
  import Banner from '$lib/components/primitives/Banner.svelte';
  import NumberedSteps from '$lib/components/primitives/NumberedSteps.svelte';
  import NumberedStep from '$lib/components/primitives/NumberedStep.svelte';
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
    <Avatar
      name={github.user.name ?? github.user.login}
      email={github.user.login}
      url={github.user.avatar_url}
      size={40}
    />
    <div class="who">
      <div class="login">@{github.user.login}</div>
      {#if github.user.name}<div class="name">{github.user.name}</div>{/if}
    </div>
    <Button variant="ghost" iconLeft="LogOut" label="Sign out" onclick={signOut} />
  </div>
{:else if stage === 'starting'}
  <div class="working">
    <Spinner size="md" />
    <span>Requesting device code…</span>
  </div>
{:else if stage === 'waiting' && code}
  <div class="flow">
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
      </NumberedStep>
      <NumberedStep n={3}>
        <strong>Authorize Feathers</strong>
        <span class="muted">This card will update once you approve.</span>
      </NumberedStep>
    </NumberedSteps>
    <div class="waiting"><Spinner size="xs" /> Waiting for authorization…</div>
  </div>
{:else if stage === 'error'}
  <Banner tone="error" title="Sign-in failed">
    {errorMsg}
    {#snippet actions()}
      <Button onclick={startSignIn} label="Try again" />
    {/snippet}
  </Banner>
{:else}
  <div class="signed-out">
    <Icon name="Github" size={32} />
    <p>Sign in to see your pull requests and create new ones.</p>
    <Button variant="primary" iconLeft="LogIn" label="Sign in with GitHub" onclick={startSignIn} />
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
  .who { flex: 1; min-width: 0; }
  .login { font-family: var(--font-mono); font-size: var(--fs-sm); font-weight: var(--weight-medium); }
  .name { color: var(--fg-subtle); font-size: var(--fs-xs); }
  .signed-out {
    display: flex; flex-direction: column; align-items: center; gap: var(--sp-3);
    padding: var(--sp-5);
    text-align: center;
    color: var(--fg-subtle);
  }
  .signed-out p { margin: 0; max-width: 280px; }
  .working {
    display: flex; align-items: center; gap: var(--sp-3);
    padding: var(--sp-4);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
  }
  .flow { padding: var(--sp-3) 0; }
  .flow a {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--accent-fg);
    font-size: var(--fs-xs);
    text-decoration: none;
  }
  .flow a:hover { text-decoration: underline; }
  .code {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 4px 8px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--fg);
    cursor: pointer;
  }
  .waiting {
    margin-top: var(--sp-3);
    display: flex; align-items: center; gap: 8px;
    color: var(--fg-subtle); font-size: var(--fs-xs);
  }
  .muted { color: var(--fg-subtle); font-size: var(--fs-xs); }
</style>
