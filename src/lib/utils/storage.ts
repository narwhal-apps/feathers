/** SSR-safe integer read from localStorage with bounds-checking. Returns
 *  `fallback` if there's no window, the value is missing/non-numeric, or
 *  it falls outside [min, max] (defends against stale entries written
 *  under earlier bounds). */
export function loadStorageInt(
  key: string,
  fallback: number,
  min: number,
  max: number,
): number {
  if (typeof window === 'undefined') return fallback;
  const raw = window.localStorage.getItem(key);
  const n = raw === null ? NaN : parseInt(raw, 10);
  return Number.isFinite(n) && n >= min && n <= max ? n : fallback;
}
