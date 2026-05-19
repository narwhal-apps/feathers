<script lang="ts">
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Kbd from '$lib/components/primitives/Kbd.svelte';

  let open = $state(false);
  let hoverTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  let wrapEl = $state<HTMLDivElement | null>(null);

  function show() {
    if (hoverTimer) clearTimeout(hoverTimer);
    open = true;
  }
  function scheduleHide() {
    hoverTimer = setTimeout(() => (open = false), 150);
  }
  function toggle() {
    open = !open;
  }

  $effect(() => {
    if (!open) return;
    function onDocClick(e: MouseEvent) {
      if (wrapEl && !wrapEl.contains(e.target as Node)) open = false;
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') open = false;
    }
    document.addEventListener('mousedown', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="wrap"
  bind:this={wrapEl}
  onmouseenter={show}
  onmouseleave={scheduleHide}
>
  <button
    class="trigger"
    onclick={toggle}
    aria-haspopup="true"
    aria-expanded={open}
    title="Keyboard shortcuts"
  >
    <Icon name="Keyboard" size={16} />
  </button>

  {#if open}
    <div class="popover" role="tooltip">
      <h4>Keyboard shortcuts</h4>

      <div class="group">
        <h5>Navigation</h5>
        <ul>
          <li><Kbd keys={['⌘', '1']} /> <span>Changes</span></li>
          <li><Kbd keys={['⌘', '2']} /> <span>History</span></li>
          <li><Kbd keys={['⌘', '3']} /> <span>Pull requests</span></li>
          <li><Kbd keys={['⌘', 'B']} /> <span>Switch branch</span></li>
          <li><Kbd keys={['⌘', 'O']} /> <span>Switch repository</span></li>
        </ul>
      </div>

      <div class="group">
        <h5>Actions</h5>
        <ul>
          <li><Kbd keys={['⌘', 'P']} /> <span>Push</span></li>
          <li><Kbd keys={['⌘', 'R']} /> <span>Create PR</span></li>
          <li><Kbd keys={['⌘', '⇧', 'A']} /> <span>Open in editor</span></li>
          <li><Kbd keys={['⌘', '⇧', 'G']} /> <span>Open on GitHub</span></li>
          <li><Kbd keys={['⌘', ',']} /> <span>Settings</span></li>
        </ul>
      </div>

      <div class="group">
        <h5>Changes</h5>
        <ul>
          <li><Kbd keys={['↑', '↓']} /> <span>Navigate files</span></li>
          <li><Kbd keys={['⌘', '↵']} /> <span>Commit</span></li>
        </ul>
      </div>
    </div>
  {/if}
</div>

<style>
  .wrap {
    position: relative;
    display: flex;
    align-items: center;
  }
  .trigger {
    width: 26px;
    height: 26px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--r-sm);
    color: var(--fg-muted);
    cursor: pointer;
    transition: background var(--t-fast), border-color var(--t-fast), color var(--t-fast);
  }
  .trigger:hover {
    background: var(--bg-elev-2);
    border-color: var(--border);
    color: var(--fg);
  }
  .trigger[aria-expanded='true'] {
    background: var(--bg-elev-2);
    border-color: var(--border-strong);
    color: var(--fg);
  }

  .popover {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    min-width: 240px;
    padding: 12px 14px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-2);
    z-index: 60;
  }
  h4 {
    margin: 0 0 10px;
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    color: var(--fg);
  }
  .group + .group {
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }
  h5 {
    margin: 0 0 6px;
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wider);
    color: var(--fg-subtle);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--fs-xs);
    color: var(--fg-muted);
    white-space: nowrap;
  }
</style>
