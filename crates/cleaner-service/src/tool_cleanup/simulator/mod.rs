mod discovery;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use chrono::Utc;
use cleaner_domain::{
    FileKind, ToolCleanupKind, ToolCleanupOperation, ToolCleanupPlan, ToolCleanupReceipt,
};
use cleaner_store::Store;
use uuid::Uuid;

use crate::ServiceError;
use crate::fs_safety::{directory_identity, ensure_same_identity, file_identity};

pub(crate) use discovery::discover;
use discovery::{
    is_quicktime_file, metadata_identifier_is_testmanager, safe_relative_file_name,
    simulator_device_state,
};

pub(super) fn validate(home: &Path, plan: &ToolCleanupPlan) -> Result<(), ServiceError> {
    let ToolCleanupOperation::SimulatorAttachments {
        device_udid, files, ..
    } = &plan.operation
    else {
        return Err(ServiceError::UnsafeToolTarget(
            "simulator cleanup is not an exact attachment plan".to_owned(),
        ));
    };
    if plan.kind != ToolCleanupKind::SimulatorTestRecordings
        || files.is_empty()
        || files.len() > 10_000
        || plan.target_identity.kind != FileKind::Directory
    {
        return Err(ServiceError::UnsafeToolTarget(
            "simulator attachment plan has invalid bounds".to_owned(),
        ));
    }
    let devices_root = home.join("Library/Developer/CoreSimulator/Devices");
    let relative = plan.target_path.strip_prefix(&devices_root).map_err(|_| {
        ServiceError::UnsafeToolTarget("attachment root escaped CoreSimulator".to_owned())
    })?;
    let components = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if components.len() != 7
        || components[0] != *device_udid
        || components[1..5] != ["data", "Containers", "Data", "InternalDaemon"]
        || components[6] != "Attachments"
    {
        return Err(ServiceError::UnsafeToolTarget(
            "attachment root differs from the compiled simulator layout".to_owned(),
        ));
    }
    let container = plan.target_path.parent().ok_or_else(|| {
        ServiceError::UnsafeToolTarget("attachment container has no parent".to_owned())
    })?;
    if !metadata_identifier_is_testmanager(container)? {
        return Err(ServiceError::UnsafeToolTarget(
            "attachment container is not owned by com.apple.testmanagerd".to_owned(),
        ));
    }
    let total = files.iter().try_fold(0_u64, |sum, item| {
        if !safe_relative_file_name(&item.relative_name)
            || item.identity.kind != FileKind::File
            || item.identity.link_count != 1
            || item.identity.device != plan.target_identity.device
        {
            return Err(ServiceError::UnsafeToolTarget(
                "simulator attachment manifest contains an unsafe file".to_owned(),
            ));
        }
        Ok(sum.saturating_add(item.allocated_bytes))
    })?;
    if total != plan.bytes_before {
        return Err(ServiceError::UnsafeToolTarget(
            "simulator attachment byte manifest changed".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn execute(
    plan_id: Uuid,
    plan: ToolCleanupPlan,
    store: &Store,
) -> Result<ToolCleanupReceipt, ServiceError> {
    let ToolCleanupOperation::SimulatorAttachments {
        device_udid,
        device_was_booted,
        files,
        ..
    } = &plan.operation
    else {
        return Err(ServiceError::UnsafeToolTarget(
            "simulator operation changed".to_owned(),
        ));
    };
    if let Some(job) = active_test_job()? {
        return Err(ServiceError::ActiveDeveloperJob(job));
    }
    let state = simulator_device_state(device_udid)?;
    if state == "Booted" {
        if !device_was_booted {
            return Err(ServiceError::UnsafeToolTarget(
                "simulator booted after preview; run the cleanup again".to_owned(),
            ));
        }
        shutdown_device(device_udid)?;
        wait_for_shutdown(device_udid)?;
    } else if state != "Shutdown" {
        return Err(ServiceError::UnsafeToolTarget(format!(
            "simulator is in unexpected state {state}"
        )));
    }
    if let Some(job) = active_test_job()? {
        return Err(ServiceError::ActiveDeveloperJob(job));
    }
    let root_identity = directory_identity(&plan.target_path)?;
    ensure_same_identity(
        &root_identity,
        &plan.target_identity,
        "simulator attachment root",
    )?;
    let mut reclaimed = 0_u64;
    for item in files {
        let path = plan.target_path.join(&item.relative_name);
        let current = file_identity(&path)?;
        ensure_same_identity(&current, &item.identity, "simulator recording")?;
        if current.link_count != 1 || !is_quicktime_file(&path)? {
            return Err(ServiceError::UnsafeToolTarget(format!(
                "recording changed after preview: {}",
                item.relative_name
            )));
        }
        store.append_tool_cleanup_detail(
            plan_id,
            &format!("removing verified test recording {}", item.relative_name),
        )?;
        std::fs::remove_file(&path).map_err(|source| ServiceError::ToolExecution {
            path: path.clone(),
            source,
        })?;
        if path.exists() {
            return Err(ServiceError::UnsafeToolTarget(format!(
                "recording still exists after deletion: {}",
                item.relative_name
            )));
        }
        reclaimed = reclaimed.saturating_add(item.allocated_bytes);
    }
    let count = files.len();
    Ok(ToolCleanupReceipt {
        plan_id,
        kind: ToolCleanupKind::SimulatorTestRecordings,
        target_path: plan.target_path,
        bytes_before: plan.bytes_before,
        bytes_after: plan.bytes_before.saturating_sub(reclaimed),
        reclaimed_bytes: reclaimed,
        completed_at: Utc::now(),
        summary: format!("Removed {count} verified old simulator test recordings"),
    })
}

fn active_test_job() -> Result<Option<String>, ServiceError> {
    for process in ["xcodebuild", "xctest", "XCTest"] {
        let status = Command::new("/usr/bin/pgrep")
            .args(["-x", process])
            .env_clear()
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|source| ServiceError::ToolExecution {
                path: PathBuf::from("/usr/bin/pgrep"),
                source,
            })?;
        if status.success() {
            return Ok(Some(process.to_owned()));
        }
    }
    Ok(None)
}

fn shutdown_device(device_udid: &str) -> Result<(), ServiceError> {
    let output = Command::new("/usr/bin/xcrun")
        .args(["simctl", "shutdown", device_udid])
        .env_clear()
        .output()
        .map_err(|source| ServiceError::ToolExecution {
            path: PathBuf::from("/usr/bin/xcrun"),
            source,
        })?;
    if !output.status.success() {
        return Err(ServiceError::ToolFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(())
}

fn wait_for_shutdown(udid: &str) -> Result<(), ServiceError> {
    for _ in 0..50 {
        if simulator_device_state(udid)? == "Shutdown" {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(ServiceError::ToolFailed(
        "simulator did not finish shutting down within five seconds".to_owned(),
    ))
}
