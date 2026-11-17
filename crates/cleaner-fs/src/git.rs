//! Read-only Git queries used to keep committed folders out of cleanup.

use std::path::Path;
use std::process::{Command, Stdio};

const GIT: &str = "/usr/bin/git";

/// True only when `entry` (relative to `project_root`) is covered by the
/// project's ignore rules and Git tracks no file inside it.
///
/// Every failure (Git missing, not a repository, unreadable index) counts as
/// "not ignored", so a candidate is dropped rather than risked.
pub(crate) fn is_ignored_and_untracked(project_root: &Path, entry: &Path) -> bool {
    let ignored = Command::new(GIT)
        .arg("-C")
        .arg(project_root)
        .args(["check-ignore", "-q", "--"])
        .arg(entry)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !ignored {
        return false;
    }
    Command::new(GIT)
        .arg("-C")
        .arg(project_root)
        .args(["ls-files", "--"])
        .arg(entry)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .is_ok_and(|output| output.status.success() && output.stdout.is_empty())
}
