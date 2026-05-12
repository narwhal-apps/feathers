<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import type { BranchInfo, CommitPage, PullRequest, AppError } from '$lib/types';

  let { id, onClose }: { id: string; onClose: (created?: PullRequest) => void } = $props();

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

  let prefilled = false;
  $effect(() => {
    if (prefilled) return;
    const top = log.data?.commits[0];
    if (!top || title) return;
    title = top.summary;
    prefilled = true;
  });
  $effect(() => {
    if (!base && defaultBase) base = defaultBase.name;
  });
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
</script>

<Modal
  title="Open pull request"
  onClose={close}
  width="md"
  actions={{
    secondary: { label: 'Cancel', onclick: close, disabled: busy },
    primary: {
      label: busy ? 'Creating…' : draft ? 'Open draft' : 'Open pull request',
      onclick: submit,
      loading: busy,
      disabled: busy || !title.trim() || !head || !base || base === head?.name,
    },
  }}
>
  {#snippet body()}
    <form class="form" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <div class="branches">
        <span class="branch base"><Icon name="GitBranch" size={12} />
          <select bind:value={base} disabled={busy}>
            {#each (branches.data ?? []).filter((b) => !b.is_remote) as b (b.name)}
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
    </form>
  {/snippet}
</Modal>

<style>
  .form { display: contents; }

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

</style>
