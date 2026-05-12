/**
 * Word-level diff for intra-line highlighting in the diff view.
 *
 * Tokenizes both strings into runs of word-chars / whitespace / punctuation,
 * runs Hunt–McIlroy LCS, and returns the character ranges (in plain-text
 * coordinates) that are added on the new side and removed on the old side.
 *
 * Also returns a similarity ratio so callers can skip the highlight when
 * the two lines barely overlap (full-line repaint reads as noise, not signal).
 */

type DiffSeg = { kind: 'eq' | 'del' | 'add'; text: string };

export type IntraResult = {
  delRanges: Array<[number, number]>;
  addRanges: Array<[number, number]>;
  /** LCS-character-count divided by the longer line's length. */
  ratio: number;
};

function tokenize(s: string): string[] {
  return s.match(/[\w$]+|\s+|[^\s\w$]/g) ?? [];
}

function diffTokens(a: string[], b: string[]): DiffSeg[] {
  const m = a.length;
  const n = b.length;
  // LCS DP — small lines so the O(m·n) cost is fine.
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array<number>(n + 1).fill(0));
  for (let i = 0; i < m; i++) {
    for (let j = 0; j < n; j++) {
      dp[i + 1][j + 1] = a[i] === b[j] ? dp[i][j] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
  }
  const out: DiffSeg[] = [];
  let i = m;
  let j = n;
  while (i > 0 && j > 0) {
    if (a[i - 1] === b[j - 1]) { out.push({ kind: 'eq', text: a[--i] }); j--; }
    else if (dp[i - 1][j] >= dp[i][j - 1]) { out.push({ kind: 'del', text: a[--i] }); }
    else { out.push({ kind: 'add', text: b[--j] }); }
  }
  while (i > 0) out.push({ kind: 'del', text: a[--i] });
  while (j > 0) out.push({ kind: 'add', text: b[--j] });
  out.reverse();
  // Coalesce adjacent same-kind segments so ranges are tight.
  const merged: DiffSeg[] = [];
  for (const seg of out) {
    const last = merged[merged.length - 1];
    if (last && last.kind === seg.kind) last.text += seg.text;
    else merged.push({ ...seg });
  }
  return merged;
}

export function intraLineRanges(oldText: string, newText: string): IntraResult {
  const segs = diffTokens(tokenize(oldText), tokenize(newText));
  let oldOff = 0;
  let newOff = 0;
  let eqChars = 0;
  const delRanges: Array<[number, number]> = [];
  const addRanges: Array<[number, number]> = [];
  for (const seg of segs) {
    const len = seg.text.length;
    if (seg.kind === 'eq') {
      eqChars += len;
      oldOff += len;
      newOff += len;
    } else if (seg.kind === 'del') {
      delRanges.push([oldOff, oldOff + len]);
      oldOff += len;
    } else {
      addRanges.push([newOff, newOff + len]);
      newOff += len;
    }
  }
  const longest = Math.max(oldText.length, newText.length, 1);
  return { delRanges, addRanges, ratio: eqChars / longest };
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
