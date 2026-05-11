mod common;

use feathers_app_lib::error::AppError;
use feathers_app_lib::git_core::{op, repo, stage, stash};

fn induce_apply_conflict(dir_path: &std::path::Path, was_pop: bool) -> git2::Repository {
    let mut r = repo::open(dir_path).unwrap();
    common::fixtures::write_file(dir_path, "a.txt", "stashed edit\n");
    stash::create(&mut r, None, false, false).unwrap();
    common::fixtures::write_file(dir_path, "a.txt", "main edit\n");
    common::fixtures::stage(&r, "a.txt");
    {
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let head = r.head().unwrap().peel_to_commit().unwrap();
        let mut idx = r.index().unwrap();
        let tree = r.find_tree(idx.write_tree().unwrap()).unwrap();
        r.commit(Some("HEAD"), &sig, &sig, "main edit", &tree, &[&head]).unwrap();
    }
    if was_pop {
        stash::pop(&mut r, 0).unwrap();
    } else {
        stash::apply(&mut r, 0).unwrap();
    }
    r
}

#[test]
fn op_continue_finishes_a_resolved_apply_keeping_the_stash() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = induce_apply_conflict(dir.path(), false);

    // Resolve: write a merged version + stage it.
    std::fs::write(dir.path().join("a.txt"), "resolved\n").unwrap();
    stage::stage_files(&r, &["a.txt".to_string()]).unwrap();

    op::op_continue(&r).unwrap();

    let st = op::state(&r).unwrap();
    assert!(matches!(st.kind, op::OpKind::Clean));
    // Stash kept (apply, not pop).
    assert_eq!(stash::list(&mut r).unwrap().len(), 1);
}

#[test]
fn op_continue_finishes_a_resolved_pop_dropping_the_stash() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = induce_apply_conflict(dir.path(), true);

    std::fs::write(dir.path().join("a.txt"), "resolved\n").unwrap();
    stage::stage_files(&r, &["a.txt".to_string()]).unwrap();

    op::op_continue(&r).unwrap();

    let st = op::state(&r).unwrap();
    assert!(matches!(st.kind, op::OpKind::Clean));
    // Stash dropped (pop semantics).
    assert!(stash::list(&mut r).unwrap().is_empty());
}

#[test]
fn op_continue_with_conflicts_remaining_returns_error() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = induce_apply_conflict(dir.path(), false);

    // Don't resolve. Just call continue.
    let err = op::op_continue(&r).unwrap_err();
    match err {
        AppError::MergeConflict { .. } => {}
        other => panic!("expected MergeConflict, got {other:?}"),
    }
}

#[test]
fn op_continue_with_mismatched_stash_oid_errors_and_clears_sidecar() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = induce_apply_conflict(dir.path(), true); // pop variant

    std::fs::write(dir.path().join("a.txt"), "resolved\n").unwrap();
    stage::stage_files(&r, &["a.txt".to_string()]).unwrap();

    // Tamper the sidecar to contain a fake stash_oid.
    stash::write_sidecar_for_test(&r, 0, true, "0000000000000000000000000000000000000000").unwrap();

    let err = op::op_continue(&r).unwrap_err();
    match err {
        AppError::Git { message } => assert!(message.contains("no longer at expected position")),
        other => panic!("expected Git error, got {other:?}"),
    }
    // Sidecar deleted, state Clean again.
    let st = op::state(&r).unwrap();
    assert!(matches!(st.kind, op::OpKind::Clean));
    // Stash itself untouched.
    assert_eq!(stash::list(&mut r).unwrap().len(), 1);
}

#[test]
fn op_abort_with_stash_sidecar_clears_resolution_and_keeps_stash() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = induce_apply_conflict(dir.path(), true);

    op::op_abort(&r).unwrap();

    let st = op::state(&r).unwrap();
    assert!(matches!(st.kind, op::OpKind::Clean));
    // Stash kept — user can retry.
    assert_eq!(stash::list(&mut r).unwrap().len(), 1);
}

#[test]
fn op_continue_recovery_path_with_no_conflicts_just_clears_sidecar_for_apply() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    let mut rw = repo::open(dir.path()).unwrap();

    // Make a real stash so the oid is valid.
    common::fixtures::write_file(dir.path(), "a.txt", "v2\n");
    stash::create(&mut rw, None, false, false).unwrap();
    let entries = stash::list(&mut rw).unwrap();
    let oid = entries[0].oid.clone();

    // Plant a recovery sidecar (no conflicts in index, was_pop=false).
    stash::write_sidecar_for_test(&r, 0, false, &oid).unwrap();

    op::op_continue(&r).unwrap();

    let st = op::state(&r).unwrap();
    assert!(matches!(st.kind, op::OpKind::Clean));
    // Stash kept (apply mode).
    assert_eq!(stash::list(&mut rw).unwrap().len(), 1);
}

#[test]
fn op_continue_recovery_path_with_no_conflicts_drops_stash_for_pop() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    let mut rw = repo::open(dir.path()).unwrap();

    // Make a real stash so the oid is valid.
    common::fixtures::write_file(dir.path(), "a.txt", "v2\n");
    stash::create(&mut rw, None, false, false).unwrap();
    let entries = stash::list(&mut rw).unwrap();
    let oid = entries[0].oid.clone();

    // Plant a recovery sidecar (no conflicts in index, was_pop=true).
    stash::write_sidecar_for_test(&r, 0, true, &oid).unwrap();

    op::op_continue(&r).unwrap();

    let st = op::state(&r).unwrap();
    assert!(matches!(st.kind, op::OpKind::Clean));
    // Stash dropped (pop semantics).
    assert!(stash::list(&mut rw).unwrap().is_empty());
}
