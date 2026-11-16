//! Bounded filesystem discovery and plan execution primitives.

#[cfg(not(unix))]
compile_error!("cleaner-fs currently supports Unix filesystems only");

mod error;
mod executor;
mod metadata;
mod quarantine;
mod scanner;
mod storage_analysis;
mod storage_catalog;
mod storage_project_catalog;

pub use error::FsError;
pub use executor::PlanExecutor;
pub use quarantine::QuarantineExecutor;
pub use scanner::Scanner;
pub use storage_analysis::StorageAnalyzer;

pub(crate) const QUARANTINE_RELATIVE: &str = "Library/Application Support/Mac Cleaner/Quarantine";
