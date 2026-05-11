mod common;

use feathers_app_lib::error::AppError;
use feathers_app_lib::git_core::{repo, stash};

#[test]
fn list_returns_empty_for_unstashed_repo() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();

    let entries = stash::list(&mut r).unwrap();
    assert!(entries.is_empty());
}

#[test]
fn create_with_no_message_uses_wip_auto_message() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "alpha edited\n");

    stash::create(&mut r, None, false, false).unwrap();

    let entries = stash::list(&mut r).unwrap();
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e.index, 0);
    assert!(e.message.starts_with("WIP on main:"));
    assert_eq!(e.branch, "main");
    assert!(!e.oid.is_empty());
    assert!(!e.short_oid.is_empty());
    assert!(e.time > 0);
}

#[test]
fn create_with_user_message_stores_it_verbatim() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "alpha edited\n");

    stash::create(&mut r, Some("WIP: dropdown refactor"), false, false).unwrap();

    let entries = stash::list(&mut r).unwrap();
    assert_eq!(entries.len(), 1);
    // libgit2 prefixes the user message with "On <branch>: " — message field
    // includes that prefix.
    assert!(entries[0].message.contains("WIP: dropdown refactor"));
    assert_eq!(entries[0].branch, "main");
}

#[test]
fn create_with_include_untracked_captures_new_files() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "newfile.txt", "brand new\n");

    stash::create(&mut r, None, true, false).unwrap();

    // Untracked file should be gone from working tree (now in stash).
    assert!(!dir.path().join("newfile.txt").exists());
    let entries = stash::list(&mut r).unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn create_without_include_untracked_leaves_new_files_alone() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "alpha edited\n");
    common::fixtures::write_file(dir.path(), "newfile.txt", "brand new\n");

    stash::create(&mut r, None, false, false).unwrap();

    // newfile.txt is still untracked; only the tracked edit was stashed.
    assert!(dir.path().join("newfile.txt").exists());
    let raw = std::fs::read_to_string(dir.path().join("a.txt")).unwrap();
    assert_eq!(raw, "alpha\n");
}

#[test]
fn create_with_keep_index_keeps_staged_files_staged() {
    use feathers_app_lib::git_core::status;

    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "alpha edited\n");
    common::fixtures::write_file(dir.path(), "b.txt", "beta unstaged\n");
    common::fixtures::stage(&r, "a.txt");

    stash::create(&mut r, None, false, true).unwrap();

    let snap = status::status(&r).unwrap();
    // a.txt was staged before stash; with keep_index, it stays staged.
    assert!(snap.staged.iter().any(|f| f.path == "a.txt"));
    // b.txt was unstaged and got stashed away.
    assert!(snap.unstaged.iter().all(|f| f.path != "b.txt"));
}

#[test]
fn create_with_no_changes_returns_nothing_to_stash() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();

    let err = stash::create(&mut r, None, false, false).unwrap_err();
    match err {
        AppError::Git { message } => assert!(message.contains("nothing to stash")),
        other => panic!("expected Git error, got {other:?}"),
    }
}

#[test]
fn list_returns_newest_first_with_index_zero() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();

    common::fixtures::write_file(dir.path(), "a.txt", "v2\n");
    stash::create(&mut r, Some("first stash"), false, false).unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "v3\n");
    stash::create(&mut r, Some("second stash"), false, false).unwrap();

    let entries = stash::list(&mut r).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].index, 0);
    assert!(entries[0].message.contains("second stash"));
    assert_eq!(entries[1].index, 1);
    assert!(entries[1].message.contains("first stash"));
}

#[test]
fn drop_at_removes_stash_at_given_index() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();

    common::fixtures::write_file(dir.path(), "a.txt", "v2\n");
    stash::create(&mut r, Some("first"), false, false).unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "v3\n");
    stash::create(&mut r, Some("second"), false, false).unwrap();

    stash::drop_at(&mut r, 0).unwrap();

    let entries = stash::list(&mut r).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].message.contains("first"));
}

#[test]
fn drop_at_out_of_range_returns_git_error() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let mut r = repo::open(dir.path()).unwrap();

    let err = stash::drop_at(&mut r, 5).unwrap_err();
    match err {
        AppError::Git { message } => assert!(message.contains("no stash at index")),
        other => panic!("expected Git error, got {other:?}"),
    }
}
