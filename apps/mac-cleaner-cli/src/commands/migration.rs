use std::path::Path;

use anyhow::Result;
use cleaner_service::AppService;

pub(crate) fn run(service: &AppService, from: &Path, json: bool) -> Result<()> {
    let summary = service.import_legacy(from)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
        return Ok(());
    }

    println!("Legacy import from {}", summary.source_root.display());
    println!("Imported settings: {}", summary.imported_settings.len());
    println!(
        "Imported history entries: {}",
        summary.imported_history_entries
    );
    println!(
        "Ignored retired settings: {}",
        summary.ignored_settings.len()
    );
    if summary.already_imported {
        println!("This source was already imported; no duplicate data was created.");
    } else {
        println!("The original JSON files were preserved unchanged.");
    }
    Ok(())
}
