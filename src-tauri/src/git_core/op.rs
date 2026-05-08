use crate::error::AppError;
use git2::{Repository, RepositoryState, ResetType};
use serde::Serialize;

/// Snapshot of any in-progress repo operation (merge, rebase, cherry-pick…)
/// plus the conflicted paths the user still has to resolve. Drives the FE's
/// "Resolve conflicts" modal and its Continue / Abort buttons.
#[derive(Debug, Clone, Serialize)]
pub struct OpState {
    pub kind: OpKind,
    pub conflicted: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpKind {
    Clean,
    Merge,
    Rebase,
    CherryPick,
    Revert,
    Bisect,
    ApplyMailbox,
}

impl OpKind {
    fn from_state(s: RepositoryState) -> Self {
        match s {
            RepositoryState::Clean => OpKind::Clean,
            RepositoryState::Merge => OpKind::Merge,
            RepositoryState::Rebase
            | RepositoryState::RebaseInteractive
            | RepositoryState::RebaseMerge => OpKind::Rebase,
            RepositoryState::CherryPick | RepositoryState::CherryPickSequence => {
                OpKind::CherryPick
            }
            RepositoryState::Revert | RepositoryState::RevertSequence => OpKind::Revert,
            RepositoryState::Bisect => OpKind::Bisect,
            RepositoryState::ApplyMailbox | RepositoryState::ApplyMailboxOrRebase => {
                OpKind::ApplyMailbox
            }
        }
    }
}

pub fn state(repo: &Repository) -> Result<OpState, AppError> {
    let kind = OpKind::from_state(repo.state());
    let conflicted = collect_conflicted_paths(repo)?;
    Ok(OpState { kind, conflicted })
}

fn collect_conflicted_paths(repo: &Repository) -> Result<Vec<String>, AppError> {
    let idx = repo.index()?;
    if !idx.has_conflicts() {
        return Ok(vec![]);
    }
    let mut paths: Vec<String> = idx
        .conflicts()?
        .filter_map(|c| c.ok())
        .filter_map(|c| {
            let entry = c.our.or(c.their).or(c.ancestor)?;
            Some(String::from_utf8_lossy(&entry.path).into_owned())
        })
        .collect();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Continue an in-progress merge or rebase. The index must be conflict-free
/// before calling — returns `MergeConflict` otherwise so the FE keeps the
/// user on the Resolve panel.
pub fn op_continue(repo: &Repository) -> Result<(), AppError> {
    if repo.index()?.has_conflicts() {
        return Err(AppError::MergeConflict {
            paths: collect_conflicted_paths(repo)?,
        });
    }
    match repo.state() {
        RepositoryState::Rebase
        | RepositoryState::RebaseInteractive
        | RepositoryState::RebaseMerge => continue_rebase(repo),
        RepositoryState::Merge => continue_merge(repo),
        RepositoryState::CherryPick | RepositoryState::CherryPickSequence => {
            continue_cherrypick(repo)
        }
        RepositoryState::Revert | RepositoryState::RevertSequence => {
            continue_revert(repo)
        }
        RepositoryState::Clean => Ok(()),
        other => Err(AppError::Git {
            message: format!("continue not supported for state {other:?}"),
        }),
    }
}

fn continue_rebase(repo: &Repository) -> Result<(), AppError> {
    let mut rb = repo.open_rebase(None)?;
    let sig = repo.signature()?;

    // Commit the step the user just resolved. If the rebase wasn't actually
    // paused on a step (e.g. continue called after manual fixes that re-ran
    // the iterator), `commit` may report Applied — treat it as a no-op.
    match rb.commit(None, &sig, None) {
        Ok(_) => {}
        Err(e) if e.code() == git2::ErrorCode::Applied => {}
        Err(e) => return Err(e.into()),
    }

    while let Some(op) = rb.next() {
        op?;
        if repo.index()?.has_conflicts() {
            // Pause again — the FE will see Rebase + conflicts and re-show
            // the panel with the new conflicted set.
            return Ok(());
        }
        rb.commit(None, &sig, None)?;
    }
    rb.finish(None)?;
    Ok(())
}

fn continue_merge(repo: &Repository) -> Result<(), AppError> {
    let head_commit = repo.head()?.peel_to_commit()?;
    let merge_head_oid = read_merge_head(repo)?;
    let merge_commit = repo.find_commit(merge_head_oid)?;

    let msg = read_merge_msg(repo)
        .unwrap_or_else(|| format!("Merge commit '{}'", merge_commit.id()));

    let sig = repo.signature()?;
    let mut idx = repo.index()?;
    let tree_oid = idx.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &tree,
        &[&head_commit, &merge_commit],
    )?;
    repo.cleanup_state()?;
    Ok(())
}

fn continue_cherrypick(repo: &Repository) -> Result<(), AppError> {
    let source_oid = read_cherrypick_head(repo)?;
    let source = repo.find_commit(source_oid)?;
    crate::git_core::history::finalize_cherrypick(repo, &source)
}

fn continue_revert(repo: &Repository) -> Result<(), AppError> {
    let source_oid = read_revert_head(repo)?;
    let source = repo.find_commit(source_oid)?;
    crate::git_core::history::finalize_revert(repo, &source)
}

fn read_cherrypick_head(repo: &Repository) -> Result<git2::Oid, AppError> {
    let path = repo.path().join("CHERRY_PICK_HEAD");
    let raw = std::fs::read_to_string(&path).map_err(|e| AppError::Io {
        message: format!("CHERRY_PICK_HEAD: {e}"),
    })?;
    let line = raw.lines().next().ok_or_else(|| AppError::Git {
        message: "CHERRY_PICK_HEAD is empty".into(),
    })?;
    git2::Oid::from_str(line.trim()).map_err(|e| AppError::Git { message: e.to_string() })
}

fn read_revert_head(repo: &Repository) -> Result<git2::Oid, AppError> {
    let path = repo.path().join("REVERT_HEAD");
    let raw = std::fs::read_to_string(&path).map_err(|e| AppError::Io {
        message: format!("REVERT_HEAD: {e}"),
    })?;
    let line = raw.lines().next().ok_or_else(|| AppError::Git {
        message: "REVERT_HEAD is empty".into(),
    })?;
    git2::Oid::from_str(line.trim()).map_err(|e| AppError::Git { message: e.to_string() })
}

/// Abort whatever's in progress. For rebase: hands off to libgit2's abort
/// (which restores ORIG_HEAD). For merge / revert / cherry-pick / bisect:
/// hard-resets HEAD and clears the .git state files.
pub fn op_abort(repo: &Repository) -> Result<(), AppError> {
    match repo.state() {
        RepositoryState::Rebase
        | RepositoryState::RebaseInteractive
        | RepositoryState::RebaseMerge => {
            let mut rb = repo.open_rebase(None)?;
            rb.abort()?;
            Ok(())
        }
        RepositoryState::Merge
        | RepositoryState::Revert
        | RepositoryState::RevertSequence
        | RepositoryState::CherryPick
        | RepositoryState::CherryPickSequence => {
            let head = repo.head()?.peel(git2::ObjectType::Commit)?;
            repo.reset(&head, ResetType::Hard, None)?;
            repo.cleanup_state()?;
            Ok(())
        }
        RepositoryState::Clean => Ok(()),
        other => Err(AppError::Git {
            message: format!("abort not supported for state {other:?}"),
        }),
    }
}

fn read_merge_head(repo: &Repository) -> Result<git2::Oid, AppError> {
    let path = repo.path().join("MERGE_HEAD");
    let raw = std::fs::read_to_string(&path).map_err(|e| AppError::Io {
        message: format!("MERGE_HEAD: {e}"),
    })?;
    let line = raw.lines().next().ok_or_else(|| AppError::Git {
        message: "MERGE_HEAD is empty".into(),
    })?;
    git2::Oid::from_str(line.trim()).map_err(|e| AppError::Git { message: e.to_string() })
}

fn read_merge_msg(repo: &Repository) -> Option<String> {
    std::fs::read_to_string(repo.path().join("MERGE_MSG")).ok()
}
