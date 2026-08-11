use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use cleaner_domain::{ToolCleanupKind, ToolCleanupOperation, ToolCleanupPlan, ToolCleanupReceipt};
use uuid::Uuid;

use crate::ServiceError;
use crate::fs_safety::{directory_allocated_bytes, directory_identity, ensure_same_identity};

pub(super) fn find_executable() -> Result<PathBuf, ServiceError> {
    for candidate in [
        Path::new("/opt/homebrew/bin/uv"),
        Path::new("/usr/local/bin/uv"),
    ] {
        if let Ok(resolved) = candidate.canonicalize()
            && resolved.is_file()
            && (resolved.starts_with("/opt/homebrew/Cellar/uv")
                || resolved.starts_with("/usr/local/Cellar/uv"))
        {
            return Ok(resolved);
        }
    }
    Err(ServiceError::UnsafeToolTarget(
        "a trusted Homebrew uv executable was not found".to_owned(),
    ))
}

pub(super) fn validate(home: &Path, plan: &ToolCleanupPlan) -> Result<(), ServiceError> {
    let ToolCleanupOperation::Command {
        executable_path,
        arguments,
    } = &plan.operation
    else {
        return Err(ServiceError::UnsafeToolTarget(
            "uv cleanup is not a compiled command plan".to_owned(),
        ));
    };
    let expected_arguments = vec![
        "cache".to_owned(),
        "prune".to_owned(),
        "--cache-dir".to_owned(),
        plan.target_path.to_string_lossy().into_owned(),
        "--color".to_owned(),
        "never".to_owned(),
        "--no-progress".to_owned(),
        "--no-config".to_owned(),
    ];
    if plan.kind != ToolCleanupKind::UvCachePrune
        || plan.target_path != home.join(".cache/uv")
        || *arguments != expected_arguments
        || !executable_path.is_file()
        || !(executable_path.starts_with("/opt/homebrew/Cellar/uv")
            || executable_path.starts_with("/usr/local/Cellar/uv"))
    {
        return Err(ServiceError::UnsafeToolTarget(
            "uv plan differs from the compiled command contract".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn execute(
    plan_id: Uuid,
    plan: ToolCleanupPlan,
    home: &Path,
) -> Result<ToolCleanupReceipt, ServiceError> {
    let ToolCleanupOperation::Command {
        executable_path,
        arguments,
    } = &plan.operation
    else {
        return Err(ServiceError::UnsafeToolTarget(
            "uv operation changed".to_owned(),
        ));
    };
    let current_identity = directory_identity(&plan.target_path)?;
    ensure_same_identity(&current_identity, &plan.target_identity, "uv cache root")?;
    let bytes_before = directory_allocated_bytes(&plan.target_path, current_identity.device)?;
    let output = Command::new(executable_path)
        .args(arguments)
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin")
        .output()
        .map_err(|source| ServiceError::ToolExecution {
            path: executable_path.clone(),
            source,
        })?;
    if !output.status.success() {
        return Err(ServiceError::ToolFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    let bytes_after = if plan.target_path.exists() {
        directory_allocated_bytes(&plan.target_path, current_identity.device)?
    } else {
        0
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut summary = [stderr.trim(), stdout.trim()]
        .into_iter()
        .find(|line| !line.is_empty())
        .unwrap_or("uv cache prune completed")
        .to_owned();
    summary.truncate(2_000);
    Ok(ToolCleanupReceipt {
        plan_id,
        kind: ToolCleanupKind::UvCachePrune,
        target_path: plan.target_path,
        bytes_before,
        bytes_after,
        reclaimed_bytes: bytes_before.saturating_sub(bytes_after),
        completed_at: Utc::now(),
        summary,
    })
}
