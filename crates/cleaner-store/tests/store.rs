use chrono::{Duration, Utc};
use cleaner_domain::{
    CATALOG_VERSION, CleanupPlan, ExecutionMode, FileIdentity, FileKind, PlanState,
    ToolCleanupFile, ToolCleanupKind, ToolCleanupOperation, ToolCleanupPlan, ToolCleanupReceipt,
    ToolCleanupState,
};
use cleaner_store::Store;
use std::path::PathBuf;
use uuid::Uuid;

fn empty_plan() -> CleanupPlan {
    let now = Utc::now();
    CleanupPlan {
        id: Uuid::new_v4(),
        catalog_version: CATALOG_VERSION,
        created_at: now,
        expires_at: now + Duration::minutes(5),
        state: PlanState::Prepared,
        execution_mode: ExecutionMode::DryRun,
        items: Vec::new(),
        total_bytes: 0,
        total_files: 0,
    }
}

#[test]
fn large_simulator_manifest_round_trips_portable_timestamps()
-> Result<(), Box<dyn std::error::Error>> {
    let store = Store::open_in_memory()?;
    let now = Utc::now();
    let identity = FileIdentity {
        device: 1,
        inode: 2,
        size: 3,
        modified_unix_nanos: 1_786_000_000_123_456_789,
        link_count: 1,
        kind: FileKind::File,
    };
    let files = (0_u64..500)
        .map(|index| ToolCleanupFile {
            relative_name: format!("recording-{index:04}"),
            identity: FileIdentity {
                inode: index + 10,
                ..identity.clone()
            },
            allocated_bytes: 4_096,
        })
        .collect::<Vec<_>>();
    let plan = ToolCleanupPlan {
        id: Uuid::new_v4(),
        kind: ToolCleanupKind::SimulatorTestRecordings,
        state: ToolCleanupState::Prepared,
        target_path: PathBuf::from("/Users/test/Library/Developer/CoreSimulator/Attachments"),
        target_identity: FileIdentity {
            kind: FileKind::Directory,
            ..identity
        },
        operation: ToolCleanupOperation::SimulatorAttachments {
            device_udid: "TEST-DEVICE".to_owned(),
            device_name: "Test iPhone".to_owned(),
            device_was_booted: true,
            files,
        },
        bytes_before: 500 * 4_096,
        created_at: now,
        expires_at: now + Duration::minutes(5),
    };
    store.save_tool_cleanup_plan(&plan)?;
    assert_eq!(store.load_tool_cleanup_plan(plan.id)?, plan);
    Ok(())
}

#[test]
fn startup_recovery_fails_approved_operations_without_resuming()
-> Result<(), Box<dyn std::error::Error>> {
    let store = Store::open_in_memory()?;
    let plan = empty_plan();
    store.save_plan(&plan)?;
    store.mark_approved(plan.id, Uuid::new_v4(), Utc::now())?;

    assert_eq!(store.fail_incomplete_on_startup()?, 1);
    assert!(store.mark_executing(plan.id).is_err());
    Ok(())
}

#[test]
fn settings_round_trip_as_typed_json() -> Result<(), Box<dyn std::error::Error>> {
    let store = Store::open_in_memory()?;
    store.set_setting("monitor_interval", &15_u64)?;
    assert_eq!(store.get_setting::<u64>("monitor_interval")?, Some(15));
    Ok(())
}

#[test]
fn tool_cleanup_is_journaled_before_and_after_execution() -> Result<(), Box<dyn std::error::Error>>
{
    let store = Store::open_in_memory()?;
    let now = Utc::now();
    let plan = ToolCleanupPlan {
        id: Uuid::new_v4(),
        kind: ToolCleanupKind::UvCachePrune,
        state: ToolCleanupState::Prepared,
        target_path: PathBuf::from("/Users/test/.cache/uv"),
        target_identity: FileIdentity {
            device: 1,
            inode: 2,
            size: 3,
            modified_unix_nanos: 4,
            link_count: 1,
            kind: FileKind::Directory,
        },
        operation: ToolCleanupOperation::Command {
            executable_path: PathBuf::from("/opt/homebrew/Cellar/uv/1/bin/uv"),
            arguments: vec!["cache".to_owned(), "prune".to_owned()],
        },
        bytes_before: 1_000,
        created_at: now,
        expires_at: now + Duration::minutes(5),
    };
    store.save_tool_cleanup_plan(&plan)?;
    assert_eq!(store.load_tool_cleanup_plan(plan.id)?, plan);
    store.mark_tool_cleanup_approved(plan.id, Uuid::new_v4(), now)?;
    store.mark_tool_cleanup_executing(plan.id)?;
    store.append_tool_cleanup_detail(plan.id, "verified exact item")?;
    let receipt = ToolCleanupReceipt {
        plan_id: plan.id,
        kind: plan.kind,
        target_path: plan.target_path,
        bytes_before: 1_000,
        bytes_after: 400,
        reclaimed_bytes: 600,
        completed_at: now,
        summary: "pruned".to_owned(),
    };
    store.save_tool_cleanup_receipt(&receipt)?;
    let saved = store
        .latest_tool_cleanup_receipt(ToolCleanupKind::UvCachePrune)?
        .ok_or("saved tool receipt is missing")?;
    assert_eq!(saved.reclaimed_bytes, receipt.reclaimed_bytes);
    assert_eq!(store.recent_tool_cleanup_receipts(10)?, vec![receipt]);
    assert!(store.mark_tool_cleanup_executing(plan.id).is_err());
    Ok(())
}
