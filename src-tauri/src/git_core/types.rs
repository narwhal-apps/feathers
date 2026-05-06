use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadInfo {
    /// Branch shorthand (e.g. "main") or "HEAD" if detached.
    pub branch: String,
    /// True if HEAD is detached.
    pub detached: bool,
    /// Short SHA of HEAD's commit, or empty if the repo has no commits.
    pub short_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
    /// Short SHA of the branch tip.
    pub short_sha: String,
    /// Commits ahead/behind upstream (None if no upstream).
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Typechange,
    Untracked,
    Conflicted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    /// For renames, the original path.
    pub old_path: Option<String>,
    pub status: FileStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSnapshot {
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
    pub conflicted: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub oid: String,
    pub short_sha: String,
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    pub author_when: i64,        // seconds since unix epoch
    pub parent_oids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOpts {
    /// Ref to start from. None = HEAD.
    pub start_ref: Option<String>,
    pub max: usize,
    /// Continue after this OID (for pagination).
    pub before_oid: Option<String>,
}

impl Default for LogOpts {
    fn default() -> Self {
        Self { start_ref: None, max: 50, before_oid: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitPage {
    pub commits: Vec<CommitInfo>,
    /// OID to pass as `before_oid` for the next page; None when no more.
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineKind {
    Add,
    Del,
    Ctx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_no: Option<u32>,
    pub new_no: Option<u32>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    pub path: String,
    pub old_path: Option<String>,
    pub status: FileStatus,
    pub binary: bool,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPayload {
    pub files: Vec<DiffFile>,
}
