mod common;

use feathers_app_lib::error::AppError;
use feathers_app_lib::git_core::{branch, repo};

#[test]
fn list_branches_returns_main_as_head() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    let bs = branch::list_branches(&r).unwrap();
    let main = bs.iter().find(|b| b.name == "main").expect("main present");
    assert!(main.is_head);
    assert!(!main.is_remote);
    assert!(main.ahead.is_none()); // no upstream
    assert!(main.behind.is_none());
    assert_eq!(main.short_sha.len(), 7);
}

#[test]
fn list_branches_includes_a_second_local_branch() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    r.branch("feature/x", &head, false).unwrap();
    let bs = branch::list_branches(&r).unwrap();
    assert!(bs.iter().any(|b| b.name == "feature/x" && !b.is_head));
}

#[test]
fn checkout_switches_head_to_the_named_branch() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    r.branch("feature/x", &head, false).unwrap();

    branch::checkout(&r, "feature/x").unwrap();

    let bs = branch::list_branches(&r).unwrap();
    let head_b = bs.iter().find(|b| b.is_head).expect("a head branch");
    assert_eq!(head_b.name, "feature/x");
}

#[test]
fn checkout_errors_when_unknown_branch() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    let err = branch::checkout(&r, "nope").unwrap_err();
    match err {
        AppError::Git { message } => assert!(message.contains("nope")),
        other => panic!("expected Git error, got {other:?}"),
    }
}

#[test]
fn checkout_errors_dirty_when_working_tree_modified() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    r.branch("feature/x", &head, false).unwrap();

    // Modify a tracked file (unstaged) — checkout must refuse.
    common::fixtures::write_file(dir.path(), "a.txt", "alpha changed\n");

    let err = branch::checkout(&r, "feature/x").unwrap_err();
    match err {
        AppError::Dirty { paths } => assert!(paths.iter().any(|p| p == "a.txt")),
        other => panic!("expected Dirty error, got {other:?}"),
    }
}

#[test]
fn checkout_allows_switching_with_only_untracked_files() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    r.branch("feature/x", &head, false).unwrap();

    common::fixtures::write_file(dir.path(), "untracked.txt", "u\n");

    branch::checkout(&r, "feature/x").unwrap();
    let bs = branch::list_branches(&r).unwrap();
    let head_b = bs.iter().find(|b| b.is_head).unwrap();
    assert_eq!(head_b.name, "feature/x");
}
