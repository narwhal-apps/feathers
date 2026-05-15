mod common;

use feathers_app_lib::git_core::{conflicts, history, op, repo, status};
use git2::Oid;

fn signature() -> git2::Signature<'static> {
    git2::Signature::now("Test", "test@example.com").expect("sig")
}

fn commit_on(repo: &git2::Repository, refname: &str, file: &str, contents: &str, msg: &str) -> Oid {
    use std::path::Path;
    let abs = repo.workdir().unwrap().join(file);
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent).ok();
    }
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
    repo.commit(Some(refname), &signature(), &signature(), msg, &tree, &parent_refs)
        .unwrap()
}

/// Build a repo with a conflict on `a.txt` from a cherry-pick.
fn repo_with_conflict() -> (tempfile::TempDir, git2::Repository) {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "shared\n")]);
    let r = repo::open(dir.path()).unwrap();
    {
        // Scope so the Commit borrow is released before we return `r`.
        let a = r.head().unwrap().peel_to_commit().unwrap();
        r.branch("feature", &a, false).unwrap();
    }
    let _b = commit_on(&r, "refs/heads/main", "a.txt", "main edit\n", "main edit");
    r.set_head("refs/heads/feature").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
        .unwrap();
    let c = commit_on(&r, "refs/heads/feature", "a.txt", "feat edit\n", "feat edit");
    r.set_head("refs/heads/main").unwrap();
    r.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
        .unwrap();
    history::cherrypick(&r, c).unwrap();
    assert!(matches!(op::state(&r).unwrap().kind, op::OpKind::CherryPick));
    assert!(r.index().unwrap().has_conflicts());
    (dir, r)
}

#[test]
fn auto_resolve_stages_files_without_markers() {
    let (dir, r) = repo_with_conflict();

    // User cleans up the file in their editor — no markers left.
    std::fs::write(dir.path().join("a.txt"), "resolved\n").unwrap();

    let resolved = conflicts::auto_resolve_clean(&r).unwrap();
    assert_eq!(resolved, vec!["a.txt".to_string()]);
    assert!(!r.index().unwrap().has_conflicts());
}

#[test]
fn auto_resolve_skips_files_that_still_have_markers() {
    let (dir, r) = repo_with_conflict();

    // User opened the file but hasn't finished — markers still present.
    std::fs::write(
        dir.path().join("a.txt"),
        "<<<<<<< HEAD\nmain edit\n=======\nfeat edit\n>>>>>>> feature\n",
    )
    .unwrap();

    let resolved = conflicts::auto_resolve_clean(&r).unwrap();
    assert!(resolved.is_empty());
    assert!(r.index().unwrap().has_conflicts());
}

#[test]
fn status_call_auto_resolves_clean_conflicts() {
    // End-to-end: status() should drop the file from the conflicted set
    // once the working tree is clean. This is what the FE relies on.
    let (dir, r) = repo_with_conflict();
    std::fs::write(dir.path().join("a.txt"), "resolved by user\n").unwrap();

    let snap = status::status(&r).unwrap();
    assert!(
        snap.conflicted.is_empty(),
        "expected conflicted set to be empty after auto-resolve, got {:?}",
        snap.conflicted,
    );
    // The file is now staged (because stage_files moved it from conflicted
    // to a single index entry).
    assert!(
        snap.staged.iter().any(|f| f.path == "a.txt"),
        "expected a.txt to appear in the staged set, got staged={:?}",
        snap.staged,
    );
}

#[test]
fn status_call_leaves_unresolved_conflicts_alone() {
    let (dir, r) = repo_with_conflict();
    std::fs::write(
        dir.path().join("a.txt"),
        "<<<<<<< HEAD\nx\n=======\ny\n>>>>>>> feat\n",
    )
    .unwrap();

    let snap = status::status(&r).unwrap();
    assert_eq!(snap.conflicted.len(), 1);
    assert_eq!(snap.conflicted[0].path, "a.txt");
}
