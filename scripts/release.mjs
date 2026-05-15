#!/usr/bin/env node
/**
 * Release helper.
 *
 *   pnpm release
 *
 * Walks the operator through:
 *   1. Verifying the working tree is clean and on main
 *   2. Picking a bump (major / minor / patch)
 *   3. Previewing the new CHANGELOG entry built from commits since the last tag
 *   4. Bumping the version in package.json + tauri.conf.json + Cargo.toml
 *   5. Committing, tagging, and pushing both — which kicks off release.yml.
 *
 * Pure Node, no extra deps. Run with --dry-run to skip every mutating step.
 */
import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createInterface } from 'node:readline/promises';
import { stdin as input, stdout as output } from 'node:process';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const PKG = resolve(ROOT, 'package.json');
const TAURI_CONF = resolve(ROOT, 'src-tauri/tauri.conf.json');
const CARGO = resolve(ROOT, 'src-tauri/Cargo.toml');
const CHANGELOG = resolve(ROOT, 'CHANGELOG.md');

const DRY = process.argv.includes('--dry-run');

const C = {
  reset: '\x1b[0m',
  dim: '\x1b[2m',
  bold: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
};
const log = {
  step: (m) => console.log(`${C.cyan}▸${C.reset} ${m}`),
  ok: (m) => console.log(`${C.green}✓${C.reset} ${m}`),
  warn: (m) => console.log(`${C.yellow}!${C.reset} ${m}`),
  err: (m) => console.error(`${C.red}✗${C.reset} ${m}`),
  dim: (m) => console.log(`${C.dim}${m}${C.reset}`),
};

function sh(cmd, opts = {}) {
  return execSync(cmd, { cwd: ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], ...opts }).trim();
}
function shRun(cmd) {
  if (DRY) { log.dim(`[dry-run] ${cmd}`); return; }
  execSync(cmd, { cwd: ROOT, stdio: 'inherit' });
}
function fail(msg) { log.err(msg); process.exit(1); }

// ─── Preflight ──────────────────────────────────────────────────────────

function preflight() {
  log.step('Preflight');
  // Working tree must be clean (else commit semantics get confusing).
  const dirty = sh('git status --porcelain');
  if (dirty) fail(`Working tree is dirty:\n${dirty}\n\nCommit or stash first.`);

  // Must be on main.
  const branch = sh('git rev-parse --abbrev-ref HEAD');
  if (branch !== 'main') fail(`Releases cut from main only — currently on '${branch}'.`);

  // Must be up to date with origin/main so the tag points at the same commit
  // GitHub Actions will see.
  try { sh('git fetch --quiet origin main'); } catch { fail('git fetch origin main failed.'); }
  const local = sh('git rev-parse HEAD');
  const remote = sh('git rev-parse origin/main');
  if (local !== remote) fail("Local main doesn't match origin/main. Pull or push first.");

  log.ok(`On main, clean, in sync with origin (${local.slice(0, 7)})`);
}

// ─── Version handling ───────────────────────────────────────────────────

function readVersions() {
  const pkg = JSON.parse(readFileSync(PKG, 'utf8'));
  const tauri = JSON.parse(readFileSync(TAURI_CONF, 'utf8'));
  const cargoSrc = readFileSync(CARGO, 'utf8');
  const cargoMatch = cargoSrc.match(/^version\s*=\s*"([^"]+)"/m);
  if (!cargoMatch) fail('Could not find version in src-tauri/Cargo.toml');
  return { pkg: pkg.version, tauri: tauri.version, cargo: cargoMatch[1] };
}

function assertVersionsAligned(v) {
  if (v.pkg !== v.tauri || v.pkg !== v.cargo) {
    fail(
      `Version mismatch across files — fix manually before releasing:\n` +
      `  package.json:      ${v.pkg}\n` +
      `  tauri.conf.json:   ${v.tauri}\n` +
      `  src-tauri/Cargo:   ${v.cargo}`,
    );
  }
}

function bump(version, kind) {
  const m = version.match(/^(\d+)\.(\d+)\.(\d+)$/);
  if (!m) fail(`Current version '${version}' is not semver MAJOR.MINOR.PATCH.`);
  let [_, maj, min, pat] = m.map((x, i) => (i === 0 ? x : Number(x)));
  if (kind === 'major') { maj += 1; min = 0; pat = 0; }
  else if (kind === 'minor') { min += 1; pat = 0; }
  else if (kind === 'patch') { pat += 1; }
  else fail(`Unknown bump '${kind}'`);
  return `${maj}.${min}.${pat}`;
}

function writeVersions(next) {
  // package.json — re-stringify with stable 2-space indent + trailing newline
  // so the diff is minimal.
  const pkg = JSON.parse(readFileSync(PKG, 'utf8'));
  pkg.version = next;
  if (!DRY) writeFileSync(PKG, JSON.stringify(pkg, null, 2) + '\n');

  const tauri = JSON.parse(readFileSync(TAURI_CONF, 'utf8'));
  tauri.version = next;
  if (!DRY) writeFileSync(TAURI_CONF, JSON.stringify(tauri, null, 2) + '\n');

  // Cargo.toml — surgical replacement of the first `version = "..."` line.
  const cargoSrc = readFileSync(CARGO, 'utf8');
  const updated = cargoSrc.replace(/^version\s*=\s*"[^"]+"/m, `version = "${next}"`);
  if (updated === cargoSrc) fail('Failed to rewrite version in Cargo.toml');
  if (!DRY) writeFileSync(CARGO, updated);
}

// ─── Commit collection ──────────────────────────────────────────────────

const TYPE_SECTIONS = [
  ['feat',     'Features'],
  ['fix',      'Fixes'],
  ['perf',     'Performance'],
  ['refactor', 'Refactors'],
  ['docs',     'Documentation'],
  ['test',     'Tests'],
  ['build',    'Build'],
  ['ci',       'CI'],
  ['style',    'Style'],
  ['chore',    'Chore'],
];
const SECTION_ORDER = TYPE_SECTIONS.map(([, s]) => s).concat('Other');
const TYPE_TO_SECTION = Object.fromEntries(TYPE_SECTIONS);

function lastTag() {
  try {
    return sh('git describe --tags --abbrev=0 --match "v*"');
  } catch {
    return null;
  }
}

function collectCommits(since) {
  // %h<TAB>%s — short SHA + subject. Range: since..HEAD, or all if no tag yet.
  const range = since ? `${since}..HEAD` : 'HEAD';
  const out = sh(`git log ${range} --no-merges --pretty=format:%h%x09%s`);
  if (!out) return [];
  return out.split('\n').map((line) => {
    const tab = line.indexOf('\t');
    return { sha: line.slice(0, tab), subject: line.slice(tab + 1) };
  });
}

function classify(commit) {
  const m = commit.subject.match(/^(\w+)(?:\([^)]*\))?!?:\s*(.+)$/);
  if (!m) return { section: 'Other', message: commit.subject };
  const type = m[1].toLowerCase();
  // Skip the release commit itself — it's noise in its own changelog entry.
  if (type === 'release') return null;
  return { section: TYPE_TO_SECTION[type] ?? 'Other', message: m[2] };
}

function groupCommits(commits) {
  const groups = Object.fromEntries(SECTION_ORDER.map((s) => [s, []]));
  for (const c of commits) {
    const r = classify(c);
    if (!r) continue;
    groups[r.section].push({ sha: c.sha, message: r.message });
  }
  return groups;
}

function renderEntry(version, groups) {
  const today = new Date().toISOString().slice(0, 10);
  const lines = [`## [${version}] — ${today}`, ''];
  for (const section of SECTION_ORDER) {
    const items = groups[section];
    if (!items.length) continue;
    lines.push(`### ${section}`);
    for (const { sha, message } of items) lines.push(`- ${message} (${sha})`);
    lines.push('');
  }
  return lines.join('\n').trimEnd() + '\n';
}

function prependChangelog(entry) {
  const HEADER = `# Changelog\n\nAll notable changes to this project. Generated by \`pnpm release\`.\n\n`;
  let body;
  if (existsSync(CHANGELOG)) {
    const current = readFileSync(CHANGELOG, 'utf8');
    // Strip the existing header so we can re-prepend our entry between
    // the header and the prior entries.
    const headerMatch = current.match(/^# Changelog[\s\S]*?\n(?=## |$)/);
    if (headerMatch) {
      body = HEADER + entry + '\n' + current.slice(headerMatch[0].length);
    } else {
      body = HEADER + entry + '\n' + current;
    }
  } else {
    body = HEADER + entry;
  }
  if (!DRY) writeFileSync(CHANGELOG, body);
}

// ─── Prompts ────────────────────────────────────────────────────────────

async function ask(rl, prompt) {
  const answer = await rl.question(prompt);
  return answer.trim();
}

async function pickBump(rl, current) {
  const choices = ['patch', 'minor', 'major'];
  console.log(`\nCurrent version: ${C.bold}${current}${C.reset}`);
  console.log(`  1) patch  → ${bump(current, 'patch')}`);
  console.log(`  2) minor  → ${bump(current, 'minor')}`);
  console.log(`  3) major  → ${bump(current, 'major')}`);
  const a = (await ask(rl, '\nBump type [1-3 or patch/minor/major]: ')).toLowerCase();
  if (['1', 'patch', 'p'].includes(a)) return 'patch';
  if (['2', 'minor', 'm'].includes(a)) return 'minor';
  if (['3', 'major', 'M'].includes(a)) return 'major';
  if (choices.includes(a)) return a;
  fail(`Unrecognised bump '${a}'.`);
}

async function confirm(rl, prompt) {
  const a = (await ask(rl, `${prompt} [y/N] `)).toLowerCase();
  return a === 'y' || a === 'yes';
}

// ─── Main ───────────────────────────────────────────────────────────────

async function main() {
  if (DRY) log.warn('Running with --dry-run — no files written, no git changes.');
  preflight();

  const versions = readVersions();
  assertVersionsAligned(versions);
  const current = versions.pkg;

  const tag = lastTag();
  if (tag) log.ok(`Last tag: ${tag}`);
  else log.warn('No prior tag — collecting all commits.');

  const commits = collectCommits(tag);
  if (!commits.length) fail('No commits since last tag — nothing to release.');
  log.ok(`${commits.length} commit${commits.length === 1 ? '' : 's'} since ${tag ?? 'repo start'}`);

  const rl = createInterface({ input, output });
  try {
    const kind = await pickBump(rl, current);
    const next = bump(current, kind);
    const groups = groupCommits(commits);
    const entry = renderEntry(next, groups);

    console.log(`\n${C.bold}── CHANGELOG preview ──${C.reset}\n`);
    process.stdout.write(entry);
    console.log(`${C.bold}─────────────────────${C.reset}\n`);

    if (!(await confirm(rl, `Release v${next} — write files, commit, tag, push?`))) {
      log.warn('Aborted.');
      process.exit(0);
    }

    log.step('Writing CHANGELOG and version files');
    prependChangelog(entry);
    writeVersions(next);

    log.step('Staging + committing');
    shRun(`git add CHANGELOG.md package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml`);
    shRun(`git commit -m "release: v${next}"`);

    log.step('Tagging');
    shRun(`git tag v${next}`);

    log.step('Pushing main + tag');
    shRun(`git push origin main`);
    shRun(`git push origin v${next}`);

    log.ok(`Released v${next}. release.yml will draft the GitHub release shortly.`);
    log.dim('  Watch: https://github.com/narwhal-apps/feathers/actions');
  } finally {
    rl.close();
  }
}

main().catch((e) => { log.err(e.message ?? String(e)); process.exit(1); });
