mod common;

use feathers_app_lib::git_core::{repo, status, types::FileStatus};

#[test]
fn status_finds_untracked_modified_and_staged_files() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n"), ("b.txt", "beta\n")]);
    let r = repo::open(dir.path()).unwrap();

    // Modify a tracked file (unstaged)
    common::fixtures::write_file(dir.path(), "a.txt", "alpha changed\n");

    // Add an untracked file
    common::fixtures::write_file(dir.path(), "c.txt", "charlie\n");

    // Stage a tracked file (b.txt edited then staged)
    common::fixtures::write_file(dir.path(), "b.txt", "beta v2\n");
    common::fixtures::stage(&r, "b.txt");

    let snap = status::status(&r).unwrap();

    assert!(snap.staged.iter().any(|f| f.path == "b.txt" && f.status == FileStatus::Modified));
    assert!(snap.unstaged.iter().any(|f| f.path == "a.txt" && f.status == FileStatus::Modified));
    assert!(snap.untracked.iter().any(|f| f.path == "c.txt" && f.status == FileStatus::Untracked));
}

#[test]
fn status_is_empty_for_clean_repo() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    let snap = status::status(&r).unwrap();
    assert!(snap.staged.is_empty());
    assert!(snap.unstaged.is_empty());
    assert!(snap.untracked.is_empty());
    assert!(snap.conflicted.is_empty());
}
