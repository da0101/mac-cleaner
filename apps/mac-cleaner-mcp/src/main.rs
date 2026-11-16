use std::fmt::Write as _;
use std::sync::Arc;

use anyhow::{Context, Result};
use cleaner_domain::{
    CandidateId, CleanupPlan, ExecutionMode, QuarantineAction, QuarantineActionPlan, ScanReport,
    StorageAnalysis, SystemSnapshot, ToolCleanupPlan,
};
use cleaner_service::AppService;
use directories::{BaseDirs, ProjectDirs};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::schemars;
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router, transport::stdio};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PrepareCleanupRequest {
    /// Opaque candidate IDs returned by `cleanup_candidates`. Paths are never accepted.
    candidate_ids: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ExecutePlanRequest {
    /// UUID returned by the matching prepare tool.
    plan_id: String,
    /// Exact plan-specific phrase returned by the matching prepare tool, after the user confirms it.
    confirmation: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PrepareToolCleanupRequest {
    /// Fixed cleanup kind: `uv-cache` or `simulator-recordings`.
    kind: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PrepareRecoveryRequest {
    /// Opaque recovery-entry UUID returned by `list_recovery_items`.
    entry_id: String,
    /// Fixed action: `restore` or `purge`.
    action: String,
}

#[derive(Clone)]
struct MacCleanerMcp {
    service: Arc<AppService>,
}

#[tool_router]
impl MacCleanerMcp {
    #[tool(
        description = "Run a fresh read-only Mac Cleaner storage analysis. Returns exact measured paths, allocated sizes, growth since the prior or baseline scan, safety classifications, recommendations, and the whole-volume accounting gap. This can take a while because it measures known developer storage roots.",
        annotations(
            title = "Analyze Mac storage",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn analyze_storage(&self) -> CallToolResult {
        match self.service.analyze_storage().await {
            Ok(report) => storage_result(&report),
            Err(error) => tool_error("Storage analysis failed", &error),
        }
    }

    #[tool(
        description = "Return Mac Cleaner's most recently saved storage analysis without scanning the disk again. Use this first for fast inspection; run analyze_storage only when the report is missing or stale.",
        annotations(
            title = "Get latest storage report",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn latest_storage_report(&self) -> CallToolResult {
        match self.service.latest_storage_analysis() {
            Ok(Some(report)) => storage_result(&report),
            Ok(None) => CallToolResult::error(vec![ContentBlock::text(
                "No saved storage report exists yet. Run analyze_storage to create one.",
            )]),
            Err(error) => tool_error("Could not load the latest storage report", &error),
        }
    }

    #[tool(
        description = "Capture current read-only system health: SSD capacity and free space, RAM availability, compression and swap, memory pressure, and the largest processes with developer-process protection reasons.",
        annotations(
            title = "Inspect Mac system health",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn system_health(&self) -> CallToolResult {
        match self.service.system_snapshot().await {
            Ok(snapshot) => system_result(&snapshot),
            Err(error) => tool_error("System-health capture failed", &error),
        }
    }

    #[tool(
        description = "Run Mac Cleaner's conservative cleanup-candidate discovery without changing anything. Returns only versioned catalog candidates plus skipped paths and safety reasons. Findings are evidence, not deletion authority.",
        annotations(
            title = "Discover reviewable cleanup candidates",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn cleanup_candidates(&self) -> CallToolResult {
        match self.service.scan().await {
            Ok(report) => scan_result(&report),
            Err(error) => tool_error("Cleanup-candidate discovery failed", &error),
        }
    }

    #[tool(
        description = "Prepare a five-minute exact cleanup plan from opaque candidate IDs returned by cleanup_candidates. This does not change files. The plan always moves independently rediscovered catalog items to Mac Cleaner Recovery; it cannot accept paths, repositories, SDKs, runtimes, Docker databases, or unknown data.",
        annotations(
            title = "Prepare recoverable cleanup",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn prepare_cleanup(
        &self,
        Parameters(request): Parameters<PrepareCleanupRequest>,
    ) -> CallToolResult {
        if request.candidate_ids.is_empty() {
            return input_error("At least one opaque candidate ID is required.");
        }
        if let Err(error) = self.service.scan().await {
            return tool_error("Fresh cleanup discovery failed", &error);
        }
        let ids = request
            .candidate_ids
            .into_iter()
            .map(CandidateId)
            .collect::<Vec<_>>();
        match self.service.prepare_plan(&ids, ExecutionMode::Quarantine) {
            Ok(plan) => cleanup_plan_result(&plan),
            Err(error) => tool_error("Cleanup plan preparation failed", &error),
        }
    }

    #[tool(
        description = "Execute one unexpired cleanup plan only after showing the exact plan to the user and receiving explicit confirmation. The confirmation must exactly match the phrase returned by prepare_cleanup. Items are moved to Mac Cleaner Recovery and still occupy disk space until separately purged.",
        annotations(
            title = "Move approved items to Recovery",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn execute_cleanup(
        &self,
        Parameters(request): Parameters<ExecutePlanRequest>,
    ) -> CallToolResult {
        let plan_id = match parse_uuid(&request.plan_id) {
            Ok(value) => value,
            Err(result) => return result,
        };
        let plan = match self.service.plan(plan_id) {
            Ok(value) => value,
            Err(error) => return tool_error("Cleanup plan lookup failed", &error),
        };
        if plan.execution_mode != ExecutionMode::Quarantine {
            return input_error("MCP can execute only recoverable quarantine plans.");
        }
        let expected = cleanup_confirmation(plan_id);
        if request.confirmation != expected {
            return confirmation_error(&expected);
        }
        if let Err(error) = self.service.approve_plan(plan_id) {
            return tool_error("Cleanup approval failed", &error);
        }
        match self.service.execute_approved(plan_id).await {
            Ok(receipt) => structured_result(
                &receipt,
                format!(
                    "Moved {} exact catalog item(s) totaling {} into Mac Cleaner Recovery. This is recoverable and has not yet reclaimed disk space.",
                    receipt.items.len(),
                    format_bytes(plan.total_bytes),
                ),
            ),
            Err(error) => tool_error("Cleanup execution failed", &error),
        }
    }

    #[tool(
        description = "Prepare a five-minute plan for one fixed tool-owned cleanup: `uv-cache` or `simulator-recordings`. Simulator cleanup targets only verified old test video attachments, never the simulator runtime, app data, Xcode, or iOS SDKs. No files change during preparation.",
        annotations(
            title = "Prepare tool-owned cleanup",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn prepare_tool_cleanup(
        &self,
        Parameters(request): Parameters<PrepareToolCleanupRequest>,
    ) -> CallToolResult {
        let result = match request.kind.as_str() {
            "uv-cache" => self.service.prepare_uv_cache_prune().await,
            "simulator-recordings" => self.service.prepare_simulator_recordings_cleanup().await,
            _ => return input_error("kind must be `uv-cache` or `simulator-recordings`."),
        };
        match result {
            Ok(plan) => tool_cleanup_plan_result(&plan),
            Err(error) => tool_error("Tool cleanup preparation failed", &error),
        }
    }

    #[tool(
        description = "Execute one unexpired fixed tool-cleanup plan after showing it to the user and receiving its exact confirmation phrase. This permanently removes only the prepared uv cache entries or verified old simulator test recordings and never accepts a path.",
        annotations(
            title = "Run approved tool cleanup",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn execute_tool_cleanup(
        &self,
        Parameters(request): Parameters<ExecutePlanRequest>,
    ) -> CallToolResult {
        let plan_id = match parse_uuid(&request.plan_id) {
            Ok(value) => value,
            Err(result) => return result,
        };
        let plan = match self.service.tool_cleanup_plan(plan_id) {
            Ok(value) => value,
            Err(error) => return tool_error("Tool cleanup plan lookup failed", &error),
        };
        let expected = tool_cleanup_confirmation(plan_id);
        if request.confirmation != expected {
            return confirmation_error(&expected);
        }
        if let Err(error) = self.service.approve_tool_cleanup(plan_id) {
            return tool_error("Tool cleanup approval failed", &error);
        }
        match self.service.execute_tool_cleanup(plan_id).await {
            Ok(receipt) => structured_result(
                &receipt,
                format!(
                    "Approved {:?} cleanup reclaimed {}. {}",
                    plan.kind,
                    format_bytes(receipt.reclaimed_bytes),
                    receipt.summary,
                ),
            ),
            Err(error) => tool_error("Tool cleanup execution failed", &error),
        }
    }

    #[tool(
        description = "List active Mac Cleaner Recovery entries. These were previously moved by an exact approved cleanup and still occupy disk space. Listing never changes files.",
        annotations(
            title = "List Recovery items",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn list_recovery_items(&self) -> CallToolResult {
        match self.service.active_quarantine(100) {
            Ok(entries) => {
                let bytes = entries
                    .iter()
                    .fold(0_u64, |sum, item| sum.saturating_add(item.bytes));
                structured_result(
                    &entries,
                    format!(
                        "Mac Cleaner Recovery contains {} active item(s) totaling {}. Purge requires a separate prepared plan and explicit confirmation.",
                        entries.len(),
                        format_bytes(bytes),
                    ),
                )
            }
            Err(error) => tool_error("Recovery listing failed", &error),
        }
    }

    #[tool(
        description = "Prepare a five-minute `restore` or `purge` plan for exactly one opaque Recovery entry UUID. Preparation changes nothing. Purge permanently reclaims the entry's space; restore moves it back to its original verified location.",
        annotations(
            title = "Prepare Recovery action",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    fn prepare_recovery_action(
        &self,
        Parameters(request): Parameters<PrepareRecoveryRequest>,
    ) -> CallToolResult {
        let entry_id = match parse_uuid(&request.entry_id) {
            Ok(value) => value,
            Err(result) => return result,
        };
        let action = match request.action.as_str() {
            "restore" => QuarantineAction::Restore,
            "purge" => QuarantineAction::Purge,
            _ => return input_error("action must be `restore` or `purge`."),
        };
        match self.service.prepare_quarantine_action(entry_id, action) {
            Ok(plan) => recovery_plan_result(&plan),
            Err(error) => tool_error("Recovery action preparation failed", &error),
        }
    }

    #[tool(
        description = "Execute one unexpired Recovery action after showing it to the user and receiving the exact plan-specific confirmation. Purge permanently deletes one Recovery entry; restore moves that exact entry back. No path is accepted.",
        annotations(
            title = "Run approved Recovery action",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn execute_recovery_action(
        &self,
        Parameters(request): Parameters<ExecutePlanRequest>,
    ) -> CallToolResult {
        let action_id = match parse_uuid(&request.plan_id) {
            Ok(value) => value,
            Err(result) => return result,
        };
        let plan = match self.service.quarantine_action_plan(action_id) {
            Ok(value) => value,
            Err(error) => return tool_error("Recovery action lookup failed", &error),
        };
        let expected = recovery_confirmation(&plan);
        if request.confirmation != expected {
            return confirmation_error(&expected);
        }
        if let Err(error) = self.service.approve_quarantine_action(action_id) {
            return tool_error("Recovery action approval failed", &error);
        }
        match self.service.execute_quarantine_action(action_id).await {
            Ok(entry) => structured_result(
                &entry,
                format!(
                    "Completed {:?} for one Recovery item ({}).",
                    plan.action,
                    format_bytes(entry.bytes),
                ),
            ),
            Err(error) => tool_error("Recovery action failed", &error),
        }
    }
}

#[tool_handler(
    name = "mac-cleaner-storage",
    version = "2.0.0-alpha.1",
    instructions = "Local access to Mac Cleaner's Rust storage, CPU/RAM, and safety engine. Inspect first. Mutation tools accept only opaque IDs from Mac Cleaner, never paths. Always prepare a short-lived exact plan, show its items/size/action to the user, obtain explicit confirmation, then call the matching destructive execution tool with the returned phrase. General cleanup moves items to Recovery first; permanent purge is a separate confirmed plan. Never infer that protected or unknown data is safe, and never bypass Mac Cleaner with raw filesystem or Docker commands."
)]
impl ServerHandler for MacCleanerMcp {}

#[tokio::main]
async fn main() -> Result<()> {
    let service = MacCleanerMcp {
        service: Arc::new(open_service()?),
    };
    let running = service
        .serve(stdio())
        .await
        .context("cannot start Mac Cleaner MCP over stdio")?;
    running
        .waiting()
        .await
        .context("Mac Cleaner MCP transport failed")?;
    Ok(())
}

fn open_service() -> Result<AppService> {
    let base = BaseDirs::new().context("macOS home directory is unavailable")?;
    let project = ProjectDirs::from("com", "da0101", "Mac Cleaner")
        .context("macOS application data directory is unavailable")?;
    AppService::open(
        base.home_dir().to_path_buf(),
        project.data_dir().to_path_buf(),
    )
    .context("cannot initialize Mac Cleaner MCP engine")
}

fn storage_result(report: &StorageAnalysis) -> CallToolResult {
    structured_result(report, storage_summary(report))
}

fn system_result(snapshot: &SystemSnapshot) -> CallToolResult {
    let top_cpu = snapshot
        .processes
        .iter()
        .max_by(|left, right| left.cpu_percent.total_cmp(&right.cpu_percent));
    let top_cpu_summary = top_cpu.map_or_else(
        || "no process sample".to_owned(),
        |process| format!("{} at {:.1}%", process.name, process.cpu_percent),
    );
    let summary = format!(
        "CPU: {:.1}% ({:?}) across {} logical cores; top sampled process: {}. SSD free: {} of {}. RAM available: {} of {}; swap used: {}; pressure: {:?}. Top memory and CPU processes plus protection reasons are included in structuredContent.",
        snapshot.cpu.usage_percent,
        snapshot.cpu.pressure,
        snapshot.cpu.logical_cores,
        top_cpu_summary,
        format_bytes(snapshot.disk.available_bytes),
        format_bytes(snapshot.disk.total_bytes),
        format_bytes(snapshot.memory.available_bytes),
        format_bytes(snapshot.memory.total_bytes),
        format_bytes(snapshot.memory.swap_used_bytes),
        snapshot.memory.pressure,
    );
    structured_result(snapshot, summary)
}

fn cleanup_plan_result(plan: &CleanupPlan) -> CallToolResult {
    let confirmation = cleanup_confirmation(plan.id);
    plan_result(
        plan,
        &confirmation,
        format!(
            "Prepared recoverable plan {} for {} exact item(s), {} and {} files. Show the complete plan to the user. After explicit approval, call execute_cleanup with confirmation `{}`. Moving to Recovery does not reclaim disk space until a separately confirmed purge.",
            plan.id,
            plan.items.len(),
            format_bytes(plan.total_bytes),
            plan.total_files,
            confirmation,
        ),
    )
}

fn tool_cleanup_plan_result(plan: &ToolCleanupPlan) -> CallToolResult {
    let confirmation = tool_cleanup_confirmation(plan.id);
    plan_result(
        plan,
        &confirmation,
        format!(
            "Prepared {:?} plan {} for exactly {} at {}. Show this plan to the user. After explicit approval, call execute_tool_cleanup with confirmation `{}`.",
            plan.kind,
            plan.id,
            format_bytes(plan.bytes_before),
            plan.target_path.display(),
            confirmation,
        ),
    )
}

fn recovery_plan_result(plan: &QuarantineActionPlan) -> CallToolResult {
    let confirmation = recovery_confirmation(plan);
    plan_result(
        plan,
        &confirmation,
        format!(
            "Prepared {:?} action {} for one Recovery item: {} at {}. Show this exact action to the user. After explicit approval, call execute_recovery_action with confirmation `{}`.",
            plan.action,
            plan.id,
            format_bytes(plan.entry.bytes),
            plan.entry.original_path.display(),
            confirmation,
        ),
    )
}

fn plan_result<T: Serialize>(plan: &T, confirmation: &str, summary: String) -> CallToolResult {
    match serde_json::to_value(plan) {
        Ok(plan) => {
            let mut result = CallToolResult::success(vec![ContentBlock::text(summary)]);
            result.structured_content = Some(serde_json::json!({
                "plan": plan,
                "required_confirmation": confirmation,
            }));
            result
        }
        Err(error) => tool_error("Could not encode the prepared plan", &error),
    }
}

fn cleanup_confirmation(plan_id: Uuid) -> String {
    format!("MOVE {plan_id} TO RECOVERY")
}

fn tool_cleanup_confirmation(plan_id: Uuid) -> String {
    format!("RUN TOOL CLEANUP {plan_id}")
}

fn recovery_confirmation(plan: &QuarantineActionPlan) -> String {
    match plan.action {
        QuarantineAction::Restore => format!("RESTORE RECOVERY ITEM {}", plan.id),
        QuarantineAction::Purge => format!("PERMANENTLY PURGE RECOVERY ITEM {}", plan.id),
    }
}

fn parse_uuid(value: &str) -> Result<Uuid, CallToolResult> {
    Uuid::parse_str(value).map_err(|_| input_error("The plan or entry ID must be a valid UUID."))
}

fn confirmation_error(expected: &str) -> CallToolResult {
    input_error(&format!(
        "Confirmation did not match this exact plan. Show the plan to the user and, only after explicit approval, submit exactly: `{expected}`"
    ))
}

fn input_error(message: &str) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.to_owned())])
}

fn scan_result(report: &ScanReport) -> CallToolResult {
    let candidate_bytes = report
        .candidates
        .iter()
        .fold(0_u64, |total, item| total.saturating_add(item.bytes));
    let summary = format!(
        "Found {} conservative review candidates totaling {}. Skipped {} paths because they were protected, unsafe, active, or outside the compiled catalog. No files were changed.",
        report.candidates.len(),
        format_bytes(candidate_bytes),
        report.skipped.len(),
    );
    structured_result(report, summary)
}

fn storage_summary(report: &StorageAnalysis) -> String {
    let mut summary = format!(
        "Storage snapshot {} captured {}. SSD free: {} of {}; measured reviewable items: {}. The complete structuredContent contains all {} measured hotspots.\nLargest measured paths:\n",
        report.snapshot_id,
        report.captured_at,
        format_bytes(report.available_bytes),
        format_bytes(report.total_bytes),
        format_bytes(report.reviewable_bytes),
        report.hotspots.len(),
    );
    for hotspot in report.hotspots.iter().take(20) {
        let growth = hotspot
            .delta_bytes
            .map_or_else(|| "no comparable baseline".to_owned(), format_signed_bytes);
        let _ = writeln!(
            summary,
            "- {} — {} — {} — {:?} — change {}",
            hotspot.display_name,
            format_bytes(hotspot.bytes),
            hotspot.path.display(),
            hotspot.disposition,
            growth,
        );
    }
    if let Some(comparison) = &report.comparison {
        let _ = write!(
            summary,
            "Whole-volume change: {}; measured-path change: {}; unattributed accounting gap: {}.",
            format_signed_bytes(comparison.volume_used_delta_bytes),
            format_signed_bytes(comparison.measured_hotspot_delta_bytes),
            format_signed_bytes(comparison.accounting_gap_bytes),
        );
    }
    summary
}

fn structured_result<T: Serialize>(value: &T, summary: String) -> CallToolResult {
    match serde_json::to_value(value) {
        Ok(value) => {
            let mut result = CallToolResult::success(vec![ContentBlock::text(summary)]);
            result.structured_content = Some(value);
            result
        }
        Err(error) => tool_error("Could not encode the Mac Cleaner report", &error),
    }
}

fn tool_error(title: &str, error: &impl std::fmt::Display) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(format!("{title}: {error}"))])
}

fn format_signed_bytes(bytes: i64) -> String {
    if bytes < 0 {
        format!("-{}", format_bytes(bytes.unsigned_abs()))
    } else {
        format!("+{}", format_bytes(bytes.unsigned_abs()))
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut unit = 0_usize;
    let mut divisor = 1_u64;
    while bytes / divisor >= 1_024 && unit < UNITS.len() - 1 {
        divisor = divisor.saturating_mul(1_024);
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        let whole = bytes / divisor;
        let hundredths = (bytes % divisor).saturating_mul(100) / divisor;
        format!("{whole}.{hundredths:02} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{MacCleanerMcp, cleanup_confirmation, tool_cleanup_confirmation};

    #[test]
    fn exposed_tools_are_closed_world_and_mutations_are_explicitly_destructive()
    -> Result<(), Box<dyn std::error::Error>> {
        let tools = MacCleanerMcp::tool_router().list_all();
        let names = tools
            .iter()
            .map(|tool| tool.name.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "analyze_storage",
                "cleanup_candidates",
                "execute_cleanup",
                "execute_recovery_action",
                "execute_tool_cleanup",
                "latest_storage_report",
                "list_recovery_items",
                "prepare_cleanup",
                "prepare_recovery_action",
                "prepare_tool_cleanup",
                "system_health"
            ]
        );
        for tool in tools {
            let annotations = tool
                .annotations
                .as_ref()
                .ok_or("tool annotations missing")?;
            assert_eq!(annotations.open_world_hint, Some(false));
            let is_execution = tool.name.starts_with("execute_");
            assert_eq!(annotations.destructive_hint, Some(is_execution));
            assert_eq!(
                annotations.read_only_hint,
                Some(matches!(
                    tool.name.as_ref(),
                    "analyze_storage"
                        | "cleanup_candidates"
                        | "latest_storage_report"
                        | "list_recovery_items"
                        | "system_health"
                ))
            );
        }
        Ok(())
    }

    #[test]
    fn confirmations_are_bound_to_one_plan() {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        assert_ne!(cleanup_confirmation(first), cleanup_confirmation(second));
        assert!(cleanup_confirmation(first).contains(&first.to_string()));
        assert!(tool_cleanup_confirmation(first).contains(&first.to_string()));
    }
}
