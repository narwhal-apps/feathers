<script lang="ts">
  import { browser } from '$app/environment';
  import { listen } from '@tauri-apps/api/event';
  import '$lib/styles/reset.css';
  import '$lib/styles/tokens.css';
  import '$lib/styles/theme.dark.css';
  import '$lib/styles/theme.light.css';

  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Titlebar from '$lib/components/shell/Titlebar.svelte';
  import DialogHost from '$lib/components/primitives/DialogHost.svelte';
  import { queryClient } from '$lib/query/client';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { theme } from '$lib/stores/theme.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { github } from '$lib/stores/github.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { gitUrlToWebUrl } from '$lib/utils/git-url';

  let { children } = $props();

  // The Settings window runs the same SvelteKit bundle but in a separate
  // Tauri webview — it has its own minimal drag region and doesn't want
  // the main repo titlebar.
  const isSettings = $derived($page.url.pathname.startsWith('/settings'));

  // Same cache key as Titlebar / Pull Requests tab — single fetch shared.
  const remoteUrl = createQuery<string | null>(
    () => repos.activeRepoId ? queryKeys.repoRemoteUrl(repos.activeRepoId) : ['noop'],
    () => repos.activeRepoId
      ? invoke<string | null>('repo_remote_url', { id: repos.activeRepoId })
      : Promise.resolve(null),
  );
  const webBase = $derived(gitUrlToWebUrl(remoteUrl.data ?? null));

  // Mirror the reactive theme store to <html data-theme="...">.
  $effect(() => {
    if (!browser) return;
    document.documentElement.dataset.theme = theme.effective;
  });

  // Load the persisted repo list on app start (was previously done by the
  // sidebar; the titlebar now depends on this for the RepoSwitcher).
  $effect(() => {
    if (!browser) return;
    repos.refresh();
    github.refresh();
    settings.refresh();
  });

  // External changes (terminal commits, branch switches, file edits) come in
  // as `repo_changed` events from the per-repo FS watcher. The watcher tags
  // each batch with a `kind` hint:
  //   - 'workdir' — working-tree edits, `.git/index`, etc. → status+op-state
  //   - 'refs'    — branch/HEAD/MERGE_HEAD/FETCH_HEAD/stash sidecar changes
  //                 → also branches + log + log-unpushed.
  // The watcher already drops pure-noise batches (e.g. `.git/objects/`).
  $effect(() => {
    if (!browser) return;
    const stop = listen<{ id: string; kind: 'refs' | 'workdir' }>('repo_changed', (e) => {
      const { id, kind } = e.payload;
      const keys: (readonly (string | number | null)[])[] = [
        queryKeys.repoStatus(id),
        queryKeys.repoOpState(id),
      ];
      if (kind === 'refs') {
        keys.push(queryKeys.repoBranches(id));
        // ['repo', id, 'log'] is a prefix that matches every paginated
        // log query (queryKeys.repoLog uses a 4th `before` slot).
        keys.push(['repo', id, 'log']);
        keys.push(queryKeys.repoLogUnpushed(id));
      }
      queryClient.invalidateMany(keys);
    });
    return () => { stop.then((unlisten) => unlisten()); };
  });

  // Settings written from the Settings window — re-pull and re-apply.
  $effect(() => {
    if (!browser) return;
    const stop = listen('settings_changed', () => {
      settings.refresh();
    });
    return () => { stop.then((unlisten) => unlisten()); };
  });

  // Window title follows the active repo so macOS Mission Control / Dock
  // tooltips show something useful when several Feathers windows are open.
  $effect(() => {
    if (!browser) return;
    const r = repos.activeRepo;
    document.title = r ? `Feathers — ${r.name}` : 'Feathers';
  });

  // Global keyboard shortcuts. Shortcuts that take focus into a text field
  // (none yet) would need extra "ignore when typing" guards.
  $effect(() => {
    if (!browser) return;
    function isTyping(target: EventTarget | null): boolean {
      if (!(target instanceof HTMLElement)) return false;
      const tag = target.tagName;
      return tag === 'INPUT' || tag === 'TEXTAREA' || target.isContentEditable;
    }
    function onKey(e: KeyboardEvent) {
      // ⌘ required, alt/ctrl never. Shift modifies a few combos. Ignore repeats.
      if (e.repeat || !e.metaKey || e.altKey || e.ctrlKey) return;
      // Don't hijack typing in inputs (e.g. ⌘B = bold in some text fields).
      if (isTyping(e.target)) return;

      const repoId = repos.activeRepoId;
      const k = e.key.toLowerCase();

      // ⌘⇧X — repo-level "open" actions.
      if (e.shiftKey) {
        if (k === 'a' && repoId) {
          e.preventDefault();
          invoke('repo_open_in_editor', { id: repoId }).catch((err) =>
            alert(`Failed to open editor: ${String(err)}`),
          );
          return;
        }
        if (k === 'g' && webBase) {
          e.preventDefault();
          openUrl(webBase).catch(() => {});
          return;
        }
        return;
      }

      // Plain ⌘.
      if (k === '1' && repoId) { e.preventDefault(); goto(`/repo/${repoId}/changes/`); return; }
      if (k === '2' && repoId) { e.preventDefault(); goto(`/repo/${repoId}/history/`); return; }
      if (k === '3' && repoId) { e.preventDefault(); goto(`/repo/${repoId}/pull-requests/`); return; }
      if (k === 'b' && repoId) { e.preventDefault(); ui.openBranchSwitcher(); return; }
      if (k === 'o') { e.preventDefault(); ui.openRepoSwitcher(); return; }
      if (k === 'p' && repoId) { e.preventDefault(); ui.push(); return; }
      if (k === 'r' && repoId) { e.preventDefault(); ui.createPr(); return; }
      if (k === ',') { e.preventDefault(); invoke('open_settings_window').catch(() => {}); return; }
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  });
</script>

{#if !isSettings}
  <Titlebar />
{/if}
<main class="page" class:full={isSettings}>
  {@render children?.()}
</main>

<DialogHost />

<style>
  :global(html, body) { height: 100%; }
  .page {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 40px);
    overflow: hidden;
    background: var(--bg);
    color: var(--fg);
    min-width: 0;
  }
  .page.full { height: 100vh; }
</style>
