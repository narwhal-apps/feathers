<script lang="ts">
  import { goto } from '$app/navigation';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import FeatherMark from '$lib/components/shell/FeatherMark.svelte';
  import CloneModal from '$lib/components/dialogs/CloneModal.svelte';
  import { openRepoFlow } from '$lib/components/dialogs/openRepo';
  import { repos } from '$lib/stores/repos.svelte';
  import type { RepoSummary } from '$lib/types';

  const version = import.meta.env.VITE_APP_VERSION;

  let cloneOpen = $state(false);

  // Show at most a handful so the screen stays composed; recency-by-id is
  // close enough — newer repos get higher UUIDs because we mint them in order.
  const recents = $derived<RepoSummary[]>(
    [...repos.knownRepos].slice(-5).reverse(),
  );

  function pick(r: RepoSummary): void {
    repos.activeRepoId = r.id;
    goto(`/repo/${r.id}/changes/`);
  }

  // Hide common dev-folder names — same convention as the RepoSwitcher.
  const generic = new Set([
    'Developer', 'Development', 'dev', 'code', 'src',
    'projects', 'Projects', 'work', 'Work', 'repos', 'Repos',
  ]);
  function org(path: string): string | null {
    const parts = path.replace(/\/+$/, '').split('/');
    if (parts.length < 2) return null;
    const parent = parts[parts.length - 2];
    if (!parent || generic.has(parent)) return null;
    return parent;
  }
</script>

<svelte:head>
  <title>Feathers</title>
</svelte:head>

<section class="welcome">
  <div class="watermark" aria-hidden="true">
    <FeatherMark size={520} />
  </div>

  <div class="frame">
    <header class="hero">
      <div class="mark"><FeatherMark size={56} /></div>
      <h1>Feathers</h1>
      <p class="tagline">A featherweight git client. Quiet, fast, opinionated.</p>
    </header>

    <div class="cta">
      <button class="btn primary" onclick={openRepoFlow}>
        <Icon name="FolderOpen" size={14} />
        <span>Open repository</span>
        <kbd>⌘O</kbd>
      </button>
      <button class="btn ghost" onclick={() => (cloneOpen = true)}>
        <Icon name="DownloadCloud" size={14} />
        <span>Clone from URL</span>
      </button>
    </div>

    {#if recents.length > 0}
      <section class="recents">
        <div class="rule">
          <span>Recent</span>
        </div>
        <ul>
          {#each recents as r (r.id)}
            <li>
              <button class="recent" onclick={() => pick(r)}>
                <span class="avatar">{r.name.trim()[0]?.toUpperCase() ?? '?'}</span>
                <span class="meta">
                  <span class="name">{r.name}</span>
                  <span class="path">
                    {#if org(r.path)}<span class="org">{org(r.path)}</span> · {/if}{r.path}
                  </span>
                </span>
                <Icon name="ChevronRight" size={14} />
              </button>
            </li>
          {/each}
        </ul>
      </section>
    {/if}
  </div>

  <footer class="legend">
    <span class="hint"><kbd>⌘O</kbd> open</span>
    <span class="dot">·</span>
    <span class="hint"><kbd>⌘,</kbd> settings</span>
    <span class="dot">·</span>
    <span class="hint">v{version}</span>
  </footer>
</section>

{#if cloneOpen}
  <CloneModal onClose={() => (cloneOpen = false)} />
{/if}

<style>
  .welcome {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--fg);
    overflow: hidden;
    /* Whisper of texture so the cream isn't dead-flat. */
    background:
      radial-gradient(120% 90% at 50% 0%, color-mix(in srgb, var(--accent-bg-soft, var(--bg-elev-1)) 35%, transparent) 0%, transparent 60%),
      var(--bg);
  }
  .welcome::before {
    content: '';
    position: absolute; inset: 0;
    background-image: var(--grain);
    opacity: 0.6;
    pointer-events: none;
    mix-blend-mode: overlay;
  }
  .watermark {
    position: absolute;
    right: -120px;
    bottom: -160px;
    color: var(--accent-fg);
    opacity: 0.04;
    pointer-events: none;
    transform: rotate(18deg);
    z-index: 0;
  }

  .frame {
    position: relative;
    z-index: 1;
    width: min(560px, 92%);
    display: flex;
    flex-direction: column;
    gap: var(--sp-5);
    animation: rise 0.6s cubic-bezier(0.2, 0.7, 0.2, 1) both;
  }

  .hero { display: flex; flex-direction: column; align-items: center; gap: var(--sp-2); text-align: center; }
  .mark {
    color: var(--accent-fg);
    transform-origin: 60% 40%;
    animation: drop 0.7s cubic-bezier(0.2, 0.7, 0.1, 1) both;
  }
  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 44px;
    font-weight: var(--weight-bold, 700);
    letter-spacing: -0.02em;
    line-height: 1;
  }
  .tagline {
    margin: 0;
    color: var(--fg-muted);
    font-size: var(--fs-md);
    line-height: 1.5;
    max-width: 420px;
  }

  .cta {
    display: flex;
    gap: var(--sp-3);
    justify-content: center;
    flex-wrap: wrap;
  }
  .btn {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 10px 16px;
    background: var(--bg-elev-2);
    color: var(--fg);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    cursor: pointer;
    transition: transform var(--t-fast), background var(--t-fast), border-color var(--t-fast), box-shadow var(--t-fast);
    box-shadow: var(--inset-top, 0 1px 0 rgba(255,255,255,0.04) inset);
  }
  .btn:hover {
    background: var(--bg-elev-3);
    border-color: var(--border-strong);
    transform: translateY(-1px);
  }
  .btn:active { transform: translateY(0); }
  .btn.primary {
    background: linear-gradient(180deg, var(--accent-700, var(--accent-fg)), var(--accent-800, var(--accent-fg)));
    color: var(--accent-50, white);
    border-color: transparent;
    box-shadow:
      0 1px 0 rgba(255, 255, 255, 0.15) inset,
      0 6px 16px -8px rgba(0, 0, 0, 0.4);
  }
  .btn.primary:hover {
    background: linear-gradient(180deg, var(--accent-600, var(--accent-fg)), var(--accent-700, var(--accent-fg)));
  }
  .btn.ghost { background: transparent; }
  .btn kbd {
    display: inline-block;
    margin-left: 4px;
    padding: 1px 6px;
    background: rgba(255, 255, 255, 0.18);
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: var(--weight-medium);
    letter-spacing: 0;
  }

  .recents { display: flex; flex-direction: column; gap: var(--sp-2); margin-top: var(--sp-3); }
  .rule {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    font-weight: var(--weight-semibold);
  }
  .rule::before, .rule::after {
    content: '';
    flex: 1;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--border), transparent);
  }
  .recents ul { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 4px; }
  .recent {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--r-md);
    color: var(--fg);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast), transform var(--t-fast);
  }
  .recent:hover {
    background: var(--bg-elev-2);
    border-color: var(--border);
    transform: translateX(2px);
  }
  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px; height: 28px;
    flex-shrink: 0;
    border-radius: var(--r-sm);
    background: linear-gradient(135deg, var(--accent-600, var(--accent-fg)), var(--accent-800, var(--accent-fg)));
    color: var(--accent-50, white);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: var(--weight-bold);
  }
  .meta { flex: 1; min-width: 0; display: flex; flex-direction: column; line-height: 1.3; }
  .name { font-size: var(--fs-sm); font-weight: var(--weight-medium); }
  .path {
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    color: var(--fg-subtle);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .org { color: var(--accent-fg); font-weight: var(--weight-semibold); }
  .recent :global(svg:last-of-type) { color: var(--fg-subtle); flex-shrink: 0; }

  .legend {
    position: absolute;
    bottom: var(--sp-4);
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    color: var(--fg-subtle);
    font-size: var(--fs-2xs);
    z-index: 1;
  }
  .legend kbd {
    display: inline-block;
    padding: 1px 6px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fg-muted);
    margin-right: 4px;
  }
  .hint { display: inline-flex; align-items: center; }
  .dot { opacity: 0.4; }

  @keyframes rise {
    from { opacity: 0; transform: translateY(8px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  @keyframes drop {
    from { opacity: 0; transform: translateY(-12px) rotate(-8deg); }
    to   { opacity: 1; transform: translateY(0) rotate(0); }
  }
</style>
