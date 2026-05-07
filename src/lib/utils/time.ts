/** Compact relative-time formatter matching the rest of the app's vibe. */
export function relTime(secs: number): string {
  const ms = Date.now() - secs * 1000;
  const m = Math.round(ms / 60000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m}m ago`;
  const h = Math.round(m / 60);
  if (h < 24) return `${h}h ago`;
  const d = Math.round(h / 24);
  if (d < 30) return `${d}d ago`;
  const mo = Math.round(d / 30);
  if (mo < 12) return `${mo}mo ago`;
  const y = Math.round(mo / 12);
  return `${y}y ago`;
}
