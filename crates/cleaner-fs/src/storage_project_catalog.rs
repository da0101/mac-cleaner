use std::path::Path;

use cleaner_domain::StorageCategory::{ProjectBuild, ProjectDependencies, TestArtifacts};

use crate::storage_catalog::HotspotSpec;

#[derive(Clone, Copy)]
pub(crate) enum ProjectKind {
    Flutter,
    Node,
    Rust,
}

pub(crate) fn project_specs(root: &Path, kind: ProjectKind) -> Vec<HotspotSpec> {
    match kind {
        ProjectKind::Flutter => vec![
            HotspotSpec::project(
                root.join("build"),
                "Flutter project build output",
                ProjectBuild,
                "Run `flutter clean` only for this exact inactive project.",
            ),
            HotspotSpec::project(
                root.join(".dart_tool"),
                "Flutter project tool state",
                ProjectBuild,
                "Use `flutter clean` only for this exact inactive project.",
            ),
            HotspotSpec::project(
                root.join("ios/build"),
                "iOS project build output",
                ProjectBuild,
                "Use the project's Xcode or Flutter clean workflow while it is inactive.",
            ),
            HotspotSpec::project(
                root.join("ios/DerivedData"),
                "Project-local Xcode DerivedData",
                ProjectBuild,
                "Use the project's Xcode clean workflow while it is inactive.",
            ),
            HotspotSpec::project(
                root.join("android/app/build"),
                "Android app build output",
                ProjectBuild,
                "Use the project's Gradle or Flutter clean workflow while it is inactive.",
            ),
        ],
        ProjectKind::Node => vec![
            HotspotSpec::project(
                root.join("node_modules"),
                "Node project dependencies",
                ProjectDependencies,
                "Remove only through this inactive project's package manager; retain its lockfile.",
            ),
            HotspotSpec::project(
                root.join(".yarn/cache"),
                "Yarn project cache",
                ProjectDependencies,
                "Use this project's Yarn workflow and preserve offline archives when required.",
            ),
        ],
        ProjectKind::Rust => vec![HotspotSpec::project(
            root.join("target"),
            "Rust project build output",
            ProjectBuild,
            "Run `cargo clean` only for this exact inactive project.",
        )],
    }
}

pub(crate) fn project_test_specs(root: &Path) -> Vec<HotspotSpec> {
    ["test-results", "playwright-report", "maestro-reports"]
        .into_iter()
        .map(|name| {
            HotspotSpec::project(
                root.join(name),
                "Automated test artifacts",
                TestArtifacts,
                "Review recordings and reports for this exact inactive project before removal.",
            )
        })
        .collect()
}
