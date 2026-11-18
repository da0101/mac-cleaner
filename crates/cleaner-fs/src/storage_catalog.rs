use std::path::PathBuf;

use cleaner_domain::{StorageCategory, StorageDisposition};

pub(crate) struct HotspotSpec {
    pub path: PathBuf,
    pub display_name: String,
    pub category: StorageCategory,
    pub disposition: StorageDisposition,
    pub reason: String,
    pub recommendation: String,
    pub recovery_cost: String,
}

impl HotspotSpec {
    pub(crate) fn project_cache(path: PathBuf) -> Self {
        spec_abs(
            path,
            "Project model/download cache",
            StorageCategory::PackageCache,
            StorageDisposition::ToolManaged,
            "Downloaded project assets can be large but may be needed offline.",
            "Review with the project owner before using its documented cache command.",
            "May require a large download or regeneration.",
        )
    }

    pub(crate) fn project(
        path: PathBuf,
        name: &str,
        category: StorageCategory,
        recommendation: &str,
    ) -> Self {
        spec_abs(
            path,
            name,
            category,
            StorageDisposition::Reviewable,
            "Generated project data can be recreated from source and lockfiles.",
            recommendation,
            "The next build or test run will regenerate data and may take longer.",
        )
    }
}

// Keeping the read-only catalog together makes coverage and safety review easier.
#[allow(clippy::too_many_lines)]
pub(crate) fn fixed_specs() -> Vec<HotspotSpec> {
    use StorageCategory::{
        AndroidDevelopment, AppleDevelopment, ApplicationSupport, ContainerData, PackageCache,
    };
    use StorageDisposition::{Protected, Reviewable, ToolManaged};
    vec![
        spec(
            "Library/Containers/com.docker.docker/Data/vms/0/data",
            "Docker Desktop virtual disk",
            ContainerData,
            ToolManaged,
            "Images, containers, build cache, and volumes share this disk.",
            "Open Docker Desktop and inspect usage; prune only selected unused objects. Never raw-delete the virtual disk.",
            "Deleting images requires re-downloads; deleting volumes can destroy data.",
        ),
        spec(
            "Library/Developer/CoreSimulator/Devices",
            "Apple simulator devices",
            AppleDevelopment,
            ToolManaged,
            "Simulator devices and their app data can grow substantially.",
            "Use Xcode Devices and Simulators or simctl to remove only shutdown devices you no longer need.",
            "Removed devices lose installed apps and local simulator data.",
        ),
        spec(
            "Library/Developer/Xcode/DerivedData",
            "Xcode DerivedData",
            AppleDevelopment,
            Reviewable,
            "Build products and indexes are rebuildable.",
            "Close active Xcode builds, then use Xcode or the project clean workflow.",
            "Xcode will rebuild and re-index projects.",
        ),
        spec(
            "Library/Developer/Xcode/iOS DeviceSupport",
            "Xcode device support",
            AppleDevelopment,
            ToolManaged,
            "Debug support is retained for connected iOS versions.",
            "Remove only obsolete OS versions through Xcode-aware maintenance.",
            "Physical-device debugging may need to download support again.",
        ),
        spec(
            "Library/Android/sdk",
            "Android SDK and system images",
            AndroidDevelopment,
            Protected,
            "This is an installed development toolchain, not garbage.",
            "Use Android Studio SDK Manager to uninstall only specific unused images or NDK versions.",
            "Flutter and Android builds fail if required SDK components are removed.",
        ),
        spec(
            ".cache/uv",
            "uv package cache",
            PackageCache,
            Reviewable,
            "Downloaded Python packages and build artifacts are reproducible.",
            "Use `uv cache prune` after confirming no environment setup is running.",
            "Future environments may download packages again.",
        ),
        spec(
            ".cache/huggingface",
            "Hugging Face model cache",
            PackageCache,
            ToolManaged,
            "Downloaded models can be essential for offline work.",
            "Review models in the owning tool and remove only ones you can re-download.",
            "Models may require large downloads.",
        ),
        spec(
            ".cache/puppeteer",
            "Puppeteer browser cache",
            PackageCache,
            Reviewable,
            "Downloaded browser builds can be restored by Puppeteer.",
            "Use Puppeteer's supported browser/cache command when no browser job is active.",
            "Browser binaries will download again.",
        ),
        spec(
            ".gradle",
            "Gradle user home",
            PackageCache,
            ToolManaged,
            "Contains caches plus configuration, wrappers, and toolchains.",
            "Let Gradle perform its own retention; never delete the whole directory.",
            "Raw deletion can break builds and remove configuration.",
        ),
        spec(
            ".npm/_cacache",
            "npm content cache",
            PackageCache,
            Reviewable,
            "npm's content-addressed cache can be verified and rebuilt.",
            "Use `npm cache verify`; clean only after review if space is urgent.",
            "Packages will download again.",
        ),
        spec(
            ".npm/_npx",
            "npx package cache",
            PackageCache,
            Reviewable,
            "Temporary npx-installed packages can be recreated.",
            "Review cached tools and clear only when no npx job is running.",
            "npx tools will download again.",
        ),
        spec(
            ".yarn/berry/cache",
            "Yarn global package cache",
            PackageCache,
            Reviewable,
            "Yarn archives can regrow quickly during JavaScript development.",
            "Use Yarn's supported cache workflow only when package jobs are idle.",
            "Dependencies may need to download again.",
        ),
        spec(
            ".pub-cache",
            "Dart and Flutter package cache",
            PackageCache,
            Protected,
            "Contains packages and global activation state used by Flutter projects.",
            "Prefer `dart pub cache gc` after reviewing affected projects; never raw-delete it.",
            "Projects may fail offline and need `pub get` again.",
        ),
        spec(
            ".cargo",
            "Cargo home",
            PackageCache,
            Protected,
            "Contains Rust package downloads, installed tools, and configuration.",
            "Use Cargo-aware maintenance and keep installed binaries and configuration.",
            "Rust builds or installed developer tools can fail after raw deletion.",
        ),
        spec(
            ".rustup",
            "Rust toolchains",
            PackageCache,
            Protected,
            "Installed Rust compilers and standard libraries are development tools.",
            "Use rustup to uninstall only toolchains you have confirmed are unused.",
            "Rust projects can stop compiling.",
        ),
        spec(
            "Library/Caches",
            "Application caches",
            ApplicationSupport,
            ToolManaged,
            "Application-owned caches can regrow and may contain active browser or developer state.",
            "Inspect child applications and use their supported maintenance; never bulk-delete this directory.",
            "Applications may lose offline state or immediately download data again.",
        ),
        spec(
            "Library/Application Support/Claude/vm_bundles",
            "Claude application runtime",
            ApplicationSupport,
            Protected,
            "Application runtime state is not a generic cache.",
            "Use Claude's supported maintenance or reinstall flow only.",
            "Raw deletion can stop Claude from starting.",
        ),
        spec(
            ".claude",
            "Claude Code data",
            ApplicationSupport,
            Protected,
            "Claude Code keeps session transcripts, settings and project memory here.",
            "Review inside Claude Code; Mac Cleaner will not delete this directory.",
            "Deleted sessions and memory cannot be restored.",
        ),
        spec(
            ".codex/sessions",
            "Codex session history",
            ApplicationSupport,
            Protected,
            "Session history can include the active conversation and development context.",
            "Review old sessions individually; Mac Cleaner will not delete this directory.",
            "Deleted sessions and their context cannot be restored.",
        ),
        spec(
            "Library/Application Support/Google/Chrome/OptGuideOnDeviceModel",
            "Chrome on-device model",
            ApplicationSupport,
            ToolManaged,
            "Chrome manages this downloaded model inside Application Support.",
            "Review Chrome's settings and components; do not treat Application Support as cache.",
            "Chrome may download the model again.",
        ),
    ]
}

fn spec(
    path: &str,
    name: &str,
    category: StorageCategory,
    disposition: StorageDisposition,
    reason: &str,
    recommendation: &str,
    cost: &str,
) -> HotspotSpec {
    spec_abs(
        PathBuf::from(path),
        name,
        category,
        disposition,
        reason,
        recommendation,
        cost,
    )
}

fn spec_abs(
    path: PathBuf,
    name: &str,
    category: StorageCategory,
    disposition: StorageDisposition,
    reason: &str,
    recommendation: &str,
    cost: &str,
) -> HotspotSpec {
    HotspotSpec {
        path,
        display_name: name.into(),
        category,
        disposition,
        reason: reason.into(),
        recommendation: recommendation.into(),
        recovery_cost: cost.into(),
    }
}

/// The per-user macOS temporary and cache area (`/var/folders/..`). Compilers, Xcode and
/// installers write large short-lived files here, so growth here is otherwise invisible.
pub(crate) fn system_temp_spec() -> Option<HotspotSpec> {
    let user_root = std::env::temp_dir().parent()?.to_path_buf();
    // Only the per-user folder under /var/folders is ever measured.
    if !user_root.starts_with("/var/folders") && !user_root.starts_with("/private/var/folders") {
        return None;
    }
    Some(spec_abs(
        user_root,
        "System temporary files",
        StorageCategory::ApplicationSupport,
        StorageDisposition::Protected,
        "macOS and developer tools write temporary build and installer files here; most are removed on reboot.",
        "Restarting the Mac clears most of it. Mac Cleaner only reports it.",
        "Managed by macOS; no action in Mac Cleaner.",
    ))
}
