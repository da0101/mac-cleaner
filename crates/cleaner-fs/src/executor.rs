use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

use chrono::Utc;
use cleaner_domain::{
    ApprovedPlan, CleanupCandidate, ExecutionItemReceipt, ExecutionMode, ExecutionReceipt,
};
use cleaner_policy::{catalog, catalog_root, is_protected};
use uuid::Uuid;

use crate::metadata::{
    DEFAULT_MAX_ENTRIES, identity_from_metadata, metadata_no_follow, summarize_tree,
    unsafe_candidate, validate_home_state, validate_metadata,
};
use crate::{FsError, QUARANTINE_RELATIVE};

#[derive(Clone, Debug)]
pub struct PlanExecutor {
    home: PathBuf,
}

impl PlanExecutor {
    #[must_use]
    pub fn new(home: PathBuf) -> Self {
        Self { home }
    }

    pub fn execute(&self, approved: ApprovedPlan) -> Result<ExecutionReceipt, FsError> {
        validate_home_state(&self.home)?;
        let started_at = Utc::now();
        let plan = approved.into_plan();
        if plan.is_expired(started_at) {
            return Err(FsError::ExpiredPlan);
        }
        for candidate in &plan.items {
            self.revalidate(candidate)?;
        }

        if plan.execution_mode == ExecutionMode::DryRun {
            let completed_at = Utc::now();
            return Ok(ExecutionReceipt {
                plan_id: plan.id,
                mode: plan.execution_mode,
                started_at,
                completed_at,
                items: plan
                    .items
                    .into_iter()
                    .map(|item| ExecutionItemReceipt {
                        quarantine_entry_id: None,
                        candidate_id: item.id,
                        target_id: Some(item.target_id),
                        relative_entry: Some(item.relative_entry),
                        original_path: item.lexical_path,
                        quarantine_path: None,
                        bytes: item.bytes,
                        files: item.files,
                        completed_at,
                        identity: Some(item.identity),
                    })
                    .collect(),
            });
        }

        let quarantine = ensure_private_quarantine(&self.home, plan.id)?;
        let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut receipts = Vec::with_capacity(plan.items.len());
        for item in &plan.items {
            let destination = quarantine.join(&item.id.0);
            if destination.exists() {
                rollback_moves(&moved)?;
                return Err(FsError::QuarantineCollision(destination));
            }
            if let Err(source) = fs::rename(&item.lexical_path, &destination) {
                rollback_moves(&moved)?;
                return Err(FsError::Io {
                    path: item.lexical_path.clone(),
                    source,
                });
            }
            moved.push((item.lexical_path.clone(), destination.clone()));
            receipts.push(ExecutionItemReceipt {
                quarantine_entry_id: Some(Uuid::new_v4()),
                candidate_id: item.id.clone(),
                target_id: Some(item.target_id.clone()),
                relative_entry: Some(item.relative_entry.clone()),
                original_path: item.lexical_path.clone(),
                quarantine_path: Some(destination),
                bytes: item.bytes,
                files: item.files,
                completed_at: Utc::now(),
                identity: Some(item.identity.clone()),
            });
        }

        Ok(ExecutionReceipt {
            plan_id: plan.id,
            mode: plan.execution_mode,
            started_at,
            completed_at: Utc::now(),
            items: receipts,
        })
    }

    fn revalidate(&self, candidate: &CleanupCandidate) -> Result<(), FsError> {
        let target = catalog()
            .iter()
            .find(|target| target.id == candidate.target_id.0)
            .ok_or_else(|| FsError::UnknownTarget(candidate.target_id.0.clone()))?;
        let root = catalog_root(&self.home, target)?;
        let expected = root.join(&candidate.relative_entry);
        if expected != candidate.lexical_path || is_protected(&self.home, &expected) {
            return Err(unsafe_candidate(
                &expected,
                "candidate path no longer matches policy",
            ));
        }
        let home_device = metadata_no_follow(&self.home)?.dev();
        let root_metadata = metadata_no_follow(&root)?;
        validate_metadata(&root, &root_metadata, home_device)?;
        let metadata = metadata_no_follow(&expected)?;
        validate_metadata(&expected, &metadata, root_metadata.dev())?;
        if identity_from_metadata(&metadata)? != candidate.identity {
            return Err(FsError::CandidateChanged(expected));
        }
        let resolved = expected.canonicalize().map_err(|source| FsError::Io {
            path: expected.clone(),
            source,
        })?;
        if resolved != candidate.resolved_path {
            return Err(FsError::CandidateChanged(expected));
        }
        let summary = summarize_tree(&expected, root_metadata.dev(), DEFAULT_MAX_ENTRIES)?;
        if summary.bytes != candidate.bytes || summary.files != candidate.files {
            return Err(FsError::CandidateChanged(expected));
        }
        Ok(())
    }
}

fn ensure_private_quarantine(home: &Path, plan_id: Uuid) -> Result<PathBuf, FsError> {
    let relative = Path::new(QUARANTINE_RELATIVE).join(plan_id.to_string());
    let mut current = home.to_path_buf();
    let device = metadata_no_follow(home)?.dev();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(FsError::UnsafeQuarantine(relative));
        };
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if !metadata.is_dir()
                    || metadata.file_type().is_symlink()
                    || metadata.dev() != device
                {
                    return Err(FsError::UnsafeQuarantine(current));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|source| FsError::Io {
                    path: current.clone(),
                    source,
                })?;
                fs::set_permissions(&current, fs::Permissions::from_mode(0o700)).map_err(
                    |source| FsError::Io {
                        path: current.clone(),
                        source,
                    },
                )?;
            }
            Err(source) => {
                return Err(FsError::Io {
                    path: current,
                    source,
                });
            }
        }
    }
    Ok(current)
}

fn rollback_moves(moved: &[(PathBuf, PathBuf)]) -> Result<(), FsError> {
    for (original, destination) in moved.iter().rev() {
        fs::rename(destination, original).map_err(|source| FsError::RollbackFailed {
            original: original.clone(),
            quarantine: destination.clone(),
            source,
        })?;
    }
    Ok(())
}
