// Mirror of src-tauri/src/git_core/types.rs and repo_registry.rs.
// Keep in sync by hand (Tauri 2 doesn't auto-generate types yet).

export type RepoId = string;

export interface RepoSummary {
  id: RepoId;
  name: string;
  path: string;
}

export interface HeadInfo {
  branch: string;
  detached: boolean;
  short_sha: string;
}

export interface RepoOpenResult {
  id: RepoId;
  summary: RepoSummary;
  head: HeadInfo;
}

export type FileStatus =
  | 'added'
  | 'modified'
  | 'deleted'
  | 'renamed'
  | 'typechange'
  | 'untracked'
  | 'conflicted';

export interface FileChange {
  path: string;
  old_path: string | null;
  status: FileStatus;
}

export interface StatusSnapshot {
  staged: FileChange[];
  unstaged: FileChange[];
  untracked: FileChange[];
  conflicted: FileChange[];
}

export interface BranchInfo {
  name: string;
  is_head: boolean;
  is_remote: boolean;
  short_sha: string;
  ahead: number | null;
  behind: number | null;
}

export interface CommitInfo {
  oid: string;
  short_sha: string;
  summary: string;
  author_name: string;
  author_email: string;
  author_when: number;
  parent_oids: string[];
}

export interface LogOpts {
  start_ref?: string | null;
  max: number;
  before_oid?: string | null;
}

export interface CommitPage {
  commits: CommitInfo[];
  next_cursor: string | null;
}

export interface CommitOpts {
  amend: boolean;
}

export type DiffLineKind = 'add' | 'del' | 'ctx';

export interface DiffLine {
  kind: DiffLineKind;
  old_no: number | null;
  new_no: number | null;
  text: string;
}

export interface DiffHunk {
  header: string;
  lines: DiffLine[];
}

export interface DiffFile {
  path: string;
  old_path: string | null;
  status: FileStatus;
  binary: boolean;
  hunks: DiffHunk[];
}

export interface DiffPayload {
  files: DiffFile[];
}

export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export interface GitHubUser {
  login: string;
  name: string | null;
  avatar_url: string;
  html_url: string;
}

export interface PrUser {
  login: string;
  avatar_url: string;
}
export interface PrRef {
  ref: string;
  sha: string;
}
export interface PullRequest {
  number: number;
  title: string;
  state: string;
  draft: boolean;
  html_url: string;
  user: PrUser;
  head: PrRef;
  base: PrRef;
  created_at: string;
  updated_at: string;
}

// Note: StashApply is encoded as a tagged object because the Rust enum carries
// associated data. All the simpler Repository-state variants serialize as
// snake_case strings.
export type OpKind =
  | 'clean'
  | 'merge'
  | 'rebase'
  | 'cherry_pick'
  | 'revert'
  | 'bisect'
  | 'apply_mailbox'
  | { stash_apply: { was_pop: boolean; conflicts_present: boolean } };

export interface OpState {
  kind: OpKind;
  conflicted: string[];
}

/** True if `kind` is the StashApply variant. Narrows the type so callers can
 *  read `.was_pop` / `.conflicts_present`. */
export function isStashApply(
  kind: OpKind,
): kind is { stash_apply: { was_pop: boolean; conflicts_present: boolean } } {
  return typeof kind === 'object' && kind !== null && 'stash_apply' in kind;
}

/** Returns true if the OpKind represents any in-progress operation (i.e. not 'clean'). */
export function isOpInProgress(kind: OpKind): boolean {
  return kind !== 'clean';
}

/** Human-readable label for an OpKind, suitable for tooltips and inline error
 *  messages. Returns short verbs like "merge", "cherry-pick", "stash apply",
 *  empty string for 'clean'. */
export function opKindLabel(kind: OpKind): string {
  if (typeof kind === 'object' && kind !== null) {
    if ('stash_apply' in kind) return 'stash apply';
  }
  switch (kind) {
    case 'clean': return '';
    case 'cherry_pick': return 'cherry-pick';
    case 'apply_mailbox': return 'mailbox';
    case 'merge':
    case 'rebase':
    case 'revert':
    case 'bisect':
      return kind;
  }
}

// Backend AppError as a tagged union (Rust serde tag = "kind", snake_case).
export type AppError =
  | { kind: 'repo_not_found'; id: string }
  | { kind: 'dirty'; paths: string[] }
  | { kind: 'merge_conflict'; paths: string[] }
  | { kind: 'unmerged'; name: string }
  | { kind: 'auth'; message: string }
  | { kind: 'github_rate_limited'; retry_after: number }
  | { kind: 'not_a_github_repo' }
  | { kind: 'network'; message: string }
  | { kind: 'io'; message: string }
  | { kind: 'git'; message: string }
  | { kind: 'cancelled' };

export type ThemeName = 'dark' | 'light';

export interface AppSettings {
  theme_override: ThemeName | null;
  /** Canonical path of the most recently active repo. The welcome page
   *  resolves it to a known-repo id on launch and redirects there.
   *  Path-based (not id-based) because registry ids are minted fresh on
   *  every process start, so an id wouldn't survive a restart. */
  last_active_repo_path: string | null;
}

export interface GitIdentity {
  name: string | null;
  email: string | null;
}

export type ResetMode = 'soft' | 'mixed' | 'hard';

export interface StashEntry {
  index: number;
  message: string;
  oid: string;
  short_oid: string;
  branch: string;
  time: number;
}
