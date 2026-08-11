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
