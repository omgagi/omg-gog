//! Shell completion generation command.
//!
//! REQ-CLI-020: `completion <shell>` generates shell completion scripts
//! using clap_complete.

use clap::Args;

/// Supported shell types for completion generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl std::str::FromStr for ShellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(ShellType::Bash),
            "zsh" => Ok(ShellType::Zsh),
            "fish" => Ok(ShellType::Fish),
            "powershell" | "ps" => Ok(ShellType::PowerShell),
            _ => Err(format!(
                "unknown shell: '{}'. Valid shells: bash, zsh, fish, powershell",
                s
            )),
        }
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::Bash => write!(f, "bash"),
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::Fish => write!(f, "fish"),
            ShellType::PowerShell => write!(f, "powershell"),
        }
    }
}

/// Arguments for the `completion` command.
#[derive(Args, Debug)]
pub struct CompletionArgs {
    /// Shell to generate completions for (bash, zsh, fish, powershell)
    pub shell: String,
}

/// Generate shell completions and write to the given writer.
pub fn generate_completions(
    shell_str: &str,
    writer: &mut impl std::io::Write,
) -> Result<(), String> {
    let shell: ShellType = shell_str.parse()?;

    let clap_shell = match shell {
        ShellType::Bash => clap_complete::Shell::Bash,
        ShellType::Zsh => clap_complete::Shell::Zsh,
        ShellType::Fish => clap_complete::Shell::Fish,
        ShellType::PowerShell => clap_complete::Shell::PowerShell,
    };

    let mut cmd = <super::root::Cli as clap::CommandFactory>::command();
    clap_complete::generate(clap_shell, &mut cmd, "omega-google", writer);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLI-020: Shell type parsing
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-020 (Must)
    // Acceptance: Parse valid shell names
    #[test]
    fn req_cli_020_shell_type_parsing() {
        assert_eq!("bash".parse::<ShellType>().unwrap(), ShellType::Bash);
        assert_eq!("zsh".parse::<ShellType>().unwrap(), ShellType::Zsh);
        assert_eq!("fish".parse::<ShellType>().unwrap(), ShellType::Fish);
        assert_eq!(
            "powershell".parse::<ShellType>().unwrap(),
            ShellType::PowerShell
        );
        assert_eq!("ps".parse::<ShellType>().unwrap(), ShellType::PowerShell);
    }

    // Requirement: REQ-CLI-020 (Must)
    // Acceptance: Case insensitive parsing
    #[test]
    fn req_cli_020_shell_type_case_insensitive() {
        assert_eq!("BASH".parse::<ShellType>().unwrap(), ShellType::Bash);
        assert_eq!("Zsh".parse::<ShellType>().unwrap(), ShellType::Zsh);
        assert_eq!("FISH".parse::<ShellType>().unwrap(), ShellType::Fish);
        assert_eq!(
            "PowerShell".parse::<ShellType>().unwrap(),
            ShellType::PowerShell
        );
    }

    // Requirement: REQ-CLI-020 (Must)
    // Acceptance: Invalid shell returns error
    #[test]
    fn req_cli_020_shell_type_invalid() {
        assert!("invalid".parse::<ShellType>().is_err());
        assert!("".parse::<ShellType>().is_err());
        assert!("tcsh".parse::<ShellType>().is_err());
    }

    // Requirement: REQ-CLI-020 (Must)
    // Acceptance: Shell type display
    #[test]
    fn req_cli_020_shell_type_display() {
        assert_eq!(format!("{}", ShellType::Bash), "bash");
        assert_eq!(format!("{}", ShellType::Zsh), "zsh");
        assert_eq!(format!("{}", ShellType::Fish), "fish");
        assert_eq!(format!("{}", ShellType::PowerShell), "powershell");
    }

    // Requirement: REQ-CLI-020 (Must)
    // Acceptance: Generate bash completions (non-empty output)
    #[test]
    fn req_cli_020_generate_bash() {
        let mut buf = Vec::new();
        generate_completions("bash", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty(), "bash completions should not be empty");
        assert!(
            output.contains("omega-google"),
            "should reference the binary name"
        );
    }

    // Requirement: REQ-CLI-020 (Must)
    // Acceptance: Generate zsh completions (non-empty output)
    #[test]
    fn req_cli_020_generate_zsh() {
        let mut buf = Vec::new();
        generate_completions("zsh", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty(), "zsh completions should not be empty");
    }

    // Requirement: REQ-CLI-020 (Must)
    // Acceptance: Generate fish completions (non-empty output)
    #[test]
    fn req_cli_020_generate_fish() {
        let mut buf = Vec::new();
        generate_completions("fish", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty(), "fish completions should not be empty");
    }

    // Requirement: REQ-CLI-020 (Must)
    // Acceptance: Generate powershell completions (non-empty output)
    #[test]
    fn req_cli_020_generate_powershell() {
        let mut buf = Vec::new();
        generate_completions("powershell", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(
            !output.is_empty(),
            "powershell completions should not be empty"
        );
    }

    // Requirement: REQ-CLI-020 (Must)
    // Acceptance: Invalid shell returns error
    #[test]
    fn req_cli_020_generate_invalid_shell() {
        let mut buf = Vec::new();
        let result = generate_completions("invalid", &mut buf);
        assert!(result.is_err());
    }
}
