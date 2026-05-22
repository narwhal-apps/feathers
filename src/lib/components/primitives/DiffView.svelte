<script lang="ts">
  import type { DiffPayload, DiffFile, DiffHunk, DiffLine, FileStatus, ThemeName } from '$lib/types';
  import { browser } from '$app/environment';
  import { detectLang, highlightLines } from '$lib/syntax/highlighter';
  import { intraLineRanges, wrapHtmlRanges } from '$lib/utils/word-diff';
  import { theme } from '$lib/stores/theme.svelte';
  import { SvelteMap, SvelteSet } from 'svelte/reactivity';
  import FileIcon from '$lib/components/file/FileIcon.svelte';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';

  type ViewMode = 'unified' | 'split';
  const VIEW_KEY = 'feathers:diff-view-mode';
  let mode = $state<ViewMode>(
    browser
      ? ((localStorage.getItem(VIEW_KEY) as ViewMode | null) ?? 'unified')
      : 'unified',
  );
  $effect(() => {
    if (browser) localStorage.setItem(VIEW_KEY, mode);
  });

  /** Index-wise pairs of consecutive del→add runs in a hunk. For runs of
   *  unequal length, only min(N, M) pairs are returned — the leftover
   *  lines are pure adds or deletes and don't get intra-line highlighting. */
  function consecutiveDelAddPairs(lines: DiffLine[]): Array<[number, number]> {
    const pairs: Array<[number, number]> = [];
    let i = 0;
    while (i < lines.length) {
      if (lines[i].kind !== 'del') { i++; continue; }
      const dStart = i;
      while (i < lines.length && lines[i].kind === 'del') i++;
      const aStart = i;
      while (i < lines.length && lines[i].kind === 'add') i++;
      const dCount = aStart - dStart;
      const aCount = i - aStart;
      const n = Math.min(dCount, aCount);
      for (let k = 0; k < n; k++) pairs.push([dStart + k, aStart + k]);
    }
    return pairs;
  }

  // Pair consecutive del/add runs into rows for the split view. Context lines
  // align on both sides; lone adds/dels leave the opposite cell empty. Each
  // entry carries the line's original index so we can look up its highlighted
  // HTML in `hl[file.path][hunkIdx][lineIdx]`.
  type SplitRow = {
    old: { line: DiffLine; idx: number } | null;
    new: { line: DiffLine; idx: number } | null;
  };
  function pairLines(lines: DiffLine[]): SplitRow[] {
    const rows: SplitRow[] = [];
    let i = 0;
    while (i < lines.length) {
      const l = lines[i];
      if (l.kind === 'ctx') {
        rows.push({ old: { line: l, idx: i }, new: { line: l, idx: i } });
        i++;
        continue;
      }
      const dels: { line: DiffLine; idx: number }[] = [];
      const adds: { line: DiffLine; idx: number }[] = [];
      while (i < lines.length && lines[i].kind === 'del') {
        dels.push({ line: lines[i], idx: i });
        i++;
      }
      while (i < lines.length && lines[i].kind === 'add') {
        adds.push({ line: lines[i], idx: i });
        i++;
      }
      const n = Math.max(dels.length, adds.length);
      for (let k = 0; k < n; k++) {
        rows.push({ old: dels[k] ?? null, new: adds[k] ?? null });
      }
    }
    return rows;
  }

  let {
    payload,
    fileHref,
    onDiscardHunk,
  }: {
    payload: DiffPayload | null;
    /** Optional resolver returning a remote URL for the file, or null when
     *  the file shouldn't be linkable (e.g. newly added). */
    fileHref?: (file: DiffFile) => string | null;
    /** When provided, each hunk gets a "discard hunk" button calling this. */
    onDiscardHunk?: (file: DiffFile, hunkIndex: number) => void;
  } = $props();

  // hl.get(file.path)?.[hunkIdx]?.[lineIdx] = highlighted HTML for that line.
  // SvelteMap so per-file publish is O(1) instead of spreading a growing record.
  const hl = new SvelteMap<string, string[][]>();

  // Track which payload + theme the cached highlights were built for.
  // Highlighting kicks off lazily per file when it enters viewport — see
  // `maybeHighlightFile` below. The all-files-up-front loop was the
  // freeze: Shiki's `codeToTokensBase` is synchronous on the main thread,
  // so a 30-file commit blocked the UI for the sum of all 30's tokenize
  // costs even when only file #1 was on screen.
  let hlPayloadRef: DiffPayload | null = null;
  let hlTheme: ThemeName | null = null;
  /** Files currently being highlighted — dedupes overlapping triggers
   *  (effect re-running, visibility callback firing, etc.). */
  const hlInFlight = new Set<string>();

  $effect(() => {
    const t = theme.effective;
    const p = payload;
    if (!p) {
      hl.clear();
      hlPayloadRef = null;
      hlTheme = null;
      return;
    }
    if (hlPayloadRef === p && hlTheme === t) return;
    const themeChanged = hlTheme !== t;
    hlPayloadRef = p;
    hlTheme = t;
    // Keep existing highlights visible while re-highlighting so the UI
    // doesn't flash unstyled text. Old entries are overwritten once the
    // new async highlight completes. Only clear on theme change (colors
    // would be wrong) or when a file is no longer in the payload.
    if (themeChanged) {
      hl.clear();
    } else {
      const currentPaths = new Set(p.files.map((f) => f.path));
      for (const key of hl.keys()) {
        if (!currentPaths.has(key)) hl.delete(key);
      }
    }
    hlInFlight.clear();
    for (const file of p.files) {
      if (visibleFiles.has(file.path)) maybeHighlightFile(file, true);
    }
  });

  /** Kick off highlight for one file. Idempotent: skips files already
   *  cached or in flight, binary files, and unsupported languages.
   *  Pass `force` to re-highlight even if cached (used on payload change). */
  function maybeHighlightFile(file: DiffFile, force = false): void {
    if (!hlPayloadRef || !hlTheme) return;
    if (file.binary) return;
    if (!force && hl.has(file.path)) return;
    if (hlInFlight.has(file.path)) return;
    const lang = detectLang(file.path);
    if (!lang) return;
    const t = hlTheme;
    const payloadAtStart = hlPayloadRef;
    hlInFlight.add(file.path);
    void (async () => {
      try {
        const hunkHtml: string[][] = [];
        for (const hunk of file.hunks) {
          // Bail if the user has navigated away or flipped theme mid-flight.
          if (hlPayloadRef !== payloadAtStart || hlTheme !== t) return;
          const lines = hunk.lines.map((l) => l.text);
          const html = await highlightLines(lines, lang, t);
          // Word-level intra-line highlighting. Walks consecutive del→add
          // runs and pairs them index-wise; for each pair we mark the
          // changed substrings in both the del and the add line. Skipped
          // when the two lines barely overlap — full-line repaint reads
          // as noise, not signal. (`intraLineRanges` cheaply returns
          // ratio: 0 for over-long or wildly different lines so we don't
          // pay for Myers in those cases.)
          const pairs = consecutiveDelAddPairs(hunk.lines);
          for (const [delIdx, addIdx] of pairs) {
            const delText = hunk.lines[delIdx].text;
            const addText = hunk.lines[addIdx].text;
            const r = intraLineRanges(delText, addText);
            if (r.ratio < 0.3) continue;
            html[delIdx] = wrapHtmlRanges(html[delIdx], r.delRanges, 'intra-del');
            html[addIdx] = wrapHtmlRanges(html[addIdx], r.addRanges, 'intra-add');
          }
          hunkHtml.push(html);
          // Yield between hunks so the main thread can render frames /
          // handle input even while a big file is being highlighted.
          await new Promise<void>((r) => setTimeout(r));
        }
        if (hlPayloadRef !== payloadAtStart || hlTheme !== t) return;
        hl.set(file.path, hunkHtml);
      } finally {
        hlInFlight.delete(file.path);
      }
    })();
  }

  // Memoize pairLines per hunk so split-view rows aren't recomputed on every
  // render. Map keyed by hunk reference; rebuilds only when mode/payload changes.
  const splitRows = $derived.by(() => {
    const m = new Map<DiffHunk, SplitRow[]>();
    if (mode !== 'split' || !payload) return m;
    for (const file of payload.files) {
      for (const hunk of file.hunks) {
        m.set(hunk, pairLines(hunk.lines));
      }
    }
    return m;
  });

  // Memoize line counts per file (cheap but called multiple times per render).
  const fileCounts = $derived.by(() => {
    const m = new Map<DiffFile, { adds: number; dels: number }>();
    if (!payload) return m;
    for (const file of payload.files) m.set(file, countLines(file));
    return m;
  });

  // Virtualize at the file level: render hunks only when the file scrolls
  // within ~500px of the viewport. One-shot IntersectionObserver per file —
  // once visible, the file stays mounted (preserves browser ⌘F).
  // Highlight is kicked off here too so we don't pay Shiki's main-thread
  // tokenize cost for off-screen files.
  const visibleFiles = new SvelteSet<string>();

  function visible(node: HTMLElement, file: DiffFile) {
    const io = new IntersectionObserver((entries) => {
      for (const e of entries) {
        if (e.isIntersecting) {
          visibleFiles.add(file.path);
          maybeHighlightFile(file);
          io.unobserve(node);
        }
      }
    }, { rootMargin: '500px' });
    io.observe(node);
    return { destroy: () => io.disconnect() };
  }

  const isVisible = (path: string) => visibleFiles.has(path);

  // Second virtualization layer: even within a visible file, only hunks
  // within ~500px of the viewport mount their lines body. A single huge
  // file used to mount tens of thousands of <div> nodes the moment its
  // article scrolled in — that DOM-mount pass was the residual freeze
  // after the highlighter caps. With per-hunk IO, mount cost becomes
  // O(viewport) instead of O(file).
  //
  // Key by `file.path:hunkIdx` (positional, not header-based). On a
  // constantly-changing file every save shifts hunk line numbers — and
  // therefore hunk headers. Index-based keys keep the same DOM node
  // (and visibility/observer state) stable across those edits so the
  // hunk doesn't unmount + remount + re-fire IO on every save.
  const visibleHunks = new SvelteSet<string>();

  /** Per-hunk render cap. The hunk-level virtualization above stops the
   *  freeze for many-hunk files, but a file with one huge hunk (a log
   *  append, generated code, a binary blob diffed as text) still mounts
   *  every line at once when that single hunk's IO fires. Render only
   *  the first N lines by default; the user can expand a hunk if they
   *  want to read the rest. */
  const MAX_LINES_PER_HUNK_RENDER = 500;
  /** Hunks the user has explicitly expanded past the cap. Keyed by
   *  `file.path:hunkIdx` — same key shape as `visibleHunks`. */
  const expandedHunks = new SvelteSet<string>();

  function visibleHunk(node: HTMLElement, key: string) {
    const io = new IntersectionObserver((entries) => {
      for (const e of entries) {
        if (e.isIntersecting) {
          visibleHunks.add(key);
          io.unobserve(node);
        }
      }
    }, { rootMargin: '500px' });
    io.observe(node);
    return { destroy: () => io.disconnect() };
  }

  const isHunkVisible = (key: string) => visibleHunks.has(key);

  function estimateHunkHeight(hunk: DiffHunk): number {
    // 18px line-height + 4px vertical padding ≈ 22px per line.
    return Math.max(22, hunk.lines.length * 22);
  }

  /** Per-session collapsed state — clicking a file header toggles its body. */
  const collapsed = new SvelteSet<string>();
  function toggleCollapsed(path: string) {
    if (collapsed.has(path)) collapsed.delete(path);
    else collapsed.add(path);
  }

  function estimateHeight(file: DiffFile): number {
    let lines = 0;
    for (const h of file.hunks) lines += h.lines.length;
    return Math.max(60, lines * 22 + file.hunks.length * 36);
  }

  // Visibility persists across payload changes by design. With keyed each
  // blocks, an article whose file.path is unchanged keeps the same DOM node
  // — and our `use:visible` action is one-shot (it unobserves after first
  // intersection). Clearing visibleFiles on payload change would strand
  // those reused nodes as permanent placeholders. Letting the set grow
  // across the session is fine: the entries are just file paths, and a
  // returning path renders its hunks immediately, which is what we want.

  function basename(p: string): string {
    const i = p.lastIndexOf('/');
    return i < 0 ? p : p.slice(i + 1);
  }
  function dirname(p: string): string {
    const i = p.lastIndexOf('/');
    return i < 0 ? '' : p.slice(0, i);
  }

  function countLines(file: DiffFile): { adds: number; dels: number } {
    let adds = 0, dels = 0;
    for (const h of file.hunks) {
      for (const l of h.lines) {
        if (l.kind === 'add') adds++;
        else if (l.kind === 'del') dels++;
      }
    }
    return { adds, dels };
  }

  type RenameParts = {
    prefix: string;
    oldChanged: string;
    newChanged: string;
    suffix: string;
  };
  function highlightRenameParts(oldPath: string, newPath: string): RenameParts {
    let prefixLen = 0;
    const minLen = Math.min(oldPath.length, newPath.length);
    while (prefixLen < minLen && oldPath[prefixLen] === newPath[prefixLen]) prefixLen++;
    let suffixLen = 0;
    while (
      suffixLen < minLen - prefixLen &&
      oldPath[oldPath.length - 1 - suffixLen] === newPath[newPath.length - 1 - suffixLen]
    ) suffixLen++;
    return {
      prefix: oldPath.slice(0, prefixLen),
      oldChanged: oldPath.slice(prefixLen, oldPath.length - suffixLen),
      newChanged: newPath.slice(prefixLen, newPath.length - suffixLen),
      suffix: oldPath.slice(oldPath.length - suffixLen),
    };
  }

  type StatusTone = 'add' | 'del' | 'mod' | 'ren' | 'mod';
  function statusLabel(s: FileStatus): { text: string; tone: StatusTone } {
    switch (s) {
      case 'added':      return { text: 'Added',     tone: 'add' };
      case 'untracked':  return { text: 'New file',  tone: 'add' };
      case 'deleted':    return { text: 'Deleted',   tone: 'del' };
      case 'renamed':    return { text: 'Renamed',   tone: 'ren' };
      case 'modified':   return { text: 'Modified',  tone: 'mod' };
      case 'typechange': return { text: 'Type changed', tone: 'mod' };
      case 'conflicted': return { text: 'Conflicted', tone: 'del' };
    }
  }
</script>

{#if !payload || payload.files.length === 0}
  <div class="empty">No changes.</div>
{:else}
  <div class="controls">
    <div class="seg" role="group" aria-label="Diff view mode">
      <button
        type="button"
        class:on={mode === 'unified'}
        onclick={() => (mode = 'unified')}
        aria-pressed={mode === 'unified'}
        title="Unified diff"
      >
        <Icon name="AlignJustify" size={12} /> Unified
      </button>
      <button
        type="button"
        class:on={mode === 'split'}
        onclick={() => (mode = 'split')}
        aria-pressed={mode === 'split'}
        title="Side-by-side diff"
      >
        <Icon name="Columns2" size={12} /> Split
      </button>
    </div>
  </div>
  {#each payload.files as file (file.path + ':' + (file.old_path ?? ''))}
    {@const counts = fileCounts.get(file) ?? { adds: 0, dels: 0 }}
    {@const lbl = statusLabel(file.status)}
    {@const dir = dirname(file.path)}
    {@const href = fileHref ? fileHref(file) : null}
    {@const isCollapsed = collapsed.has(file.path)}
    <article class="file" class:collapsed={isCollapsed} use:visible={file}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <header
        class="file-header"
        onclick={() => toggleCollapsed(file.path)}
        onkeydown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            toggleCollapsed(file.path);
          }
        }}
        role="button"
        tabindex="0"
        aria-expanded={!isCollapsed}
        aria-label={isCollapsed ? `Expand ${file.path}` : `Collapse ${file.path}`}
      >
        <span class="chevron" aria-hidden="true">
          <Icon name={isCollapsed ? 'ChevronRight' : 'ChevronDown'} size={12} />
        </span>
        <FileIcon fileName={basename(file.path)} size={14} />
        <span class="name">
          <span class="basename">{basename(file.path)}</span>
          {#if dir}<span class="dir">{dir}</span>{/if}
        </span>
        {#if file.old_path && file.old_path !== file.path}
          <span class="old" title="Renamed from {file.old_path}">← {basename(file.old_path)}</span>
        {/if}
        <span class="status status-{lbl.tone}">{lbl.text}</span>
        {#if !file.binary && (counts.adds > 0 || counts.dels > 0)}
          <span class="counts">
            {#if counts.adds > 0}
              <span class="count-add">+{counts.adds}</span>
            {/if}
            {#if counts.dels > 0}
              <span class="count-del">−{counts.dels}</span>
            {/if}
          </span>
        {/if}
        {#if href}
          <button
            class="open-link"
            type="button"
            title="Open on remote"
            aria-label="Open {file.path} on remote"
            onclick={(e) => { e.stopPropagation(); openUrl(href); }}
          >
            <Icon name="ExternalLink" size={12} />
          </button>
        {/if}
      </header>
      {#if isCollapsed}
        <!-- body hidden -->
      {:else if file.binary}
        <div class="binary">Binary file — diff not shown.</div>
      {:else if file.status === 'renamed' && file.old_path && file.hunks.length === 0}
        {@const rp = highlightRenameParts(file.old_path, file.path)}
        <div class="rename-banner">
          <Icon name="ArrowRight" size={14} />
          <span class="rename-paths">
            <span class="rename-path">
              {rp.prefix}<span class="rename-highlight rename-del">{rp.oldChanged}</span>{rp.suffix}
            </span>
            <span class="rename-arrow">→</span>
            <span class="rename-path">
              {rp.prefix}<span class="rename-highlight rename-add">{rp.newChanged}</span>{rp.suffix}
            </span>
          </span>
        </div>
      {:else if !isVisible(file.path)}
        <div class="file-placeholder" style:height="{estimateHeight(file)}px"></div>
      {:else if mode === 'unified'}
        {@const fileHl = hl.get(file.path)}
        <div class="body">
          {#each file.hunks as hunk, hunkIdx (hunkIdx)}
            {@const hkey = file.path + ':' + hunkIdx}
            {@const expanded = expandedHunks.has(hkey)}
            {@const cap = expanded ? hunk.lines.length : Math.min(hunk.lines.length, MAX_LINES_PER_HUNK_RENDER)}
            {@const hiddenLines = hunk.lines.length - cap}
            <div class="hunk" use:visibleHunk={hkey}>
              <div class="hunk-header">
                <span class="hunk-header-text">{hunk.header}</span>
                {#if onDiscardHunk}
                  <button
                    class="hunk-discard"
                    type="button"
                    title="Discard this hunk"
                    aria-label="Discard hunk"
                    onclick={() => onDiscardHunk?.(file, hunkIdx)}
                  >
                    <Icon name="Undo2" size={11} />
                  </button>
                {/if}
              </div>
              {#if isHunkVisible(hkey)}
                <div class="lines">
                  {#each hunk.lines.slice(0, cap) as line, lineIdx (lineIdx)}
                    <div class="line line-{line.kind}">
                      <span class="ln ln-old">{line.old_no ?? ''}</span>
                      <span class="ln ln-new">{line.new_no ?? ''}</span>
                      <span class="prefix">{line.kind === 'add' ? '+' : line.kind === 'del' ? '−' : ' '}</span>
                      {#if fileHl?.[hunkIdx]?.[lineIdx] != null}
                        <span class="text">{@html fileHl[hunkIdx][lineIdx]}</span>
                      {:else}
                        <span class="text">{line.text}</span>
                      {/if}
                    </div>
                  {/each}
                </div>
                {#if hiddenLines > 0}
                  <button class="hunk-expand" type="button" onclick={() => expandedHunks.add(hkey)}>
                    Show {hiddenLines.toLocaleString()} more line{hiddenLines === 1 ? '' : 's'}
                  </button>
                {/if}
              {:else}
                <div class="hunk-placeholder" style:height="{estimateHunkHeight(hunk)}px"></div>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <!-- Split (side-by-side) view -->
        {@const fileHl = hl.get(file.path)}
        <div class="body split-body">
          {#each file.hunks as hunk, hunkIdx (hunkIdx)}
            {@const hkey = file.path + ':' + hunkIdx}
            {@const expanded = expandedHunks.has(hkey)}
            <div class="hunk split-hunk" use:visibleHunk={hkey}>
              <div class="hunk-header">
                <span class="hunk-header-text">{hunk.header}</span>
                {#if onDiscardHunk}
                  <button
                    class="hunk-discard"
                    type="button"
                    title="Discard this hunk"
                    aria-label="Discard hunk"
                    onclick={() => onDiscardHunk?.(file, hunkIdx)}
                  >
                    <Icon name="Undo2" size={11} />
                  </button>
                {/if}
              </div>
              {#if isHunkVisible(hkey)}
                {@const allRows = splitRows.get(hunk) ?? []}
                {@const rowCap = expanded ? allRows.length : Math.min(allRows.length, MAX_LINES_PER_HUNK_RENDER)}
                {@const hiddenRows = allRows.length - rowCap}
                <div class="split-rows">
                  {#each allRows.slice(0, rowCap) as row, rowIdx (rowIdx)}
                    <div class="split-row">
                      <div class="split-side {row.old ? `line-${row.old.line.kind}` : 'line-empty'}">
                        <span class="ln">{row.old?.line.old_no ?? ''}</span>
                        <span class="prefix">{row.old?.line.kind === 'del' ? '−' : ' '}</span>
                        {#if row.old && fileHl?.[hunkIdx]?.[row.old.idx] != null}
                          <span class="text">{@html fileHl[hunkIdx][row.old.idx]}</span>
                        {:else}
                          <span class="text">{row.old?.line.text ?? ''}</span>
                        {/if}
                      </div>
                      <div class="split-side {row.new ? `line-${row.new.line.kind}` : 'line-empty'}">
                        <span class="ln">{row.new?.line.new_no ?? ''}</span>
                        <span class="prefix">{row.new?.line.kind === 'add' ? '+' : ' '}</span>
                        {#if row.new && fileHl?.[hunkIdx]?.[row.new.idx] != null}
                          <span class="text">{@html fileHl[hunkIdx][row.new.idx]}</span>
                        {:else}
                          <span class="text">{row.new?.line.text ?? ''}</span>
                        {/if}
                      </div>
                    </div>
                  {/each}
                </div>
                {#if hiddenRows > 0}
                  <button class="hunk-expand" type="button" onclick={() => expandedHunks.add(hkey)}>
                    Show {hiddenRows.toLocaleString()} more row{hiddenRows === 1 ? '' : 's'}
                  </button>
                {/if}
              {:else}
                <div class="hunk-placeholder" style:height="{estimateHunkHeight(hunk)}px"></div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </article>
  {/each}
{/if}

<style>
  .empty {
    padding: var(--sp-6);
    color: var(--fg-subtle);
    text-align: center;
    font-size: var(--fs-sm);
  }
  .file {
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    background: var(--bg-elev-1);
    /* Side + bottom margin gives the file breathing room from the
       page edges and following content. The first file's top margin
       is added below so its sticky header can land flush with the
       page's scroll-container top when stuck. */
    margin: 0 var(--sp-3) var(--sp-3);
    /* No overflow: hidden — sticky headers need their scroll ancestor
       to be the page-level diff column, not this article. */
  }
  .file.collapsed {
    /* Round all corners when there's no body to anchor the bottom. */
    border-radius: var(--r-md);
  }
  /* Sticky to the top of the scrolling diff column. The article doesn't
     create a scroll context, so the header sticks against the nearest
     ancestor with overflow: auto (the page-level diff container). */
  .file-header {
    position: sticky;
    top: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border);
    border-top-left-radius: var(--r-md);
    border-top-right-radius: var(--r-md);
    background: var(--bg-elev-2);
    color: var(--fg);
    min-width: 0;
    cursor: pointer;
    user-select: none;
  }
  .file.collapsed .file-header {
    /* Round the bottom corners too when the body is hidden. */
    border-bottom: 1px solid var(--border);
    border-bottom-left-radius: var(--r-md);
    border-bottom-right-radius: var(--r-md);
  }
  .file-header:hover { background: var(--bg-elev-3); }
  .file-header:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: -2px;
  }
  .chevron {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-subtle);
    flex-shrink: 0;
    transition: color var(--t-fast);
  }
  .file-header:hover .chevron { color: var(--fg); }
  .name {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
    overflow: hidden;
  }
  .basename {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    font-weight: var(--weight-semibold);
    color: var(--fg);
    flex-shrink: 0;
    max-width: 60ch;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dir {
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    color: var(--fg-subtle);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .old {
    font-family: var(--font-mono);
    font-size: var(--fs-2xs);
    color: var(--fg-subtle);
  }

  .rename-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px var(--sp-3);
    background: var(--rename-bg);
    color: var(--rename);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    border-bottom-left-radius: var(--r-md);
    border-bottom-right-radius: var(--r-md);
  }
  .rename-paths {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .rename-path {
    color: var(--fg-muted);
  }
  .rename-arrow {
    color: var(--rename);
    font-weight: var(--weight-semibold);
  }
  .rename-highlight {
    padding: 1px 4px;
    border-radius: var(--r-sm);
    font-weight: var(--weight-semibold);
  }
  .rename-del {
    background: color-mix(in srgb, var(--removed) 20%, transparent);
    color: var(--removed);
  }
  .rename-add {
    background: color-mix(in srgb, var(--added) 20%, transparent);
    color: var(--added);
  }

  /* Status pill: subtle tinted chip per kind, using each tone's bg + fg. */
  .status {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    height: 20px;
    padding: 0 8px;
    border-radius: var(--r-pill);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-wider);
    text-transform: uppercase;
    border: 1px solid transparent;
  }
  .status-add {
    background: color-mix(in srgb, var(--added) 14%, transparent);
    color: var(--added);
    border-color: color-mix(in srgb, var(--added) 28%, transparent);
  }
  .status-del {
    background: color-mix(in srgb, var(--removed) 14%, transparent);
    color: var(--removed);
    border-color: color-mix(in srgb, var(--removed) 28%, transparent);
  }
  .status-mod {
    background: var(--warn-bg);
    color: var(--warn);
    border-color: var(--warn-border);
  }
  .status-ren {
    background: var(--rename-bg);
    color: var(--rename);
    border-color: var(--rename-border);
  }

  /* Two adjacent badges showing line counts. */
  .counts {
    display: inline-flex;
    align-items: stretch;
    height: 20px;
    border-radius: var(--r-sm);
    overflow: hidden;
    border: 1px solid var(--border);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
  }
  .count-add,
  .count-del {
    display: inline-flex;
    align-items: center;
    padding: 0 7px;
  }
  .count-add {
    background: color-mix(in srgb, var(--added) 14%, transparent);
    color: var(--added);
  }
  .count-del {
    background: color-mix(in srgb, var(--removed) 14%, transparent);
    color: var(--removed);
  }
  .count-add + .count-del {
    border-left: 1px solid var(--border);
  }

  .open-link {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 22px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    transition: color var(--t-fast), border-color var(--t-fast), background var(--t-fast);
  }
  .open-link:hover {
    color: var(--accent-fg);
    border-color: var(--accent-bg-strong);
    background: var(--accent-bg-soft);
  }

  .binary { padding: var(--sp-3); color: var(--fg-subtle); font-size: var(--fs-sm); }

  /* Reserves vertical space for files outside the viewport so scroll
     position stays stable until IntersectionObserver swaps in real hunks. */
  .file-placeholder {
    width: 100%;
  }

  /* Same job as .file-placeholder but per hunk — sized to the hunk's
     line count so scroll bar position doesn't jump when a hunk swaps
     between placeholder and real lines. */
  .hunk-placeholder {
    width: 100%;
  }

  /* Inline "Show N more lines" button at the bottom of a capped hunk.
     Same width as a line so it reads as part of the diff stream. */
  .hunk-expand {
    display: block;
    width: 100%;
    padding: var(--sp-1) var(--sp-3);
    background: var(--bg-elev-2);
    border: none;
    border-top: 1px dashed var(--border);
    color: var(--accent-fg);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    text-align: center;
    cursor: pointer;
    transition: background var(--t-fast);
  }
  .hunk-expand:hover { background: var(--bg-elev-3); }

  /* No horizontal scroll — long lines wrap. Sidesteps the rabbit hole
     of getting line backgrounds to fill consistently across hunks of
     differing widths. */
  .body {
    overflow: hidden;
  }

  .hunk { border-top: 1px solid var(--border); }
  .hunk-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: var(--sp-1) var(--sp-3);
    background: var(--hunk-bg);
    color: var(--hunk-fg);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    /* Sharp top-left corner — industrial code-block feel. */
    border-top-left-radius: 0;
  }
  .hunk-header-text { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .hunk-discard {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 18px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--r-sm);
    color: var(--fg-subtle);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--t-fast), color var(--t-fast), background var(--t-fast), border-color var(--t-fast);
  }
  .hunk:hover .hunk-discard { opacity: 1; }
  .hunk-discard:hover {
    color: var(--removed);
    background: color-mix(in srgb, var(--removed) 14%, transparent);
    border-color: color-mix(in srgb, var(--removed) 28%, transparent);
  }
  .lines { font-family: var(--font-mono); font-size: var(--fs-xs); }
  .line {
    display: grid;
    grid-template-columns: 40px 40px 16px 1fr;
    /* Top-align so wrapped continuation rows don't drop the line numbers
       and prefix into vertical centering with the wrapped content. */
    align-items: start;
    line-height: 18px;
    padding: 2px var(--sp-2);
    /* pre-wrap preserves leading whitespace (indentation) but allows
       wrapping at line breaks. overflow-wrap: anywhere also breaks
       inside long unbreakable tokens (URLs, base64, etc.). */
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .line-add { background: var(--added-bg); }
  .line-del { background: var(--removed-bg); }
  /* Intra-line word diff. Wrapper spans carry only a background tint
     so they stack cleanly over Shiki's syntax colours. Inner wrappers
     (a logical range may be split across syntax-span boundaries) stay
     flat — only the first and last wrapper round the leading/trailing
     edges, so the whole logical range reads as one rounded pill. */
  .text :global(.intra-add) {
    background: color-mix(in srgb, var(--added) 28%, transparent);
  }
  .text :global(.intra-del) {
    background: color-mix(in srgb, var(--removed) 28%, transparent);
  }
  .text :global(.intra-start) { border-top-left-radius: 3px; border-bottom-left-radius: 3px; }
  .text :global(.intra-end)   { border-top-right-radius: 3px; border-bottom-right-radius: 3px; }
  /* Line numbers and the +/− prefix are presentational — exclude them
     from text selection so copy gives clean code without gutter chrome. */
  .line .ln {
    color: var(--fg-subtle);
    text-align: right;
    padding-right: var(--sp-1);
    font-variant-numeric: tabular-nums;
    user-select: none;
    -webkit-user-select: none;
  }
  .line .prefix {
    color: var(--fg-subtle);
    text-align: center;
    user-select: none;
    -webkit-user-select: none;
  }
  .line-add .prefix { color: var(--added); }
  .line-del .prefix { color: var(--removed); }
  .line .text { color: var(--fg); user-select: text; -webkit-user-select: text; }

  /* View-mode toggle */
  .controls {
    display: flex;
    justify-content: flex-end;
    /* DiffView mounts directly inside the page's scrollable diff
       column with no padding — give the controls their own page-edge
       margin (matches .file's margin). */
    margin: var(--sp-3) var(--sp-3) var(--sp-2);
  }
  .seg {
    display: inline-flex;
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    overflow: hidden;
    background: var(--bg-elev-1);
  }
  .seg button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 9px;
    background: transparent;
    border: none;
    color: var(--fg-muted);
    font-size: var(--fs-2xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-tight);
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .seg button + button { border-left: 1px solid var(--border); }
  .seg button:hover { color: var(--fg); }
  .seg button.on {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
  }
  .seg button :global(svg) { color: inherit; }

  /* Split view — same wrapping approach as the unified body. */
  .split-body {
    overflow: hidden;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
  }
  .split-hunk { border-top: 1px solid var(--border); }
  .split-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    column-gap: 0;
    border-bottom: 0;
    /* Stretch each side to the full row height so wrapped lines on one
       side don't leave a gap on the other side. */
    align-items: stretch;
  }
  .split-side {
    display: grid;
    grid-template-columns: 40px 16px 1fr;
    align-items: start;
    line-height: 18px;
    padding: 2px var(--sp-2);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    border-right: 1px solid var(--border);
    min-width: 0;
  }
  .split-side:last-child { border-right: none; }
  .split-side.line-add { background: var(--added-bg); }
  .split-side.line-del { background: var(--removed-bg); }
  .split-side.line-empty {
    background: color-mix(in srgb, var(--fg-faint) 6%, transparent);
  }
  .split-side .ln {
    color: var(--fg-subtle);
    text-align: right;
    padding-right: var(--sp-1);
    font-variant-numeric: tabular-nums;
    user-select: none;
    -webkit-user-select: none;
  }
  .split-side .prefix {
    color: var(--fg-subtle);
    text-align: center;
    user-select: none;
    -webkit-user-select: none;
  }
  .split-side.line-add .prefix { color: var(--added); }
  .split-side.line-del .prefix { color: var(--removed); }
  .split-side .text { color: var(--fg); user-select: text; -webkit-user-select: text; }
</style>
