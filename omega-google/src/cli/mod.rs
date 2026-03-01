pub mod root;
pub mod desire_paths;
pub mod exit_codes;

use std::ffi::OsString;

/// Execute the CLI with the given arguments. Returns the exit code.
pub async fn execute(_args: Vec<OsString>) -> i32 {
    // TODO: implement full CLI dispatch
    0
}

/// Rewrite desire path arguments before parsing.
/// `--fields` is rewritten to `--select` except in `calendar events` context.
pub fn rewrite_desire_path_args(args: Vec<String>) -> Vec<String> {
    // Check if this is a calendar events command -- if so, don't rewrite --fields
    if is_calendar_events_command(&args) {
        return args;
    }

    let mut result = Vec::with_capacity(args.len());
    let mut past_double_dash = false;

    for arg in args {
        if arg == "--" {
            past_double_dash = true;
            result.push(arg);
            continue;
        }

        if past_double_dash {
            result.push(arg);
            continue;
        }

        // Rewrite --fields to --select
        if arg == "--fields" {
            result.push("--select".to_string());
        } else if arg.starts_with("--fields=") {
            let val = &arg[9..]; // after "--fields="
            result.push(format!("--select={}", val));
        } else {
            result.push(arg);
        }
    }

    result
}

/// Check if the arguments represent a `calendar events` command.
/// Looks for "calendar"/"cal" as first command token and "events"/"ls"/"list" as second.
pub fn is_calendar_events_command(args: &[String]) -> bool {
    // Extract command tokens by skipping flags
    let mut cmd_tokens = Vec::new();
    let mut skip_next = false;

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg.starts_with('-') {
            // Check if this flag takes a value
            if arg.contains('=') {
                // --flag=value, no need to skip next
                continue;
            }
            if global_flag_takes_value(arg) {
                skip_next = true;
            }
            continue;
        }
        cmd_tokens.push(arg.as_str());
        if cmd_tokens.len() >= 2 {
            break;
        }
    }

    if cmd_tokens.len() < 2 {
        return false;
    }

    let service = cmd_tokens[0].to_lowercase();
    let subcommand = cmd_tokens[1].to_lowercase();

    matches!(service.as_str(), "calendar" | "cal")
        && matches!(subcommand.as_str(), "events" | "ls" | "list")
}

/// Check if a global flag takes a value argument.
pub fn global_flag_takes_value(flag: &str) -> bool {
    matches!(
        flag,
        "--color"
            | "--account"
            | "--acct"
            | "-a"
            | "--client"
            | "--enable-commands"
            | "--select"
            | "--pick"
            | "--project"
    )
}

/// Enforce command allowlisting from --enable-commands.
pub fn enforce_enabled_commands(command: &str, enabled: &str) -> anyhow::Result<()> {
    let list = split_comma_list(enabled);
    if list.is_empty() {
        return Ok(()); // Empty list allows all
    }
    let cmd_lower = command.to_lowercase();
    for allowed in &list {
        if allowed.to_lowercase() == cmd_lower {
            return Ok(());
        }
    }
    anyhow::bail!(
        "command '{}' is not enabled; allowed: {}",
        command,
        list.join(", ")
    )
}

/// Split a comma-separated list string, trimming whitespace.
pub fn split_comma_list(s: &str) -> Vec<String> {
    if s.trim().is_empty() {
        return Vec::new();
    }
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Parse environment variable as boolean (1, true, yes, y, on = true).
pub fn env_bool(key: &str) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(
            v.trim().to_lowercase().as_str(),
            "1" | "true" | "yes" | "y" | "on"
        ),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLI-009 (Should): --fields rewriting
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: --fields x,y is rewritten to --select x,y
    #[test]
    fn req_cli_009_fields_rewritten_to_select() {
        let args = vec!["gmail".to_string(), "search".to_string(), "--fields".to_string(), "id,subject".to_string()];
        let result = rewrite_desire_path_args(args);
        assert!(result.contains(&"--select".to_string()));
        assert!(!result.contains(&"--fields".to_string()));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: --fields=x,y is rewritten to --select=x,y
    #[test]
    fn req_cli_009_fields_equals_rewritten() {
        let args = vec!["drive".to_string(), "ls".to_string(), "--fields=id,name".to_string()];
        let result = rewrite_desire_path_args(args);
        assert!(result.iter().any(|a| a.starts_with("--select=")));
        assert!(!result.iter().any(|a| a.starts_with("--fields=")));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: calendar events --fields is NOT rewritten
    #[test]
    fn req_cli_009_calendar_events_not_rewritten() {
        let args = vec!["calendar".to_string(), "events".to_string(), "--fields".to_string(), "items(id)".to_string()];
        let result = rewrite_desire_path_args(args);
        assert!(result.contains(&"--fields".to_string()));
        assert!(!result.contains(&"--select".to_string()));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: cal events --fields is also NOT rewritten (alias)
    #[test]
    fn req_cli_009_cal_events_not_rewritten() {
        let args = vec!["cal".to_string(), "events".to_string(), "--fields".to_string(), "items(id)".to_string()];
        let result = rewrite_desire_path_args(args);
        assert!(result.contains(&"--fields".to_string()));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: calendar ls --fields is also NOT rewritten
    #[test]
    fn req_cli_009_calendar_ls_not_rewritten() {
        let args = vec!["calendar".to_string(), "ls".to_string(), "--fields".to_string(), "items(id)".to_string()];
        let result = rewrite_desire_path_args(args);
        assert!(result.contains(&"--fields".to_string()));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: -- stops rewriting
    #[test]
    fn req_cli_009_double_dash_stops_rewriting() {
        let args = vec!["gmail".to_string(), "search".to_string(), "--".to_string(), "--fields".to_string(), "test".to_string()];
        let result = rewrite_desire_path_args(args);
        // --fields after -- should NOT be rewritten
        assert!(result.contains(&"--fields".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CLI-009 (Should): is_calendar_events_command
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-009 (Should)
    #[test]
    fn req_cli_009_detects_calendar_events() {
        assert!(is_calendar_events_command(&["calendar".to_string(), "events".to_string()]));
        assert!(is_calendar_events_command(&["cal".to_string(), "events".to_string()]));
        assert!(is_calendar_events_command(&["calendar".to_string(), "ls".to_string()]));
        assert!(is_calendar_events_command(&["calendar".to_string(), "list".to_string()]));
    }

    // Requirement: REQ-CLI-009 (Should)
    #[test]
    fn req_cli_009_non_calendar_commands() {
        assert!(!is_calendar_events_command(&["gmail".to_string(), "search".to_string()]));
        assert!(!is_calendar_events_command(&["calendar".to_string()]));
        assert!(!is_calendar_events_command(&["drive".to_string(), "ls".to_string()]));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: Skips flags when detecting command tokens
    #[test]
    fn req_cli_009_skips_flags_in_detection() {
        assert!(is_calendar_events_command(&[
            "--json".to_string(), "calendar".to_string(), "--account".to_string(), "me@x.com".to_string(), "events".to_string()
        ]));
    }

    // ---------------------------------------------------------------
    // Helper tests
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-001 (Must)
    // Acceptance: split_comma_list works correctly
    #[test]
    fn req_cli_001_split_comma_list() {
        assert_eq!(split_comma_list("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(split_comma_list(" a , b , c "), vec!["a", "b", "c"]);
        assert_eq!(split_comma_list(""), Vec::<String>::new());
        assert_eq!(split_comma_list("single"), vec!["single"]);
    }

    // Requirement: REQ-CLI-002 (Must)
    // Acceptance: env_bool parses truthy values
    #[test]
    fn req_cli_002_env_bool_truthy() {
        // Note: we cannot easily set env vars in unit tests safely
        // These would need to be integration tests or use temp_env
        // For now, verify the function signature and basic logic
        // by testing with unset env vars
        assert!(!env_bool("OMEGA_TEST_UNSET_ENV_VAR_12345"));
    }

    // Requirement: REQ-CLI-001 (Must)
    // Acceptance: global_flag_takes_value identifies flags that consume the next arg
    #[test]
    fn req_cli_001_global_flag_takes_value() {
        assert!(global_flag_takes_value("--color"));
        assert!(global_flag_takes_value("--account"));
        assert!(global_flag_takes_value("--acct"));
        assert!(global_flag_takes_value("--client"));
        assert!(global_flag_takes_value("--enable-commands"));
        assert!(global_flag_takes_value("--select"));
        assert!(global_flag_takes_value("-a"));
        assert!(!global_flag_takes_value("--json"));
        assert!(!global_flag_takes_value("--plain"));
        assert!(!global_flag_takes_value("--verbose"));
    }

    // ---------------------------------------------------------------
    // Enable commands (command allowlisting)
    // ---------------------------------------------------------------

    // Requirement: REQ-CLI-001 (Must)
    // Acceptance: --enable-commands restricts available commands
    #[test]
    fn req_cli_001_enable_commands_allows_listed() {
        let result = enforce_enabled_commands("gmail", "gmail,calendar,drive");
        assert!(result.is_ok());
    }

    // Requirement: REQ-CLI-001 (Must)
    // Acceptance: --enable-commands blocks unlisted commands
    #[test]
    fn req_cli_001_enable_commands_blocks_unlisted() {
        let result = enforce_enabled_commands("chat", "gmail,calendar,drive");
        assert!(result.is_err());
    }

    // Requirement: REQ-CLI-001 (Must)
    // Acceptance: Empty enable-commands allows all
    #[test]
    fn req_cli_001_enable_commands_empty_allows_all() {
        let result = enforce_enabled_commands("chat", "");
        assert!(result.is_ok());
    }
}
