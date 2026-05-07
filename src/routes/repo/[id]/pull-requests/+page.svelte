<script lang="ts">
  import Icon from '$lib/components/primitives/Icon.svelte';
</script>

<div class="empty">
  <Icon name="GitPullRequest" size={28} />
  <h2>Pull requests</h2>
  <p>GitHub integration is coming. Sign in to GitHub to list, view, and create pull requests for this repository.</p>
</div>

<style>
  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-3);
    padding: var(--sp-6);
    color: var(--fg-subtle);
    text-align: center;
  }
  .empty :global(svg) { color: var(--fg-muted); }
  .empty h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    color: var(--fg);
  }
  .empty p {
    margin: 0;
    max-width: 420px;
    font-size: var(--fs-sm);
    line-height: var(--lh-body);
  }
</style>
