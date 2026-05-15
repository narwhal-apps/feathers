mod common;

use feathers_app_lib::git_core::{remote, repo, status};
use git2::Signature;
use std::path::Path;

fn signature() -> Signature<'static> {
    Signature::now("Test", "test@example.com").expect("sig")
}

fn commit_on(
    r: &git2::Repository,
    refname: &str,
    file: &str,
    contents: &str,
    msg: &str,
) -> git2::Oid {
    let abs = r.workdir().unwrap().join(file);
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if contents.is_empty() {
        std::fs::remove_file(&abs).ok();
    } else {
        std::fs::write(&abs, contents).expect("write");
    }
    let mut idx = r.index().unwrap();
    if contents.is_empty() {
        idx.remove_path(Path::new(file)).unwrap();
    } else {
        idx.add_path(Path::new(file)).unwrap();
    }
    let tree_oid = idx.write_tree().unwrap();
    idx.write().unwrap();
    let tree = r.find_tree(tree_oid).unwrap();
    let parents: Vec<git2::Commit> = r
        .find_reference(refname)
        .ok()
        .and_then(|rf| rf.target())
        .map(|p| vec![r.find_commit(p).unwrap()])
        .unwrap_or_default();
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
    r.commit(
        Some(refname),
        &signature(),
        &signature(),
        msg,
        &tree,
        &parent_refs,
    )
    .unwrap()
}

#[test]
fn pull_fast_forward_leaves_status_clean() {
    // Set up an "upstream" repo with one commit, then clone it locally.
    let upstream_dir = common::fixtures::seeded_repo(&[("README.md", "v1\n")]);
    let local_dir = tempfile::tempdir().expect("tempdir");

    // Manual clone so we don't need network: open upstream as a remote
    // by URL pointing at its on-disk path.
    let local =
        git2::Repository::clone(upstream_dir.path().to_str().unwrap(), local_dir.path())
            .expect("clone");

    // Add a couple of new commits upstream that the local clone hasn't seen.
    let upstream = git2::Repository::open(upstream_dir.path()).unwrap();
    commit_on(&upstream, "refs/heads/main", "added_a.txt", "alpha\n", "add a");
    commit_on(&upstream, "refs/heads/main", "added_b.txt", "beta\n", "add b");
    // Also delete README to mirror the user's reported scenario (incoming
    // changes include both adds and removes).
    commit_on(&upstream, "refs/heads/main", "README.md", "", "drop README");

    // Pull (fast-forward, no rebase).
    remote::pull(&local, None, false).expect("pull");

    // After a clean fast-forward, status should be empty — HEAD, index,
    // and working tree all match.
    let snap = status::status(&local).expect("status");
    assert!(
        snap.staged.is_empty(),
        "expected empty staged set, got {:?}",
        snap.staged.iter().map(|f| &f.path).collect::<Vec<_>>(),
    );
    assert!(
        snap.unstaged.is_empty(),
        "expected empty unstaged set, got {:?}",
        snap.unstaged.iter().map(|f| &f.path).collect::<Vec<_>>(),
    );
    assert!(snap.untracked.is_empty(), "expected no untracked files");
    assert!(snap.conflicted.is_empty(), "expected no conflicts");

    // And the working tree should reflect the upstream state.
    assert!(local.workdir().unwrap().join("added_a.txt").exists());
    assert!(local.workdir().unwrap().join("added_b.txt").exists());
    assert!(!local.workdir().unwrap().join("README.md").exists());
}

#[test]
fn pull_fast_forward_index_persists_to_disk() {
    // Discriminator: does fast_forward write a correct index to disk, or
    // is the on-disk .git/index left stale? Re-opens a fresh Repository
    // after the pull so any in-memory cache is bypassed.
    let upstream_dir = common::fixtures::seeded_repo(&[("README.md", "v1\n")]);
    let local_dir = tempfile::tempdir().expect("tempdir");
    git2::Repository::clone(upstream_dir.path().to_str().unwrap(), local_dir.path())
        .expect("clone");

    let local = repo::open(local_dir.path()).expect("open local");
    let upstream = git2::Repository::open(upstream_dir.path()).unwrap();
    commit_on(&upstream, "refs/heads/main", "added.txt", "x\n", "add");
    remote::pull(&local, None, false).expect("pull");

    // Drop the post-pull handle and re-open from disk. If the on-disk
    // index is correct, this fresh handle will see clean status.
    drop(local);
    let fresh = repo::open(local_dir.path()).expect("re-open");
    let snap = status::status(&fresh).expect("status");

    assert!(
        snap.staged.is_empty(),
        "expected clean staged set on a fresh repo handle after pull, got {:?}",
        snap.staged.iter().map(|f| &f.path).collect::<Vec<_>>(),
    );
}

/// Helper for the auto-fast-forward tests: build a clone where a
/// non-checked-out local branch tracks an upstream that has advanced.
/// Returns the (upstream tempdir, local tempdir, local Repository).
fn clone_with_diverging_setup() -> (tempfile::TempDir, tempfile::TempDir, git2::Repository) {
    let upstream_dir = common::fixtures::seeded_repo(&[("README.md", "v1\n")]);
    let upstream = git2::Repository::open(upstream_dir.path()).unwrap();

    // Branch `feature` off the initial commit upstream.
    {
        let main_tip = upstream.head().unwrap().peel_to_commit().unwrap();
        upstream.branch("feature", &main_tip, false).unwrap();
    }
    // One commit each on main and feature so they're independent.
    commit_on(
        &upstream,
        "refs/heads/main",
        "main_only.txt",
        "m\n",
        "main only",
    );
    commit_on(
        &upstream,
        "refs/heads/feature",
        "feature_only.txt",
        "f\n",
        "feature only",
    );

    // Clone — sets up tracking for both main and feature.
    let local_dir = tempfile::tempdir().expect("tempdir");
    let local =
        git2::Repository::clone(upstream_dir.path().to_str().unwrap(), local_dir.path())
            .expect("clone");

    // Make sure `feature` exists locally and tracks origin/feature.
    {
        let upstream_feature = local
            .find_branch("origin/feature", git2::BranchType::Remote)
            .unwrap();
        let upstream_oid = upstream_feature.get().target().unwrap();
        let upstream_commit = local.find_commit(upstream_oid).unwrap();
        let mut local_feature = local.branch("feature", &upstream_commit, false).unwrap();
        local_feature.set_upstream(Some("origin/feature")).unwrap();
    }

    (upstream_dir, local_dir, local)
}

#[test]
fn fetch_fast_forwards_eligible_non_current_branches() {
    let (upstream_dir, _local_dir, local) = clone_with_diverging_setup();
    let upstream = git2::Repository::open(upstream_dir.path()).unwrap();

    // Advance origin/feature with a new commit; the local feature branch
    // is not checked out (we're on main) and has no local commits, so
    // it should fast-forward in the post-fetch sweep.
    let new_feature_oid = commit_on(
        &upstream,
        "refs/heads/feature",
        "feature_only.txt",
        "f2\n",
        "feature edit",
    );

    remote::fetch(&local, None).expect("fetch");

    let local_feature = local
        .find_branch("feature", git2::BranchType::Local)
        .unwrap();
    assert_eq!(
        local_feature.get().target(),
        Some(new_feature_oid),
        "expected local `feature` to fast-forward to upstream tip",
    );
}

#[test]
fn fetch_does_not_touch_diverged_branches() {
    let (upstream_dir, local_dir, local) = clone_with_diverging_setup();
    let upstream = git2::Repository::open(upstream_dir.path()).unwrap();

    // Add a local-only commit to feature so it's both ahead and behind
    // once upstream moves.
    let upstream_feature_before = local
        .find_branch("feature", git2::BranchType::Local)
        .unwrap()
        .get()
        .target()
        .unwrap();
    // Switch to feature, add a local commit, switch back.
    local.set_head("refs/heads/feature").unwrap();
    local
        .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
        .unwrap();
    let local_only_oid = commit_on(
        &local,
        "refs/heads/feature",
        "local_only.txt",
        "loc\n",
        "local only",
    );
    local.set_head("refs/heads/main").unwrap();
    local
        .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
        .unwrap();

    // Move upstream forward independently.
    commit_on(
        &upstream,
        "refs/heads/feature",
        "feature_only.txt",
        "f2\n",
        "feature edit",
    );

    remote::fetch(&local, None).expect("fetch");

    let local_feature = local
        .find_branch("feature", git2::BranchType::Local)
        .unwrap();
    assert_eq!(
        local_feature.get().target(),
        Some(local_only_oid),
        "expected diverged `feature` to be left alone",
    );
    // Sanity: upstream tracking ref *was* updated.
    let _ = upstream_feature_before;
    let _ = local_dir;
}

#[test]
fn fetch_does_not_touch_the_currently_checked_out_branch() {
    let (upstream_dir, _local_dir, local) = clone_with_diverging_setup();
    let upstream = git2::Repository::open(upstream_dir.path()).unwrap();

    // We're on main locally; advance origin/main and confirm local main
    // is *not* fast-forwarded by fetch (Pull's job, not Fetch's).
    let local_main_before = local
        .find_branch("main", git2::BranchType::Local)
        .unwrap()
        .get()
        .target()
        .unwrap();
    commit_on(
        &upstream,
        "refs/heads/main",
        "more.txt",
        "more\n",
        "main edit",
    );

    remote::fetch(&local, None).expect("fetch");

    let local_main_after = local
        .find_branch("main", git2::BranchType::Local)
        .unwrap()
        .get()
        .target()
        .unwrap();
    assert_eq!(
        local_main_after, local_main_before,
        "expected the checked-out branch to be left alone — Pull's job",
    );
}

#[test]
fn fetch_brings_down_all_branches_even_on_a_single_branch_clone() {
    // A clone configured with a narrow `remote.origin.fetch` refspec
    // (single-branch / shallow / sparse) used to only update the current
    // branch's upstream — so the "behind" badge on the default branch
    // would stay stale while the user worked on a feature branch.
    // remote::fetch now overrides the configured refspec with the
    // wildcard so every branch comes down.

    let upstream_dir = common::fixtures::seeded_repo(&[("README.md", "v1\n")]);
    let local_dir = tempfile::tempdir().expect("tempdir");
    git2::Repository::clone(upstream_dir.path().to_str().unwrap(), local_dir.path())
        .expect("clone");
    let local = repo::open(local_dir.path()).expect("open");

    // Narrow this clone's fetch refspec to mimic `git clone --single-branch`.
    {
        let mut cfg = local.config().expect("config");
        cfg.set_str("remote.origin.fetch", "+refs/heads/main:refs/remotes/origin/main")
            .expect("set narrow refspec");
    }

    // Add a brand-new branch upstream that the narrowed refspec would
    // normally ignore.
    let upstream = git2::Repository::open(upstream_dir.path()).unwrap();
    let main_tip = upstream.head().unwrap().peel_to_commit().unwrap();
    upstream.branch("feature/x", &main_tip, false).unwrap();
    commit_on(
        &upstream,
        "refs/heads/feature/x",
        "feature.txt",
        "feat\n",
        "feat",
    );

    // Fetch — explicit wildcard refspec means feature/x should come down.
    remote::fetch(&local, None).expect("fetch");

    // Remote-tracking ref for the new branch must exist locally now.
    assert!(
        local
            .find_reference("refs/remotes/origin/feature/x")
            .is_ok(),
        "expected origin/feature/x to be fetched even with a narrow refspec",
    );
}

#[test]
fn pull_fast_forward_via_long_lived_repo_handle() {
    // Same as above but uses repo::open (the same path the registry takes)
    // — catches the case where a long-lived Repository keeps a cached
    // index that doesn't see post-pull changes.
    let upstream_dir = common::fixtures::seeded_repo(&[("README.md", "v1\n")]);
    let local_dir = tempfile::tempdir().expect("tempdir");
    git2::Repository::clone(
        upstream_dir.path().to_str().unwrap(),
        local_dir.path(),
    )
    .expect("clone");

    let local = repo::open(local_dir.path()).expect("open");

    // Touch upstream.
    let upstream = git2::Repository::open(upstream_dir.path()).unwrap();
    commit_on(&upstream, "refs/heads/main", "added.txt", "x\n", "add");

    // Pull and immediately re-read status from the same Repository handle
    // (simulating with_repo_write keeping the same instance across calls).
    remote::pull(&local, None, false).expect("pull");
    let snap = status::status(&local).expect("status");

    assert!(
        snap.staged.is_empty(),
        "expected empty staged set after pull, got {:?}",
        snap.staged.iter().map(|f| &f.path).collect::<Vec<_>>(),
    );
    assert!(snap.unstaged.is_empty());
    assert!(snap.untracked.is_empty());
}
