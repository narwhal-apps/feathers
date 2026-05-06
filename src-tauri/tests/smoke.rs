// Smoke integration test: confirms the crate exposes app_version_inner
// and that it returns the package version string.

#[test]
fn app_version_returns_cargo_pkg_version() {
    let v = feathers_app_lib::commands::app::app_version_inner();
    assert_eq!(v, env!("CARGO_PKG_VERSION"));
}
