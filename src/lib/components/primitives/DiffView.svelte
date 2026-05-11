<script lang="ts">
  import type { DiffPayload, DiffFile, DiffHunk, DiffLine, FileStatus } from '$lib/types';
  import { browser } from '$app/environment';
  import { detectLang, highlightLines } from '$lib/syntax/highlighter';
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

  // Skip the highlighter entirely when payload+theme reference is unchanged.
  let hlPayloadRef: DiffPayload | null = null;
  let hlTheme: string | null = null;

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
    hlPayloadRef = p;
    hlTheme = t;
    hl.clear();
    let cancelled = false;

    (async () => {
      for (const file of p.files) {
        if (file.binary) continue;
        const lang = detectLang(file.path);
        if (!lang) continue;
        const hunkHtml: string[][] = [];
        for (const hunk of file.hunks) {
          const lines = hunk.lines.map((l) => l.text);
          const html = await highlightLines(lines, lang, t);
          hunkHtml.push(html);
        }
        if (cancelled) return;
        // Publish progressively so files highlight as they finish.
        hl.set(file.path, hunkHtml);
      }
    })();

    return () => { cancelled = true; };
  });

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
  const visibleFiles = new SvelteSet<string>();

  function visible(node: HTMLElement, opts: { path: string }) {
    const io = new IntersectionObserver((entries) => {
      for (const e of entries) {
        if (e.isIntersecting) {
          visibleFiles.add(opts.path);
          io.unobserve(node);
        }
      }
    }, { rootMargin: '500px' });
    io.observe(node);
    return { destroy: () => io.disconnect() };
  }

  const isVisible = (path: string) => visibleFiles.has(path);

  function estimateHeight(file: DiffFile): number {
    let lines = 0;
    for (const h of file.hunks) lines += h.lines.length;
    return Math.max(60, lines * 22 + file.hunks.length * 36);
  }

  // When the payload reference changes, reset visibility so newly mounted
  // files start hidden; the IO callback will fire immediately for any that
  // happen to be in the viewport.
  $effect(() => {
    const _ = payload;
    visibleFiles.clear();
  });

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
    <article class="file" use:visible={{ path: file.path }}>
      <header class="file-header">
        <FileIcon fileName={basename(file.path)} size={14} />
        <span class="name">
          <span class="basename">{basename(file.path)}</span>
          {#if dir}<span class="dir">{dir}</span>{/if}
        </span>
        {#if file.old_path && file.old_path !== file.path}
          <span class="old" title="Renamed from {file.old_path}">← {file.old_path}</span>
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
            onclick={() => openUrl(href)}
          >
            <Icon name="ExternalLink" size={12} />
          </button>
        {/if}
      </header>
      {#if file.binary}
        <div class="binary">Binary file — diff not shown.</div>
      {:else if !isVisible(file.path)}
        <div class="file-placeholder" style:height="{estimateHeight(file)}px"></div>
      {:else if mode === 'unified'}
        {@const fileHl = hl.get(file.path)}
        <div class="body">
          {#each file.hunks as hunk, hunkIdx (hunk.header)}
            <div class="hunk">
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
              <div class="lines">
                {#each hunk.lines as line, lineIdx ((line.old_no ?? 'a') + ':' + (line.new_no ?? 'a') + ':' + line.kind + ':' + lineIdx)}
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
            </div>
          {/each}
        </div>
      {:else}
        <!-- Split (side-by-side) view -->
        {@const fileHl = hl.get(file.path)}
        <div class="body split-body">
          {#each file.hunks as hunk, hunkIdx (hunk.header)}
            {@const rows = splitRows.get(hunk) ?? []}
            <div class="hunk split-hunk">
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
              <div class="split-rows">
                {#each rows as row, rowIdx ((row.old?.idx ?? 'a') + ':' + (row.new?.idx ?? 'a') + ':' + rowIdx)}
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
    margin-bottom: var(--sp-3);
    overflow: hidden;
  }
  .file-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border);
    background: var(--bg-elev-2);
    color: var(--fg);
    min-width: 0;
  }
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

  /* Per-file horizontal scroll: long lines reveal a single scrollbar
     under all hunks of this file. The header stays fixed (not in body). */
  .body {
    overflow-x: auto;
    overflow-y: hidden;
  }

  .hunk { border-top: 1px solid var(--border); min-width: max-content; }
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
  .lines { font-family: var(--font-mono); font-size: var(--fs-xs); min-width: max-content; }
  .line {
    display: grid;
    grid-template-columns: 40px 40px 16px 1fr;
    align-items: center;
    line-height: 18px;
    padding: 0 var(--sp-2);
    white-space: pre;
    min-width: max-content;
  }
  .line-add { background: var(--added-bg); }
  .line-del { background: var(--removed-bg); }
  .line .ln { color: var(--fg-subtle); text-align: right; padding-right: var(--sp-1); font-variant-numeric: tabular-nums; }
  .line .prefix { color: var(--fg-subtle); text-align: center; }
  .line-add .prefix { color: var(--added); }
  .line-del .prefix { color: var(--removed); }
  .line .text { color: var(--fg); }

  /* View-mode toggle */
  .controls {
    display: flex;
    justify-content: flex-end;
    margin-bottom: var(--sp-2);
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

  /* Split view */
  .split-body {
    overflow-x: auto;
    overflow-y: hidden;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
  }
  .split-hunk { border-top: 1px solid var(--border); min-width: max-content; }
  .split-rows { min-width: max-content; }
  .split-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    column-gap: 0;
    border-bottom: 0;
  }
  .split-side {
    display: grid;
    grid-template-columns: 40px 16px 1fr;
    align-items: center;
    line-height: 18px;
    padding: 0 var(--sp-2);
    white-space: pre;
    min-width: max-content;
    border-right: 1px solid var(--border);
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
  }
  .split-side .prefix { color: var(--fg-subtle); text-align: center; }
  .split-side.line-add .prefix { color: var(--added); }
  .split-side.line-del .prefix { color: var(--removed); }
  .split-side .text { color: var(--fg); }
</style>
