use feathers_app_lib::persistence::store::{
    AppConfig, AppSettings, ConfigStore, FileStore, ThemeName,
};
use tempfile::tempdir;

#[test]
fn loads_default_when_file_absent() {
    let dir = tempdir().unwrap();
    let store = FileStore::new(dir.path().join("config.json"));
    let cfg = store.load().expect("load");
    assert_eq!(cfg.schema, AppConfig::current_schema());
    assert!(cfg.known_repos.is_empty());
    assert_eq!(cfg.settings, AppSettings::default());
}

#[test]
fn migrates_schema_1_to_2() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.json");
    // Write a schema-1 file by hand (no `settings` field).
    std::fs::write(
        &path,
        r#"{"schema":1,"known_repos":["/tmp/foo"]}"#,
    ).unwrap();
    let store = FileStore::new(path.clone());
    let cfg = store.load().expect("load");
    assert_eq!(cfg.schema, 2);
    assert_eq!(cfg.known_repos, vec!["/tmp/foo".to_string()]);
    assert_eq!(cfg.settings, AppSettings::default());
    // The migration should have rewritten the file with the new schema.
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains("\"schema\": 2"));
    assert!(raw.contains("\"settings\""));
}

#[test]
fn round_trips_theme_override() {
    let dir = tempdir().unwrap();
    let store = FileStore::new(dir.path().join("config.json"));
    let mut cfg = store.load().unwrap();
    cfg.settings.theme_override = Some(ThemeName::Light);
    store.save(&cfg).unwrap();
    let reloaded = store.load().unwrap();
    assert_eq!(reloaded.settings.theme_override, Some(ThemeName::Light));
}

#[test]
fn round_trips_last_active_repo_path() {
    let dir = tempdir().unwrap();
    let store = FileStore::new(dir.path().join("config.json"));
    let mut cfg = store.load().unwrap();
    cfg.settings.last_active_repo_path = Some("/Users/me/code/proj".to_string());
    store.save(&cfg).unwrap();
    let reloaded = store.load().unwrap();
    assert_eq!(
        reloaded.settings.last_active_repo_path,
        Some("/Users/me/code/proj".to_string()),
    );

    // Clearing roundtrips too.
    let mut cfg = reloaded;
    cfg.settings.last_active_repo_path = None;
    store.save(&cfg).unwrap();
    assert_eq!(store.load().unwrap().settings.last_active_repo_path, None);
}

#[test]
fn loads_pre_existing_config_without_last_active_repo_path_field() {
    // Configs persisted before last_active_repo_path existed must still
    // load — covered by `#[serde(default)]` on the field. Includes the
    // case where the older `last_active_repo_id` field is present (its
    // value is silently ignored — registry ids weren't stable anyway).
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(
        &path,
        r#"{"schema":2,"known_repos":[],"settings":{"theme_override":"dark","last_active_repo_id":"stale-uuid"}}"#,
    )
    .unwrap();
    let store = FileStore::new(path);
    let cfg = store.load().expect("load");
    assert_eq!(cfg.settings.theme_override, Some(ThemeName::Dark));
    assert_eq!(cfg.settings.last_active_repo_path, None);
}
