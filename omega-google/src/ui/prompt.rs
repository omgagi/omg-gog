// Confirmation prompts (respects --force, --no-input)
//
// The primary confirm logic lives in ui/mod.rs via Ui::confirm().
// This module provides additional prompt utilities.

use std::io::Write;

/// Prompt the user for a yes/no confirmation on stderr.
/// Returns true for yes, false for no.
///
/// If force is true, returns true without prompting.
/// If no_input is true, returns an error.
pub fn confirm(prompt: &str, force: bool, no_input: bool) -> anyhow::Result<bool> {
    if force {
        return Ok(true);
    }
    if no_input {
        anyhow::bail!("confirmation required but --no-input is set");
    }

    eprint!("{} [y/N] ", prompt);
    std::io::stderr().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    Ok(trimmed == "y" || trimmed == "yes")
}

/// Prompt the user for text input on stderr.
/// Returns the input string (trimmed).
///
/// If no_input is true, returns an error.
pub fn prompt_input(prompt: &str, no_input: bool) -> anyhow::Result<String> {
    if no_input {
        anyhow::bail!("input required but --no-input is set");
    }

    eprint!("{}: ", prompt);
    std::io::stderr().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
