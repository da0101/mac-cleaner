use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use cleaner_domain::{ExecutionMode, QuarantineAction, QuarantineState};
use cleaner_service::AppService;
use filetime::{FileTime, set_file_mtime};
use tempfile::TempDir;

fn create_old_cache(home: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let candidate = home.join("Library/Caches/com.da0101.mac-cleaner/expired");
    fs::create_dir_all(&candidate)?;
    let file = candidate.join("cache.bin");
    fs::write(&file, vec![1_u8; 4_096])?;
    let old = FileTime::from_system_time(SystemTime::now() - Duration::from_hours(960));
    set_file_mtime(&file, old)?;
    set_file_mtime(&candidate, old)?;
    Ok(())
}

#[tokio::test]
async fn approval_is_exact_and_single_use() -> Result<(), Box<dyn std::error::Error>> {
    let fake = TempDir::new()?;
    let home = fake.path().join("home");
    let data = fake.path().join("state");
    fs::create_dir_all(&home)?;
    create_old_cache(&home)?;
    let canary = home.join("Documents/repository/source.dart");
    fs::create_dir_all(canary.parent().ok_or("canary parent missing")?)?;
    fs::write(&canary, "safe")?;

    let service = AppService::open(home, data)?;
    let scan = service.scan().await?;
    let candidate = scan.candidates.first().ok_or("candidate missing")?;
    let plan = service.prepare_plan(
        std::slice::from_ref(&candidate.id),
        ExecutionMode::Quarantine,
    )?;
    service.approve_plan(plan.id)?;
    let receipt = service.execute_approved(plan.id).await?;

    assert!(canary.exists());
    assert_eq!(receipt.items.len(), 1);
    assert!(service.execute_approved(plan.id).await.is_err());
    assert_eq!(service.recent_history(10)?.len(), 1);
    Ok(())
}

async fn quarantine_one_cache(
    fake: &TempDir,
) -> Result<(AppService, std::path::PathBuf), Box<dyn std::error::Error>> {
    let home = fake.path().join("home");
    let data = fake.path().join("state");
    fs::create_dir_all(&home)?;
    create_old_cache(&home)?;
    let service = AppService::open(home, data)?;
    let scan = service.scan().await?;
    let candidate = scan.candidates.first().ok_or("candidate missing")?;
    let original = candidate.lexical_path.clone();
    let plan = service.prepare_plan(
        std::slice::from_ref(&candidate.id),
        ExecutionMode::Quarantine,
    )?;
    service.approve_plan(plan.id)?;
    service.execute_approved(plan.id).await?;
    Ok((service, original))
}

#[tokio::test]
async fn exact_quarantine_entry_can_be_restored() -> Result<(), Box<dyn std::error::Error>> {
    let fake = TempDir::new()?;
    let (service, original) = quarantine_one_cache(&fake).await?;
    let entry = service
        .active_quarantine(10)?
        .into_iter()
        .next()
        .ok_or("quarantine entry missing")?;
    let action = service.prepare_quarantine_action(entry.id, QuarantineAction::Restore)?;
    service.approve_quarantine_action(action.id)?;
    let restored = service.execute_quarantine_action(action.id).await?;

    assert_eq!(restored.state, QuarantineState::Restored);
    assert!(original.exists());
    assert!(!entry.quarantine_path.exists());
    assert!(service.active_quarantine(10)?.is_empty());
    Ok(())
}

#[tokio::test]
async fn exact_quarantine_entry_can_be_purged() -> Result<(), Box<dyn std::error::Error>> {
    let fake = TempDir::new()?;
    let (service, original) = quarantine_one_cache(&fake).await?;
    let entry = service
        .active_quarantine(10)?
        .into_iter()
        .next()
        .ok_or("quarantine entry missing")?;
    let action = service.prepare_quarantine_action(entry.id, QuarantineAction::Purge)?;
    service.approve_quarantine_action(action.id)?;
    let purged = service.execute_quarantine_action(action.id).await?;

    assert_eq!(purged.state, QuarantineState::Purged);
    assert!(!original.exists());
    assert!(!entry.quarantine_path.exists());
    assert!(service.active_quarantine(10)?.is_empty());
    Ok(())
}

#[tokio::test]
async fn restore_never_overwrites_a_recreated_original() -> Result<(), Box<dyn std::error::Error>> {
    let fake = TempDir::new()?;
    let (service, original) = quarantine_one_cache(&fake).await?;
    let entry = service
        .active_quarantine(10)?
        .into_iter()
        .next()
        .ok_or("quarantine entry missing")?;
    fs::create_dir_all(&original)?;
    fs::write(original.join("new-cache.bin"), "new owner data")?;
    let action = service.prepare_quarantine_action(entry.id, QuarantineAction::Restore)?;
    service.approve_quarantine_action(action.id)?;

    assert!(service.execute_quarantine_action(action.id).await.is_err());
    assert_eq!(
        fs::read_to_string(original.join("new-cache.bin"))?,
        "new owner data"
    );
    assert!(entry.quarantine_path.exists());
    Ok(())
}

#[tokio::test]
async fn purge_rejects_a_quarantine_entry_that_changed() -> Result<(), Box<dyn std::error::Error>> {
    let fake = TempDir::new()?;
    let (service, _) = quarantine_one_cache(&fake).await?;
    let entry = service
        .active_quarantine(10)?
        .into_iter()
        .next()
        .ok_or("quarantine entry missing")?;
    fs::write(entry.quarantine_path.join("unexpected.bin"), "changed")?;
    let action = service.prepare_quarantine_action(entry.id, QuarantineAction::Purge)?;
    service.approve_quarantine_action(action.id)?;

    assert!(service.execute_quarantine_action(action.id).await.is_err());
    assert!(entry.quarantine_path.exists());
    assert!(entry.quarantine_path.join("unexpected.bin").exists());
    Ok(())
}

#[tokio::test]
async fn restart_never_resumes_an_approved_quarantine_action()
-> Result<(), Box<dyn std::error::Error>> {
    let fake = TempDir::new()?;
    let (service, _) = quarantine_one_cache(&fake).await?;
    let home = service.home().to_path_buf();
    let data = service.data_dir().to_path_buf();
    let entry = service
        .active_quarantine(10)?
        .into_iter()
        .next()
        .ok_or("quarantine entry missing")?;
    let action = service.prepare_quarantine_action(entry.id, QuarantineAction::Purge)?;
    service.approve_quarantine_action(action.id)?;
    drop(service);

    let restarted = AppService::open(home, data)?;
    assert!(
        restarted
            .execute_quarantine_action(action.id)
            .await
            .is_err()
    );
    assert!(entry.quarantine_path.exists());
    assert_eq!(restarted.active_quarantine(10)?.len(), 1);
    Ok(())
}

fn age(path: &Path, hours: u64) -> Result<(), Box<dyn std::error::Error>> {
    let old = FileTime::from_system_time(SystemTime::now() - Duration::from_hours(hours));
    set_file_mtime(path, old)?;
    Ok(())
}

#[tokio::test]
async fn auto_clean_is_off_by_default_and_only_removes_opted_in_idle_categories()
-> Result<(), Box<dyn std::error::Error>> {
    use cleaner_domain::AutoCleanSettings;

    let fake = TempDir::new()?;
    let home = fake.path().join("home");
    let data = fake.path().join("state");
    fs::create_dir_all(&home)?;

    // An idle npx cache, an idle npm cache the owner did not opt into, and a source file.
    let npx = home.join(".npm/_npx");
    fs::create_dir_all(npx.join("pkg"))?;
    fs::write(npx.join("pkg/index.js"), vec![1_u8; 4_096])?;
    for path in [npx.join("pkg/index.js"), npx.join("pkg"), npx.clone()] {
        age(&path, 72)?;
    }
    let npm = home.join(".npm/_cacache");
    fs::create_dir_all(&npm)?;
    fs::write(npm.join("entry"), vec![1_u8; 4_096])?;
    for path in [npm.join("entry"), npm.clone()] {
        age(&path, 72)?;
    }
    let canary = home.join("Documents/repository/source.dart");
    fs::create_dir_all(canary.parent().ok_or("canary parent missing")?)?;
    fs::write(&canary, "safe")?;

    let service = AppService::open(home, data)?;

    // Nothing is enabled until the owner opts in.
    let idle = service.auto_tick().await?;
    assert!(idle.storage.is_none());
    assert_eq!(service.auto_log(10)?.len(), 0);

    let mut settings = AutoCleanSettings::default();
    settings.storage.enabled = true;
    settings.storage.target_ids = vec!["npm-npx-cache".to_owned()];
    service.set_auto_settings(settings)?;

    let report = service.auto_tick().await?;
    let storage = report.storage.ok_or("storage run missing")?;
    assert_eq!(storage.items, 1);
    assert!(storage.bytes_freed > 0);
    assert!(!npx.exists(), "opted-in idle cache is gone");
    assert!(
        npm.exists(),
        "category the owner did not choose is untouched"
    );
    assert!(canary.exists());
    assert!(
        service.active_quarantine(10)?.is_empty(),
        "purged, not parked"
    );

    let log = service.auto_log(10)?;
    assert_eq!(log.len(), 1);
    assert!(log[0].summary.contains("Removed 1 item"));

    // The next tick is not due yet, so nothing else is scanned or removed.
    let again = service.auto_tick().await?;
    assert!(again.storage.is_none());
    Ok(())
}

#[tokio::test]
async fn auto_clean_never_removes_recently_changed_items() -> Result<(), Box<dyn std::error::Error>>
{
    use cleaner_domain::AutoCleanSettings;

    let fake = TempDir::new()?;
    let home = fake.path().join("home");
    let data = fake.path().join("state");
    let fresh = home.join(".npm/_npx");
    fs::create_dir_all(&fresh)?;
    fs::write(fresh.join("index.js"), "just now")?;

    let service = AppService::open(home, data)?;
    let mut settings = AutoCleanSettings::default();
    settings.storage.enabled = true;
    settings.storage.target_ids = vec!["npm-npx-cache".to_owned()];
    service.set_auto_settings(settings)?;

    let report = service.auto_tick().await?;
    assert_eq!(report.storage.ok_or("storage run missing")?.items, 0);
    assert!(fresh.exists());
    Ok(())
}

#[tokio::test]
async fn auto_settings_reject_protected_apps_and_quit_reports_are_logged()
-> Result<(), Box<dyn std::error::Error>> {
    use cleaner_domain::{AutoCleanSettings, AutoLogKind};

    let fake = TempDir::new()?;
    let home = fake.path().join("home");
    fs::create_dir_all(&home)?;
    let service = AppService::open(home, fake.path().join("state"))?;

    let mut bad = AutoCleanSettings::default();
    bad.ram.quit_apps = vec!["Docker Desktop".to_owned()];
    assert!(service.set_auto_settings(bad).is_err());
    assert!(service.auto_settings()?.ram.quit_apps.is_empty());

    service.auto_report_quit("Slack", 900 * 1_048_576)?;
    let log = service.auto_log(5)?;
    assert_eq!(log[0].kind, AutoLogKind::RamQuit);
    assert!(log[0].summary.contains("Slack"));
    Ok(())
}
