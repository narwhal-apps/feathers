<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import FeatherMark from '$lib/components/shell/FeatherMark.svelte';

  const version = import.meta.env.VITE_APP_VERSION;
  const sha = import.meta.env.VITE_BUILD_SHA;

  function open(url: string): void {
    openUrl(url).catch(() => {});
  }
</script>

<div class="about">
  <div class="hero">
    <div class="mark"><FeatherMark size={64} /></div>
    <div class="word">Feathers</div>
    <div class="version">v{version} · {sha}</div>
    <div class="tagline">Built with Tauri + SvelteKit</div>
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
