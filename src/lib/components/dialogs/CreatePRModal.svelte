<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { BranchInfo, CommitPage, PullRequest, AppError } from '$lib/types';

  let { id, onClose }: { id: string; onClose: (created?: PullRequest) => void } = $props();

  // Pull pieces from the same caches the rest of the app uses.
  const branches = createQuery<BranchInfo[] | null>(
    () => queryKeys.repoBranches(id),
    () => invoke<BranchInfo[]>('branch_list', { id }),
  );
  const log = createQuery<CommitPage>(
    () => queryKeys.repoLog(id),
    () => invoke<CommitPage>('commit_log', { id, opts: { max: 5 } }),
  );

  const head = $derived(branches.data?.find((b) => b.is_head) ?? null);
  const localBranches = $derived(
    (branches.data ?? []).filter((b) => !b.is_remote && !b.is_head),
  );
  // Default base: prefer "main", then "master", else first non-head local.
  const defaultBase = $derived(
    branches.data?.find((b) => !b.is_remote && b.name === 'main') ??
    branches.data?.find((b) => !b.is_remote && b.name === 'master') ??
    localBranches[0] ??
    null,
  );

  let title = $state('');
  let body = $state('');
  let base = $state('');
  let draft = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let titleEl = $state<HTMLInputElement | null>(null);

  // Pre-fill title (and body, if commit has one beyond the summary) from the
  // most recent commit on the current branch — matches GitHub's web UI.
  let prefilled = false;
  $effect(() => {
    if (prefilled) return;
    const top = log.data?.commits[0];
    if (!top || title) return;
    title = top.summary;
    prefilled = true;
  });
  // Default base picker once branches load.
  $effect(() => {
    if (!base && defaultBase) base = defaultBase.name;
  });
  // Focus the title field once it's populated.
  $effect(() => {
    if (titleEl && title && !busy) {
      titleEl.focus();
      titleEl.setSelectionRange(title.length, title.length);
    }
  });

  async function submit() {
    if (busy) return;
    const t = title.trim();
    if (!t || !head || !base || base === head.name) return;
    busy = true;
    error = null;
    try {
      const pr = await invoke<PullRequest>('github_create_pr', {
        id,
        title: t,
        body: body.trim() || null,
        base,
        draft,
      });
      queryClient.invalidate(queryKeys.repoPullRequests(id));
      onClose(pr);
      // Open the new PR for follow-up.
      try { await openUrl(pr.html_url); } catch { /* ignore */ }
    } catch (err) {
      const e = err as AppError;
      error =
        typeof e === 'object' && e !== null && 'message' in e
          ? (e as { message: string }).message
          : JSON.stringify(err);
    } finally {
      busy = false;
    }
  }

  function close() { if (!busy) onClose(); }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
  $effect(() => {
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  });
</script>

<div
  class="backdrop"
  role="presentation"
  onclick={(e) => { if (e.target === e.currentTarget) close(); }}
  onkeydown={() => {}}
>
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="create-pr-title">
    <header class="head">
      <h2 id="create-pr-title">Create pull request</h2>
      <button class="close" onclick={close} aria-label="Close">
        <Icon name="X" size={14} />
      </button>
    </header>

    <form class="body" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <div class="branches">
        <span class="branch base"><Icon name="GitBranch" size={12} />
          <select bind:value={base} disabled={busy}>
            {#each (branches.data ?? []).filter((b) => !b.is_remote) as b}
              <option value={b.name}>{b.name}</option>
            {/each}
          </select>
        </span>
        <Icon name="ArrowLeft" size={14} class="arrow" />
        <span class="branch head"><Icon name="GitBranch" size={12} />
          <span class="branch-name">{head?.name ?? '—'}</span>
        </span>
      </div>

      <label class="field">
        <span class="label">Title</span>
        <input
          class="input"
          type="text"
          bind:value={title}
          bind:this={titleEl}
          disabled={busy}
          placeholder="A short summary"
          required
        />
      </label>

      <label class="field">
        <span class="label">Description <span class="muted">(optional, Markdown)</span></span>
        <textarea
          class="input message"
          bind:value={body}
          disabled={busy}
          rows="5"
          placeholder="Add context, screenshots, links — anything reviewers will need."
          onkeydown={(e) => {
            if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') { e.preventDefault(); submit(); }
          }}
        ></textarea>
      </label>

      <label class="toggle">
        <input type="checkbox" bind:checked={draft} disabled={busy} />
        <span>Open as draft</span>
      </label>

      {#if error}
        <div class="err">
          <Icon name="AlertTriangle" size={14} />
          <span>{error}</span>
        </div>
      {/if}

      <footer class="foot">
        <button type="button" class="btn ghost" onclick={close} disabled={busy}>Cancel</button>
        <button
          type="submit"
          class="btn primary"
          disabled={busy || !title.trim() || !head || !base || base === head?.name}
        >
          {busy ? 'Creating…' : draft ? 'Create draft' : 'Create pull request'}
        </button>
      </footer>
    </form>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in srgb, #000 55%, transparent);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 10vh;
    z-index: 200;
  }
  .modal {
    width: min(560px, calc(100vw - 32px));
    background: var(--bg-elev-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-3);
    overflow: hidden;
    position: relative;
  }
  .modal::before {
    content: "";
    position: absolute; inset: 0;
    background-image: var(--grain);
    opacity: 0.35;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    position: relative; z-index: 1;
  }
  .head h2 {
    margin: 0;
    font-size: var(--fs-md);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    color: var(--fg);
  }
  .close {
    width: 26px; height: 26px;
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

  .body {
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    position: relative; z-index: 1;
  }

  .branches {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
  }
  .branch {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--accent-fg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
  }
  .branch :global(svg) { color: var(--accent-fg); }
  .branch.base select {
    background: transparent;
    border: 1px solid var(--accent-bg-strong);
    border-radius: var(--r-sm);
    padding: 2px 6px;
    color: var(--accent-fg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    outline: none;
  }
  .branch.head .branch-name {
    padding: 2px 6px;
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    background: var(--bg);
  }
  .branches :global(.arrow) { color: var(--fg-muted); }

  .field { display: flex; flex-direction: column; gap: 6px; }
  .label {
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
    font-weight: var(--weight-semibold);
  }
  .label .muted {
    text-transform: none;
    letter-spacing: var(--tracking-tight);
    color: var(--fg-faint);
    font-weight: var(--weight-medium);
    margin-left: 4px;
  }
  .input {
    width: 100%;
    padding: 8px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--fs-sm);
    line-height: 1.4;
    outline: none;
    transition: border-color var(--t-fast);
  }
  .input.message {
    resize: vertical;
    min-height: 100px;
    max-height: 280px;
    font-family: var(--font-mono);
  }
  .input:focus { border-color: var(--accent-500); }
  .input::placeholder { color: var(--fg-subtle); }

  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    cursor: pointer;
    user-select: none;
  }
  .toggle input { accent-color: var(--accent-500); margin: 0; }

  .err {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 10px 12px;
    background: color-mix(in srgb, var(--removed) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--removed) 35%, transparent);
    border-radius: var(--r-md);
    color: var(--fg);
    font-size: var(--fs-xs);
    line-height: 1.4;
  }
  .err :global(svg) { color: var(--removed); flex-shrink: 0; margin-top: 2px; }

  .foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .btn {
    height: 32px;
    padding: 0 14px;
    border-radius: var(--r-sm);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    border: 1px solid transparent;
    transition: background var(--t-fast), color var(--t-fast), border-color var(--t-fast);
  }
  .btn.primary { background: var(--accent-500); color: var(--accent-on); }
  .btn.primary:hover:not(:disabled) { background: var(--accent-400); }
  .btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.ghost {
    background: transparent;
    color: var(--fg-muted);
    border-color: var(--border);
  }
  .btn.ghost:hover:not(:disabled) { color: var(--fg); border-color: var(--border-strong); }
</style>
