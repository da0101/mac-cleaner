//! Generated project data and tool caches: discovered narrowly, moved whole.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::time::{Duration, SystemTime};

use chrono::Utc;
use cleaner_domain::ExecutionMode;
use cleaner_fs::{PlanExecutor, Scanner};
use cleaner_policy::{approve_plan, prepare_plan};
use filetime::{FileTime, set_file_mtime};
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn node_project(home: &Path, name: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let project = home.join("Documents/GitHub").join(name);
    let modules = project.join("node_modules");
    fs::create_dir_all(modules.join("left-pad/lib"))?;
    fs::create_dir_all(modules.join(".bin"))?;
    fs::write(project.join("package.json"), "{}")?;
    fs::write(project.join("package-lock.json"), "{}")?;
    fs::write(project.join("index.js"), "source")?;
    fs::write(modules.join("left-pad/lib/index.js"), vec![1_u8; 4_096])?;
    symlink("../left-pad/lib/index.js", modules.join(".bin/left-pad"))?;
    Ok(project)
}

#[test]
fn node_modules_with_bin_symlinks_is_a_candidate_but_sources_are_not() -> TestResult {
    let fake = TempDir::new()?;
    node_project(fake.path(), "web")?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    let ids: Vec<_> = report
        .candidates
        .iter()
        .map(|candidate| candidate.target_id.0.as_str())
        .collect();
    assert_eq!(ids, ["project-node-modules"]);
    assert!(
        report.candidates[0]
            .lexical_path
            .ends_with("web/node_modules")
    );
    Ok(())
}

#[test]
fn moving_node_modules_keeps_lockfile_and_sources() -> TestResult {
    let fake = TempDir::new()?;
    let project = node_project(fake.path(), "web")?;
    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    let candidate = report.candidates.first().ok_or("candidate missing")?;

    let plan = prepare_plan(
        &report.candidates,
        std::slice::from_ref(&candidate.id),
        Utc::now(),
        Duration::from_mins(5),
        ExecutionMode::Quarantine,
    )?;
    let receipt =
        PlanExecutor::new(fake.path().to_path_buf()).execute(approve_plan(plan, Utc::now())?)?;

    assert!(!project.join("node_modules").exists());
    assert!(project.join("package.json").exists());
    assert!(project.join("package-lock.json").exists());
    assert!(project.join("index.js").exists());
    assert!(
        receipt.items[0]
            .quarantine_path
            .as_ref()
            .is_some_and(|path| path.join(".bin/left-pad").symlink_metadata().is_ok())
    );
    Ok(())
}

#[test]
fn generated_folder_without_its_project_marker_is_ignored() -> TestResult {
    let fake = TempDir::new()?;
    let project = fake.path().join("Documents/GitHub/notes");
    fs::create_dir_all(project.join("node_modules/x"))?;
    fs::create_dir_all(project.join("target/x"))?;
    fs::write(project.join("node_modules/x/a"), "a")?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert!(report.candidates.is_empty());
    Ok(())
}

#[test]
fn generated_folder_holding_a_repository_is_refused() -> TestResult {
    let fake = TempDir::new()?;
    let project = fake.path().join("Documents/GitHub/app");
    fs::create_dir_all(project.join("target/.git"))?;
    fs::write(project.join("Cargo.toml"), "[package]")?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert!(report.candidates.is_empty());
    assert!(
        report
            .skipped
            .iter()
            .any(|skip| skip.reason.contains("protected project sentinel"))
    );
    Ok(())
}

#[test]
fn symlinked_generated_folder_is_refused() -> TestResult {
    let fake = TempDir::new()?;
    let project = fake.path().join("Documents/GitHub/app");
    let elsewhere = fake.path().join("Documents/precious");
    fs::create_dir_all(&project)?;
    fs::create_dir_all(&elsewhere)?;
    fs::write(project.join("package.json"), "{}")?;
    symlink(&elsewhere, project.join("node_modules"))?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert!(report.candidates.is_empty());
    Ok(())
}

#[test]
fn deep_build_product_copies_of_sentinels_no_longer_block_derived_data() -> TestResult {
    let fake = TempDir::new()?;
    let derived = fake
        .path()
        .join("Library/Developer/Xcode/DerivedData/Runner-abc");
    let deep = derived.join("Build/Products/Debug-iphonesimulator/App.framework");
    fs::create_dir_all(&deep)?;
    fs::write(deep.join("Podfile"), "copied by the build")?;
    fs::write(deep.join("pubspec.yaml"), "copied by the build")?;
    let old = FileTime::from_system_time(SystemTime::now() - Duration::from_hours(960));
    for path in [
        deep.join("Podfile"),
        deep.join("pubspec.yaml"),
        deep.clone(),
        derived.join("Build/Products/Debug-iphonesimulator"),
        derived.join("Build/Products"),
        derived.join("Build"),
        derived.clone(),
    ] {
        set_file_mtime(&path, old)?;
    }

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert_eq!(report.candidates.len(), 1);
    assert_eq!(report.candidates[0].target_id.0, "xcode-derived-data");
    Ok(())
}

#[test]
fn recent_derived_data_is_reported_with_a_reason() -> TestResult {
    let fake = TempDir::new()?;
    let derived = fake
        .path()
        .join("Library/Developer/Xcode/DerivedData/Fresh-abc/Build");
    fs::create_dir_all(&derived)?;
    fs::write(derived.join("out.o"), "new")?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert!(report.candidates.is_empty());
    assert!(
        report
            .skipped
            .iter()
            .any(|skip| skip.reason.starts_with("used within the last 14 days"))
    );
    Ok(())
}

#[test]
fn named_tool_caches_are_whole_directory_candidates() -> TestResult {
    let fake = TempDir::new()?;
    fs::create_dir_all(fake.path().join(".npm/_cacache/index-v5"))?;
    fs::write(fake.path().join(".npm/_cacache/index-v5/a"), "a")?;
    fs::write(fake.path().join(".npm/config-keep"), "keep")?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert_eq!(report.candidates.len(), 1);
    assert!(report.candidates[0].lexical_path.ends_with(".npm/_cacache"));
    Ok(())
}

fn git(project: &Path, args: &[&str]) -> TestResult {
    let status = std::process::Command::new("/usr/bin/git")
        .arg("-C")
        .arg(project)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .status()?;
    assert!(status.success(), "git {args:?} failed");
    Ok(())
}

fn ios_project(home: &Path) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let project = home.join("Documents/GitHub/phone");
    fs::create_dir_all(project.join("ios/Runner.xcodeproj"))?;
    fs::create_dir_all(project.join("ios/build/Release"))?;
    fs::write(project.join("ios/build/Release/app.o"), vec![1_u8; 4_096])?;
    Ok(project)
}

fn ios_build_offered(home: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    // Look ahead of the idle window so only the Git rules decide.
    let report =
        Scanner::new(home.to_path_buf()).scan_at(Utc::now() + chrono::Duration::days(30))?;
    Ok(report
        .candidates
        .iter()
        .any(|candidate| candidate.target_id.0 == "project-ios-build"))
}

#[test]
fn nested_ios_build_is_offered_only_when_git_ignores_and_does_not_track_it() -> TestResult {
    let fake = TempDir::new()?;
    let project = ios_project(fake.path())?;

    // Not a Git repository: fail closed.
    assert!(!ios_build_offered(fake.path())?);

    git(&project, &["init", "-q"])?;
    // A repository that does not ignore the folder: still not offered.
    assert!(!ios_build_offered(fake.path())?);

    fs::write(project.join(".gitignore"), "/ios/build/\n")?;
    assert!(ios_build_offered(fake.path())?);

    // Anything tracked inside the folder makes it ineligible again.
    git(&project, &["add", "-f", "ios/build/Release/app.o"])?;
    assert!(!ios_build_offered(fake.path())?);
    Ok(())
}

#[test]
fn nested_ios_build_is_skipped_while_recently_used() -> TestResult {
    let fake = TempDir::new()?;
    let project = ios_project(fake.path())?;
    git(&project, &["init", "-q"])?;
    fs::write(project.join(".gitignore"), "/ios/build/\n")?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    assert!(
        report
            .candidates
            .iter()
            .all(|candidate| candidate.target_id.0 != "project-ios-build")
    );
    assert!(
        report
            .skipped
            .iter()
            .any(|skip| skip.target_id.0 == "project-ios-build"
                && skip.reason.starts_with("used within"))
    );
    Ok(())
}

#[test]
fn project_yarn_cache_keeps_the_yarn_config_and_lockfile() -> TestResult {
    let fake = TempDir::new()?;
    let project = fake.path().join("Documents/GitHub/web");
    fs::create_dir_all(project.join(".yarn/berry-cache"))?;
    fs::write(project.join(".yarn/berry-cache/pkg.zip"), vec![1_u8; 4_096])?;
    fs::write(project.join("package.json"), "{}")?;
    fs::write(project.join("yarn.lock"), "lock")?;
    fs::write(project.join(".yarnrc.yml"), "nodeLinker: node-modules")?;
    git(&project, &["init", "-q"])?;
    fs::write(project.join(".gitignore"), ".yarn/\n")?;

    let report = Scanner::new(fake.path().to_path_buf()).scan()?;
    let candidate = report
        .candidates
        .iter()
        .find(|candidate| candidate.target_id.0 == "project-yarn-offline-cache")
        .ok_or("yarn cache missing")?;
    assert!(candidate.lexical_path.ends_with("web/.yarn/berry-cache"));
    assert!(candidate.relative_entry.ends_with("web/.yarn/berry-cache"));

    let plan = prepare_plan(
        &report.candidates,
        std::slice::from_ref(&candidate.id),
        Utc::now(),
        Duration::from_mins(5),
        ExecutionMode::Quarantine,
    )?;
    PlanExecutor::new(fake.path().to_path_buf()).execute(approve_plan(plan, Utc::now())?)?;
    assert!(!project.join(".yarn/berry-cache").exists());
    assert!(project.join("yarn.lock").exists());
    assert!(project.join(".yarnrc.yml").exists());
    Ok(())
}
