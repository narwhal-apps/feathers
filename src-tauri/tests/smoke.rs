// Smoke integration test: confirms the crate exposes app_version_inner
// and that it returns the package version string.

mod common;

#[test]
fn app_version_returns_cargo_pkg_version() {
    let v = feathers_app_lib::commands::app::app_version_inner();
    assert_eq!(v, env!("CARGO_PKG_VERSION"));
}

#[test]
fn fixture_seeded_repo_has_initial_commit() {
    let dir = common::fixtures::seeded_repo(&[("README.md", "hello\n")]);
    let repo = git2::Repository::open(dir.path()).expect("open");
    let head = repo.head().expect("head").peel_to_commit().expect("peel");
    assert_eq!(head.message(), Some("commit: README.md"));
}
