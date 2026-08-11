use std::path::{Path, PathBuf};

use cleaner_domain::StorageDisposition;

#[derive(Clone, Copy)]
pub(crate) enum ProjectKind {
    Flutter,
    Node,
}

pub(crate) struct HotspotSpec {
    pub path: PathBuf,
    pub display_name: String,
    pub disposition: StorageDisposition,
    pub reason: String,
    pub recommendation: String,
    pub recovery_cost: String,
}

impl HotspotSpec {
    pub(crate) fn project_cache(path: PathBuf) -> Self {
        Self {
            display_name: "Project model/download cache".into(),
            path,
            disposition: StorageDisposition::ToolManaged,
            reason: "Downloaded project assets can be large but may be needed offline.".into(),
            recommendation:
                "Review with the project owner before using its documented cache command.".into(),
            recovery_cost: "May require a large download or regeneration.".into(),
        }
    }
}

// Keeping the versioned cleanup catalog together makes policy review materially easier.
#[allow(clippy::too_many_lines)]
pub(crate) fn fixed_specs() -> Vec<HotspotSpec> {
    use StorageDisposition::{Protected, Reviewable, ToolManaged};
    vec![
        spec(
            "Library/Containers/com.docker.docker/Data/vms/0/data",
            "Docker Desktop virtual disk",
            ToolManaged,
            "Images, containers, build cache, and volumes share this disk.",
            "Open Docker Desktop and inspect usage; prune only selected unused objects. Never raw-delete the virtual disk.",
            "Deleting images requires re-downloads; deleting volumes can destroy data.",
        ),
        spec(
            "Library/Developer/CoreSimulator/Devices",
            "Apple simulator devices",
            ToolManaged,
            "Simulator runtimes and device data can grow substantially.",
            "Use Xcode Devices and Simulators or simctl to remove only shutdown devices you no longer need.",
            "Removed devices lose their installed apps and local simulator data.",
        ),
        spec(
            "Library/Developer/Xcode/DerivedData",
            "Xcode DerivedData",
            Reviewable,
            "Build products and indexes are rebuildable.",
            "Close active Xcode builds, then use Xcode/project clean or review eligible DerivedData below.",
            "Xcode will rebuild and re-index projects.",
        ),
        spec(
            "Library/Developer/Xcode/iOS DeviceSupport",
            "Xcode device support",
            ToolManaged,
            "Debug support is retained for connected iOS versions.",
            "Remove only obsolete OS versions through Xcode-aware maintenance.",
            "Physical-device debugging may need to download support again.",
        ),
        spec(
            "Library/Android/sdk",
            "Android SDK and system images",
            Protected,
            "This is an installed development toolchain, not garbage.",
            "Use Android Studio SDK Manager to uninstall specific unused images or NDK versions.",
            "Flutter/Android builds fail if required SDK components are removed.",
        ),
        spec(
            ".cache/uv",
            "uv package cache",
            Reviewable,
            "Downloaded Python packages and build artifacts are reproducible.",
            "Use `uv cache prune` after confirming no environment setup is running.",
            "Future environments may download packages again.",
        ),
        spec(
            ".cache/huggingface",
            "Hugging Face model cache",
            ToolManaged,
            "Downloaded models can be essential for offline work.",
            "Review models in the owning tool and remove only ones you can re-download.",
            "Models may require large downloads.",
        ),
        spec(
            ".cache/puppeteer",
            "Puppeteer browser cache",
            Reviewable,
            "Downloaded browser builds can be restored by Puppeteer.",
            "Use Puppeteer's supported browser/cache command when no browser job is active.",
            "Browser binaries will download again.",
        ),
        spec(
            ".gradle",
            "Gradle user home",
            ToolManaged,
            "Contains caches plus configuration, wrappers, and toolchains.",
            "Let Gradle perform its own retention; never delete the whole directory.",
            "Raw deletion can break builds and remove configuration.",
        ),
        spec(
            ".npm/_cacache",
            "npm content cache",
            Reviewable,
            "npm's content-addressed cache can be verified and rebuilt.",
            "Use `npm cache verify`; clean only after review if space is urgent.",
            "Packages will download again.",
        ),
        spec(
            ".npm/_npx",
            "npx package cache",
            Reviewable,
            "Temporary npx-installed packages can be recreated.",
            "Review cached tools and clear only when no npx job is running.",
            "npx tools will download again.",
        ),
        spec(
            ".pub-cache",
            "Dart and Flutter package cache",
            Protected,
            "Contains packages and global activation state used by Flutter projects.",
            "Prefer `dart pub cache gc` after reviewing affected projects; never raw-delete it.",
            "Projects may fail offline and need `pub get` again.",
        ),
        spec(
            "Library/Application Support/Claude/vm_bundles",
            "Claude application runtime",
            Protected,
            "Application runtime state is not a generic cache.",
            "Use Claude's supported maintenance or reinstall flow only.",
            "Raw deletion can stop Claude from starting.",
        ),
        spec(
            "Library/Application Support/Google/Chrome/OptGuideOnDeviceModel",
            "Chrome on-device model",
            ToolManaged,
            "Chrome manages this downloaded model inside Application Support.",
            "Review Chrome's own settings/components; do not treat Application Support as cache.",
            "Chrome may download the model again.",
        ),
    ]
}

pub(crate) fn project_specs(root: &Path, kind: ProjectKind) -> Vec<HotspotSpec> {
    use StorageDisposition::Reviewable;
    match kind {
        ProjectKind::Flutter => vec![
            spec_abs(
                root.join("build"),
                "Flutter project build output",
                Reviewable,
                "Generated output is recreated by Flutter.",
                "For this exact inactive project, run `flutter clean` from the project root.",
                "The next build will take longer.",
            ),
            spec_abs(
                root.join(".dart_tool"),
                "Flutter project tool state",
                Reviewable,
                "Generated Dart tool metadata is recreated with package resolution.",
                "Clean only through `flutter clean` for this exact inactive project.",
                "The next build may run `pub get` and regenerate metadata.",
            ),
        ],
        ProjectKind::Node => vec![
            spec_abs(
                root.join("node_modules"),
                "Node project dependencies",
                Reviewable,
                "Installed dependencies are reproducible from the lockfile.",
                "For this exact inactive project, remove with its package manager and retain the lockfile.",
                "A full dependency install will be required.",
            ),
            spec_abs(
                root.join(".yarn/berry-cache"),
                "Yarn project cache",
                Reviewable,
                "Project package archives may be reproducible from the lockfile.",
                "Use this project's Yarn cache workflow; verify it is not configured for offline installs.",
                "Dependencies may need to download again.",
            ),
        ],
    }
}

fn spec(
    path: &str,
    name: &str,
    disposition: StorageDisposition,
    reason: &str,
    recommendation: &str,
    cost: &str,
) -> HotspotSpec {
    spec_abs(
        PathBuf::from(path),
        name,
        disposition,
        reason,
        recommendation,
        cost,
    )
}

fn spec_abs(
    path: PathBuf,
    name: &str,
    disposition: StorageDisposition,
    reason: &str,
    recommendation: &str,
    cost: &str,
) -> HotspotSpec {
    HotspotSpec {
        path,
        display_name: name.into(),
        disposition,
        reason: reason.into(),
        recommendation: recommendation.into(),
        recovery_cost: cost.into(),
    }
}
