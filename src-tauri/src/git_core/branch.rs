use crate::error::AppError;
use crate::git_core::{op, status};
use crate::git_core::types::BranchInfo;
use git2::{BranchType, Repository};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MergeOutcome {
    UpToDate,
    FastForward,
    Merged,
    Conflicted,
}

pub fn list_branches(repo: &Repository) -> Result<Vec<BranchInfo>, AppError> {
    let head_ref = repo.head().ok();
    let mut out = vec![];

    let iter = repo.branches(None)?;
    for b in iter {
        let (branch, btype) = b?;
        let name = branch.name()?.unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }

        let target = branch.get().target();
        let short_sha = target
            .and_then(|oid| repo.find_object(oid, None).ok())
            .and_then(|obj| obj.short_id().ok())
            .and_then(|buf| buf.as_str().map(str::to_string))
            .unwrap_or_default();

        let last_commit_time = target
            .and_then(|oid| repo.find_commit(oid).ok())
            .map(|c| c.time().seconds())
            .unwrap_or(0);

        let is_head = !matches!(btype, BranchType::Remote)
            && head_ref
                .as_ref()
                .and_then(|h| h.name())
                .map(|hn| hn == branch.get().name().unwrap_or(""))
                .unwrap_or(false);

        // ahead/behind only meaningful for local branches with upstreams.
        let (ahead, behind) = if matches!(btype, BranchType::Local) {
            match branch.upstream() {
                Ok(up) => {
                    let local = target;
                    let remote = up.get().target();
                    match (local, remote) {
                        (Some(l), Some(r)) => match repo.graph_ahead_behind(l, r) {
                            Ok((a, b)) => (Some(a), Some(b)),
                            Err(_) => (None, None),
                        },
                        _ => (None, None),
                    }
                }
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        };

        out.push(BranchInfo {
            name,
            is_head,
            is_remote: matches!(btype, BranchType::Remote),
            short_sha,
            ahead,
            behind,
            last_commit_time,
        });
    }

    Ok(out)
}

/// Checkout a branch by name. Accepts:
///   - a local branch name ("feature-x"): just switches HEAD to it.
///   - a remote-tracking branch ("origin/feature-x"): creates a local
///     branch that tracks it (or reuses the existing same-named local),
///     then switches HEAD.
///
/// Errors with `AppError::Dirty { paths }` if the working tree has staged,
/// unstaged or conflicted changes; `AppError::Git { ... }` for not-found /
/// other libgit2 failures.
pub fn checkout(repo: &Repository, branch_name: &str, allow_dirty: bool) -> Result<(), AppError> {
    if !allow_dirty {
        let snap = status::status(repo)?;
        if !snap.staged.is_empty() || !snap.unstaged.is_empty() || !snap.conflicted.is_empty() {
            let mut paths: Vec<String> = snap
                .staged
                .iter()
                .chain(snap.unstaged.iter())
                .chain(snap.conflicted.iter())
                .map(|f| f.path.clone())
                .collect();
            paths.sort();
            paths.dedup();
            return Err(AppError::Dirty { paths });
        }
    }

    // 1. Local branch with this exact name? Switch to it.
    if let Ok(branch) = repo.find_branch(branch_name, BranchType::Local) {
        return finalize_checkout(repo, &branch);
    }

    // 2. Remote-tracking branch like "origin/feature-x"? Create / reuse the
    //    matching local branch, then check it out.
    if let Ok(remote_branch) = repo.find_branch(branch_name, BranchType::Remote) {
        let local_name = branch_name
            .splitn(2, '/')
            .nth(1)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::Git {
                message: format!("invalid remote branch name: {branch_name}"),
            })?;

        let local_branch = match repo.find_branch(local_name, BranchType::Local) {
            Ok(b) => b,
            Err(_) => {
                let target = remote_branch.get().peel_to_commit()?;
                let mut b = repo.branch(local_name, &target, false)?;
                // Best-effort: tracking config is nice but not required.
                let _ = b.set_upstream(Some(branch_name));
                b
            }
        };
        return finalize_checkout(repo, &local_branch);
    }

    Err(AppError::Git {
        message: format!("branch not found: {branch_name}"),
    })
}

fn finalize_checkout(repo: &Repository, branch: &git2::Branch<'_>) -> Result<(), AppError> {
    let refname = branch
        .get()
        .name()
        .ok_or_else(|| AppError::Git {
            message: "branch has no ref name".into(),
        })?
        .to_string();
    let target = branch.get().peel_to_commit()?;
    let tree = target.tree()?;
    let mut opts = git2::build::CheckoutBuilder::new();
    opts.safe();
    repo.checkout_tree(tree.as_object(), Some(&mut opts))?;
    repo.set_head(&refname)?;
    Ok(())
}

/// Create a new local branch. If `from` is `Some(name)`, the new branch starts
/// at that local branch's tip; otherwise it starts at HEAD. Optionally checkout
/// the new branch (which uses the same dirty-tree refusal rules as `checkout`).
pub fn create(
    repo: &Repository,
    name: &str,
    from: Option<&str>,
    checkout: bool,
) -> Result<(), AppError> {
    let start_commit = match from {
        Some(from_name) => {
            let b = repo
                .find_branch(from_name, BranchType::Local)
                .map_err(|_| AppError::Git {
                    message: format!("local branch not found: {from_name}"),
                })?;
            b.get().peel_to_commit()?
        }
        None => repo.head()?.peel_to_commit()?,
    };
    repo.branch(name, &start_commit, false)?;
    if checkout {
        let head_oid = repo.head().ok().and_then(|h| h.target());
        let same_commit = head_oid == Some(start_commit.id());
        return self::checkout(repo, name, same_commit);
    }
    Ok(())
}

/// Create a new local branch at an arbitrary commit and check it out.
/// Refuses to overwrite an existing branch. Refuses on dirty tree because
/// the checkout step would clobber working changes.
pub fn create_at(repo: &Repository, name: &str, oid: git2::Oid) -> Result<(), AppError> {
    if repo.find_branch(name, BranchType::Local).is_ok() {
        return Err(AppError::Git {
            message: format!("branch '{name}' already exists"),
        });
    }
    let target = repo.find_commit(oid)?;
    repo.branch(name, &target, false)?;
    self::checkout(repo, name, false)
}

/// Rename a local branch. Refuses to overwrite an existing branch.
pub fn rename(repo: &Repository, old_name: &str, new_name: &str) -> Result<(), AppError> {
    if old_name == new_name {
        return Ok(());
    }
    let mut branch = repo
        .find_branch(old_name, BranchType::Local)
        .map_err(|_| AppError::Git {
            message: format!("local branch not found: {old_name}"),
        })?;
    branch.rename(new_name, false)?;
    Ok(())
}

/// Delete a local branch by name. Refuses to delete the current HEAD branch.
/// If `force` is false, also refuses when the branch tip is not reachable
/// from HEAD (i.e. would lose commits) and returns `AppError::Unmerged`.
pub fn delete(repo: &Repository, name: &str, force: bool) -> Result<(), AppError> {
    let head_ref = repo.head().ok();
    if let Some(h) = head_ref.as_ref().and_then(|h| h.shorthand()) {
        if h == name {
            return Err(AppError::Git {
                message: format!("cannot delete the current branch: {name}"),
            });
        }
    }

    let mut branch = repo.find_branch(name, BranchType::Local)?;
    let branch_oid = branch.get().target().ok_or_else(|| AppError::Git {
        message: format!("branch has no target: {name}"),
    })?;

    if !force {
        let head_oid = repo.head()?.target().ok_or_else(|| AppError::Git {
            message: "HEAD has no target".into(),
        })?;
        let merged =
            head_oid == branch_oid || repo.graph_descendant_of(head_oid, branch_oid).unwrap_or(false);
        if !merged {
            return Err(AppError::Unmerged {
                name: name.to_string(),
            });
        }
    }

    branch.delete()?;
    Ok(())
}

/// Merge a local branch into the current HEAD. Used by "Update from default
/// branch" — mirrors GitHub Desktop's behaviour: straight merge, no rebase.
///
/// Returns silently when HEAD already contains the source branch's tip
/// (up-to-date). Fast-forwards when possible. On conflict, writes MERGE_MSG
/// and leaves the repo in Merge state so the existing Resolve panel handles it.
pub fn merge_branch(repo: &Repository, source_name: &str) -> Result<MergeOutcome, AppError> {
    op::require_clean(repo)?;

    let snap = status::status(repo)?;
    if !snap.staged.is_empty() || !snap.unstaged.is_empty() || !snap.conflicted.is_empty() {
        let mut paths: Vec<String> = snap
            .staged
            .iter()
            .chain(snap.unstaged.iter())
            .chain(snap.conflicted.iter())
            .map(|f| f.path.clone())
            .collect();
        paths.sort();
        paths.dedup();
        return Err(AppError::Dirty { paths });
    }

    let source = repo
        .find_branch(source_name, BranchType::Local)
        .map_err(|_| AppError::Git {
            message: format!("local branch not found: {source_name}"),
        })?;
    let source_oid = source.get().target().ok_or_else(|| AppError::Git {
        message: format!("branch has no target: {source_name}"),
    })?;
    let annotated = repo.find_annotated_commit(source_oid)?;

    let (analysis, _) = repo.merge_analysis(&[&annotated])?;
    if analysis.is_up_to_date() {
        return Ok(MergeOutcome::UpToDate);
    }

    if analysis.is_fast_forward() {
        fast_forward_head(repo, source_oid)?;
        return Ok(MergeOutcome::FastForward);
    }

    repo.merge(&[&annotated], None, None)?;

    let head_name = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(str::to_string))
        .unwrap_or_else(|| "HEAD".into());
    let merge_msg = format!("Merge branch '{source_name}' into {head_name}\n");

    if repo.index()?.has_conflicts() {
        std::fs::write(repo.path().join("MERGE_MSG"), &merge_msg)?;
        return Ok(MergeOutcome::Conflicted);
    }

    let head_commit = repo.head()?.peel_to_commit()?;
    let source_commit = repo.find_commit(source_oid)?;
    let sig = repo.signature()?;
    let mut idx = repo.index()?;
    let tree_oid = idx.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &merge_msg,
        &tree,
        &[&head_commit, &source_commit],
    )?;
    repo.cleanup_state()?;
    Ok(MergeOutcome::Merged)
}

fn fast_forward_head(repo: &Repository, target_oid: git2::Oid) -> Result<(), AppError> {
    let head_ref_name = repo
        .head()?
        .name()
        .ok_or_else(|| AppError::Git {
            message: "HEAD has no name".into(),
        })?
        .to_string();

    let new_commit = repo.find_commit(target_oid)?;
    repo.checkout_tree(
        new_commit.as_object(),
        Some(git2::build::CheckoutBuilder::new().safe()),
    )?;

    let mut head_ref_mut = repo.find_reference(&head_ref_name)?;
    head_ref_mut.set_target(target_oid, "fast-forward")?;
    repo.set_head(&head_ref_name)?;
    Ok(())
}
