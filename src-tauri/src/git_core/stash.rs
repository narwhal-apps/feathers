use crate::error::AppError;
use crate::git_core::types::{FileChange, FileStatus, StashEntry};
use git2::{Delta, DiffOptions, Oid, Patch, Repository, StashFlags};

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

/// Render a `git2::Diff` to a unified-diff string. Returns "" when the diff has
/// no deltas or when libgit2 declines to produce a patch.
fn diff_to_patch_string(diff: &git2::Diff<'_>) -> Result<String, AppError> {
    if diff.deltas().len() == 0 {
        return Ok(String::new());
    }
    let Some(mut p) = Patch::from_diff(diff, 0)? else {
        return Ok(String::new());
    };
    let buf = p.to_buf()?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Return a unified-diff string for one path inside a stash. Returns "" if the
/// path doesn't appear in the stash diff. For stashes created with
/// `--include-untracked`, falls through to parent(2) (the untracked tree) when
/// the tracked diff has no entry for the path.
pub fn diff_file(repo: &mut Repository, index: usize, path: &str) -> Result<String, AppError> {
    let oid = stash_oid_at(repo, index)?;
    let stash_commit = repo.find_commit(oid)?;
    let stash_tree = stash_commit.tree()?;
    let parent = stash_commit.parent(0)?;
    let parent_tree = parent.tree()?;

    let mut opts = DiffOptions::new();
    opts.pathspec(path);
    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&stash_tree), Some(&mut opts))?;
    let s = diff_to_patch_string(&diff)?;
    if !s.is_empty() {
        return Ok(s);
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
        let s = diff_to_patch_string(&u_diff)?;
        if !s.is_empty() {
            return Ok(s);
        }
    }

    Ok(String::new())
}
