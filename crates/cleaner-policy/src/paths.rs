use std::path::{Component, Path};

use crate::PolicyError;

const PROTECTED_HOME_PREFIXES: &[&str] = &[
    "Desktop",
    "Documents",
    "Downloads",
    "Movies",
    "Music",
    "Pictures",
    ".gradle",
    ".pub-cache",
    ".ssh",
    "Library/Application Support/GitHub Desktop",
    "Library/Keychains",
    "Library/Preferences/com.github.GitHubClient.plist",
];

const PROTECTED_SENTINELS: &[&str] = &[
    ".git",
    "pubspec.yaml",
    "pubspec.lock",
    "Podfile",
    "Podfile.lock",
    "gradlew",
    "gradle.properties",
    "settings.gradle",
    "settings.gradle.kts",
    "build.gradle",
    "build.gradle.kts",
];

pub fn validate_home(home: &Path) -> Result<(), PolicyError> {
    if !home.is_absolute() || home.parent().is_none() || home == Path::new("/") {
        return Err(PolicyError::InvalidHome(home.to_path_buf()));
    }
    Ok(())
}

#[must_use]
pub fn is_safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path.components().all(|component| {
            matches!(component, Component::Normal(_))
                && component != Component::Normal("..".as_ref())
        })
}

#[must_use]
pub fn is_protected(home: &Path, candidate: &Path) -> bool {
    if candidate == home || !candidate.starts_with(home) {
        return true;
    }

    PROTECTED_HOME_PREFIXES.iter().any(|relative| {
        let protected = home.join(relative);
        candidate.starts_with(&protected) || protected.starts_with(candidate)
    })
}

#[must_use]
pub fn protected_sentinels() -> &'static [&'static str] {
    PROTECTED_SENTINELS
}
