use crate::error::AppError;
use crate::git_core::types::StashEntry;
use git2::{Oid, Repository, StashFlags};

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
