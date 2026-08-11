use std::sync::Arc;

use cleaner_domain::{QuarantineAction, QuarantineEntry};
use cleaner_service::AppService;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;
use uuid::Uuid;

use super::helpers::{confirm_native, display_error};

#[tauri::command]
pub(crate) fn active_quarantine(
    service: State<'_, Arc<AppService>>,
    limit: u32,
) -> Result<Vec<QuarantineEntry>, String> {
    service.active_quarantine(limit).map_err(display_error)
}

#[tauri::command]
pub(crate) async fn confirm_quarantine_action(
    app: AppHandle,
    service: State<'_, Arc<AppService>>,
    entry_id: String,
    action: QuarantineAction,
) -> Result<QuarantineEntry, String> {
    let entry_id = Uuid::parse_str(&entry_id).map_err(display_error)?;
    let plan = service
        .prepare_quarantine_action(entry_id, action)
        .map_err(display_error)?;
    let (title, verb, consequence) = match action {
        QuarantineAction::Restore => (
            "Confirm exact restore",
            "Restore this item",
            format!("Restore to:\n{}", plan.entry.original_path.display()),
        ),
        QuarantineAction::Purge => (
            "Confirm permanent deletion",
            "Delete permanently",
            "This cannot be undone and is the only step that reclaims disk space.".to_owned(),
        ),
    };
    let message = format!(
        "Action: {:?}\nEntry: {}\nQuarantine path:\n{}\n\n{}\n\nThe item identity and exact path will be revalidated immediately before the action.",
        action,
        plan.entry.id,
        plan.entry.quarantine_path.display(),
        consequence,
    );
    let confirmed = confirm_native(&app, title, message, MessageDialogKind::Warning, verb).await?;
    if !confirmed {
        return Err("Quarantine action cancelled. Nothing was changed.".to_owned());
    }
    service
        .approve_quarantine_action(plan.id)
        .map_err(display_error)?;
    service
        .execute_quarantine_action(plan.id)
        .await
        .map_err(display_error)
}
