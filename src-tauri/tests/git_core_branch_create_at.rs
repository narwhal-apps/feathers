mod common;

use feathers_app_lib::error::AppError;
use feathers_app_lib::git_core::{branch, repo};
use git2::Oid;

#[test]
fn create_at_makes_branch_at_given_oid_and_checks_out() {
    // Two-commit history; create a branch at commit #1 (the initial).
    let dir = common::fixtures::seeded_repo(&[
        ("a.txt", "first\n"),
        ("b.txt", "second\n"),
    ]);
    let r = repo::open(dir.path()).unwrap();
    let head_commit = r.head().unwrap().peel_to_commit().unwrap();
    let parent = head_commit.parents().next().unwrap();
    let parent_oid = parent.id();

    branch::create_at(&r, "from-initial", parent_oid).unwrap();

    let bs = branch::list_branches(&r).unwrap();
    let head_b = bs.iter().find(|b| b.is_head).expect("a head branch");
    assert_eq!(head_b.name, "from-initial");
    let new_head = r.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(new_head.id(), parent_oid);
}

#[test]
fn create_at_rejects_existing_branch_name() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    let head_oid: Oid = r.head().unwrap().target().unwrap();

    branch::create_at(&r, "feat/x", head_oid).unwrap();
    let err = branch::create_at(&r, "feat/x", head_oid).unwrap_err();
    match err {
        AppError::Git { message } => assert!(message.contains("feat/x")),
        other => panic!("expected Git error, got {other:?}"),
    }
}

#[test]
fn create_at_refuses_when_working_tree_dirty() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x\n")]);
    let r = repo::open(dir.path()).unwrap();
    let head_oid: Oid = r.head().unwrap().target().unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "y\n");

    let err = branch::create_at(&r, "feat/y", head_oid).unwrap_err();
    match err {
        AppError::Dirty { .. } => {}
        other => panic!("expected Dirty, got {other:?}"),
    }
}
