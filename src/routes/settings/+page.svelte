<script lang="ts">
  import { browser } from '$app/environment';
  import FeatherMark from '$lib/components/shell/FeatherMark.svelte';
  import SettingsSidebar from '$lib/components/settings/SettingsSidebar.svelte';
  import GeneralPane from '$lib/components/settings/GeneralPane.svelte';
  import AccountPane from '$lib/components/settings/AccountPane.svelte';
  import GitIdentityPane from '$lib/components/settings/GitIdentityPane.svelte';
  import RepositoriesPane from '$lib/components/settings/RepositoriesPane.svelte';
  import AboutPane from '$lib/components/settings/AboutPane.svelte';

  type Pane = 'general' | 'account' | 'git' | 'repos' | 'about';
  const validPanes: Pane[] = ['general', 'account', 'git', 'repos', 'about'];

  function paneFromHash(): Pane {
    if (!browser) return 'general';
    const hash = window.location.hash.replace(/^#/, '');
    return (validPanes as string[]).includes(hash) ? (hash as Pane) : 'general';
  }

  let active = $state<Pane>(paneFromHash());

  function select(p: Pane): void {
    active = p;
    if (browser) window.location.hash = p;
  }

  $effect(() => {
    if (!browser) return;
    function onHash() { active = paneFromHash(); }
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  });

  // Esc closes the window — main close is ⌘W (Tauri default).
  $effect(() => {
    if (!browser) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        import('@tauri-apps/api/webviewWindow').then(({ getCurrentWebviewWindow }) => {
          getCurrentWebviewWindow().close();
        });
      }
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  });
</script>

<div class="root">
  <header class="drag" data-tauri-drag-region>
    <div class="lights" data-tauri-drag-region></div>
    <span class="brand" data-tauri-drag-region title="Feathers"><FeatherMark size={16} /></span>
  </header>
  <div class="settings">
    <SettingsSidebar {active} onSelect={select} />
    <section class="pane">
      {#if active === 'general'}
        <h1>General</h1>
        <GeneralPane />
      {:else if active === 'account'}
        <h1>Account</h1>
        <AccountPane />
      {:else if active === 'git'}
        <h1>Git identity</h1>
        <GitIdentityPane />
      {:else if active === 'repos'}
        <h1>Repositories</h1>
        <RepositoriesPane />
      {:else}
        <h1>About</h1>
        <AboutPane />
      {/if}
    </section>
  </div>
</div>

<style>
  .root {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg);
    color: var(--fg);
    font-family: var(--font-sans);
  }
  .drag {
    display: flex;
    align-items: center;
    height: 40px;
    flex-shrink: 0;
    background: var(--bg-elev-1);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }
  /* Reserve space for the macOS traffic lights (positioned at x=16). */
  .lights { width: 72px; height: 100%; flex-shrink: 0; }
  .brand {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    color: var(--accent-fg);
  }
  .settings { flex: 1; display: flex; min-height: 0; }
  .pane {
    flex: 1;
    padding: var(--sp-5);
    overflow-y: auto;
  }
  h1 {
    margin: 0 0 var(--sp-4);
    font-size: var(--fs-lg);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
  }
</style>
