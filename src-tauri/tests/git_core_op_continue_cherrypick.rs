mod common;

use feathers_app_lib::git_core::{history, op, repo, stage};
use git2::Oid;

fn signature() -> git2::Signature<'static> {
    git2::Signature::now("Test", "test@example.com").expect("sig")
}

fn commit_on(repo: &git2::Repository, refname: &str, file: &str, contents: &str, msg: &str) -> Oid {
    use std::path::Path;
    let abs = repo.workdir().unwrap().join(file);
    if let Some(parent) = abs.parent() { std::fs::create_dir_all(parent).ok(); }
    std::fs::write(&abs, contents).expect("write");
    let mut idx = repo.index().unwrap();
    idx.add_path(Path::new(file)).unwrap();
    let tree_oid = idx.write_tree().unwrap();
    idx.write().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let parents: Vec<git2::Commit> = repo
        .find_reference(refname)
        .ok()
        .and_then(|r| r.target())
        .map(|p| vec![repo.find_commit(p).unwrap()])
        .unwrap_or_default();
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
    repo.commit(Some(refname), &signature(), &signature(), msg, &tree, &parent_refs).unwrap()
}

#[test]
fn op_continue_finishes_a_resolved_cherrypick() {
    // A; main: B(a=m); feature: C(a=f); cherry-pick C onto main → conflict.
    let dir = common::fixtures::seeded_repo(&[("a.txt", "shared\n")]);
    let r = repo::open(dir.path()).unwrap();
    let a = r.head().unwrap().peel_to_commit().unwrap();
    r.branch("feature", &a, false).unwrap();
    let _b = commit_on(&r, "refs/heads/main", "a.txt", "main edit\n", "main edit");
    r.set_head("refs/heads/feature").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();
    let c = commit_on(&r, "refs/heads/feature", "a.txt", "feat edit\n", "feat edit");
    r.set_head("refs/heads/main").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();

    history::cherrypick(&r, c).unwrap();
    assert!(matches!(op::state(&r).unwrap().kind, op::OpKind::CherryPick));

    // Resolve the conflict: write a merged version + stage it.
    std::fs::write(dir.path().join("a.txt"), "resolved\n").unwrap();
    stage::stage_files(&r, &["a.txt".into()]).unwrap();

    op::op_continue(&r).unwrap();

    let st = op::state(&r).unwrap();
    assert!(matches!(st.kind, op::OpKind::Clean));
    let new_head = r.head().unwrap().peel_to_commit().unwrap();
    let msg = new_head.message().unwrap_or("");
    assert!(msg.contains("feat edit"));
    assert!(msg.contains("cherry picked from commit"));
    assert!(msg.contains(&c.to_string()));
}

#[test]
fn op_continue_finishes_a_resolved_revert() {
    let dir = common::fixtures::seeded_repo(&[
        ("a.txt", "alpha\n"),
        ("a.txt", "beta\n"),
    ]);
    let r = repo::open(dir.path()).unwrap();
    // Make a parallel branch that also touches a.txt to force conflict on revert.
    let head = r.head().unwrap().peel_to_commit().unwrap();
    let parent = head.parent(0).unwrap();
    r.branch("p", &parent, false).unwrap();
    r.set_head("refs/heads/p").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();
    let _ = commit_on(&r, "refs/heads/p", "a.txt", "gamma\n", "gamma");
    // Switch back to main and try to revert the head commit (which set "beta").
    r.set_head("refs/heads/main").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();

    history::revert(&r, head.id()).unwrap();
    // Revert of `beta` on main is clean (no parallel work on main itself), so
    // OpState should be Clean already and message should start with "Revert".
    let st = op::state(&r).unwrap();
    assert!(matches!(st.kind, op::OpKind::Clean));
    let new_head = r.head().unwrap().peel_to_commit().unwrap();
    assert!(new_head.message().unwrap_or("").starts_with("Revert"));
}
