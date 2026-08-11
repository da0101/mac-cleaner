use std::process::Command;
use std::sync::Arc;

use cleaner_domain::{ScanReport, StorageAnalysis, SystemSnapshot};
use cleaner_service::AppService;
use serde::Deserialize;
use tauri::State;

use super::helpers::display_error;

#[tauri::command]
pub(crate) async fn get_snapshot(
    service: State<'_, Arc<AppService>>,
) -> Result<SystemSnapshot, String> {
    service.system_snapshot().await.map_err(display_error)
}

#[tauri::command]
pub(crate) async fn start_scan(service: State<'_, Arc<AppService>>) -> Result<ScanReport, String> {
    service.scan().await.map_err(display_error)
}

#[tauri::command]
pub(crate) async fn analyze_storage(
    service: State<'_, Arc<AppService>>,
) -> Result<StorageAnalysis, String> {
    service.analyze_storage().await.map_err(display_error)
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum StorageOwner {
    Docker,
    Xcode,
}

#[tauri::command]
pub(crate) fn open_storage_owner(owner: StorageOwner) -> Result<(), String> {
    let application = match owner {
        StorageOwner::Docker => "Docker",
        StorageOwner::Xcode => "Xcode",
    };
    let status = Command::new("/usr/bin/open")
        .args(["-a", application])
        .status()
        .map_err(display_error)?;
    if !status.success() {
        return Err(format!("macOS could not open {application}."));
    }
    Ok(())
}
