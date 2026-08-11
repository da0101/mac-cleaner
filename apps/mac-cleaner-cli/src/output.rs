use cleaner_domain::CleanupCandidate;

pub(crate) fn print_scan(candidates: &[CleanupCandidate]) {
    println!("Mac Cleaner — reviewable opportunities");
    if candidates.is_empty() {
        println!("No eligible entries were found. Nothing was changed.");
        return;
    }
    for candidate in candidates {
        println!("\n{}", candidate.display_name);
        println!("  ID:       {}", candidate.id.0);
        println!("  Path:     {}", candidate.lexical_path.display());
        println!("  Size:     {}", format_bytes(candidate.bytes));
        println!("  Recovery: {}", candidate.recovery);
    }
    println!("\nNothing was changed. Select exact IDs with `mac-cleaner clean --candidate ID`.");
}

pub(crate) fn print_plan(plan: &cleaner_domain::CleanupPlan) {
    println!("Mac Cleaner cleanup plan {}", plan.id);
    println!("Expires: {}", plan.expires_at);
    println!("Mode: {:?}", plan.execution_mode);
    println!(
        "Impact: {} across {} filesystem entries",
        format_bytes(plan.total_bytes),
        plan.total_files
    );
    for item in &plan.items {
        println!("\n  {}", item.lexical_path.display());
        println!("  Why: {}", item.reason);
        println!("  Recovery: {}", item.recovery);
    }
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut unit = 0_usize;
    let mut divisor = 1_u64;
    while bytes / divisor >= 1_000 && unit < UNITS.len() - 1 {
        divisor = divisor.saturating_mul(1_000);
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        let whole = bytes / divisor;
        let decimal = (bytes % divisor).saturating_mul(10) / divisor;
        format!("{whole}.{decimal} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::format_bytes;

    #[test]
    fn byte_format_is_human_readable() {
        assert_eq!(format_bytes(999), "999 B");
        assert_eq!(format_bytes(1_500), "1.5 KB");
    }
}
