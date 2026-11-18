//! Versioned target catalog and fail-closed cleanup authorization.

mod auto;
mod catalog;
mod error;
mod paths;
mod plans;
mod quarantine;

pub use auto::{
    app_bundle, group_app_energy, is_never_quit, never_quit_names, plan_ram_relief,
    validate_auto_settings,
};
pub use catalog::{
    PROJECTS_ROOT, TargetLayout, TargetSpec, candidate_path, catalog, catalog_root, marker_present,
    safe_auto_catalog,
};
pub use error::PolicyError;
pub use paths::{is_protected, is_safe_relative, protected_sentinels, validate_home};
pub use plans::{approve_plan, prepare_plan};
pub use quarantine::{approve_quarantine_action, prepare_quarantine_action};

#[cfg(test)]
mod tests;
