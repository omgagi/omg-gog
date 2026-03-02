// Progress hints on stderr
//
// The primary progress/hint output lives in ui/mod.rs via Ui::progress().
// This module provides additional structured progress reporting.

use std::io::Write;

/// Write a progress message to stderr.
pub fn progress(msg: &str) {
    eprint!("{}", msg);
    let _ = std::io::stderr().flush();
}

/// Write a progress message with a newline to stderr.
pub fn progress_ln(msg: &str) {
    eprintln!("{}", msg);
}

/// Write a status update to stderr (e.g., "Fetching page 2...").
pub fn status(operation: &str, detail: &str) {
    eprintln!("{}: {}", operation, detail);
}

/// Write a completion message to stderr.
pub fn done(msg: &str) {
    eprintln!("{}", msg);
}
