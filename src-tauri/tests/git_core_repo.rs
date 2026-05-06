mod common;

use feathers_app_lib::git_core::repo;

#[test]
fn open_returns_repo_for_a_valid_path() {
    let dir = common::fixtures::seeded_repo(&[("README.md", "hi\n")]);
    let r = repo::open(dir.path()).expect("open");
    assert!(r.path().exists());
}

#[test]
fn open_errors_for_a_non_repo() {
    let dir = tempfile::tempdir().unwrap();
    let r = repo::open(dir.path());
    assert!(r.is_err());
}

#[test]
fn head_info_returns_main_after_initial_commit() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    let h = repo::head_info(&r).unwrap();
    assert_eq!(h.branch, "main");
    assert!(!h.detached);
    assert_eq!(h.short_sha.len(), 7);
}
