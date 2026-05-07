use crate::error::AppError;
use crate::git_core::types::{CommitInfo, CommitPage, LogOpts};
use git2::{Oid, Repository, Signature, Sort};

pub fn log(repo: &Repository, opts: LogOpts) -> Result<CommitPage, AppError> {
    let mut walk = repo.revwalk()?;
    walk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;

    match &opts.start_ref {
        Some(r) => walk.push_ref(r)?,
        None => walk.push_head()?,
    }

    // Skip past the cursor commit if pagination requested.
    let mut skipping = opts.before_oid.is_some();
    let cursor_oid = opts
        .before_oid
        .as_deref()
        .and_then(|s| Oid::from_str(s).ok());

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
        let short_sha = c.as_object().short_id()?.as_str().unwrap_or("").to_string();
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

    Ok(CommitPage {
        commits,
        next_cursor,
    })
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct CommitOpts {
    /// Replace HEAD instead of creating a new commit on top of it.
    #[serde(default)]
    pub amend: bool,
}

/// Create a new commit from the current index. Falls back to a placeholder
/// signature if `user.name` / `user.email` aren't configured.
pub fn create(repo: &Repository, message: &str, opts: CommitOpts) -> Result<String, AppError> {
    if message.trim().is_empty() {
        return Err(AppError::Git {
            message: "commit message cannot be empty".into(),
        });
    }

    let sig = repo.signature().or_else(|_| {
        Signature::now("Unknown", "unknown@local").map_err(AppError::from)
    })?;

    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    if opts.amend {
        let head_commit = repo.head()?.peel_to_commit()?;
        let oid = head_commit.amend(
            Some("HEAD"),
            Some(&sig),
            Some(&sig),
            None,
            Some(message),
            Some(&tree),
        )?;
        return Ok(oid.to_string());
    }

    let parents: Vec<git2::Commit> = match repo.head() {
        Ok(h) => vec![h.peel_to_commit()?],
        Err(e)
            if e.code() == git2::ErrorCode::UnbornBranch
                || e.code() == git2::ErrorCode::NotFound =>
        {
            vec![]
        }
        Err(e) => return Err(e.into()),
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)?;
    Ok(oid.to_string())
}

/// Undo the most recent commit by soft-resetting HEAD to its first parent.
/// The tree and index keep the committed contents, so the changes reappear
/// as staged work the user can re-commit.
pub fn undo_last(repo: &Repository) -> Result<(), AppError> {
    let head_commit = repo.head()?.peel_to_commit()?;
    if head_commit.parent_count() == 0 {
        return Err(AppError::Git {
            message: "cannot undo: this is the first commit on the branch".into(),
        });
    }
    let parent = head_commit.parent(0)?;
    repo.reset(parent.as_object(), git2::ResetType::Soft, None)?;
    Ok(())
}
