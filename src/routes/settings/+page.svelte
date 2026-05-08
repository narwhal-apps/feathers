<script lang="ts">
  import { browser } from '$app/environment';
  import SettingsSidebar from '$lib/components/settings/SettingsSidebar.svelte';
  import GeneralPane from '$lib/components/settings/GeneralPane.svelte';

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

<div class="settings">
  <SettingsSidebar {active} onSelect={select} />
  <section class="pane">
    {#if active === 'general'}
      <h1>General</h1>
      <GeneralPane />
    {:else if active === 'account'}
      <h1>Account</h1>
      <p class="placeholder">Coming up.</p>
    {:else if active === 'git'}
      <h1>Git identity</h1>
      <p class="placeholder">Coming up.</p>
    {:else if active === 'repos'}
      <h1>Repositories</h1>
      <p class="placeholder">Coming up.</p>
    {:else}
      <h1>About</h1>
      <p class="placeholder">Coming up.</p>
    {/if}
  </section>
</div>

<style>
  .settings {
    display: flex;
    height: 100vh;
    background: var(--bg);
    color: var(--fg);
    font-family: var(--font-sans);
    /* Pad the top so content clears the overlay traffic-light strip. */
    padding-top: 28px;
  }
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
  .placeholder { color: var(--fg-subtle); font-size: var(--fs-sm); }
</style>
