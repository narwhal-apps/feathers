use crate::error::AppError;
use crate::git_core::conflicts;
use crate::git_core::types::{FileChange, FileStatus, StatusSnapshot};
use git2::{Repository, Status, StatusOptions};

pub fn status(repo: &Repository) -> Result<StatusSnapshot, AppError> {
    // Mirror GitHub Desktop: if the user has cleaned a conflicted file in
    // their editor, auto-stage it before computing the snapshot so the UI
    // surfaces the resolution without a manual click. Best-effort — any
    // failure here just leaves the file in the conflicted set, where the
    // existing manual "Resolved" button still works.
    let _ = conflicts::auto_resolve_clean(repo);

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut snap = StatusSnapshot {
        staged: vec![],
        unstaged: vec![],
        untracked: vec![],
        conflicted: vec![],
    };

    for entry in statuses.iter() {
        let s = entry.status();
        let path = entry.path().unwrap_or("").to_string();
        if path.is_empty() {
            continue;
        }

        if s.contains(Status::CONFLICTED) {
            snap.conflicted.push(FileChange {
                path,
                old_path: None,
                status: FileStatus::Conflicted,
            });
            continue;
        }
        if s.contains(Status::WT_NEW) && !s.contains(Status::INDEX_NEW) {
            snap.untracked.push(FileChange {
                path,
                old_path: None,
                status: FileStatus::Untracked,
            });
            continue;
        }

        // Staged side (index vs HEAD).
        if let Some(kind) = staged_kind(s) {
            snap.staged.push(FileChange {
                path: path.clone(),
                old_path: None,
                status: kind,
            });
        }
        // Unstaged side (workdir vs index).
        if let Some(kind) = unstaged_kind(s) {
            snap.unstaged.push(FileChange {
                path,
                old_path: None,
                status: kind,
            });
        }
    }

    Ok(snap)
}

fn staged_kind(s: Status) -> Option<FileStatus> {
    if s.contains(Status::INDEX_NEW) {
        Some(FileStatus::Added)
    } else if s.contains(Status::INDEX_MODIFIED) {
        Some(FileStatus::Modified)
    } else if s.contains(Status::INDEX_DELETED) {
        Some(FileStatus::Deleted)
    } else if s.contains(Status::INDEX_RENAMED) {
        Some(FileStatus::Renamed)
    } else if s.contains(Status::INDEX_TYPECHANGE) {
        Some(FileStatus::Typechange)
    } else {
        None
    }
}

fn unstaged_kind(s: Status) -> Option<FileStatus> {
    if s.contains(Status::WT_MODIFIED) {
        Some(FileStatus::Modified)
    } else if s.contains(Status::WT_DELETED) {
        Some(FileStatus::Deleted)
    } else if s.contains(Status::WT_RENAMED) {
        Some(FileStatus::Renamed)
    } else if s.contains(Status::WT_TYPECHANGE) {
        Some(FileStatus::Typechange)
    } else {
        None
    }
}
