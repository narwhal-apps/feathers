mod common;

use feathers_app_lib::git_core::{repo, stage, status, types::FileStatus};

#[test]
fn stage_files_promotes_unstaged_modification_to_staged() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "alpha v2\n");

    stage::stage_files(&r, &["a.txt".to_string()]).unwrap();

    let snap = status::status(&r).unwrap();
    assert!(snap.staged.iter().any(|f| f.path == "a.txt" && f.status == FileStatus::Modified));
    assert!(snap.unstaged.iter().all(|f| f.path != "a.txt"));
}

#[test]
fn stage_files_promotes_untracked_to_added() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "new.txt", "hello\n");

    stage::stage_files(&r, &["new.txt".to_string()]).unwrap();

    let snap = status::status(&r).unwrap();
    assert!(snap.staged.iter().any(|f| f.path == "new.txt" && f.status == FileStatus::Added));
    assert!(snap.untracked.iter().all(|f| f.path != "new.txt"));
}

#[test]
fn stage_files_records_a_deletion() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    std::fs::remove_file(dir.path().join("a.txt")).unwrap();

    stage::stage_files(&r, &["a.txt".to_string()]).unwrap();

    let snap = status::status(&r).unwrap();
    assert!(snap.staged.iter().any(|f| f.path == "a.txt" && f.status == FileStatus::Deleted));
    assert!(snap.unstaged.iter().all(|f| f.path != "a.txt"));
}

#[test]
fn unstage_files_returns_a_modification_to_unstaged() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "alpha v2\n");
    stage::stage_files(&r, &["a.txt".to_string()]).unwrap();

    stage::unstage_files(&r, &["a.txt".to_string()]).unwrap();

    let snap = status::status(&r).unwrap();
    assert!(snap.unstaged.iter().any(|f| f.path == "a.txt" && f.status == FileStatus::Modified));
    assert!(snap.staged.iter().all(|f| f.path != "a.txt"));
}

#[test]
fn unstage_files_returns_an_added_file_to_untracked() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "new.txt", "hello\n");
    stage::stage_files(&r, &["new.txt".to_string()]).unwrap();

    stage::unstage_files(&r, &["new.txt".to_string()]).unwrap();

    let snap = status::status(&r).unwrap();
    assert!(snap.untracked.iter().any(|f| f.path == "new.txt" && f.status == FileStatus::Untracked));
    assert!(snap.staged.iter().all(|f| f.path != "new.txt"));
}
