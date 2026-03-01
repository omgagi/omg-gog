# Architecture: omega-google

## Scope

This document covers the complete system architecture for omega-google, a Rust reimplementation of gogcli. It spans all 15 Google Workspace services, cross-cutting infrastructure (auth, config, HTTP, output, UI), and supporting features (agent mode, shell completion, email tracking). This architecture supports incremental delivery across milestones M1 through M6.

## Overview

omega-google is a single Rust binary CLI that authenticates with Google APIs via OAuth2, makes raw REST calls to 15 Google Workspace services, and formats output for humans, scripts, and LLM agents. The architecture separates concerns into layered modules: CLI parsing feeds into service handlers, which use a shared authenticated HTTP client to call Google APIs, with responses rendered through a unified output system.

```
                    +-------------------+
                    |     main.rs       |
                    |  (tokio runtime)  |
                    +--------+----------+
                             |
                    +--------v----------+
                    |    cli/ module     |
                    | (clap derive API) |
                    +--------+----------+
                             |
              +--------------+--------------+
              |              |              |
     +--------v---+  +------v------+  +----v--------+
     | config/    |  |  auth/      |  | output/     |
     | (JSON5 r,  |  | (OAuth2,    |  | (JSON/TSV/  |
     |  JSON w)   |  |  keyring,   |  |  text fmt)  |
     +--------+---+  |  JWT)       |  +----+--------+
              |      +------+------+       |
              |             |              |
              +------+------+------+-------+
                     |             |
              +------v------+ +---v--------+
              |   http/     | |   ui/      |
              | (reqwest,   | | (crossterm |
              |  retry,     | |  colors,   |
              |  circuit    | |  stderr)   |
              |  breaker)   | +------------+
              +------+------+
                     |
     +---------------+---------------+
     |       |       |       |       |
  +--v-+ +--v-+ +--v-+ +--v-+ +--v-+  ...
  |gmail| |cal | |drive| |docs| |chat |
  +-----+ +----+ +-----+ +----+ +-----+
        services/ (15 Google APIs)
```

## Project Structure

```
omega-google/
+-- Cargo.toml
+-- flake.nix
+-- flake.lock
+-- .github/
|   +-- workflows/
|       +-- ci.yml
+-- src/
|   +-- main.rs                      # Entry point: tokio runtime, CLI dispatch
|   +-- lib.rs                       # Library root: re-exports for testing
|   +-- cli/
|   |   +-- mod.rs                   # CLI root, RootFlags, CLI struct
|   |   +-- root.rs                  # Root flag definitions, env resolution
|   |   +-- desire_paths.rs          # Argument rewriting, alias dispatch
|   |   +-- exit_codes.rs            # Stable exit code definitions
|   |   +-- auth.rs                  # auth subcommand tree
|   |   +-- config_cmd.rs            # config subcommand tree
|   |   +-- version.rs               # version command
|   |   +-- time_cmd.rs              # time now command
|   |   +-- open.rs                  # open command (URL generation)
|   |   +-- agent.rs                 # agent/schema/exit-codes commands
|   |   +-- completion.rs            # shell completion generation
|   |   +-- gmail.rs                 # gmail subcommand tree
|   |   +-- calendar.rs              # calendar subcommand tree
|   |   +-- drive.rs                 # drive subcommand tree
|   |   +-- docs.rs                  # docs subcommand tree
|   |   +-- sheets.rs                # sheets subcommand tree
|   |   +-- slides.rs                # slides subcommand tree
|   |   +-- forms.rs                 # forms subcommand tree
|   |   +-- chat.rs                  # chat subcommand tree
|   |   +-- classroom.rs             # classroom subcommand tree
|   |   +-- tasks.rs                 # tasks subcommand tree
|   |   +-- contacts.rs              # contacts subcommand tree
|   |   +-- people.rs                # people subcommand tree
|   |   +-- groups.rs                # groups subcommand tree
|   |   +-- keep.rs                  # keep subcommand tree
|   |   +-- appscript.rs             # appscript subcommand tree
|   +-- config/
|   |   +-- mod.rs                   # Config file read/write, path resolution
|   |   +-- file.rs                  # Config file structure (serde)
|   |   +-- credentials.rs           # OAuth credential file management
|   |   +-- paths.rs                 # Platform-specific config/keyring paths
|   +-- auth/
|   |   +-- mod.rs                   # Auth module root
|   |   +-- oauth.rs                 # OAuth2 desktop + manual + remote flows
|   |   +-- token.rs                 # Token storage, refresh, resolution
|   |   +-- keyring.rs               # Keyring abstraction (OS + file fallback)
|   |   +-- service_account.rs       # JWT auth for service accounts
|   |   +-- scopes.rs                # Per-service scope definitions and readonly variants
|   |   +-- account.rs               # Account resolution logic (flag > env > default)
|   +-- http/
|   |   +-- mod.rs                   # HTTP module root
|   |   +-- client.rs                # Authenticated reqwest client builder
|   |   +-- retry.rs                 # Retry middleware (429 backoff, 5xx retry)
|   |   +-- circuit_breaker.rs       # Circuit breaker state machine
|   |   +-- middleware.rs            # Request/response middleware chain
|   +-- output/
|   |   +-- mod.rs                   # Output module root
|   |   +-- mode.rs                  # OutputMode enum, context propagation
|   |   +-- json.rs                  # JSON formatter with transforms
|   |   +-- plain.rs                 # TSV/plain formatter
|   |   +-- text.rs                  # Human-friendly colored text formatter
|   |   +-- transform.rs            # --results-only, --select field projection
|   +-- ui/
|   |   +-- mod.rs                   # UI module root
|   |   +-- color.rs                 # Color detection and crossterm styling
|   |   +-- progress.rs              # Progress hints on stderr
|   |   +-- prompt.rs                # Confirmation prompts (respects --force, --no-input)
|   +-- error/
|   |   +-- mod.rs                   # Error types and formatting
|   |   +-- api_error.rs             # Google API error parsing and user-friendly messages
|   |   +-- exit.rs                  # ExitError with stable exit codes
|   +-- time/
|   |   +-- mod.rs                   # Date/time parsing utilities
|   |   +-- parse.rs                 # Flexible parser: RFC3339, relative, weekday, duration
|   +-- services/
|   |   +-- mod.rs                   # Service module root, ServiceContext struct
|   |   +-- common.rs                # Shared types: pagination, list helpers
|   |   +-- export.rs                # Shared Drive export logic (used by Docs, Sheets, Slides)
|   |   +-- gmail/
|   |   |   +-- mod.rs               # Gmail service root
|   |   |   +-- types.rs             # Gmail API request/response types
|   |   |   +-- search.rs            # Thread/message search
|   |   |   +-- thread.rs            # Thread get/modify/attachments
|   |   |   +-- message.rs           # Message get/attachment
|   |   |   +-- send.rs              # Send email (MIME construction)
|   |   |   +-- drafts.rs            # Draft CRUD
|   |   |   +-- labels.rs            # Label CRUD + name resolution
|   |   |   +-- watch.rs             # Pub/Sub watch + webhook server
|   |   |   +-- history.rs           # History listing
|   |   |   +-- batch.rs             # Batch modify/delete
|   |   |   +-- settings.rs          # Filters, forwarding, send-as, delegates, vacation
|   |   |   +-- mime.rs              # MIME message construction/parsing
|   |   |   +-- tracking.rs          # Email tracking pixel (M6)
|   |   +-- calendar/
|   |   |   +-- mod.rs               # Calendar service root
|   |   |   +-- types.rs             # Calendar API types
|   |   |   +-- events.rs            # Event list/get/create/update/delete
|   |   |   +-- calendars.rs         # Calendar list, ACL
|   |   |   +-- freebusy.rs          # Free/busy queries
|   |   |   +-- respond.rs           # RSVP
|   |   |   +-- search.rs            # Cross-calendar search
|   |   |   +-- special.rs           # Focus time, OOO, working location
|   |   |   +-- colors.rs            # Color definitions
|   |   +-- drive/
|   |   |   +-- mod.rs               # Drive service root
|   |   |   +-- types.rs             # Drive API types
|   |   |   +-- list.rs              # List/search files
|   |   |   +-- files.rs             # Get/download/upload/copy
|   |   |   +-- folders.rs           # Mkdir, move
|   |   |   +-- permissions.rs       # Share, permissions, unshare
|   |   |   +-- comments.rs          # File comments
|   |   |   +-- drives.rs            # Shared drives
|   |   +-- docs/
|   |   |   +-- mod.rs               # Docs service root
|   |   |   +-- types.rs             # Docs API types
|   |   |   +-- export.rs            # Export PDF/DOCX/TXT
|   |   |   +-- content.rs           # Create, copy, info, cat
|   |   |   +-- edit.rs              # Write, insert, delete, find-replace
|   |   |   +-- sedmat.rs            # Sed-like editing engine
|   |   |   +-- markdown.rs          # Markdown-to-Docs formatting
|   |   |   +-- comments.rs          # Document comments
|   |   +-- sheets/
|   |   |   +-- mod.rs               # Sheets service root
|   |   |   +-- types.rs             # Sheets API types
|   |   |   +-- read.rs              # Get/read cells
|   |   |   +-- write.rs             # Update/append/clear cells
|   |   |   +-- format.rs            # Cell formatting
|   |   |   +-- structure.rs         # Insert rows/cols, create, metadata
|   |   |   +-- a1.rs                # A1 notation parser
|   |   +-- slides/
|   |   |   +-- mod.rs               # Slides service root
|   |   |   +-- types.rs             # Slides API types
|   |   |   +-- export.rs            # Export PDF/PPTX
|   |   |   +-- presentations.rs     # Create, copy, info
|   |   |   +-- slides_ops.rs        # Add/delete/read/replace slides
|   |   |   +-- notes.rs             # Speaker notes
|   |   |   +-- markdown.rs          # Markdown to slides
|   |   +-- forms/
|   |   |   +-- mod.rs               # Forms service root
|   |   |   +-- types.rs             # Forms API types
|   |   |   +-- forms.rs             # Get/create form
|   |   |   +-- responses.rs         # List/get responses
|   |   +-- chat/
|   |   |   +-- mod.rs               # Chat service root
|   |   |   +-- types.rs             # Chat API types
|   |   |   +-- spaces.rs            # List/find/create spaces
|   |   |   +-- messages.rs          # List/send messages
|   |   |   +-- dm.rs                # DM space/send
|   |   +-- classroom/
|   |   |   +-- mod.rs               # Classroom service root
|   |   |   +-- types.rs             # Classroom API types
|   |   |   +-- courses.rs           # Course CRUD
|   |   |   +-- roster.rs            # Students/teachers
|   |   |   +-- coursework.rs        # Assignments
|   |   |   +-- materials.rs         # Course materials
|   |   |   +-- submissions.rs       # Student submissions
|   |   |   +-- announcements.rs     # Announcements
|   |   |   +-- topics.rs            # Topics
|   |   |   +-- invitations.rs       # Course invitations
|   |   |   +-- guardians.rs         # Guardian management
|   |   +-- tasks/
|   |   |   +-- mod.rs               # Tasks service root
|   |   |   +-- types.rs             # Tasks API types
|   |   |   +-- tasklists.rs         # Task list CRUD
|   |   |   +-- tasks_ops.rs         # Task CRUD, done/undo/clear
|   |   +-- contacts/
|   |   |   +-- mod.rs               # Contacts service root
|   |   |   +-- types.rs             # People API types (contacts)
|   |   |   +-- contacts.rs          # Search/list/get/create/update/delete
|   |   |   +-- directory.rs         # Workspace directory
|   |   |   +-- other.rs             # Other contacts
|   |   +-- people/
|   |   |   +-- mod.rs               # People service root
|   |   |   +-- types.rs             # People API types (profiles)
|   |   |   +-- profile.rs           # Me/get/search/relations
|   |   +-- groups/
|   |   |   +-- mod.rs               # Groups service root
|   |   |   +-- types.rs             # Cloud Identity API types
|   |   |   +-- groups.rs            # List groups, members
|   |   +-- keep/
|   |   |   +-- mod.rs               # Keep service root
|   |   |   +-- types.rs             # Keep API types
|   |   |   +-- notes.rs             # List/get/search notes
|   |   |   +-- attachments.rs       # Download attachments
|   |   +-- appscript/
|   |       +-- mod.rs               # Apps Script service root
|   |       +-- types.rs             # Apps Script API types
|   |       +-- projects.rs          # Get/create/content/run
|   +-- tracking/
|       +-- mod.rs                   # Email tracking (M6)
|       +-- pixel.rs                 # AES-GCM pixel URL generation
|       +-- config.rs                # Tracking configuration
+-- tests/
    +-- integration/
    |   +-- mod.rs                   # Integration test harness (opt-in)
    +-- cli_smoke.rs                 # CLI smoke tests (binary invocation)
```

## Modules

### Module 1: `main.rs` -- Entry Point

- **Responsibility**: Initialize tokio runtime, parse CLI arguments, dispatch to command handlers, format and return exit codes.
- **Public interface**: `fn main()` only.
- **Dependencies**: `cli`, `error`
- **Implementation order**: 1

The main function is deliberately thin. It creates the tokio runtime, calls the CLI dispatcher, and translates errors into exit codes.

```rust
#[tokio::main]
async fn main() {
    let exit_code = cli::execute(std::env::args_os().skip(1).collect()).await;
    std::process::exit(exit_code);
}
```

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Tokio runtime fails to start | System resource exhaustion | Panic on startup | None; hard failure | Binary does not run |
| CLI parse error | Invalid arguments | clap error return | Print formatted error to stderr, exit 2 | User sees help |

### Module 2: `cli/` -- Command Line Interface

- **Responsibility**: Define all clap command structs, root flags, desire path aliases, argument rewriting, and command dispatch to service handlers.
- **Public interface**:
  - `pub async fn execute(args: Vec<OsString>) -> i32`
  - `pub struct RootFlags` (shared across all commands)
  - `pub struct Cli` (top-level clap struct)
- **Dependencies**: `config`, `auth`, `http`, `output`, `ui`, `error`, `services/*`, `time`
- **Implementation order**: 2

```rust
/// Root flags shared by all commands. Embedded in the top-level Cli struct.
#[derive(clap::Args, Debug, Clone)]
pub struct RootFlags {
    /// Output JSON to stdout (best for scripting)
    #[arg(long, short = 'j', aliases = ["machine"], env = "GOG_JSON",
          conflicts_with = "plain")]
    pub json: bool,

    /// Output stable, parseable text to stdout (TSV; no colors)
    #[arg(long, short = 'p', aliases = ["tsv"], env = "GOG_PLAIN",
          conflicts_with = "json")]
    pub plain: bool,

    /// Color output: auto|always|never
    #[arg(long, default_value = "auto", env = "GOG_COLOR")]
    pub color: ColorMode,

    /// Account email for API commands
    #[arg(long, short = 'a', aliases = ["acct"], env = "GOG_ACCOUNT")]
    pub account: Option<String>,

    /// OAuth client name
    #[arg(long, env = "GOG_CLIENT")]
    pub client: Option<String>,

    /// Enable verbose logging
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Do not make changes; print intended actions
    #[arg(long, short = 'n', aliases = ["noop", "preview", "dryrun"])]
    pub dry_run: bool,

    /// Skip confirmations for destructive commands
    #[arg(long, short = 'y', aliases = ["yes", "assume-yes"])]
    pub force: bool,

    /// Never prompt; fail instead (useful for CI)
    #[arg(long, aliases = ["non-interactive", "noninteractive"])]
    pub no_input: bool,

    /// In JSON mode, emit only the primary result
    #[arg(long = "results-only")]
    pub results_only: bool,

    /// In JSON mode, select comma-separated fields
    #[arg(long = "select", aliases = ["pick", "project"])]
    pub select: Option<String>,

    /// Comma-separated list of enabled top-level commands
    #[arg(long = "enable-commands", env = "GOG_ENABLE_COMMANDS")]
    pub enable_commands: Option<String>,
}

/// Top-level CLI structure with all service subcommands.
#[derive(clap::Parser, Debug)]
#[command(name = "omega-google", version, about = "Google Workspace CLI")]
pub struct Cli {
    #[command(flatten)]
    pub flags: RootFlags,

    #[command(subcommand)]
    pub command: Command,
}

/// All top-level commands and desire path aliases.
#[derive(clap::Subcommand, Debug)]
pub enum Command {
    // Service commands
    Auth(auth::AuthCommand),
    Config(config_cmd::ConfigCommand),
    Gmail(gmail::GmailCommand),
    #[command(alias = "cal")]
    Calendar(calendar::CalendarCommand),
    #[command(alias = "drv")]
    Drive(drive::DriveCommand),
    #[command(alias = "doc")]
    Docs(docs::DocsCommand),
    #[command(alias = "sheet")]
    Sheets(sheets::SheetsCommand),
    #[command(alias = "slide")]
    Slides(slides::SlidesCommand),
    #[command(alias = "form")]
    Forms(forms::FormsCommand),
    Chat(chat::ChatCommand),
    #[command(alias = "class")]
    Classroom(classroom::ClassroomCommand),
    #[command(alias = "task")]
    Tasks(tasks::TasksCommand),
    #[command(alias = "contact")]
    Contacts(contacts::ContactsCommand),
    #[command(alias = "person")]
    People(people::PeopleCommand),
    #[command(alias = "group")]
    Groups(groups::GroupsCommand),
    Keep(keep::KeepCommand),
    #[command(name = "appscript", aliases = ["script", "apps-script"])]
    AppScript(appscript::AppScriptCommand),
    Time(time_cmd::TimeCommand),
    Version(version::VersionCommand),
    #[command(alias = "browse")]
    Open(open::OpenCommand),
    Completion(completion::CompletionCommand),
    Agent(agent::AgentCommand),
    Schema(agent::SchemaCommand),
    #[command(name = "exit-codes", aliases = ["exitcodes"])]
    ExitCodes(agent::ExitCodesCommand),

    // Desire path aliases (M2)
    Send(gmail::GmailSendCommand),
    #[command(alias = "list")]
    Ls(drive::DriveLsCommand),
    #[command(alias = "find")]
    Search(drive::DriveSearchCommand),
    #[command(alias = "dl")]
    Download(drive::DriveDownloadCommand),
    #[command(alias = "up")]
    Upload(drive::DriveUploadCommand),
    Login(auth::AuthAddCommand),
    Logout(auth::AuthRemoveCommand),
    Status(auth::AuthStatusCommand),
    Me(people::PeopleMeCommand),
    #[command(alias = "who-am-i")]
    Whoami(people::PeopleMeCommand),
}
```

**Desire path argument rewriting**: Before clap parsing, the `execute` function rewrites `--fields` to `--select` (except when the first two command tokens are `calendar events`), matching gogcli behavior.

**Command allowlisting**: After parsing, if `--enable-commands` is set, the top-level command name is checked against the allowlist. Non-allowed commands return exit code 1 with a clear error message.

**GOG_AUTO_JSON**: After parsing, if `GOG_AUTO_JSON` is truthy and stdout is not a TTY and neither `--json` nor `--plain` was set, JSON mode is activated.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Unknown command | Typo or version mismatch | clap parse error | Print suggestions, exit 2 | User corrects input |
| Conflicting flags | `--json` + `--plain` | clap conflicts_with | Print error, exit 2 | User removes one flag |
| Blocked command | `--enable-commands` restriction | Allowlist check | Print blocked message, exit 1 | Agent adjusts |

#### Security Considerations

- **Trust boundary**: CLI arguments come from potentially untrusted sources (LLM agents, scripts)
- **Command allowlisting**: `--enable-commands` restricts what an untrusted caller can invoke
- **Argument injection**: clap handles argument parsing safely; no shell expansion

### Module 3: `config/` -- Configuration Management

- **Responsibility**: Read/write config file (JSON5 read, JSON write), manage OAuth credential files, resolve platform-specific paths.
- **Public interface**:
  - `pub fn config_dir() -> Result<PathBuf>`
  - `pub fn config_path() -> Result<PathBuf>`
  - `pub fn ensure_dir() -> Result<PathBuf>`
  - `pub fn read_config() -> Result<ConfigFile>`
  - `pub fn write_config(cfg: &ConfigFile) -> Result<()>`
  - `pub fn read_client_credentials(client: &str) -> Result<ClientCredentials>`
  - `pub fn write_client_credentials(client: &str, creds: &ClientCredentials) -> Result<()>`
  - `pub fn service_account_path(email: &str) -> Result<PathBuf>`
  - `pub struct ConfigFile`
  - `pub struct ClientCredentials`
- **Dependencies**: `serde`, `serde_json`, `json5` crate
- **Implementation order**: 3

```rust
/// Configuration file structure. Maps to $CONFIG_DIR/omega-google/config.json.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyring_backend: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_timezone: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_aliases: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_clients: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_domains: Option<HashMap<String, String>>,

    /// Preserve unknown fields for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// OAuth client credentials parsed from Google's downloaded JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCredentials {
    pub client_id: String,
    pub client_secret: String,
}
```

**Atomic writes**: Config is written to a `.tmp` file first, then renamed into place. File permissions are set to 0600.

**JSON5 reading**: The `json5` crate parses config files, allowing comments and trailing commas. Writes use standard JSON via `serde_json` for maximum portability.

**Path resolution**: Uses `dirs::config_dir()` to find the platform-specific config directory, then appends `omega-google/`.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Config dir not writable | Permissions | `std::fs` error | Clear error with path, exit 10 | Cannot save config |
| Malformed config.json | Hand-edited errors | JSON5 parse error | Clear error with line/col, exit 10 | Cannot read config |
| Missing credentials.json | Not yet set up | File not found | Prompt to run `auth credentials`, exit 4 | Cannot authenticate |
| Atomic write fails | Disk full, rename fail | `std::fs` error | Leave original file intact | Old config preserved |

#### Security Considerations

- **Sensitive data**: `credentials.json` contains OAuth client ID and secret (semi-public but treated as sensitive)
- **File permissions**: All files written with 0600 (owner-only read/write)
- **No secrets on disk**: Refresh tokens are NEVER stored in config files; only in keyring

### Module 4: `auth/` -- Authentication and Credential Management

- **Responsibility**: OAuth2 flows (desktop, manual, remote), token management, keyring abstraction, service account JWT, scope mapping, account resolution.
- **Public interface**:
  - `pub trait CredentialStore: Send + Sync`
  - `pub struct KeyringStore` (implements `CredentialStore`)
  - `pub async fn oauth_flow(opts: OAuthFlowOptions) -> Result<TokenData>`
  - `pub fn resolve_account(flags: &RootFlags, config: &ConfigFile) -> Result<String>`
  - `pub fn scopes_for_services(services: &[Service], opts: &ScopeOptions) -> Vec<String>`
  - `pub async fn service_account_token(email: &str, scopes: &[String]) -> Result<String>`
  - `pub enum Service` (15 variants)
  - `pub struct ScopeOptions` (readonly, drive_scope)
  - `pub struct TokenData`
- **Dependencies**: `config`, `oauth2`, `keyring`, `jsonwebtoken`, `reqwest` (for token exchange)
- **Implementation order**: 4

```rust
/// All 15 supported Google services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    Gmail,
    Calendar,
    Chat,
    Classroom,
    Drive,
    Docs,
    Slides,
    Contacts,
    Tasks,
    People,
    Sheets,
    Forms,
    AppScript,
    Groups,
    Keep,
}

/// Controls scope selection for OAuth consent.
#[derive(Debug, Clone, Default)]
pub struct ScopeOptions {
    pub readonly: bool,
    pub drive_scope: DriveScopeMode,
}

#[derive(Debug, Clone, Default)]
pub enum DriveScopeMode {
    #[default]
    Full,
    Readonly,
    File,
}

/// Abstraction over credential storage.
pub trait CredentialStore: Send + Sync {
    fn get_token(&self, client: &str, email: &str) -> Result<TokenData>;
    fn set_token(&self, client: &str, email: &str, token: &TokenData) -> Result<()>;
    fn delete_token(&self, client: &str, email: &str) -> Result<()>;
    fn list_tokens(&self) -> Result<Vec<TokenData>>;
    fn keys(&self) -> Result<Vec<String>>;
    fn get_default_account(&self, client: &str) -> Result<Option<String>>;
    fn set_default_account(&self, client: &str, email: &str) -> Result<()>;
}

/// Token data stored in the keyring. RefreshToken is never serialized to disk files.
#[derive(Debug, Clone)]
pub struct TokenData {
    pub client: String,
    pub email: String,
    pub services: Vec<Service>,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token: String, // Never logged, never written to disk files
}

/// Stored format inside keyring (refresh_token is in this payload).
#[derive(Debug, Serialize, Deserialize)]
struct StoredToken {
    refresh_token: String,
    #[serde(default)]
    services: Vec<String>,
    #[serde(default)]
    scopes: Vec<String>,
    #[serde(default)]
    created_at: Option<String>,
}
```

**Account resolution order**: `--account` flag > `GOG_ACCOUNT` env > alias resolution > `account_clients` domain matching > keyring default > single stored token.

**Keyring key format**: `token:<client>:<email>` where client defaults to `default`.

**OAuth2 desktop flow**: Starts a local HTTP server on an ephemeral port, opens the browser, receives the redirect, exchanges the code for tokens.

**Manual flow**: Prints the auth URL, user pastes the redirect URL, code is extracted and exchanged.

**Remote two-step flow**: Step 1 generates auth URL with a random state parameter (cached to a temporary file). Step 2 accepts the redirect URL, validates the state, and exchanges the code.

**Service account JWT**: Reads the service account JSON key file, constructs a JWT with the `jsonwebtoken` crate (RS256), exchanges it for an access token via Google's token endpoint, supporting `subject` for domain-wide delegation.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Keyring locked (macOS) | Keychain locked after sleep | Keychain error string | Print "unlock keychain" instructions, exit 4 | Cannot access tokens |
| Keyring unavailable (Linux) | No D-Bus / no Secret Service | Timeout after 5s | Fall back to file backend with password prompt | Degraded but functional |
| Token expired | Normal token lifecycle | 401 from Google API | Auto-refresh via refresh_token | Transparent to user |
| Refresh token revoked | User revoked in Google settings | 400 from token exchange | Print "re-authenticate" message, exit 4 | Must re-auth |
| No credentials.json | First-time setup | File not found | Print setup instructions, exit 4 | Must configure first |
| Service account key invalid | Wrong format / expired | JWT construction error | Clear error with key file path | Cannot use service account |
| Ephemeral port conflict | Port already in use | Bind error | Try next port or fall back to manual flow | Minor delay |
| D-Bus timeout on Linux | gnome-keyring not running | 5-second timeout | Suggest `GOG_KEYRING_BACKEND=file`, exit 4 | Clear guidance |

#### Security Considerations

- **Trust boundary**: Refresh tokens are high-value secrets with long lifetimes
- **Sensitive data**: Refresh tokens stored only in OS keyring (never on disk as plain files)
- **File backend encryption**: When keyring falls back to file, payloads are encrypted with a password
- **Credential logging**: Refresh tokens and access tokens are NEVER logged, even at verbose/debug level
- **Scope minimization**: `--readonly` and `--services` flags limit requested scopes to least-privilege
- **TLS enforcement**: OAuth token exchange uses TLS 1.2+ (inherited from HTTP module)
- **State parameter**: OAuth flows validate `state` to prevent CSRF attacks

#### Performance Budget

- **Token refresh**: < 2s including network round-trip
- **Keyring access**: < 100ms for get/set operations
- **Scope calculation**: < 1ms (pure computation)

### Module 5: `http/` -- HTTP Client and Retry Logic

- **Responsibility**: Build an authenticated reqwest client with automatic token injection, retry on 429/5xx, circuit breaker, and TLS enforcement.
- **Public interface**:
  - `pub struct ApiClient`
  - `pub async fn build_client(service: Service, email: &str, client_name: &str) -> Result<ApiClient>`
  - `pub struct RetryConfig`
  - `pub struct CircuitBreaker`
  - impl `ApiClient`: `get`, `post`, `put`, `patch`, `delete` methods that handle auth and retry
- **Dependencies**: `auth`, `reqwest`, `tokio`
- **Implementation order**: 5

```rust
/// Authenticated HTTP client for Google API calls.
/// Handles token injection, retry, and circuit breaking.
pub struct ApiClient {
    client: reqwest::Client,
    token_source: TokenSource,
    retry_config: RetryConfig,
    circuit_breaker: Arc<CircuitBreaker>,
}

/// Determines how the client obtains access tokens.
enum TokenSource {
    /// OAuth2 refresh token flow (most commands).
    OAuth {
        client_id: String,
        client_secret: String,
        refresh_token: String,
        cached_token: Arc<tokio::sync::Mutex<Option<CachedAccessToken>>>,
    },
    /// Service account JWT flow (Keep, domain-wide delegation).
    ServiceAccount {
        key_json: Vec<u8>,
        subject: String,
        scopes: Vec<String>,
        cached_token: Arc<tokio::sync::Mutex<Option<CachedAccessToken>>>,
    },
}

struct CachedAccessToken {
    access_token: String,
    expires_at: std::time::Instant,
}

/// Retry configuration matching gogcli defaults.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries_429: u32,      // Default: 3
    pub max_retries_5xx: u32,      // Default: 1
    pub base_delay: Duration,       // Default: 1s
    pub server_error_delay: Duration, // Default: 1s
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries_429: 3,
            max_retries_5xx: 1,
            base_delay: Duration::from_secs(1),
            server_error_delay: Duration::from_secs(1),
        }
    }
}

/// Circuit breaker: opens after 5 consecutive 5xx failures,
/// resets after 30 seconds.
pub struct CircuitBreaker {
    state: Mutex<CircuitState>,
}

struct CircuitState {
    failures: u32,
    last_failure: Option<Instant>,
    open: bool,
}

const CIRCUIT_BREAKER_THRESHOLD: u32 = 5;
const CIRCUIT_BREAKER_RESET_TIME: Duration = Duration::from_secs(30);

impl ApiClient {
    /// Execute an authenticated API request with retry and circuit breaking.
    pub async fn request(&self, method: Method, url: &str) -> RequestBuilder { ... }

    /// GET with automatic retry.
    pub async fn get(&self, url: &str) -> Result<Response> { ... }

    /// POST with body replay support for retries.
    pub async fn post(&self, url: &str, body: impl Into<Body>) -> Result<Response> { ... }

    // ... put, patch, delete
}
```

**Retry algorithm**: On 429 responses, the client checks the `Retry-After` header first (seconds or HTTP date). If absent, it uses exponential backoff: `base_delay * 2^attempt` with random jitter of 0-50% of the delay. On 5xx responses, a flat delay is used.

**Body replay**: For POST/PUT/PATCH requests that need retry, the body is read into memory before the first attempt. The `GetBody` equivalent in Rust is achieved by cloning the `Bytes`.

**TLS enforcement**: The reqwest client is built with `.min_tls_version(tls::Version::TLS_1_2)`.

**Timeout**: Default request timeout is 30 seconds, configurable per-request for long operations like file uploads.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| 429 rate limited | Google API quota exceeded | HTTP 429 status | Exponential backoff with jitter, max 3 retries | Transparent retry; fails after limit |
| 5xx server error | Google API transient failure | HTTP 5xx status | Single retry after 1s delay | Usually recovers |
| Circuit breaker open | 5 consecutive 5xx failures | CircuitBreaker state check | Return CircuitBreakerError immediately, reset after 30s | Fast failure, exit 8 |
| Network timeout | Connectivity loss | reqwest timeout | Clear error message with URL | User retries manually |
| Token refresh failure | Revoked or expired refresh token | 400 from token endpoint | Prompt to re-authenticate, exit 4 | Must re-auth |
| TLS handshake failure | Certificate issues, protocol mismatch | reqwest TLS error | Clear error; no downgrade below TLS 1.2 | Cannot connect |
| Request body too large for retry | Very large upload | Memory constraint | Skip retry for streaming uploads | Upload fails; user retries |

#### Security Considerations

- **TLS enforcement**: Minimum TLS 1.2; no fallback to insecure protocols
- **Token in Authorization header**: Bearer token added per-request; never logged
- **Retry-After header**: Parsed safely (integer seconds or HTTP date); invalid values treated as 0
- **No credential leakage**: Request logging at verbose level redacts Authorization header value

#### Performance Budget

- **Overhead per request** (excluding network): < 5ms (token cache check + header injection)
- **Retry sleep**: Cancellable via tokio cancellation token
- **Connection pooling**: reqwest maintains a connection pool; reused across requests to same host
- **Streaming**: File uploads/downloads use streaming bodies; no full-memory buffering for large files

### Module 6: `output/` -- Output Formatting

- **Responsibility**: Render command results in JSON, plain/TSV, or human-friendly text format. Apply `--results-only` and `--select` transforms.
- **Public interface**:
  - `pub enum OutputMode { Json, Plain, Text }`
  - `pub fn resolve_mode(flags: &RootFlags, is_tty: bool) -> Result<OutputMode>`
  - `pub fn write_json<T: Serialize>(writer: &mut impl Write, value: &T, transform: &JsonTransform) -> Result<()>`
  - `pub fn write_plain(writer: &mut impl Write, rows: &[Vec<String>]) -> Result<()>`
  - `pub trait TextOutput` (for human-friendly rendering)
  - `pub struct JsonTransform { pub results_only: bool, pub select: Vec<String> }`
- **Dependencies**: `serde`, `serde_json`
- **Implementation order**: 6

```rust
/// Output mode resolved from flags, env vars, and TTY detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Structured JSON (--json or GOG_JSON=1 or GOG_AUTO_JSON piped)
    Json,
    /// Tab-separated values, stable for scripting (--plain or GOG_PLAIN=1)
    Plain,
    /// Human-friendly text with colors and alignment (default on TTY)
    Text,
}

/// JSON post-processing transforms.
#[derive(Debug, Clone, Default)]
pub struct JsonTransform {
    /// Strip envelope fields, emit only primary results.
    pub results_only: bool,
    /// Project objects to these fields only (supports dot-path notation).
    pub select: Vec<String>,
}
```

**Results-only logic**: Strips known envelope keys (`nextPageToken`, `next_cursor`, `has_more`, `count`, `query`, `dry_run`), then if exactly one non-meta key remains, unwraps to its value. Falls back to known result key names (`files`, `threads`, `messages`, `events`, etc.).

**Field selection**: `--select id,name,file.mimeType` projects JSON objects (or each element in arrays) to only the requested fields. Dot-path notation traverses nested objects. Missing fields are silently omitted.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| JSON serialization fails | Unexpected data type | serde error | Return internal error | Should not happen with typed data |
| Write to stdout fails | Broken pipe | io::Error | Silently exit (SIGPIPE handling) | Normal for piped output |
| Invalid --select path | User typo | Field not found in object | Silently omit field | Partial output |

#### Performance Budget

- **JSON formatting**: < 5ms for typical API responses (< 100KB)
- **Transform application**: < 2ms for results-only + field selection
- **No intermediate allocation for plain mode**: Direct write to stdout

### Module 7: `ui/` -- Terminal User Interface

- **Responsibility**: Colored output to stderr, progress hints, confirmation prompts, NO_COLOR support.
- **Public interface**:
  - `pub struct Ui { stdout: Writer, stderr: Writer }`
  - `pub fn new(opts: UiOptions) -> Result<Ui>`
  - `pub fn error(&self, msg: &str)` -- colored error to stderr
  - `pub fn warn(&self, msg: &str)` -- colored warning to stderr
  - `pub fn hint(&self, msg: &str)` -- hint text to stderr
  - `pub fn progress(&self, msg: &str)` -- progress text to stderr
  - `pub fn confirm(&self, prompt: &str, flags: &RootFlags) -> Result<bool>`
  - `pub fn is_tty_stdout() -> bool`
- **Dependencies**: `crossterm`
- **Implementation order**: 7

```rust
#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

pub struct UiOptions {
    pub color: ColorMode,
}

pub struct Ui {
    use_color: bool,
}

impl Ui {
    /// Confirm prompts return Ok(true) if --force is set.
    /// Return Err if --no-input is set and a prompt would be shown.
    pub fn confirm(&self, prompt: &str, flags: &RootFlags) -> Result<bool> {
        if flags.force { return Ok(true); }
        if flags.no_input {
            return Err(Error::NoInputPrompt(prompt.to_string()));
        }
        // Prompt on stderr, read from stdin
        eprint!("{} [y/N] ", prompt);
        // ... read line
    }
}
```

**Color resolution**: Colors enabled when `ColorMode::Auto` AND stdout is a TTY AND `NO_COLOR` is not set, OR `ColorMode::Always`. Colors disabled for `ColorMode::Never` or when `NO_COLOR` is set. Colors are always disabled for JSON and plain output modes.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Terminal not a TTY | Piped stdin/stdout | `is_tty()` check | Skip color, skip prompts | Graceful degradation |
| Prompt with --no-input | CI/automation context | Flag check | Return error, exit 1 | User adds --force |

### Module 8: `error/` -- Error Types

- **Responsibility**: Define typed errors with thiserror for library code, map to stable exit codes, format user-friendly error messages from Google API errors.
- **Public interface**:
  - `pub enum OmegaError` (thiserror enum with variants for each error category)
  - `pub struct ExitError { code: i32, source: Box<dyn Error> }`
  - `pub fn exit_code_for(err: &OmegaError) -> i32`
  - `pub fn format_api_error(status: u16, body: &str) -> String`
- **Dependencies**: `thiserror`, `anyhow`
- **Implementation order**: 8

```rust
/// Stable exit codes matching gogcli conventions.
pub mod codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERIC_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 2;
    pub const EMPTY_RESULTS: i32 = 3;  // --fail-empty
    pub const AUTH_REQUIRED: i32 = 4;
    pub const NOT_FOUND: i32 = 5;
    pub const PERMISSION_DENIED: i32 = 6;
    pub const RATE_LIMITED: i32 = 7;
    pub const RETRYABLE: i32 = 8;      // circuit breaker, transient
    pub const CONFIG_ERROR: i32 = 10;
    pub const CANCELLED: i32 = 130;    // SIGINT
}

#[derive(Debug, thiserror::Error)]
pub enum OmegaError {
    #[error("authentication required: {message}")]
    AuthRequired { message: String },

    #[error("not found: {resource}")]
    NotFound { resource: String },

    #[error("permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("rate limited")]
    RateLimited,

    #[error("circuit breaker open")]
    CircuitBreakerOpen,

    #[error("config error: {message}")]
    ConfigError { message: String },

    #[error("empty results")]
    EmptyResults,

    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

**Google API error parsing**: When a Google API returns an error JSON body (`{"error": {"code": 404, "message": "..."}}`), the error module parses it and provides a user-friendly message. Common patterns (401 -> "re-authenticate", 403 -> "check permissions/scopes", 404 -> "resource not found") get specific guidance.

### Module 9: `time/` -- Date/Time Parsing

- **Responsibility**: Parse flexible date/time inputs matching gogcli conventions (RFC3339, date-only, relative dates, weekday names, durations).
- **Public interface**:
  - `pub fn parse_datetime(input: &str, tz: &Tz) -> Result<DateTime<Tz>>`
  - `pub fn parse_date(input: &str) -> Result<NaiveDate>`
  - `pub fn parse_duration(input: &str) -> Result<Duration>`
- **Dependencies**: `chrono`, `chrono-tz`
- **Implementation order**: 9

**Accepted formats**:
- `YYYY-MM-DD` -- date only
- `YYYY-MM-DDTHH:MM:SS` / `YYYY-MM-DD HH:MM:SS` -- datetime without timezone (use default TZ)
- RFC3339 -- full datetime with timezone
- `now`, `today`, `tomorrow`, `yesterday` -- relative
- Weekday names: `monday`, `next friday`, `last tuesday`
- Durations: `24h`, `7d`, `30m`, `1h30m`

### Module 10: `services/` -- Google API Service Implementations

- **Responsibility**: Implement all 15 Google Workspace API integrations. Each service is a submodule with API types, request/response handling, and business logic.
- **Public interface**: Each service exposes functions that accept an `ApiClient` and command-specific parameters, returning typed responses.
- **Dependencies**: `http` (ApiClient), `output`, `serde`
- **Implementation order**: Incrementally per milestone (M2-M5)

#### Service Context Pattern

Every service handler receives a `ServiceContext` that bundles the common dependencies:

```rust
/// Shared context passed to all service handlers.
pub struct ServiceContext {
    pub client: ApiClient,
    pub output_mode: OutputMode,
    pub json_transform: JsonTransform,
    pub ui: Ui,
    pub flags: RootFlags,
}

impl ServiceContext {
    /// Write output in the appropriate format.
    pub fn write_output<T: Serialize + TextOutput + PlainOutput>(
        &self,
        value: &T,
    ) -> Result<()> {
        match self.output_mode {
            OutputMode::Json => write_json(&mut stdout(), value, &self.json_transform),
            OutputMode::Plain => value.write_plain(&mut stdout()),
            OutputMode::Text => value.write_text(&mut stdout(), &self.ui),
        }
    }

    /// Write paginated output with nextPageToken hint on stderr.
    pub fn write_paginated<T: Serialize + TextOutput + PlainOutput>(
        &self,
        value: &T,
        next_page_token: Option<&str>,
    ) -> Result<()> {
        self.write_output(value)?;
        if let Some(token) = next_page_token {
            self.ui.hint(&format!("# Next page: --page {}", token));
        }
        Ok(())
    }
}
```

#### API Type Pattern

Each service defines request and response types using serde with `flatten` for forward compatibility:

```rust
/// Example: Gmail thread list response.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadListResponse {
    #[serde(default)]
    pub threads: Vec<Thread>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u32>,
    /// Capture unknown fields for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: String,
    pub snippet: Option<String>,
    pub history_id: Option<String>,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

#### Service Handler Pattern

```rust
/// Example: Gmail search handler.
pub async fn search(
    ctx: &ServiceContext,
    query: &str,
    max_results: Option<u32>,
    page_token: Option<&str>,
) -> Result<()> {
    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/threads?q={}&maxResults={}&pageToken={}",
        urlencoding::encode(query),
        max_results.unwrap_or(20),
        page_token.unwrap_or(""),
    );

    let resp = ctx.client.get(&url).await?;
    let status = resp.status();

    if !status.is_success() {
        return Err(parse_api_error(status.as_u16(), &resp.text().await?));
    }

    let body: ThreadListResponse = resp.json().await?;
    ctx.write_paginated(&body, body.next_page_token.as_deref())
}
```

#### Text and Plain Output Traits

Each response type implements formatting traits:

```rust
/// Human-friendly text output (default mode).
pub trait TextOutput {
    fn write_text(&self, w: &mut impl Write, ui: &Ui) -> Result<()>;
}

/// Tab-separated output for scripting (--plain mode).
pub trait PlainOutput {
    fn write_plain(&self, w: &mut impl Write) -> Result<()>;
}
```

#### Per-Service API Endpoints

| Service | Base URL | API Version |
|---------|----------|-------------|
| Gmail | `https://gmail.googleapis.com/gmail/v1` | v1 |
| Calendar | `https://www.googleapis.com/calendar/v3` | v3 |
| Drive | `https://www.googleapis.com/drive/v3` | v3 |
| Docs | `https://docs.googleapis.com/v1` | v1 |
| Sheets | `https://sheets.googleapis.com/v4` | v4 |
| Slides | `https://slides.googleapis.com/v1` | v1 |
| Forms | `https://forms.googleapis.com/v1` | v1 |
| Chat | `https://chat.googleapis.com/v1` | v1 |
| Classroom | `https://classroom.googleapis.com/v1` | v1 |
| Tasks | `https://tasks.googleapis.com/tasks/v1` | v1 |
| People (Contacts) | `https://people.googleapis.com/v1` | v1 |
| Cloud Identity (Groups) | `https://cloudidentity.googleapis.com/v1` | v1 |
| Keep | `https://keep.googleapis.com/v1` | v1 |
| Apps Script | `https://script.googleapis.com/v1` | v1 |

#### Service Implementation Order

Services are implemented by milestone:

| Order | Milestone | Service | Complexity | Files (approx) |
|-------|-----------|---------|------------|-----------------|
| 1 | M2 | Gmail | High | 12 |
| 2 | M2 | Calendar | High | 9 |
| 3 | M2 | Drive | Medium | 8 |
| 4 | M3 | Docs | Very High (sedmat) | 8 |
| 5 | M3 | Sheets | Medium | 7 |
| 6 | M3 | Slides | Medium | 7 |
| 7 | M3 | Forms | Low | 4 |
| 8 | M4 | Chat | Low | 5 |
| 9 | M4 | Classroom | High | 10 |
| 10 | M4 | Tasks | Low | 4 |
| 11 | M4 | Contacts | Medium | 5 |
| 12 | M4 | People | Low | 3 |
| 13 | M5 | Groups | Low | 3 |
| 14 | M5 | Keep | Medium | 4 |
| 15 | M5 | Apps Script | Low | 3 |

#### Failure Modes (per service)

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| 401 Unauthorized | Token expired or revoked | HTTP 401 | Auto-refresh; if refresh fails, exit 4 | Transparent or re-auth |
| 403 Forbidden | Insufficient scopes or permissions | HTTP 403 | Suggest `--services` re-auth or permission fix, exit 6 | User fixes permissions |
| 404 Not Found | Invalid resource ID | HTTP 404 | Clear message with resource type, exit 5 | User corrects ID |
| 400 Bad Request | Invalid parameters | HTTP 400 | Show Google's error message, exit 1 | User fixes input |
| Workspace-only API | Consumer account accessing Chat/Keep/Groups | HTTP 403 with specific error | Detect and explain Workspace requirement | Clear guidance |
| File too large for export | Google Workspace file size limits | Google API error | Suggest alternative format or chunking | Documented limitation |

### Module 11: `tracking/` -- Email Tracking (M6)

- **Responsibility**: Generate AES-GCM encrypted tracking pixels, manage tracking configuration, query tracking opens via Cloudflare Worker.
- **Public interface**:
  - `pub fn generate_pixel_url(config: &TrackingConfig, metadata: &TrackingMetadata) -> Result<String>`
  - `pub struct TrackingConfig`
  - `pub struct TrackingMetadata`
- **Dependencies**: `aes-gcm`, `base64`, `http` (for querying worker)
- **Implementation order**: 15 (M6)

#### Security Considerations

- **Encryption**: Tracking payloads encrypted with AES-GCM (256-bit key)
- **Key storage**: Tracking key stored in keyring, not on disk
- **Pixel URL**: Contains no plaintext metadata; encrypted blob only

## Failure Modes (System-Level)

| Scenario | Affected Modules | Detection | Recovery Strategy | Degraded Behavior |
|----------|-----------------|-----------|-------------------|-------------------|
| No network connectivity | http, all services | reqwest connection error | Clear error message, suggest checking connection | Offline commands (open, version, config) still work |
| Google API outage | all services | 5xx errors past retry limit | Circuit breaker opens; clear error with status | Other services may still work |
| Keyring service unavailable | auth | Timeout or D-Bus error | Fall back to encrypted file backend | Full functionality with password prompt |
| Disk full | config | Write error | Atomic write prevents corruption; error message | Cannot save config/state; read-only operations work |
| SIGINT during operation | all | Signal handler | Cancel in-progress requests via tokio cancellation | Clean exit with code 130 |
| Corrupted config file | config | JSON5 parse error | Clear error with path; suggest manual fix or deletion | Cannot load config |
| Stale access token | auth, http | 401 response | Automatic refresh via refresh token | Transparent to user |
| Process killed during atomic write | config | Temp file left behind | Next write cleans up temp file | No corruption |

## Security Model

### Trust Boundaries

- **User input (CLI args)**: Untrusted. Validated by clap. Command allowlisting provides additional restriction.
- **Google API responses**: Semi-trusted. Deserialized via serde with typed structs. Unknown fields captured in `flatten` maps.
- **Config files**: Trusted (user's own files). But parsed defensively (JSON5 parser, validated fields).
- **Keyring data**: Trusted storage. OS keyring or encrypted file backend.
- **Network**: Untrusted. All traffic over TLS 1.2+.

### Data Classification

| Data | Classification | Storage | Access Control |
|------|---------------|---------|---------------|
| OAuth client ID/secret | Internal | Config files (0600) | File permissions |
| Refresh tokens | Secret | OS keyring (encrypted) | OS keyring ACL |
| Access tokens | Secret | In-memory only | Process memory |
| Service account keys | Secret | Config files (0600) | File permissions |
| Config file (aliases, settings) | Internal | Config files (0600) | File permissions |
| Tracking encryption key | Secret | OS keyring (encrypted) | OS keyring ACL |
| API responses | Varies | stdout (not stored) | Terminal access |

### Attack Surface

- **CLI argument injection**: Risk -- malicious args from untrusted callers. Mitigation -- clap parsing, `--enable-commands` allowlisting.
- **Credential theft via debug logging**: Risk -- tokens in log output. Mitigation -- never log tokens/secrets at any level.
- **Man-in-the-middle**: Risk -- intercepted API calls. Mitigation -- TLS 1.2+ enforced, no certificate pinning bypass.
- **Keyring compromise**: Risk -- OS keyring accessed by malware. Mitigation -- OS-level keyring protection; not in omega-google's control.
- **Config file tampering**: Risk -- modified config redirects auth. Mitigation -- 0600 permissions; user responsibility.

## Graceful Degradation

| Dependency | Normal Behavior | Degraded Behavior | User Impact |
|-----------|----------------|-------------------|-------------|
| OS keyring | Transparent credential access | Falls back to encrypted file backend | One-time password prompt |
| Google API | API calls succeed | Retry with backoff; circuit breaker | Delayed responses or failure after retry exhaustion |
| Network | Online operations | Offline commands still work (config, version, open, time) | Clear error for API commands |
| TTY (stdin) | Interactive prompts | `--no-input` mode; skip prompts | Must use `--force` for destructive operations |
| TTY (stdout) | Colored text output | Plain text (no colors); auto-JSON if GOG_AUTO_JSON | Piped output works cleanly |
| D-Bus (Linux) | Secret Service keyring | Force file backend when no D-Bus | Encrypted file storage |
| Browser | OAuth redirect flow | Manual flow or remote flow | User pastes URL manually |

## Performance Budgets

| Operation | Latency (p50) | Latency (p99) | Memory | Notes |
|-----------|---------------|---------------|--------|-------|
| CLI startup (no command) | 20ms | 50ms | 10MB | clap parse, no I/O |
| Config file load | 1ms | 5ms | 1MB | Small JSON file |
| Keyring access | 5ms | 50ms | 1MB | OS API call |
| Token refresh | 200ms | 2s | 5MB | Network round-trip |
| API call overhead | 2ms | 10ms | 1MB | Auth header + retry setup |
| Gmail search (10 results) | 300ms | 1s | 10MB | Network-bound |
| Drive download (100MB) | - | - | 20MB | Streaming; not buffered in memory |
| Drive upload (100MB) | - | - | 20MB | Streaming; not buffered in memory |
| JSON output formatting | 1ms | 5ms | 5MB | serde serialization |
| `--all` pagination (100 pages) | - | - | 50MB | Results accumulated |

## Data Flow

### Command Execution Flow

```
1. main.rs: tokio::main starts async runtime
   |
2. cli::execute(args)
   |
   +-- Rewrite desire path args (--fields -> --select)
   |
   +-- Cli::parse(args) via clap
   |
   +-- Enforce --enable-commands allowlist
   |
   +-- Resolve output mode (flags + env + TTY detection)
   |   +-- GOG_AUTO_JSON check (post-parse)
   |
   +-- Construct Ui (color mode)
   |
   +-- Match command variant
       |
3. Command handler (e.g., gmail::search)
   |
   +-- Load config (config::read_config)
   |
   +-- Resolve account (auth::resolve_account)
   |   +-- --account flag > GOG_ACCOUNT > alias > domain map > default
   |
   +-- Build ApiClient (http::build_client)
   |   +-- Read credentials (config::read_client_credentials)
   |   +-- Read refresh token (auth::CredentialStore::get_token)
   |   +-- Construct authenticated client with retry + circuit breaker
   |
   +-- Construct ServiceContext
   |
4. Service function (services::gmail::search)
   |
   +-- Build API URL with query parameters
   |
   +-- ctx.client.get(url).await
   |   |
   |   +-- Inject Authorization: Bearer <access_token>
   |   |   +-- Auto-refresh if expired
   |   |
   |   +-- Send request
   |   |
   |   +-- On 429: exponential backoff + retry (up to 3x)
   |   +-- On 5xx: retry once after 1s; record circuit breaker failure
   |   +-- On success: reset circuit breaker
   |
   +-- Parse response JSON into typed struct
   |
   +-- Map HTTP errors to OmegaError (401->AuthRequired, 404->NotFound, etc.)
   |
5. ctx.write_output(&response)
   |
   +-- JSON mode: serde_json::to_writer_pretty + transforms
   +-- Plain mode: tab-separated field extraction
   +-- Text mode: colored, aligned human output
   |
6. Return exit code
   +-- Ok(()) -> 0
   +-- Err(e) -> exit_code_for(&e)
```

### Token Refresh Flow

```
1. ApiClient prepares request
   |
2. Check cached access token
   |
   +-- Valid (not expired with 60s margin): use it
   +-- Expired or missing:
       |
       +-- OAuth: POST to https://oauth2.googleapis.com/token
       |   with client_id, client_secret, refresh_token
       |
       +-- ServiceAccount: Generate JWT, POST to token endpoint
       |   with assertion, grant_type=jwt-bearer
       |
       +-- Cache new access token with expiry
   |
3. Add Authorization: Bearer <token> header
```

## Design Decisions

| Decision | Alternatives Considered | Justification |
|----------|------------------------|---------------|
| Raw REST API calls | google-apis-rs generated clients | Full control over request construction; no dependency on generated code quality; consistent patterns across all 15 APIs; smaller binary; easier to debug |
| clap derive API | clap builder API; other CLI frameworks | Derive is most ergonomic for 15+ service command trees; compile-time validation; automatic help generation; familiar Rust pattern |
| tokio async runtime | blocking-only; async-std | reqwest requires tokio; all I/O is async; natural fit for concurrent operations like `--all` pagination |
| Single binary (not workspace) | Cargo workspace with lib crate | Simpler build; single Cargo.toml; no internal versioning complexity; `lib.rs` still enables unit testing |
| `serde(flatten)` for unknown fields | Strict deserialization; `serde_json::Value` only | Forward compatibility with Google API changes; typed access to known fields; unknown fields preserved for `--json` passthrough |
| `thiserror` for library, `anyhow` for CLI | `thiserror` only; `anyhow` only | Library errors need typed variants for exit code mapping; CLI-level errors benefit from anyhow's context chaining |
| Keyring service name `omega-google` | Reuse `gogcli` namespace | Clean separation; no conflict with existing gogcli tokens; fresh start matches "no migration" policy |
| `json5` crate for config reading | TOML; standard JSON only | Matches gogcli behavior (JSON5 read, JSON write); comments in config are user-friendly |
| chrono for date/time | time crate; manual parsing | Mature, well-tested; timezone support via chrono-tz; matches the flexible parsing requirements |
| Nix flake for build | Makefile; just; cargo-dist | Reproducible builds; single static binary; dev shell with all dependencies; matches omega-tools ecosystem preference |
| Body-in-memory for retry replay | Streaming with tee; no retry for large bodies | Google API request bodies are typically small (< 1MB); large uploads handled separately with streaming (no retry) |

## External Dependencies

### Runtime Dependencies (Cargo.toml)

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.x | CLI framework (derive API) |
| `tokio` | 1.x | Async runtime (full features) |
| `reqwest` | 0.12.x | HTTP client (rustls-tls, json, multipart, stream) |
| `serde` | 1.x | Serialization framework |
| `serde_json` | 1.x | JSON serialization |
| `json5` | 0.4.x | JSON5 config file reading |
| `keyring` | 3.x | OS credential storage |
| `oauth2` | 4.x | OAuth2 protocol types (optional; may use raw REST instead) |
| `jsonwebtoken` | 9.x | JWT generation for service accounts |
| `crossterm` | 0.28.x | Terminal colors and capabilities |
| `chrono` | 0.4.x | Date/time with timezone |
| `chrono-tz` | 0.10.x | IANA timezone database |
| `thiserror` | 2.x | Typed error derivation |
| `anyhow` | 1.x | Error context chaining |
| `base64` | 0.22.x | Base64 encoding (MIME, JWT) |
| `urlencoding` | 2.x | URL percent-encoding |
| `regex` | 1.x | Regular expressions (sedmat, parsing) |
| `rand` | 0.8.x | Random jitter for backoff |
| `dirs` | 5.x | Platform config directory resolution |
| `aes-gcm` | 0.10.x | AES-GCM encryption for tracking (M6) |
| `tracing` | 0.1.x | Structured logging (verbose mode) |

### Dev Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tempfile` | 3.x | Temporary files for tests |
| `assert_cmd` | 2.x | CLI binary invocation tests |
| `predicates` | 3.x | Test assertions |
| `pretty_assertions` | 1.x | Readable test diffs |
| `wiremock` | 0.6.x | HTTP mock server for API tests |
| `tokio-test` | 0.4.x | Async test utilities |

### Nix Flake Structure

```nix
{
  description = "omega-google: Google Workspace CLI in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common args for all crane builds
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          buildInputs = with pkgs; [
            openssl
            # macOS-specific
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        };

        # Build just the cargo dependencies for caching
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the binary
        omega-google = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in {
        packages = {
          default = omega-google;
          omega-google = omega-google;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ omega-google ];
          packages = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-nextest
          ];
        };

        checks = {
          inherit omega-google;
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          fmt = craneLib.cargoFmt { src = commonArgs.src; };
          nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
          });
        };
      }
    );
}
```

## Milestone Implementation Guide

### M1: Scaffolding + Auth + Config (Foundation)

**Build order within M1**:

1. `Cargo.toml` + `flake.nix` -- project compiles with all dependencies
2. `error/` -- error types and exit codes (everything depends on this)
3. `config/` -- config file management (auth needs paths)
4. `ui/` -- terminal output (all commands need this)
5. `output/` -- output formatting (all commands need this)
6. `time/` -- date/time parsing (used by auth and services)
7. `auth/` -- OAuth, keyring, scopes (depends on config, error)
8. `http/` -- authenticated HTTP client (depends on auth)
9. `cli/` root -- root flags, command dispatch skeleton
10. `cli/` auth commands -- auth add/remove/list/status/services/tokens/alias/credentials/keyring
11. `cli/` config commands -- config get/set/unset/list/keys/path
12. `cli/` version + time commands
13. Unit tests for all M1 modules

**Deliverable**: A binary that can authenticate with Google, manage config, and has the HTTP infrastructure ready for services.

### M2: Core Services (Gmail + Calendar + Drive)

**Build order within M2**:

1. `services/common.rs` -- pagination, ServiceContext
2. `services/gmail/types.rs` -- API types
3. `services/gmail/` -- search, thread, message, labels, send, drafts, watch, history, batch, settings
4. `services/calendar/types.rs` -- API types
5. `services/calendar/` -- events, calendars, freebusy, respond, search, special, colors
6. `services/drive/types.rs` -- API types
7. `services/drive/` -- list, files, folders, permissions, comments, drives
8. `services/export.rs` -- shared Drive export logic
9. `cli/` service command files -- gmail.rs, calendar.rs, drive.rs
10. `cli/desire_paths.rs` -- send, ls, search, download, upload, login, logout, status, me, whoami
11. Integration test scaffolding

**Deliverable**: Fully functional Gmail, Calendar, and Drive access with all subcommands.

### M3: Productivity Services (Docs + Sheets + Slides + Forms)

**Build order within M3**:

1. `services/docs/` -- export, content, edit, comments
2. `services/docs/sedmat.rs` -- sed-like editing engine (most complex component)
3. `services/docs/markdown.rs` -- markdown-to-Docs formatting
4. `services/sheets/` -- read, write, format, structure, a1 parser
5. `services/slides/` -- export, presentations, slide ops, notes, markdown
6. `services/forms/` -- forms, responses
7. CLI command files for each service

### M4: Collaboration Services (Chat + Classroom + Tasks + Contacts + People)

### M5: Admin/Workspace Services (Groups + Keep + Apps Script)

### M6: Polish (Tracking + Completion + Allowlisting + Agent + Tests)

## Requirement Traceability

| Requirement ID | Architecture Section | Module(s) |
|---------------|---------------------|-----------|
| REQ-SCAFFOLD-001 | Module 1: main.rs, Project Structure | `Cargo.toml`, `src/main.rs`, `src/lib.rs` |
| REQ-SCAFFOLD-002 | Nix Flake Structure | `flake.nix` (devShells) |
| REQ-SCAFFOLD-003 | Nix Flake Structure | `flake.nix` (packages) |
| REQ-SCAFFOLD-004 | External Dependencies | `Cargo.toml` |
| REQ-SCAFFOLD-005 | Project Structure, Modules 2-11 | `src/cli/`, `src/config/`, `src/auth/`, `src/http/`, `src/output/`, `src/ui/`, `src/error/` |
| REQ-CLI-001 | Module 2: cli/ | `src/cli/root.rs` (RootFlags struct) |
| REQ-CLI-002 | Module 2: cli/ | `src/cli/root.rs` (env attributes on RootFlags fields) |
| REQ-CLI-003 | Module 2: cli/ | `src/cli/version.rs` |
| REQ-CLI-004 | Module 2: cli/ | `src/cli/version.rs` |
| REQ-CLI-005 | Module 9: time/ | `src/cli/time_cmd.rs`, `src/time/mod.rs` |
| REQ-CLI-006 | Module 7: ui/ | `src/ui/mod.rs` (stderr for errors, stdout for data) |
| REQ-CLI-007 | Module 8: error/ | `src/error/exit.rs` (exit code constants and mapping) |
| REQ-CLI-008 | Module 8: error/ | `src/error/mod.rs` (error formatting, not raw clap output) |
| REQ-CLI-009 | Module 2: cli/ | `src/cli/desire_paths.rs` (--fields rewriting) |
| REQ-CLI-010 | Module 2: cli/ | `src/cli/mod.rs` (Command::Send variant) |
| REQ-CLI-011 | Module 2: cli/ | `src/cli/mod.rs` (Command::Ls variant) |
| REQ-CLI-012 | Module 2: cli/ | `src/cli/mod.rs` (Command::Search variant) |
| REQ-CLI-013 | Module 2: cli/ | `src/cli/mod.rs` (Command::Download variant) |
| REQ-CLI-014 | Module 2: cli/ | `src/cli/mod.rs` (Command::Upload variant) |
| REQ-CLI-015 | Module 2: cli/ | `src/cli/mod.rs` (Command::Login variant) |
| REQ-CLI-016 | Module 2: cli/ | `src/cli/mod.rs` (Command::Logout variant) |
| REQ-CLI-017 | Module 2: cli/ | `src/cli/mod.rs` (Command::Status variant) |
| REQ-CLI-018 | Module 2: cli/ | `src/cli/mod.rs` (Command::Me, Command::Whoami variants) |
| REQ-CLI-019 | Module 2: cli/ | `src/cli/open.rs` |
| REQ-CLI-020 | Module 2: cli/ | `src/cli/completion.rs` |
| REQ-CLI-021 | Module 2: cli/, Module 10: services/ | `src/cli/root.rs` (dry_run flag), all service handlers |
| REQ-CLI-022 | Module 7: ui/ | `src/ui/prompt.rs` (force flag handling) |
| REQ-CLI-023 | Module 7: ui/ | `src/ui/prompt.rs` (no_input flag handling) |
| REQ-CLI-024 | Module 10: services/ | `src/services/common.rs` (pagination helpers) |
| REQ-CONFIG-001 | Module 3: config/ | `src/config/file.rs`, `src/config/paths.rs` |
| REQ-CONFIG-002 | Module 3: config/ | `src/config/file.rs` (ConfigFile struct with flatten) |
| REQ-CONFIG-003 | Module 2: cli/, Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-004 | Module 2: cli/, Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-005 | Module 2: cli/, Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-006 | Module 2: cli/, Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-007 | Module 2: cli/, Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/mod.rs` |
| REQ-CONFIG-008 | Module 2: cli/, Module 3: config/ | `src/cli/config_cmd.rs`, `src/config/paths.rs` |
| REQ-CONFIG-009 | Module 3: config/ | `src/config/credentials.rs` |
| REQ-AUTH-001 | Module 4: auth/, Module 2: cli/ | `src/cli/auth.rs`, `src/config/credentials.rs` |
| REQ-AUTH-002 | Module 4: auth/, Module 2: cli/ | `src/cli/auth.rs`, `src/config/credentials.rs` |
| REQ-AUTH-003 | Module 4: auth/ | `src/auth/oauth.rs` |
| REQ-AUTH-004 | Module 4: auth/ | `src/auth/oauth.rs` (manual flow branch) |
| REQ-AUTH-005 | Module 4: auth/ | `src/auth/oauth.rs` (remote two-step flow) |
| REQ-AUTH-006 | Module 4: auth/ | `src/auth/oauth.rs` (force_consent parameter) |
| REQ-AUTH-007 | Module 4: auth/, Module 2: cli/ | `src/cli/auth.rs`, `src/auth/token.rs` |
| REQ-AUTH-008 | Module 4: auth/, Module 2: cli/ | `src/cli/auth.rs`, `src/auth/keyring.rs` |
| REQ-AUTH-009 | Module 4: auth/, Module 2: cli/ | `src/cli/auth.rs` |
| REQ-AUTH-010 | Module 4: auth/, Module 2: cli/ | `src/cli/auth.rs`, `src/auth/scopes.rs` |
| REQ-AUTH-011 | Module 4: auth/, Module 2: cli/ | `src/cli/auth.rs`, `src/auth/keyring.rs` |
| REQ-AUTH-012 | Module 4: auth/, Module 3: config/ | `src/cli/auth.rs`, `src/config/file.rs` |
| REQ-AUTH-013 | Module 4: auth/ | `src/auth/keyring.rs` |
| REQ-AUTH-014 | Module 4: auth/ | `src/auth/keyring.rs` (file fallback) |
| REQ-AUTH-015 | Module 4: auth/, Module 3: config/ | `src/auth/keyring.rs`, `src/config/file.rs` |
| REQ-AUTH-016 | Module 4: auth/ | `src/auth/scopes.rs` |
| REQ-AUTH-017 | Module 4: auth/ | `src/auth/service_account.rs` |
| REQ-AUTH-018 | Module 4: auth/, Module 2: cli/ | `src/cli/auth.rs`, `src/auth/keyring.rs` |
| REQ-AUTH-019 | Module 4: auth/ | `src/auth/account.rs` |
| REQ-AUTH-020 | Module 4: auth/ | `src/auth/keyring.rs` (timeout logic) |
| REQ-HTTP-001 | Module 5: http/ | `src/http/client.rs` |
| REQ-HTTP-002 | Module 5: http/ | `src/http/retry.rs` |
| REQ-HTTP-003 | Module 5: http/ | `src/http/retry.rs` |
| REQ-HTTP-004 | Module 5: http/ | `src/http/circuit_breaker.rs` |
| REQ-HTTP-005 | Module 5: http/ | `src/http/retry.rs` (body replay) |
| REQ-HTTP-006 | Module 5: http/ | `src/http/retry.rs` (tokio::select! with cancellation) |
| REQ-OUTPUT-001 | Module 6: output/ | `src/output/mode.rs`, `src/output/json.rs`, `src/output/plain.rs`, `src/output/text.rs` |
| REQ-OUTPUT-002 | Module 6: output/ | `src/output/transform.rs` (results_only) |
| REQ-OUTPUT-003 | Module 6: output/ | `src/output/transform.rs` (field selection) |
| REQ-OUTPUT-004 | Module 6: output/, Module 2: cli/ | `src/output/mode.rs`, `src/cli/root.rs` (GOG_AUTO_JSON) |
| REQ-OUTPUT-005 | Module 7: ui/ | `src/ui/color.rs` |
| REQ-OUTPUT-006 | Module 6: output/ | `src/output/csv.rs` (M6) |
| REQ-UI-001 | Module 7: ui/ | `src/ui/color.rs` |
| REQ-UI-002 | Module 7: ui/ | `src/ui/progress.rs` |
| REQ-UI-003 | Module 8: error/ | `src/error/api_error.rs` |
| REQ-GMAIL-001 | Module 10: services/gmail | `src/services/gmail/search.rs` |
| REQ-GMAIL-002 | Module 10: services/gmail | `src/services/gmail/search.rs` (message search) |
| REQ-GMAIL-003 | Module 10: services/gmail | `src/services/gmail/thread.rs` |
| REQ-GMAIL-004 | Module 10: services/gmail | `src/services/gmail/thread.rs` (modify) |
| REQ-GMAIL-005 | Module 10: services/gmail | `src/services/gmail/thread.rs` (attachments) |
| REQ-GMAIL-006 | Module 10: services/gmail | `src/services/gmail/message.rs` |
| REQ-GMAIL-007 | Module 10: services/gmail | `src/services/gmail/message.rs` (attachment download) |
| REQ-GMAIL-008 | Module 10: services/gmail | `src/services/gmail/thread.rs` (URL) |
| REQ-GMAIL-009 | Module 10: services/gmail | `src/services/gmail/labels.rs` |
| REQ-GMAIL-010 | Module 10: services/gmail | `src/services/gmail/send.rs`, `src/services/gmail/mime.rs` |
| REQ-GMAIL-011 | Module 10: services/gmail | `src/services/gmail/drafts.rs` |
| REQ-GMAIL-012 | Module 10: services/gmail | `src/services/gmail/watch.rs` |
| REQ-GMAIL-013 | Module 10: services/gmail | `src/services/gmail/history.rs` |
| REQ-GMAIL-014 | Module 10: services/gmail | `src/services/gmail/batch.rs` |
| REQ-GMAIL-015 | Module 10: services/gmail | `src/services/gmail/settings.rs` (filters) |
| REQ-GMAIL-016 | Module 10: services/gmail | `src/services/gmail/settings.rs` (forwarding) |
| REQ-GMAIL-017 | Module 10: services/gmail | `src/services/gmail/settings.rs` (send-as) |
| REQ-GMAIL-018 | Module 10: services/gmail | `src/services/gmail/settings.rs` (delegates) |
| REQ-GMAIL-019 | Module 10: services/gmail | `src/services/gmail/settings.rs` (vacation) |
| REQ-GMAIL-020 | Module 10: services/gmail | `src/services/gmail/settings.rs` (auto-forward) |
| REQ-CAL-001 | Module 10: services/calendar | `src/services/calendar/calendars.rs` |
| REQ-CAL-002 | Module 10: services/calendar | `src/services/calendar/calendars.rs` (ACL) |
| REQ-CAL-003 | Module 10: services/calendar | `src/services/calendar/events.rs` |
| REQ-CAL-004 | Module 10: services/calendar | `src/services/calendar/events.rs` (get) |
| REQ-CAL-005 | Module 10: services/calendar | `src/services/calendar/events.rs` (create) |
| REQ-CAL-006 | Module 10: services/calendar | `src/services/calendar/events.rs` (update) |
| REQ-CAL-007 | Module 10: services/calendar | `src/services/calendar/events.rs` (delete) |
| REQ-CAL-008 | Module 10: services/calendar | `src/services/calendar/freebusy.rs` |
| REQ-CAL-009 | Module 10: services/calendar | `src/services/calendar/respond.rs` |
| REQ-CAL-010 | Module 10: services/calendar | `src/services/calendar/search.rs` |
| REQ-CAL-011 | Module 10: services/calendar | `src/services/calendar/events.rs` (time) |
| REQ-CAL-012 | Module 10: services/calendar | `src/services/calendar/calendars.rs` (users) |
| REQ-CAL-013 | Module 10: services/calendar | `src/services/calendar/events.rs` (team) |
| REQ-CAL-014 | Module 10: services/calendar | `src/services/calendar/colors.rs` |
| REQ-CAL-015 | Module 10: services/calendar | `src/services/calendar/events.rs` (conflicts) |
| REQ-CAL-016 | Module 10: services/calendar | `src/services/calendar/respond.rs` (propose-time) |
| REQ-CAL-017 | Module 10: services/calendar | `src/services/calendar/special.rs` (focus-time) |
| REQ-CAL-018 | Module 10: services/calendar | `src/services/calendar/special.rs` (OOO) |
| REQ-CAL-019 | Module 10: services/calendar | `src/services/calendar/special.rs` (working-location) |
| REQ-CAL-020 | Module 9: time/ | `src/time/parse.rs` |
| REQ-CAL-021 | Module 10: services/calendar | `src/services/calendar/events.rs` (recurrence) |
| REQ-CAL-022 | Module 10: services/calendar | `src/services/calendar/events.rs` (weekday enrichment) |
| REQ-DRIVE-001 | Module 10: services/drive | `src/services/drive/list.rs` |
| REQ-DRIVE-002 | Module 10: services/drive | `src/services/drive/list.rs` (search) |
| REQ-DRIVE-003 | Module 10: services/drive | `src/services/drive/files.rs` (get) |
| REQ-DRIVE-004 | Module 10: services/drive | `src/services/drive/files.rs` (download) |
| REQ-DRIVE-005 | Module 10: services/drive | `src/services/drive/files.rs` (upload) |
| REQ-DRIVE-006 | Module 10: services/drive | `src/services/drive/folders.rs` (mkdir) |
| REQ-DRIVE-007 | Module 10: services/drive | `src/services/drive/files.rs` (delete) |
| REQ-DRIVE-008 | Module 10: services/drive | `src/services/drive/folders.rs` (move) |
| REQ-DRIVE-009 | Module 10: services/drive | `src/services/drive/files.rs` (rename) |
| REQ-DRIVE-010 | Module 10: services/drive | `src/services/drive/permissions.rs` (share) |
| REQ-DRIVE-011 | Module 10: services/drive | `src/services/drive/permissions.rs` (list) |
| REQ-DRIVE-012 | Module 10: services/drive | `src/services/drive/permissions.rs` (unshare) |
| REQ-DRIVE-013 | Module 10: services/drive | `src/services/drive/files.rs` (URL) |
| REQ-DRIVE-014 | Module 10: services/drive | `src/services/drive/drives.rs` |
| REQ-DRIVE-015 | Module 10: services/drive | `src/services/drive/files.rs` (copy) |
| REQ-DRIVE-016 | Module 10: services/drive | `src/services/drive/comments.rs` |
| REQ-DRIVE-017 | Module 10: services/drive | `src/services/drive/list.rs` (--all-drives default) |
| REQ-DOCS-001 | Module 10: services/docs | `src/services/docs/export.rs`, `src/services/export.rs` |
| REQ-DOCS-002 | Module 10: services/docs | `src/services/docs/content.rs` (info) |
| REQ-DOCS-003 | Module 10: services/docs | `src/services/docs/content.rs` (create) |
| REQ-DOCS-004 | Module 10: services/docs | `src/services/docs/content.rs` (copy) |
| REQ-DOCS-005 | Module 10: services/docs | `src/services/docs/content.rs` (cat) |
| REQ-DOCS-006 | Module 10: services/docs | `src/services/docs/content.rs` (list-tabs) |
| REQ-DOCS-007 | Module 10: services/docs | `src/services/docs/comments.rs` |
| REQ-DOCS-008 | Module 10: services/docs | `src/services/docs/edit.rs` (write) |
| REQ-DOCS-009 | Module 10: services/docs | `src/services/docs/edit.rs` (insert) |
| REQ-DOCS-010 | Module 10: services/docs | `src/services/docs/edit.rs` (delete) |
| REQ-DOCS-011 | Module 10: services/docs | `src/services/docs/edit.rs` (find-replace) |
| REQ-DOCS-012 | Module 10: services/docs | `src/services/docs/edit.rs` (edit) |
| REQ-DOCS-013 | Module 10: services/docs | `src/services/docs/edit.rs` (update) |
| REQ-DOCS-014 | Module 10: services/docs | `src/services/docs/sedmat.rs` |
| REQ-DOCS-015 | Module 10: services/docs | `src/services/docs/edit.rs` (clear) |
| REQ-DOCS-016 | Module 10: services/docs | `src/services/docs/markdown.rs` |
| REQ-SHEETS-001 | Module 10: services/sheets | `src/services/sheets/read.rs` |
| REQ-SHEETS-002 | Module 10: services/sheets | `src/services/sheets/write.rs` (update) |
| REQ-SHEETS-003 | Module 10: services/sheets | `src/services/sheets/write.rs` (append) |
| REQ-SHEETS-004 | Module 10: services/sheets | `src/services/sheets/structure.rs` (insert) |
| REQ-SHEETS-005 | Module 10: services/sheets | `src/services/sheets/write.rs` (clear) |
| REQ-SHEETS-006 | Module 10: services/sheets | `src/services/sheets/format.rs` |
| REQ-SHEETS-007 | Module 10: services/sheets | `src/services/sheets/read.rs` (notes) |
| REQ-SHEETS-008 | Module 10: services/sheets | `src/services/sheets/structure.rs` (metadata) |
| REQ-SHEETS-009 | Module 10: services/sheets | `src/services/sheets/structure.rs` (create) |
| REQ-SHEETS-010 | Module 10: services/sheets | `src/services/sheets/structure.rs` (copy) |
| REQ-SHEETS-011 | Module 10: services/sheets | `src/services/sheets/read.rs` (export), `src/services/export.rs` |
| REQ-SHEETS-012 | Module 10: services/sheets | `src/services/sheets/a1.rs` |
| REQ-SLIDES-001 | Module 10: services/slides | `src/services/slides/export.rs`, `src/services/export.rs` |
| REQ-SLIDES-002 | Module 10: services/slides | `src/services/slides/presentations.rs` (info) |
| REQ-SLIDES-003 | Module 10: services/slides | `src/services/slides/presentations.rs` (create) |
| REQ-SLIDES-004 | Module 10: services/slides | `src/services/slides/markdown.rs` |
| REQ-SLIDES-005 | Module 10: services/slides | `src/services/slides/presentations.rs` (copy) |
| REQ-SLIDES-006 | Module 10: services/slides | `src/services/slides/slides_ops.rs` (list) |
| REQ-SLIDES-007 | Module 10: services/slides | `src/services/slides/slides_ops.rs` (add) |
| REQ-SLIDES-008 | Module 10: services/slides | `src/services/slides/slides_ops.rs` (delete) |
| REQ-SLIDES-009 | Module 10: services/slides | `src/services/slides/slides_ops.rs` (read) |
| REQ-SLIDES-010 | Module 10: services/slides | `src/services/slides/notes.rs` |
| REQ-SLIDES-011 | Module 10: services/slides | `src/services/slides/slides_ops.rs` (replace) |
| REQ-FORMS-001 | Module 10: services/forms | `src/services/forms/forms.rs` (get) |
| REQ-FORMS-002 | Module 10: services/forms | `src/services/forms/forms.rs` (create) |
| REQ-FORMS-003 | Module 10: services/forms | `src/services/forms/responses.rs` (list) |
| REQ-FORMS-004 | Module 10: services/forms | `src/services/forms/responses.rs` (get) |
| REQ-CHAT-001 | Module 10: services/chat | `src/services/chat/spaces.rs` (list) |
| REQ-CHAT-002 | Module 10: services/chat | `src/services/chat/spaces.rs` (find) |
| REQ-CHAT-003 | Module 10: services/chat | `src/services/chat/spaces.rs` (create) |
| REQ-CHAT-004 | Module 10: services/chat | `src/services/chat/messages.rs` (list) |
| REQ-CHAT-005 | Module 10: services/chat | `src/services/chat/messages.rs` (send) |
| REQ-CHAT-006 | Module 10: services/chat | `src/services/chat/messages.rs` (threads list) |
| REQ-CHAT-007 | Module 10: services/chat | `src/services/chat/dm.rs` (space) |
| REQ-CHAT-008 | Module 10: services/chat | `src/services/chat/dm.rs` (send) |
| REQ-CLASS-001 | Module 10: services/classroom | `src/services/classroom/courses.rs` |
| REQ-CLASS-002 | Module 10: services/classroom | `src/services/classroom/roster.rs` (students) |
| REQ-CLASS-003 | Module 10: services/classroom | `src/services/classroom/roster.rs` (teachers) |
| REQ-CLASS-004 | Module 10: services/classroom | `src/services/classroom/roster.rs` (combined) |
| REQ-CLASS-005 | Module 10: services/classroom | `src/services/classroom/coursework.rs` |
| REQ-CLASS-006 | Module 10: services/classroom | `src/services/classroom/materials.rs` |
| REQ-CLASS-007 | Module 10: services/classroom | `src/services/classroom/submissions.rs` |
| REQ-CLASS-008 | Module 10: services/classroom | `src/services/classroom/announcements.rs` |
| REQ-CLASS-009 | Module 10: services/classroom | `src/services/classroom/topics.rs` |
| REQ-CLASS-010 | Module 10: services/classroom | `src/services/classroom/invitations.rs` |
| REQ-CLASS-011 | Module 10: services/classroom | `src/services/classroom/guardians.rs` |
| REQ-CLASS-012 | Module 10: services/classroom | `src/services/classroom/guardians.rs` (invitations) |
| REQ-CLASS-013 | Module 10: services/classroom | `src/services/classroom/roster.rs` (profile) |
| REQ-TASKS-001 | Module 10: services/tasks | `src/services/tasks/tasklists.rs` (list) |
| REQ-TASKS-002 | Module 10: services/tasks | `src/services/tasks/tasklists.rs` (create) |
| REQ-TASKS-003 | Module 10: services/tasks | `src/services/tasks/tasks_ops.rs` (list) |
| REQ-TASKS-004 | Module 10: services/tasks | `src/services/tasks/tasks_ops.rs` (get) |
| REQ-TASKS-005 | Module 10: services/tasks | `src/services/tasks/tasks_ops.rs` (add) |
| REQ-TASKS-006 | Module 10: services/tasks | `src/services/tasks/tasks_ops.rs` (update) |
| REQ-TASKS-007 | Module 10: services/tasks | `src/services/tasks/tasks_ops.rs` (done) |
| REQ-TASKS-008 | Module 10: services/tasks | `src/services/tasks/tasks_ops.rs` (undo) |
| REQ-TASKS-009 | Module 10: services/tasks | `src/services/tasks/tasks_ops.rs` (delete) |
| REQ-TASKS-010 | Module 10: services/tasks | `src/services/tasks/tasks_ops.rs` (clear) |
| REQ-CONTACTS-001 | Module 10: services/contacts | `src/services/contacts/contacts.rs` (search) |
| REQ-CONTACTS-002 | Module 10: services/contacts | `src/services/contacts/contacts.rs` (list) |
| REQ-CONTACTS-003 | Module 10: services/contacts | `src/services/contacts/contacts.rs` (get) |
| REQ-CONTACTS-004 | Module 10: services/contacts | `src/services/contacts/contacts.rs` (create) |
| REQ-CONTACTS-005 | Module 10: services/contacts | `src/services/contacts/contacts.rs` (update) |
| REQ-CONTACTS-006 | Module 10: services/contacts | `src/services/contacts/contacts.rs` (delete) |
| REQ-CONTACTS-007 | Module 10: services/contacts | `src/services/contacts/directory.rs` |
| REQ-CONTACTS-008 | Module 10: services/contacts | `src/services/contacts/other.rs` |
| REQ-PEOPLE-001 | Module 10: services/people | `src/services/people/profile.rs` (me) |
| REQ-PEOPLE-002 | Module 10: services/people | `src/services/people/profile.rs` (get) |
| REQ-PEOPLE-003 | Module 10: services/people | `src/services/people/profile.rs` (search) |
| REQ-PEOPLE-004 | Module 10: services/people | `src/services/people/profile.rs` (relations) |
| REQ-GROUPS-001 | Module 10: services/groups | `src/services/groups/groups.rs` (list) |
| REQ-GROUPS-002 | Module 10: services/groups | `src/services/groups/groups.rs` (members) |
| REQ-GROUPS-003 | Module 8: error/ | `src/error/api_error.rs` (Cloud Identity error guidance) |
| REQ-KEEP-001 | Module 10: services/keep | `src/services/keep/notes.rs` (list) |
| REQ-KEEP-002 | Module 10: services/keep | `src/services/keep/notes.rs` (get) |
| REQ-KEEP-003 | Module 10: services/keep | `src/services/keep/notes.rs` (search) |
| REQ-KEEP-004 | Module 10: services/keep | `src/services/keep/attachments.rs` |
| REQ-KEEP-005 | Module 4: auth/ | `src/auth/service_account.rs` |
| REQ-SCRIPT-001 | Module 10: services/appscript | `src/services/appscript/projects.rs` (get) |
| REQ-SCRIPT-002 | Module 10: services/appscript | `src/services/appscript/projects.rs` (content) |
| REQ-SCRIPT-003 | Module 10: services/appscript | `src/services/appscript/projects.rs` (run) |
| REQ-SCRIPT-004 | Module 10: services/appscript | `src/services/appscript/projects.rs` (create) |
| REQ-TRACK-001 | Module 11: tracking/ | `src/tracking/config.rs` |
| REQ-TRACK-002 | Module 11: tracking/ | `src/tracking/mod.rs` (query opens) |
| REQ-TRACK-003 | Module 11: tracking/ | `src/tracking/config.rs` (status) |
| REQ-TRACK-004 | Module 11: tracking/, Module 10: services/gmail | `src/tracking/pixel.rs`, `src/services/gmail/send.rs` |
| REQ-AGENT-001 | Module 2: cli/ | `src/cli/agent.rs` (exit-codes command) |
| REQ-AGENT-002 | Module 2: cli/ | `src/cli/agent.rs` (schema command) |
| REQ-AGENT-003 | Module 2: cli/ | `src/cli/root.rs` (enable_commands enforcement) |
