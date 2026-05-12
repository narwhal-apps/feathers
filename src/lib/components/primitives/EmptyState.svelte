<script lang="ts">
  type Size = 'sm' | 'md' | 'lg';

  let {
    illustration,
    title,
    description,
    size = 'md',
    actions,
  }: {
    illustration?: string;
    title: string;
    description?: string;
    size?: Size;
    actions?: import('svelte').Snippet;
  } = $props();

  const illustrationUrl = $derived(
    illustration ? `url(/illustrations/${illustration}.svg)` : null,
  );
</script>

<div class="empty size-{size}">
  {#if illustrationUrl}
    <div
      class="illustration"
      style="--ill-src: {illustrationUrl}"
      aria-hidden="true"
    ></div>
  {/if}
  <p class="title">{title}</p>
  {#if description}<p class="desc">{description}</p>{/if}
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: var(--sp-2);
    padding: var(--sp-5) var(--sp-3);
    color: var(--fg-muted);
    /* Sit visually a bit above center; pure-center reads as "lost in
       space" — a slight upward bias feels intentional. */
    margin-top: -4vh;
  }

  /* Mask-based recolor so we can tint the silhouette via theme tokens
     without editing the source SVGs. The illustrations are single-path
     black on transparent — perfect mask material. */
  .illustration {
    background-color: var(--fg-subtle);
    -webkit-mask-image: var(--ill-src);
    mask-image: var(--ill-src);
    -webkit-mask-size: contain;
    mask-size: contain;
    -webkit-mask-repeat: no-repeat;
    mask-repeat: no-repeat;
    -webkit-mask-position: center;
    mask-position: center;
    opacity: 0.35;
    margin-bottom: var(--sp-2);
    transition: opacity var(--t-fast);
  }
  .size-sm .illustration { width: 72px;  height: 72px; }
  .size-md .illustration { width: 120px; height: 120px; }
  .size-lg .illustration { width: 180px; height: 180px; }

  .title {
    margin: 0;
    color: var(--fg-muted);
    font-size: var(--fs-md);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-tight);
  }
  .size-sm .title { font-size: var(--fs-sm); }

  .desc {
    margin: 0;
    color: var(--fg-subtle);
    font-size: var(--fs-sm);
    max-width: 36ch;
    line-height: 1.5;
  }
  .size-sm .desc { font-size: var(--fs-xs); }

  .actions {
    margin-top: var(--sp-3);
    display: flex;
    gap: var(--sp-2);
  }
</style>
