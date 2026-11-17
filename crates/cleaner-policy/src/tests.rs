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
