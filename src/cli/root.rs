// Root flag definitions, env resolution

use super::agent::AgentArgs;
use super::appscript::AppScriptArgs;
use super::calendar::CalendarArgs;
use super::chat::ChatArgs;
use super::classroom::ClassroomArgs;
use super::completion::CompletionArgs;
use super::contacts::ContactsArgs;
use super::docs::DocsArgs;
use super::drive::DriveArgs;
use super::forms::FormsArgs;
use super::gmail::GmailArgs;
use super::groups::GroupsArgs;
use super::keep::KeepArgs;
use super::open::OpenArgs;
use super::people::PeopleArgs;
use super::sheets::SheetsArgs;
use super::slides::SlidesArgs;
use super::tasks::TasksArgs;
use clap::{Args, Parser, Subcommand};

/// omega-google: Google Workspace CLI
#[derive(Parser, Debug)]
#[command(
    name = "omega-google",
    version,
    about = "Google Workspace CLI",
    disable_help_subcommand = false
)]
pub struct Cli {
    #[command(flatten)]
    pub flags: RootFlags,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Global flags available on every command.
#[derive(Args, Debug, Clone, Default)]
pub struct RootFlags {
    /// Output JSON (mutually exclusive with --plain and --csv)
    #[arg(long, short = 'j', global = true, env = "GOG_JSON", aliases = ["machine"], conflicts_with_all = ["plain", "csv"])]
    pub json: bool,

    /// Output plain tab-separated values (mutually exclusive with --json and --csv)
    #[arg(long, short = 'p', global = true, env = "GOG_PLAIN", conflicts_with_all = ["json", "csv"])]
    pub plain: bool,

    /// Output CSV (mutually exclusive with --json and --plain)
    #[arg(long, global = true, env = "GOG_CSV", conflicts_with_all = ["json", "plain"])]
    pub csv: bool,

    /// Color mode: auto, always, never
    #[arg(long, global = true, env = "GOG_COLOR", default_value = "auto")]
    pub color: String,

    /// Account email or alias to use
    #[arg(long, short = 'a', global = true, env = "GOG_ACCOUNT", alias = "user")]
    pub account: Option<String>,

    /// OAuth client name
    #[arg(long, global = true, env = "GOG_CLIENT")]
    pub client: Option<String>,

    /// Enable verbose output
    #[arg(long, short = 'v', global = true, env = "GOG_VERBOSE")]
    pub verbose: bool,

    /// Dry run: show what would happen without executing
    #[arg(
        long,
        short = 'n',
        global = true,
        env = "GOG_DRY_RUN",
        alias = "dryrun"
    )]
    pub dry_run: bool,

    /// Skip confirmation prompts
    #[arg(long, short = 'y', global = true, aliases = ["yes"])]
    pub force: bool,

    /// Disable interactive prompts (error if confirmation needed)
    #[arg(long, global = true, env = "GOG_NO_INPUT", alias = "batch")]
    pub no_input: bool,

    /// Select specific fields (comma-separated, supports dot-path notation)
    #[arg(long, global = true)]
    pub select: Option<String>,

    /// Strip envelope, return only primary result
    #[arg(long, global = true)]
    pub results_only: bool,

    /// Restrict available commands (comma-separated list)
    #[arg(
        long,
        global = true,
        env = "GOG_ENABLE_COMMANDS",
        alias = "enable-cmds"
    )]
    pub enable_commands: Option<String>,
}

/// Top-level subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage authentication and accounts
    Auth(AuthArgs),

    /// Manage configuration
    Config(ConfigArgs),

    /// Print version information
    Version,

    /// Date/time utilities
    Time(TimeArgs),

    // M2 services
    /// Gmail operations (search, send, labels, drafts, settings)
    Gmail(GmailArgs),

    /// Google Calendar operations (events, calendars, scheduling)
    #[command(alias = "cal")]
    Calendar(CalendarArgs),

    /// Google Drive operations (files, folders, sharing)
    Drive(DriveArgs),

    /// Google Docs operations
    #[command(alias = "doc")]
    Docs(DocsArgs),

    /// Google Sheets operations
    #[command(alias = "sheet")]
    Sheets(SheetsArgs),

    /// Google Slides operations
    #[command(alias = "slide")]
    Slides(SlidesArgs),

    /// Google Forms operations
    #[command(alias = "form")]
    Forms(FormsArgs),

    // M4 services
    /// Google Chat operations (spaces, messages, threads, DMs)
    Chat(ChatArgs),

    /// Google Classroom operations (courses, roster, coursework)
    #[command(alias = "class")]
    Classroom(ClassroomArgs),

    /// Google Tasks operations (task lists, tasks)
    #[command(alias = "task")]
    Tasks(TasksArgs),

    /// Google Contacts operations (search, create, update, delete)
    #[command(alias = "contact")]
    Contacts(ContactsArgs),

    /// Google People operations (profiles, search, relations)
    #[command(alias = "person")]
    People(PeopleArgs),

    // M5 services
    /// Google Groups operations (list, members)
    #[command(alias = "group")]
    Groups(GroupsArgs),

    /// Google Keep operations (notes, attachments)
    Keep(KeepArgs),

    /// Google Apps Script operations
    #[command(name = "appscript", aliases = ["script", "apps-script"])]
    AppScript(AppScriptArgs),

    // M6 polish commands
    /// Open a Google resource in the browser (offline URL generation)
    #[command(alias = "browse")]
    Open(OpenArgs),

    /// Generate shell completions
    Completion(CompletionArgs),

    /// Print exit code table
    #[command(name = "exit-codes")]
    ExitCodes,

    /// Print machine-readable CLI schema as JSON
    #[command(alias = "help-json")]
    Schema(super::agent::SchemaArgs),

    /// Agent-oriented commands (exit-codes, schema)
    Agent(AgentArgs),
}

// --- Auth subcommands ---

#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Store OAuth credentials file
    Credentials(AuthCredentialsArgs),

    /// Add an account (OAuth flow)
    Add(AuthAddArgs),

    /// Remove an account
    Remove(AuthRemoveArgs),

    /// List authenticated accounts
    List,

    /// Show current account status
    Status,

    /// List available services and their scopes
    Services,

    /// Manage stored tokens
    Tokens(AuthTokensArgs),

    /// Manage account aliases
    Alias(AuthAliasArgs),
}

#[derive(Args, Debug)]
pub struct AuthCredentialsArgs {
    /// Path to the OAuth client credentials JSON file
    pub path: String,
}

#[derive(Args, Debug)]
pub struct AuthAddArgs {
    /// Perform manual code-copy flow (no local server)
    #[arg(long)]
    pub manual: bool,

    /// Remote/headless flow
    #[arg(long)]
    pub remote: bool,

    /// Force re-consent even if token exists
    #[arg(long)]
    pub force_consent: bool,

    /// Comma-separated list of services to authorize
    #[arg(long)]
    pub services: Option<String>,

    /// Request read-only scopes
    #[arg(long)]
    pub readonly: bool,

    /// Drive scope level: full, readonly, or file
    #[arg(long)]
    pub drive_scope: Option<String>,
}

#[derive(Args, Debug)]
pub struct AuthRemoveArgs {
    /// Email of the account to remove
    pub email: String,
}

// --- Auth tokens subcommands ---

#[derive(Args, Debug)]
pub struct AuthTokensArgs {
    #[command(subcommand)]
    pub command: AuthTokensCommand,
}

#[derive(Subcommand, Debug)]
pub enum AuthTokensCommand {
    /// List all stored tokens
    List,

    /// Delete a stored token
    Delete(AuthTokensDeleteArgs),
}

#[derive(Args, Debug)]
pub struct AuthTokensDeleteArgs {
    /// Email of the token to delete
    pub email: String,
}

// --- Auth alias subcommands ---

#[derive(Args, Debug)]
pub struct AuthAliasArgs {
    #[command(subcommand)]
    pub command: AuthAliasCommand,
}

#[derive(Subcommand, Debug)]
pub enum AuthAliasCommand {
    /// Set an account alias
    Set(AuthAliasSetArgs),

    /// Remove an account alias
    Unset(AuthAliasUnsetArgs),

    /// List all aliases
    List,
}

#[derive(Args, Debug)]
pub struct AuthAliasSetArgs {
    /// Alias name
    pub alias: String,

    /// Email to map the alias to
    pub email: String,
}

#[derive(Args, Debug)]
pub struct AuthAliasUnsetArgs {
    /// Alias name to remove
    pub alias: String,
}

// --- Config subcommands ---

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Get a config value
    Get(ConfigGetArgs),

    /// Set a config value
    Set(ConfigSetArgs),

    /// Remove a config value
    Unset(ConfigUnsetArgs),

    /// List all config values
    List,

    /// List all known config keys
    Keys,

    /// Print the config file path
    Path,
}

#[derive(Args, Debug)]
pub struct ConfigGetArgs {
    /// Config key to read
    pub key: String,
}

#[derive(Args, Debug)]
pub struct ConfigSetArgs {
    /// Config key to set
    pub key: String,

    /// Value to set
    pub value: String,
}

#[derive(Args, Debug)]
pub struct ConfigUnsetArgs {
    /// Config key to remove
    pub key: String,
}

// --- Time subcommands ---

#[derive(Args, Debug)]
pub struct TimeArgs {
    #[command(subcommand)]
    pub command: TimeCommand,
}

#[derive(Subcommand, Debug)]
pub enum TimeCommand {
    /// Print the current time
    Now,
}
