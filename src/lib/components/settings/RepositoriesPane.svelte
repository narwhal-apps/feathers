<script lang="ts">
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import type { RepoSummary } from '$lib/types';

  let confirmingId = $state<string | null>(null);

  $effect(() => {
    repos.refresh();
    let unlisten: (() => void) | null = null;
    import('@tauri-apps/api/event').then(({ listen }) =>
      listen('repo_changed', () => repos.refresh()).then((u) => { unlisten = u; }),
    );
    return () => { unlisten?.(); };
  });

  function avatarLetter(name: string): string {
    return (name.trim()[0] ?? '?').toUpperCase();
  }

  async function forget(r: RepoSummary): Promise<void> {
    if (confirmingId !== r.id) {
      confirmingId = r.id;
      return;
    }
    confirmingId = null;
    try {
      await repos.close(r.id);
    } catch {
      // The list is refetched anyway; a stale row falls off naturally.
      await repos.refresh();
    }
  }

  function cancelConfirm(): void {
    confirmingId = null;
  }
</script>

{#if repos.knownRepos.length === 0}
  <div class="empty">No repositories yet.</div>
{:else}
  <ul class="list">
    {#each repos.knownRepos as r (r.id)}
      <li class="row">
        <span class="avatar">{avatarLetter(r.name)}</span>
        <div class="info">
          <div class="name">{r.name}</div>
          <div class="path">{r.path}</div>
        </div>
        {#if r.id === repos.activeRepoId}
          <span class="badge" title="Close this repo first">Currently open</span>
          <button class="btn" disabled>Forget</button>
        {:else if confirmingId === r.id}
          <button class="btn ghost" onclick={cancelConfirm}>Cancel</button>
          <button class="btn danger" onclick={() => forget(r)}>
            <Icon name="Trash2" size={12} />
            Confirm
          </button>
        {:else}
          <button class="btn ghost" onclick={() => forget(r)}>Forget</button>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<style>
  .empty {
    padding: var(--sp-5);
    text-align: center;
    color: var(--fg-subtle);
    font-size: var(--fs-sm);
  }
  .list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: var(--sp-2); }
  .row {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-elev-1);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
  }
  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px; height: 28px;
    border-radius: var(--r-sm);
    background: linear-gradient(135deg, var(--accent-600), var(--accent-800));
    color: var(--accent-50);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: var(--weight-bold);
    flex-shrink: 0;
  }
  .info { flex: 1; min-width: 0; }
  .name { font-size: var(--fs-sm); font-weight: var(--weight-medium); }
  .path { font-size: var(--fs-xs); color: var(--fg-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .badge {
    padding: 2px 8px;
    background: var(--accent-bg-soft);
    color: var(--accent-fg);
    border-radius: var(--r-sm);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
  }
  .btn {
    display: inline-flex; align-items: center; gap: 4px;
    padding: 4px 10px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    cursor: pointer;
  }
  .btn.ghost { background: transparent; }
  .btn.danger { background: var(--danger-bg, #c00); color: white; border-color: transparent; }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
