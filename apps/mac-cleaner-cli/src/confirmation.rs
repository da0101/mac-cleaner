use std::io::{self, Write};

use anyhow::{Result, bail};

pub(crate) fn require_typed_confirmation(plan_id: uuid::Uuid) -> Result<()> {
    print!("\nType the exact plan ID to approve, or press Return to cancel:\n> ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim() != plan_id.to_string() {
        bail!("cleanup cancelled; the plan ID did not match")
    }
    Ok(())
}
