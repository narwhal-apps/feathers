<script lang="ts">
  let { name, email, size = 18 }: { name: string; email?: string; size?: number } = $props();

  // Stable color from a string — small djb2 hash, then map into one of N hues.
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
</script>

<span
  class="avatar"
  style:width="{size}px"
  style:height="{size}px"
  style:font-size="{Math.max(8, Math.round(size * 0.42))}px"
  style:background="linear-gradient(135deg, hsl({hue}, 70%, 55%), hsl({(hue + 40) % 360}, 70%, 45%))"
  aria-label={name || 'unknown author'}
  title={name}
>{initials}</span>

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
  }
</style>
