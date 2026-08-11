use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use cleaner_domain::{FileIdentity, FileKind, ToolCleanupFile};
use serde::Deserialize;

use crate::ServiceError;
use crate::fs_safety::{directory_identity, identity_from_metadata};

#[derive(Debug)]
pub(crate) struct RecordingTarget {
    pub(crate) path: PathBuf,
    pub(crate) identity: FileIdentity,
    pub(crate) device_udid: String,
    pub(crate) device_name: String,
    pub(crate) device_was_booted: bool,
    pub(crate) files: Vec<ToolCleanupFile>,
    pub(crate) bytes: u64,
}

#[derive(Debug, Deserialize)]
struct SimctlDeviceList {
    devices: HashMap<String, Vec<SimctlDevice>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SimctlDevice {
    udid: String,
    name: String,
    state: String,
    data_path: PathBuf,
}

pub(crate) fn discover(
    home: &Path,
    minimum_age: Duration,
) -> Result<RecordingTarget, ServiceError> {
    let list = simulator_device_list()?;
    let devices_root = home.join("Library/Developer/CoreSimulator/Devices");
    let now = SystemTime::now();
    let mut best: Option<RecordingTarget> = None;
    for device in list.devices.into_values().flatten() {
        if !valid_data_path(&devices_root, &device) {
            continue;
        }
        let Some(path) = find_testmanager_attachments(&device.data_path)? else {
            continue;
        };
        let identity = directory_identity(&path)?;
        let mut files = Vec::new();
        let children = std::fs::read_dir(&path).map_err(|source| ServiceError::ToolExecution {
            path: path.clone(),
            source,
        })?;
        for child in children {
            let child = child.map_err(|source| ServiceError::ToolExecution {
                path: path.clone(),
                source,
            })?;
            let Some(name) = child.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !safe_relative_file_name(&name) {
                continue;
            }
            let file_path = child.path();
            let Ok(metadata) = std::fs::symlink_metadata(&file_path) else {
                continue;
            };
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.dev() != identity.device
                || metadata.nlink() != 1
                || metadata.len() == 0
            {
                continue;
            }
            let modified = metadata.modified().unwrap_or(SystemTime::now());
            if now.duration_since(modified).unwrap_or_default() < minimum_age
                || !is_quicktime_file(&file_path)?
            {
                continue;
            }
            files.push(ToolCleanupFile {
                relative_name: name,
                identity: identity_from_metadata(&metadata, FileKind::File),
                allocated_bytes: metadata.blocks().saturating_mul(512),
            });
        }
        files.sort_by(|left, right| left.relative_name.cmp(&right.relative_name));
        let bytes = files
            .iter()
            .fold(0_u64, |sum, file| sum.saturating_add(file.allocated_bytes));
        if bytes == 0 {
            continue;
        }
        let target = RecordingTarget {
            path,
            identity,
            device_udid: device.udid,
            device_name: device.name,
            device_was_booted: device.state == "Booted",
            files,
            bytes,
        };
        if best
            .as_ref()
            .is_none_or(|current| target.bytes > current.bytes)
        {
            best = Some(target);
        }
    }
    best.ok_or_else(|| {
        ServiceError::UnsafeToolTarget(
            "no verified simulator test recordings older than 72 hours were found".to_owned(),
        )
    })
}

fn valid_data_path(devices_directory: &Path, device: &SimctlDevice) -> bool {
    let Some(device_root) = device.data_path.parent() else {
        return false;
    };
    device
        .data_path
        .file_name()
        .is_some_and(|name| name == "data")
        && device_root.parent() == Some(devices_directory)
        && device_root
            .file_name()
            .is_some_and(|name| name == device.udid.as_str())
}

fn simulator_device_list() -> Result<SimctlDeviceList, ServiceError> {
    let output = Command::new("/usr/bin/xcrun")
        .args(["simctl", "list", "devices", "--json"])
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
    serde_json::from_slice(&output.stdout).map_err(ServiceError::SimulatorMetadata)
}

pub(super) fn simulator_device_state(udid: &str) -> Result<String, ServiceError> {
    simulator_device_list()?
        .devices
        .into_values()
        .flatten()
        .find(|device| device.udid == udid)
        .map(|device| device.state)
        .ok_or_else(|| {
            ServiceError::UnsafeToolTarget(format!("simulator {udid} is no longer available"))
        })
}

fn find_testmanager_attachments(data_path: &Path) -> Result<Option<PathBuf>, ServiceError> {
    let internal = data_path.join("Containers/Data/InternalDaemon");
    let Ok(children) = std::fs::read_dir(&internal) else {
        return Ok(None);
    };
    for child in children {
        let child = child.map_err(|source| ServiceError::ToolExecution {
            path: internal.clone(),
            source,
        })?;
        let container = child.path();
        let Ok(metadata) = std::fs::symlink_metadata(&container) else {
            continue;
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        if metadata_identifier_is_testmanager(&container)? {
            let attachments = container.join("Attachments");
            if directory_identity(&attachments).is_ok() {
                return Ok(Some(attachments));
            }
        }
    }
    Ok(None)
}

pub(super) fn metadata_identifier_is_testmanager(container: &Path) -> Result<bool, ServiceError> {
    let metadata = container.join(".com.apple.mobile_container_manager.metadata.plist");
    let file_metadata = match std::fs::symlink_metadata(&metadata) {
        Ok(value) if value.is_file() && !value.file_type().is_symlink() => value,
        _ => return Ok(false),
    };
    if file_metadata.len() > 1_000_000 {
        return Ok(false);
    }
    let output = Command::new("/usr/bin/plutil")
        .args(["-extract", "MCMMetadataIdentifier", "raw", "-o", "-"])
        .arg(&metadata)
        .env_clear()
        .output()
        .map_err(|source| ServiceError::ToolExecution {
            path: PathBuf::from("/usr/bin/plutil"),
            source,
        })?;
    Ok(output.status.success()
        && String::from_utf8_lossy(&output.stdout).trim() == "com.apple.testmanagerd")
}

pub(super) fn safe_relative_file_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && !name.starts_with('.')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && Path::new(name).components().count() == 1
}

pub(super) fn is_quicktime_file(path: &Path) -> Result<bool, ServiceError> {
    let mut file = File::open(path).map_err(|source| ServiceError::ToolExecution {
        path: path.to_path_buf(),
        source,
    })?;
    let mut header = [0_u8; 16];
    let read = file
        .read(&mut header)
        .map_err(|source| ServiceError::ToolExecution {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(read >= 12
        && (&header[4..8] == b"ftyp"
            || (read >= 16 && &header[4..8] == b"wide" && &header[12..16] == b"mdat")))
}

#[cfg(test)]
mod tests {
    use super::{is_quicktime_file, safe_relative_file_name};

    #[test]
    fn simulator_manifest_accepts_only_single_safe_file_names() {
        assert!(safe_relative_file_name("93F138E0-9433.sb-test"));
        assert!(!safe_relative_file_name("../recording"));
        assert!(!safe_relative_file_name("folder/recording"));
        assert!(!safe_relative_file_name(".hidden"));
    }

    #[test]
    fn quicktime_signature_recognizes_complete_and_unfinished_recordings()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let complete = directory.path().join("complete");
        let unfinished = directory.path().join("unfinished");
        let unrelated = directory.path().join("unrelated");
        std::fs::write(&complete, b"\0\0\0\x18ftypqt  \0\0\0\0")?;
        std::fs::write(&unfinished, b"\0\0\0\x08wide\0\0\0\0mdat")?;
        std::fs::write(&unrelated, b"personal document")?;
        assert!(is_quicktime_file(&complete)?);
        assert!(is_quicktime_file(&unfinished)?);
        assert!(!is_quicktime_file(&unrelated)?);
        Ok(())
    }
}
