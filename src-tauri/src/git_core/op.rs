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
    StashApply { was_pop: bool, conflicts_present: bool },
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

impl std::fmt::Display for OpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OpKind::Clean => "clean",
            OpKind::Merge => "merge",
            OpKind::Rebase => "rebase",
            OpKind::CherryPick => "cherry-pick",
            OpKind::Revert => "revert",
            OpKind::Bisect => "bisect",
            OpKind::ApplyMailbox => "mailbox",
            OpKind::StashApply { .. } => "stash apply",
        };
        f.write_str(s)
    }
}

/// Error out unless the repo is in a Clean op state. Used by every history /
/// stash mutation that can't safely run with a merge/rebase/cherry-pick/etc.
/// in flight. Centralised here so the error string ("<op> in progress —
/// finish or abort it first") stays consistent.
pub fn require_clean(repo: &Repository) -> Result<(), AppError> {
    let st = state(repo)?;
    if !matches!(st.kind, OpKind::Clean) {
        return Err(AppError::Git {
            message: format!("{} in progress — finish or abort it first", st.kind),
        });
    }
    Ok(())
}

pub fn state(repo: &Repository) -> Result<OpState, AppError> {
    let raw_state = repo.state();
    let conflicted = collect_conflicted_paths(repo)?;

    // Real repository operation wins: a real merge/rebase/cherry-pick/revert
    // takes precedence over our stash sidecar.
    if !matches!(raw_state, RepositoryState::Clean) {
        let kind = OpKind::from_state(raw_state);
        return Ok(OpState { kind, conflicted });
    }

    // Otherwise check our stash sidecar.
    if let Some(sc) = crate::git_core::stash::read_sidecar(repo)? {
        let conflicts_present = !conflicted.is_empty();
        let _ = sc.stash_oid; // sidecar's stash_oid only needed at continue time
        return Ok(OpState {
            kind: OpKind::StashApply {
                was_pop: sc.was_pop,
                conflicts_present,
            },
            conflicted,
        });
    }

    Ok(OpState {
        kind: OpKind::Clean,
        conflicted,
    })
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
        RepositoryState::Clean => {
            // Stash apply uses our sidecar; check for it.
            if let Some(sc) = crate::git_core::stash::read_sidecar(repo)? {
                continue_stash_apply(repo, sc)
            } else {
                Ok(())
            }
        }
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

fn continue_stash_apply(
    repo: &Repository,
    sc: crate::git_core::stash::StashApplySidecar,
) -> Result<(), AppError> {
    // The has_conflicts check at the top of op_continue already guards us, so
    // by the time we get here the index is conflict-free.
    if sc.was_pop {
        // Verify the stash at sc.index still has the expected oid before
        // dropping. If anything changed (another window dropped/created a
        // stash), we refuse to drop the wrong thing — but we still clear our
        // sidecar so the user can move on.
        let actual = stash_oid_at_for_continue(repo, sc.index);
        match actual {
            Ok(oid) if oid.to_string() == sc.stash_oid => {
                // Safe to drop. Re-open as a mutable handle because
                // stash_drop requires &mut self in git2-rs.
                let mut repo_mut = git2::Repository::open(
                    repo.path().parent().unwrap_or(repo.path()),
                )?;
                repo_mut.stash_drop(sc.index)?;
                // If we crash between stash_drop above and delete_sidecar
                // below, the next op_continue sees the sidecar pointing at a
                // dropped/shifted stash, hits the mismatch arm, and self-heals
                // by deleting the sidecar. Noisy but safe.
            }
            _ => {
                crate::git_core::stash::delete_sidecar(repo)?;
                return Err(AppError::Git {
                    message: "stash no longer at expected position — drop manually".into(),
                });
            }
        }
    }
    crate::git_core::stash::delete_sidecar(repo)?;
    Ok(())
}

/// Walk stashes via a freshly-opened mutable handle so we can use
/// stash_foreach (which requires &mut). Used at op_continue time to verify
/// the stash at the stored index still has the expected oid.
///
/// Re-opening the repo here is safe because git2::Repository is a thin
/// handle around the on-disk `.git/`; multiple handles coexist fine.
fn stash_oid_at_for_continue(repo: &Repository, index: usize) -> Result<git2::Oid, AppError> {
    let mut repo_mut = git2::Repository::open(repo.path().parent().unwrap_or(repo.path()))?;
    let mut found: Option<git2::Oid> = None;
    repo_mut.stash_foreach(|i, _msg, oid| {
        if i == index {
            found = Some(*oid);
            return false;
        }
        true
    })?;
    found.ok_or_else(|| AppError::Git {
        message: format!("no stash at index {index}"),
    })
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
        RepositoryState::Clean => {
            // Stash apply: discard in-progress conflict resolution by
            // hard-resetting back to HEAD, then clear sidecar.
            // Stash itself is untouched.
            if crate::git_core::stash::read_sidecar(repo)?.is_some() {
                let head = repo.head()?.peel(git2::ObjectType::Commit)?;
                repo.reset(&head, ResetType::Hard, None)?;
                crate::git_core::stash::delete_sidecar(repo)?;
            }
            Ok(())
        }
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
