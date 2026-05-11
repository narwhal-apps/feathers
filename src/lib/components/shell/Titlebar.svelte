<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Button from '$lib/components/primitives/Button.svelte';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import RepoSwitcher from '$lib/components/shell/RepoSwitcher.svelte';
  import BranchSwitcher from '$lib/components/shell/BranchSwitcher.svelte';
  import FeatherMark from '$lib/components/shell/FeatherMark.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { github } from '$lib/stores/github.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { queryClient } from '$lib/query/client';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { gitUrlToWebUrl } from '$lib/utils/git-url';
  import { relTime } from '$lib/utils/time';
  import CreatePRModal from '$lib/components/dialogs/CreatePRModal.svelte';
  import type { AppError, BranchInfo } from '$lib/types';

  const active = $derived(repos.activeRepo);

  // Re-uses the same cache key as BranchSwitcher, so both share one fetch.
  const branches = createQuery<BranchInfo[] | null>(
    () => (active ? queryKeys.repoBranches(active.id) : ['noop']),
    () =>
      active
        ? invoke<BranchInfo[]>('branch_list', { id: active.id })
        : Promise.resolve(null),
  );
  const remoteUrl = createQuery<string | null>(
    () => (active ? queryKeys.repoRemoteUrl(active.id) : ['noop']),
    () =>
      active
        ? invoke<string | null>('repo_remote_url', { id: active.id })
        : Promise.resolve(null),
  );
  const webBase = $derived(gitUrlToWebUrl(remoteUrl.data ?? null));
  const head = $derived(branches.data?.find((b) => b.is_head) ?? null);
  // ahead/behind are null when the local branch has no upstream configured.
  const hasUpstream = $derived(
    head != null && (head.ahead != null || head.behind != null),
  );
  const ahead = $derived(head?.ahead ?? 0);
  const behind = $derived(head?.behind ?? 0);

  // Create-PR button is shown only when origin is on github.com, the
  // branch is published, and we're not sitting on the repo's default
  // branch (no point opening a PR from main → main).
  const isGithubRepo = $derived(
    !!remoteUrl.data && /github\.com[:/]/.test(remoteUrl.data),
  );
  const defaultBranch = $derived(
    branches.data?.find(
      (b) => !b.is_remote && (b.name === 'main' || b.name === 'master'),
    ) ?? null,
  );
  const onDefaultBranch = $derived(
    head != null && defaultBranch != null && head.name === defaultBranch.name,
  );
  const canCreatePr = $derived(
    isGithubRepo && hasUpstream && head != null && !onDefaultBranch,
  );

  let busy = $state<null | 'fetch' | 'pull' | 'push' | 'publish'>(null);
  let createPrOpen = $state(false);

  function startCreatePr() {
    if (!active || !canCreatePr || !head) return;
    if (github.user) {
      // Signed in — use the in-app modal so we can call the GitHub API.
      createPrOpen = true;
    } else {
      // Signed out — fall back to GitHub's web UI for the same branch.
      const url = `${webBase}/pull/new/${encodeURIComponent(head.name)}`;
      openUrl(url);
    }
  }

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
      // Fetch & pull both consult the remote → mark the timestamp.
      if (kind === 'fetch' || kind === 'pull') repos.markFetched(active.id);
      // Narrow per-op so we only refetch what actually changed.
      const id = active.id;
      if (kind === 'fetch') {
        queryClient.invalidateMany([
          queryKeys.repoBranches(id),
          ['repo', id, 'log'],
          queryKeys.repoLogUnpushed(id),
        ]);
      } else if (kind === 'pull') {
        queryClient.invalidateMany([
          queryKeys.repoStatus(id),
          queryKeys.repoBranches(id),
          ['repo', id, 'log'],
          queryKeys.repoLogUnpushed(id),
          queryKeys.repoOpState(id),
          ['repo', id, 'diff'],
        ]);
      } else {
        // push / publish — only the remote-tracking refs and unpushed log change.
        queryClient.invalidateMany([
          queryKeys.repoBranches(id),
          queryKeys.repoLogUnpushed(id),
        ]);
      }
    } catch (err) {
      reportError(`Failed to ${kind}`, err);
    } finally {
      busy = null;
    }
  }

  // Tick once a minute so "5m ago" stays accurate while the user sits idle.
  let now = $state(Date.now());
  $effect(() => {
    const t = setInterval(() => (now = Date.now()), 60_000);
    return () => clearInterval(t);
  });
  const lastFetchedAt = $derived(
    active ? repos.lastFetched[active.id] : undefined,
  );
  const lastFetchedLabel = $derived.by(() => {
    if (!lastFetchedAt) return '';
    // relTime expects seconds-since-epoch.
    void now; // pull `now` into the dep graph so it re-derives every minute.
    return relTime(Math.floor(lastFetchedAt / 1000));
  });

  const doFetch = () =>
    run('fetch', () => invoke('repo_fetch', { id: active!.id }));
  const doPull = () =>
    run('pull', () => invoke('repo_pull', { id: active!.id, rebase: false }));
  const doPush = () =>
    run('push', () => invoke('repo_push', { id: active!.id }));
  const doPublish = () =>
    run('publish', () => invoke('repo_publish', { id: active!.id }));

  // ⌘P / ⌘R signal handlers — same lastReq guard the switchers use.
  let lastPushReq: number | null = null;
  $effect(() => {
    const req = ui.pushRequest;
    if (req != null && req !== lastPushReq) {
      lastPushReq = req;
      // Push is meaningful only when there's something to push.
      if (active && hasUpstream && ahead > 0 && busy == null) doPush();
    }
  });
  let lastCreatePrReq: number | null = null;
  $effect(() => {
    const req = ui.createPrRequest;
    if (req != null && req !== lastCreatePrReq) {
      lastCreatePrReq = req;
      if (canCreatePr) startCreatePr();
    }
  });
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="lights-spacer" data-tauri-drag-region></div>
  <span class="brand" data-tauri-drag-region title="Feathers">
    <FeatherMark size={20} />
  </span>

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
    {#if lastFetchedLabel}
      <span
        class="fetch-meta"
        title="Last fetched at {new Date(lastFetchedAt!).toLocaleString()}"
      >
        fetched {lastFetchedLabel}
      </span>
    {/if}
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
        title={behind === 0
          ? 'Up to date with upstream'
          : `${behind} commit${behind === 1 ? '' : 's'} behind`}
      />
      <Button
        label={busy === 'push' ? 'Pushing…' : 'Push'}
        icon="ArrowUp"
        badge={ahead > 0 ? ahead : undefined}
        variant="primary"
        size="sm"
        disabled={!active || busy !== null || ahead === 0}
        onclick={doPush}
        title={ahead === 0
          ? 'Nothing to push'
          : `Push ${ahead} commit${ahead === 1 ? '' : 's'} (⌘P)`}
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
    {#if canCreatePr && head}
      <Button
        label="Create PR"
        icon="GitPullRequest"
        variant="ghost"
        size="sm"
        disabled={busy !== null}
        onclick={startCreatePr}
        title={github.user
          ? `Open a pull request from ${head.name} (⌘R)`
          : `Open ${head.name} on github.com to create a pull request (⌘R)`}
      />
    {/if}
  </div>
</header>

{#if createPrOpen && active}
  <CreatePRModal id={active.id} onClose={() => (createPrOpen = false)} />
{/if}

<style>
  .titlebar {
    position: relative;
    z-index: 50;
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    height: 40px;
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
    transition:
      color var(--t-fast),
      background var(--t-fast),
      border-color var(--t-fast);
  }
  .repo-link:hover {
    color: var(--accent-fg);
    background: var(--accent-bg-soft);
    border-color: var(--accent-bg-strong);
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
  /* Inline "fetched 5m ago" sitting right before the Fetch button. */
  .fetch-meta {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fg-faint);
    line-height: 1;
    text-transform: lowercase;
    letter-spacing: var(--tracking-tight);
    white-space: nowrap;
    margin-right: 2px;
  }
</style>
