<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Avatar from '$lib/components/primitives/Avatar.svelte';
  import DiffView from '$lib/components/primitives/DiffView.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import EmptyState from '$lib/components/primitives/EmptyState.svelte';
  import Tag from '$lib/components/primitives/Tag.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import { relTime } from '$lib/utils/time';
  import { commitAvatarUrl } from '$lib/utils/avatar';
  import { gitUrlToWebUrl, fileUrlOnRemote } from '$lib/utils/git-url';
  import type { CommitPage, DiffFile, DiffPayload } from '$lib/types';

  let { id, onClose }: { id: string; onClose: () => void } = $props();

  const log = createQuery<CommitPage>(
    () => queryKeys.repoLogUnpushed(id),
    () => invoke<CommitPage>('commit_log_unpushed', { id, max: 50 }),
  );

  const commits = $derived(log.data?.commits ?? []);

  let selectedOid = $state<string | null>(null);

  $effect(() => {
    if (commits.length === 0) {
      selectedOid = null;
      return;
    }
    if (!selectedOid || !commits.some((c) => c.oid === selectedOid)) {
      selectedOid = commits[0].oid;
    }
  });

  const diff = createQuery<DiffPayload>(
    () => queryKeys.repoDiffCommit(id, selectedOid ?? ''),
    () =>
      selectedOid == null
        ? Promise.resolve({ files: [] })
        : invoke<DiffPayload>('diff_commit', { id, oid: selectedOid }),
  );

  const remoteUrl = createQuery<string | null>(
    () => queryKeys.repoRemoteUrl(id),
    () => invoke<string | null>('repo_remote_url', { id }),
  );
  const webBase = $derived(gitUrlToWebUrl(remoteUrl.data ?? null));

  function fileHref(file: DiffFile): string | null {
    if (!webBase || !selectedOid) return null;
    if (file.status === 'deleted') return null;
    return fileUrlOnRemote(webBase, selectedOid, file.path);
  }
</script>

<Modal {onClose} width="lg" align="center">
  {#snippet head()}
    <header class="head">
      <h2>
        Unpushed commits
        {#if commits.length > 0}
          <Tag tone="accent" size="sm">{commits.length}</Tag>
        {/if}
      </h2>
      <button class="close" onclick={onClose} aria-label="Close">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6L6 18M6 6l12 12"/></svg>
      </button>
    </header>
  {/snippet}

  {#snippet body()}
    <div class="split">
      <aside class="commits">
        {#if log.data}
          {#if commits.length === 0}
            <p class="hint">No unpushed commits.</p>
          {:else}
            <ul>
              {#each commits as c (c.oid)}
                <li class:selected={selectedOid === c.oid}>
                  <button class="row" onclick={() => (selectedOid = c.oid)}>
                    <Avatar name={c.author_name} email={c.author_email} url={commitAvatarUrl(c.author_email)} size={20} />
                    <div class="row-text">
                      <div class="row-1">
                        <span class="message">{c.summary || '(no message)'}</span>
                        <span class="sha">{c.short_sha}</span>
                      </div>
                      <div class="row-2">
                        <span class="who">{c.author_name}</span>
                        <span class="when">· {relTime(c.author_when)}</span>
                      </div>
                    </div>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        {:else if log.error}
          <p class="err">{String(log.error)}</p>
        {:else}
          <p class="hint">Loading…</p>
        {/if}
      </aside>

      <section class="diff">
        {#if selectedOid == null}
          <EmptyState
            illustration="space-cockpit"
            title="Select a commit to view its diff"
            size="sm"
          />
        {:else if diff.data}
          <DiffView payload={diff.data} {fileHref} />
        {:else if diff.error}
          <div class="err">{String(diff.error)}</div>
        {:else}
          <div class="hint">Loading diff…</div>
        {/if}
      </section>
    </div>
  {/snippet}
</Modal>

<style>
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 16px 18px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .head h2 {
    margin: 0;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: var(--fs-lg);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    color: var(--fg);
  }
  .close {
    width: 28px; height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .close:hover { background: var(--bg-elev-3); color: var(--fg); }

  .split {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 360px 1fr;
    height: 100%;
  }
  .commits {
    border-right: 1px solid var(--border);
    overflow-y: auto;
    background: var(--bg-elev-1);
  }
  .commits ul { list-style: none; margin: 0; padding: 6px 0; }
  .commits li { padding: 0; }
  .commits li.selected { background: var(--accent-bg-medium); }
  .commits li.selected .message { color: var(--accent-fg); }
  .commits li button.row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
    color: inherit;
  }
  .commits li button.row:hover { background: var(--bg-elev-2); }
  .commits li.selected button.row:hover { background: var(--accent-bg-medium); }
  .row-text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .row-1 { display: flex; align-items: baseline; gap: 8px; min-width: 0; }
  .message {
    flex: 1;
    min-width: 0;
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sha {
    color: var(--fg-subtle);
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }
  .row-2 {
    display: flex;
    gap: 4px;
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
  }
  .who { color: var(--fg-muted); }

  .diff {
    overflow-y: auto;
    padding: var(--sp-3);
    background: var(--bg);
    min-width: 0;
  }
  .hint { color: var(--fg-subtle); padding: var(--sp-3); font-size: var(--fs-sm); }
  .err { color: var(--removed); padding: var(--sp-3); font-size: var(--fs-sm); }
</style>
