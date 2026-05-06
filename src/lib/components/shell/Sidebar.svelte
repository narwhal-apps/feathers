<script lang="ts">
  import Icon from '$lib/components/primitives/Icon.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
</script>

<aside class="sidebar">
  <section class="section">
    <header class="section-header">
      <span>Repos</span>
      <Button label="Open" variant="ghost" size="sm" disabled />
    </header>
    <div class="empty">
      <Icon name="FolderOpen" size={18} />
      <p>No repositories yet</p>
      <p class="hint">Repository opening lands in milestone 2.</p>
    </div>
  </section>

  <section class="section">
    <header class="section-header">Branches</header>
    <div class="empty subtle">—</div>
  </section>

  <section class="section">
    <header class="section-header">Changes</header>
    <div class="empty subtle">—</div>
  </section>

  <section class="section">
    <header class="section-header">Pull Requests</header>
    <div class="empty subtle">—</div>
  </section>
</aside>

<style>
  .sidebar {
    width: 240px;
    flex-shrink: 0;
    background: var(--bg-elev-1);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: var(--sp-3) 0;
  }
  .section { padding: var(--sp-2) 0; }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--sp-3);
    margin-bottom: var(--sp-2);
    color: var(--fg-subtle);
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 600;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-1);
    color: var(--fg-muted);
    font-size: var(--fs-sm);
    padding: var(--sp-3);
    text-align: center;
  }
  .empty p { margin: 0; }
  .empty .hint { color: var(--fg-subtle); font-size: var(--fs-xs); }
  .empty.subtle { color: var(--fg-subtle); padding: var(--sp-1) var(--sp-3); }
</style>
