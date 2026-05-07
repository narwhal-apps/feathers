use crate::error::AppError;
use crate::git_core::status;
use crate::git_core::types::BranchInfo;
use git2::{BranchType, Repository};

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
        });
    }

    Ok(out)
}

/// Checkout a local branch by name. Errors with `AppError::Dirty { paths }` if
/// there are staged or unstaged changes (untracked files are allowed).
/// Errors with `AppError::Git { ... }` if the branch is not found or libgit2
/// refuses the checkout for any other reason.
pub fn checkout(repo: &Repository, branch_name: &str) -> Result<(), AppError> {
    // Refuse if working tree has tracked modifications.
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

    // Resolve the branch (local only — remotes need a tracking branch first).
    let branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|_| AppError::Git {
            message: format!("local branch not found: {branch_name}"),
        })?;
    let refname = branch
        .get()
        .name()
        .ok_or_else(|| AppError::Git {
            message: "branch has no ref name".into(),
        })?
        .to_string();

    // Move HEAD's working tree to the branch tip, then point HEAD at the ref.
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
        return self::checkout(repo, name);
    }
    Ok(())
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
