<script lang="ts">
  import { browser } from '$app/environment';
  import '$lib/styles/reset.css';
  import '$lib/styles/tokens.css';
  import '$lib/styles/theme.dark.css';
  import '$lib/styles/theme.light.css';

  import Titlebar from '$lib/components/shell/Titlebar.svelte';
  import Sidebar from '$lib/components/shell/Sidebar.svelte';

  let { children } = $props();

  $effect(() => {
    if (!browser) return;
    const mql = window.matchMedia('(prefers-color-scheme: dark)');
    const apply = (dark: boolean) => {
      document.documentElement.dataset.theme = dark ? 'dark' : 'light';
    };
    apply(mql.matches);
    const onChange = (e: MediaQueryListEvent) => apply(e.matches);
    mql.addEventListener('change', onChange);
    return () => mql.removeEventListener('change', onChange);
  });
</script>

<Titlebar />
<div class="layout">
  <Sidebar />
  <main class="page">
    {@render children?.()}
  </main>
</div>

<style>
  :global(html, body) { height: 100%; }
  .layout {
    display: flex;
    height: calc(100vh - 56px);
    overflow: hidden;
  }
  .page {
    flex: 1;
    overflow: auto;
    background: var(--bg);
    color: var(--fg);
  }
</style>
