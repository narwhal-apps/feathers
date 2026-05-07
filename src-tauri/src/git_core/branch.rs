use crate::error::AppError;
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
