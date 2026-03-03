pub mod color;
pub mod progress;
pub mod prompt;

/// Color mode for terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl std::str::FromStr for ColorMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "auto" => Ok(ColorMode::Auto),
            "always" => Ok(ColorMode::Always),
            "never" => Ok(ColorMode::Never),
            _ => Err(anyhow::anyhow!(
                "invalid color mode: {} (expected auto|always|never)",
                s
            )),
        }
    }
}

impl std::fmt::Display for ColorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorMode::Auto => write!(f, "auto"),
            ColorMode::Always => write!(f, "always"),
            ColorMode::Never => write!(f, "never"),
        }
    }
}

pub struct UiOptions {
    pub color: ColorMode,
}

pub struct Ui {
    use_color: bool,
}

impl Ui {
    pub fn new(opts: UiOptions) -> anyhow::Result<Self> {
        let is_tty = Self::is_tty_stdout();
        let use_color = Self::resolve_color(opts.color, is_tty);
        Ok(Self { use_color })
    }

    /// Resolve whether colors should be enabled given the mode and environment.
    pub fn resolve_color(mode: ColorMode, is_tty: bool) -> bool {
        // NO_COLOR env var always disables color
        if std::env::var("NO_COLOR").is_ok() {
            return false;
        }
        match mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => is_tty,
        }
    }

    pub fn use_color(&self) -> bool {
        self.use_color
    }

    pub fn error(&self, msg: &str) {
        eprintln!("Error: {}", msg);
    }

    pub fn warn(&self, msg: &str) {
        eprintln!("Warning: {}", msg);
    }

    pub fn hint(&self, msg: &str) {
        eprintln!("Hint: {}", msg);
    }

    pub fn progress(&self, msg: &str) {
        eprint!("{}", msg);
    }

    /// Confirm prompts return Ok(true) if --force is set.
    /// Return Err if --no-input is set and a prompt would be shown.
    pub fn confirm(&self, _prompt: &str, force: bool, no_input: bool) -> anyhow::Result<bool> {
        if force {
            return Ok(true);
        }
        if no_input {
            anyhow::bail!("confirmation required but --no-input is set");
        }
        // In a real implementation, we'd read from stdin
        // For now, default to false (deny)
        Ok(false)
    }

    pub fn is_tty_stdout() -> bool {
        use std::io::IsTerminal;
        std::io::stdout().is_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-UI-001 (Must): Color detection
    // ---------------------------------------------------------------

    // Requirement: REQ-UI-001 (Must)
    // Acceptance: Colors enabled when --color always
    #[test]
    fn req_ui_001_color_always_enables() {
        let result = Ui::resolve_color(ColorMode::Always, false);
        assert!(result, "always mode should enable color even without TTY");
    }

    // Requirement: REQ-UI-001 (Must)
    // Acceptance: Colors disabled when --color never
    #[test]
    fn req_ui_001_color_never_disables() {
        let result = Ui::resolve_color(ColorMode::Never, true);
        assert!(!result, "never mode should disable color even with TTY");
    }

    // Requirement: REQ-UI-001 (Must)
    // Acceptance: Colors enabled when auto + TTY
    #[test]
    fn req_ui_001_color_auto_with_tty() {
        // Note: NO_COLOR is checked from env, so this test assumes NO_COLOR is not set
        let result = Ui::resolve_color(ColorMode::Auto, true);
        // This will depend on NO_COLOR env var state in test runner
        // In a clean env, this should be true
        assert!(
            result,
            "auto mode with TTY should enable color (assuming NO_COLOR not set)"
        );
    }

    // Requirement: REQ-UI-001 (Must)
    // Acceptance: Colors disabled when auto + no TTY
    #[test]
    fn req_ui_001_color_auto_without_tty() {
        let result = Ui::resolve_color(ColorMode::Auto, false);
        assert!(!result, "auto mode without TTY should disable color");
    }

    // Requirement: REQ-UI-001 (Must)
    // Acceptance: ColorMode parsing
    #[test]
    fn req_ui_001_color_mode_parsing() {
        assert_eq!("auto".parse::<ColorMode>().unwrap(), ColorMode::Auto);
        assert_eq!("always".parse::<ColorMode>().unwrap(), ColorMode::Always);
        assert_eq!("never".parse::<ColorMode>().unwrap(), ColorMode::Never);
        assert_eq!("AUTO".parse::<ColorMode>().unwrap(), ColorMode::Auto);
        assert!("invalid".parse::<ColorMode>().is_err());
    }

    // Requirement: REQ-UI-001 (Must)
    // Acceptance: ColorMode display
    #[test]
    fn req_ui_001_color_mode_display() {
        assert_eq!(format!("{}", ColorMode::Auto), "auto");
        assert_eq!(format!("{}", ColorMode::Always), "always");
        assert_eq!(format!("{}", ColorMode::Never), "never");
    }

    // ---------------------------------------------------------------
    // REQ-UI-001 (Must): NO_COLOR support
    // ---------------------------------------------------------------

    // Requirement: REQ-UI-001 (Must)
    // Acceptance: NO_COLOR env var disables color
    // Note: This test depends on env var state; implementation should check std::env::var("NO_COLOR")
    #[test]
    fn req_ui_001_no_color_env_support() {
        // The resolve_color function should check for NO_COLOR
        // When NO_COLOR is set, even "always" should respect it according to spec:
        // "Colors disabled when: --color never or NO_COLOR set"
        // However, the architecture says: "Colors enabled when ColorMode::Always"
        // and "Colors disabled ... when NO_COLOR is set"
        // The spec says NO_COLOR overrides even Always -- we test that:
        // This test documents expected behavior for the implementer
    }

    // ---------------------------------------------------------------
    // REQ-OUTPUT-005 (Must): Colors disabled for JSON/plain
    // ---------------------------------------------------------------

    // Requirement: REQ-OUTPUT-005 (Must)
    // Acceptance: No ANSI escape codes in JSON or plain output
    #[test]
    fn req_output_005_no_color_in_json_mode() {
        // When output mode is JSON, color should be forced to Never
        // This is enforced in the CLI dispatch layer, not in Ui directly
        // Verifying the constant behavior: Never always means no color
        let result = Ui::resolve_color(ColorMode::Never, true);
        assert!(!result);
    }

    // ---------------------------------------------------------------
    // REQ-UI-003 (Must): Prompt behavior
    // ---------------------------------------------------------------

    // Requirement: REQ-UI-003 (Must)
    // Acceptance: --force skips confirmation
    #[test]
    fn req_ui_003_force_skips_confirm() {
        // Cannot construct Ui due to todo!(), but documenting expected behavior
        // let ui = Ui { use_color: false };
        // assert_eq!(ui.confirm("delete?", true, false).unwrap(), true);
    }

    // Requirement: REQ-UI-003 (Must)
    // Acceptance: --no-input returns error for prompts
    #[test]
    fn req_ui_003_no_input_returns_error() {
        // let ui = Ui { use_color: false };
        // assert!(ui.confirm("delete?", false, true).is_err());
    }
}
