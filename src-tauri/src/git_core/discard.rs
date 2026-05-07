use crate::error::AppError;
use crate::git_core::status;
use git2::{build::CheckoutBuilder, Repository};
use std::collections::HashSet;
use std::path::Path;

/// Discard workdir changes for the given paths.
///
/// For each path:
///   - If untracked, remove the file from disk.
///   - If tracked (modified, deleted, type-changed, etc.), restore it from
///     HEAD by force-checkout. This overwrites unstaged AND staged workdir
///     state, returning the file to its committed contents.
///
/// Conflicted paths are left alone — discard for those goes through a
/// dedicated "abort merge" / "use ours/theirs" flow (out of MVP scope here).
pub fn discard_paths(repo: &Repository, paths: &[String]) -> Result<(), AppError> {
    if paths.is_empty() {
        return Ok(());
    }

    let snap = status::status(repo)?;
    let untracked: HashSet<&str> =
        snap.untracked.iter().map(|f| f.path.as_str()).collect();

    let workdir = repo
        .workdir()
        .ok_or_else(|| AppError::Git {
            message: "repository has no working directory (bare repo?)".into(),
        })?
        .to_path_buf();

    let mut to_checkout: Vec<&str> = Vec::new();
    for p in paths {
        if untracked.contains(p.as_str()) {
            // Resolve safely under the workdir; refuse anything that escapes.
            let abs = workdir.join(p);
            if !abs.starts_with(&workdir) {
                return Err(AppError::Git {
                    message: format!("refusing to discard path outside workdir: {p}"),
                });
            }
            match std::fs::remove_file(&abs) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(AppError::Io { message: e.to_string() }),
            }
        } else {
            to_checkout.push(p.as_str());
        }
    }

    if !to_checkout.is_empty() {
        let mut opts = CheckoutBuilder::new();
        opts.force();
        opts.remove_untracked(false);
        for p in &to_checkout {
            opts.path(Path::new(p));
        }
        repo.checkout_head(Some(&mut opts))?;
    }

    Ok(())
}
