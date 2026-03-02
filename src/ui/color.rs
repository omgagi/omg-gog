// Color detection and crossterm styling
//
// The primary color mode logic lives in ui/mod.rs:
//   - ColorMode enum
//   - Ui::resolve_color()
//
// This module re-exports those and provides additional
// color detection utilities.

pub use super::ColorMode;
pub use super::Ui;

/// Check if the terminal supports colors based on environment variables.
/// Returns true if colors should be enabled in auto mode.
pub fn terminal_supports_color() -> bool {
    // Check for common indicators that colors are NOT supported
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check TERM for dumb terminal
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Check if stdout is a TTY
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// Resolve whether to use color given a mode string and TTY detection.
pub fn should_use_color(mode: &str) -> bool {
    let color_mode: ColorMode = mode.parse().unwrap_or(ColorMode::Auto);
    Ui::resolve_color(color_mode, terminal_supports_color())
}
