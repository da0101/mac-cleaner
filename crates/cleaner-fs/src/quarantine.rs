use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

use chrono::Utc;
use cleaner_domain::{
    ApprovedQuarantineAction, FileKind, QuarantineAction, QuarantineEntry, QuarantineState,
};
use cleaner_policy::{candidate_path, catalog, catalog_root};

use crate::metadata::{
    DEFAULT_MAX_ENTRIES, identity_from_metadata, metadata_no_follow, summarize_tree,
    unsafe_candidate, validate_home_state, validate_metadata,
};
use crate::{FsError, QUARANTINE_RELATIVE};

#[derive(Clone, Debug)]
pub struct QuarantineExecutor {
    home: PathBuf,
}

impl QuarantineExecutor {
    #[must_use]
    pub fn new(home: PathBuf) -> Self {
        Self { home }
    }

    pub fn execute(&self, approved: ApprovedQuarantineAction) -> Result<QuarantineEntry, FsError> {
        validate_home_state(&self.home)?;
        let plan = approved.into_plan();
        if plan.is_expired(Utc::now()) {
            return Err(FsError::ExpiredPlan);
        }
        self.revalidate(&plan.entry, plan.action)?;

        match plan.action {
            QuarantineAction::Restore => {
                fs::rename(&plan.entry.quarantine_path, &plan.entry.original_path).map_err(
                    |source| FsError::Io {
                        path: plan.entry.quarantine_path.clone(),
                        source,
                    },
                )?;
            }
            QuarantineAction::Purge => match plan.entry.identity.kind {
                FileKind::File => fs::remove_file(&plan.entry.quarantine_path),
                FileKind::Directory => fs::remove_dir_all(&plan.entry.quarantine_path),
            }
            .map_err(|source| FsError::Io {
                path: plan.entry.quarantine_path.clone(),
                source,
            })?,
        }

        if plan.entry.quarantine_path.exists() {
            return Err(FsError::QuarantineChanged(plan.entry.quarantine_path));
        }
        let now = Utc::now();
        Ok(QuarantineEntry {
            state: match plan.action {
                QuarantineAction::Restore => QuarantineState::Restored,
                QuarantineAction::Purge => QuarantineState::Purged,
            },
            updated_at: now,
            ..plan.entry
        })
    }

    fn revalidate(&self, entry: &QuarantineEntry, action: QuarantineAction) -> Result<(), FsError> {
        if entry.state != QuarantineState::Active {
            return Err(unsafe_candidate(
                &entry.quarantine_path,
                "quarantine entry is no longer active",
            ));
        }
        let root = self.home.join(QUARANTINE_RELATIVE);
        validate_quarantine_path(&root, entry)?;
        let root_metadata = metadata_no_follow(&root)?;
        validate_metadata(&root, &root_metadata, metadata_no_follow(&self.home)?.dev())?;
        let metadata = metadata_no_follow(&entry.quarantine_path)?;
        validate_metadata(&entry.quarantine_path, &metadata, root_metadata.dev())?;
        if identity_from_metadata(&metadata)? != entry.identity {
            return Err(FsError::QuarantineChanged(entry.quarantine_path.clone()));
        }
        let allow_internal_links = catalog()
            .iter()
            .find(|target| target.id == entry.target_id.0)
            .is_some_and(cleaner_policy::TargetSpec::allows_internal_links);
        let summary = summarize_tree(
            &entry.quarantine_path,
            root_metadata.dev(),
            DEFAULT_MAX_ENTRIES,
            allow_internal_links,
        )?;
        if summary.bytes != entry.bytes || summary.files != entry.files {
            return Err(FsError::QuarantineChanged(entry.quarantine_path.clone()));
        }

        if action == QuarantineAction::Restore {
            let target = catalog()
                .iter()
                .find(|target| target.id == entry.target_id.0)
                .ok_or_else(|| FsError::UnknownTarget(entry.target_id.0.clone()))?;
            let catalog_root = catalog_root(&self.home, target)?;
            let expected =
                candidate_path(&self.home, target, &entry.relative_entry).map_err(|_| {
                    unsafe_candidate(
                        &entry.original_path,
                        "restore destination no longer matches the compiled catalog",
                    )
                })?;
            if expected != entry.original_path {
                return Err(unsafe_candidate(
                    &entry.original_path,
                    "restore destination no longer matches the compiled catalog",
                ));
            }
            let catalog_metadata = metadata_no_follow(&catalog_root)?;
            validate_metadata(
                &catalog_root,
                &catalog_metadata,
                metadata_no_follow(&self.home)?.dev(),
            )?;
            if fs::symlink_metadata(&entry.original_path).is_ok() {
                return Err(FsError::RestoreDestinationOccupied(
                    entry.original_path.clone(),
                ));
            }
        }
        Ok(())
    }
}

fn validate_quarantine_path(root: &Path, entry: &QuarantineEntry) -> Result<(), FsError> {
    let relative = entry
        .quarantine_path
        .strip_prefix(root)
        .map_err(|_| FsError::UnsafeQuarantine(entry.quarantine_path.clone()))?;
    let components = relative.components().collect::<Vec<_>>();
    let expected_plan = entry.plan_id.to_string();
    let expected_candidate = &entry.candidate_id.0;
    if components.len() != 2
        || components[0] != Component::Normal(expected_plan.as_ref())
        || components[1] != Component::Normal(expected_candidate.as_ref())
    {
        return Err(FsError::UnsafeQuarantine(entry.quarantine_path.clone()));
    }
    Ok(())
}
