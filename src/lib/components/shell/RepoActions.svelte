<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Button from '$lib/components/primitives/Button.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { github } from '$lib/stores/github.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { queryClient } from '$lib/query/client';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { gitUrlToWebUrl } from '$lib/utils/git-url';
  import { relTime } from '$lib/utils/time';
  import { notify } from '$lib/utils/dialog.svelte';
  import { formatError } from '$lib/utils/error';
  import CreatePRModal from '$lib/components/dialogs/CreatePRModal.svelte';
  import type { AppError, BranchInfo, PullRequest } from '$lib/types';

  const active = $derived(repos.activeRepo);

  // Same cache keys as the rest of the app, so these queries are free
  // when the data is already loaded by another consumer.
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

  const prs = createQuery<PullRequest[]>(
    () =>
      active && github.user && isGithubRepo
        ? queryKeys.repoPullRequests(active.id)
        : ['noop'],
    () =>
      active && github.user && isGithubRepo
        ? invoke<PullRequest[]>('github_list_prs', { id: active.id })
        : Promise.resolve([] as PullRequest[]),
  );
  const existingPr = $derived(
    head
      ? prs.data?.find((pr) => pr.state === 'open' && pr.head.ref === head.name) ?? null
      : null,
  );

  let busy = $state<null | 'fetch' | 'pull' | 'push' | 'publish'>(null);
  let createPrOpen = $state(false);
  const restrictedRepos = new Set<string>();

  function openPrInBrowser() {
    if (!webBase || !head) return;
    const url = `${webBase}/pull/new/${encodeURIComponent(head.name)}`;
    openUrl(url);
  }

  const prsForbidden = $derived(
    prs.error != null &&
    typeof prs.error === 'object' &&
    'kind' in (prs.error as Record<string, unknown>) &&
    (prs.error as { kind: string }).kind === 'forbidden',
  );

  function startCreatePr() {
    if (!active || !canCreatePr || !head) return;
    if (!github.user || prsForbidden || restrictedRepos.has(active.id)) {
      openPrInBrowser();
    } else {
      createPrOpen = true;
    }
  }

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

  async function run<T>(kind: NonNullable<typeof busy>, fn: () => Promise<T>) {
    if (!active || busy) return;
    busy = kind;
    try {
      await fn();
      if (kind === 'fetch' || kind === 'pull') repos.markFetched(active.id);
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

  // ⌘P / ⌘R signal handlers — fired by the global keymap in +layout.svelte.
  let lastPushReq: number | null = null;
  $effect(() => {
    const req = ui.pushRequest;
    if (req != null && req !== lastPushReq) {
      lastPushReq = req;
      if (!active || busy != null) return;
      if (hasUpstream && ahead > 0) doPush();
      else if (!hasUpstream && head) doPublish();
    }
  });
  let lastCreatePrReq: number | null = null;
  $effect(() => {
    const req = ui.createPrRequest;
    if (req != null && req !== lastCreatePrReq) {
      lastCreatePrReq = req;
      if (existingPr) openUrl(existingPr.html_url).catch(() => {});
      else if (canCreatePr) startCreatePr();
    }
  });
</script>

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
    label="Fetch"
    iconLeft="ArrowDownToLine"
    variant="ghost"
    size="sm"
    loading={busy === 'fetch'}
    disabled={!active || busy !== null}
    onclick={doFetch}
  />
  {#if hasUpstream}
    <Button
      label="Pull"
      iconLeft="ArrowDown"
      badge={behind > 0 ? behind : undefined}
      variant="ghost"
      size="sm"
      loading={busy === 'pull'}
      disabled={!active || busy !== null || behind === 0}
      onclick={doPull}
      title={behind === 0
        ? 'Up to date with upstream'
        : `${behind} commit${behind === 1 ? '' : 's'} behind`}
    />
    <Button
      label="Push"
      iconLeft="ArrowUp"
      badge={ahead > 0 ? ahead : undefined}
      variant="primary"
      size="sm"
      loading={busy === 'push'}
      disabled={!active || busy !== null || ahead === 0}
      onclick={doPush}
      title={ahead === 0
        ? 'Nothing to push'
        : `Push ${ahead} commit${ahead === 1 ? '' : 's'} (⌘P)`}
    />
  {:else if active && head}
    <Button
      label="Publish branch"
      iconLeft="CloudUpload"
      variant="primary"
      size="sm"
      loading={busy === 'publish'}
      disabled={busy !== null}
      onclick={doPublish}
      title="Push {head.name} to origin and set it as the upstream (⌘P)"
    />
  {/if}
  {#if canCreatePr && head}
    {#if existingPr}
      <Button
        label="Show PR"
        iconLeft="GitPullRequest"
        variant="ghost"
        size="sm"
        disabled={busy !== null}
        onclick={() => openUrl(existingPr.html_url).catch(() => {})}
        title="Open #{existingPr.number}: {existingPr.title} (⌘R)"
      />
    {:else}
      <Button
        label="Create PR"
        iconLeft="GitPullRequest"
        variant="ghost"
        size="sm"
        disabled={busy !== null}
        onclick={startCreatePr}
        title={github.user
          ? `Open a pull request from ${head.name} (⌘R)`
          : `Open ${head.name} on github.com to create a pull request (⌘R)`}
      />
    {/if}
  {/if}
</div>

{#if createPrOpen && active}
  <CreatePRModal
    id={active.id}
    onClose={() => (createPrOpen = false)}
    onRestricted={() => { if (active) restrictedRepos.add(active.id); }}
  />
{/if}

<style>
  .actions {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }
  /* Inline "fetched 5m ago" sitting right before the Fetch button. */
  .fetch-meta {
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    color: var(--fg-faint);
    line-height: 1;
    text-transform: lowercase;
    letter-spacing: var(--tracking-tight);
    white-space: nowrap;
    margin-right: 2px;
  }
</style>
