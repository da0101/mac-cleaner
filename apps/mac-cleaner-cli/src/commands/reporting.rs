use anyhow::Result;
use cleaner_service::AppService;

use crate::output::{format_bytes, print_scan};

pub(crate) async fn run_status(service: &AppService, json: bool) -> Result<()> {
    let snapshot = service.system_snapshot().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&snapshot)?);
    } else {
        println!("Mac Cleaner — read-only system status");
        println!(
            "Memory: {} available of {} ({:?} pressure)",
            format_bytes(snapshot.memory.available_bytes),
            format_bytes(snapshot.memory.total_bytes),
            snapshot.memory.pressure,
        );
        println!(
            "        {} reclaimable, {} compressed, {} swap, {} completely free",
            format_bytes(snapshot.memory.reclaimable_bytes),
            format_bytes(snapshot.memory.compressed_bytes),
            format_bytes(snapshot.memory.swap_used_bytes),
            format_bytes(snapshot.memory.free_bytes),
        );
        println!(
            "Disk:   {} available of {}",
            format_bytes(snapshot.disk.available_bytes),
            format_bytes(snapshot.disk.total_bytes)
        );
    }
    Ok(())
}

pub(crate) async fn run_scan(service: &AppService, json: bool) -> Result<()> {
    let report = service.scan().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_scan(&report.candidates);
        if !report.skipped.is_empty() {
            println!(
                "\nProtected/skipped entries: {} (use --json for reasons)",
                report.skipped.len()
            );
        }
    }
    Ok(())
}

pub(crate) fn run_history(service: &AppService, limit: u32, json: bool) -> Result<()> {
    let receipts = service.recent_history(limit)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&receipts)?);
    } else if receipts.is_empty() {
        println!("No completed Rust cleanup operations.");
    } else {
        for receipt in receipts {
            println!(
                "{}  {}  {:?}  {} item(s)",
                receipt.completed_at,
                receipt.plan_id,
                receipt.mode,
                receipt.items.len()
            );
        }
    }
    Ok(())
}
