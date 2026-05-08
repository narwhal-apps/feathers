// We test the pure helpers, not the Tauri commands themselves (those wrap
// these helpers and require a Tauri runtime). Sets HOME to a temp dir so
// the global `~/.gitconfig` we write to is sandboxed per-test.

use feathers_app_lib::commands::settings::{
    GitIdentity, read_git_identity_from, write_git_identity_to,
};
use tempfile::TempDir;

fn temp_home() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn returns_none_when_unset() {
    let home = temp_home();
    let id = read_git_identity_from(home.path()).expect("read");
    assert_eq!(id, GitIdentity { name: None, email: None });
}

#[test]
fn round_trips_name_and_email() {
    let home = temp_home();
    write_git_identity_to(home.path(), "Ada Lovelace", "ada@example.com").unwrap();
    let id = read_git_identity_from(home.path()).unwrap();
    assert_eq!(id.name.as_deref(), Some("Ada Lovelace"));
    assert_eq!(id.email.as_deref(), Some("ada@example.com"));
}

#[test]
fn empty_strings_unset_keys() {
    let home = temp_home();
    write_git_identity_to(home.path(), "Ada", "ada@example.com").unwrap();
    write_git_identity_to(home.path(), "", "").unwrap();
    let id = read_git_identity_from(home.path()).unwrap();
    assert_eq!(id, GitIdentity { name: None, email: None });
}

#[test]
fn rejects_half_set_pair() {
    let home = temp_home();
    let err = write_git_identity_to(home.path(), "Ada", "").unwrap_err();
    assert!(format!("{err:?}").contains("both"));
}

#[test]
fn rejects_invalid_email() {
    let home = temp_home();
    let err = write_git_identity_to(home.path(), "Ada", "no-at-sign").unwrap_err();
    assert!(format!("{err:?}").contains("email"));
}
