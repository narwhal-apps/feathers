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

export type OpKind =
  | 'clean'
  | 'merge'
  | 'rebase'
  | 'cherry_pick'
  | 'revert'
  | 'bisect'
  | 'apply_mailbox';

export interface OpState {
  kind: OpKind;
  conflicted: string[];
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
}

export interface GitIdentity {
  name: string | null;
  email: string | null;
}

export type ResetMode = 'soft' | 'mixed' | 'hard';
