use anyhow::Result;
use cleaner_domain::QuarantineAction;
use cleaner_service::AppService;

use crate::cli::QuarantineCommand;
use crate::confirmation::require_typed_confirmation;
use crate::output::format_bytes;

pub(crate) async fn run(service: &AppService, command: QuarantineCommand) -> Result<()> {
    match command {
        QuarantineCommand::List { limit, json } => list(service, limit, json)?,
        QuarantineCommand::Restore { entry } => {
            run_action(service, entry, QuarantineAction::Restore).await?;
        }
        QuarantineCommand::Purge { entry } => {
            run_action(service, entry, QuarantineAction::Purge).await?;
        }
    }
    Ok(())
}

fn list(service: &AppService, limit: u32, json: bool) -> Result<()> {
    let entries = service.active_quarantine(limit)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else if entries.is_empty() {
        println!("Mac Cleaner quarantine is empty.");
    } else {
        for entry in entries {
            println!("\n{}", entry.id);
            println!("  Original:   {}", entry.original_path.display());
            println!("  Quarantine: {}", entry.quarantine_path.display());
            println!("  Size:       {}", format_bytes(entry.bytes));
        }
        println!("\nQuarantined files still use disk space.");
    }
    Ok(())
}

async fn run_action(
    service: &AppService,
    entry: uuid::Uuid,
    action: QuarantineAction,
) -> Result<()> {
    let plan = service.prepare_quarantine_action(entry, action)?;
    println!("Mac Cleaner quarantine action {}", plan.id);
    println!("Action: {:?}", plan.action);
    println!("Quarantine: {}", plan.entry.quarantine_path.display());
    if action == QuarantineAction::Restore {
        println!("Restore to: {}", plan.entry.original_path.display());
    } else {
        println!("This permanently deletes one entry and cannot be undone.");
    }
    require_typed_confirmation(plan.id)?;
    service.approve_quarantine_action(plan.id)?;
    let completed = service.execute_quarantine_action(plan.id).await?;
    println!("Completed: {:?}", completed.state);
    Ok(())
}
