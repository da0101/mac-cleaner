use std::sync::Arc;

use cleaner_domain::{CandidateId, CleanupPlan, ExecutionMode, ExecutionReceipt, ScanReport};
use cleaner_service::AppService;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;
use uuid::Uuid;

use super::helpers::{confirm_native, display_error};

#[tauri::command]
pub(crate) fn latest_scan(service: State<'_, Arc<AppService>>) -> Option<ScanReport> {
    service.latest_scan()
}

#[tauri::command]
pub(crate) fn prepare_plan(
    service: State<'_, Arc<AppService>>,
    ids: Vec<String>,
    dry_run: bool,
) -> Result<CleanupPlan, String> {
    let ids = ids.into_iter().map(CandidateId).collect::<Vec<_>>();
    let mode = if dry_run {
        ExecutionMode::DryRun
    } else {
        ExecutionMode::Quarantine
    };
    service.prepare_plan(&ids, mode).map_err(display_error)
}

#[tauri::command]
pub(crate) async fn confirm_and_execute_plan(
    app: AppHandle,
    service: State<'_, Arc<AppService>>,
    plan_id: String,
) -> Result<ExecutionReceipt, String> {
    let plan_id = Uuid::parse_str(&plan_id).map_err(display_error)?;
    let plan = service.plan(plan_id).map_err(display_error)?;
    let message = plan_confirmation_message(&plan);
    let confirmed = confirm_native(
        &app,
        "Confirm exact cleanup plan",
        message,
        MessageDialogKind::Warning,
        "Move to quarantine",
    )
    .await?;
    if !confirmed {
        return Err("Cleanup cancelled. Nothing was changed.".to_owned());
    }

    service.approve_plan(plan_id).map_err(display_error)?;
    service
        .execute_approved(plan_id)
        .await
        .map_err(display_error)
}

#[tauri::command]
pub(crate) fn recent_history(
    service: State<'_, Arc<AppService>>,
    limit: u32,
) -> Result<Vec<ExecutionReceipt>, String> {
    service.recent_history(limit).map_err(display_error)
}

fn plan_confirmation_message(plan: &CleanupPlan) -> String {
    let paths = plan
        .items
        .iter()
        .map(|item| format!("• {}\n  {}", item.lexical_path.display(), item.recovery))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "Plan: {}\nExpires: {}\nItems: {}\n\n{}\n\nEvery item will be revalidated immediately before it is moved. Files remain recoverable in Mac Cleaner quarantine.",
        plan.id,
        plan.expires_at,
        plan.items.len(),
        paths
    )
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use cleaner_domain::{CATALOG_VERSION, CleanupPlan, ExecutionMode, PlanState};
    use uuid::Uuid;

    use super::plan_confirmation_message;

    #[test]
    fn native_confirmation_names_the_exact_plan_and_recovery_boundary() {
        let now = Utc::now();
        let plan = CleanupPlan {
            id: Uuid::new_v4(),
            catalog_version: CATALOG_VERSION,
            created_at: now,
            expires_at: now + Duration::minutes(5),
            state: PlanState::Prepared,
            execution_mode: ExecutionMode::Quarantine,
            items: Vec::new(),
            total_bytes: 0,
            total_files: 0,
        };
        let message = plan_confirmation_message(&plan);
        assert!(message.contains(&plan.id.to_string()));
        assert!(message.contains("revalidated"));
        assert!(message.contains("recoverable"));
    }
}
