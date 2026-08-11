use std::fs;

use cleaner_store::Store;
use tempfile::TempDir;

#[test]
fn migration_is_copy_only_sanitized_and_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let legacy = TempDir::new()?;
    let settings_path = legacy.path().join("settings.json");
    let history_path = legacy.path().join("cleanup_history.json");
    fs::write(
        &settings_path,
        r#"{
          "theme": "system",
          "dashboard_status_interval_seconds": 15,
          "auto_clean_interval_seconds": 300,
          "ram_purge_interval_seconds": 60
        }"#,
    )?;
    fs::write(
        &history_path,
        r#"[{
          "timestamp": "2026-04-01T12:00:00",
          "items_cleaned": 4,
          "total_cleaned": "1.2 GB",
          "total_cleaned_bytes": 1200000000,
          "details": [{"name": "not imported", "path": "/secret"}]
        }]"#,
    )?;
    let store = Store::open_in_memory()?;

    let summary = store.import_legacy(legacy.path())?;
    assert_eq!(
        summary.imported_settings,
        ["monitor_interval_seconds", "theme"]
    );
    assert_eq!(summary.imported_history_entries, 1);
    assert!(
        summary
            .ignored_settings
            .contains(&"auto_clean_interval_seconds".to_owned())
    );
    assert!(settings_path.exists());
    assert!(history_path.exists());
    assert_eq!(store.legacy_history(10)?.len(), 1);

    let repeated = store.import_legacy(legacy.path())?;
    assert!(repeated.already_imported);
    assert_eq!(store.legacy_history(10)?.len(), 1);
    Ok(())
}
