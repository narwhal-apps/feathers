<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { page } from '$app/stores';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import EmptyState from '$lib/components/primitives/EmptyState.svelte';
  import Avatar from '$lib/components/primitives/Avatar.svelte';
  import Spinner from '$lib/components/primitives/Spinner.svelte';
  import SignInModal from '$lib/components/dialogs/SignInModal.svelte';
  import CreatePRModal from '$lib/components/dialogs/CreatePRModal.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
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
  </header>

  <div class="page-body">
  {#if !github.hydrated}
    <div class="state hint"><Spinner size="sm" /><span>Loading…</span></div>
  {:else if !github.user}
    <div class="state cta">
      <span class="cta-icon" aria-hidden="true">
        <Icon name="GitPullRequest" size={26} />
      </span>
      <h3>Sign in to GitHub</h3>
      <p>Pull requests, your real avatar, and tokens you didn't have to copy-paste.</p>
      <Button variant="primary" size="lg" iconLeft="LogIn" label="Sign in" onclick={() => (signInOpen = true)} />
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
          <Button
            variant="primary"
            iconLeft="ExternalLink"
            label="Approve in GitHub"
            onclick={() => open(`https://github.com/orgs/${restrictedOrg}/oauth_policies`)}
          />
        {/if}
        <Button
          variant="secondary"
          iconLeft="Plus"
          label="Create PR anyway"
          onclick={() => (createOpen = true)}
        />
      </div>
    </div>
  {:else if prs.data && prs.data.length === 0}
    <EmptyState
      illustration="landing-space-capsule"
      title="Inbox zero"
      description="Nothing open for review. Mission accomplished."
    />

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
    <div class="state hint"><Spinner size="sm" /><span>Loading pull requests…</span></div>
  {/if}
  </div>
</div>

{#if signInOpen}
  <SignInModal onClose={() => (signInOpen = false)} />
{/if}
{#if createOpen}
  <CreatePRModal {id} onClose={() => (createOpen = false)} />
{/if}

<style>
  .page {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    padding: var(--sp-3) var(--sp-4);
    color: var(--fg);
  }
  /* Fills the area below the header. EmptyState's min-height: 100%
     resolves against this region (not the whole page), so it centers
     between header and bottom — and the page itself doesn't scroll. */
  .page-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
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
    /* Hug the glyphs so the count badge sits on the visual centerline,
       not the line-box centerline. */
    line-height: 1;
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
  .state.hint {
    font-size: var(--fs-sm);
    flex-direction: row;
    gap: 8px;
  }
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
