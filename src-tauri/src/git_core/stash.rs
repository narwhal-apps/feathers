use crate::error::AppError;
use crate::git_core::diff::diff_to_payload;
use crate::git_core::op;
use crate::git_core::types::{DiffPayload, FileChange, FileStatus, StashEntry};
use git2::{Delta, DiffOptions, Oid, Repository, StashFlags};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// List all stashes, newest first (index 0 = newest).
pub fn list(repo: &mut Repository) -> Result<Vec<StashEntry>, AppError> {
    // stash_foreach can't borrow repo mutably inside its callback, so we
    // collect (index, message, oid) tuples first and look up commit metadata
    // afterwards.
    let mut raw: Vec<(usize, String, Oid)> = Vec::new();
    repo.stash_foreach(|index, msg, oid| {
        raw.push((index, msg.to_string(), *oid));
        true
    })?;

    let mut out = Vec::with_capacity(raw.len());
    for (index, message, oid) in raw {
        let commit = repo.find_commit(oid)?;
        let short_oid = commit
            .as_object()
            .short_id()
            .ok()
            .and_then(|b| b.as_str().map(str::to_string))
            .unwrap_or_default();
        let branch = parse_branch_from_message(&message);
        out.push(StashEntry {
            index,
            message,
            oid: oid.to_string(),
            short_oid,
            branch,
            time: commit.time().seconds(),
        });
    }
    Ok(out)
}

fn parse_branch_from_message(msg: &str) -> String {
    // Auto-message: "WIP on <branch>: <oid> <subject>"
    // User message: "On <branch>: <user message>"
    if let Some(rest) = msg.strip_prefix("WIP on ") {
        if let Some(colon) = rest.find(':') {
            return rest[..colon].to_string();
        }
    }
    if let Some(rest) = msg.strip_prefix("On ") {
        if let Some(colon) = rest.find(':') {
            return rest[..colon].to_string();
        }
    }
    String::new()
}

/// Snapshot working-tree + (optionally) untracked into a new stash.
/// Returns the stash commit oid. `AppError::Git { message: "nothing to stash" }`
/// when the working tree has nothing to capture.
pub fn create(
    repo: &mut Repository,
    message: Option<&str>,
    include_untracked: bool,
    keep_index: bool,
) -> Result<Oid, AppError> {
    let stasher = repo.signature()?;
    let mut flags = StashFlags::DEFAULT;
    if include_untracked {
        flags |= StashFlags::INCLUDE_UNTRACKED;
    }
    if keep_index {
        flags |= StashFlags::KEEP_INDEX;
    }

    repo.stash_save2(&stasher, message, Some(flags))
        .map_err(|e| {
            // libgit2 returns NotFound when there's nothing to stash.
            if e.code() == git2::ErrorCode::NotFound {
                AppError::Git {
                    message: "nothing to stash".into(),
                }
            } else {
                e.into()
            }
        })
}

/// Drop the stash at the given index. `AppError::Git { message: "no stash at index N" }`
/// if the index is out of range.
pub fn drop_at(repo: &mut Repository, index: usize) -> Result<(), AppError> {
    repo.stash_drop(index).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            AppError::Git {
                message: format!("no stash at index {index}"),
            }
        } else {
            e.into()
        }
    })
}

/// Look up the stash commit oid at the given index. Errors with
/// `Git { message: "no stash at index N" }` if the index is out of range.
fn stash_oid_at(repo: &mut Repository, index: usize) -> Result<Oid, AppError> {
    let mut found: Option<Oid> = None;
    repo.stash_foreach(|i, _msg, oid| {
        if i == index {
            found = Some(*oid);
            return false; // stop iterating
        }
        true
    })?;
    found.ok_or_else(|| AppError::Git {
        message: format!("no stash at index {index}"),
    })
}

fn map_delta(d: Delta) -> Option<FileStatus> {
    match d {
        Delta::Added => Some(FileStatus::Added),
        Delta::Deleted => Some(FileStatus::Deleted),
        Delta::Modified => Some(FileStatus::Modified),
        Delta::Renamed => Some(FileStatus::Renamed),
        Delta::Typechange => Some(FileStatus::Typechange),
        Delta::Untracked => Some(FileStatus::Untracked),
        Delta::Conflicted => Some(FileStatus::Conflicted),
        _ => None,
    }
}

/// Collect FileChange entries from a `git2::Diff` into `out`.
fn collect_file_changes(
    diff: &git2::Diff<'_>,
    out: &mut Vec<FileChange>,
) -> Result<(), AppError> {
    diff.foreach(
        &mut |delta, _| {
            let status = match map_delta(delta.status()) {
                Some(s) => s,
                None => return true,
            };
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            let old_path = if delta.status() == Delta::Renamed {
                delta
                    .old_file()
                    .path()
                    .map(|p| p.to_string_lossy().into_owned())
            } else {
                None
            };
            out.push(FileChange {
                path,
                old_path,
                status,
            });
            true
        },
        None,
        None,
        None,
    )?;
    Ok(())
}

/// List the files changed in the stash at the given index. Compares the stash
/// commit's tree to its first parent (the WIP-base). For stashes created with
/// `--include-untracked`, untracked files live in the third parent's tree
/// (libgit2 stores them on a separate parent), so we merge those in too.
pub fn show_files(repo: &mut Repository, index: usize) -> Result<Vec<FileChange>, AppError> {
    let oid = stash_oid_at(repo, index)?;
    let stash_commit = repo.find_commit(oid)?;
    let stash_tree = stash_commit.tree()?;
    let parent = stash_commit.parent(0)?;
    let parent_tree = parent.tree()?;

    let mut opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&stash_tree), Some(&mut opts))?;

    let mut out = Vec::new();
    collect_file_changes(&diff, &mut out)?;

    // Untracked files (when stashed with INCLUDE_UNTRACKED) live in parent(2).
    if stash_commit.parent_count() >= 3 {
        let untracked = stash_commit.parent(2)?;
        let untracked_tree = untracked.tree()?;
        let mut u_opts = DiffOptions::new();
        u_opts.include_untracked(true);
        let u_diff = repo.diff_tree_to_tree(None, Some(&untracked_tree), Some(&mut u_opts))?;
        collect_file_changes(&u_diff, &mut out)?;
    }

    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Return a structured diff payload for one path inside a stash. Returns an
/// empty payload (`files: []`) if the path doesn't appear in the stash diff.
/// For stashes created with `--include-untracked`, falls through to parent(2)
/// (the untracked tree) when the tracked diff has no entry for the path.
pub fn diff_file(
    repo: &mut Repository,
    index: usize,
    path: &str,
) -> Result<DiffPayload, AppError> {
    let oid = stash_oid_at(repo, index)?;
    let stash_commit = repo.find_commit(oid)?;
    let stash_tree = stash_commit.tree()?;
    let parent = stash_commit.parent(0)?;
    let parent_tree = parent.tree()?;

    let mut opts = DiffOptions::new();
    opts.pathspec(path);
    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&stash_tree), Some(&mut opts))?;
    if diff.deltas().len() > 0 {
        let payload = diff_to_payload(&diff)?;
        if !payload.files.is_empty() {
            return Ok(payload);
        }
    }

    // Untracked files (when stashed with INCLUDE_UNTRACKED) live on parent(2).
    if stash_commit.parent_count() >= 3 {
        let untracked = stash_commit.parent(2)?;
        let untracked_tree = untracked.tree()?;
        let mut u_opts = DiffOptions::new();
        u_opts.pathspec(path);
        u_opts.include_untracked(true);
        let u_diff =
            repo.diff_tree_to_tree(None, Some(&untracked_tree), Some(&mut u_opts))?;
        if u_diff.deltas().len() > 0 {
            let payload = diff_to_payload(&u_diff)?;
            if !payload.files.is_empty() {
                return Ok(payload);
            }
        }
    }

    Ok(DiffPayload { files: vec![] })
}

/// Sidecar describing an in-flight `stash_apply` that may have left conflicts
/// in the index. Lives at `.git/feathers/STASH_APPLY.json`. The presence of
/// this file (combined with libgit2's RepositoryState being Clean) is how
/// `op::state()` detects we're mid-stash-apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashApplySidecar {
    pub index: usize,
    pub was_pop: bool,
    pub stash_oid: String,
}

const SIDECAR_DIR: &str = "feathers";
const SIDECAR_FILE: &str = "STASH_APPLY.json";

fn sidecar_path(repo: &Repository) -> PathBuf {
    repo.path().join(SIDECAR_DIR).join(SIDECAR_FILE)
}

pub(crate) fn write_sidecar(repo: &Repository, sc: &StashApplySidecar) -> Result<(), AppError> {
    let dir = repo.path().join(SIDECAR_DIR);
    fs::create_dir_all(&dir).map_err(|e| AppError::Io {
        message: format!("create {SIDECAR_DIR} dir: {e}"),
    })?;
    let raw = serde_json::to_string(sc).map_err(|e| AppError::Io {
        message: format!("serialize sidecar: {e}"),
    })?;
    let final_path = sidecar_path(repo);
    let tmp_path = final_path.with_extension("json.tmp");
    fs::write(&tmp_path, raw).map_err(|e| AppError::Io {
        message: format!("write sidecar tmp: {e}"),
    })?;
    fs::rename(&tmp_path, &final_path).map_err(|e| AppError::Io {
        message: format!("rename sidecar tmp -> final: {e}"),
    })?;
    Ok(())
}

pub(crate) fn read_sidecar(repo: &Repository) -> Result<Option<StashApplySidecar>, AppError> {
    let path = sidecar_path(repo);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| AppError::Io {
        message: format!("read sidecar: {e}"),
    })?;
    let sc: StashApplySidecar = serde_json::from_str(&raw).map_err(|e| AppError::Io {
        message: format!("parse sidecar: {e}"),
    })?;
    Ok(Some(sc))
}

pub(crate) fn delete_sidecar(repo: &Repository) -> Result<(), AppError> {
    let path = sidecar_path(repo);
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(&path).map_err(|e| AppError::Io {
        message: format!("delete sidecar: {e}"),
    })?;
    Ok(())
}

/// Test-only convenience to plant a sidecar without going through apply.
/// Public so integration tests in `tests/` can call it; otherwise unused.
#[doc(hidden)]
pub fn write_sidecar_for_test(
    repo: &Repository,
    index: usize,
    was_pop: bool,
    stash_oid: &str,
) -> Result<(), AppError> {
    write_sidecar(
        repo,
        &StashApplySidecar {
            index,
            was_pop,
            stash_oid: stash_oid.to_string(),
        },
    )
}

fn require_clean_op_state(repo: &Repository) -> Result<(), AppError> {
    let st = op::state(repo)?;
    if !matches!(st.kind, op::OpKind::Clean) {
        return Err(AppError::Git {
            message: format!("{:?} in progress — finish or abort it first", st.kind),
        });
    }
    Ok(())
}

/// Apply the stash at `index` onto the working tree. Writes a sidecar before
/// invoking libgit2, then:
///   - if the apply produced no conflicts, removes the sidecar; the stash is kept.
///   - if conflicts exist, leaves the sidecar in place. The Resolve panel
///     (driven by `op::state()` returning `StashApply`) takes over.
pub fn apply(repo: &mut Repository, index: usize) -> Result<(), AppError> {
    require_clean_op_state(repo)?;

    let oid = stash_oid_at(repo, index)?;
    write_sidecar(
        repo,
        &StashApplySidecar {
            index,
            was_pop: false,
            stash_oid: oid.to_string(),
        },
    )?;

    if let Err(e) = do_stash_apply(repo, index) {
        // Sidecar would otherwise lock out future apply attempts; roll it back.
        let _ = delete_sidecar(repo);
        return Err(e);
    }

    if !repo.index()?.has_conflicts() {
        delete_sidecar(repo)?;
    }
    Ok(())
}

/// Apply the stash at `index` and drop it on success. On conflict, the stash
/// is NOT dropped — the user needs to resolve via the Resolve panel; the
/// `op_continue` arm will drop the stash after a clean resolution.
pub fn pop(repo: &mut Repository, index: usize) -> Result<(), AppError> {
    require_clean_op_state(repo)?;

    let oid = stash_oid_at(repo, index)?;
    write_sidecar(
        repo,
        &StashApplySidecar {
            index,
            was_pop: true,
            stash_oid: oid.to_string(),
        },
    )?;

    if let Err(e) = do_stash_apply(repo, index) {
        // Sidecar would otherwise lock out future apply attempts; roll it back.
        let _ = delete_sidecar(repo);
        return Err(e);
    }

    if !repo.index()?.has_conflicts() {
        // Clean apply: drop the stash and clear the sidecar.
        repo.stash_drop(index)?;
        delete_sidecar(repo)?;
    }
    Ok(())
}

fn do_stash_apply(repo: &mut Repository, index: usize) -> Result<(), AppError> {
    // Default StashApplyOptions: libgit2 writes conflict markers + index
    // entries on conflict and returns Ok. If a future version of git2-rs or
    // libgit2 changes that, the conflict tests in this module will fail and
    // surface the breakage.
    repo.stash_apply(index, None).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            AppError::Git {
                message: format!("no stash at index {index}"),
            }
        } else {
            e.into()
        }
    })
}
