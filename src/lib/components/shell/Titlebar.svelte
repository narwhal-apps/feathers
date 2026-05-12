<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Button from '$lib/components/primitives/Button.svelte';
  import Avatar from '$lib/components/primitives/Avatar.svelte';
  import RepoSwitcher from '$lib/components/shell/RepoSwitcher.svelte';
  import BranchSwitcher from '$lib/components/shell/BranchSwitcher.svelte';
  import FeatherMark from '$lib/components/shell/FeatherMark.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { github } from '$lib/stores/github.svelte';
  import { queryClient } from '$lib/query/client';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { gitUrlToWebUrl } from '$lib/utils/git-url';
  import { notify } from '$lib/utils/dialog.svelte';
  import { formatError } from '$lib/utils/error';
  import type { AppError } from '$lib/types';

  const active = $derived(repos.activeRepo);

  // Used here only to power the repo external-link button. Action buttons
  // (fetch / pull / push / create PR) live in RepoActions.svelte, which
  // re-queries the same cache key — single fetch, two consumers.
  const remoteUrl = createQuery<string | null>(
    () => (active ? queryKeys.repoRemoteUrl(active.id) : ['noop']),
    () =>
      active
        ? invoke<string | null>('repo_remote_url', { id: active.id })
        : Promise.resolve(null),
  );
  const webBase = $derived(gitUrlToWebUrl(remoteUrl.data ?? null));

  // GitHub user-menu (avatar dropdown). Shown only when signed in.
  let userMenuOpen = $state(false);
  let userMenuEl = $state<HTMLDivElement | null>(null);

  function refreshPRs() {
    if (!active) return;
    queryClient.invalidate(queryKeys.repoPullRequests(active.id));
    userMenuOpen = false;
  }
  async function signOutGithub() {
    try {
      await github.signOut();
      if (active) queryClient.invalidate(queryKeys.repoPullRequests(active.id));
    } catch (err) {
      reportError('Failed to sign out', err);
    } finally {
      userMenuOpen = false;
    }
  }

  $effect(() => {
    if (!userMenuOpen) return;
    function onDocClick(e: MouseEvent) {
      if (userMenuEl && !userMenuEl.contains(e.target as Node)) userMenuOpen = false;
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') userMenuOpen = false;
    }
    document.addEventListener('mousedown', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  });

  function reportError(prefix: string, err: unknown) {
    const e = err as AppError;
    if (e?.kind === 'merge_conflict') {
      const text =
        `${prefix}: merge conflict in ${e.paths.length} file${e.paths.length === 1 ? '' : 's'}.\n\n` +
        e.paths.slice(0, 10).join('\n') +
        (e.paths.length > 10 ? `\n…and ${e.paths.length - 10} more` : '');
      notify(text, { kind: 'error', durationMs: 0 });
      return;
    }
    notify(`${prefix}: ${formatError(err)}`, { kind: 'error', durationMs: 0 });
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="lights-spacer" data-tauri-drag-region></div>
  <span class="brand" data-tauri-drag-region title="Feathers">
    <FeatherMark size={20} />
  </span>

  <div class="cluster" data-tauri-drag-region>
    <RepoSwitcher />
    {#if active && webBase}
      <Button
        variant="ghost"
        size="sm"
        iconOnly="ExternalLink"
        label="Open repository on remote"
        title="Open repository on remote — {webBase}"
        onclick={() => openUrl(webBase)}
      />
    {/if}
    {#if active}
      <span class="separator" aria-hidden="true" data-tauri-drag-region></span>
      <BranchSwitcher />
    {/if}
  </div>

  <div class="spacer" data-tauri-drag-region></div>

  <div class="actions">
    {#if github.user}
      <div class="user-menu" bind:this={userMenuEl}>
        <button
          class="user-trigger"
          onclick={() => (userMenuOpen = !userMenuOpen)}
          aria-haspopup="menu"
          aria-expanded={userMenuOpen}
          title="Signed in as {github.user.login}"
        >
          <Avatar
            name={github.user.name ?? github.user.login}
            email={github.user.login}
            url={github.user.avatar_url}
            size={22}
          />
        </button>
        {#if userMenuOpen}
          <div class="user-pop" role="menu">
            <div class="user-info">
              {#if github.user.name}
                <div class="user-name">{github.user.name}</div>
              {/if}
              <div class="user-login">@{github.user.login}</div>
            </div>
            <div class="user-actions">
              <Button
                variant="ghost"
                size="sm"
                iconLeft="RefreshCw"
                label="Refresh"
                onclick={refreshPRs}
              />
              <Button
                variant="ghost"
                size="sm"
                iconLeft="LogOut"
                label="Sign out"
                onclick={signOutGithub}
              />
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</header>

<style>
  .titlebar {
    position: relative;
    z-index: 50;
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    height: var(--titlebar-h);
    padding: 0 var(--sp-3) 0 0;
    background: linear-gradient(
        180deg,
        color-mix(in srgb, var(--bg-elev-1) 100%, transparent) 0%,
        color-mix(in srgb, var(--bg-elev-1) 96%, transparent) 100%
      ),
      var(--bg-elev-1);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }
  .titlebar::after {
    content: '';
    position: absolute;
    inset: 0;
    background-image: var(--grain);
    opacity: 0.5;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .lights-spacer {
    width: 72px;
    height: 100%;
    flex-shrink: 0;
    position: relative;
    z-index: 1;
  }
  .brand {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    margin-right: 2px;
    position: relative;
    z-index: 1;
  }
  .cluster {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    position: relative;
    z-index: 1;
  }
  .separator {
    width: 4px;
    height: 4px;
    border-radius: var(--r-pill);
    background: var(--fg-faint);
    margin: 0 var(--sp-1);
  }
  .spacer {
    flex: 1;
    position: relative;
    z-index: 1;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    position: relative;
    z-index: 1;
  }

  .user-menu {
    position: relative;
    margin-left: 2px;
    display: flex;
    align-items: center;
  }
  .user-trigger {
    /* Match the sm Button height (26px) so the avatar lines up with
       Push / Pull / Fetch in the same row. flex (not inline-flex) and
       line-height: 1 avoid any text-baseline offset from the inline
       layout context the actions row sits in. */
    width: 26px;
    height: 26px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--r-pill);
    box-sizing: border-box;
    line-height: 1;
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast);
  }
  .user-trigger :global(.avatar) {
    /* The avatar span is inline-flex by default — neutralise its inline
       baseline so it sits dead-centre inside the trigger. */
    display: flex;
    vertical-align: middle;
  }
  .user-trigger:hover { background: var(--bg-elev-2); border-color: var(--border); }
  .user-trigger[aria-expanded="true"] {
    background: var(--bg-elev-2);
    border-color: var(--border-strong);
  }
  .user-pop {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    min-width: 200px;
    padding: 4px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-2);
    z-index: 60;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .user-info {
    padding: 8px 10px 6px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 2px;
  }
  .user-name {
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    line-height: 1.2;
  }
  .user-login {
    color: var(--fg-subtle);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    line-height: 1.3;
    margin-top: 2px;
  }
  .user-actions {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  /* Make the action buttons fill the popover width and left-align labels. */
  .user-actions :global(.btn) {
    width: 100%;
    justify-content: flex-start;
  }
</style>
