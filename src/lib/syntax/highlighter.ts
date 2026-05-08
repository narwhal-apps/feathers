// Lazy, singleton Shiki highlighter.
// We bundle two themes (one per app theme) and load grammars on demand
// the first time a file of that language is rendered.
//
// The highlighter and language grammars live in module scope so the cost
// is paid once per session. Languages are deduped via _loading promises.

import type { BundledLanguage, HighlighterGeneric } from 'shiki';
import type { ThemeName } from '$lib/types';

type Hl = HighlighterGeneric<BundledLanguage, string>;

const THEME_DARK = 'github-dark-default';
const THEME_LIGHT = 'github-light-default';

let _hlPromise: Promise<Hl> | null = null;
const _loaded = new Set<string>();
const _loading = new Map<string, Promise<void>>();

async function getHighlighter(): Promise<Hl> {
  if (!_hlPromise) {
    _hlPromise = (async () => {
      const { createHighlighter } = await import('shiki');
      return (await createHighlighter({
        themes: [THEME_DARK, THEME_LIGHT],
        langs: [],
      })) as Hl;
    })();
  }
  return _hlPromise;
}

async function ensureLang(lang: BundledLanguage): Promise<void> {
  if (_loaded.has(lang)) return;
  let p = _loading.get(lang);
  if (!p) {
    p = (async () => {
      const hl = await getHighlighter();
      try {
        await hl.loadLanguage(lang);
        _loaded.add(lang);
      } finally {
        _loading.delete(lang);
      }
    })();
    _loading.set(lang, p);
  }
  await p;
}

// Map common file extensions / basenames to Shiki bundled language ids.
const EXT_TO_LANG: Record<string, BundledLanguage> = {
  // JS family
  js: 'javascript', mjs: 'javascript', cjs: 'javascript',
  jsx: 'jsx',
  ts: 'typescript', mts: 'typescript', cts: 'typescript',
  tsx: 'tsx',
  svelte: 'svelte', vue: 'vue',
  // Web
  html: 'html', htm: 'html', xml: 'xml',
  css: 'css', scss: 'scss', sass: 'sass', less: 'less',
  // Data
  json: 'json', jsonc: 'jsonc',
  yaml: 'yaml', yml: 'yaml',
  toml: 'toml',
  md: 'markdown', markdown: 'markdown', mdx: 'mdx',
  // Systems
  rs: 'rust',
  c: 'c', h: 'c',
  cpp: 'cpp', cc: 'cpp', cxx: 'cpp', hpp: 'cpp', hh: 'cpp', hxx: 'cpp',
  cs: 'csharp',
  go: 'go',
  swift: 'swift',
  kt: 'kotlin', kts: 'kotlin',
  // Scripting
  py: 'python', pyi: 'python',
  rb: 'ruby',
  php: 'php',
  lua: 'lua',
  sh: 'shell', bash: 'shell', zsh: 'shell',
  // Other
  sql: 'sql',
  java: 'java',
  scala: 'scala',
  dart: 'dart',
  ex: 'elixir', exs: 'elixir',
  hs: 'haskell',
  dockerfile: 'docker',
};

const BASENAME_TO_LANG: Record<string, BundledLanguage> = {
  'dockerfile': 'docker',
  'makefile': 'make',
  'cmakelists.txt': 'cmake',
};

export function detectLang(path: string): BundledLanguage | null {
  const file = path.split('/').pop()?.toLowerCase() ?? '';
  if (BASENAME_TO_LANG[file]) return BASENAME_TO_LANG[file];
  const dot = file.lastIndexOf('.');
  if (dot < 0) return null;
  const ext = file.slice(dot + 1);
  return EXT_TO_LANG[ext] ?? null;
}

const HTML_ESCAPE: Record<string, string> = {
  '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;', "'": '&#39;',
};
function esc(s: string): string {
  return s.replace(/[<>&"']/g, (c) => HTML_ESCAPE[c]);
}

/**
 * Highlight an array of source lines as a single contextual block (so
 * multi-line strings/comments span correctly within the block) and return
 * one HTML string per input line.
 *
 * If the language hasn't loaded yet or shiki fails for any reason, returns
 * escaped plain-text HTML — the diff still renders, just without colors.
 */
export async function highlightLines(
  lines: string[],
  lang: BundledLanguage,
  themeName: ThemeName,
): Promise<string[]> {
  try {
    await ensureLang(lang);
    const hl = await getHighlighter();
    const code = lines.join('\n');
    const tokenized = hl.codeToTokensBase(code, {
      lang,
      theme: themeName === 'dark' ? THEME_DARK : THEME_LIGHT,
    });
    return tokenized.map((tokens) =>
      tokens
        .map((t) => {
          const style = t.color ? `color:${t.color}` : '';
          return `<span style="${style}">${esc(t.content)}</span>`;
        })
        .join(''),
    );
  } catch {
    return lines.map(esc);
  }
}
