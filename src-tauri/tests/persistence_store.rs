use feathers_app_lib::persistence::store::{AppConfig, ConfigStore, FileStore};

#[test]
fn load_returns_default_when_file_missing() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStore::new(dir.path().join("config.json"));
    let cfg = store.load().unwrap();
    assert_eq!(cfg.schema, AppConfig::current_schema());
    assert!(cfg.known_repos.is_empty());
}

#[test]
fn save_then_load_roundtrips() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStore::new(dir.path().join("config.json"));
    let cfg = AppConfig {
        schema: AppConfig::current_schema(),
        known_repos: vec!["/foo/bar".into(), "/baz".into()],
        settings: Default::default(),
    };
    store.save(&cfg).unwrap();
    let loaded = store.load().unwrap();
    assert_eq!(loaded, cfg);
}
