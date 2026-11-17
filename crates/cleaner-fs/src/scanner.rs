use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use cleaner_domain::{
    CATALOG_VERSION, CandidateId, CleanupCandidate, ScanReport, ScanSkip, TargetId,
};
use cleaner_policy::{
    TargetLayout, TargetSpec, candidate_path, catalog, catalog_root, marker_present,
};
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

            for path in discover_entries(target, &root, home_device)? {
                match self.inspect_entry(target, &root, &resolved_root, home_device, path) {
                    Ok(Some(candidate)) if is_old_enough(&candidate, target, now) => {
                        candidates.push(candidate);
                    }
                    Ok(Some(candidate)) => {
                        let days = target.minimum_age.as_secs() / 86_400;
                        skipped.push(skip(
                            target,
                            &candidate.lexical_path,
                            &format!("used within the last {days} days"),
                        ));
                    }
                    Ok(None) => {}
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
        match candidate_path(&self.home, target, &relative) {
            Ok(expected) if expected == path => {}
            _ => {
                return Err(unsafe_candidate(
                    &path,
                    "entry does not match its catalog target",
                ));
            }
        }

        if target.require_git_ignored
            && let TargetLayout::Project { entry, .. } = target.layout
        {
            let project_root = path
                .ancestors()
                .nth(target.entry_depth())
                .map(Path::to_path_buf)
                .unwrap_or_default();
            if !crate::git::is_ignored_and_untracked(&project_root, Path::new(entry)) {
                return Err(unsafe_candidate(
                    &path,
                    "Git does not ignore it, or tracks files inside it",
                ));
            }
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

        let summary = summarize_tree(
            &path,
            device,
            self.max_entries,
            target.allows_internal_links(),
        )?;
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

/// Lists the paths a target may contribute, without judging them yet.
fn discover_entries(
    target: &TargetSpec,
    root: &Path,
    device: u64,
) -> Result<Vec<PathBuf>, FsError> {
    let read = |dir: &Path| -> Result<Vec<PathBuf>, FsError> {
        Ok(fs::read_dir(dir)
            .map_err(|source| FsError::Io {
                path: dir.to_path_buf(),
                source,
            })?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect())
    };
    let is_real_dir = |path: &Path| {
        metadata_no_follow(path).is_ok_and(|metadata| {
            metadata.is_dir() && !metadata.file_type().is_symlink() && metadata.dev() == device
        })
    };
    Ok(match target.layout {
        TargetLayout::Children => read(root)?,
        TargetLayout::Named(entry) => {
            let path = root.join(entry);
            if fs::symlink_metadata(&path).is_ok() {
                vec![path]
            } else {
                Vec::new()
            }
        }
        TargetLayout::Project { marker, entry } => {
            let mut found = Vec::new();
            for project in read(root)?.into_iter().filter(|path| is_real_dir(path)) {
                let mut roots = vec![project.clone()];
                if let Ok(children) = read(&project) {
                    roots.extend(children.into_iter().filter(|path| is_real_dir(path)));
                }
                for project_root in roots {
                    if !marker_present(&project_root, marker) {
                        continue;
                    }
                    let path = project_root.join(entry);
                    if fs::symlink_metadata(&path).is_ok() {
                        found.push(path);
                    }
                }
            }
            found
        }
    })
}

fn stable_candidate_id(target_id: &str, relative: &OsStr) -> CandidateId {
    let mut digest = Sha256::new();
    digest.update(target_id.as_bytes());
    digest.update([0]);
    digest.update(relative.as_bytes());
    CandidateId(format!("{target_id}:{:x}", digest.finalize()))
}

fn is_old_enough(candidate: &CleanupCandidate, target: &TargetSpec, now: DateTime<Utc>) -> bool {
    if target.minimum_age.is_zero() {
        return true;
    }
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
