<script lang="ts">
  import Button from '$lib/components/primitives/Button.svelte';
  import Avatar from '$lib/components/primitives/Avatar.svelte';
  import EmptyState from '$lib/components/primitives/EmptyState.svelte';
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
  <EmptyState
    illustration="rocket-launch"
    title="No repositories yet"
    description="Open or clone one from the welcome screen to get started."
    size="md"
  />
{:else}
  <ul class="list">
    {#each repos.knownRepos as r (r.id)}
      <li class="row">
        <Avatar name={r.name} size={28} />
        <div class="info">
          <div class="name">{r.name}</div>
          <div class="path">{r.path}</div>
        </div>
        {#if r.id === repos.activeRepoId}
          <span class="badge" title="Close this repo first">Currently open</span>
          <Button size="sm" label="Forget" disabled />
        {:else if confirmingId === r.id}
          <Button variant="ghost" size="sm" label="Cancel" onclick={cancelConfirm} />
          <Button variant="danger" size="sm" iconLeft="Trash2" label="Confirm" onclick={() => forget(r)} />
        {:else}
          <Button variant="danger" size="sm" label="Forget" onclick={() => forget(r)} />
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<style>
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
</style>
