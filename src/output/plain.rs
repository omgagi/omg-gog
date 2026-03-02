// TSV/plain formatter
//
// The primary plain output logic lives in output/mod.rs:
//   - write_plain()
//   - PlainOutput trait
//
// This module re-exports those and provides additional
// plain formatting utilities.

pub use super::write_plain;
pub use super::PlainOutput;

/// Convert a JSON value to plain tab-separated rows.
/// Objects become key-value rows; arrays of objects become header + data rows.
pub fn json_to_plain_rows(value: &serde_json::Value) -> Vec<Vec<String>> {
    match value {
        serde_json::Value::Object(map) => {
            map.iter()
                .map(|(k, v)| vec![k.clone(), value_to_string(v)])
                .collect()
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return vec![];
            }
            // Try to extract headers from first object
            if let Some(first_obj) = arr.first().and_then(|v| v.as_object()) {
                let headers: Vec<String> = first_obj.keys().cloned().collect();
                let mut rows = vec![headers.clone()];
                for item in arr {
                    let row: Vec<String> = headers
                        .iter()
                        .map(|h| {
                            item.get(h)
                                .map(value_to_string)
                                .unwrap_or_default()
                        })
                        .collect();
                    rows.push(row);
                }
                rows
            } else {
                arr.iter()
                    .map(|v| vec![value_to_string(v)])
                    .collect()
            }
        }
        other => vec![vec![value_to_string(other)]],
    }
}

fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}
