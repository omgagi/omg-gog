pub mod mode;
pub mod json;
pub mod plain;
pub mod text;
pub mod transform;

use serde::Serialize;

/// Output mode resolved from flags, env vars, and TTY detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Structured JSON (--json or GOG_JSON=1 or GOG_AUTO_JSON piped)
    Json,
    /// Tab-separated values, stable for scripting (--plain or GOG_PLAIN=1)
    Plain,
    /// Human-friendly text with colors and alignment (default on TTY)
    Text,
    /// Comma-separated values (--csv or GOG_CSV=1)
    Csv,
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
/// When GOG_AUTO_JSON is truthy and stdout is not a TTY, defaults to JSON.
pub fn resolve_mode(json_flag: bool, plain_flag: bool, is_tty: bool) -> anyhow::Result<OutputMode> {
    if json_flag && plain_flag {
        anyhow::bail!("cannot use both --json and --plain");
    }
    if json_flag {
        return Ok(OutputMode::Json);
    }
    if plain_flag {
        return Ok(OutputMode::Plain);
    }
    // GOG_AUTO_JSON: when stdout is not a TTY, default to JSON
    if !is_tty && crate::cli::env_bool("GOG_AUTO_JSON") {
        return Ok(OutputMode::Json);
    }
    // Default to Text mode
    Ok(OutputMode::Text)
}

/// Resolve output mode with CSV support.
/// When GOG_AUTO_JSON is truthy and stdout is not a TTY, defaults to JSON.
pub fn resolve_mode_full(json_flag: bool, plain_flag: bool, csv_flag: bool, is_tty: bool) -> anyhow::Result<OutputMode> {
    let exclusive_count = [json_flag, plain_flag, csv_flag].iter().filter(|&&f| f).count();
    if exclusive_count > 1 {
        anyhow::bail!("--json, --plain, and --csv are mutually exclusive");
    }
    if json_flag {
        return Ok(OutputMode::Json);
    }
    if plain_flag {
        return Ok(OutputMode::Plain);
    }
    if csv_flag {
        return Ok(OutputMode::Csv);
    }
    // GOG_AUTO_JSON: when stdout is not a TTY, default to JSON
    if !is_tty && crate::cli::env_bool("GOG_AUTO_JSON") {
        return Ok(OutputMode::Json);
    }
    Ok(OutputMode::Text)
}

/// Write CSV output. Header row followed by data rows, comma-separated.
/// Fields containing commas, quotes, or newlines are quoted.
pub fn write_csv(
    writer: &mut impl std::io::Write,
    headers: &[String],
    rows: &[Vec<String>],
) -> anyhow::Result<()> {
    writeln!(writer, "{}", headers.iter().map(|h| csv_escape(h)).collect::<Vec<_>>().join(","))?;
    for row in rows {
        writeln!(writer, "{}", row.iter().map(|f| csv_escape(f)).collect::<Vec<_>>().join(","))?;
    }
    Ok(())
}

/// Escape a CSV field: quote if it contains comma, quote, or newline.
pub fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Write JSON output with optional transforms.
pub fn write_json<T: Serialize>(
    writer: &mut impl std::io::Write,
    value: &T,
    xform: &JsonTransform,
) -> anyhow::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-OUTPUT-006: CSV output mode
    // ---------------------------------------------------------------

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: CSV mode exists in OutputMode enum
    #[test]
    fn req_output_006_csv_mode_exists() {
        let mode = OutputMode::Csv;
        assert_eq!(mode, OutputMode::Csv);
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: write_csv produces correct output
    #[test]
    fn req_output_006_write_csv_basic() {
        let headers = vec!["name".to_string(), "value".to_string()];
        let rows = vec![
            vec!["foo".to_string(), "1".to_string()],
            vec!["bar".to_string(), "2".to_string()],
        ];
        let mut buf = Vec::new();
        write_csv(&mut buf, &headers, &rows).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, "name,value\nfoo,1\nbar,2\n");
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: CSV escapes fields with commas
    #[test]
    fn req_output_006_csv_escape_comma() {
        let result = csv_escape("hello, world");
        assert_eq!(result, "\"hello, world\"");
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: CSV escapes fields with quotes
    #[test]
    fn req_output_006_csv_escape_quotes() {
        let result = csv_escape("say \"hello\"");
        assert_eq!(result, "\"say \"\"hello\"\"\"");
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: CSV escapes fields with newlines
    #[test]
    fn req_output_006_csv_escape_newline() {
        let result = csv_escape("line1\nline2");
        assert_eq!(result, "\"line1\nline2\"");
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: CSV does not escape plain fields
    #[test]
    fn req_output_006_csv_no_escape_plain() {
        let result = csv_escape("simple");
        assert_eq!(result, "simple");
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: write_csv with escaped fields
    #[test]
    fn req_output_006_write_csv_escaped() {
        let headers = vec!["name".to_string(), "description".to_string()];
        let rows = vec![vec!["test".to_string(), "has, comma".to_string()]];
        let mut buf = Vec::new();
        write_csv(&mut buf, &headers, &rows).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("\"has, comma\""));
    }

    // ---------------------------------------------------------------
    // REQ-OUTPUT-006: resolve_mode_full
    // ---------------------------------------------------------------

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: CSV flag resolves to Csv mode
    #[test]
    fn req_output_006_resolve_csv() {
        let mode = resolve_mode_full(false, false, true, true).unwrap();
        assert_eq!(mode, OutputMode::Csv);
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: Mutually exclusive flags error
    #[test]
    fn req_output_006_resolve_exclusive() {
        assert!(resolve_mode_full(true, true, false, true).is_err());
        assert!(resolve_mode_full(true, false, true, true).is_err());
        assert!(resolve_mode_full(false, true, true, true).is_err());
        assert!(resolve_mode_full(true, true, true, true).is_err());
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: No flags defaults to Text
    #[test]
    fn req_output_006_resolve_default_text() {
        let mode = resolve_mode_full(false, false, false, true).unwrap();
        assert_eq!(mode, OutputMode::Text);
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: JSON flag still works in full resolver
    #[test]
    fn req_output_006_resolve_json() {
        let mode = resolve_mode_full(true, false, false, true).unwrap();
        assert_eq!(mode, OutputMode::Json);
    }

    // Requirement: REQ-OUTPUT-006 (Could)
    // Acceptance: Plain flag still works in full resolver
    #[test]
    fn req_output_006_resolve_plain() {
        let mode = resolve_mode_full(false, true, false, true).unwrap();
        assert_eq!(mode, OutputMode::Plain);
    }
}
