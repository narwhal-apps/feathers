use crate::error::AppError;
use crate::git_core::types::{CommitInfo, CommitPage, LogOpts};
use git2::{Oid, Repository, Sort};

pub fn log(repo: &Repository, opts: LogOpts) -> Result<CommitPage, AppError> {
    let mut walk = repo.revwalk()?;
    walk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;

    match &opts.start_ref {
        Some(r) => walk.push_ref(r)?,
        None => walk.push_head()?,
    }

    // Skip past the cursor commit if pagination requested.
    let mut skipping = opts.before_oid.is_some();
    let cursor_oid = opts.before_oid.as_deref().and_then(|s| Oid::from_str(s).ok());

    let mut commits = Vec::with_capacity(opts.max);
    let mut next_cursor: Option<String> = None;

    for oid_result in walk.by_ref() {
        let oid = oid_result?;
        if skipping {
            if Some(oid) == cursor_oid {
                skipping = false;
                // Don't continue here - we want to process this commit
            } else {
                continue;
            }
        }
        if commits.len() == opts.max {
            next_cursor = Some(oid.to_string());
            break;
        }
        let c = repo.find_commit(oid)?;
        let short_sha = c.as_object().short_id()?
            .as_str().unwrap_or("").to_string();
        commits.push(CommitInfo {
            oid: oid.to_string(),
            short_sha,
            summary: c.summary().unwrap_or("").to_string(),
            author_name: c.author().name().unwrap_or("").to_string(),
            author_email: c.author().email().unwrap_or("").to_string(),
            author_when: c.time().seconds(),
            parent_oids: (0..c.parent_count())
                .filter_map(|i| c.parent_id(i).ok().map(|o| o.to_string()))
                .collect(),
        });
    }

    Ok(CommitPage { commits, next_cursor })
}
