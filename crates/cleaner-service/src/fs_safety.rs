use std::os::unix::fs::MetadataExt;
use std::path::Path;

use cleaner_domain::{FileIdentity, FileKind};

use crate::ServiceError;

pub(super) fn create_private_directory(path: &Path) -> Result<(), ServiceError> {
    std::fs::create_dir_all(path).map_err(|source| ServiceError::DataDirectory {
        path: path.to_path_buf(),
        source,
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).map_err(
            |source| ServiceError::DataDirectory {
                path: path.to_path_buf(),
                source,
            },
        )?;
    }
    Ok(())
}

pub(super) fn ensure_same_identity(
    current: &FileIdentity,
    expected: &FileIdentity,
    label: &str,
) -> Result<(), ServiceError> {
    if current.device != expected.device
        || current.inode != expected.inode
        || current.kind != expected.kind
        || current.size != expected.size
        || current.modified_unix_nanos != expected.modified_unix_nanos
        || current.link_count != expected.link_count
    {
        return Err(ServiceError::UnsafeToolTarget(format!(
            "{label} identity changed after preview"
        )));
    }
    Ok(())
}

pub(super) fn file_identity(path: &Path) -> Result<FileIdentity, ServiceError> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|source| ServiceError::ToolExecution {
            path: path.to_path_buf(),
            source,
        })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(ServiceError::UnsafeToolTarget(format!(
            "{} is not a real file",
            path.display()
        )));
    }
    Ok(identity_from_metadata(&metadata, FileKind::File))
}

pub(super) fn directory_identity(path: &Path) -> Result<FileIdentity, ServiceError> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|source| ServiceError::ToolExecution {
            path: path.to_path_buf(),
            source,
        })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(ServiceError::UnsafeToolTarget(format!(
            "{} is not a real directory",
            path.display()
        )));
    }
    Ok(identity_from_metadata(&metadata, FileKind::Directory))
}

pub(super) fn identity_from_metadata(metadata: &std::fs::Metadata, kind: FileKind) -> FileIdentity {
    FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        size: metadata.len(),
        modified_unix_nanos: metadata
            .mtime()
            .saturating_mul(1_000_000_000)
            .saturating_add(metadata.mtime_nsec()),
        link_count: metadata.nlink(),
        kind,
    }
}

pub(super) fn directory_allocated_bytes(path: &Path, device: u64) -> Result<u64, ServiceError> {
    let mut stack = vec![path.to_path_buf()];
    let mut entries = 0_u64;
    let mut bytes = 0_u64;
    while let Some(current) = stack.pop() {
        entries = entries.saturating_add(1);
        if entries > 2_000_000 {
            return Err(ServiceError::UnsafeToolTarget(
                "uv cache exceeds the entry safety limit".to_owned(),
            ));
        }
        let metadata =
            std::fs::symlink_metadata(&current).map_err(|source| ServiceError::ToolExecution {
                path: current.clone(),
                source,
            })?;
        if metadata.file_type().is_symlink() || metadata.dev() != device {
            continue;
        }
        bytes = bytes.saturating_add(metadata.blocks().saturating_mul(512));
        if metadata.is_dir() {
            let children =
                std::fs::read_dir(&current).map_err(|source| ServiceError::ToolExecution {
                    path: current.clone(),
                    source,
                })?;
            for child in children {
                let child = child.map_err(|source| ServiceError::ToolExecution {
                    path: current.clone(),
                    source,
                })?;
                stack.push(child.path());
            }
        }
    }
    Ok(bytes)
}
