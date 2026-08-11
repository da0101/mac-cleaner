use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use cleaner_domain::{
    CATALOG_VERSION, CandidateId, CleanupCandidate, ScanReport, ScanSkip, TargetId,
};
use cleaner_policy::{TargetSpec, catalog, catalog_root, is_protected, is_safe_relative};
use sha2::{Digest, Sha256};

use crate::FsError;
use crate::metadata::{
    DEFAULT_MAX_ENTRIES, identity_from_metadata, metadata_no_follow, summarize_tree,
    unsafe_candidate, validate_home_state, validate_metadata,
};

#[derive(Clone, Debug)]
pub struct Scanner {
    home: PathBuf,
    max_entries: u64,
}

impl Scanner {
    #[must_use]
    pub fn new(home: PathBuf) -> Self {
        Self {
            home,
            max_entries: DEFAULT_MAX_ENTRIES,
        }
    }

    #[must_use]
    pub fn with_max_entries(mut self, max_entries: u64) -> Self {
        self.max_entries = max_entries;
        self
    }

    pub fn scan(&self) -> Result<ScanReport, FsError> {
        self.scan_at(Utc::now())
    }

    pub fn scan_at(&self, now: DateTime<Utc>) -> Result<ScanReport, FsError> {
        validate_home_state(&self.home)?;
        let started_at = Utc::now();
        let canonical_home = self.home.canonicalize().map_err(|source| FsError::Io {
            path: self.home.clone(),
            source,
        })?;
        let home_device = metadata_no_follow(&self.home)?.dev();
        let mut candidates = Vec::new();
        let mut skipped = Vec::new();

        for target in catalog() {
            let root = catalog_root(&self.home, target)?;
            if !root.exists() {
                continue;
            }
            let root_metadata = metadata_no_follow(&root)?;
            if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
                skipped.push(skip(target, &root, "catalog root is not a real directory"));
                continue;
            }
            if root_metadata.dev() != home_device {
                skipped.push(skip(
                    target,
                    &root,
                    "catalog root crosses a filesystem boundary",
                ));
                continue;
            }
            let resolved_root = match root.canonicalize() {
                Ok(path) if path.starts_with(&canonical_home) => path,
                Ok(_) => {
                    skipped.push(skip(
                        target,
                        &root,
                        "catalog root resolves outside the home",
                    ));
                    continue;
                }
                Err(error) => {
                    skipped.push(skip(
                        target,
                        &root,
                        &format!("cannot resolve root: {error}"),
                    ));
                    continue;
                }
            };

            let entries = fs::read_dir(&root).map_err(|source| FsError::Io {
                path: root.clone(),
                source,
            })?;
            for entry in entries {
                let entry = entry.map_err(|source| FsError::Io {
                    path: root.clone(),
                    source,
                })?;
                match self.inspect_entry(target, &root, &resolved_root, home_device, entry.path()) {
                    Ok(Some(candidate)) if is_old_enough(&candidate, target, now) => {
                        candidates.push(candidate);
                    }
                    Ok(_) => {}
                    Err(FsError::UnsafeCandidate { path, reason }) => {
                        skipped.push(skip(target, &path, &reason));
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.bytes));
        Ok(ScanReport {
            started_at,
            completed_at: Utc::now(),
            candidates,
            skipped,
        })
    }

    fn inspect_entry(
        &self,
        target: &TargetSpec,
        root: &Path,
        resolved_root: &Path,
        device: u64,
        path: PathBuf,
    ) -> Result<Option<CleanupCandidate>, FsError> {
        let relative = path
            .strip_prefix(root)
            .map_err(|_| unsafe_candidate(&path, "entry is outside its catalog root"))?
            .to_path_buf();
        if !is_safe_relative(&relative) || relative.components().count() != 1 {
            return Err(unsafe_candidate(&path, "entry is not one direct child"));
        }
        if is_protected(&self.home, &path) {
            return Err(unsafe_candidate(&path, "entry intersects a protected path"));
        }

        let metadata = metadata_no_follow(&path)?;
        validate_metadata(&path, &metadata, device)?;
        let resolved = path.canonicalize().map_err(|source| FsError::Io {
            path: path.clone(),
            source,
        })?;
        if !resolved.starts_with(resolved_root) {
            return Err(unsafe_candidate(
                &path,
                "entry resolves outside its catalog root",
            ));
        }

        let summary = summarize_tree(&path, device, self.max_entries)?;
        Ok(Some(CleanupCandidate {
            id: stable_candidate_id(target.id, relative.as_os_str()),
            target_id: TargetId(target.id.to_owned()),
            display_name: format!("{} — {}", target.display_name, relative.to_string_lossy()),
            lexical_path: path,
            resolved_path: resolved,
            relative_entry: relative,
            category: target.category.to_owned(),
            reason: target.reason.to_owned(),
            owner: target.owner.to_owned(),
            recovery: target.recovery.to_owned(),
            recovery_mode: target.recovery_mode,
            safety_tier: target.safety_tier,
            bytes: summary.bytes,
            files: summary.files,
            last_modified: DateTime::<Utc>::from(summary.newest_modified),
            identity: identity_from_metadata(&metadata)?,
            catalog_version: CATALOG_VERSION,
        }))
    }
}

fn stable_candidate_id(target_id: &str, relative: &OsStr) -> CandidateId {
    let mut digest = Sha256::new();
    digest.update(target_id.as_bytes());
    digest.update([0]);
    digest.update(relative.as_bytes());
    CandidateId(format!("{target_id}:{:x}", digest.finalize()))
}

fn is_old_enough(candidate: &CleanupCandidate, target: &TargetSpec, now: DateTime<Utc>) -> bool {
    let Ok(minimum_age) = chrono::Duration::from_std(target.minimum_age) else {
        return false;
    };
    candidate.last_modified <= now - minimum_age
}

fn skip(target: &TargetSpec, path: &Path, reason: &str) -> ScanSkip {
    ScanSkip {
        target_id: TargetId(target.id.to_owned()),
        path: path.to_path_buf(),
        reason: reason.to_owned(),
    }
}
