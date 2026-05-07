mod common;

use feathers_app_lib::git_core::{commit, repo, types::LogOpts};

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
