mod common;

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
