<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { page } from '$app/stores';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Avatar from '$lib/components/primitives/Avatar.svelte';
  import SignInModal from '$lib/components/dialogs/SignInModal.svelte';
  import CreatePRModal from '$lib/components/dialogs/CreatePRModal.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import { github } from '$lib/stores/github.svelte';
  import { gitUrlToWebUrl } from '$lib/utils/git-url';
  import { relTime } from '$lib/utils/time';
  import type { PullRequest, AppError } from '$lib/types';

  const id = $derived($page.params.id ?? '');

  const remoteUrl = createQuery<string | null>(
    () => queryKeys.repoRemoteUrl(id),
    () => invoke<string | null>('repo_remote_url', { id }),
  );
  // True only when origin is on github.com.
  const isGithubRepo = $derived.by(() => {
    const u = remoteUrl.data;
    if (!u) return false;
    return /(github\.com[:/])/.test(u);
  });

  const prs = createQuery<PullRequest[]>(
    () =>
      github.user && isGithubRepo
        ? queryKeys.repoPullRequests(id)
        : ['noop'],
    () =>
      github.user && isGithubRepo
        ? invoke<PullRequest[]>('github_list_prs', { id })
        : Promise.resolve([] as PullRequest[]),
  );

  let signInOpen = $state(false);
  let createOpen = $state(false);
  let timestamp = $state(Date.now());
  // Tick once a minute so "5 min ago" stays accurate while you sit on the tab.
  $effect(() => {
    const t = setInterval(() => (timestamp = Date.now()), 60_000);
    return () => clearInterval(t);
  });

  function formatError(err: unknown): string {
    if (typeof err === 'string') return err;
    const e = err as AppError;
    if (typeof e === 'object' && e !== null && 'message' in e) {
      return (e as { message: string }).message;
    }
    return JSON.stringify(err);
  }
  function reportError(prefix: string, err: unknown) {
    alert(`${prefix}: ${formatError(err)}`);
  }

  async function signOut() {
    try {
      await github.signOut();
      queryClient.invalidate(['repo', id, 'pull-requests']);
    } catch (e) {
      reportError('Failed to sign out', e);
    }
  }

  async function refreshPRs() {
    queryClient.invalidate(queryKeys.repoPullRequests(id));
  }

  function open(url: string) { openUrl(url); }
</script>

<div class="page">
  <header class="page-head">
    <div class="title-cluster">
      <h2>Pull requests</h2>
      {#if prs.data && prs.data.length > 0}
        <span class="count">{prs.data.length}</span>
      {/if}
    </div>
    <div class="head-actions">
      {#if github.user}
        <span class="me" title="Signed in as {github.user.login}">
          <Avatar name={github.user.name ?? github.user.login} email={github.user.login} url={github.user.avatar_url} size={20} />
          <span>{github.user.login}</span>
        </span>
        <button
          class="primary-sm"
          onclick={() => (createOpen = true)}
          disabled={!isGithubRepo}
          title={isGithubRepo ? 'Create a pull request from the current branch' : 'Origin is not on github.com'}
        >
          <Icon name="Plus" size={12} /> Create PR
        </button>
        <button class="ghost" onclick={refreshPRs} disabled={!isGithubRepo} title="Refresh">
          <Icon name="RefreshCw" size={12} /> Refresh
        </button>
        <button class="ghost" onclick={signOut} title="Sign out of GitHub">
          Sign out
        </button>
      {/if}
    </div>
  </header>

  {#if !github.hydrated}
    <div class="state hint">Loading…</div>
  {:else if !github.user}
    <div class="state cta">
      <Icon name="GitPullRequest" size={28} />
      <h3>Sign in to GitHub</h3>
      <p>Pull requests, your real avatar, and tokens you didn't have to copy-paste.</p>
      <button class="primary" onclick={() => (signInOpen = true)}>
        <Icon name="LogIn" size={14} />
        Sign in
      </button>
    </div>
  {:else if !isGithubRepo}
    <div class="state hint">
      <Icon name="GitFork" size={20} />
      <p>
        {remoteUrl.data
          ? 'Origin lives somewhere other than github.com.'
          : 'No origin remote — nothing to compare.'}
      </p>
    </div>
  {:else if prs.error}
    <div class="state err">
      <Icon name="AlertTriangle" size={16} />
      <p>{formatError(prs.error)}</p>
      <p class="err-hint">You can still create a PR from the current branch.</p>
    </div>
  {:else if prs.data && prs.data.length === 0}
    <div class="state hint">
      <Icon name="GitPullRequest" size={28} />
      <p>Nothing open. Inbox zero for review.</p>
    </div>
  {:else if prs.data}
    <ul class="prs">
      {#each prs.data as pr}
        {#key timestamp}
          <li>
            <button class="row" onclick={() => open(pr.html_url)} title="Open #{pr.number} on GitHub">
              <span class="pr-icon" class:draft={pr.draft}>
                <Icon name={pr.draft ? 'GitPullRequestDraft' : 'GitPullRequest'} size={16} />
              </span>
              <div class="pr-text">
                <div class="pr-title">
                  <span class="title-text">{pr.title}</span>
                  <span class="pr-num">#{pr.number}</span>
                </div>
                <div class="pr-meta">
                  <Avatar name={pr.user.login} email={pr.user.login} url={pr.user.avatar_url} size={14} />
                  <span class="who">{pr.user.login}</span>
                  <span class="dot">·</span>
                  <span class="branch"><Icon name="GitBranch" size={11} /> {pr.head.ref} → {pr.base.ref}</span>
                  <span class="dot">·</span>
                  <span class="when">updated {relTime(Math.floor(new Date(pr.updated_at).getTime() / 1000))}</span>
                </div>
              </div>
              <Icon name="ExternalLink" size={12} class="open-icon" />
            </button>
          </li>
        {/key}
      {/each}
    </ul>
  {:else if prs.loading}
    <div class="state hint">Loading pull requests…</div>
  {/if}
</div>

{#if signInOpen}
  <SignInModal onClose={() => (signInOpen = false)} />
{/if}
{#if createOpen}
  <CreatePRModal {id} onClose={() => (createOpen = false)} />
{/if}

<style>
  .page {
    height: 100%;
    overflow-y: auto;
    padding: var(--sp-3) var(--sp-4);
    color: var(--fg);
  }
  .page-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: var(--sp-3);
  }
  .title-cluster {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
  }
  .count {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    border-radius: var(--r-pill);
    padding: 1px 8px;
    font-size: var(--fs-2xs);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-bold);
  }
  .head-actions {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .me {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--fg-muted);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
  }
  .ghost {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 26px;
    padding: 0 10px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    transition: color var(--t-fast), border-color var(--t-fast);
  }
  .ghost:hover:not(:disabled) { color: var(--fg); border-color: var(--border-strong); }
  .ghost:disabled { opacity: 0.5; cursor: not-allowed; }
  .primary-sm {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 26px;
    padding: 0 10px;
    background: var(--accent-500);
    color: var(--accent-on);
    border: none;
    border-radius: var(--r-sm);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    transition: background var(--t-fast);
  }
  .primary-sm:hover:not(:disabled) { background: var(--accent-400); }
  .primary-sm:disabled { opacity: 0.5; cursor: not-allowed; }

  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    padding: var(--sp-8) var(--sp-4);
    color: var(--fg-subtle);
    text-align: center;
  }
  .state :global(svg) { color: var(--fg-muted); }
  .state.hint { font-size: var(--fs-sm); }
  .state.err { color: var(--removed); font-size: var(--fs-sm); }
  .state.err .err-hint { color: var(--fg-muted); font-size: var(--fs-xs); margin-top: 4px; }
  .state.cta { padding: var(--sp-8) var(--sp-4); }
  .state.cta h3 {
    margin: 0;
    font-size: var(--fs-md);
    font-weight: var(--weight-semibold);
    color: var(--fg);
  }
  .state.cta p {
    margin: 0;
    max-width: 420px;
    color: var(--fg-muted);
  }
  .primary {
    margin-top: 8px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 14px;
    background: var(--accent-500);
    color: var(--accent-on);
    border: none;
    border-radius: var(--r-md);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    transition: background var(--t-fast);
  }
  .primary:hover { background: var(--accent-400); }

  .prs { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; }
  .prs li { padding: 0; }
  .row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    width: 100%;
    padding: 12px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: inherit;
    text-align: left;
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast);
  }
  .row:hover {
    background: var(--bg-elev-2);
    border-color: var(--border-strong);
  }
  .pr-icon {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--r-sm);
    background: color-mix(in srgb, var(--added) 14%, transparent);
    color: var(--added);
    margin-top: 2px;
  }
  .pr-icon.draft {
    background: color-mix(in srgb, var(--fg-faint) 30%, transparent);
    color: var(--fg-muted);
  }
  .pr-text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
  .pr-title {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
  }
  .title-text {
    flex: 1;
    min-width: 0;
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .pr-num {
    color: var(--fg-subtle);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }
  .pr-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--fg-muted);
    font-size: var(--fs-xs);
    flex-wrap: wrap;
  }
  .pr-meta .who { color: var(--fg); }
  .pr-meta .dot { color: var(--fg-faint); }
  .pr-meta .branch {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
  }
  .pr-meta .branch :global(svg) { color: var(--fg-subtle); }
  .row :global(.open-icon) { color: var(--fg-subtle); margin-top: 2px; flex-shrink: 0; }
  .row:hover :global(.open-icon) { color: var(--accent-fg); }
</style>
