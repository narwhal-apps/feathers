<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Modal from '$lib/components/primitives/Modal.svelte';
  import Field from '$lib/components/primitives/Field.svelte';
  import Input from '$lib/components/primitives/Input.svelte';
  import TextArea from '$lib/components/primitives/TextArea.svelte';
  import Banner from '$lib/components/primitives/Banner.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryClient } from '$lib/query/client';
  import { queryKeys } from '$lib/query/keys';
  import { gitUrlToWebUrl } from '$lib/utils/git-url';
  import type { BranchInfo, CommitPage, PullRequest, AppError } from '$lib/types';

  let { id, onClose, onRestricted }: {
    id: string;
    onClose: (created?: PullRequest) => void;
    onRestricted?: () => void;
  } = $props();

  const branches = createQuery<BranchInfo[] | null>(
    () => queryKeys.repoBranches(id),
    () => invoke<BranchInfo[]>('branch_list', { id }),
  );
  const log = createQuery<CommitPage>(
    () => queryKeys.repoLog(id),
    () => invoke<CommitPage>('commit_log', { id, opts: { max: 5 } }),
  );

  const remoteUrl = createQuery<string | null>(
    () => queryKeys.repoRemoteUrl(id),
    () => invoke<string | null>('repo_remote_url', { id }),
  );
  const webBase = $derived(gitUrlToWebUrl(remoteUrl.data ?? null));

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
  let description = $state('');
  let base = $state('');
  let draft = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  /** Populated alongside `error` so the banner can offer a fallback "Open
   *  on GitHub" button — the user can finish the PR in the browser when
   *  the in-app create fails (typically: PR already exists, validation
   *  rejected, token expired, network glitch). Null until we've resolved
   *  the repo's web URL. */
  let manualPrUrl = $state<string | null>(null);
  let titleEl = $state<HTMLInputElement | null>(null);

  let prefilled = false;
  let cursorPlaced = false;
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
  // One-shot: once the prefill lands and the input mounts, focus and
  // park the caret at the end. Reading `cursorPlaced` (not `title`)
  // means typing does NOT retrigger this — otherwise every keystroke
  // would snap the caret back to the end of the line.
  $effect(() => {
    if (cursorPlaced || !titleEl || !prefilled || busy) return;
    titleEl.focus();
    titleEl.setSelectionRange(title.length, title.length);
    cursorPlaced = true;
  });

  async function submit() {
    if (busy) return;
    const t = title.trim();
    if (!t || !head || !base || base === head.name) return;
    busy = true;
    error = null;
    manualPrUrl = null;
    try {
      const pr = await invoke<PullRequest>('github_create_pr', {
        id,
        title: t,
        body: description.trim() || null,
        base,
        draft,
      });
      queryClient.invalidate(queryKeys.repoPullRequests(id));
      onClose(pr);
      try { await openUrl(pr.html_url); } catch { /* ignore */ }
    } catch (err) {
      const e = err as AppError;
      const isForbidden =
        typeof e === 'object' && e !== null && 'kind' in e && (e as { kind: string }).kind === 'forbidden';

      if (isForbidden && webBase && head) {
        const url = `${webBase}/pull/new/${encodeURIComponent(head.name)}`;
        void openUrl(url);
        onRestricted?.();
        onClose();
        return;
      }

      error =
        typeof e === 'object' && e !== null && 'message' in e
          ? (e as { message: string }).message
          : JSON.stringify(err);
      if (webBase && head && base) {
        manualPrUrl = `${webBase}/compare/${encodeURIComponent(base)}...${encodeURIComponent(head.name)}?expand=1`;
      }
    } finally {
      busy = false;
    }
  }

  function openManual(): void {
    if (!manualPrUrl) return;
    void openUrl(manualPrUrl);
    onClose();
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

      <Field label="Title">
        <Input
          bind:value={title}
          bind:ref={titleEl}
          disabled={busy}
          placeholder="A short summary"
          required
        />
      </Field>

      <Field label="Description (Markdown)" optional>
        <TextArea
          variant="mono"
          bind:value={description}
          disabled={busy}
          rows={5}
          placeholder="Add context, screenshots, links — anything reviewers will need."
          onkeydown={(e) => {
            if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') { e.preventDefault(); submit(); }
          }}
        />
      </Field>

      <label class="toggle">
        <input type="checkbox" bind:checked={draft} disabled={busy} />
        <span>Open as draft</span>
      </label>

      {#if error}
        <Banner tone="error">
          {error}
          {#snippet actions()}
            {#if manualPrUrl}
              <Button label="Open on GitHub" iconLeft="ExternalLink" onclick={openManual} />
            {/if}
          {/snippet}
        </Banner>
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
    overflow: hidden;
  }
  .branch {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    color: var(--accent-fg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
  }
  .branch :global(svg) { color: var(--accent-fg); flex-shrink: 0; }
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
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .branch.head .branch-name {
    padding: 2px 6px;
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    background: var(--bg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 200px;
  }
  .branches :global(.arrow) { color: var(--fg-muted); }

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
</style>
