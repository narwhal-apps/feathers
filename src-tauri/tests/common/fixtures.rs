use git2::{Repository, Signature};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Build a tempdir-backed Git repo with the given commits.
/// Each commit is a (filename, contents) pair on `main`.
/// The first commit becomes the repo's initial commit.
pub fn seeded_repo(commits: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("init");

    // Force the default branch to "main" so tests are deterministic across hosts.
    repo.set_head("refs/heads/main").ok();

    let sig = Signature::now("Test", "test@example.com").expect("sig");
    let mut parents: Vec<git2::Oid> = vec![];

    for (path, contents) in commits {
        let abs = dir.path().join(path);
        if let Some(parent) = abs.parent() { fs::create_dir_all(parent).ok(); }
        fs::write(&abs, contents).expect("write file");

        let mut index = repo.index().expect("index");
        index.add_path(Path::new(path)).expect("add_path");
        let tree_oid = index.write_tree().expect("write_tree");
        index.write().expect("write index");
        let tree = repo.find_tree(tree_oid).expect("find_tree");

        let parent_commits: Vec<_> = parents.iter()
            .map(|oid| repo.find_commit(*oid).expect("find_commit"))
            .collect();
        let parent_refs: Vec<&git2::Commit> = parent_commits.iter().collect();

        let oid = repo
            .commit(Some("HEAD"), &sig, &sig, &format!("commit: {path}"), &tree, &parent_refs)
            .expect("commit");
        parents = vec![oid];
    }

    dir
}

/// Write a file in an existing repo without staging or committing it.
#[allow(dead_code)]
pub fn write_file(repo_dir: &Path, rel: &str, contents: &str) {
    let abs = repo_dir.join(rel);
    if let Some(parent) = abs.parent() { fs::create_dir_all(parent).ok(); }
    fs::write(&abs, contents).expect("write file");
}

/// Stage a path in an existing repo.
#[allow(dead_code)]
pub fn stage(repo: &Repository, rel: &str) {
    let mut index = repo.index().expect("index");
    index.add_path(Path::new(rel)).expect("add_path");
    index.write().expect("write index");
}
