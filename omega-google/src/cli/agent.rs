//! Agent-oriented commands: exit-codes, schema/help-json.
//!
//! REQ-AGENT-001: `exit-codes` command prints the exit code table.
//! REQ-AGENT-002: `schema` command outputs the CLI command tree as JSON.

use clap::{Args, Subcommand};

/// Agent subcommands (also available as top-level commands).
#[derive(Args, Debug)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommand,
}

#[derive(Subcommand, Debug)]
pub enum AgentCommand {
    /// Print exit code table
    ExitCodes,

    /// Print machine-readable CLI schema as JSON
    Schema(SchemaArgs),
}

/// Arguments for the `schema` / `help-json` command.
#[derive(Args, Debug)]
pub struct SchemaArgs {
    /// Specific command to show schema for (omit for full tree)
    pub command: Option<String>,

    /// Include hidden commands and flags
    #[arg(long)]
    pub include_hidden: bool,
}

/// An exit code entry for display.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExitCodeEntry {
    pub code: i32,
    pub name: String,
    pub description: String,
}

/// Return the list of all exit codes.
pub fn exit_code_table() -> Vec<ExitCodeEntry> {
    use crate::error::exit::codes;

    vec![
        ExitCodeEntry {
            code: codes::SUCCESS,
            name: "SUCCESS".to_string(),
            description: "Command completed successfully".to_string(),
        },
        ExitCodeEntry {
            code: codes::GENERIC_ERROR,
            name: "GENERIC_ERROR".to_string(),
            description: "Unspecified error".to_string(),
        },
        ExitCodeEntry {
            code: codes::USAGE_ERROR,
            name: "USAGE_ERROR".to_string(),
            description: "Invalid arguments or usage".to_string(),
        },
        ExitCodeEntry {
            code: codes::EMPTY_RESULTS,
            name: "EMPTY_RESULTS".to_string(),
            description: "Query returned no results".to_string(),
        },
        ExitCodeEntry {
            code: codes::AUTH_REQUIRED,
            name: "AUTH_REQUIRED".to_string(),
            description: "Authentication required or token expired".to_string(),
        },
        ExitCodeEntry {
            code: codes::NOT_FOUND,
            name: "NOT_FOUND".to_string(),
            description: "Resource not found".to_string(),
        },
        ExitCodeEntry {
            code: codes::PERMISSION_DENIED,
            name: "PERMISSION_DENIED".to_string(),
            description: "Insufficient permissions".to_string(),
        },
        ExitCodeEntry {
            code: codes::RATE_LIMITED,
            name: "RATE_LIMITED".to_string(),
            description: "API rate limit exceeded".to_string(),
        },
        ExitCodeEntry {
            code: codes::RETRYABLE,
            name: "RETRYABLE".to_string(),
            description: "Transient error, safe to retry".to_string(),
        },
        ExitCodeEntry {
            code: codes::CONFIG_ERROR,
            name: "CONFIG_ERROR".to_string(),
            description: "Configuration file error".to_string(),
        },
        ExitCodeEntry {
            code: codes::CANCELLED,
            name: "CANCELLED".to_string(),
            description: "Operation cancelled (SIGINT)".to_string(),
        },
    ]
}

/// Schema entry for a CLI argument/flag.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArgSchema {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
    pub r#type: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    pub hidden: bool,
    pub global: bool,
}

/// Schema entry for a CLI command.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CommandSchema {
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<ArgSchema>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subcommands: Vec<CommandSchema>,
    pub hidden: bool,
}

/// Build the schema for a clap Command tree.
pub fn build_schema(cmd: &clap::Command, include_hidden: bool) -> CommandSchema {
    let args: Vec<ArgSchema> = cmd
        .get_arguments()
        .filter(|a| include_hidden || !a.is_hide_set())
        .filter(|a| a.get_id() != "help" && a.get_id() != "version")
        .map(|a| {
            let short = a.get_short().map(|c| format!("-{}", c));
            let type_name = if a.get_action().takes_values() {
                "string"
            } else {
                "bool"
            };
            let default = a
                .get_default_values()
                .first()
                .map(|v| v.to_string_lossy().to_string());
            let help = a.get_help().map(|h| h.to_string());

            ArgSchema {
                name: a.get_id().to_string(),
                short,
                r#type: type_name.to_string(),
                required: a.is_required_set(),
                default,
                help,
                hidden: a.is_hide_set(),
                global: a.is_global_set(),
            }
        })
        .collect();

    let subcommands: Vec<CommandSchema> = cmd
        .get_subcommands()
        .filter(|s| include_hidden || !s.is_hide_set())
        .filter(|s| s.get_name() != "help")
        .map(|s| build_schema(s, include_hidden))
        .collect();

    let aliases: Vec<String> = cmd
        .get_all_aliases()
        .map(|a| a.to_string())
        .collect();

    CommandSchema {
        name: cmd.get_name().to_string(),
        aliases,
        about: cmd.get_about().map(|a| a.to_string()),
        args,
        subcommands,
        hidden: cmd.is_hide_set(),
    }
}

/// Generate the full CLI schema, optionally filtered to a specific command.
pub fn generate_schema(
    command_filter: Option<&str>,
    include_hidden: bool,
) -> serde_json::Value {
    let cmd = <super::root::Cli as clap::CommandFactory>::command();

    let schema = if let Some(filter) = command_filter {
        // Find the specific subcommand
        if let Some(sub) = find_subcommand(&cmd, filter) {
            build_schema(sub, include_hidden)
        } else {
            // Return an error object if not found
            return serde_json::json!({
                "error": format!("unknown command: '{}'", filter)
            });
        }
    } else {
        build_schema(&cmd, include_hidden)
    };

    serde_json::to_value(&schema).unwrap_or_else(|e| {
        serde_json::json!({ "error": format!("serialization error: {}", e) })
    })
}

/// Find a subcommand by name (case-insensitive, supports aliases).
fn find_subcommand<'a>(cmd: &'a clap::Command, name: &str) -> Option<&'a clap::Command> {
    let lower = name.to_lowercase();
    cmd.get_subcommands().find(|s| {
        s.get_name().to_lowercase() == lower
            || s.get_all_aliases().any(|a| a.to_lowercase() == lower)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-AGENT-001: Exit codes table
    // ---------------------------------------------------------------

    // Requirement: REQ-AGENT-001 (Must)
    // Acceptance: Exit code table has all known codes
    #[test]
    fn req_agent_001_exit_code_table_complete() {
        let table = exit_code_table();
        assert!(!table.is_empty());

        let codes_present: Vec<i32> = table.iter().map(|e| e.code).collect();
        assert!(codes_present.contains(&0), "should contain SUCCESS=0");
        assert!(codes_present.contains(&1), "should contain GENERIC_ERROR=1");
        assert!(codes_present.contains(&2), "should contain USAGE_ERROR=2");
        assert!(codes_present.contains(&3), "should contain EMPTY_RESULTS=3");
        assert!(codes_present.contains(&4), "should contain AUTH_REQUIRED=4");
        assert!(codes_present.contains(&5), "should contain NOT_FOUND=5");
        assert!(codes_present.contains(&6), "should contain PERMISSION_DENIED=6");
        assert!(codes_present.contains(&7), "should contain RATE_LIMITED=7");
        assert!(codes_present.contains(&8), "should contain RETRYABLE=8");
        assert!(codes_present.contains(&10), "should contain CONFIG_ERROR=10");
        assert!(codes_present.contains(&130), "should contain CANCELLED=130");
    }

    // Requirement: REQ-AGENT-001 (Must)
    // Acceptance: Each entry has name and description
    #[test]
    fn req_agent_001_exit_code_entries_complete() {
        let table = exit_code_table();
        for entry in &table {
            assert!(!entry.name.is_empty(), "code {} should have a name", entry.code);
            assert!(
                !entry.description.is_empty(),
                "code {} should have a description",
                entry.code
            );
        }
    }

    // Requirement: REQ-AGENT-001 (Must)
    // Acceptance: Exit code table is JSON-serializable
    #[test]
    fn req_agent_001_exit_code_table_serializable() {
        let table = exit_code_table();
        let json = serde_json::to_string_pretty(&table).unwrap();
        assert!(json.contains("SUCCESS"));
        assert!(json.contains("GENERIC_ERROR"));

        // Verify it can round-trip
        let parsed: Vec<ExitCodeEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), table.len());
    }

    // ---------------------------------------------------------------
    // REQ-AGENT-002: Schema generation
    // ---------------------------------------------------------------

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Full schema generation returns valid JSON
    #[test]
    fn req_agent_002_full_schema_generation() {
        let schema = generate_schema(None, false);
        assert!(schema.is_object(), "schema should be a JSON object");
        assert_eq!(schema["name"], "omega-google");
        assert!(
            schema["subcommands"].is_array(),
            "should have subcommands array"
        );
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Schema includes known commands
    #[test]
    fn req_agent_002_schema_includes_commands() {
        let schema = generate_schema(None, false);
        let subcommands = schema["subcommands"].as_array().unwrap();
        let names: Vec<&str> = subcommands
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();

        assert!(names.contains(&"auth"), "should include auth");
        assert!(names.contains(&"gmail"), "should include gmail");
        assert!(names.contains(&"calendar"), "should include calendar");
        assert!(names.contains(&"drive"), "should include drive");
        assert!(names.contains(&"version"), "should include version");
        assert!(names.contains(&"config"), "should include config");
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Schema for specific command works
    #[test]
    fn req_agent_002_schema_specific_command() {
        let schema = generate_schema(Some("gmail"), false);
        assert!(schema.is_object());
        assert_eq!(schema["name"], "gmail");
        assert!(
            schema["subcommands"].is_array(),
            "gmail should have subcommands"
        );
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Unknown command returns error
    #[test]
    fn req_agent_002_schema_unknown_command() {
        let schema = generate_schema(Some("nonexistent"), false);
        assert!(schema.get("error").is_some());
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Schema includes global args
    #[test]
    fn req_agent_002_schema_includes_global_args() {
        let schema = generate_schema(None, false);
        let args = schema["args"].as_array().unwrap();
        let arg_names: Vec<&str> = args.iter().map(|a| a["name"].as_str().unwrap()).collect();

        assert!(arg_names.contains(&"json"), "should include --json flag");
        assert!(arg_names.contains(&"plain"), "should include --plain flag");
        assert!(arg_names.contains(&"verbose"), "should include --verbose flag");
        assert!(arg_names.contains(&"account"), "should include --account flag");
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Schema args have type information
    #[test]
    fn req_agent_002_schema_arg_types() {
        let schema = generate_schema(None, false);
        let args = schema["args"].as_array().unwrap();

        let json_arg = args.iter().find(|a| a["name"] == "json").unwrap();
        assert_eq!(json_arg["type"], "bool");

        let account_arg = args.iter().find(|a| a["name"] == "account").unwrap();
        assert_eq!(account_arg["type"], "string");
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Schema includes aliases
    #[test]
    fn req_agent_002_schema_includes_aliases() {
        let schema = generate_schema(Some("calendar"), false);
        // Calendar has alias "cal"
        let aliases = schema["aliases"].as_array().unwrap();
        let alias_strs: Vec<&str> = aliases.iter().filter_map(|a| a.as_str()).collect();
        assert!(
            alias_strs.contains(&"cal"),
            "calendar should have 'cal' alias"
        );
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Schema respects include_hidden flag
    #[test]
    fn req_agent_002_schema_hidden_flag() {
        // Both should succeed without error
        let schema_visible = generate_schema(None, false);
        let schema_all = generate_schema(None, true);
        assert!(schema_visible.is_object());
        assert!(schema_all.is_object());
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: build_schema produces correct structure
    #[test]
    fn req_agent_002_build_schema_structure() {
        let cmd = <super::super::root::Cli as clap::CommandFactory>::command();
        let schema = build_schema(&cmd, false);

        assert_eq!(schema.name, "omega-google");
        assert!(!schema.subcommands.is_empty());
        assert!(!schema.args.is_empty());
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Case-insensitive command lookup
    #[test]
    fn req_agent_002_case_insensitive_lookup() {
        let schema = generate_schema(Some("Gmail"), false);
        assert_eq!(schema["name"], "gmail");
    }

    // Requirement: REQ-AGENT-002 (Must)
    // Acceptance: Alias lookup works
    #[test]
    fn req_agent_002_alias_lookup() {
        let schema = generate_schema(Some("cal"), false);
        assert_eq!(schema["name"], "calendar");
    }
}
