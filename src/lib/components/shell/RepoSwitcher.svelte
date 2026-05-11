<script lang="ts">
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { repos } from '$lib/stores/repos.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { openRepoFlow } from '$lib/components/dialogs/openRepo';
  import CloneModal from '$lib/components/dialogs/CloneModal.svelte';
  import { goto } from '$app/navigation';
  import type { RepoSummary } from '$lib/types';

  let open = $state(false);
  let cloneOpen = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);

  const active = $derived(repos.activeRepo);
  const list = $derived(repos.knownRepos);

  // Group repos by their parent directory name — matches the common
  // `~/Developer/<org>/<repo>` convention. Repos that live directly in a
  // non-org folder (or whose parent looks like a personal home dir) fall
  // into "Other".
  function groupOf(path: string): string {
    // Strip trailing slashes, then split.
    const parts = path.replace(/\/+$/, '').split('/');
    if (parts.length < 2) return 'Other';
    const parent = parts[parts.length - 2];
    if (!parent) return 'Other';
    // Skip generic dev-folder names — they're not orgs.
    const generic = new Set([
      'Developer', 'Development', 'dev', 'code', 'src',
      'projects', 'Projects', 'work', 'Work', 'repos', 'Repos',
    ]);
    if (generic.has(parent)) return 'Other';
    return parent;
  }

  type RepoGroup = { name: string; repos: RepoSummary[] };
  const grouped = $derived.by((): RepoGroup[] => {
    const buckets = new Map<string, RepoSummary[]>();
    for (const r of list) {
      const key = groupOf(r.path);
      const arr = buckets.get(key) ?? [];
      arr.push(r);
      buckets.set(key, arr);
    }
    // Group order: active repo's group first, "Other" last, rest alpha.
    const activeGroup = active ? groupOf(active.path) : null;
    const sortedKeys = Array.from(buckets.keys()).sort((a, b) => {
      if (a === activeGroup && b !== activeGroup) return -1;
      if (b === activeGroup && a !== activeGroup) return 1;
      if (a === 'Other') return 1;
      if (b === 'Other') return -1;
      return a.localeCompare(b);
    });
    return sortedKeys.map((name) => ({
      name,
      repos: buckets.get(name)!.sort((a, b) => a.name.localeCompare(b.name)),
    }));
  });
  // Hide section headers when there's only one group — no point.
  const showHeaders = $derived(grouped.length > 1);

  function close() { open = false; }

  function pick(r: RepoSummary) {
    repos.activeRepoId = r.id;
    close();
    goto(`/repo/${r.id}/changes/`);
  }

  async function add() {
    close();
    await openRepoFlow();
  }
  function startClone() {
    close();
    cloneOpen = true;
  }

  function onDocClick(e: MouseEvent) {
    if (!open) return;
    const t = e.target as Node;
    if (triggerEl && (triggerEl === t || triggerEl.contains(t))) return;
    const menu = document.getElementById('repo-switcher-menu');
    if (menu && menu.contains(t)) return;
    close();
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  $effect(() => {
    document.addEventListener('click', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('click', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  });

  // External request (⌘O): toggle the dropdown ONLY when the request
  // counter actually advances — otherwise any unrelated reactive change
  // tracked by this effect would re-toggle the open state.
  let lastRepoReq: number | null = null;
  $effect(() => {
    const req = ui.repoSwitcherRequest;
    if (req != null && req !== lastRepoReq) {
      lastRepoReq = req;
      open = !open;
    }
  });

  function avatarLetter(name: string): string {
    return (name.trim()[0] ?? '?').toUpperCase();
  }
</script>

{#if list.length === 0}
  <div class="wrap">
    <button
      class="empty-trigger"
      bind:this={triggerEl}
      onclick={() => (open = !open)}
      aria-haspopup="menu"
      aria-expanded={open}
    >
      <Icon name="FolderOpen" size={12} />
      <span>Open repository</span>
      <Icon name="ChevronsUpDown" size={12} />
    </button>
    {#if open}
      <div id="repo-switcher-menu" class="menu" role="menu">
        <button class="add" role="menuitem" onclick={add}>
          <span>Open existing repository…</span>
          <Icon name="FolderOpen" size={14} />
        </button>
        <button class="add" role="menuitem" onclick={startClone}>
          <span>Clone repository…</span>
          <Icon name="DownloadCloud" size={14} />
        </button>
      </div>
    {/if}
  </div>
{:else}
  <div class="wrap">
    <button
      class="trigger"
      bind:this={triggerEl}
      onclick={() => (open = !open)}
      aria-haspopup="menu"
      aria-expanded={open}
    >
      {#if active}
        <span class="avatar">{avatarLetter(active.name)}</span>
        <span class="name">{active.name}</span>
      {:else}
        <span class="avatar muted">·</span>
        <span class="name muted">Select repository</span>
      {/if}
      <Icon name="ChevronsUpDown" size={12} />
    </button>

    {#if open}
      <div id="repo-switcher-menu" class="menu" role="menu">
        <ul>
          {#each grouped as g (g.name)}
            {#if showHeaders}
              <li class="group-head">{g.name}</li>
            {/if}
            {#each g.repos as r (r.id)}
              <li>
                <button
                  class="item"
                  class:active={r.id === active?.id}
                  role="menuitem"
                  onclick={() => pick(r)}
                >
                  <span class="item-name">{r.name}</span>
                  {#if r.id === active?.id}
                    <Icon name="Check" size={14} />
                  {/if}
                </button>
              </li>
            {/each}
          {/each}
        </ul>
        <div class="divider"></div>
        <button class="add" role="menuitem" onclick={add}>
          <span>Open existing repository…</span>
          <Icon name="FolderOpen" size={14} />
        </button>
        <button class="add" role="menuitem" onclick={startClone}>
          <span>Clone repository…</span>
          <Icon name="DownloadCloud" size={14} />
        </button>
      </div>
    {/if}
  </div>
{/if}

{#if cloneOpen}
  <CloneModal onClose={() => (cloneOpen = false)} />
{/if}

<style>
  .wrap { position: relative; }

  .trigger,
  .empty-trigger {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 28px;
    padding: 0 10px 0 4px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    color: var(--fg);
    /* Repo name is the most important context in the entire app —
       give it a real typographic moment instead of pill-button text. */
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-tight);
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast);
    box-shadow: var(--inset-top);
  }
  .trigger:hover,
  .empty-trigger:hover {
    background: var(--bg-elev-3);
    border-color: var(--border-strong);
  }
  .empty-trigger {
    padding: 0 10px;
    font-family: var(--font-sans);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
  }

  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: var(--r-sm);
    background: linear-gradient(135deg, var(--accent-600), var(--accent-800));
    color: var(--accent-50);
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: var(--weight-bold);
    letter-spacing: 0;
    flex-shrink: 0;
    box-shadow: var(--inset-top), 0 1px 2px rgba(0, 0, 0, 0.3);
  }
  .avatar.muted {
    background: var(--bg-elev-1);
    color: var(--fg-subtle);
    box-shadow: none;
  }
  .name { color: var(--fg); max-width: 220px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .name.muted { color: var(--fg-subtle); font-weight: var(--weight-medium); }
  .trigger :global(svg:last-of-type) { color: var(--fg-subtle); margin-left: 2px; }

  .menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    min-width: 280px;
    max-width: 380px;
    background: var(--bg-elev-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-3);
    padding: 6px;
    z-index: 10;
  }
  .menu::before {
    content: "";
    position: absolute; inset: 0;
    border-radius: var(--r-lg);
    background-image: var(--grain);
    opacity: 0.4;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .menu ul { list-style: none; margin: 0; padding: 0; position: relative; z-index: 1; }
  .menu li { padding: 0; }
  .menu li.group-head {
    padding: 8px 10px 4px;
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    font-weight: var(--weight-semibold);
  }
  /* Tighten the gap between consecutive group headers (only happens when
     a group has zero repos, but defensive). */
  .menu li.group-head + .group-head { margin-top: 0; padding-top: 4px; }
  /* First group's header should hug the menu edge. */
  .menu ul > li.group-head:first-child { padding-top: 4px; }

  .item {
    display: flex;
    align-items: center;
    width: 100%;
    gap: var(--sp-2);
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    font-weight: var(--weight-medium);
    cursor: pointer;
    text-align: left;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .item:hover { background: var(--bg-elev-2); color: var(--fg); }
  .item.active {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    font-weight: var(--weight-semibold);
  }
  .item-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 6px 4px;
    position: relative; z-index: 1;
  }

  .add {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    width: 100%;
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    cursor: pointer;
    transition: background var(--t-fast);
    position: relative; z-index: 1;
  }
  .add:hover { background: var(--accent-bg-soft); color: var(--accent-fg); }
</style>
