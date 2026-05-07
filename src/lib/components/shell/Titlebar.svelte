<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Button from '$lib/components/primitives/Button.svelte';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import RepoSwitcher from '$lib/components/shell/RepoSwitcher.svelte';
  import BranchSwitcher from '$lib/components/shell/BranchSwitcher.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { queryClient } from '$lib/query/client';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { gitUrlToWebUrl } from '$lib/utils/git-url';
  import type { AppError, BranchInfo } from '$lib/types';

  const active = $derived(repos.activeRepo);

  // Re-uses the same cache key as BranchSwitcher, so both share one fetch.
  const branches = createQuery<BranchInfo[] | null>(
    () => active ? queryKeys.repoBranches(active.id) : ['noop'],
    () => active ? invoke<BranchInfo[]>('branch_list', { id: active.id }) : Promise.resolve(null),
  );
  const remoteUrl = createQuery<string | null>(
    () => active ? queryKeys.repoRemoteUrl(active.id) : ['noop'],
    () => active ? invoke<string | null>('repo_remote_url', { id: active.id }) : Promise.resolve(null),
  );
  const webBase = $derived(gitUrlToWebUrl(remoteUrl.data ?? null));
  const head = $derived(branches.data?.find((b) => b.is_head) ?? null);
  // ahead/behind are null when the local branch has no upstream configured.
  const hasUpstream = $derived(head != null && (head.ahead != null || head.behind != null));
  const ahead  = $derived(head?.ahead  ?? 0);
  const behind = $derived(head?.behind ?? 0);

  let busy = $state<null | 'fetch' | 'pull' | 'push' | 'publish'>(null);

  function reportError(prefix: string, err: unknown) {
    const e = err as AppError;
    if (e?.kind === 'merge_conflict') {
      alert(
        `${prefix}: merge conflict in ${e.paths.length} file${e.paths.length === 1 ? '' : 's'}.\n\n` +
          e.paths.slice(0, 10).join('\n') +
          (e.paths.length > 10 ? `\n…and ${e.paths.length - 10} more` : ''),
      );
      return;
    }
    const msg =
      typeof e === 'object' && e !== null && 'message' in e
        ? (e as { message: string }).message
        : JSON.stringify(err);
    alert(`${prefix}: ${msg}`);
  }

  async function run<T>(kind: NonNullable<typeof busy>, fn: () => Promise<T>) {
    if (!active || busy) return;
    busy = kind;
    try {
      await fn();
      // Anything that touches refs can change branches/log/status.
      queryClient.invalidate(['repo', active.id]);
    } catch (err) {
      reportError(`Failed to ${kind}`, err);
    } finally {
      busy = null;
    }
  }

  const doFetch   = () => run('fetch',   () => invoke('repo_fetch',   { id: active!.id }));
  const doPull    = () => run('pull',    () => invoke('repo_pull',    { id: active!.id, rebase: false }));
  const doPush    = () => run('push',    () => invoke('repo_push',    { id: active!.id }));
  const doPublish = () => run('publish', () => invoke('repo_publish', { id: active!.id }));
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="lights-spacer" data-tauri-drag-region></div>

  <div class="cluster" data-tauri-drag-region>
    <RepoSwitcher />
    {#if active && webBase}
      <button
        class="repo-link"
        onclick={() => openUrl(webBase)}
        title="Open repository on remote — {webBase}"
        aria-label="Open repository on remote"
      >
        <Icon name="ExternalLink" size={12} />
      </button>
    {/if}
    {#if active}
      <span class="separator" aria-hidden="true" data-tauri-drag-region></span>
      <BranchSwitcher />
    {/if}
  </div>

  <div class="spacer" data-tauri-drag-region></div>

  <div class="actions">
    <Button
      label={busy === 'fetch' ? 'Fetching…' : 'Fetch'}
      icon="ArrowDownToLine"
      variant="ghost"
      size="sm"
      disabled={!active || busy !== null}
      onclick={doFetch}
    />
    {#if hasUpstream}
      <Button
        label={busy === 'pull' ? 'Pulling…' : 'Pull'}
        icon="ArrowDown"
        badge={behind > 0 ? behind : undefined}
        variant="ghost"
        size="sm"
        disabled={!active || busy !== null || behind === 0}
        onclick={doPull}
        title={behind === 0 ? 'Up to date with upstream' : `${behind} commit${behind === 1 ? '' : 's'} behind`}
      />
      <Button
        label={busy === 'push' ? 'Pushing…' : 'Push'}
        icon="ArrowUp"
        badge={ahead > 0 ? ahead : undefined}
        variant="primary"
        size="sm"
        disabled={!active || busy !== null || ahead === 0}
        onclick={doPush}
        title={ahead === 0 ? 'Nothing to push' : `${ahead} commit${ahead === 1 ? '' : 's'} ahead`}
      />
    {:else if active && head}
      <Button
        label={busy === 'publish' ? 'Publishing…' : 'Publish branch'}
        icon="CloudUpload"
        variant="primary"
        size="sm"
        disabled={busy !== null}
        onclick={doPublish}
        title="Push {head.name} to origin and set it as the upstream"
      />
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
    height: 56px;
    padding: 0 var(--sp-4) 0 0;
    background:
      linear-gradient(180deg,
        color-mix(in srgb, var(--bg-elev-1) 100%, transparent) 0%,
        color-mix(in srgb, var(--bg-elev-1) 96%, transparent) 100%),
      var(--bg-elev-1);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }
  .titlebar::after {
    content: "";
    position: absolute;
    inset: 0;
    background-image: var(--grain);
    opacity: 0.5;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .lights-spacer { width: 80px; height: 100%; flex-shrink: 0; position: relative; z-index: 1; }
  .cluster { display: flex; align-items: center; gap: var(--sp-2); position: relative; z-index: 1; }
  .separator {
    width: 4px; height: 4px;
    border-radius: var(--r-pill);
    background: var(--fg-faint);
    margin: 0 var(--sp-1);
  }
  .repo-link {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    transition: color var(--t-fast), background var(--t-fast), border-color var(--t-fast);
  }
  .repo-link:hover {
    color: var(--accent-fg);
    background: var(--accent-bg-soft);
    border-color: var(--accent-bg-strong);
  }
  .spacer { flex: 1; position: relative; z-index: 1; }
  .actions { display: flex; gap: var(--sp-2); position: relative; z-index: 1; }
</style>
