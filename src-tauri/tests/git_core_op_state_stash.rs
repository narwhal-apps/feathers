mod common;

use feathers_app_lib::git_core::{op, repo, stash};

#[test]
fn state_returns_clean_when_no_sidecar() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();

    let st = op::state(&r).unwrap();
    assert!(matches!(st.kind, op::OpKind::Clean));
}

#[test]
fn state_returns_stash_apply_when_sidecar_exists_without_conflicts() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();

    // Manually plant a sidecar mid-state (simulating an interrupted apply).
    stash::write_sidecar_for_test(&r, 0, false, "abcdef0123456789").unwrap();

    let st = op::state(&r).unwrap();
    match st.kind {
        op::OpKind::StashApply { was_pop, conflicts_present } => {
            assert!(!was_pop);
            assert!(!conflicts_present);
        }
        other => panic!("expected StashApply, got {other:?}"),
    }
}

#[test]
fn state_real_repository_op_wins_over_sidecar() {
    use git2::Repository;
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();

    // Plant sidecar.
    stash::write_sidecar_for_test(&r, 0, false, "abcdef0123456789").unwrap();

    // Force a Merge state by writing MERGE_HEAD pointing at HEAD itself.
    let head_oid = r.head().unwrap().target().unwrap();
    std::fs::write(
        r.path().join("MERGE_HEAD"),
        format!("{}\n", head_oid),
    )
    .unwrap();
    // Re-open so libgit2 picks up the new RepositoryState.
    let r2 = Repository::open(dir.path()).unwrap();

    let st = op::state(&r2).unwrap();
    assert!(matches!(st.kind, op::OpKind::Merge));
}
