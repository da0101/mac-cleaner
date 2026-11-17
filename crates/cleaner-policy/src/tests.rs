use std::path::Path;

use crate::{catalog, catalog_root, is_protected, safe_auto_catalog};

#[test]
fn automatic_cleanup_catalog_starts_empty() {
    assert!(safe_auto_catalog().is_empty());
}

#[test]
fn catalog_never_points_at_a_protected_prefix() {
    let home = Path::new("/Users/tester");
    for target in catalog() {
        assert!(catalog_root(home, target).is_ok(), "{}", target.id);
    }
}

#[test]
fn github_desktop_state_is_protected_in_both_directions() {
    let home = Path::new("/Users/tester");
    let state = home.join("Library/Application Support/GitHub Desktop");
    assert!(is_protected(home, &state));
    assert!(is_protected(home, &state.join("IndexedDB/repositories")));
    assert!(is_protected(
        home,
        &home.join("Library/Application Support")
    ));
}

#[test]
fn source_and_toolchain_roots_are_protected() {
    let home = Path::new("/Users/tester");
    assert!(is_protected(home, &home.join("Documents/GitHub/app")));
    assert!(is_protected(home, &home.join(".pub-cache/hosted")));
    assert!(is_protected(home, &home.join(".gradle/caches")));
    assert!(is_protected(home, Path::new("/tmp/outside-home")));
}

#[test]
fn named_and_project_targets_reject_anything_but_their_exact_entry() -> Result<(), &'static str> {
    use crate::{TargetLayout, candidate_path};
    let home = Path::new("/Users/tester");
    let named = catalog()
        .iter()
        .find(|target| target.id == "npm-content-cache")
        .ok_or("npm target missing")?;
    assert!(candidate_path(home, named, Path::new("_cacache")).is_ok());
    assert!(candidate_path(home, named, Path::new("_npx")).is_err());
    assert!(candidate_path(home, named, Path::new("_cacache/index")).is_err());

    let project = catalog()
        .iter()
        .find(|target| target.id == "project-node-modules")
        .ok_or("node target missing")?;
    assert!(matches!(project.layout, TargetLayout::Project { .. }));
    // No marker file exists under this fake home, so even a well-formed path fails closed.
    assert!(candidate_path(home, project, Path::new("app/node_modules")).is_err());
    assert!(candidate_path(home, project, Path::new("node_modules")).is_err());
    assert!(candidate_path(home, project, Path::new("app/src")).is_err());
    assert!(candidate_path(home, project, Path::new("a/b/c/node_modules")).is_err());
    assert!(candidate_path(home, project, Path::new("app/../node_modules")).is_err());
    Ok(())
}

#[test]
fn only_project_targets_may_be_discovered_below_documents() {
    for target in catalog() {
        if target.home_relative_root.starts_with("Documents") {
            assert!(target.allows_internal_links(), "{}", target.id);
            assert!(
                matches!(target.layout, crate::TargetLayout::Project { .. }),
                "{}",
                target.id
            );
        }
    }
}

#[test]
fn nested_project_entries_need_their_marker_and_exact_shape()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{candidate_path, marker_present};
    let home = tempfile::TempDir::new()?;
    let project = home.path().join("Documents/GitHub/app");
    std::fs::create_dir_all(project.join("ios/Runner.xcodeproj"))?;
    std::fs::create_dir_all(project.join("ios/build"))?;
    let target = catalog()
        .iter()
        .find(|target| target.id == "project-ios-build")
        .ok_or("ios build target missing")?;
    assert_eq!(target.entry_depth(), 2);
    assert!(target.require_git_ignored);
    assert!(marker_present(&project, "ios/*.xcodeproj"));
    assert!(!marker_present(&project, "ios/*.xcworkspace"));

    assert!(candidate_path(home.path(), target, std::path::Path::new("app/ios/build")).is_ok());
    // Wrong tail, wrong depth, missing marker, or traversal all fail closed.
    assert!(candidate_path(home.path(), target, std::path::Path::new("app/ios/src")).is_err());
    assert!(candidate_path(home.path(), target, std::path::Path::new("ios/build")).is_err());
    assert!(candidate_path(home.path(), target, std::path::Path::new("a/b/c/ios/build")).is_err());
    assert!(candidate_path(home.path(), target, std::path::Path::new("other/ios/build")).is_err());
    assert!(
        candidate_path(
            home.path(),
            target,
            std::path::Path::new("app/../ios/build")
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn xcode_symlinked_project_markers_do_not_count() -> Result<(), Box<dyn std::error::Error>> {
    use crate::marker_present;
    let dir = tempfile::TempDir::new()?;
    std::fs::create_dir_all(dir.path().join("ios"))?;
    std::os::unix::fs::symlink("/tmp", dir.path().join("ios/Fake.xcodeproj"))?;
    assert!(!marker_present(dir.path(), "ios/*.xcodeproj"));
    Ok(())
}

#[test]
fn newly_added_home_targets_keep_expected_ages() -> Result<(), &'static str> {
    for (id, hours) in [("xcode-device-support", 72), ("chrome-on-device-model", 24)] {
        let target = catalog()
            .iter()
            .find(|target| target.id == id)
            .ok_or("target missing")?;
        assert_eq!(target.minimum_age.as_secs(), hours * 3_600, "{id}");
    }
    Ok(())
}
