import { describe, it, expect } from 'vitest';
import { relTime } from './time';

describe('relTime', () => {
  // All cases pin `now` so the test isn't time-dependent.
  const NOW = 1_700_000_000;

  it('"just now" inside the first half-minute (rounding)', () => {
    // relTime uses Math.round on minutes, so anything <30s rounds to 0 → "just now".
    expect(relTime(NOW - 0, NOW)).toBe('just now');
    expect(relTime(NOW - 29, NOW)).toBe('just now');
  });

  it('minutes', () => {
    // 30s rounds up to 1m.
    expect(relTime(NOW - 30, NOW)).toBe('1m ago');
    expect(relTime(NOW - 60, NOW)).toBe('1m ago');
    expect(relTime(NOW - 5 * 60, NOW)).toBe('5m ago');
    // 59m exactly stays at 59m, but 59m30s rounds to 60m which trips the hour branch.
    expect(relTime(NOW - 59 * 60, NOW)).toBe('59m ago');
  });

  it('hours', () => {
    expect(relTime(NOW - 60 * 60, NOW)).toBe('1h ago');
    expect(relTime(NOW - 5 * 60 * 60, NOW)).toBe('5h ago');
    expect(relTime(NOW - 23 * 60 * 60, NOW)).toBe('23h ago');
  });

  it('days', () => {
    expect(relTime(NOW - 24 * 60 * 60, NOW)).toBe('1d ago');
    expect(relTime(NOW - 7 * 24 * 60 * 60, NOW)).toBe('7d ago');
  });

  it('months', () => {
    expect(relTime(NOW - 30 * 24 * 60 * 60, NOW)).toBe('1mo ago');
    expect(relTime(NOW - 90 * 24 * 60 * 60, NOW)).toBe('3mo ago');
  });

  it('years', () => {
    expect(relTime(NOW - 365 * 24 * 60 * 60, NOW)).toBe('1y ago');
    expect(relTime(NOW - 3 * 365 * 24 * 60 * 60, NOW)).toBe('3y ago');
  });

  it('uses Date.now()/1000 by default', () => {
    // Just confirm the call shape works without explicit `now`.
    const result = relTime(Math.floor(Date.now() / 1000) - 120);
    expect(result).toMatch(/m ago$/);
  });
});
