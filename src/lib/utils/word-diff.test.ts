import { describe, it, expect } from 'vitest';
import { intraLineRanges, wrapHtmlRanges } from './word-diff';

describe('intraLineRanges', () => {
  it('marks identical lines as 100% similar with no ranges', () => {
    const r = intraLineRanges('hello world', 'hello world');
    expect(r.delRanges).toHaveLength(0);
    expect(r.addRanges).toHaveLength(0);
    expect(r.ratio).toBe(1);
  });

  it('finds single-word substitution ranges', () => {
    const r = intraLineRanges('hello world', 'hello there');
    // Old has "world" deleted, new has "there" added.
    expect(r.delRanges.length).toBeGreaterThan(0);
    expect(r.addRanges.length).toBeGreaterThan(0);
    // Common chars: "hello " → ratio = 6 / 11 ≈ 0.545
    expect(r.ratio).toBeGreaterThan(0.5);
    expect(r.ratio).toBeLessThan(0.6);
  });

  it('returns ratio < 1 when one side is empty', () => {
    const r1 = intraLineRanges('hello', '');
    expect(r1.ratio).toBe(0);
    expect(r1.delRanges).toHaveLength(1);
    expect(r1.delRanges[0]).toEqual([0, 5]);

    const r2 = intraLineRanges('', 'hello');
    expect(r2.ratio).toBe(0);
    expect(r2.addRanges[0]).toEqual([0, 5]);
  });

  it('handles both lines empty', () => {
    const r = intraLineRanges('', '');
    expect(r.delRanges).toHaveLength(0);
    expect(r.addRanges).toHaveLength(0);
    // longest = max(0,0,1) = 1, eqChars = 0 → 0.
    expect(r.ratio).toBe(0);
  });

  it('ratio uses the longer-side length as denominator', () => {
    const r = intraLineRanges('a', 'a much longer string');
    // Common is just "a" (1 char). longest = 20.
    expect(r.ratio).toBeCloseTo(1 / 20, 2);
  });

  it('returns ranges in old/new line coordinates', () => {
    const r = intraLineRanges('foo bar baz', 'foo qux baz');
    // The "bar" → "qux" substitution sits at offset 4-7 in both.
    const del = r.delRanges[0];
    const add = r.addRanges[0];
    expect(del).toBeDefined();
    expect(add).toBeDefined();
    // Verify the slices match the diffed words.
    expect('foo bar baz'.slice(del![0], del![1])).toBe('bar');
    expect('foo qux baz'.slice(add![0], add![1])).toBe('qux');
  });
});

describe('wrapHtmlRanges', () => {
  it('returns the input unchanged when no ranges given', () => {
    expect(wrapHtmlRanges('hello', [], 'mark')).toBe('hello');
  });

  it('wraps a single contiguous range with intra-start + intra-end', () => {
    const out = wrapHtmlRanges('hello world', [[6, 11]], 'mark');
    expect(out).toBe('hello <span class="mark intra-start intra-end">world</span>');
  });

  it('skips over HTML tags without breaking them', () => {
    // The leading <span> tag spans plain-text offsets 0-5 ("hello").
    // The range [6, 11] targets "world" inside the second span.
    const html = '<span>hello</span> <span>world</span>';
    const out = wrapHtmlRanges(html, [[6, 11]], 'mark');
    // The wrapper opens at "world", closes after "world", around or
    // inside the inner <span> — we just assert the marker wraps the
    // right text and tag structure stays intact.
    expect(out).toContain('class="mark intra-start');
    expect(out).toContain('intra-end');
    expect(out).toContain('world');
    // No mangled tag fragments.
    expect(out).not.toMatch(/<span[^>]*<span/);
  });

  it('handles multiple non-contiguous ranges', () => {
    const out = wrapHtmlRanges('a b c d e', [[0, 1], [4, 5]], 'mark');
    // Each range gets its own start+end markers.
    expect(out).toContain('intra-start intra-end">a</span>');
    expect(out).toContain('intra-start intra-end">c</span>');
  });

  it('preserves HTML entities (treats them as one plain-text char)', () => {
    // "&amp;" is one logical char in plain text.
    const html = 'x&amp;y';
    // Range covers "&" (offset 1-2 in plain text).
    const out = wrapHtmlRanges(html, [[1, 2]], 'mark');
    expect(out).toContain('&amp;');
    expect(out).toContain('class="mark');
  });

  it('marks the first wrapper of a split range with intra-start, last with intra-end', () => {
    // Range spans across a tag boundary so it splits into two wrappers.
    const html = '<span>aa</span><span>bb</span>';
    const out = wrapHtmlRanges(html, [[0, 4]], 'mark');
    // First wrapper has intra-start, last has intra-end. Inner ones (none
    // here) would have neither.
    expect(out).toContain('class="mark intra-start"');
    expect(out).toContain('class="mark intra-end"');
    // It should NOT collapse into one wrapper — the tag would be mis-nested.
    const wrapperCount = (out.match(/class="mark/g) ?? []).length;
    expect(wrapperCount).toBe(2);
  });
});
