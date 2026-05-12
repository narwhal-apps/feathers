<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import FeatherMark from '$lib/components/shell/FeatherMark.svelte';
  import { updater } from '$lib/stores/updater.svelte';
  import { relTime } from '$lib/utils/time';

  const version = import.meta.env.VITE_APP_VERSION;
  const sha = import.meta.env.VITE_BUILD_SHA;

  function open(url: string): void {
    openUrl(url).catch(() => {});
  }

  /** Human label for the current updater state — drives the status line
   *  next to the "Check for updates" button. */
  const statusLabel = $derived.by(() => {
    const s = updater.state;
    switch (s.status) {
      case 'checking': return 'Checking…';
      case 'available': return `Update available — Feathers v${s.version}`;
      case 'downloading': return `Downloading… ${s.pct}%`;
      case 'ready': return 'Update installed — relaunch to apply';
      case 'up-to-date': return `You're on the latest version · checked ${relTime(Math.floor(s.checkedAt / 1000))}`;
      case 'error': return s.message;
      default: return '';
    }
  });
</script>

<div class="about">
  <div class="hero">
    <div class="mark"><FeatherMark size={64} /></div>
    <div class="word">Feathers</div>
    <div class="version">v{version} · {sha}</div>
    <div class="tagline">Built with Tauri + SvelteKit</div>
  </div>

  <div class="updates">
    <div class="updates-row">
      <Button
        variant="secondary"
        size="md"
        iconLeft="RefreshCw"
        label={updater.state.status === 'checking' ? 'Checking…' : 'Check for updates'}
        loading={updater.state.status === 'checking'}
        disabled={updater.state.status === 'downloading'}
        onclick={() => updater.checkNow()}
      />
      {#if updater.state.status === 'available'}
        <Button
          variant="primary"
          size="md"
          iconLeft="Download"
          label="Install now"
          onclick={() => updater.download()}
        />
      {:else if updater.state.status === 'ready'}
        <Button
          variant="primary"
          size="md"
          iconLeft="RotateCw"
          label="Relaunch"
          onclick={() => import('@tauri-apps/plugin-process').then((m) => m.relaunch())}
        />
      {/if}
    </div>
    {#if statusLabel}
      <div class="status status-{updater.state.status}">{statusLabel}</div>
    {/if}
  </div>

  <div class="links">
    <button class="link" onclick={() => open('https://github.com/narwhal-apps/feathers')}>
      <Icon name="Github" size={14} />
      View on GitHub
    </button>
    <button class="link" onclick={() => open('https://github.com/narwhal-apps/feathers/issues/new')}>
      <Icon name="Bug" size={14} />
      Report an issue
    </button>
  </div>
</div>

<style>
  .about { display: flex; flex-direction: column; align-items: center; gap: var(--sp-5); padding: var(--sp-5) 0; }
  .hero { display: flex; flex-direction: column; align-items: center; gap: 4px; }
  .mark { color: var(--accent-fg); }
  .word {
    font-family: var(--font-mono);
    font-size: var(--fs-xl);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
  }
  .version {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--fg-subtle);
  }
  .tagline {
    margin-top: var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--fg-subtle);
  }

  .updates {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-2);
  }
  .updates-row { display: inline-flex; align-items: center; gap: var(--sp-2); }
  .status {
    font-size: var(--fs-xs);
    color: var(--fg-subtle);
    text-align: center;
  }
  .status-available { color: var(--accent-fg); font-weight: var(--weight-semibold); }
  .status-ready { color: var(--added); font-weight: var(--weight-semibold); }
  .status-error { color: var(--removed); }

  .links { display: flex; gap: var(--sp-3); }
  .link {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 6px 12px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-xs);
    font-weight: var(--weight-medium);
    cursor: pointer;
  }
  .link:hover { background: var(--bg-elev-3); }
</style>
