<script lang="ts">
  import type { DiffPayload, DiffFile, FileStatus } from '$lib/types';
  import { detectLang, highlightLines } from '$lib/syntax/highlighter';
  import { theme } from '$lib/stores/theme.svelte';
  import FileIcon from '$lib/components/file/FileIcon.svelte';
  import Icon from '$lib/components/primitives/Icon.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';

  let {
    payload,
    fileHref,
  }: {
    payload: DiffPayload | null;
    /** Optional resolver returning a remote URL for the file, or null when
     *  the file shouldn't be linkable (e.g. newly added). */
    fileHref?: (file: DiffFile) => string | null;
  } = $props();

  // hl[file.path][hunkIdx][lineIdx] = highlighted HTML for that line.
  let hl = $state<Record<string, string[][]>>({});

  $effect(() => {
    const t = theme.value;
    const p = payload;
    if (!p) { hl = {}; return; }
    let cancelled = false;

    (async () => {
      const next: Record<string, string[][]> = {};
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
        next[file.path] = hunkHtml;
        // Publish progressively so files highlight as they finish.
        hl = { ...hl, [file.path]: hunkHtml };
      }
      if (cancelled) return;
      hl = next;
    })();

    return () => { cancelled = true; };
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
  {#each payload.files as file}
    {@const counts = countLines(file)}
    {@const lbl = statusLabel(file.status)}
    {@const dir = dirname(file.path)}
    {@const href = fileHref ? fileHref(file) : null}
    <article class="file">
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
      {:else}
        <div class="body">
          {#each file.hunks as hunk, hunkIdx}
            <div class="hunk">
              <div class="hunk-header">{hunk.header}</div>
              <div class="lines">
                {#each hunk.lines as line, lineIdx}
                  <div class="line line-{line.kind}">
                    <span class="ln ln-old">{line.old_no ?? ''}</span>
                    <span class="ln ln-new">{line.new_no ?? ''}</span>
                    <span class="prefix">{line.kind === 'add' ? '+' : line.kind === 'del' ? '−' : ' '}</span>
                    {#if hl[file.path]?.[hunkIdx]?.[lineIdx] != null}
                      <span class="text">{@html hl[file.path][hunkIdx][lineIdx]}</span>
                    {:else}
                      <span class="text">{line.text}</span>
                    {/if}
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
    background: color-mix(in srgb, #f59e0b 14%, transparent);
    color: #f59e0b;
    border-color: color-mix(in srgb, #f59e0b 30%, transparent);
  }
  .status-ren {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    border-color: var(--accent-bg-strong);
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

  /* Per-file horizontal scroll: long lines reveal a single scrollbar
     under all hunks of this file. The header stays fixed (not in body). */
  .body {
    overflow-x: auto;
    overflow-y: hidden;
  }

  .hunk { border-top: 1px solid var(--border); min-width: max-content; }
  .hunk-header {
    padding: var(--sp-1) var(--sp-3);
    background: var(--hunk-bg);
    color: var(--accent-fg);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
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
</style>
