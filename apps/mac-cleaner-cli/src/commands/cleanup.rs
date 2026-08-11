use anyhow::Result;
use cleaner_domain::ExecutionMode;
use cleaner_service::AppService;

use crate::confirmation::require_typed_confirmation;
use crate::output::print_plan;

pub(crate) async fn run(service: &AppService, candidate: Vec<String>, dry_run: bool) -> Result<()> {
    service.scan().await?;
    let selected = candidate
        .into_iter()
        .map(cleaner_domain::CandidateId)
        .collect::<Vec<_>>();
    let mode = if dry_run {
        ExecutionMode::DryRun
    } else {
        ExecutionMode::Quarantine
    };
    let plan = service.prepare_plan(&selected, mode)?;
    print_plan(&plan);
    require_typed_confirmation(plan.id)?;
    service.approve_plan(plan.id)?;
    let receipt = service.execute_approved(plan.id).await?;
    println!(
        "\nCompleted {} item(s). Mode: {:?}.",
        receipt.items.len(),
        receipt.mode
    );
    if receipt.mode == ExecutionMode::Quarantine {
        println!("Files are in Mac Cleaner's private quarantine and remain recoverable.");
    }
    Ok(())
}
