use crate::error::AppError;
use crate::git_core::{op, status};
use git2::{Oid, Repository, ResetType};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResetMode { Soft, Mixed, Hard }

impl From<ResetMode> for ResetType {
    fn from(m: ResetMode) -> Self {
        match m {
            ResetMode::Soft => ResetType::Soft,
            ResetMode::Mixed => ResetType::Mixed,
            ResetMode::Hard => ResetType::Hard,
        }
    }
}

fn require_clean_tree(repo: &Repository) -> Result<(), AppError> {
    let snap = status::status(repo)?;
    let mut paths: Vec<String> = snap.staged.iter().map(|c| c.path.clone()).collect();
    paths.extend(snap.unstaged.iter().map(|c| c.path.clone()));
    paths.extend(snap.conflicted.iter().map(|c| c.path.clone()));
    paths.sort();
    paths.dedup();
    if !paths.is_empty() {
        return Err(AppError::Dirty { paths });
    }
    Ok(())
}


/// Cherry-pick a commit onto HEAD. If the result has no conflicts, commit
/// immediately with the source's message + a `(cherry picked from commit
/// <oid>)` trailer. If conflicts arise, leave the OpState as CherryPick and
/// let the user resolve via the existing Resolve panel; `op_continue` will
/// finish the commit.
pub fn cherrypick(repo: &Repository, oid: Oid) -> Result<(), AppError> {
    op::require_clean(repo)?;
    require_clean_tree(repo)?;

    let target = repo.find_commit(oid)?;
    repo.cherrypick(&target, None)?;

    if repo.index()?.has_conflicts() {
        // Leave the state alone — Resolve panel takes over.
        return Ok(());
    }

    finalize_cherrypick(repo, &target)
}

/// Internal: finalize a clean cherry-pick by writing the commit and clearing
/// the .git state files. Also reused by `op_continue`'s CherryPick arm.
pub(crate) fn finalize_cherrypick(repo: &Repository, source: &git2::Commit<'_>) -> Result<(), AppError> {
    let head_commit = repo.head()?.peel_to_commit()?;
    let mut idx = repo.index()?;
    let tree_oid = idx.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let sig = repo.signature()?;

    let trailer = format!("(cherry picked from commit {})", source.id());
    let body = source.message().unwrap_or("");
    let msg = if body.contains(&trailer) {
        body.to_string()
    } else if body.ends_with('\n') {
        format!("{body}\n{trailer}\n")
    } else {
        format!("{body}\n\n{trailer}\n")
    };

    repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&head_commit])?;
    repo.cleanup_state()?;
    Ok(())
}

/// Revert a commit on top of HEAD. Same conflict semantics as cherry-pick;
/// `finalize_revert` produces a `Revert "<subject>"` commit body.
pub fn revert(repo: &Repository, oid: Oid) -> Result<(), AppError> {
    op::require_clean(repo)?;
    require_clean_tree(repo)?;

    let target = repo.find_commit(oid)?;
    if target.parent_count() > 1 {
        return Err(AppError::Git {
            message: "merge commit revert not supported yet — pick a parent".into(),
        });
    }
    repo.revert(&target, None)?;

    if repo.index()?.has_conflicts() {
        return Ok(());
    }
    finalize_revert(repo, &target)
}

/// Internal: finalize a clean revert. Reused by `op_continue`'s Revert arm.
pub(crate) fn finalize_revert(repo: &Repository, source: &git2::Commit<'_>) -> Result<(), AppError> {
    let head_commit = repo.head()?.peel_to_commit()?;
    let mut idx = repo.index()?;
    let tree_oid = idx.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let sig = repo.signature()?;

    let subject = source.summary().unwrap_or("commit");
    let msg = format!(
        "Revert \"{subject}\"\n\nThis reverts commit {}.\n",
        source.id()
    );

    repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&head_commit])?;
    repo.cleanup_state()?;
    Ok(())
}

/// Move HEAD to the given commit. Soft + Mixed never refuse — they preserve
/// working-tree changes. Hard refuses only if another op is in progress
/// (you can't reset --hard during a merge).
pub fn reset(repo: &Repository, oid: Oid, mode: ResetMode) -> Result<(), AppError> {
    if matches!(mode, ResetMode::Hard) {
        op::require_clean(repo)?;
    }
    let target = repo.find_object(oid, None)?;
    repo.reset(&target, mode.into(), None)?;
    Ok(())
}
