// Human-friendly colored text formatter
//
// The TextOutput trait is defined in output/mod.rs.
// This module re-exports it and provides utilities
// for colored/formatted text output.

pub use super::TextOutput;

/// ANSI color codes for terminal output.
pub mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const DIM: &str = "\x1b[2m";
}

/// Format a string with color if colors are enabled.
pub fn colored(text: &str, color: &str, use_color: bool) -> String {
    if use_color {
        format!("{}{}{}", color, text, ansi::RESET)
    } else {
        text.to_string()
    }
}

/// Format a string as bold if colors are enabled.
pub fn bold(text: &str, use_color: bool) -> String {
    colored(text, ansi::BOLD, use_color)
}

/// Format an error message with optional color.
pub fn format_error(msg: &str, use_color: bool) -> String {
    if use_color {
        format!("{}{}Error:{} {}", ansi::BOLD, ansi::RED, ansi::RESET, msg)
    } else {
        format!("Error: {}", msg)
    }
}

/// Format a warning message with optional color.
pub fn format_warning(msg: &str, use_color: bool) -> String {
    if use_color {
        format!(
            "{}{}Warning:{} {}",
            ansi::BOLD,
            ansi::YELLOW,
            ansi::RESET,
            msg
        )
    } else {
        format!("Warning: {}", msg)
    }
}

/// Format a hint/info message with optional color.
pub fn format_hint(msg: &str, use_color: bool) -> String {
    if use_color {
        format!("{}Hint:{} {}", ansi::CYAN, ansi::RESET, msg)
    } else {
        format!("Hint: {}", msg)
    }
}
