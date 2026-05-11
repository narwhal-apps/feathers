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
  import { formatError } from '$lib/utils/error';
  import type { PullRequest } from '$lib/types';

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

  function reportError(prefix: string, err: unknown) {
    alert(`${prefix}: ${formatError(err)}`);
  }

  // Split a string into alternating text + URL segments so the template can
  // render each URL as an openUrl-on-click link. Matches http(s) URLs only.
  function splitUrls(text: string): Array<{ text: string; href?: string }> {
    const re = /https?:\/\/[^\s)`'"]+/g;
    const out: Array<{ text: string; href?: string }> = [];
    let last = 0;
    for (const m of text.matchAll(re)) {
      const i = m.index ?? 0;
      if (i > last) out.push({ text: text.slice(last, i) });
      out.push({ text: m[0], href: m[0] });
      last = i + m[0].length;
    }
    if (last < text.length) out.push({ text: text.slice(last) });
    return out.length ? out : [{ text }];
  }

  // Detect the most common org-restriction message and pull out the org name
  // so we can offer a one-click "Approve in GitHub" action that opens the
  // org's OAuth policies page directly.
  function oauthRestrictionOrg(message: string): string | null {
    const m = message.match(/the\s+`?([^`\s]+)`?\s+organization has enabled OAuth App/i);
    return m?.[1] ?? null;
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
      <span class="cta-icon" aria-hidden="true">
        <Icon name="GitPullRequest" size={26} />
      </span>
      <h3>Sign in to GitHub</h3>
      <p>Pull requests, your real avatar, and tokens you didn't have to copy-paste.</p>
      <button class="primary" onclick={() => (signInOpen = true)}>
        <Icon name="LogIn" size={14} />
        <span>Sign in</span>
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
    {@const errMsg = formatError(prs.error)}
    {@const restrictedOrg = oauthRestrictionOrg(errMsg)}
    <div class="state err">
      <span class="err-icon" aria-hidden="true">
        <Icon name="AlertTriangle" size={22} />
      </span>
      <h3 class="err-title">
        {restrictedOrg ? `${restrictedOrg} restricts OAuth apps` : "Couldn't load pull requests"}
      </h3>
      <p class="err-msg">
        {#each splitUrls(errMsg) as seg}
          {#if seg.href}
            <a href={seg.href} onclick={(e) => { e.preventDefault(); open(seg.href!); }}>{seg.text}</a>
          {:else}
            {seg.text}
          {/if}
        {/each}
      </p>
      <div class="err-actions">
        {#if restrictedOrg}
          <button
            class="primary"
            onclick={() => open(`https://github.com/orgs/${restrictedOrg}/oauth_policies`)}
          >
            <Icon name="ExternalLink" size={14} />
            <span>Approve in GitHub</span>
          </button>
        {/if}
        <button class="ghost-btn" onclick={() => (createOpen = true)}>
          <Icon name="Plus" size={14} />
          <span>Create PR anyway</span>
        </button>
      </div>
    </div>
  {:else if prs.data && prs.data.length === 0}
    <div class="state hint">
      <Icon name="GitPullRequest" size={28} />
      <p>Nothing open. Inbox zero for review.</p>
    </div>
  {:else if prs.data}
    <ul class="prs">
      {#each prs.data as pr (pr.number)}
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
                <span class="when">updated {relTime(Math.floor(new Date(pr.updated_at).getTime() / 1000), Math.floor(timestamp / 1000))}</span>
              </div>
            </div>
            <Icon name="ExternalLink" size={12} class="open-icon" />
          </button>
        </li>
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
  .state.err {
    padding: var(--sp-10, 64px) var(--sp-4);
    gap: var(--sp-3);
  }
  .err-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 56px;
    border-radius: var(--r-pill);
    background: color-mix(in srgb, var(--removed) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--removed) 36%, transparent);
    margin-bottom: var(--sp-1);
  }
  .err-icon :global(svg) { color: var(--removed); }
  .err-title {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--weight-semibold);
    color: var(--fg);
    letter-spacing: var(--tracking-tight);
  }
  .err-msg {
    margin: 0;
    max-width: 520px;
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    line-height: 1.55;
    text-align: center;
  }
  .err-msg a {
    color: var(--accent-fg);
    text-decoration: underline;
    text-decoration-color: color-mix(in srgb, var(--accent-fg) 40%, transparent);
    text-underline-offset: 2px;
    overflow-wrap: anywhere;
  }
  .err-msg a:hover { text-decoration-color: var(--accent-fg); }
  .err-actions {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-top: var(--sp-2);
  }
  .ghost-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 36px;
    padding: 0 14px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    line-height: 1;
    cursor: pointer;
    box-sizing: border-box;
    transition: color var(--t-fast), border-color var(--t-fast);
  }
  .ghost-btn :global(svg) { color: var(--fg-subtle); }
  .ghost-btn:hover { color: var(--fg); border-color: var(--border-strong); }
  .ghost-btn:hover :global(svg) { color: var(--fg); }
  .state.cta {
    padding: var(--sp-10, 64px) var(--sp-4);
    gap: var(--sp-3);
  }
  .cta-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 56px;
    border-radius: var(--r-pill);
    background: var(--accent-bg-soft);
    border: 1px solid var(--accent-bg-medium);
    margin-bottom: var(--sp-1);
  }
  .cta-icon :global(svg) { color: var(--accent-fg); }
  .state.cta h3 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--weight-semibold);
    color: var(--fg);
    letter-spacing: var(--tracking-tight);
  }
  .state.cta p {
    margin: 0;
    max-width: 380px;
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    line-height: 1.5;
  }
  .primary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 36px;
    padding: 0 14px;
    background: var(--accent-500);
    color: var(--accent-on);
    border: 1px solid var(--accent-500);
    border-radius: var(--r-md);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    line-height: 1;
    cursor: pointer;
    box-sizing: border-box;
    transition: background var(--t-fast), border-color var(--t-fast), transform var(--t-fast);
  }
  .primary :global(svg) { color: var(--accent-on); }
  .primary:hover { background: var(--accent-400); border-color: var(--accent-400); }
  .primary:active { transform: translateY(1px); }

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
