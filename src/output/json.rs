// JSON formatter with transforms
//
// The primary JSON output logic lives in output/mod.rs:
//   - write_json()
//
// This module re-exports that function and provides additional
// JSON formatting utilities.

pub use super::write_json;

/// Write a JSON value directly to a writer (no transforms).
pub fn write_json_raw(
    writer: &mut impl std::io::Write,
    value: &serde_json::Value,
) -> anyhow::Result<()> {
    let output = serde_json::to_string_pretty(value)?;
    writeln!(writer, "{}", output)?;
    Ok(())
}

/// Serialize a value to a pretty-printed JSON string.
pub fn to_pretty_json<T: serde::Serialize>(value: &T) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(value)?)
}
