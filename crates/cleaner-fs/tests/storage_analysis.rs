use std::fs;
use std::os::unix::fs::symlink;

use cleaner_domain::StorageDisposition;
use cleaner_fs::StorageAnalyzer;
use tempfile::tempdir;

#[test]
fn reports_generated_project_output_without_nominating_source_or_git()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let home = temp.path();
    let project = home.join("Documents/GitHub/example");
    fs::create_dir_all(project.join("build"))?;
    fs::create_dir_all(project.join(".git"))?;
    fs::write(project.join("pubspec.yaml"), b"name: example\n")?;
    fs::write(project.join("lib.dart"), vec![1_u8; 8_192])?;
    fs::write(project.join(".git/index"), vec![2_u8; 8_192])?;
    fs::write(project.join("build/output.bin"), vec![3_u8; 16_384])?;

    let report =
        StorageAnalyzer::new(home.to_path_buf()).analyze(500_000_000_000, 5_000_000_000)?;

    assert_eq!(report.pressure, cleaner_domain::StoragePressure::Critical);
    assert!(report.hotspots.iter().any(|item| {
        item.path == project.join("build") && item.disposition == StorageDisposition::Reviewable
    }));
    assert!(
        report
            .hotspots
            .iter()
            .all(|item| !item.path.ends_with(".git"))
    );
    assert!(
        report
            .hotspots
            .iter()
            .all(|item| !item.path.ends_with("lib.dart"))
    );
    Ok(())
}

#[test]
fn symbolic_links_are_not_followed_during_read_only_measurement()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let home = temp.path();
    let project = home.join("Documents/GitHub/example");
    let outside = tempdir()?;
    fs::create_dir_all(&project)?;
    fs::write(project.join("package.json"), b"{}")?;
    fs::write(outside.path().join("secret"), vec![9_u8; 65_536])?;
    symlink(outside.path(), project.join("node_modules"))?;

    let report =
        StorageAnalyzer::new(home.to_path_buf()).analyze(500_000_000_000, 50_000_000_000)?;

    assert!(
        report
            .hotspots
            .iter()
            .all(|item| item.path != project.join("node_modules"))
    );
    Ok(())
}

#[test]
fn oversized_trees_fail_closed_with_an_explanation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let home = temp.path();
    let cache = home.join(".npm/_cacache");
    fs::create_dir_all(&cache)?;
    fs::write(cache.join("one"), b"one")?;
    fs::write(cache.join("two"), b"two")?;

    let report = StorageAnalyzer::new(home.to_path_buf())
        .with_max_entries(1)
        .analyze(500_000_000_000, 50_000_000_000)?;

    assert!(report.hotspots.is_empty());
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains("entry limit"))
    );
    Ok(())
}
