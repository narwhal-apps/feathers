<script lang="ts">
  import type { DiffPayload } from '$lib/types';

  let { payload }: { payload: DiffPayload | null } = $props();
</script>

{#if !payload || payload.files.length === 0}
  <div class="empty">No changes.</div>
{:else}
  {#each payload.files as file}
    <article class="file">
      <header class="file-header">
        <span class="path">{file.path}</span>
        {#if file.old_path && file.old_path !== file.path}
          <span class="old">← {file.old_path}</span>
        {/if}
        <span class="status status-{file.status}">{file.status}</span>
      </header>
      {#if file.binary}
        <div class="binary">Binary file — diff not shown.</div>
      {:else}
        {#each file.hunks as hunk}
          <div class="hunk">
            <div class="hunk-header">{hunk.header}</div>
            <div class="lines">
              {#each hunk.lines as line}
                <div class="line line-{line.kind}">
                  <span class="ln ln-old">{line.old_no ?? ''}</span>
                  <span class="ln ln-new">{line.new_no ?? ''}</span>
                  <span class="prefix">{line.kind === 'add' ? '+' : line.kind === 'del' ? '−' : ' '}</span>
                  <span class="text">{line.text}</span>
                </div>
              {/each}
            </div>
          </div>
        {/each}
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
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--fg);
    background: var(--bg-elev-2);
  }
  .path { font-weight: 600; }
  .old { color: var(--fg-subtle); }
  .status { margin-left: auto; color: var(--fg-muted); font-style: italic; }

  .binary { padding: var(--sp-3); color: var(--fg-subtle); font-size: var(--fs-sm); }

  .hunk { border-top: 1px solid var(--border); }
  .hunk-header {
    padding: var(--sp-1) var(--sp-3);
    background: rgba(20, 184, 166, 0.06);
    color: var(--accent-300);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
  }
  .lines { font-family: var(--font-mono); font-size: var(--fs-xs); }
  .line {
    display: grid;
    grid-template-columns: 40px 40px 16px 1fr;
    align-items: center;
    line-height: 18px;
    padding: 0 var(--sp-2);
    white-space: pre;
  }
  .line-add { background: var(--added-bg); }
  .line-del { background: var(--removed-bg); }
  .line .ln { color: var(--fg-subtle); text-align: right; padding-right: var(--sp-1); font-variant-numeric: tabular-nums; }
  .line .prefix { color: var(--fg-subtle); text-align: center; }
  .line-add .prefix { color: var(--added); }
  .line-del .prefix { color: var(--removed); }
  .line .text { color: var(--fg); }
</style>
