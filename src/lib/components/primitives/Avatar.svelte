<script lang="ts">
  let {
    name,
    email,
    url,
    size = 18,
  }: {
    name: string;
    email?: string;
    /** Real avatar URL (e.g. GitHub). Falls back to gradient + initials. */
    url?: string | null;
    size?: number;
  } = $props();

  function hashHue(s: string): number {
    let h = 5381;
    for (let i = 0; i < s.length; i++) h = ((h << 5) + h + s.charCodeAt(i)) & 0xffffffff;
    return Math.abs(h) % 360;
  }

  const seed = $derived(email || name || '?');
  const hue = $derived(hashHue(seed));
  const initials = $derived(
    (name || '?')
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((p) => p[0]?.toUpperCase() ?? '')
      .join('') || '?',
  );

  // If the URL fails to load, fall back to the gradient.
  let imgFailed = $state(false);
  const showImg = $derived(!!url && !imgFailed);
</script>

<span
  class="avatar"
  class:has-img={showImg}
  style:width="{size}px"
  style:height="{size}px"
  style:font-size="{Math.max(8, Math.round(size * 0.42))}px"
  style:background={showImg
    ? 'transparent'
    : `linear-gradient(135deg, hsl(${hue}, 70%, 55%), hsl(${(hue + 40) % 360}, 70%, 45%))`}
  aria-label={name || 'unknown author'}
  title={name}
>
  {#if showImg}
    <img src={url} alt="" referrerpolicy="no-referrer" onerror={() => (imgFailed = true)} />
  {:else}
    {initials}
  {/if}
</span>

<style>
  .avatar {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r-pill);
    color: #fff;
    font-family: var(--font-sans);
    font-weight: var(--weight-bold);
    text-shadow: 0 1px 0 rgba(0, 0, 0, 0.25);
    flex-shrink: 0;
    user-select: none;
    overflow: hidden;
  }
  .avatar.has-img {
    text-shadow: none;
    background: var(--bg-elev-2);
  }
  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
</style>
