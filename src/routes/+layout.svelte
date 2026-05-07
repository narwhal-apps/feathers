<script lang="ts">
  import { browser } from '$app/environment';
  import { listen } from '@tauri-apps/api/event';
  import '$lib/styles/reset.css';
  import '$lib/styles/tokens.css';
  import '$lib/styles/theme.dark.css';
  import '$lib/styles/theme.light.css';

  import Titlebar from '$lib/components/shell/Titlebar.svelte';
  import { queryClient } from '$lib/query/client';
  import { theme } from '$lib/stores/theme.svelte';
  import { repos } from '$lib/stores/repos.svelte';

  let { children } = $props();

  // Mirror the reactive theme store to <html data-theme="...">.
  $effect(() => {
    if (!browser) return;
    document.documentElement.dataset.theme = theme.value;
  });

  // Load the persisted repo list on app start (was previously done by the
  // sidebar; the titlebar now depends on this for the RepoSwitcher).
  $effect(() => {
    if (!browser) return;
    repos.refresh();
  });

  // External changes (terminal commits, branch switches, file edits) come in
  // as `repo_changed` events from the per-repo FS watcher. Invalidate that
  // repo's queries so any visible data refreshes.
  $effect(() => {
    if (!browser) return;
    const stop = listen<{ id: string }>('repo_changed', (e) => {
      queryClient.invalidate(['repo', e.payload.id]);
    });
    return () => { stop.then((unlisten) => unlisten()); };
  });
</script>

<Titlebar />
<main class="page">
  {@render children?.()}
</main>

<style>
  :global(html, body) { height: 100%; }
  .page {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 56px);
    overflow: hidden;
    background: var(--bg);
    color: var(--fg);
    min-width: 0;
  }
</style>
