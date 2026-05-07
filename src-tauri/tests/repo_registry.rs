mod common;

use feathers_app_lib::repo_registry::RepoRegistry;

#[test]
fn add_returns_a_repo_id_and_get_returns_the_handle() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let reg = RepoRegistry::new();
    let id = reg.add(dir.path().to_path_buf()).unwrap();
    let handle = reg.get(&id).unwrap();
    assert_eq!(handle.path, dir.path().canonicalize().unwrap());
}

#[test]
fn add_twice_for_same_path_returns_same_id() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let reg = RepoRegistry::new();
    let id1 = reg.add(dir.path().to_path_buf()).unwrap();
    let id2 = reg.add(dir.path().to_path_buf()).unwrap();
    assert_eq!(id1, id2);
    assert_eq!(reg.list().len(), 1);
}

#[test]
fn add_rejects_a_non_repo_path() {
    let dir = tempfile::tempdir().unwrap();
    let reg = RepoRegistry::new();
    assert!(reg.add(dir.path().to_path_buf()).is_err());
}

#[test]
fn remove_drops_the_handle() {
    let dir = common::fixtures::seeded_repo(&[("a.txt", "x")]);
    let reg = RepoRegistry::new();
    let id = reg.add(dir.path().to_path_buf()).unwrap();
    reg.remove(&id).unwrap();
    assert!(reg.get(&id).is_err());
}
