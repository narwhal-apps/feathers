mod common;

use feathers_app_lib::git_core::{commit, commit::CommitOpts, repo, stage, types::LogOpts};

#[test]
fn log_returns_commits_in_reverse_chronological_order() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "1"), ("b.txt", "2"), ("c.txt", "3")]);
    let r = repo::open(dir.path()).unwrap();
    let page = commit::log(
        &r,
        LogOpts {
            max: 10,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page.commits.len(), 3);
    assert!(page.commits[0].summary.contains("c.txt"));
    assert!(page.commits[2].summary.contains("a.txt"));
    assert!(page.next_cursor.is_none());
}

#[test]
fn log_pagination_emits_next_cursor() {
    let pairs: Vec<(String, String)> = (1..=10)
        .map(|i| (format!("f{i}.txt"), format!("v{i}")))
        .collect();
    let pair_refs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();
    let dir = common::fixtures::seeded_repo(&pair_refs);
    let r = repo::open(dir.path()).unwrap();
    let page1 = commit::log(
        &r,
        LogOpts {
            max: 5,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page1.commits.len(), 5);
    assert!(page1.next_cursor.is_some());
    let page2 = commit::log(
        &r,
        LogOpts {
            max: 5,
            before_oid: page1.next_cursor.clone(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page2.commits.len(), 5);
    // No overlap between pages.
    assert!(page2
        .commits
        .iter()
        .all(|c| !page1.commits.iter().any(|p| p.oid == c.oid)));
}

#[test]
fn create_commits_staged_changes() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();

    common::fixtures::write_file(dir.path(), "b.txt", "beta\n");
    stage::stage_files(&r, &["b.txt".to_string()]).unwrap();

    let oid = commit::create(&r, "add b.txt", CommitOpts::default()).unwrap();
    assert_eq!(oid.len(), 40);

    let head_commit = r.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head_commit.id().to_string(), oid);
    assert_eq!(head_commit.summary(), Some("add b.txt"));

    let log = commit::log(
        &r,
        LogOpts {
            max: 10,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(log.commits.len(), 2);
}

#[test]
fn create_rejects_empty_message() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let r = repo::open(dir.path()).unwrap();
    common::fixtures::write_file(dir.path(), "b.txt", "y");
    stage::stage_files(&r, &["b.txt".to_string()]).unwrap();

    let err = commit::create(&r, "   ", CommitOpts::default()).unwrap_err();
    assert!(matches!(
        err,
        feathers_app_lib::error::AppError::Git { .. }
    ));
}

#[test]
fn create_amend_replaces_head() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "alpha\n")]);
    let r = repo::open(dir.path()).unwrap();
    let original = r.head().unwrap().peel_to_commit().unwrap();

    common::fixtures::write_file(dir.path(), "a.txt", "alpha v2\n");
    stage::stage_files(&r, &["a.txt".to_string()]).unwrap();

    let new_oid = commit::create(&r, "amended", CommitOpts { amend: true }).unwrap();
    let head_commit = r.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head_commit.id().to_string(), new_oid);
    assert_eq!(head_commit.summary(), Some("amended"));
    // Same parents as the original (here, none — initial commit).
    assert_eq!(head_commit.parent_count(), original.parent_count());
}
