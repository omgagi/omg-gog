pub mod mode;
pub mod json;
pub mod plain;
pub mod text;
pub mod transform;

use serde::{Deserialize, Serialize};

/// Output mode resolved from flags, env vars, and TTY detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Structured JSON (--json or GOG_JSON=1 or GOG_AUTO_JSON piped)
    Json,
    /// Tab-separated values, stable for scripting (--plain or GOG_PLAIN=1)
    Plain,
    /// Human-friendly text with colors and alignment (default on TTY)
    Text,
}

/// JSON post-processing transforms.
#[derive(Debug, Clone, Default)]
pub struct JsonTransform {
    /// Strip envelope fields, emit only primary results.
    pub results_only: bool,
    /// Project objects to these fields only (supports dot-path notation).
    pub select: Vec<String>,
}

/// Resolve the output mode from flags and TTY detection.
pub fn resolve_mode(json_flag: bool, plain_flag: bool, _is_tty: bool) -> anyhow::Result<OutputMode> {
    if json_flag && plain_flag {
        anyhow::bail!("cannot use both --json and --plain");
    }
    if json_flag {
        return Ok(OutputMode::Json);
    }
    if plain_flag {
        return Ok(OutputMode::Plain);
    }
    // Default to Text mode
    Ok(OutputMode::Text)
}

/// Write JSON output with optional transforms.
pub fn write_json<T: Serialize>(
    writer: &mut impl std::io::Write,
    value: &T,
    xform: &JsonTransform,
) -> anyhow::Result<()> {
    use std::io::Write;
    let mut json_value = serde_json::to_value(value)?;

    if xform.results_only {
        json_value = transform::unwrap_primary(json_value);
    }

    if !xform.select.is_empty() {
        json_value = transform::select_fields(json_value, &xform.select);
    }

    let output = serde_json::to_string_pretty(&json_value)?;
    writeln!(writer, "{}", output)?;
    Ok(())
}

/// Write plain/TSV output.
pub fn write_plain(
    writer: &mut impl std::io::Write,
    rows: &[Vec<String>],
) -> anyhow::Result<()> {
    use std::io::Write;
    for row in rows {
        writeln!(writer, "{}", row.join("\t"))?;
    }
    Ok(())
}

/// Trait for human-friendly text output.
pub trait TextOutput {
    fn write_text(&self, w: &mut impl std::io::Write, use_color: bool) -> anyhow::Result<()>;
}

/// Trait for plain/TSV output.
pub trait PlainOutput {
    fn write_plain(&self, w: &mut impl std::io::Write) -> anyhow::Result<()>;
}
