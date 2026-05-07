use crate::error::AppError;
use crate::git_core::status;
use git2::{build::CheckoutBuilder, DiffOptions, Patch, Repository};
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

/// Discard a single hunk's worth of workdir changes for `path` by splicing
/// the corresponding HEAD lines back over the workdir's lines for the same
/// range. The `hunk_index` is the zero-based index of the hunk within the
/// file's HEAD→workdir diff (matching what the FE rendered).
pub fn discard_hunk(
    repo: &Repository,
    path: &str,
    hunk_index: usize,
) -> Result<(), AppError> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| AppError::Git {
            message: "repository has no working directory (bare repo?)".into(),
        })?
        .to_path_buf();
    let abs = workdir.join(path);
    if !abs.starts_with(&workdir) {
        return Err(AppError::Git {
            message: format!("refusing to discard path outside workdir: {path}"),
        });
    }

    // HEAD contents (empty when the file is brand new).
    let head_tree = repo.head().and_then(|h| h.peel_to_tree()).ok();
    let old_content: String = match head_tree
        .as_ref()
        .and_then(|t| t.get_path(Path::new(path)).ok())
    {
        Some(entry) => {
            let blob = repo.find_blob(entry.id())?;
            String::from_utf8(blob.content().to_vec()).map_err(|_| AppError::Git {
                message: "binary file: per-hunk discard not supported".into(),
            })?
        }
        None => String::new(),
    };

    // Workdir contents (empty when the file was deleted).
    let new_content: String = match std::fs::read_to_string(&abs) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(AppError::Io { message: e.to_string() }),
    };

    // Recompute the same diff the FE saw so hunk indices line up.
    let mut opts = DiffOptions::new();
    opts.context_lines(3)
        .interhunk_lines(0)
        .ignore_submodules(true)
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true)
        .show_binary(false)
        .pathspec(path);
    let diff = repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut opts))?;

    let patch = Patch::from_diff(&diff, 0)?.ok_or_else(|| AppError::Git {
        message: format!("no patch for {path}"),
    })?;
    let n = patch.num_hunks();
    if hunk_index >= n {
        return Err(AppError::Git {
            message: format!("hunk {hunk_index} out of range (file has {n})"),
        });
    }
    let (hunk, _) = patch.hunk(hunk_index)?;
    let old_start = hunk.old_start() as usize;
    let old_lines = hunk.old_lines() as usize;
    let new_start = hunk.new_start() as usize;
    let new_lines = hunk.new_lines() as usize;

    // split_inclusive keeps trailing newlines so the splice preserves the
    // file's line structure.
    let old_split: Vec<&str> = old_content.split_inclusive('\n').collect();
    let new_split: Vec<&str> = new_content.split_inclusive('\n').collect();

    let old_offset = old_start.saturating_sub(1);
    let new_offset = new_start.saturating_sub(1);

    let old_segment = old_split
        .get(old_offset..old_offset + old_lines)
        .ok_or_else(|| AppError::Git {
            message: "hunk old range out of bounds".into(),
        })?;
    let new_before = new_split.get(..new_offset).unwrap_or(&[]);
    let new_after = new_split.get(new_offset + new_lines..).unwrap_or(&[]);

    let mut result = String::with_capacity(new_content.len());
    for s in new_before { result.push_str(s); }
    for s in old_segment { result.push_str(s); }
    for s in new_after { result.push_str(s); }

    // If the spliced result is empty AND HEAD doesn't have the file, the
    // workdir file is now meaningless — remove it.
    if result.is_empty() && old_content.is_empty() {
        match std::fs::remove_file(&abs) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(AppError::Io { message: e.to_string() }),
        }
        return Ok(());
    }

    std::fs::write(&abs, result).map_err(|e| AppError::Io { message: e.to_string() })?;
    Ok(())
}
