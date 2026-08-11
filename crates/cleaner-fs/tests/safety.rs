use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::time::{Duration, SystemTime};

use chrono::Utc;
use cleaner_domain::{ExecutionMode, SafetyTier};
use cleaner_fs::{PlanExecutor, Scanner};
use cleaner_policy::{approve_plan, prepare_plan};
use filetime::{FileTime, set_file_mtime};
use tempfile::TempDir;

fn old_time() -> FileTime {
    FileTime::from_system_time(SystemTime::now() - Duration::from_hours(960))
}

fn create_old_candidate(home: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let candidate = home
        .join("Library/Caches/com.da0101.mac-cleaner")
        .join(name);
    fs::create_dir_all(&candidate)?;
    let file = candidate.join("cache.bin");
    fs::write(&file, vec![7_u8; 8_192])?;
    set_file_mtime(&file, old_time())?;
    set_file_mtime(&candidate, old_time())?;
    Ok(())
}

#[test]
fn scan_discovers_only_compiled_review_targets() -> Result<(), Box<dyn std::error::Error>> {
    let fake = TempDir::new()?;
    create_old_candidate(fake.path(), "expired")?;
    fs::create_dir_all(fake.path().join("Documents/GitHub/important"))?;
    fs::write(
        fake.path().join("Documents/GitHub/important/source.rs"),
        "safe",
    )?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert_eq!(report.candidates.len(), 1);
    assert_eq!(report.candidates[0].safety_tier, SafetyTier::ReviewRequired);
    assert!(
        fake.path()
            .join("Documents/GitHub/important/source.rs")
            .exists()
    );
    Ok(())
}

#[test]
fn scan_skips_symlinked_candidates_and_project_sentinels() -> Result<(), Box<dyn std::error::Error>>
{
    let fake = TempDir::new()?;
    let root = fake.path().join("Library/Caches/com.da0101.mac-cleaner");
    fs::create_dir_all(&root)?;
    symlink(fake.path().join("Documents"), root.join("linked"))?;
    create_old_candidate(fake.path(), "looks-like-project")?;
    fs::create_dir_all(root.join("looks-like-project/.git"))?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert!(report.candidates.is_empty());
    assert_eq!(report.skipped.len(), 2);
    Ok(())
}

#[test]
fn approved_plan_moves_only_the_exact_candidate_to_private_quarantine()
-> Result<(), Box<dyn std::error::Error>> {
    let fake = TempDir::new()?;
    create_old_candidate(fake.path(), "expired")?;
    let outside = fake.path().join("Documents/outside-canary.txt");
    fs::create_dir_all(outside.parent().ok_or("canary has no parent")?)?;
    fs::write(&outside, "must survive")?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    let candidate = report.candidates.first().ok_or("candidate missing")?;
    let original = candidate.lexical_path.clone();
    let plan = prepare_plan(
        &report.candidates,
        std::slice::from_ref(&candidate.id),
        Utc::now(),
        Duration::from_mins(5),
        ExecutionMode::Quarantine,
    )?;
    let approved = approve_plan(plan, Utc::now())?;
    let receipt = PlanExecutor::new(fake.path().to_path_buf()).execute(approved)?;

    assert!(!original.exists());
    assert!(outside.exists());
    assert_eq!(receipt.items.len(), 1);
    assert!(
        receipt.items[0]
            .quarantine_path
            .as_ref()
            .is_some_and(|path| path.exists())
    );
    Ok(())
}

#[test]
fn changed_candidate_fails_closed_before_any_move() -> Result<(), Box<dyn std::error::Error>> {
    let fake = TempDir::new()?;
    create_old_candidate(fake.path(), "first")?;
    create_old_candidate(fake.path(), "second")?;
    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert_eq!(report.candidates.len(), 2);
    let selected = report
        .candidates
        .iter()
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();
    let plan = prepare_plan(
        &report.candidates,
        &selected,
        Utc::now(),
        Duration::from_mins(5),
        ExecutionMode::Quarantine,
    )?;
    fs::write(report.candidates[1].lexical_path.join("new.bin"), "changed")?;
    let approved = approve_plan(plan, Utc::now())?;

    assert!(
        PlanExecutor::new(fake.path().to_path_buf())
            .execute(approved)
            .is_err()
    );
    assert!(report.candidates[0].lexical_path.exists());
    assert!(report.candidates[1].lexical_path.exists());
    Ok(())
}
