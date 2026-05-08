mod common;

use feathers_app_lib::error::AppError;
use feathers_app_lib::git_core::{history, op, repo, status};
use feathers_app_lib::git_core::history::ResetMode;

fn signature() -> git2::Signature<'static> {
    git2::Signature::now("Test", "test@example.com").expect("sig")
}

fn commit_on(repo: &git2::Repository, refname: &str, file: &str, contents: &str, msg: &str) -> git2::Oid {
    use std::path::Path;
    let abs = repo.workdir().unwrap().join(file);
    if let Some(parent) = abs.parent() { std::fs::create_dir_all(parent).ok(); }
    std::fs::write(&abs, contents).expect("write");
    let mut idx = repo.index().unwrap();
    idx.add_path(Path::new(file)).unwrap();
    let tree_oid = idx.write_tree().unwrap();
    idx.write().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let parent_oid = repo
        .find_reference(refname)
        .ok()
        .and_then(|r| r.target());
    let parents: Vec<git2::Commit> = parent_oid
        .map(|p| vec![repo.find_commit(p).unwrap()])
        .unwrap_or_default();
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
    repo.commit(Some(refname), &signature(), &signature(), msg, &tree, &parent_refs).unwrap()
}

#[test]
fn reset_soft_moves_head_keeps_changes_staged() {
    let dir = common::fixtures::seeded_repo(&[
        ("a.txt", "1"),
        ("a.txt", "2"),
        ("a.txt", "3"),
    ]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    let grand = head.parent(0).unwrap().parent(0).unwrap();

    history::reset(&r, grand.id(), ResetMode::Soft).unwrap();

    assert_eq!(r.head().unwrap().target().unwrap(), grand.id());
    let snap = status::status(&r).unwrap();
    // Two commits' worth of changes are now in the index, none unstaged.
    assert!(!snap.staged.is_empty());
    assert!(snap.unstaged.is_empty());
}

#[test]
fn reset_mixed_moves_head_unstages_changes() {
    let dir = common::fixtures::seeded_repo(&[
        ("a.txt", "1"),
        ("a.txt", "2"),
    ]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    let parent = head.parent(0).unwrap();

    history::reset(&r, parent.id(), ResetMode::Mixed).unwrap();

    assert_eq!(r.head().unwrap().target().unwrap(), parent.id());
    let snap = status::status(&r).unwrap();
    assert!(snap.staged.is_empty());
    assert!(!snap.unstaged.is_empty());
}

#[test]
fn reset_hard_moves_head_and_discards_changes() {
    let dir = common::fixtures::seeded_repo(&[
        ("a.txt", "1"),
        ("a.txt", "2"),
    ]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    let parent = head.parent(0).unwrap();

    history::reset(&r, parent.id(), ResetMode::Hard).unwrap();

    assert_eq!(r.head().unwrap().target().unwrap(), parent.id());
    let snap = status::status(&r).unwrap();
    assert!(snap.staged.is_empty());
    assert!(snap.unstaged.is_empty());
    let raw = std::fs::read_to_string(dir.path().join("a.txt")).unwrap();
    assert_eq!(raw, "1");
}

#[test]
fn cherrypick_clean_applies_and_commits() {
    // Build:  main: A   feature: A -> B (modifies different file).
    // Cherry-pick B onto main → should commit on main without conflict.
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    let a = r.head().unwrap().peel_to_commit().unwrap();
    r.branch("feature", &a, false).unwrap();
    r.set_head("refs/heads/feature").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();
    let b_oid = commit_on(&r, "refs/heads/feature", "b.txt", "bravo\n", "add b");
    r.set_head("refs/heads/main").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();

    history::cherrypick(&r, b_oid).unwrap();

    // OpState is back to Clean and HEAD has a new commit on main with the same message.
    let st = op::state(&r).unwrap();
    matches!(st.kind, op::OpKind::Clean);
    let new_head = r.head().unwrap().peel_to_commit().unwrap();
    let msg = new_head.message().unwrap_or("");
    assert!(msg.contains("add b"));
    assert!(msg.contains("cherry picked from commit"));
    assert!(msg.contains(&b_oid.to_string()));
}

#[test]
fn cherrypick_conflict_leaves_op_state_cherrypick() {
    // Build: A -> B(main: a.txt="m") and A -> C(feat: a.txt="f"). Cherry-pick C onto main.
    let dir = common::fixtures::seeded_repo(&[("a.txt", "shared\n")]);
    let r = repo::open(dir.path()).unwrap();
    let a = r.head().unwrap().peel_to_commit().unwrap();
    r.branch("feature", &a, false).unwrap();
    let b_oid = commit_on(&r, "refs/heads/main", "a.txt", "main edit\n", "main edit");
    let _ = b_oid;
    r.set_head("refs/heads/feature").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();
    let c_oid = commit_on(&r, "refs/heads/feature", "a.txt", "feat edit\n", "feat edit");
    r.set_head("refs/heads/main").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();

    history::cherrypick(&r, c_oid).unwrap();

    let st = op::state(&r).unwrap();
    matches!(st.kind, op::OpKind::CherryPick);
    assert!(!st.conflicted.is_empty());
}

#[test]
fn revert_clean_creates_inverse_commit() {
    let dir = common::fixtures::seeded_repo(&[
        ("a.txt", "alpha\n"),
        ("a.txt", "alpha and bravo\n"),
    ]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();

    history::revert(&r, head.id()).unwrap();

    let st = op::state(&r).unwrap();
    matches!(st.kind, op::OpKind::Clean);
    let new_head = r.head().unwrap().peel_to_commit().unwrap();
    let msg = new_head.message().unwrap_or("");
    assert!(msg.starts_with("Revert"));
    let raw = std::fs::read_to_string(dir.path().join("a.txt")).unwrap();
    assert_eq!(raw, "alpha\n");
}

#[test]
fn cherrypick_refuses_dirty_tree() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "y");

    let err = history::cherrypick(&r, head.id()).unwrap_err();
    match err {
        AppError::Dirty { .. } => {}
        other => panic!("expected Dirty, got {other:?}"),
    }
}
