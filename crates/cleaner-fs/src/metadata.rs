use std::ffi::OsStr;
use std::fs::{self, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use cleaner_domain::{FileIdentity, FileKind};
use cleaner_policy::{protected_sentinels, validate_home};

use crate::FsError;

pub(crate) const DEFAULT_MAX_ENTRIES: u64 = 250_000;

#[derive(Debug)]
pub(crate) struct TreeSummary {
    pub bytes: u64,
    pub files: u64,
    pub newest_modified: SystemTime,
}

pub(crate) fn summarize_tree(
    root: &Path,
    device: u64,
    max_entries: u64,
) -> Result<TreeSummary, FsError> {
    let mut stack = vec![root.to_path_buf()];
    let mut bytes = 0_u64;
    let mut files = 0_u64;
    let mut newest_modified = UNIX_EPOCH;

    while let Some(path) = stack.pop() {
        if files >= max_entries {
            return Err(unsafe_candidate(root, "entry count exceeds the scan limit"));
        }
        if has_protected_name(&path) {
            return Err(unsafe_candidate(
                root,
                "tree contains a protected project sentinel",
            ));
        }
        let metadata = metadata_no_follow(&path)?;
        validate_metadata(&path, &metadata, device)?;
        files = files.saturating_add(1);
        bytes = bytes.saturating_add(metadata.blocks().saturating_mul(512));
        let modified = metadata.modified().map_err(|source| FsError::Io {
            path: path.clone(),
            source,
        })?;
        newest_modified = newest_modified.max(modified);

        if metadata.is_dir() {
            let entries = fs::read_dir(&path).map_err(|source| FsError::Io {
                path: path.clone(),
                source,
            })?;
            for entry in entries {
                stack.push(
                    entry
                        .map_err(|source| FsError::Io {
                            path: path.clone(),
                            source,
                        })?
                        .path(),
                );
            }
        }
    }

    Ok(TreeSummary {
        bytes,
        files,
        newest_modified,
    })
}

pub(crate) fn validate_home_state(home: &Path) -> Result<(), FsError> {
    validate_home(home)?;
    let metadata = metadata_no_follow(home)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(FsError::UnsafeHome(home.to_path_buf()));
    }
    Ok(())
}

pub(crate) fn validate_metadata(
    path: &Path,
    metadata: &Metadata,
    device: u64,
) -> Result<(), FsError> {
    if metadata.file_type().is_symlink() {
        return Err(unsafe_candidate(
            path,
            "symbolic links are never cleanup candidates",
        ));
    }
    if metadata.dev() != device {
        return Err(unsafe_candidate(path, "path crosses a filesystem boundary"));
    }
    if metadata.nlink() > 1 && metadata.is_file() {
        return Err(unsafe_candidate(
            path,
            "hard-linked files are never cleanup candidates",
        ));
    }
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(unsafe_candidate(
            path,
            "special filesystem objects are protected",
        ));
    }
    Ok(())
}

pub(crate) fn metadata_no_follow(path: &Path) -> Result<Metadata, FsError> {
    fs::symlink_metadata(path).map_err(|source| FsError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn identity_from_metadata(metadata: &Metadata) -> Result<FileIdentity, FsError> {
    let modified = metadata.modified().map_err(|_| FsError::InvalidTimestamp)?;
    let modified_nanos = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|_| FsError::InvalidTimestamp)?
        .as_nanos();
    let modified_unix_nanos =
        i64::try_from(modified_nanos).map_err(|_| FsError::InvalidTimestamp)?;
    Ok(FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        size: metadata.size(),
        modified_unix_nanos,
        link_count: metadata.nlink(),
        kind: if metadata.is_dir() {
            FileKind::Directory
        } else {
            FileKind::File
        },
    })
}

pub(crate) fn unsafe_candidate(path: &Path, reason: &str) -> FsError {
    FsError::UnsafeCandidate {
        path: path.to_path_buf(),
        reason: reason.to_owned(),
    }
}

fn has_protected_name(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        protected_sentinels()
            .iter()
            .any(|protected| name == OsStr::new(protected))
    })
}
