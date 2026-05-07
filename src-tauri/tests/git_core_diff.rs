mod common;

use feathers_app_lib::git_core::{diff, repo, types::DiffLineKind};

#[test]
fn diff_workdir_shows_modifications() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "a.txt", "alpha\nbeta\n");
    let payload = diff::diff_workdir(&r, None).unwrap();
    assert_eq!(payload.files.len(), 1);
    assert_eq!(payload.files[0].path, "a.txt");
    let lines: Vec<_> = payload.files[0].hunks.iter().flat_map(|h| h.lines.iter()).collect();
    assert!(lines.iter().any(|l| l.kind == DiffLineKind::Add && l.text.contains("beta")));
}

#[test]
fn diff_commit_shows_changes_vs_parent() {
    let dir = common::fixtures::seeded_repo(&[
        ("a.txt", "alpha\n"),
        ("a.txt", "alpha\nbeta\n"),
    ]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    let payload = diff::diff_commit(&r, &head.id().to_string()).unwrap();
    assert_eq!(payload.files.len(), 1);
    assert_eq!(payload.files[0].path, "a.txt");
    let lines: Vec<_> = payload.files[0].hunks.iter().flat_map(|h| h.lines.iter()).collect();
    assert!(lines.iter().any(|l| l.kind == DiffLineKind::Add && l.text.contains("beta")));
}

#[test]
fn diff_commit_for_initial_commit_shows_all_files_added() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    let head = r.head().unwrap().peel_to_commit().unwrap();
    let payload = diff::diff_commit(&r, &head.id().to_string()).unwrap();
    assert_eq!(payload.files.len(), 1);
    let lines: Vec<_> = payload.files[0].hunks.iter().flat_map(|h| h.lines.iter()).collect();
    assert!(lines.iter().any(|l| l.kind == DiffLineKind::Add && l.text.contains("alpha")));
}
