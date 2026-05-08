<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { repos } from '$lib/stores/repos.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { gitUrlToWebUrl } from '$lib/utils/git-url';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Kbd from '$lib/components/primitives/Kbd.svelte';
  import type { BranchInfo } from '$lib/types';

  let { id }: { id: string } = $props();

  // Shared cache keys — branches and remote URL are already loaded by the
  // titlebar / branch switcher, so this is free.
  const branches = createQuery<BranchInfo[] | null>(
    () => queryKeys.repoBranches(id),
    () => invoke<BranchInfo[]>('branch_list', { id }),
  );
  const remoteUrl = createQuery<string | null>(
    () => queryKeys.repoRemoteUrl(id),
    () => invoke<string | null>('repo_remote_url', { id }),
  );

  const head = $derived(branches.data?.find((b) => b.is_head) ?? null);
  const hasUpstream = $derived(head != null && (head.ahead != null || head.behind != null));
  const ahead = $derived(head?.ahead ?? 0);
  const webBase = $derived(gitUrlToWebUrl(remoteUrl.data ?? null));
  const isGithubRepo = $derived(
    !!remoteUrl.data && /github\.com[:/]/.test(remoteUrl.data),
  );
  const defaultBranch = $derived(
    branches.data?.find((b) => !b.is_remote && (b.name === 'main' || b.name === 'master'))
      ?? null,
  );
  const onDefaultBranch = $derived(
    head != null && defaultBranch != null && head.name === defaultBranch.name,
  );
  const canCreatePr = $derived(
    isGithubRepo && hasUpstream && head != null && !onDefaultBranch,
  );
  const repoPath = $derived(repos.activeRepo?.path ?? null);

  function openInEditor() {
    invoke('repo_open_in_editor', { id }).catch((err) =>
      alert(`Failed to open editor: ${String(err)}`),
    );
  }
  function openOnGithub() {
    if (!webBase) return;
    openUrl(webBase).catch(() => {});
  }
</script>

<div class="hints">
  {#if canCreatePr && head}
    <article class="card primary">
      <div class="text">
        <strong>Create a pull request from this branch</strong>
        <p>
          <code>{head.name}</code> is published to GitHub. Open a pull request to propose your changes.
        </p>
        <div class="kbd-line">Branch menu or <Kbd keys={['⌘', 'R']} /></div>
      </div>
      <button type="button" class="btn primary-btn" onclick={() => ui.createPr()}>
        <Icon name="GitPullRequest" size={12} />
        Create pull request
      </button>
    </article>
  {/if}

  {#if hasUpstream && ahead > 0}
    <article class="card">
      <div class="text">
        <strong>Push your changes</strong>
        <p>
          {ahead} commit{ahead === 1 ? '' : 's'} ahead of <code>origin/{head?.name}</code>.
        </p>
        <div class="kbd-line">Titlebar or <Kbd keys={['⌘', 'P']} /></div>
      </div>
      <button type="button" class="btn" onclick={() => ui.push()}>
        <Icon name="ArrowUp" size={12} />
        Push
      </button>
    </article>
  {/if}

  {#if repoPath}
    <article class="card">
      <div class="text">
        <strong>Open the repository in your editor</strong>
        <p>Launches the folder in your OS default app for source folders.</p>
        <div class="kbd-line"><Kbd keys={['⌘', '⇧', 'A']} /></div>
      </div>
      <button type="button" class="btn" onclick={openInEditor}>
        <Icon name="ExternalLink" size={12} />
        Open
      </button>
    </article>
  {/if}

  {#if webBase}
    <article class="card">
      <div class="text">
        <strong>Open the repository on the remote</strong>
        <p>{webBase}</p>
        <div class="kbd-line"><Kbd keys={['⌘', '⇧', 'G']} /></div>
      </div>
      <button type="button" class="btn" onclick={openOnGithub}>
        <Icon name="ExternalLink" size={12} />
        View on remote
      </button>
    </article>
  {/if}

  <section class="more">
    <h3>Navigation shortcuts</h3>
    <ul>
      <li><Kbd keys={['⌘', '1']} /> <span>Changes</span></li>
      <li><Kbd keys={['⌘', '2']} /> <span>History</span></li>
      <li><Kbd keys={['⌘', '3']} /> <span>Pull Requests</span></li>
      <li><Kbd keys={['⌘', 'B']} /> <span>Switch branch</span></li>
      <li><Kbd keys={['⌘', 'O']} /> <span>Switch repository</span></li>
    </ul>
  </section>
</div>

<style>
  .hints {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: var(--sp-3);
    max-width: 720px;
    margin: 0 auto;
  }
  .card {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    padding: 14px 16px;
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
  }
  .card.primary {
    background: var(--accent-bg-soft);
    border-color: var(--accent-bg-strong);
  }
  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
  .text strong {
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    color: var(--fg);
  }
  .text p {
    margin: 0;
    color: var(--fg-muted);
    font-size: var(--fs-xs);
    line-height: 1.45;
  }
  .text code {
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    padding: 1px 5px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
  }
  .card.primary .text strong { color: var(--fg); }
  .kbd-line {
    margin-top: 4px;
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    line-height: 1;
  }

  .btn {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 14px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    transition: border-color var(--t-fast), background var(--t-fast);
  }
  .btn:hover { border-color: var(--border-strong); background: var(--bg-elev-2); }
  .btn :global(svg) { color: var(--fg-subtle); }
  .btn.primary-btn {
    background: var(--accent-500);
    color: var(--accent-on);
    border-color: transparent;
  }
  .btn.primary-btn :global(svg) { color: var(--accent-on); }
  .btn.primary-btn:hover { background: var(--accent-400); }

  .more {
    margin-top: 8px;
    padding: 12px 16px;
    border: 1px dashed var(--border);
    border-radius: var(--r-md);
    background: transparent;
  }
  .more h3 {
    margin: 0 0 8px;
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
  }
  .more ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 6px 16px;
  }
  .more li {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--fg-muted);
    font-size: var(--fs-xs);
  }
</style>
