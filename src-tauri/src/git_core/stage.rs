use crate::error::AppError;
use git2::{ObjectType, Repository};
use std::path::Path;

/// Stage the given paths into the index. Handles new, modified, and deleted
/// files by combining `update_all` (catches mods + deletions of tracked files)
/// and `add_all` (catches new untracked files).
pub fn stage_files(repo: &Repository, paths: &[String]) -> Result<(), AppError> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut index = repo.index()?;
    let pathspecs: Vec<&str> = paths.iter().map(|p| p.as_str()).collect();
    // Modifications + deletions of tracked files.
    index.update_all(pathspecs.iter().copied(), None)?;
    // Untracked / new files (and re-stage of modifications).
    index.add_all(pathspecs.iter().copied(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

/// Unstage the given paths — reset their index entries to HEAD's state, or
/// remove them from the index entirely if HEAD doesn't exist yet (initial commit).
pub fn unstage_files(repo: &Repository, paths: &[String]) -> Result<(), AppError> {
    if paths.is_empty() {
        return Ok(());
    }
    let pathspecs: Vec<&str> = paths.iter().map(|p| p.as_str()).collect();

    match repo.head() {
        Ok(head) => {
            let target = head.peel(ObjectType::Commit)?;
            repo.reset_default(Some(&target), pathspecs.iter().copied())?;
            Ok(())
        }
        Err(e)
            if e.code() == git2::ErrorCode::UnbornBranch
                || e.code() == git2::ErrorCode::NotFound =>
        {
            // No HEAD yet — drop entries from the index outright.
            let mut index = repo.index()?;
            for path in paths {
                // Ignore "not in index" — equivalent to nothing to unstage.
                let _ = index.remove_path(Path::new(path));
            }
            index.write()?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
