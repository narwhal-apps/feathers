/**
 * Word-level diff for intra-line highlighting in the diff view.
 *
 * Delegates to the `diff` package (Myers' algorithm with whitespace-aware
 * word tokenisation), then projects the resulting segment list into
 * character ranges in old/new line coordinates.
 *
 * Also returns a similarity ratio so callers can skip the highlight when
 * the two lines barely overlap (full-line repaint reads as noise, not
 * signal).
 */

import { diffWordsWithSpace } from 'diff';

export type IntraResult = {
  delRanges: Array<[number, number]>;
  addRanges: Array<[number, number]>;
  /** Common-character count divided by the longer line's length. */
  ratio: number;
};

/** Skip Myers above this line length. `diffWordsWithSpace` is O(N+D²) where
 *  D is the edit distance — two long, dissimilar lines (minified bundles,
 *  base64, generated SQL) push it into multi-second territory. The caller
 *  already discards low-similarity results downstream; running the
 *  algorithm just to throw the answer away is the freeze. */
export const MAX_INTRA_LINE_LEN = 1000;

/** Skip Myers when the two lines differ in length by more than this much
 *  (shorter / longer). They can't possibly meet the downstream similarity
 *  bar, so the only thing the algorithm can do is burn cycles. */
const MIN_LENGTH_RATIO = 0.3;

export function intraLineRanges(oldText: string, newText: string): IntraResult {
  // Cheap pre-checks before letting Myers run. Both bail with ratio: 0 so
  // the caller's `< 0.3` similarity gate skips the highlight as if Myers
  // had returned a poor match.
  if (oldText.length > MAX_INTRA_LINE_LEN || newText.length > MAX_INTRA_LINE_LEN) {
    return { delRanges: [], addRanges: [], ratio: 0 };
  }
  const longest = Math.max(oldText.length, newText.length);
  const shortest = Math.min(oldText.length, newText.length);
  // Only gate on length-ratio for *long* lines. Myers is O(N+D²); the cost
  // only bites once the lines are non-trivial in size. Below the floor the
  // algorithm is fast enough that the gate would just mask cases the
  // existing API contract already covered (e.g. one-side-empty pairs).
  if (longest > 32 && shortest / longest < MIN_LENGTH_RATIO) {
    return { delRanges: [], addRanges: [], ratio: 0 };
  }
  // `longest || 1` keeps the eqChars/longest division safe for the both-empty
  // case (matches the original behaviour: longest=0 → ratio=0).
  const denom = longest || 1;

  const segs = diffWordsWithSpace(oldText, newText);
  let oldOff = 0;
  let newOff = 0;
  let eqChars = 0;
  const delRanges: Array<[number, number]> = [];
  const addRanges: Array<[number, number]> = [];
  for (const seg of segs) {
    const len = seg.value.length;
    if (seg.added) {
      addRanges.push([newOff, newOff + len]);
      newOff += len;
    } else if (seg.removed) {
      delRanges.push([oldOff, oldOff + len]);
      oldOff += len;
    } else {
      eqChars += len;
      oldOff += len;
      newOff += len;
    }
  }
  return { delRanges, addRanges, ratio: eqChars / denom };
}

/**
 * Wrap plain-text character ranges of `html` in <span class="{cls}"> tags.
 *
 * The HTML is treated as syntax-highlighted output (tags + text + entities);
 * `ranges` are offsets in the plain text the HTML represents. Wrappers are
 * closed before each tag and reopened after, so the result stays valid even
 * when ranges cross syntax-span boundaries — the trade-off is multiple
 * wrapper spans for one logical range.
 *
 * The first wrapper of each range gets `intra-start` and the last gets
 * `intra-end` so a stylesheet can round just the leading and trailing
 * edges of a logical range — the inner wrappers (split by syntax spans)
 * stay flat, so the whole thing reads as one continuous pill.
 */
export function wrapHtmlRanges(
  html: string,
  ranges: Array<[number, number]>,
  cls: string,
): string {
  if (!ranges.length) return html;
  const sorted = [...ranges].sort((a, b) => a[0] - b[0]);
  let out = '';
  let plainOff = 0;
  let i = 0;
  let inIntra = false;
  let rIdx = 0;
  let inTag = false;
  /** Index in `out` of the most-recently-opened wrapper's `class="..."`
   *  closing quote — so we can append `intra-end` when the range exits. */
  let lastOpenClassQuote = -1;

  function open() {
    const r = sorted[rIdx];
    const isStart = !!r && plainOff === r[0];
    const klass = cls + (isStart ? ' intra-start' : '');
    out += `<span class="${klass}`;
    lastOpenClassQuote = out.length;
    out += '">';
    inIntra = true;
  }
  function closeWithEnd() {
    if (lastOpenClassQuote >= 0) {
      out =
        out.slice(0, lastOpenClassQuote) + ' intra-end' + out.slice(lastOpenClassQuote);
    }
    out += '</span>';
    inIntra = false;
    lastOpenClassQuote = -1;
  }
  function closeForTag() {
    out += '</span>';
    inIntra = false;
    lastOpenClassQuote = -1;
  }
  /** Run before every plain-text char or entity: closes the range if its
   *  end lies at this offset (carrying intra-end), then opens a new
   *  range if its start lies at or before this offset. */
  function syncIntra() {
    while (rIdx < sorted.length && plainOff >= sorted[rIdx][1]) {
      if (inIntra) closeWithEnd();
      rIdx++;
    }
    if (!inIntra && rIdx < sorted.length && plainOff >= sorted[rIdx][0]) open();
  }

  while (i < html.length) {
    const ch = html[i];
    if (ch === '<') {
      // If a tag boundary lands exactly on the range's end, close as a
      // real range exit (carry intra-end). Otherwise it's a forced
      // mid-range close and we'll re-open after the tag.
      while (rIdx < sorted.length && plainOff >= sorted[rIdx][1]) {
        if (inIntra) closeWithEnd();
        rIdx++;
      }
      if (inIntra) closeForTag();
      inTag = true;
      out += ch;
      i++;
    } else if (inTag) {
      out += ch;
      if (ch === '>') inTag = false;
      i++;
    } else if (ch === '&') {
      const semi = html.indexOf(';', i);
      const next = semi >= 0 ? semi + 1 : i + 1;
      syncIntra();
      out += html.slice(i, next);
      plainOff++;
      i = next;
    } else {
      syncIntra();
      out += ch;
      plainOff++;
      i++;
    }
  }
  // End-of-input cleanup: a range that runs to the very last char.
  while (rIdx < sorted.length && plainOff >= sorted[rIdx][1]) {
    if (inIntra) closeWithEnd();
    rIdx++;
  }
  if (inIntra) closeWithEnd();
  return out;
}
