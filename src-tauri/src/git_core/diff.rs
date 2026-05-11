use crate::error::AppError;
use crate::git_core::types::{DiffFile, DiffHunk, DiffLine, DiffLineKind, DiffPayload, FileStatus};
use git2::{Diff, DiffOptions, Oid, Repository};
use std::cell::RefCell;

/// Combined working-tree diff: HEAD → workdir, considering the index. This
/// shows ALL changes for a file regardless of whether they're staged, so the
/// preview pane is the same for staged, unstaged, and partially-staged files.
/// Untracked files (with content) and missing-in-workdir files are included.
pub fn diff_workdir(
    repo: &Repository,
    paths: Option<Vec<String>>,
) -> Result<DiffPayload, AppError> {
    let mut opts = base_opts();
    if let Some(ps) = paths {
        for p in ps {
            opts.pathspec(p);
        }
    }
    let head_tree = repo.head().and_then(|h| h.peel_to_tree()).ok();
    let diff = repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut opts))?;
    diff_to_payload(&diff)
}

pub fn diff_index(repo: &Repository, paths: Option<Vec<String>>) -> Result<DiffPayload, AppError> {
    let mut opts = base_opts();
    if let Some(ps) = paths {
        for p in ps {
            opts.pathspec(p);
        }
    }
    let head_tree = repo.head().and_then(|h| h.peel_to_tree()).ok();
    let diff = repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut opts))?;
    diff_to_payload(&diff)
}

pub fn diff_commit(repo: &Repository, oid_str: &str) -> Result<DiffPayload, AppError> {
    let oid = Oid::from_str(oid_str)?;
    let commit = repo.find_commit(oid)?;
    let new_tree = commit.tree()?;
    let old_tree = if commit.parent_count() == 0 {
        None
    } else {
        Some(commit.parent(0)?.tree()?)
    };
    let mut opts = base_opts();
    let diff = repo.diff_tree_to_tree(old_tree.as_ref(), Some(&new_tree), Some(&mut opts))?;
    diff_to_payload(&diff)
}

fn base_opts() -> DiffOptions {
    let mut o = DiffOptions::new();
    o.context_lines(3)
        .interhunk_lines(0)
        .ignore_submodules(true)
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true)
        .show_binary(false);
    o
}

pub(crate) fn diff_to_payload(diff: &Diff<'_>) -> Result<DiffPayload, AppError> {
    let files: RefCell<Vec<DiffFile>> = RefCell::new(vec![]);
    let current_file: RefCell<Option<DiffFile>> = RefCell::new(None);
    let current_hunk: RefCell<Option<DiffHunk>> = RefCell::new(None);

    diff.foreach(
        // file_cb
        &mut |delta, _| {
            // Flush previous file.
            if let Some(mut f) = current_file.borrow_mut().take() {
                if let Some(h) = current_hunk.borrow_mut().take() {
                    f.hunks.push(h);
                }
                files.borrow_mut().push(f);
            }
            let new_path = delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            let old_path_raw = delta
                .old_file()
                .path()
                .map(|p| p.to_string_lossy().into_owned());
            // Only emit old_path when it actually differs (rename).
            let old_path = old_path_raw.filter(|op| op != &new_path);
            let status = match delta.status() {
                git2::Delta::Added | git2::Delta::Untracked => FileStatus::Added,
                git2::Delta::Deleted => FileStatus::Deleted,
                git2::Delta::Modified => FileStatus::Modified,
                git2::Delta::Renamed | git2::Delta::Copied => FileStatus::Renamed,
                git2::Delta::Typechange => FileStatus::Typechange,
                git2::Delta::Conflicted => FileStatus::Conflicted,
                _ => FileStatus::Modified,
            };
            *current_file.borrow_mut() = Some(DiffFile {
                path: new_path,
                old_path,
                status,
                binary: delta.flags().contains(git2::DiffFlags::BINARY),
                hunks: vec![],
            });
            true
        },
        None,
        // hunk_cb
        Some(&mut |_, hunk| {
            if let Some(h) = current_hunk.borrow_mut().take() {
                if let Some(f) = current_file.borrow_mut().as_mut() {
                    f.hunks.push(h);
                }
            }
            *current_hunk.borrow_mut() = Some(DiffHunk {
                header: String::from_utf8_lossy(hunk.header())
                    .trim_end()
                    .to_string(),
                lines: vec![],
            });
            true
        }),
        // line_cb
        Some(&mut |_, _, line| {
            let kind = match line.origin() {
                '+' => DiffLineKind::Add,
                '-' => DiffLineKind::Del,
                _ => DiffLineKind::Ctx,
            };
            let text = String::from_utf8_lossy(line.content())
                .trim_end_matches('\n')
                .to_string();
            let dl = DiffLine {
                kind,
                old_no: line.old_lineno(),
                new_no: line.new_lineno(),
                text,
            };
            if let Some(h) = current_hunk.borrow_mut().as_mut() {
                h.lines.push(dl);
            }
            true
        }),
    )?;

    // Flush remaining hunk + file.
    if let Some(mut f) = current_file.borrow_mut().take() {
        if let Some(h) = current_hunk.borrow_mut().take() {
            f.hunks.push(h);
        }
        files.borrow_mut().push(f);
    }

    Ok(DiffPayload {
        files: files.into_inner(),
    })
}
