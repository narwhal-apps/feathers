<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Pill from '$lib/components/primitives/Pill.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import RepoSwitcher from '$lib/components/shell/RepoSwitcher.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { createQuery } from '$lib/query/createQuery.svelte';
  import { queryKeys } from '$lib/query/keys';
  import type { BranchInfo } from '$lib/types';

  const active = $derived(repos.activeRepo);

  const branches = createQuery<BranchInfo[] | null>(
    () => active ? queryKeys.repoBranches(active.id) : ['noop'],
    () => active ? invoke<BranchInfo[]>('branch_list', { id: active.id }) : Promise.resolve(null),
  );

  const headBranch = $derived(branches.data?.find((b) => b.is_head) ?? null);
  const ahead = $derived(headBranch?.ahead ?? 0);
  const behind = $derived(headBranch?.behind ?? 0);
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="lights-spacer" data-tauri-drag-region></div>

  <RepoSwitcher />

  {#if active && headBranch}
    <span class="sep" data-tauri-drag-region>/</span>
    <Pill label={headBranch.name} tone="accent" />
    {#if ahead > 0 || behind > 0}
      <span class="counts" data-tauri-drag-region>↓ {behind}  ↑ {ahead}</span>
    {/if}
  {/if}

  <div class="spacer" data-tauri-drag-region></div>

  <div class="actions">
    <Button label="Fetch" variant="ghost" size="sm" disabled />
    <Button label="Push"  variant="primary" size="sm" disabled />
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    height: 56px;
    padding: 0 var(--sp-4) 0 0;
    background: var(--bg-elev-1);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }
  .lights-spacer { width: 80px; height: 100%; flex-shrink: 0; }
  .sep   { color: var(--fg-subtle); }
  .counts { color: var(--fg-muted); font-family: var(--font-mono); font-size: var(--fs-xs); font-variant-numeric: tabular-nums; }
  .spacer { flex: 1; }
  .actions { display: flex; gap: var(--sp-2); }
</style>
