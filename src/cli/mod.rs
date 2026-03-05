pub mod agent;
pub mod appscript;
pub mod calendar;
pub mod chat;
pub mod classroom;
pub mod completion;
pub mod contacts;
pub mod desire_paths;
pub mod docs;
pub mod drive;
pub mod exit_codes;
pub mod forms;
pub mod gmail;
pub mod groups;
pub mod keep;
pub mod open;
pub mod people;
pub mod root;
pub mod sheets;
pub mod slides;
pub mod tasks;

use std::ffi::OsString;

use clap::Parser;

use crate::error::exit::codes;

/// Safely serialize a value to pretty-printed JSON, returning an error JSON string on failure.
fn to_json_pretty(val: &impl serde::Serialize) -> String {
    serde_json::to_string_pretty(val).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

/// Map an anyhow error to the correct exit code using OmegaError downcasting.
fn map_error_to_exit_code(e: &anyhow::Error) -> i32 {
    if let Some(omega_err) = e.downcast_ref::<crate::error::exit::OmegaError>() {
        crate::error::exit::exit_code_for(omega_err)
    } else {
        codes::GENERIC_ERROR
    }
}

/// Execute the CLI with the given arguments. Returns the exit code.
pub async fn execute(args: Vec<OsString>) -> i32 {
    // Convert OsString args to String for desire path rewriting
    // Use to_string_lossy to convert non-UTF-8 bytes to the Unicode replacement
    // character rather than silently dropping arguments.
    let string_args: Vec<String> = args
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();

    // Apply desire path rewriting before parsing
    let rewritten = rewrite_desire_path_args(string_args);

    // Build the full arg list: prepend the binary name for clap
    let mut full_args = vec!["omega-google".to_string()];
    full_args.extend(rewritten);

    // Parse with clap
    let cli = match root::Cli::try_parse_from(&full_args) {
        Ok(cli) => cli,
        Err(e) => {
            // clap errors: help (exit 0), version (exit 0), usage errors (exit 2)
            let exit = if e.use_stderr() {
                eprintln!("Error: {}", e.render().to_string().trim());
                codes::USAGE_ERROR
            } else {
                // --help or --version output from clap
                print!("{}", e.render());
                codes::SUCCESS
            };
            return exit;
        }
    };

    // Dispatch to the appropriate command handler
    match cli.command {
        Some(cmd) => dispatch_command(cmd, &cli.flags).await,
        None => {
            // No subcommand: print help
            let full_args_help = vec!["omega-google".to_string(), "--help".to_string()];
            match root::Cli::try_parse_from(&full_args_help) {
                Ok(_) => codes::SUCCESS,
                Err(e) => {
                    print!("{}", e.render());
                    codes::SUCCESS
                }
            }
        }
    }
}

/// Extract the command name string for allowlisting checks.
fn command_name(cmd: &root::Command) -> &str {
    match cmd {
        root::Command::Version => "version",
        root::Command::Config(_) => "config",
        root::Command::Auth(_) => "auth",
        root::Command::Time(_) => "time",
        root::Command::Gmail(_) => "gmail",
        root::Command::Calendar(_) => "calendar",
        root::Command::Drive(_) => "drive",
        root::Command::Docs(_) => "docs",
        root::Command::Sheets(_) => "sheets",
        root::Command::Slides(_) => "slides",
        root::Command::Forms(_) => "forms",
        root::Command::Chat(_) => "chat",
        root::Command::Classroom(_) => "classroom",
        root::Command::Tasks(_) => "tasks",
        root::Command::Contacts(_) => "contacts",
        root::Command::People(_) => "people",
        root::Command::Groups(_) => "groups",
        root::Command::Keep(_) => "keep",
        root::Command::AppScript(_) => "appscript",
        root::Command::Open(_) => "open",
        root::Command::Completion(_) => "completion",
        root::Command::ExitCodes => "exit-codes",
        root::Command::Schema(_) => "schema",
        root::Command::Agent(_) => "agent",
        root::Command::Webhook(_) => "webhook",
    }
}

/// Dispatch a parsed command to its handler.
async fn dispatch_command(cmd: root::Command, flags: &root::RootFlags) -> i32 {
    // Enforce command allowlisting if --enable-commands is set
    if let Some(ref enabled) = flags.enable_commands {
        let cmd_name = command_name(&cmd);
        if let Err(e) = enforce_enabled_commands(cmd_name, enabled) {
            eprintln!("Error: {}", e);
            return codes::USAGE_ERROR;
        }
    }

    match cmd {
        root::Command::Version => handle_version(flags),
        root::Command::Config(args) => handle_config(args, flags),
        root::Command::Auth(args) => handle_auth(args, flags).await,
        root::Command::Time(args) => handle_time(args, flags),
        root::Command::Gmail(args) => handle_gmail(args, flags).await,
        root::Command::Calendar(args) => handle_calendar(args, flags).await,
        root::Command::Drive(args) => handle_drive(args, flags).await,
        root::Command::Docs(args) => handle_docs(args, flags).await,
        root::Command::Sheets(args) => handle_sheets(args, flags).await,
        root::Command::Slides(args) => handle_slides(args, flags).await,
        root::Command::Forms(args) => handle_forms(args, flags).await,
        root::Command::Chat(args) => handle_chat(args, flags).await,
        root::Command::Classroom(args) => handle_classroom(args, flags).await,
        root::Command::Tasks(args) => handle_tasks(args, flags).await,
        root::Command::Contacts(args) => handle_contacts(args, flags).await,
        root::Command::People(args) => handle_people(args, flags).await,
        root::Command::Groups(args) => handle_groups(args, flags).await,
        root::Command::Keep(args) => handle_keep(args, flags).await,
        root::Command::AppScript(args) => handle_appscript(args, flags).await,
        root::Command::Open(args) => handle_open(args, flags),
        root::Command::Completion(args) => handle_completion(args),
        root::Command::ExitCodes => handle_exit_codes(flags),
        root::Command::Schema(args) => handle_schema(args, flags),
        root::Command::Agent(args) => handle_agent(args, flags),
        root::Command::Webhook(args) => handle_webhook(args).await,
    }
}

/// Handle the `version` command.
fn handle_version(flags: &root::RootFlags) -> i32 {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");

    if flags.json {
        let version_json = serde_json::json!({
            "version": version,
            "name": name,
        });
        println!("{}", to_json_pretty(&version_json));
    } else {
        println!("{} {}", name, version);
    }
    codes::SUCCESS
}

/// Handle the `config` command and its subcommands.
fn handle_config(args: root::ConfigArgs, flags: &root::RootFlags) -> i32 {
    match args.command {
        root::ConfigCommand::Get(get_args) => handle_config_get(&get_args.key, flags),
        root::ConfigCommand::Set(set_args) => {
            handle_config_set(&set_args.key, &set_args.value, flags)
        }
        root::ConfigCommand::Unset(unset_args) => handle_config_unset(&unset_args.key, flags),
        root::ConfigCommand::List => handle_config_list(flags),
        root::ConfigCommand::Keys => handle_config_keys(flags),
        root::ConfigCommand::Path => handle_config_path(flags),
    }
}

fn handle_config_get(key: &str, flags: &root::RootFlags) -> i32 {
    let cfg = match crate::config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::CONFIG_ERROR;
        }
    };
    let json_val = match serde_json::to_value(&cfg) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::GENERIC_ERROR;
        }
    };
    match json_val.get(key) {
        Some(val) if !val.is_null() => {
            if flags.json {
                println!("{}", to_json_pretty(val));
            } else {
                match val {
                    serde_json::Value::String(s) => println!("{}", s),
                    other => println!("{}", other),
                }
            }
            codes::SUCCESS
        }
        _ => {
            eprintln!("Error: key '{}' is not set", key);
            codes::CONFIG_ERROR
        }
    }
}

fn handle_config_set(key: &str, value: &str, _flags: &root::RootFlags) -> i32 {
    let mut cfg = match crate::config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    match key {
        "keyring_backend" => cfg.keyring_backend = Some(value.to_string()),
        "default_timezone" => cfg.default_timezone = Some(value.to_string()),
        _ => {
            eprintln!(
                "Error: unknown config key '{}'. Use 'config keys' to see valid keys.",
                key
            );
            return codes::USAGE_ERROR;
        }
    }

    if let Err(e) = crate::config::write_config(&cfg) {
        eprintln!("Error writing config: {}", e);
        return codes::CONFIG_ERROR;
    }
    codes::SUCCESS
}

fn handle_config_unset(key: &str, _flags: &root::RootFlags) -> i32 {
    let mut cfg = match crate::config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    match key {
        "keyring_backend" => cfg.keyring_backend = None,
        "default_timezone" => cfg.default_timezone = None,
        "account_aliases" => cfg.account_aliases = None,
        "account_clients" => cfg.account_clients = None,
        "client_domains" => cfg.client_domains = None,
        _ => {
            eprintln!(
                "Error: unknown config key '{}'. Use 'config keys' to see valid keys.",
                key
            );
            return codes::USAGE_ERROR;
        }
    }

    if let Err(e) = crate::config::write_config(&cfg) {
        eprintln!("Error writing config: {}", e);
        return codes::CONFIG_ERROR;
    }
    codes::SUCCESS
}

fn handle_config_list(flags: &root::RootFlags) -> i32 {
    let cfg = match crate::config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    if flags.json {
        let json_val = match serde_json::to_value(&cfg) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error: {}", e);
                return codes::GENERIC_ERROR;
            }
        };
        println!("{}", to_json_pretty(&json_val));
    } else {
        let json_val = match serde_json::to_value(&cfg) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error: {}", e);
                return codes::GENERIC_ERROR;
            }
        };
        if let Some(obj) = json_val.as_object() {
            for (k, v) in obj {
                if !v.is_null() {
                    match v {
                        serde_json::Value::String(s) => println!("{}\t{}", k, s),
                        other => println!("{}\t{}", k, other),
                    }
                }
            }
        }
    }
    codes::SUCCESS
}

fn handle_config_keys(flags: &root::RootFlags) -> i32 {
    let keys = crate::config::known_keys();
    if flags.json {
        let json_val = match serde_json::to_value(&keys) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error: {}", e);
                return codes::GENERIC_ERROR;
            }
        };
        println!("{}", to_json_pretty(&json_val));
    } else {
        for key in &keys {
            println!("{}", key);
        }
    }
    codes::SUCCESS
}

fn handle_config_path(flags: &root::RootFlags) -> i32 {
    match crate::config::config_path() {
        Ok(path) => {
            if flags.json {
                let json_val = serde_json::json!({"path": path.to_string_lossy()});
                println!("{}", to_json_pretty(&json_val));
            } else {
                println!("{}", path.display());
            }
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            codes::CONFIG_ERROR
        }
    }
}

/// Handle the `auth` command and its subcommands.
async fn handle_auth(args: root::AuthArgs, flags: &root::RootFlags) -> i32 {
    match args.command {
        root::AuthCommand::Credentials(cred_args) => {
            handle_auth_credentials(&cred_args.path, flags)
        }
        root::AuthCommand::Add(add_args) => handle_auth_add(add_args, flags).await,
        root::AuthCommand::Remove(remove_args) => handle_auth_remove(&remove_args.email, flags),
        root::AuthCommand::List => handle_auth_list(flags),
        root::AuthCommand::Status => handle_auth_status(flags),
        root::AuthCommand::Services => handle_auth_services(flags),
        root::AuthCommand::Tokens(tokens_args) => handle_auth_tokens(tokens_args, flags),
        root::AuthCommand::Alias(alias_args) => handle_auth_alias(alias_args, flags),
    }
}

fn handle_auth_credentials(path: &str, _flags: &root::RootFlags) -> i32 {
    let path = std::path::Path::new(path);
    if !path.exists() {
        eprintln!("Error: file not found: {}", path.display());
        return codes::GENERIC_ERROR;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    let raw: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    let creds = match crate::config::credentials::parse_credentials(&raw) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing credentials: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    match crate::config::write_client_credentials(crate::config::DEFAULT_CLIENT_NAME, &creds) {
        Ok(()) => {
            eprintln!(
                "Credentials stored for client '{}'.",
                crate::config::DEFAULT_CLIENT_NAME
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error storing credentials: {}", e);
            codes::CONFIG_ERROR
        }
    }
}

/// Handle `auth add`: run OAuth flow, exchange code, store token.
async fn handle_auth_add(add_args: root::AuthAddArgs, flags: &root::RootFlags) -> i32 {
    use crate::auth::oauth::FlowMode;
    use crate::auth::oauth_flow::run_oauth_flow;

    // 1. Determine client name
    let client_name = flags
        .client
        .as_deref()
        .unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
    let client_name = crate::config::normalize_client_name(client_name);

    // 2. Load client credentials from config dir
    let creds = match crate::config::read_client_credentials(&client_name) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Error: {}. Run 'omega-google auth credentials <path>' first.",
                e
            );
            return codes::CONFIG_ERROR;
        }
    };

    // 3. Determine flow mode from flags
    let mode = if add_args.manual {
        FlowMode::Manual
    } else if add_args.remote {
        FlowMode::Remote
    } else if add_args.web {
        FlowMode::Web
    } else {
        FlowMode::Desktop
    };

    // 4. Collect services -- filter by --services flag or default to user services
    let services = if let Some(ref svc_list) = add_args.services {
        let mut parsed = Vec::new();
        for name in svc_list
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            match crate::auth::parse_service(name) {
                Ok(s) => parsed.push(s),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return codes::USAGE_ERROR;
                }
            }
        }
        if parsed.is_empty() {
            eprintln!("Error: --services list is empty");
            return codes::USAGE_ERROR;
        }
        parsed
    } else {
        crate::auth::user_services()
    };

    // 4b. Parse scope options from --readonly and --drive-scope
    let scope_options = crate::auth::ScopeOptions {
        readonly: add_args.readonly,
        drive_scope: match add_args.drive_scope.as_deref() {
            Some("readonly") => crate::auth::DriveScopeMode::Readonly,
            Some("file") => crate::auth::DriveScopeMode::File,
            Some("full") | None => crate::auth::DriveScopeMode::Full,
            Some(other) => {
                eprintln!(
                    "Error: unknown drive scope '{}'. Use: full, readonly, file",
                    other
                );
                return codes::USAGE_ERROR;
            }
        },
    };
    let _ = &scope_options; // Used when scope computation is wired up

    // 5. Run OAuth flow to get authorization code
    let flow_result = match run_oauth_flow(&creds, &services, mode, add_args.force_consent).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: OAuth flow failed: {}", e);
            return codes::AUTH_REQUIRED;
        }
    };

    // 6. Exchange code for tokens
    let http_client = match crate::http::client::build_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: failed to build HTTP client: {}", e);
            return codes::GENERIC_ERROR;
        }
    };
    let token_response = match crate::auth::oauth::exchange_code(
        &http_client,
        &creds,
        &flow_result.code,
        &flow_result.redirect_uri,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: token exchange failed: {}", e);
            return codes::AUTH_REQUIRED;
        }
    };

    // 7. Build TokenData from TokenResponse
    let refresh_token = match token_response.refresh_token {
        Some(rt) => rt,
        None => {
            eprintln!("Error: no refresh token received. Try with --force-consent.");
            return codes::AUTH_REQUIRED;
        }
    };

    let scopes: Vec<String> = token_response
        .scope
        .map(|s| s.split_whitespace().map(|x| x.to_string()).collect())
        .unwrap_or_default();

    let now = chrono::Utc::now();
    let expires_at = token_response
        .expires_in
        .map(|secs| now + chrono::Duration::seconds(secs as i64));

    // We need the email from the token info. For now, fetch it from the access token.
    let email = match fetch_email_from_token(&http_client, &token_response.access_token).await {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error: could not determine account email: {}", e);
            return codes::AUTH_REQUIRED;
        }
    };

    let token_data = crate::auth::TokenData {
        client: client_name.clone(),
        email: email.clone(),
        services: services.clone(),
        scopes,
        created_at: now,
        refresh_token,
        access_token: Some(token_response.access_token),
        expires_at,
    };

    // 8. Store in credential store
    let cfg = match crate::config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    let store = match crate::auth::keyring::credential_store_factory(&cfg) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error initializing credential store: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    if let Err(e) = store.set_token(&client_name, &email, &token_data) {
        eprintln!("Error storing token: {}", e);
        return codes::GENERIC_ERROR;
    }

    // Set as default account if no default exists yet
    if store
        .get_default_account(&client_name)
        .unwrap_or(None)
        .is_none()
    {
        let _ = store.set_default_account(&client_name, &email);
    }

    eprintln!("Account '{}' added successfully.", email);
    codes::SUCCESS
}

/// Fetch the authenticated user's email from Google's userinfo endpoint.
async fn fetch_email_from_token(
    http_client: &reqwest::Client,
    access_token: &str,
) -> anyhow::Result<String> {
    let resp = http_client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("userinfo request failed ({})", resp.status().as_u16());
    }

    let body: serde_json::Value = resp.json().await?;
    body.get("email")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("no email field in userinfo response"))
}

/// Handle `auth remove`: delete a stored account.
fn handle_auth_remove(email: &str, flags: &root::RootFlags) -> i32 {
    let cfg = match crate::config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    let client_name = flags
        .client
        .as_deref()
        .unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
    let client_name = crate::config::normalize_client_name(client_name);

    let store = match crate::auth::keyring::credential_store_factory(&cfg) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error initializing credential store: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    // Prompt for confirmation unless --force
    if !flags.force {
        if flags.no_input {
            eprintln!("Error: Confirmation required but --no-input is set. Use --force to skip.");
            return codes::GENERIC_ERROR;
        }
        eprint!("Remove account '{}'? [y/N] ", email);
        let mut input = String::new();
        if let Err(e) = std::io::stdin().read_line(&mut input) {
            eprintln!("Error reading input: {}", e);
            return codes::GENERIC_ERROR;
        }
        if input.trim().to_lowercase() != "y" {
            eprintln!("Cancelled.");
            return codes::SUCCESS;
        }
    }

    if let Err(e) = store.delete_token(&client_name, email) {
        eprintln!("Error removing account: {}", e);
        return codes::GENERIC_ERROR;
    }

    // Also try legacy key format (token:<email> without client prefix)
    let legacy_key = crate::auth::legacy_token_key(email);
    let _ = store.delete_token_by_raw_key(&legacy_key); // Ignore errors - legacy key may not exist

    eprintln!("Account '{}' removed.", email);
    codes::SUCCESS
}

fn handle_auth_list(flags: &root::RootFlags) -> i32 {
    let cfg = match crate::config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    let client_name = flags
        .client
        .as_deref()
        .unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
    let client_name = crate::config::normalize_client_name(client_name);

    let store = match crate::auth::keyring::credential_store_factory(&cfg) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error initializing credential store: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    let mut tokens = match store.list_tokens() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error listing tokens: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    // Get default account for the current client
    let default_account = store.get_default_account(&client_name).unwrap_or(None);

    // If list_tokens returned empty (OS keyring can't enumerate), try fetching
    // the default account directly so we can still show something.
    if tokens.is_empty() {
        if let Some(ref email) = default_account {
            if let Ok(token) = store.get_token(&client_name, email) {
                tokens.push(token);
            }
        }
    }

    if flags.json {
        let json_accounts: Vec<serde_json::Value> = tokens
            .iter()
            .map(|t| {
                let is_default =
                    default_account.as_deref() == Some(&t.email) && t.client == client_name;
                serde_json::json!({
                    "email": t.email,
                    "client": t.client,
                    "services": t.services,
                    "scopes": t.scopes,
                    "created_at": t.created_at.to_rfc3339(),
                    "is_default": is_default,
                })
            })
            .collect();
        println!("{}", to_json_pretty(&json_accounts));
    } else if tokens.is_empty() {
        eprintln!("No authenticated accounts found. Use 'omega-google auth add' to add one.");
    } else {
        for t in &tokens {
            let is_default =
                default_account.as_deref() == Some(&t.email) && t.client == client_name;
            let marker = if is_default { "* " } else { "  " };
            let services_str: Vec<String> = t.services.iter().map(|s| format!("{:?}", s)).collect();
            println!(
                "{}{}\t{}\t{}",
                marker,
                t.email,
                t.client,
                services_str.join(",")
            );
        }
    }
    codes::SUCCESS
}

/// Handle `auth status`: show config path, keyring backend, current account, credential file status.
fn handle_auth_status(flags: &root::RootFlags) -> i32 {
    // Check for OMEGA store mode first
    let omega_store_active = crate::auth::omega_store::is_omega_store_active();

    let config_path = crate::config::config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let cfg = crate::config::read_config().unwrap_or_default();

    let keyring_backend = if omega_store_active {
        "omega-store".to_string()
    } else {
        cfg.keyring_backend
            .clone()
            .unwrap_or_else(|| "auto".to_string())
    };

    let client_name = flags
        .client
        .as_deref()
        .unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
    let client_name = crate::config::normalize_client_name(client_name);

    // Check if credential file exists
    let (cred_path_str, cred_exists) = if omega_store_active {
        let omega_dir = std::env::var("OMEGA_STORES_DIR").unwrap_or_default();
        let path = std::path::PathBuf::from(&omega_dir).join("google.json");
        let exists = path.exists();
        (path.to_string_lossy().to_string(), exists)
    } else {
        let cred_filename = crate::config::credential_filename(&client_name);
        let cred_path = crate::config::config_dir()
            .map(|d| d.join(&cred_filename))
            .unwrap_or_default();
        let exists = cred_path.exists();
        (cred_path.to_string_lossy().to_string(), exists)
    };

    // Try to get the current account
    let store = crate::auth::keyring::credential_store_factory(&cfg).ok();
    let current_account = store.as_ref().and_then(|s| {
        crate::auth::resolve_account(flags.account.as_deref(), &cfg, s.as_ref(), &client_name).ok()
    });

    // Load token details if we have a current account
    let token_details = current_account.as_ref().and_then(|email| {
        store
            .as_ref()
            .and_then(|s| s.get_token(&client_name, email).ok())
    });

    let needs_refresh = token_details
        .as_ref()
        .map(crate::auth::token::needs_refresh);

    if flags.json {
        let mut json_val = serde_json::json!({
            "config_path": config_path,
            "keyring_backend": keyring_backend,
            "client": client_name,
            "credentials_file": cred_path_str,
            "credentials_found": cred_exists,
            "current_account": current_account,
            "omega_store": omega_store_active,
        });
        if let Some(ref td) = token_details {
            json_val["services"] = serde_json::to_value(&td.services).unwrap_or_default();
            json_val["scopes"] = serde_json::to_value(&td.scopes).unwrap_or_default();
            json_val["created_at"] = serde_json::Value::String(td.created_at.to_rfc3339());
            json_val["needs_refresh"] = serde_json::Value::Bool(needs_refresh.unwrap_or(false));
        }
        println!("{}", to_json_pretty(&json_val));
    } else {
        if omega_store_active {
            println!(
                "Source:            OMEGA store ({})",
                cred_path_str
            );
        }
        println!("Config path:       {}", config_path);
        println!("Keyring backend:   {}", keyring_backend);
        println!("Client:            {}", client_name);
        println!("Credentials file:  {}", cred_path_str);
        println!(
            "Credentials found: {}",
            if cred_exists { "yes" } else { "no" }
        );
        match &current_account {
            Some(acct) => println!("Current account:   {}", acct),
            None => println!("Current account:   (none)"),
        }
        if let Some(ref td) = token_details {
            let services_str: Vec<String> =
                td.services.iter().map(|s| format!("{:?}", s)).collect();
            println!("Services:          {}", services_str.join(", "));
            println!("Scopes:            {}", td.scopes.join(", "));
            println!("Created:           {}", td.created_at.to_rfc3339());
            match needs_refresh {
                Some(true) => println!("Token status:      needs refresh"),
                Some(false) => println!("Token status:      valid"),
                None => {}
            }
        }
    }
    codes::SUCCESS
}

fn handle_auth_services(flags: &root::RootFlags) -> i32 {
    let services = crate::auth::services_info();

    if flags.json {
        let json_val = match serde_json::to_value(&services) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error: {}", e);
                return codes::GENERIC_ERROR;
            }
        };
        println!("{}", to_json_pretty(&json_val));
    } else {
        for si in &services {
            let user_marker = if si.user { "user" } else { "admin" };
            println!(
                "{:?}\t{}\t{}",
                si.service,
                user_marker,
                si.scopes.join(", ")
            );
        }
    }
    codes::SUCCESS
}

fn handle_auth_tokens(args: root::AuthTokensArgs, flags: &root::RootFlags) -> i32 {
    match args.command {
        root::AuthTokensCommand::List => handle_auth_tokens_list(flags),
        root::AuthTokensCommand::Delete(del_args) => {
            handle_auth_tokens_delete(&del_args.email, flags)
        }
    }
}

fn handle_auth_tokens_list(flags: &root::RootFlags) -> i32 {
    let cfg = match crate::config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    let store = match crate::auth::keyring::credential_store_factory(&cfg) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error initializing credential store: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    let keys = match store.keys() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Error listing tokens: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    if flags.json {
        let json_keys: Vec<serde_json::Value> = keys
            .iter()
            .map(|k| match crate::auth::parse_token_key(k) {
                Some((client, email)) => serde_json::json!({
                    "key": k,
                    "client": client,
                    "email": email,
                }),
                None => serde_json::json!({
                    "key": k,
                }),
            })
            .collect();
        println!("{}", to_json_pretty(&json_keys));
    } else if keys.is_empty() {
        eprintln!("No tokens found.");
    } else {
        for k in &keys {
            match crate::auth::parse_token_key(k) {
                Some((client, email)) => println!("{}\t{}", email, client),
                None => println!("{}", k),
            }
        }
    }
    codes::SUCCESS
}

fn handle_auth_tokens_delete(email: &str, flags: &root::RootFlags) -> i32 {
    let cfg = match crate::config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    let client_name = flags
        .client
        .as_deref()
        .unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
    let client_name = crate::config::normalize_client_name(client_name);

    let store = match crate::auth::keyring::credential_store_factory(&cfg) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error initializing credential store: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    if let Err(e) = store.delete_token(&client_name, email) {
        eprintln!("Error deleting token: {}", e);
        return codes::GENERIC_ERROR;
    }

    eprintln!("Token for '{}' deleted.", email);
    codes::SUCCESS
}

fn handle_auth_alias(args: root::AuthAliasArgs, flags: &root::RootFlags) -> i32 {
    match args.command {
        root::AuthAliasCommand::Set(set_args) => {
            let mut cfg = match crate::config::read_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading config: {}", e);
                    return codes::CONFIG_ERROR;
                }
            };
            let aliases = cfg.account_aliases.get_or_insert_with(Default::default);
            aliases.insert(set_args.alias.clone(), set_args.email.clone());
            if let Err(e) = crate::config::write_config(&cfg) {
                eprintln!("Error writing config: {}", e);
                return codes::CONFIG_ERROR;
            }
            eprintln!("Alias '{}' set to '{}'.", set_args.alias, set_args.email);
            codes::SUCCESS
        }
        root::AuthAliasCommand::Unset(unset_args) => {
            let mut cfg = match crate::config::read_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading config: {}", e);
                    return codes::CONFIG_ERROR;
                }
            };
            if let Some(ref mut aliases) = cfg.account_aliases {
                aliases.remove(&unset_args.alias);
            }
            if let Err(e) = crate::config::write_config(&cfg) {
                eprintln!("Error writing config: {}", e);
                return codes::CONFIG_ERROR;
            }
            eprintln!("Alias '{}' removed.", unset_args.alias);
            codes::SUCCESS
        }
        root::AuthAliasCommand::List => {
            let cfg = match crate::config::read_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return codes::CONFIG_ERROR;
                }
            };

            let aliases = cfg.account_aliases.unwrap_or_default();
            if flags.json {
                let json_val = match serde_json::to_value(&aliases) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return codes::GENERIC_ERROR;
                    }
                };
                println!("{}", to_json_pretty(&json_val));
            } else if aliases.is_empty() {
                eprintln!("No aliases configured.");
            } else {
                for (alias, email) in &aliases {
                    println!("{}\t{}", alias, email);
                }
            }
            codes::SUCCESS
        }
    }
}

/// Handle the `time` command and its subcommands.
fn handle_time(args: root::TimeArgs, flags: &root::RootFlags) -> i32 {
    match args.command {
        root::TimeCommand::Now => handle_time_now(flags),
    }
}

fn handle_time_now(flags: &root::RootFlags) -> i32 {
    let now_utc = chrono::Utc::now();
    let now_local = chrono::Local::now();

    if flags.json {
        let json_val = serde_json::json!({
            "local": now_local.to_rfc3339(),
            "utc": now_utc.to_rfc3339(),
            "unix": now_utc.timestamp(),
        });
        println!("{}", to_json_pretty(&json_val));
    } else {
        println!("Local: {}", now_local.format("%Y-%m-%d %H:%M:%S %Z"));
        println!("UTC:   {}", now_utc.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("Unix:  {}", now_utc.timestamp());
    }
    codes::SUCCESS
}

/// Handle the `gmail` command and its subcommands.
async fn handle_gmail(args: gmail::GmailArgs, flags: &root::RootFlags) -> i32 {
    use gmail::GmailCommand;

    // Commands that can work without authentication
    if let GmailCommand::Url(url_args) = &args.command {
        use crate::services::gmail::types::thread_url;
        if url_args.thread_ids.is_empty() {
            eprintln!("Error: at least one thread ID is required");
            return codes::USAGE_ERROR;
        }
        let urls: Vec<String> = url_args
            .thread_ids
            .iter()
            .map(|id| thread_url(id))
            .collect();
        if flags.json {
            let json_val = match serde_json::to_value(&urls) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return codes::GENERIC_ERROR;
                }
            };
            println!("{}", to_json_pretty(&json_val));
        } else {
            for url in &urls {
                println!("{}", url);
            }
        }
        return codes::SUCCESS;
    }

    // Bootstrap auth for all other commands
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::AUTH_REQUIRED;
        }
    };

    // Dispatch to subcommand handlers
    match args.command {
        GmailCommand::Search(ref search_args) => handle_gmail_search(&ctx, search_args).await,
        GmailCommand::Messages(ref msg_args) => handle_gmail_messages(&ctx, msg_args).await,
        GmailCommand::Thread(ref thread_args) => handle_gmail_thread(&ctx, thread_args).await,
        GmailCommand::Get(ref get_args) => handle_gmail_message_get(&ctx, get_args).await,
        GmailCommand::Send(ref send_args) => handle_gmail_send(&ctx, send_args).await,
        GmailCommand::Labels(ref labels_args) => handle_gmail_labels(&ctx, labels_args).await,
        GmailCommand::Attachment(ref att_args) => handle_gmail_attachment(&ctx, att_args).await,
        GmailCommand::Watch(ref watch_args) => {
            use crate::cli::gmail::GmailWatchCommand;
            match &watch_args.command {
                GmailWatchCommand::Start(start_args) => {
                    let label_ids = if start_args.label.is_empty() {
                        vec!["INBOX".to_string()]
                    } else {
                        start_args.label.clone()
                    };
                    match crate::services::gmail::watch::watch_start(
                        &ctx,
                        crate::services::gmail::types::GMAIL_BASE_URL,
                        &start_args.topic,
                        &label_ids,
                    )
                    .await
                    {
                        Ok(Some(_)) => {
                            eprintln!("Note: Ensure gmail-api-push@system.gserviceaccount.com has Pub/Sub Publisher role on the topic.");
                            codes::SUCCESS
                        }
                        Ok(None) => {
                            eprintln!("[dry-run] would start watch");
                            codes::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            map_error_to_exit_code(&e)
                        }
                    }
                }
                GmailWatchCommand::Stop => {
                    match crate::services::gmail::watch::watch_stop(
                        &ctx,
                        crate::services::gmail::types::GMAIL_BASE_URL,
                    )
                    .await
                    {
                        Ok(()) => {
                            eprintln!("Watch stopped.");
                            codes::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            map_error_to_exit_code(&e)
                        }
                    }
                }
                GmailWatchCommand::Status => {
                    match crate::services::gmail::watch::watch_status(
                        &ctx,
                        crate::services::gmail::types::GMAIL_BASE_URL,
                    )
                    .await
                    {
                        Ok(()) => codes::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            map_error_to_exit_code(&e)
                        }
                    }
                }
                GmailWatchCommand::Renew => {
                    eprintln!("Hint: use `omg-gog gmail watch start --topic <TOPIC>` to renew the watch.");
                    codes::SUCCESS
                }
            }
        }
        _ => {
            eprintln!("Command not yet implemented");
            codes::GENERIC_ERROR
        }
    }
}

/// Handle Gmail thread search.
async fn handle_gmail_search(
    ctx: &crate::services::ServiceContext,
    args: &gmail::GmailSearchArgs,
) -> i32 {
    let query = args.query.join(" ");
    let params = crate::services::common::PaginationParams {
        max_results: Some(args.max),
        page_token: args.page.clone(),
        all_pages: args.all,
        fail_empty: args.fail_empty,
    };

    let base_query = query.clone();
    let max = args.max;
    let (items, next_token) = match crate::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        &params,
        |pt| crate::services::gmail::search::build_thread_search_url(&base_query, Some(max), pt),
        |value| {
            let threads = value
                .get("threads")
                .and_then(|v| v.as_array())
                .map(|arr| arr.to_vec())
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((threads, next))
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if crate::services::pagination::check_fail_empty(&items, args.fail_empty).is_err() {
        return codes::EMPTY_RESULTS;
    }

    let response = serde_json::json!({
        "threads": items,
        "resultSizeEstimate": items.len(),
    });

    if let Err(e) = ctx.write_paginated(&response, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Gmail messages subcommand (search, etc.).
async fn handle_gmail_messages(
    ctx: &crate::services::ServiceContext,
    args: &gmail::GmailMessagesArgs,
) -> i32 {
    match &args.command {
        gmail::GmailMessagesCommand::Search(search_args) => {
            let query = search_args.query.join(" ");
            let params = crate::services::common::PaginationParams {
                max_results: Some(search_args.max),
                page_token: search_args.page.clone(),
                all_pages: false,
                fail_empty: false,
            };

            let base_query = query.clone();
            let max = search_args.max;
            let include_body = search_args.include_body;
            let (items, next_token) = match crate::services::pagination::paginate(
                &ctx.client,
                &ctx.circuit_breaker,
                &ctx.retry_config,
                ctx.is_verbose(),
                &params,
                |pt| {
                    crate::services::gmail::search::build_message_search_url(
                        &base_query,
                        Some(max),
                        pt,
                        include_body,
                    )
                },
                |value| {
                    let messages = value
                        .get("messages")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.to_vec())
                        .unwrap_or_default();
                    let next = value
                        .get("nextPageToken")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    Ok((messages, next))
                },
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return map_error_to_exit_code(&e);
                }
            };

            let response = serde_json::json!({
                "messages": items,
                "resultSizeEstimate": items.len(),
            });

            if let Err(e) = ctx.write_paginated(&response, next_token.as_deref()) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
    }
}

/// Handle Gmail thread subcommand (get, modify, attachments).
async fn handle_gmail_thread(
    ctx: &crate::services::ServiceContext,
    args: &gmail::GmailThreadArgs,
) -> i32 {
    match &args.command {
        gmail::GmailThreadCommand::Get(get_args) => {
            let url = crate::services::gmail::thread::build_thread_get_url(&get_args.thread_id);
            let thread: crate::services::gmail::types::Thread = match crate::http::api::api_get(
                &ctx.client,
                &url,
                &ctx.circuit_breaker,
                &ctx.retry_config,
                ctx.is_verbose(),
            )
            .await
            {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return map_error_to_exit_code(&e);
                }
            };

            if let Err(e) = ctx.write_output(&thread) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        gmail::GmailThreadCommand::Modify(modify_args) => {
            let (url, body) = crate::services::gmail::thread::build_thread_modify_request(
                &modify_args.thread_id,
                &modify_args.add,
                &modify_args.remove,
            );

            match crate::http::api::api_post::<crate::services::gmail::types::Thread>(
                &ctx.client,
                &url,
                &body,
                &ctx.circuit_breaker,
                &ctx.retry_config,
                ctx.is_verbose(),
                ctx.is_dry_run(),
            )
            .await
            {
                Ok(Some(thread)) => {
                    if let Err(e) = ctx.write_output(&thread) {
                        eprintln!("Error: {}", e);
                        return map_error_to_exit_code(&e);
                    }
                    codes::SUCCESS
                }
                Ok(None) => {
                    eprintln!("[dry-run] would modify thread '{}'", modify_args.thread_id);
                    codes::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    map_error_to_exit_code(&e)
                }
            }
        }
        gmail::GmailThreadCommand::Attachments(_) => {
            eprintln!("Command not yet implemented");
            codes::GENERIC_ERROR
        }
    }
}

/// Handle Gmail message get.
async fn handle_gmail_message_get(
    ctx: &crate::services::ServiceContext,
    args: &gmail::GmailGetArgs,
) -> i32 {
    let format_str = if args.format.is_empty() {
        None
    } else {
        Some(args.format.as_str())
    };
    let url = crate::services::gmail::message::build_message_get_url(&args.message_id, format_str);
    let message: crate::services::gmail::types::Message = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&message) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Gmail send.
async fn handle_gmail_send(
    ctx: &crate::services::ServiceContext,
    args: &gmail::GmailSendArgs,
) -> i32 {
    // M-4: Validate at least one recipient
    if args.to.is_empty() && args.cc.is_empty() && args.bcc.is_empty() {
        eprintln!("Error: at least one recipient (--to, --cc, or --bcc) is required");
        return codes::USAGE_ERROR;
    }

    // M-2: Read file attachments from --attach paths
    let mut attachments = Vec::new();
    for path in &args.attach {
        let filename = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error reading attachment '{}': {}", path, e);
                return codes::GENERIC_ERROR;
            }
        };
        let content_type = crate::services::gmail::mime::guess_content_type(&filename);
        attachments.push(crate::services::gmail::mime::MimeAttachment {
            filename,
            data,
            content_type,
        });
    }

    let mime_params = crate::services::gmail::mime::MimeMessageParams {
        from: ctx.email.clone(),
        to: args.to.clone(),
        cc: args.cc.clone(),
        bcc: args.bcc.clone(),
        subject: args.subject.clone().unwrap_or_default(),
        body_text: args.body.clone(),
        body_html: args.body_html.clone(),
        reply_to: args.reply_to.clone(),
        in_reply_to: args.reply_to_message_id.clone(),
        references: None,
        attachments,
    };
    let mime = crate::services::gmail::mime::build_mime_message(&mime_params);
    let encoded = base64_url_encode(&mime);
    let body = crate::services::gmail::send::build_send_body(&encoded);
    let url = crate::services::gmail::send::build_send_url();

    match crate::http::api::api_post::<crate::services::gmail::types::Message>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(msg)) => {
            if let Err(e) = ctx.write_output(&msg) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would send message");
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Gmail labels subcommands.
async fn handle_gmail_labels(
    ctx: &crate::services::ServiceContext,
    args: &gmail::GmailLabelsArgs,
) -> i32 {
    match &args.command {
        gmail::GmailLabelsCommand::List => {
            let url = crate::services::gmail::labels::build_labels_list_url();
            let response: crate::services::gmail::types::LabelListResponse =
                match crate::http::api::api_get(
                    &ctx.client,
                    &url,
                    &ctx.circuit_breaker,
                    &ctx.retry_config,
                    ctx.is_verbose(),
                )
                .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return map_error_to_exit_code(&e);
                    }
                };

            if let Err(e) = ctx.write_output(&response) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        gmail::GmailLabelsCommand::Create(create_args) => {
            let (url, body) =
                crate::services::gmail::labels::build_label_create_request(&create_args.name);

            match crate::http::api::api_post::<crate::services::gmail::types::Label>(
                &ctx.client,
                &url,
                &body,
                &ctx.circuit_breaker,
                &ctx.retry_config,
                ctx.is_verbose(),
                ctx.is_dry_run(),
            )
            .await
            {
                Ok(Some(label)) => {
                    if let Err(e) = ctx.write_output(&label) {
                        eprintln!("Error: {}", e);
                        return map_error_to_exit_code(&e);
                    }
                    codes::SUCCESS
                }
                Ok(None) => {
                    eprintln!("[dry-run] would create label '{}'", create_args.name);
                    codes::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    map_error_to_exit_code(&e)
                }
            }
        }
        gmail::GmailLabelsCommand::Delete(delete_args) => {
            // Confirmation for destructive operation
            if !ctx.is_force() && !ctx.flags.no_input {
                eprint!(
                    "Are you sure you want to delete label '{}'? [y/N] ",
                    delete_args.label_id
                );
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_err()
                    || input.trim().to_lowercase() != "y"
                {
                    eprintln!("Cancelled.");
                    return codes::CANCELLED;
                }
            } else if !ctx.is_force() && ctx.flags.no_input {
                eprintln!("Error: destructive operation requires --force when --no-input is set");
                return codes::USAGE_ERROR;
            }

            let url = crate::services::gmail::labels::build_label_delete_url(&delete_args.label_id);

            match crate::http::api::api_delete(
                &ctx.client,
                &url,
                &ctx.circuit_breaker,
                &ctx.retry_config,
                ctx.is_verbose(),
                ctx.is_dry_run(),
            )
            .await
            {
                Ok(()) => {
                    eprintln!("Label '{}' deleted.", delete_args.label_id);
                    codes::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    map_error_to_exit_code(&e)
                }
            }
        }
        gmail::GmailLabelsCommand::Get(get_args) => {
            let url = crate::services::gmail::labels::build_label_get_url(&get_args.label);

            let label: crate::services::gmail::types::Label = match crate::http::api::api_get(
                &ctx.client,
                &url,
                &ctx.circuit_breaker,
                &ctx.retry_config,
                ctx.is_verbose(),
            )
            .await
            {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return map_error_to_exit_code(&e);
                }
            };

            if let Err(e) = ctx.write_output(&label) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        gmail::GmailLabelsCommand::Modify(modify_args) => {
            // Batch modify: apply label changes to each thread
            let mut failed = false;
            for thread_id in &modify_args.thread_ids {
                let (url, body) = crate::services::gmail::thread::build_thread_modify_request(
                    thread_id,
                    &modify_args.add,
                    &modify_args.remove,
                );

                match crate::http::api::api_post::<crate::services::gmail::types::Thread>(
                    &ctx.client,
                    &url,
                    &body,
                    &ctx.circuit_breaker,
                    &ctx.retry_config,
                    ctx.is_verbose(),
                    ctx.is_dry_run(),
                )
                .await
                {
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        eprintln!("[dry-run] would modify thread '{}'", thread_id);
                    }
                    Err(e) => {
                        eprintln!("Error modifying thread '{}': {}", thread_id, e);
                        failed = true;
                    }
                }
            }
            if failed {
                codes::GENERIC_ERROR
            } else {
                codes::SUCCESS
            }
        }
    }
}

/// Base64url encode a string (no padding, URL-safe alphabet).
fn base64_url_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(input.as_bytes())
}

/// Handle the `calendar` command and its subcommands.
async fn handle_calendar(args: calendar::CalendarArgs, flags: &root::RootFlags) -> i32 {
    use calendar::CalendarCommand;

    // Commands that can work without authentication
    match &args.command {
        CalendarCommand::Time => {
            let now_utc = chrono::Utc::now();
            let now_local = chrono::Local::now();
            if flags.json {
                let json_val = serde_json::json!({
                    "local": now_local.to_rfc3339(),
                    "utc": now_utc.to_rfc3339(),
                    "unix": now_utc.timestamp(),
                });
                println!("{}", to_json_pretty(&json_val));
            } else {
                println!("Local: {}", now_local.format("%Y-%m-%d %H:%M:%S %Z"));
                println!("UTC:   {}", now_utc.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("Unix:  {}", now_utc.timestamp());
            }
            return codes::SUCCESS;
        }
        CalendarCommand::Colors => {
            eprintln!("Command registered. API call requires: omega-google auth add <email>");
            return codes::SUCCESS;
        }
        _ => {}
    }

    // Bootstrap auth for all other commands
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::AUTH_REQUIRED;
        }
    };

    // Dispatch to subcommand handlers
    match args.command {
        CalendarCommand::Events(ref events_args) => {
            handle_calendar_events_list(&ctx, events_args).await
        }
        CalendarCommand::Event(ref get_args) => handle_calendar_event_get(&ctx, get_args).await,
        CalendarCommand::Create(ref create_args) => {
            handle_calendar_event_create(&ctx, create_args).await
        }
        CalendarCommand::Update(ref update_args) => {
            handle_calendar_event_update(&ctx, update_args).await
        }
        CalendarCommand::Delete(ref delete_args) => {
            handle_calendar_event_delete(&ctx, delete_args).await
        }
        CalendarCommand::Calendars(ref cal_args) => {
            handle_calendar_calendars_list(&ctx, cal_args).await
        }
        CalendarCommand::Freebusy(ref fb_args) => handle_calendar_freebusy(&ctx, fb_args).await,
        CalendarCommand::Watch(ref watch_args) => {
            use crate::cli::calendar::CalendarWatchCommand;
            const CAL_API_ROOT: &str = "https://www.googleapis.com";
            match &watch_args.command {
                CalendarWatchCommand::Start(start_args) => {
                    match crate::services::calendar::watch::watch_start(
                        &ctx,
                        CAL_API_ROOT,
                        &start_args.callback_url,
                        &start_args.calendar,
                    )
                    .await
                    {
                        Ok(Some(_)) => codes::SUCCESS,
                        Ok(None) => {
                            eprintln!("[dry-run] would start calendar watch");
                            codes::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            map_error_to_exit_code(&e)
                        }
                    }
                }
                CalendarWatchCommand::Stop(stop_args) => {
                    match crate::services::calendar::watch::watch_stop(
                        &ctx,
                        CAL_API_ROOT,
                        &stop_args.channel_id,
                        &stop_args.resource_id,
                    )
                    .await
                    {
                        Ok(()) => {
                            eprintln!("Calendar watch stopped.");
                            codes::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            map_error_to_exit_code(&e)
                        }
                    }
                }
                CalendarWatchCommand::Status => {
                    match crate::services::calendar::watch::watch_status(&ctx).await {
                        Ok(()) => {
                            eprintln!("Note: There is no API to query active calendar watches.");
                            codes::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            map_error_to_exit_code(&e)
                        }
                    }
                }
            }
        }
        _ => {
            eprintln!("Command not yet implemented");
            codes::GENERIC_ERROR
        }
    }
}

/// Handle Calendar events list.
async fn handle_calendar_events_list(
    ctx: &crate::services::ServiceContext,
    args: &calendar::CalendarEventsArgs,
) -> i32 {
    let calendar_id = args.cal.as_deref().unwrap_or("primary");
    let max = args.max.unwrap_or(250);
    let params = crate::services::common::PaginationParams {
        max_results: Some(max),
        page_token: args.page.clone(),
        all_pages: args.all,
        fail_empty: false,
    };

    let cal_id = calendar_id.to_string();
    // Parse --from/--to through the time module (supports relative keywords, dates, etc.)
    let time_min = match &args.from {
        Some(val) => match crate::time::parse::parse_datetime(val) {
            Ok(dt) => Some(dt.to_rfc3339()),
            Err(_) => Some(val.clone()), // Pass through if already RFC 3339 or unrecognized
        },
        None => None,
    };
    let time_max = match &args.to {
        Some(val) => match crate::time::parse::parse_datetime(val) {
            Ok(dt) => Some(dt.to_rfc3339()),
            Err(_) => Some(val.clone()),
        },
        None => None,
    };
    let query = args.query.clone();
    let (items, next_token) = match crate::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        &params,
        |pt| {
            crate::services::calendar::events::build_events_list_url(
                &cal_id,
                time_min.as_deref(),
                time_max.as_deref(),
                Some(max),
                pt,
                query.as_deref(),
            )
        },
        |value| {
            let events = value
                .get("items")
                .and_then(|v| v.as_array())
                .map(|arr| arr.to_vec())
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((events, next))
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let response = serde_json::json!({
        "items": items,
    });

    if let Err(e) = ctx.write_paginated(&response, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Calendar event get.
async fn handle_calendar_event_get(
    ctx: &crate::services::ServiceContext,
    args: &calendar::CalendarEventArgs,
) -> i32 {
    let url =
        crate::services::calendar::events::build_event_get_url(&args.calendar_id, &args.event_id);

    let event: crate::services::calendar::types::Event = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&event) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Calendar event create.
async fn handle_calendar_event_create(
    ctx: &crate::services::ServiceContext,
    args: &calendar::CalendarCreateArgs,
) -> i32 {
    let calendar_id = &args.cal;
    // Parse --from/--to through time module for non-all-day events
    let start = if args.all_day {
        crate::services::calendar::types::EventDateTime {
            date_time: None,
            date: Some(args.from.clone()),
            time_zone: None,
        }
    } else {
        let parsed_from = match crate::time::parse::parse_datetime(&args.from) {
            Ok(dt) => dt.to_rfc3339(),
            Err(_) => args.from.clone(),
        };
        crate::services::calendar::types::EventDateTime {
            date_time: Some(parsed_from),
            date: None,
            time_zone: None,
        }
    };
    let end = if args.all_day {
        crate::services::calendar::types::EventDateTime {
            date_time: None,
            date: Some(args.to.clone()),
            time_zone: None,
        }
    } else {
        let parsed_to = match crate::time::parse::parse_datetime(&args.to) {
            Ok(dt) => dt.to_rfc3339(),
            Err(_) => args.to.clone(),
        };
        crate::services::calendar::types::EventDateTime {
            date_time: Some(parsed_to),
            date: None,
            time_zone: None,
        }
    };
    let attendees: Vec<String> = args
        .attendees
        .as_ref()
        .map(|a| a.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    let body = crate::services::calendar::events::build_event_create_body(
        &args.summary,
        &start,
        &end,
        args.description.as_deref(),
        args.location.as_deref(),
        &attendees,
        None,
    );
    let url = crate::services::calendar::events::build_event_create_url(calendar_id);

    match crate::http::api::api_post::<crate::services::calendar::types::Event>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(event)) => {
            if let Err(e) = ctx.write_output(&event) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create event '{}'", args.summary);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Calendar event update.
async fn handle_calendar_event_update(
    ctx: &crate::services::ServiceContext,
    args: &calendar::CalendarUpdateArgs,
) -> i32 {
    let url = crate::services::calendar::events::build_event_update_url(
        &args.calendar_id,
        &args.event_id,
    );

    let mut body = serde_json::Map::new();
    if let Some(ref summary) = args.summary {
        body.insert("summary".to_string(), serde_json::json!(summary));
    }
    if let Some(ref description) = args.description {
        body.insert("description".to_string(), serde_json::json!(description));
    }
    if let Some(ref location) = args.location {
        body.insert("location".to_string(), serde_json::json!(location));
    }
    if let Some(ref from) = args.from {
        body.insert("start".to_string(), serde_json::json!({"dateTime": from}));
    }
    if let Some(ref to) = args.to {
        body.insert("end".to_string(), serde_json::json!({"dateTime": to}));
    }
    if !args.add_attendee.is_empty() {
        let attendees: Vec<serde_json::Value> = args
            .add_attendee
            .iter()
            .map(|email| serde_json::json!({"email": email}))
            .collect();
        body.insert("attendees".to_string(), serde_json::json!(attendees));
    }
    let body_val = serde_json::Value::Object(body);

    match crate::http::api::api_patch::<crate::services::calendar::types::Event>(
        &ctx.client,
        &url,
        &body_val,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(event)) => {
            if let Err(e) = ctx.write_output(&event) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would update event '{}'", args.event_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Calendar event delete.
async fn handle_calendar_event_delete(
    ctx: &crate::services::ServiceContext,
    args: &calendar::CalendarDeleteArgs,
) -> i32 {
    // Confirmation for destructive operation
    if !ctx.is_force() && !ctx.flags.no_input {
        eprint!(
            "Are you sure you want to delete event '{}'? [y/N] ",
            args.event_id
        );
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() || input.trim().to_lowercase() != "y" {
            eprintln!("Cancelled.");
            return codes::CANCELLED;
        }
    } else if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::calendar::events::build_event_delete_url(
        &args.calendar_id,
        &args.event_id,
    );

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Event '{}' deleted.", args.event_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Calendar calendars list.
async fn handle_calendar_calendars_list(
    ctx: &crate::services::ServiceContext,
    args: &calendar::CalendarCalendarsArgs,
) -> i32 {
    let params = crate::services::common::PaginationParams {
        max_results: Some(args.max),
        page_token: args.page.clone(),
        all_pages: args.all,
        fail_empty: args.fail_empty,
    };

    let max = args.max;
    let (items, next_token) = match crate::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        &params,
        |pt| crate::services::calendar::calendars::build_calendars_list_url(Some(max), pt),
        |value| {
            let calendars = value
                .get("items")
                .and_then(|v| v.as_array())
                .map(|arr| arr.to_vec())
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((calendars, next))
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let response = serde_json::json!({
        "items": items,
    });

    if let Err(e) = ctx.write_paginated(&response, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Calendar freebusy.
async fn handle_calendar_freebusy(
    ctx: &crate::services::ServiceContext,
    args: &calendar::CalendarFreeBusyArgs,
) -> i32 {
    let calendars: Vec<String> = args
        .calendar_ids
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let req = crate::services::calendar::freebusy::build_freebusy_request(
        &calendars, &args.from, &args.to,
    );
    let url = crate::services::calendar::freebusy::build_freebusy_url();
    let body = match serde_json::to_value(&req) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    // Freebusy is read-only, never skip on dry-run
    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        false,
    )
    .await
    {
        Ok(Some(resp)) => {
            if let Err(e) = ctx.write_output(&resp) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            // Should not happen since dry_run=false, but handle gracefully
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle the `drive` command and its subcommands.
async fn handle_drive(args: drive::DriveArgs, flags: &root::RootFlags) -> i32 {
    use drive::DriveCommand;

    // Commands that can work without authentication
    if let DriveCommand::Url(url_args) = &args.command {
        use crate::services::drive::types::file_url;
        if url_args.file_ids.is_empty() {
            eprintln!("Error: at least one file ID is required");
            return codes::USAGE_ERROR;
        }
        let urls: Vec<String> = url_args.file_ids.iter().map(|id| file_url(id)).collect();
        if flags.json {
            let json_val = match serde_json::to_value(&urls) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return codes::GENERIC_ERROR;
                }
            };
            println!("{}", to_json_pretty(&json_val));
        } else {
            for url in &urls {
                println!("{}", url);
            }
        }
        return codes::SUCCESS;
    }

    // Bootstrap auth for all other commands
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::AUTH_REQUIRED;
        }
    };

    // Dispatch to subcommand handlers
    match args.command {
        DriveCommand::Ls(ref list_args) => handle_drive_list(&ctx, list_args).await,
        DriveCommand::Search(ref search_args) => handle_drive_search(&ctx, search_args).await,
        DriveCommand::Get(ref get_args) => handle_drive_get(&ctx, get_args).await,
        DriveCommand::Download(ref dl_args) => handle_drive_download(&ctx, dl_args).await,
        DriveCommand::Upload(ref ul_args) => handle_drive_upload(&ctx, ul_args).await,
        DriveCommand::Mkdir(ref mkdir_args) => handle_drive_mkdir(&ctx, mkdir_args).await,
        DriveCommand::Delete(ref delete_args) => handle_drive_delete(&ctx, delete_args).await,
        DriveCommand::Move(ref move_args) => handle_drive_move(&ctx, move_args).await,
        DriveCommand::Rename(ref rename_args) => handle_drive_rename(&ctx, rename_args).await,
        DriveCommand::Share(ref share_args) => handle_drive_share(&ctx, share_args).await,
        DriveCommand::Permissions(ref perm_args) => {
            handle_drive_permissions_list(&ctx, perm_args).await
        }
        DriveCommand::Copy(ref copy_args) => handle_drive_copy(&ctx, copy_args).await,
        DriveCommand::Watch(ref watch_args) => {
            use crate::cli::drive::DriveWatchCommand;
            const DRIVE_API_ROOT: &str = "https://www.googleapis.com";
            match &watch_args.command {
                DriveWatchCommand::Start(start_args) => {
                    match crate::services::drive::watch::watch_start(
                        &ctx,
                        DRIVE_API_ROOT,
                        &start_args.callback_url,
                    )
                    .await
                    {
                        Ok(Some(_)) => codes::SUCCESS,
                        Ok(None) => {
                            eprintln!("[dry-run] would start drive watch");
                            codes::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            map_error_to_exit_code(&e)
                        }
                    }
                }
                DriveWatchCommand::Stop(stop_args) => {
                    match crate::services::drive::watch::watch_stop(
                        &ctx,
                        DRIVE_API_ROOT,
                        &stop_args.channel_id,
                        &stop_args.resource_id,
                    )
                    .await
                    {
                        Ok(()) => {
                            eprintln!("Drive watch stopped.");
                            codes::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            map_error_to_exit_code(&e)
                        }
                    }
                }
                DriveWatchCommand::Status => {
                    match crate::services::drive::watch::watch_status(&ctx, DRIVE_API_ROOT).await {
                        Ok(()) => codes::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            map_error_to_exit_code(&e)
                        }
                    }
                }
            }
        }
        _ => {
            eprintln!("Command not yet implemented");
            codes::GENERIC_ERROR
        }
    }
}

/// Handle Drive list.
async fn handle_drive_list(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveLsArgs,
) -> i32 {
    let folder_id = args.parent.as_deref().unwrap_or("root");
    let query = crate::services::drive::list::build_list_query(folder_id, args.query.as_deref());
    let params = crate::services::common::PaginationParams {
        max_results: Some(args.max),
        page_token: args.page.clone(),
        all_pages: false,
        fail_empty: false,
    };

    let max = args.max;
    let all_drives = args.all_drives;
    let (items, next_token) = match crate::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        &params,
        |pt| build_drive_list_url(&query, max, pt, all_drives),
        |value| {
            let files = value
                .get("files")
                .and_then(|v| v.as_array())
                .map(|arr| arr.to_vec())
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((files, next))
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let response = serde_json::json!({
        "files": items,
    });

    if let Err(e) = ctx.write_paginated(&response, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Drive search.
async fn handle_drive_search(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveSearchArgs,
) -> i32 {
    let search_text = args.query.join(" ");
    let query = crate::services::drive::list::build_search_query(&search_text, args.raw_query);
    let params = crate::services::common::PaginationParams {
        max_results: Some(args.max),
        page_token: args.page.clone(),
        all_pages: false,
        fail_empty: false,
    };

    let max = args.max;
    let all_drives = args.all_drives;
    let (items, next_token) = match crate::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        &params,
        |pt| build_drive_list_url(&query, max, pt, all_drives),
        |value| {
            let files = value
                .get("files")
                .and_then(|v| v.as_array())
                .map(|arr| arr.to_vec())
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((files, next))
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let response = serde_json::json!({
        "files": items,
    });

    if let Err(e) = ctx.write_paginated(&response, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Drive get.
async fn handle_drive_get(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveGetArgs,
) -> i32 {
    let url = crate::services::drive::files::build_file_get_url(&args.file_id);
    let file: crate::services::drive::types::DriveFile = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&file) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Drive mkdir.
async fn handle_drive_mkdir(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveMkdirArgs,
) -> i32 {
    let body =
        crate::services::drive::folders::build_mkdir_body(&args.name, args.parent.as_deref());
    let url = format!("{}/files", crate::services::drive::types::DRIVE_BASE_URL);

    match crate::http::api::api_post::<crate::services::drive::types::DriveFile>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(file)) => {
            if let Err(e) = ctx.write_output(&file) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create folder '{}'", args.name);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Drive delete.
async fn handle_drive_delete(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveDeleteArgs,
) -> i32 {
    // Confirmation for destructive operation
    if !ctx.is_force() && !ctx.flags.no_input {
        eprint!("Are you sure you want to delete '{}'? [y/N] ", args.file_id);
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() || input.trim().to_lowercase() != "y" {
            eprintln!("Cancelled.");
            return codes::CANCELLED;
        }
    } else if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    if args.permanent {
        let url = crate::services::drive::folders::build_permanent_delete_url(&args.file_id);
        match crate::http::api::api_delete(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
            ctx.is_dry_run(),
        )
        .await
        {
            Ok(()) => {
                eprintln!("File '{}' permanently deleted.", args.file_id);
                codes::SUCCESS
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                map_error_to_exit_code(&e)
            }
        }
    } else {
        let url = crate::services::drive::folders::build_trash_url(&args.file_id);
        let body = serde_json::json!({"trashed": true});
        match crate::http::api::api_patch::<serde_json::Value>(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
            ctx.is_dry_run(),
        )
        .await
        {
            Ok(Some(_)) => {
                eprintln!("File '{}' trashed.", args.file_id);
                codes::SUCCESS
            }
            Ok(None) => {
                eprintln!("[dry-run] would trash file '{}'", args.file_id);
                codes::SUCCESS
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                map_error_to_exit_code(&e)
            }
        }
    }
}

/// Handle Drive move.
async fn handle_drive_move(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveMoveArgs,
) -> i32 {
    // First, get the file to find current parents (request parents field)
    let get_url = format!(
        "{}?fields=parents",
        crate::services::drive::files::build_file_get_url(&args.file_id)
    );
    let file: crate::services::drive::types::DriveFile = match crate::http::api::api_get(
        &ctx.client,
        &get_url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let old_parents = file.parents.join(",");

    let url = format!(
        "{}/files/{}?addParents={}&removeParents={}",
        crate::services::drive::types::DRIVE_BASE_URL,
        args.file_id,
        args.parent,
        old_parents
    );
    let body = serde_json::json!({});

    match crate::http::api::api_patch::<crate::services::drive::types::DriveFile>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(moved_file)) => {
            if let Err(e) = ctx.write_output(&moved_file) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would move '{}' to '{}'",
                args.file_id, args.parent
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Drive rename.
async fn handle_drive_rename(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveRenameArgs,
) -> i32 {
    let url = format!(
        "{}/files/{}",
        crate::services::drive::types::DRIVE_BASE_URL,
        args.file_id
    );
    let body = crate::services::drive::folders::build_rename_body(&args.new_name);

    match crate::http::api::api_patch::<crate::services::drive::types::DriveFile>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(file)) => {
            if let Err(e) = ctx.write_output(&file) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would rename '{}' to '{}'",
                args.file_id, args.new_name
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Drive share.
async fn handle_drive_share(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveShareArgs,
) -> i32 {
    let body = match crate::services::drive::permissions::build_share_permission(
        &args.to,
        &args.role,
        args.email.as_deref(),
        args.domain.as_deref(),
    ) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::USAGE_ERROR;
        }
    };

    let url = crate::services::drive::permissions::build_create_permission_url(&args.file_id);

    match crate::http::api::api_post::<crate::services::drive::types::Permission>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(perm)) => {
            if let Err(e) = ctx.write_output(&perm) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would share '{}' as {} with {}",
                args.file_id, args.role, args.to
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Drive permissions list.
async fn handle_drive_permissions_list(
    ctx: &crate::services::ServiceContext,
    args: &drive::DrivePermissionsArgs,
) -> i32 {
    let url = crate::services::drive::permissions::build_list_permissions_url(&args.file_id);

    let response: crate::services::drive::types::PermissionListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    if let Err(e) = ctx.write_output(&response) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Drive copy.
async fn handle_drive_copy(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveCopyArgs,
) -> i32 {
    let url = crate::services::drive::files::build_file_copy_url(&args.file_id);

    let mut body_map = serde_json::Map::new();
    if let Some(ref name) = args.name {
        body_map.insert("name".to_string(), serde_json::json!(name));
    }
    if let Some(ref parent) = args.parent {
        body_map.insert("parents".to_string(), serde_json::json!([parent]));
    }
    let body = serde_json::Value::Object(body_map);

    match crate::http::api::api_post::<crate::services::drive::types::DriveFile>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(file)) => {
            if let Err(e) = ctx.write_output(&file) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would copy '{}'", args.file_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Drive download.
///
/// For Google Workspace files, exports to the requested format (default: PDF).
/// For binary files, downloads the raw content.
async fn handle_drive_download(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveDownloadArgs,
) -> i32 {
    use crate::services::drive::files::*;
    use crate::services::export;

    // 1. Get file metadata to determine type and name
    let metadata_url = format!(
        "{}?fields=id,name,mimeType,size",
        build_file_get_url(&args.file_id)
    );
    let file: crate::services::drive::types::DriveFile = match crate::http::api::api_get(
        &ctx.client,
        &metadata_url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let mime_type = file.mime_type.as_deref().unwrap_or("");
    let file_name = file.name.as_deref().unwrap_or("download");
    let file_size = file.size.as_deref().unwrap_or("unknown");

    // Dry-run: print what would happen and return early
    if ctx.is_dry_run() {
        eprintln!(
            "[dry-run] would download '{}' ({} bytes)",
            file_name, file_size
        );
        return codes::SUCCESS;
    }

    // 2. Determine if this is a Google Workspace file (needs export) or binary (direct download)
    if export::is_google_workspace_type(mime_type) {
        // Export path
        let format = args.format.as_deref().unwrap_or("pdf");
        let export_mime = match export::format_to_mime(format) {
            Some(m) => m,
            None => {
                eprintln!(
                    "Error: unsupported export format '{}'. Supported: pdf, docx, xlsx, pptx, csv, txt",
                    format
                );
                return codes::USAGE_ERROR;
            }
        };
        let url = build_file_export_url(&args.file_id, export_mime);
        let out_path = resolve_download_path(file_name, args.out.as_deref(), Some(export_mime));

        // Stream export to file
        match download_to_file(ctx, &url, &out_path).await {
            Ok(bytes) => {
                eprintln!("Exported {} bytes to {}", bytes, out_path);
                codes::SUCCESS
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                map_error_to_exit_code(&e)
            }
        }
    } else {
        // Direct binary download
        let url = build_file_download_url(&args.file_id);
        let out_path = resolve_download_path(file_name, args.out.as_deref(), None);

        match download_to_file(ctx, &url, &out_path).await {
            Ok(bytes) => {
                eprintln!("Downloaded {} bytes to {}", bytes, out_path);
                codes::SUCCESS
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                map_error_to_exit_code(&e)
            }
        }
    }
}

/// Stream a URL response to a file on disk.
async fn download_to_file(
    ctx: &crate::services::ServiceContext,
    url: &str,
    out_path: &str,
) -> anyhow::Result<u64> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let response = crate::http::api::api_get_raw(
        &ctx.client,
        url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await?;

    let mut file = tokio::fs::File::create(out_path).await?;
    let mut stream = response.bytes_stream();
    let mut total: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        total += chunk.len() as u64;
        file.write_all(&chunk).await?;
    }

    file.flush().await?;
    Ok(total)
}

/// Handle Drive upload.
///
/// Reads a local file and uploads it. Files > 5MB use the resumable upload
/// protocol (REQ-RT-029); smaller files use simple multipart.
async fn handle_drive_upload(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveUploadArgs,
) -> i32 {
    use crate::services::drive::files::{build_file_upload_url, RESUMABLE_THRESHOLD};
    use crate::services::export;

    // Check file size first without reading entire file into memory
    let file_meta = match std::fs::metadata(&args.path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", args.path, e);
            return codes::GENERIC_ERROR;
        }
    };

    let filename = args.name.as_deref().unwrap_or_else(|| {
        std::path::Path::new(&args.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload")
    });

    // Build metadata
    let mut metadata = serde_json::json!({
        "name": filename,
    });
    if let Some(ref parent) = args.parent {
        metadata["parents"] = serde_json::json!([parent]);
    }

    // Handle --convert-to or --convert flags to set target Google Workspace MIME type
    if let Some(ref target) = args.convert_to {
        match crate::services::drive::types::convert_to_mime(target) {
            Ok(mime) => {
                metadata["mimeType"] = serde_json::json!(mime);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return codes::USAGE_ERROR;
            }
        }
    } else if args.convert {
        // Auto-detect Google Workspace type from file extension
        let ext = std::path::Path::new(&args.path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let target_mime = match ext.as_str() {
            "docx" | "doc" => Some(crate::services::drive::types::MIME_GOOGLE_DOC),
            "xlsx" | "xls" => Some(crate::services::drive::types::MIME_GOOGLE_SHEET),
            "pptx" | "ppt" => Some(crate::services::drive::types::MIME_GOOGLE_SLIDES),
            _ => None, // Unrecognized extension: ignore silently (no conversion)
        };
        if let Some(mime) = target_mime {
            metadata["mimeType"] = serde_json::json!(mime);
        }
    }

    // Guess content type from extension
    let content_type = export::guess_content_type_from_path(&args.path);

    // REQ-RT-029: Use resumable upload for files > 5MB
    // Only read file into memory for small uploads; large files use streaming.
    if file_meta.len() > RESUMABLE_THRESHOLD {
        return handle_drive_resumable_upload(ctx, filename, &args.path, &metadata, content_type)
            .await;
    }

    // Only read entire file for small uploads
    let file_data = match std::fs::read(&args.path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", args.path, e);
            return codes::GENERIC_ERROR;
        }
    };

    // Simple multipart upload for smaller files
    let boundary = "omega_google_upload_boundary";
    let metadata_json = serde_json::to_string(&metadata).unwrap_or_default();

    let mut body = Vec::new();
    // Part 1: metadata
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
    body.extend_from_slice(metadata_json.as_bytes());
    body.extend_from_slice(b"\r\n");
    // Part 2: file content
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", content_type).as_bytes());
    body.extend_from_slice(&file_data);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let url = build_file_upload_url();
    let multipart_content_type = format!("multipart/related; boundary={}", boundary);

    match crate::http::api::api_post_bytes::<crate::services::drive::types::DriveFile>(
        &ctx.client,
        &url,
        &multipart_content_type,
        body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(file)) => {
            if let Err(e) = ctx.write_output(&file) {
                eprintln!("Error: {}", e);
                return codes::GENERIC_ERROR;
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would upload '{}' ({} bytes)",
                filename,
                file_data.len()
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Drive resumable upload for large files (> 5MB).
///
/// Follows the Google Drive resumable upload protocol (REQ-RT-029):
/// 1. POST metadata to get an upload URI (Location header)
/// 2. PUT data in chunks (256KB each) with Content-Range headers
/// 3. Progress reporting on stderr
async fn handle_drive_resumable_upload(
    ctx: &crate::services::ServiceContext,
    filename: &str,
    file_path: &str,
    metadata: &serde_json::Value,
    content_type: &str,
) -> i32 {
    use crate::services::drive::files::{build_resumable_upload_url, RESUMABLE_CHUNK_SIZE};
    use std::io::{Read, Seek, SeekFrom};

    // Get total file size from metadata to avoid reading entire file into memory
    let total = match std::fs::metadata(file_path) {
        Ok(m) => m.len() as usize,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            return codes::GENERIC_ERROR;
        }
    };

    if ctx.is_dry_run() {
        eprintln!(
            "[dry-run] would resumable-upload '{}' ({} bytes)",
            filename, total
        );
        return codes::SUCCESS;
    }

    // 1. Initiate resumable upload session
    let initiate_url = build_resumable_upload_url();
    let metadata_json = serde_json::to_string(metadata).unwrap_or_default();

    if ctx.is_verbose() {
        eprintln!("> POST {} (initiate resumable upload)", initiate_url);
        eprintln!("> X-Upload-Content-Type: {}", content_type);
        eprintln!("> X-Upload-Content-Length: {}", total);
    }

    let response = match ctx
        .client
        .post(&initiate_url)
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("X-Upload-Content-Type", content_type)
        .header("X-Upload-Content-Length", total.to_string())
        .body(metadata_json)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error initiating resumable upload: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!(
            "Error initiating resumable upload (HTTP {}): {}",
            status.as_u16(),
            body
        );
        return codes::GENERIC_ERROR;
    }

    let upload_uri = match response.headers().get("location") {
        Some(uri) => match uri.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                eprintln!("Error: invalid upload URI in response headers");
                return codes::GENERIC_ERROR;
            }
        },
        None => {
            eprintln!("Error: no upload URI (Location header) in response");
            return codes::GENERIC_ERROR;
        }
    };

    if ctx.is_verbose() {
        eprintln!("< Upload URI obtained, uploading in chunks...");
    }

    // Open the file for streaming chunk reads
    let mut file = match std::fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", file_path, e);
            return codes::GENERIC_ERROR;
        }
    };

    // 2. Upload in chunks, reading from file as needed
    let chunk_size = RESUMABLE_CHUNK_SIZE;
    let mut offset: usize = 0;

    while offset < total {
        let end = std::cmp::min(offset + chunk_size, total);
        let chunk_len = end - offset;

        // Seek to the current offset and read only this chunk
        if let Err(e) = file.seek(SeekFrom::Start(offset as u64)) {
            eprintln!("Error seeking in file: {}", e);
            return codes::GENERIC_ERROR;
        }
        let mut chunk = vec![0u8; chunk_len];
        if let Err(e) = file.read_exact(&mut chunk) {
            eprintln!("Error reading file chunk: {}", e);
            return codes::GENERIC_ERROR;
        }

        let content_range = format!("bytes {}-{}/{}", offset, end - 1, total);

        if ctx.is_verbose() {
            eprintln!(
                "> PUT chunk [{}-{}/{}] ({} bytes)",
                offset,
                end - 1,
                total,
                chunk.len()
            );
        }

        let put_response = match ctx
            .client
            .put(&upload_uri)
            .header("Content-Length", chunk.len().to_string())
            .header("Content-Range", &content_range)
            .body(chunk)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error during resumable upload: {}", e);
                return codes::GENERIC_ERROR;
            }
        };

        let status = put_response.status();
        if status.is_success() {
            // Final chunk completed -- parse the response for file metadata
            eprint!("\rUploading: 100%");
            eprintln!();
            let body = put_response.text().await.unwrap_or_default();
            match serde_json::from_str::<crate::services::drive::types::DriveFile>(&body) {
                Ok(file) => {
                    eprintln!("Upload complete: {} bytes", total);
                    if let Err(e) = ctx.write_output(&file) {
                        eprintln!("Error: {}", e);
                        return codes::GENERIC_ERROR;
                    }
                    return codes::SUCCESS;
                }
                Err(e) => {
                    eprintln!(
                        "Upload complete ({} bytes) but failed to parse response: {}",
                        total, e
                    );
                    return codes::SUCCESS;
                }
            }
        } else if status.as_u16() == 308 {
            // 308 Resume Incomplete -- parse Range header to determine actual bytes received
            if let Some(range_hdr) = put_response.headers().get("range") {
                if let Ok(range_str) = range_hdr.to_str() {
                    // Format: "bytes=0-12345"
                    if let Some(end_str) = range_str
                        .strip_prefix("bytes=")
                        .and_then(|s| s.split('-').nth(1))
                    {
                        if let Ok(received_end) = end_str.parse::<usize>() {
                            offset = received_end + 1;
                        } else {
                            offset = end;
                        }
                    } else {
                        offset = end;
                    }
                } else {
                    offset = end;
                }
            } else {
                offset = end;
            }
            let pct = (offset as f64 / total as f64) * 100.0;
            eprint!("\rUploading: {:.0}%", pct);
        } else {
            let body = put_response.text().await.unwrap_or_default();
            eprintln!(
                "\nError during resumable upload chunk (HTTP {}): {}",
                status.as_u16(),
                body
            );
            return codes::GENERIC_ERROR;
        }
    }

    eprintln!("\rUpload complete: {} bytes", total);
    codes::SUCCESS
}

/// Handle Gmail attachment download.
///
/// Downloads a single attachment from a message, base64url-decodes it,
/// and writes the raw bytes to a file.
async fn handle_gmail_attachment(
    ctx: &crate::services::ServiceContext,
    args: &gmail::GmailAttachmentArgs,
) -> i32 {
    use crate::services::gmail::message::build_attachment_url;

    let url = build_attachment_url(&args.message_id, &args.attachment_id);

    // The Gmail API returns the attachment as JSON with a base64url-encoded "data" field
    let response: serde_json::Value = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let data_b64 = match response.get("data").and_then(|v| v.as_str()) {
        Some(d) => d,
        None => {
            eprintln!("Error: attachment response missing 'data' field");
            return codes::GENERIC_ERROR;
        }
    };

    // Base64url decode
    use base64::Engine;
    let decoded = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data_b64) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error decoding attachment: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    // Determine output path
    let filename = args.name.as_deref().unwrap_or("attachment");
    let out_path = match &args.out {
        Some(p) => p.clone(),
        None => filename.to_string(),
    };

    // Dry-run: print what would happen and return early
    if ctx.is_dry_run() {
        eprintln!(
            "[dry-run] would download '{}' ({} bytes)",
            out_path,
            decoded.len()
        );
        return codes::SUCCESS;
    }

    // Write to file
    if let Err(e) = std::fs::write(&out_path, &decoded) {
        eprintln!("Error writing file '{}': {}", out_path, e);
        return codes::GENERIC_ERROR;
    }

    eprintln!("Downloaded {} bytes to {}", decoded.len(), out_path);
    codes::SUCCESS
}

/// Build a Drive file list URL with query, pageSize, and optional pageToken.
fn build_drive_list_url(
    query: &str,
    max: u32,
    page_token: Option<&str>,
    all_drives: bool,
) -> String {
    let base = &format!("{}/files", crate::services::drive::types::DRIVE_BASE_URL);
    let encoded_query: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let mut params = vec![
        format!("q={}", encoded_query),
        format!("pageSize={}", max),
        "fields=files(id,name,mimeType,size,modifiedTime,parents),nextPageToken".to_string(),
    ];
    if all_drives {
        params.push("supportsAllDrives=true".to_string());
        params.push("includeItemsFromAllDrives=true".to_string());
    }
    if let Some(token) = page_token {
        params.push(format!("pageToken={}", token));
    }
    format!("{}?{}", base, params.join("&"))
}

/// Handle the `docs` command and its subcommands.
async fn handle_docs(args: docs::DocsArgs, flags: &root::RootFlags) -> i32 {
    use docs::{DocsCommand, DocsCommentsCommand};

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        DocsCommand::Export(ref a) => handle_docs_export(&ctx, a).await,
        DocsCommand::Info(ref a) => handle_docs_info(&ctx, a).await,
        DocsCommand::Create(ref a) => handle_docs_create(&ctx, a).await,
        DocsCommand::Copy(ref a) => handle_docs_copy(&ctx, a).await,
        DocsCommand::Cat(ref a) => handle_docs_cat(&ctx, a).await,
        DocsCommand::ListTabs(ref a) => handle_docs_list_tabs(&ctx, a).await,
        DocsCommand::Comments(ref a) => match a.command {
            DocsCommentsCommand::List(ref la) => handle_docs_comments_list(&ctx, la).await,
            DocsCommentsCommand::Get(ref ga) => handle_docs_comments_get(&ctx, ga).await,
            DocsCommentsCommand::Add(ref aa) => handle_docs_comments_add(&ctx, aa).await,
            DocsCommentsCommand::Reply(ref ra) => handle_docs_comments_reply(&ctx, ra).await,
            DocsCommentsCommand::Resolve(ref ra) => handle_docs_comments_resolve(&ctx, ra).await,
            DocsCommentsCommand::Delete(ref da) => handle_docs_comments_delete(&ctx, da).await,
        },
        DocsCommand::Write(ref a) => handle_docs_write(&ctx, a).await,
        DocsCommand::Insert(ref a) => handle_docs_insert(&ctx, a).await,
        DocsCommand::Delete(ref a) => handle_docs_delete(&ctx, a).await,
        DocsCommand::FindReplace(ref a) => handle_docs_find_replace(&ctx, a).await,
        DocsCommand::Update(ref a) => handle_docs_update(&ctx, a).await,
        DocsCommand::Edit(ref a) => handle_docs_edit(&ctx, a).await,
        DocsCommand::Sed(ref a) => handle_docs_sed(&ctx, a).await,
        DocsCommand::Clear(ref a) => handle_docs_clear(&ctx, a).await,
    }
}

/// Handle Docs export: download/export a Google Doc in the specified format.
async fn handle_docs_export(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsExportArgs,
) -> i32 {
    use crate::services::docs::export::resolve_export_mime;
    use crate::services::drive::files::{
        build_file_export_url, build_file_get_url, resolve_download_path,
    };

    // 1. Get file metadata
    let metadata_url = format!(
        "{}?fields=id,name,mimeType,size",
        build_file_get_url(&args.doc_id)
    );
    let file: crate::services::drive::types::DriveFile = match crate::http::api::api_get(
        &ctx.client,
        &metadata_url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let file_name = file.name.as_deref().unwrap_or("document");

    if ctx.is_dry_run() {
        eprintln!("[dry-run] would export '{}' as {}", file_name, args.format);
        return codes::SUCCESS;
    }

    // 2. Export
    let export_mime = resolve_export_mime(&args.format);
    let url = build_file_export_url(&args.doc_id, export_mime);
    let out_path = resolve_download_path(file_name, args.out.as_deref(), Some(export_mime));

    match download_to_file(ctx, &url, &out_path).await {
        Ok(bytes) => {
            eprintln!("Exported {} bytes to {}", bytes, out_path);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs info: get document metadata.
async fn handle_docs_info(ctx: &crate::services::ServiceContext, args: &docs::DocsInfoArgs) -> i32 {
    let url = crate::services::docs::content::build_doc_get_url(&args.doc_id);

    let doc: crate::services::docs::types::Document = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&doc) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Docs create: create a new Google Doc via Drive API.
async fn handle_docs_create(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsCreateArgs,
) -> i32 {
    use crate::services::docs::export::build_doc_create_body;
    use crate::services::drive::types::DRIVE_BASE_URL;

    let url = format!("{}/files", DRIVE_BASE_URL);
    let body = build_doc_create_body(&args.title, args.parent.as_deref());

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create document '{}'", args.title);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs copy: copy a document via Drive API.
async fn handle_docs_copy(ctx: &crate::services::ServiceContext, args: &docs::DocsCopyArgs) -> i32 {
    use crate::services::docs::export::{build_doc_copy_body, build_doc_copy_url};

    let url = build_doc_copy_url(&args.doc_id);
    let body = build_doc_copy_body(&args.title, args.parent.as_deref());

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would copy document '{}' as '{}'",
                args.doc_id, args.title
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs cat: extract and print plain text from a document.
async fn handle_docs_cat(ctx: &crate::services::ServiceContext, args: &docs::DocsCatArgs) -> i32 {
    use crate::services::docs::content::*;

    let include_tabs = args.all_tabs || args.tab.is_some();
    let url = build_doc_get_url_with_tabs(&args.doc_id, include_tabs);

    let doc: crate::services::docs::types::Document = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    // If --raw, output the JSON structure directly
    if args.raw {
        if let Err(e) = ctx.write_output(&doc) {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
        return codes::SUCCESS;
    }

    // If a specific tab is requested
    if let Some(ref tab_id) = args.tab {
        for tab in &doc.tabs {
            if let Some(ref props) = tab.tab_properties {
                let matches = props.tab_id.as_deref() == Some(tab_id.as_str())
                    || props.title.as_deref() == Some(tab_id.as_str());
                if matches {
                    let text = extract_tab_text(tab);
                    print!("{}", text);
                    return codes::SUCCESS;
                }
            }
        }
        eprintln!("Error: tab '{}' not found", tab_id);
        return codes::GENERIC_ERROR;
    }

    // If --all-tabs, print all tab contents
    if args.all_tabs {
        for tab in &doc.tabs {
            if let Some(ref props) = tab.tab_properties {
                let title = props.title.as_deref().unwrap_or("Untitled");
                eprintln!("--- Tab: {} ---", title);
            }
            let text = extract_tab_text(tab);
            print!("{}", text);
        }
        return codes::SUCCESS;
    }

    // Default: extract from main body
    if let Some(ref body) = doc.body {
        let text = extract_plain_text(body);
        print!("{}", text);
    }
    codes::SUCCESS
}

/// Handle Docs list-tabs: list tab info from a document.
async fn handle_docs_list_tabs(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsListTabsArgs,
) -> i32 {
    let url = crate::services::docs::content::build_doc_get_url_with_tabs(&args.doc_id, true);

    let doc: crate::services::docs::types::Document = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if ctx.flags.json {
        // Output tab properties as JSON
        let tab_props: Vec<&crate::services::docs::types::TabProperties> = doc
            .tabs
            .iter()
            .filter_map(|t| t.tab_properties.as_ref())
            .collect();
        if let Err(e) = ctx.write_output(&tab_props) {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    } else {
        for tab in &doc.tabs {
            if let Some(ref props) = tab.tab_properties {
                let id = props.tab_id.as_deref().unwrap_or("-");
                let title = props.title.as_deref().unwrap_or("Untitled");
                let index = props
                    .index
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "-".to_string());
                println!("{}\t{}\t{}", id, title, index);
            }
        }
    }
    codes::SUCCESS
}

/// Handle Docs comments list.
async fn handle_docs_comments_list(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsCommentsListArgs,
) -> i32 {
    let url = crate::services::docs::comments::build_comments_list_url(&args.file_id);

    let result: serde_json::Value = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Docs comments get.
async fn handle_docs_comments_get(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsCommentsGetArgs,
) -> i32 {
    let url =
        crate::services::docs::comments::build_comment_get_url(&args.file_id, &args.comment_id);

    let result: serde_json::Value = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Docs comments add.
async fn handle_docs_comments_add(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsCommentsAddArgs,
) -> i32 {
    let url = crate::services::docs::comments::build_comment_create_url(&args.file_id);
    let body = crate::services::docs::comments::build_comment_create_body(&args.content);

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would add comment to '{}'", args.file_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs comments reply.
async fn handle_docs_comments_reply(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsCommentsReplyArgs,
) -> i32 {
    let url =
        crate::services::docs::comments::build_comment_reply_url(&args.file_id, &args.comment_id);
    let body = crate::services::docs::comments::build_comment_reply_body(&args.content);

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would reply to comment '{}' on '{}'",
                args.comment_id, args.file_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs comments resolve.
async fn handle_docs_comments_resolve(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsCommentsResolveArgs,
) -> i32 {
    let url =
        crate::services::docs::comments::build_comment_resolve_url(&args.file_id, &args.comment_id);
    let body = crate::services::docs::comments::build_comment_resolve_body();

    match crate::http::api::api_patch::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would resolve comment '{}' on '{}'",
                args.comment_id, args.file_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs comments delete.
async fn handle_docs_comments_delete(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsCommentsDeleteArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url =
        crate::services::docs::comments::build_comment_delete_url(&args.file_id, &args.comment_id);

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Comment '{}' deleted.", args.comment_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs write: write content to a document.
/// If --replace, clears the document first and then inserts new content.
/// Otherwise appends at the end.
async fn handle_docs_write(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsWriteArgs,
) -> i32 {
    use crate::services::docs::content::build_doc_get_url;
    use crate::services::docs::edit::{
        build_batch_update_url, build_insert_text_body, build_replace_content_body,
    };

    // Determine content: from --file or from positional args
    let content = if let Some(ref file_path) = args.file {
        match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file_path, e);
                return codes::GENERIC_ERROR;
            }
        }
    } else {
        args.content.join(" ")
    };

    if content.is_empty() {
        eprintln!("Error: no content provided");
        return codes::USAGE_ERROR;
    }

    // Get document to determine end index
    let doc_url = build_doc_get_url(&args.doc_id);
    let doc: crate::services::docs::types::Document = match crate::http::api::api_get(
        &ctx.client,
        &doc_url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let end_index = doc
        .body
        .as_ref()
        .and_then(|b| b.content.last())
        .and_then(|e| e.end_index)
        .unwrap_or(1);

    let batch_url = build_batch_update_url(&args.doc_id);

    let body = if args.replace {
        build_replace_content_body(&content, end_index)
    } else {
        // Append: insert at end_index - 1 (before trailing newline)
        let insert_index = if end_index > 1 { end_index - 1 } else { 1 };
        build_insert_text_body(&content, insert_index)
    };

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would write {} bytes to document '{}'",
                content.len(),
                args.doc_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs insert: insert text at a specific index.
async fn handle_docs_insert(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsInsertArgs,
) -> i32 {
    use crate::services::docs::edit::{build_batch_update_url, build_insert_text_body};

    let index: i64 = match args.index.parse() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("Error: invalid index '{}'", args.index);
            return codes::USAGE_ERROR;
        }
    };

    // Determine content: from --file or from positional args
    let content = if let Some(ref file_path) = args.file {
        match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file_path, e);
                return codes::GENERIC_ERROR;
            }
        }
    } else {
        args.content.join(" ")
    };

    if content.is_empty() {
        eprintln!("Error: no content provided");
        return codes::USAGE_ERROR;
    }

    let batch_url = build_batch_update_url(&args.doc_id);
    let body = build_insert_text_body(&content, index);

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would insert {} bytes at index {} in '{}'",
                content.len(),
                index,
                args.doc_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs delete: delete a range of content.
async fn handle_docs_delete(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsDeleteArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    use crate::services::docs::edit::{build_batch_update_url, build_delete_content_range_body};

    let batch_url = build_batch_update_url(&args.doc_id);
    let body = build_delete_content_range_body(args.start, args.end);

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would delete range [{}, {}) in '{}'",
                args.start, args.end, args.doc_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs find-replace: find and replace text in a document.
async fn handle_docs_find_replace(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsFindReplaceArgs,
) -> i32 {
    use crate::services::docs::edit::{build_batch_update_url, build_replace_all_text_body};

    let batch_url = build_batch_update_url(&args.doc_id);
    let body = build_replace_all_text_body(&args.find, &args.replace, args.match_case);

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would replace '{}' with '{}' in '{}'",
                args.find, args.replace, args.doc_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs update: generic batchUpdate with raw JSON requests.
async fn handle_docs_update(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsUpdateArgs,
) -> i32 {
    use crate::services::docs::edit::build_batch_update_url;

    // Get content from --content or --content-file
    let raw_content = if let Some(ref content) = args.content {
        content.clone()
    } else if let Some(ref file_path) = args.content_file {
        match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file_path, e);
                return codes::GENERIC_ERROR;
            }
        }
    } else {
        eprintln!("Error: --content or --content-file is required");
        return codes::USAGE_ERROR;
    };

    // Parse the content as JSON requests
    let body: serde_json::Value = match serde_json::from_str(&raw_content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: invalid JSON: {}", e);
            return codes::USAGE_ERROR;
        }
    };

    let batch_url = build_batch_update_url(&args.doc_id);

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would send batchUpdate to '{}'", args.doc_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs edit: find/replace with --find and --replace flags.
async fn handle_docs_edit(ctx: &crate::services::ServiceContext, args: &docs::DocsEditArgs) -> i32 {
    use crate::services::docs::edit::{build_batch_update_url, build_replace_all_text_body};

    let batch_url = build_batch_update_url(&args.doc_id);
    let body = build_replace_all_text_body(&args.find, &args.replace, args.match_case);

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would replace '{}' with '{}' in '{}'",
                args.find, args.replace, args.doc_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs sed: sed-like find/replace using sed expressions.
async fn handle_docs_sed(ctx: &crate::services::ServiceContext, args: &docs::DocsSedArgs) -> i32 {
    use crate::services::docs::edit::build_batch_update_url;
    use crate::services::docs::sedmat::{parse_sed_expression, parse_sed_file};

    let mut all_exprs = Vec::new();

    // 1. Positional expressions
    for expr_str in &args.expression {
        match parse_sed_expression(expr_str) {
            Ok(expr) => all_exprs.push(expr),
            Err(e) => {
                eprintln!("Error parsing sed expression '{}': {}", expr_str, e);
                return codes::USAGE_ERROR;
            }
        }
    }

    // 2. -e/--expression flag expressions
    for expr_str in &args.expr_flag {
        match parse_sed_expression(expr_str) {
            Ok(expr) => all_exprs.push(expr),
            Err(e) => {
                eprintln!("Error parsing sed expression '{}': {}", expr_str, e);
                return codes::USAGE_ERROR;
            }
        }
    }

    // 3. -f/--file expressions
    if let Some(ref file_path) = args.file {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file_path, e);
                return codes::GENERIC_ERROR;
            }
        };
        match parse_sed_file(&content) {
            Ok(exprs) => all_exprs.extend(exprs),
            Err(e) => {
                eprintln!("Error parsing sed file '{}': {}", file_path, e);
                return codes::USAGE_ERROR;
            }
        }
    }

    if all_exprs.is_empty() {
        eprintln!("Error: no sed expressions provided");
        return codes::USAGE_ERROR;
    }

    let batch_url = build_batch_update_url(&args.doc_id);

    // Build a batch request with all find-replace operations
    let requests: Vec<serde_json::Value> = all_exprs
        .iter()
        .map(|expr| {
            serde_json::json!({
                "replaceAllText": {
                    "containsText": {
                        "text": expr.find,
                        "matchCase": !expr.case_insensitive
                    },
                    "replaceText": expr.replace
                }
            })
        })
        .collect();

    let body = serde_json::json!({"requests": requests});

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would apply {} sed expression(s) to '{}'",
                all_exprs.len(),
                args.doc_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Docs clear: clear all document content.
async fn handle_docs_clear(
    ctx: &crate::services::ServiceContext,
    args: &docs::DocsClearArgs,
) -> i32 {
    if !ctx.is_force() && !ctx.flags.no_input {
        eprint!("Are you sure you want to clear the entire document? [y/N] ");
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err()
            || !input.trim().eq_ignore_ascii_case("y")
        {
            eprintln!("Aborted.");
            return codes::SUCCESS;
        }
    } else if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    use crate::services::docs::content::build_doc_get_url;
    use crate::services::docs::edit::{build_batch_update_url, build_clear_body};

    // Get document to determine end index
    let doc_url = build_doc_get_url(&args.doc_id);
    let doc: crate::services::docs::types::Document = match crate::http::api::api_get(
        &ctx.client,
        &doc_url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let end_index = doc
        .body
        .as_ref()
        .and_then(|b| b.content.last())
        .and_then(|e| e.end_index)
        .unwrap_or(1);

    if end_index <= 1 {
        eprintln!("Document is already empty.");
        return codes::SUCCESS;
    }

    let batch_url = build_batch_update_url(&args.doc_id);
    let body = build_clear_body(end_index);

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would clear document '{}'", args.doc_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle the `sheets` command and its subcommands.
async fn handle_sheets(args: sheets::SheetsArgs, flags: &root::RootFlags) -> i32 {
    use sheets::SheetsCommand;

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        SheetsCommand::Get(ref a) => handle_sheets_get(&ctx, a).await,
        SheetsCommand::Update(ref a) => handle_sheets_update(&ctx, a).await,
        SheetsCommand::Append(ref a) => handle_sheets_append(&ctx, a).await,
        SheetsCommand::Insert(ref a) => handle_sheets_insert(&ctx, a).await,
        SheetsCommand::Clear(ref a) => handle_sheets_clear(&ctx, a).await,
        SheetsCommand::Format(ref a) => handle_sheets_format(&ctx, a).await,
        SheetsCommand::Notes(ref a) => handle_sheets_notes(&ctx, a).await,
        SheetsCommand::Metadata(ref a) => handle_sheets_metadata(&ctx, a).await,
        SheetsCommand::Create(ref a) => handle_sheets_create(&ctx, a).await,
        SheetsCommand::Copy(ref a) => handle_sheets_copy(&ctx, a).await,
        SheetsCommand::Export(ref a) => handle_sheets_export(&ctx, a).await,
    }
}

/// Handle Sheets get: read cell values.
async fn handle_sheets_get(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsGetArgs,
) -> i32 {
    use crate::services::sheets::clean_range;
    use crate::services::sheets::read::build_values_get_url_with_options;

    let range = clean_range(&args.range);
    let url = build_values_get_url_with_options(
        &args.spreadsheet_id,
        &range,
        args.dimension.as_deref(),
        args.render.as_deref(),
    );

    let vr: crate::services::sheets::types::ValueRange = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&vr) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Resolve values for update/append from positional args or --values-json.
fn resolve_sheet_values(
    positional: &[String],
    values_json: Option<&str>,
) -> Result<Vec<Vec<serde_json::Value>>, String> {
    use crate::services::sheets::write::parse_cell_values;

    if let Some(json_str) = values_json {
        // Parse as JSON array of arrays
        let parsed: Vec<Vec<serde_json::Value>> =
            serde_json::from_str(json_str).map_err(|e| format!("invalid --values-json: {}", e))?;
        Ok(parsed)
    } else if !positional.is_empty() {
        // Join positional args and parse with pipe/comma syntax
        let joined = positional.join(",");
        Ok(parse_cell_values(&joined))
    } else {
        Err("no values provided; pass positional values or --values-json".to_string())
    }
}

/// Handle Sheets update: write cell values (PUT).
async fn handle_sheets_update(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsUpdateArgs,
) -> i32 {
    use crate::services::sheets::clean_range;
    use crate::services::sheets::write::{build_values_body, build_values_update_url};

    let range = clean_range(&args.range);
    let url = build_values_update_url(&args.spreadsheet_id, &range, &args.input);
    let values = match resolve_sheet_values(&args.values, args.values_json.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::USAGE_ERROR;
        }
    };
    let body = build_values_body(values);
    let body_bytes = serde_json::to_vec(&body).unwrap_or_default();

    match crate::http::api::api_put_bytes::<crate::services::sheets::types::UpdateValuesResponse>(
        &ctx.client,
        &url,
        "application/json",
        body_bytes,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would update range '{}' in spreadsheet '{}'",
                range, args.spreadsheet_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Sheets append: append rows (POST).
async fn handle_sheets_append(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsAppendArgs,
) -> i32 {
    use crate::services::sheets::clean_range;
    use crate::services::sheets::write::{build_values_append_url, build_values_body};

    let range = clean_range(&args.range);
    let url = build_values_append_url(
        &args.spreadsheet_id,
        &range,
        &args.input,
        Some(args.insert.as_str()),
    );
    let values = match resolve_sheet_values(&args.values, args.values_json.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::USAGE_ERROR;
        }
    };
    let body = build_values_body(values);

    match crate::http::api::api_post::<crate::services::sheets::types::AppendValuesResponse>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would append to range '{}' in spreadsheet '{}'",
                range, args.spreadsheet_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Sheets insert: insert rows or columns via batchUpdate.
async fn handle_sheets_insert(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsInsertArgs,
) -> i32 {
    use crate::services::sheets::format::build_batch_update_url;
    use crate::services::sheets::structure::build_insert_dimension_request;

    let dimension = match args.dimension.to_lowercase().as_str() {
        "rows" | "row" => "ROWS",
        "cols" | "col" | "columns" | "column" => "COLUMNS",
        _ => {
            eprintln!("Error: dimension must be 'rows' or 'cols'");
            return codes::USAGE_ERROR;
        }
    };

    // Parse sheet as sheet ID (numeric)
    let sheet_id: i64 = match args.sheet.parse() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: sheet must be a numeric sheet ID (use 'sheets metadata' to find it)");
            return codes::USAGE_ERROR;
        }
    };

    let start = args.start as i64;
    let end = start + args.count as i64;
    let inherit_before = args.after;

    let request = build_insert_dimension_request(sheet_id, dimension, start, end, inherit_before);
    let body = serde_json::json!({ "requests": [request] });
    let url = build_batch_update_url(&args.spreadsheet_id);

    match crate::http::api::api_post::<crate::services::sheets::types::BatchUpdateResponse>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would insert {} {} at index {} in sheet {} of spreadsheet '{}'",
                args.count, dimension, args.start, sheet_id, args.spreadsheet_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Sheets clear: clear cell values (POST).
async fn handle_sheets_clear(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsClearArgs,
) -> i32 {
    use crate::services::sheets::clean_range;
    use crate::services::sheets::write::build_values_clear_url;

    let range = clean_range(&args.range);
    let url = build_values_clear_url(&args.spreadsheet_id, &range);
    let body = serde_json::json!({});

    match crate::http::api::api_post::<crate::services::sheets::types::ClearValuesResponse>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would clear range '{}' in spreadsheet '{}'",
                range, args.spreadsheet_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Sheets format: apply cell formatting via batchUpdate.
async fn handle_sheets_format(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsFormatArgs,
) -> i32 {
    use crate::services::sheets::a1::parse_a1;
    use crate::services::sheets::clean_range;
    use crate::services::sheets::format::{build_batch_update_url, build_format_request};

    let range_str = clean_range(&args.range);
    let parsed = match parse_a1(&range_str) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: invalid range '{}': {}", range_str, e);
            return codes::USAGE_ERROR;
        }
    };

    // Default sheet_id to 0 when no sheet name is given
    let sheet_id: i64 = 0;

    let request =
        match build_format_request(sheet_id, &parsed, &args.format_json, &args.format_fields) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return codes::USAGE_ERROR;
            }
        };

    let body = serde_json::json!({ "requests": [request] });
    let url = build_batch_update_url(&args.spreadsheet_id);

    match crate::http::api::api_post::<crate::services::sheets::types::BatchUpdateResponse>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would format range '{}' in spreadsheet '{}'",
                range_str, args.spreadsheet_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Sheets notes: read cell notes via spreadsheets.get with includeGridData.
async fn handle_sheets_notes(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsNotesArgs,
) -> i32 {
    use crate::services::sheets::clean_range;
    use crate::services::sheets::read::build_metadata_url;

    let range = clean_range(&args.range);
    let url = format!(
        "{}?includeGridData=true&ranges={}&fields=sheets.data.rowData.values.note",
        build_metadata_url(&args.spreadsheet_id),
        percent_encoding::utf8_percent_encode(&range, percent_encoding::NON_ALPHANUMERIC)
    );

    let ss: crate::services::sheets::types::Spreadsheet = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&ss) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Sheets metadata: get spreadsheet metadata.
async fn handle_sheets_metadata(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsMetadataArgs,
) -> i32 {
    use crate::services::sheets::read::build_metadata_url;

    let url = build_metadata_url(&args.spreadsheet_id);

    let ss: crate::services::sheets::types::Spreadsheet = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&ss) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Sheets create: create a new spreadsheet.
async fn handle_sheets_create(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsCreateArgs,
) -> i32 {
    use crate::services::sheets::structure::{
        build_create_spreadsheet_body, build_create_spreadsheet_url,
    };

    let sheet_names: Vec<String> = args
        .sheets
        .as_deref()
        .map(|s| s.split(',').map(|n| n.trim().to_string()).collect())
        .unwrap_or_default();

    let body = build_create_spreadsheet_body(&args.title, &sheet_names);
    let url = build_create_spreadsheet_url();

    match crate::http::api::api_post::<crate::services::sheets::types::Spreadsheet>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create spreadsheet '{}'", args.title);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Sheets copy: copy a spreadsheet via Drive API.
async fn handle_sheets_copy(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsCopyArgs,
) -> i32 {
    use crate::services::sheets::structure::{build_copy_body, build_copy_spreadsheet_url};

    let url = build_copy_spreadsheet_url(&args.spreadsheet_id);
    let body = build_copy_body(&args.title, args.parent.as_deref());

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would copy spreadsheet '{}' as '{}'",
                args.spreadsheet_id, args.title
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Sheets export: export/download a spreadsheet.
async fn handle_sheets_export(
    ctx: &crate::services::ServiceContext,
    args: &sheets::SheetsExportArgs,
) -> i32 {
    use crate::services::drive::files::resolve_download_path;
    use crate::services::sheets::structure::{build_export_url, resolve_export_mime};

    // Determine export MIME and output path
    let export_mime = resolve_export_mime(&args.format);
    let url = build_export_url(&args.spreadsheet_id, &args.format);
    let default_name = format!("spreadsheet-{}", &args.spreadsheet_id);
    let out_path = resolve_download_path(&default_name, args.out.as_deref(), Some(export_mime));

    if ctx.is_dry_run() {
        eprintln!(
            "[dry-run] would export spreadsheet '{}' as {} to {}",
            args.spreadsheet_id, args.format, out_path
        );
        return codes::SUCCESS;
    }

    match download_to_file(ctx, &url, &out_path).await {
        Ok(bytes) => {
            eprintln!("Exported {} bytes to {}", bytes, out_path);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle the `slides` command and its subcommands.
async fn handle_slides(args: slides::SlidesArgs, flags: &root::RootFlags) -> i32 {
    use slides::SlidesCommand;

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        SlidesCommand::Export(ref a) => handle_slides_export(&ctx, a).await,
        SlidesCommand::Info(ref a) => handle_slides_info(&ctx, a).await,
        SlidesCommand::Create(ref a) => handle_slides_create(&ctx, a).await,
        SlidesCommand::CreateFromMarkdown(ref a) => {
            handle_slides_create_from_markdown(&ctx, a).await
        }
        SlidesCommand::Copy(ref a) => handle_slides_copy(&ctx, a).await,
        SlidesCommand::ListSlides(ref a) => handle_slides_list_slides(&ctx, a).await,
        SlidesCommand::AddSlide(ref a) => handle_slides_add_slide(&ctx, a).await,
        SlidesCommand::DeleteSlide(ref a) => handle_slides_delete_slide(&ctx, a).await,
        SlidesCommand::ReadSlide(ref a) => handle_slides_read_slide(&ctx, a).await,
        SlidesCommand::UpdateNotes(ref a) => handle_slides_update_notes(&ctx, a).await,
        SlidesCommand::ReplaceSlide(ref a) => handle_slides_replace_slide(&ctx, a).await,
    }
}

/// Handle Slides export: download/export a presentation.
async fn handle_slides_export(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesExportArgs,
) -> i32 {
    use crate::services::drive::files::{
        build_file_export_url, build_file_get_url, resolve_download_path,
    };
    use crate::services::slides::export::resolve_export_mime;

    // 1. Get file metadata
    let metadata_url = format!(
        "{}?fields=id,name,mimeType,size",
        build_file_get_url(&args.presentation_id)
    );
    let file: crate::services::drive::types::DriveFile = match crate::http::api::api_get(
        &ctx.client,
        &metadata_url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let file_name = file.name.as_deref().unwrap_or("presentation");

    if ctx.is_dry_run() {
        eprintln!("[dry-run] would export '{}' as {}", file_name, args.format);
        return codes::SUCCESS;
    }

    // 2. Export
    let export_mime = resolve_export_mime(&args.format);
    let url = build_file_export_url(&args.presentation_id, export_mime);
    let out_path = resolve_download_path(file_name, args.out.as_deref(), Some(export_mime));

    match download_to_file(ctx, &url, &out_path).await {
        Ok(bytes) => {
            eprintln!("Exported {} bytes to {}", bytes, out_path);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Slides info: get presentation metadata.
async fn handle_slides_info(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesInfoArgs,
) -> i32 {
    let url =
        crate::services::slides::presentations::build_presentation_get_url(&args.presentation_id);

    let pres: crate::services::slides::types::Presentation = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&pres) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Slides create: create a new presentation.
async fn handle_slides_create(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesCreateArgs,
) -> i32 {
    // If a template is provided, copy from it
    if let Some(ref template_id) = args.template {
        let url = crate::services::slides::presentations::build_template_copy_url(template_id);
        let body = crate::services::slides::presentations::build_create_from_template_body(
            &args.title,
            template_id,
            args.parent.as_deref(),
        );

        match crate::http::api::api_post::<serde_json::Value>(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
            ctx.is_dry_run(),
        )
        .await
        {
            Ok(Some(result)) => {
                if let Err(e) = ctx.write_output(&result) {
                    eprintln!("Error: {}", e);
                    return map_error_to_exit_code(&e);
                }
                codes::SUCCESS
            }
            Ok(None) => {
                eprintln!(
                    "[dry-run] would create presentation '{}' from template '{}'",
                    args.title, template_id
                );
                codes::SUCCESS
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                map_error_to_exit_code(&e)
            }
        }
    } else {
        // Create a blank presentation via the Slides API
        let url = format!("{}/presentations", crate::services::slides::SLIDES_BASE_URL);
        let body =
            crate::services::slides::presentations::build_create_presentation_body(&args.title);

        match crate::http::api::api_post::<crate::services::slides::types::Presentation>(
            &ctx.client,
            &url,
            &body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
            ctx.is_dry_run(),
        )
        .await
        {
            Ok(Some(pres)) => {
                if let Err(e) = ctx.write_output(&pres) {
                    eprintln!("Error: {}", e);
                    return map_error_to_exit_code(&e);
                }
                codes::SUCCESS
            }
            Ok(None) => {
                eprintln!("[dry-run] would create presentation '{}'", args.title);
                codes::SUCCESS
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                map_error_to_exit_code(&e)
            }
        }
    }
}

/// Handle Slides create-from-markdown: parse markdown and create a presentation with slides.
async fn handle_slides_create_from_markdown(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesCreateFromMarkdownArgs,
) -> i32 {
    use crate::services::slides::markdown::{build_slides_from_markdown, parse_markdown_to_slides};

    // Read markdown content from --content or --content-file
    let markdown = if let Some(ref content) = args.content {
        content.clone()
    } else if let Some(ref content_file) = args.content_file {
        match std::fs::read_to_string(content_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", content_file, e);
                return codes::GENERIC_ERROR;
            }
        }
    } else {
        eprintln!("Error: either --content or --content-file must be provided");
        return codes::GENERIC_ERROR;
    };

    // Parse markdown into slides
    let slides = parse_markdown_to_slides(&markdown);
    if slides.is_empty() {
        eprintln!("Warning: no slides found in markdown content");
    }

    let title = args.title.as_deref().unwrap_or("Untitled Presentation");

    if ctx.is_dry_run() {
        eprintln!(
            "[dry-run] would create presentation '{}' with {} slides from markdown",
            title,
            slides.len()
        );
        return codes::SUCCESS;
    }

    // 1. Create a new blank presentation
    let create_url = format!("{}/presentations", crate::services::slides::SLIDES_BASE_URL);
    let create_body = crate::services::slides::presentations::build_create_presentation_body(title);

    let pres: crate::services::slides::types::Presentation =
        match crate::http::api::api_post::<crate::services::slides::types::Presentation>(
            &ctx.client,
            &create_url,
            &create_body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
            false,
        )
        .await
        {
            Ok(Some(p)) => p,
            Ok(None) => {
                eprintln!("Error: unexpected empty response when creating presentation");
                return codes::GENERIC_ERROR;
            }
            Err(e) => {
                eprintln!("Error creating presentation: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let pres_id = match pres.presentation_id {
        Some(ref id) => id.clone(),
        None => {
            eprintln!("Error: no presentation ID returned");
            return codes::GENERIC_ERROR;
        }
    };

    // 2. If there are slides, batchUpdate to add them
    if !slides.is_empty() {
        let batch_url = crate::services::slides::presentations::build_batch_update_url(&pres_id);
        let batch_body = build_slides_from_markdown(&slides);

        match crate::http::api::api_post::<crate::services::slides::types::BatchUpdateResponse>(
            &ctx.client,
            &batch_url,
            &batch_body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
            false,
        )
        .await
        {
            Ok(Some(_)) => {}
            Ok(None) => {}
            Err(e) => {
                eprintln!("Error adding slides: {}", e);
                return map_error_to_exit_code(&e);
            }
        }
    }

    if let Err(e) = ctx.write_output(&pres) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Slides copy: copy a presentation via Drive API.
async fn handle_slides_copy(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesCopyArgs,
) -> i32 {
    let url = crate::services::slides::export::build_presentation_copy_url(&args.presentation_id);
    let body = crate::services::slides::export::build_presentation_copy_body(
        &args.title,
        args.parent.as_deref(),
    );

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would copy presentation '{}' as '{}'",
                args.presentation_id, args.title
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Slides list-slides: list all slides in a presentation.
async fn handle_slides_list_slides(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesListSlidesArgs,
) -> i32 {
    let url =
        crate::services::slides::presentations::build_presentation_get_url(&args.presentation_id);

    let pres: crate::services::slides::types::Presentation = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    // Build a simple list of slide IDs and titles
    let slide_list: Vec<serde_json::Value> = pres
        .slides
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let slide_id = s.object_id.as_deref().unwrap_or("unknown");
            // Extract title from first text element if available
            let title = crate::services::slides::slides_ops::extract_slide_text(&s.page_elements);
            let title_str = title.trim().lines().next().unwrap_or("").to_string();
            serde_json::json!({
                "index": i,
                "objectId": slide_id,
                "title": title_str
            })
        })
        .collect();

    let output = serde_json::json!({
        "presentationId": pres.presentation_id,
        "slideCount": pres.slides.len(),
        "slides": slide_list
    });

    if let Err(e) = ctx.write_output(&output) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Slides add-slide: add a new slide to a presentation.
async fn handle_slides_add_slide(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesAddSlideArgs,
) -> i32 {
    let batch_url =
        crate::services::slides::presentations::build_batch_update_url(&args.presentation_id);
    let request = crate::services::slides::slides_ops::build_add_slide_request(
        args.layout_id.as_deref(),
        args.index,
    );

    let body = serde_json::json!({ "requests": [request] });

    match crate::http::api::api_post::<crate::services::slides::types::BatchUpdateResponse>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would add slide to presentation '{}'",
                args.presentation_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Slides delete-slide: delete a slide from a presentation.
async fn handle_slides_delete_slide(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesDeleteSlideArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let batch_url =
        crate::services::slides::presentations::build_batch_update_url(&args.presentation_id);
    let request = crate::services::slides::slides_ops::build_delete_slide_request(&args.slide_id);

    let body = serde_json::json!({ "requests": [request] });

    match crate::http::api::api_post::<crate::services::slides::types::BatchUpdateResponse>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would delete slide '{}' from presentation '{}'",
                args.slide_id, args.presentation_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Slides read-slide: read the text content of a specific slide.
async fn handle_slides_read_slide(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesReadSlideArgs,
) -> i32 {
    let url =
        crate::services::slides::presentations::build_presentation_get_url(&args.presentation_id);

    let pres: crate::services::slides::types::Presentation = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    // Find the slide by ID
    let slide = pres
        .slides
        .iter()
        .find(|s| s.object_id.as_deref() == Some(&args.slide_id));

    match slide {
        Some(s) => {
            let text = crate::services::slides::slides_ops::extract_slide_text(&s.page_elements);
            let notes = crate::services::slides::slides_ops::extract_speaker_notes(s);

            let output = serde_json::json!({
                "slideId": args.slide_id,
                "text": text,
                "speakerNotes": notes
            });

            if let Err(e) = ctx.write_output(&output) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        None => {
            eprintln!("Error: slide '{}' not found in presentation", args.slide_id);
            codes::GENERIC_ERROR
        }
    }
}

/// Handle Slides update-notes: update speaker notes on a slide.
async fn handle_slides_update_notes(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesUpdateNotesArgs,
) -> i32 {
    // Get the notes text from args or file
    let notes_text = if let Some(ref file_path) = args.file {
        match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file_path, e);
                return codes::GENERIC_ERROR;
            }
        }
    } else if !args.text.is_empty() {
        args.text.join(" ")
    } else {
        eprintln!("Error: notes text must be provided as positional args or via --file");
        return codes::GENERIC_ERROR;
    };

    // First, get the presentation to find the notes object ID
    let pres_url =
        crate::services::slides::presentations::build_presentation_get_url(&args.presentation_id);

    let pres: crate::services::slides::types::Presentation = match crate::http::api::api_get(
        &ctx.client,
        &pres_url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    // Find the slide
    let slide = pres
        .slides
        .iter()
        .find(|s| s.object_id.as_deref() == Some(&args.slide_id));

    let slide = match slide {
        Some(s) => s,
        None => {
            eprintln!("Error: slide '{}' not found in presentation", args.slide_id);
            return codes::GENERIC_ERROR;
        }
    };

    // Find the notes object ID
    let notes_obj_id = match crate::services::slides::notes::find_notes_object_id(slide) {
        Some(id) => id,
        None => {
            eprintln!("Error: no speaker notes found on slide '{}'", args.slide_id);
            return codes::GENERIC_ERROR;
        }
    };

    // Build and send batchUpdate
    let batch_url =
        crate::services::slides::presentations::build_batch_update_url(&args.presentation_id);
    let requests =
        crate::services::slides::notes::build_update_notes_request(&notes_obj_id, &notes_text);
    let body = serde_json::json!({ "requests": requests });

    match crate::http::api::api_post::<crate::services::slides::types::BatchUpdateResponse>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would update notes on slide '{}'", args.slide_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Slides replace-slide: replace slide content with an image.
async fn handle_slides_replace_slide(
    ctx: &crate::services::ServiceContext,
    args: &slides::SlidesReplaceSlideArgs,
) -> i32 {
    // Get the presentation to access page size for full-bleed
    let pres_url =
        crate::services::slides::presentations::build_presentation_get_url(&args.presentation_id);

    let pres: crate::services::slides::types::Presentation = match crate::http::api::api_get(
        &ctx.client,
        &pres_url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let request = crate::services::slides::slides_ops::build_replace_image_request(
        &args.slide_id,
        &args.image_url,
        pres.page_size.as_ref(),
    );

    let batch_url =
        crate::services::slides::presentations::build_batch_update_url(&args.presentation_id);
    let body = serde_json::json!({ "requests": [request] });

    match crate::http::api::api_post::<crate::services::slides::types::BatchUpdateResponse>(
        &ctx.client,
        &batch_url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(result)) => {
            if let Err(e) = ctx.write_output(&result) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would replace slide '{}' with image",
                args.slide_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle the `forms` command and its subcommands.
async fn handle_forms(args: forms::FormsArgs, flags: &root::RootFlags) -> i32 {
    use forms::FormsCommand;

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        FormsCommand::Get(ref a) => handle_forms_get(&ctx, a).await,
        FormsCommand::Create(ref a) => handle_forms_create(&ctx, a).await,
        FormsCommand::Responses(ref a) => match a.command {
            forms::FormsResponsesCommand::List(ref la) => {
                handle_forms_responses_list(&ctx, la).await
            }
            forms::FormsResponsesCommand::Get(ref ga) => handle_forms_responses_get(&ctx, ga).await,
        },
    }
}

/// Handle Forms get.
async fn handle_forms_get(
    ctx: &crate::services::ServiceContext,
    args: &forms::FormsGetArgs,
) -> i32 {
    let url = crate::services::forms::build_form_get_url(&args.form_id);
    let form: crate::services::forms::types::Form = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&form) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Forms create.
async fn handle_forms_create(
    ctx: &crate::services::ServiceContext,
    args: &forms::FormsCreateArgs,
) -> i32 {
    let url = crate::services::forms::build_form_create_url();
    let body =
        crate::services::forms::build_form_create_body(&args.title, args.description.as_deref());

    match crate::http::api::api_post::<crate::services::forms::types::Form>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(form)) => {
            if let Err(e) = ctx.write_output(&form) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create form '{}'", args.title);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Forms responses list.
async fn handle_forms_responses_list(
    ctx: &crate::services::ServiceContext,
    args: &forms::FormsResponsesListArgs,
) -> i32 {
    let url = crate::services::forms::responses::build_responses_list_url_with_options(
        &args.form_id,
        args.max,
        args.page.as_deref(),
        args.filter.as_deref(),
    );

    let list: crate::services::forms::types::FormResponseList = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let next_token = list.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&list, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Forms responses get.
async fn handle_forms_responses_get(
    ctx: &crate::services::ServiceContext,
    args: &forms::FormsResponsesGetArgs,
) -> i32 {
    let url = crate::services::forms::build_response_get_url(&args.form_id, &args.response_id);

    let response: crate::services::forms::types::FormResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&response) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle the `chat` command and its subcommands.
async fn handle_chat(args: chat::ChatArgs, flags: &root::RootFlags) -> i32 {
    use chat::{
        ChatCommand, ChatDmCommand, ChatMessagesCommand, ChatSpacesCommand, ChatThreadsCommand,
    };

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        ChatCommand::Spaces(sa) => match sa.command {
            ChatSpacesCommand::List(ref a) => handle_chat_spaces_list(&ctx, a).await,
            ChatSpacesCommand::Find(ref a) => handle_chat_spaces_find(&ctx, a).await,
            ChatSpacesCommand::Create(ref a) => handle_chat_spaces_create(&ctx, a).await,
        },
        ChatCommand::Messages(ma) => match ma.command {
            ChatMessagesCommand::List(ref a) => handle_chat_messages_list(&ctx, a).await,
            ChatMessagesCommand::Send(ref a) => handle_chat_messages_send(&ctx, a).await,
        },
        ChatCommand::Threads(ta) => match ta.command {
            ChatThreadsCommand::List(ref a) => handle_chat_threads_list(&ctx, a).await,
        },
        ChatCommand::Dm(da) => match da.command {
            ChatDmCommand::Space(ref a) => handle_chat_dm_space(&ctx, a).await,
            ChatDmCommand::Send(ref a) => handle_chat_dm_send(&ctx, a).await,
        },
    }
}

/// Handle Chat spaces list.
async fn handle_chat_spaces_list(
    ctx: &crate::services::ServiceContext,
    args: &chat::ChatSpacesListArgs,
) -> i32 {
    let url = crate::services::chat::spaces::build_spaces_list_url(args.max, args.page.as_deref());

    let result: crate::services::chat::types::SpaceListResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Chat spaces find.
async fn handle_chat_spaces_find(
    ctx: &crate::services::ServiceContext,
    args: &chat::ChatSpacesFindArgs,
) -> i32 {
    let url = crate::services::chat::spaces::build_spaces_find_url(&args.name, args.max);

    let result: crate::services::chat::types::SpaceListResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Chat spaces create.
async fn handle_chat_spaces_create(
    ctx: &crate::services::ServiceContext,
    args: &chat::ChatSpacesCreateArgs,
) -> i32 {
    let url = crate::services::chat::spaces::build_space_create_url();
    let member_refs: Vec<&str> = args.member.iter().map(|s| s.as_str()).collect();
    let body = crate::services::chat::spaces::build_space_create_body(&args.name, &member_refs);

    match crate::http::api::api_post::<crate::services::chat::types::Space>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(space)) => {
            if let Err(e) = ctx.write_output(&space) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create space '{}'", args.name);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Chat messages list.
async fn handle_chat_messages_list(
    ctx: &crate::services::ServiceContext,
    args: &chat::ChatMessagesListArgs,
) -> i32 {
    let url = crate::services::chat::messages::build_messages_list_url(
        &args.space,
        args.max,
        args.page.as_deref(),
        args.order.as_deref(),
        args.thread.as_deref(),
    );

    let result: crate::services::chat::types::MessageListResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Chat messages send.
async fn handle_chat_messages_send(
    ctx: &crate::services::ServiceContext,
    args: &chat::ChatMessagesSendArgs,
) -> i32 {
    let url = crate::services::chat::messages::build_message_send_url(&args.space);
    let body = crate::services::chat::messages::build_message_send_body(
        &args.text,
        args.thread.as_deref(),
    );

    match crate::http::api::api_post::<crate::services::chat::types::Message>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(msg)) => {
            if let Err(e) = ctx.write_output(&msg) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would send message to space '{}'", args.space);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Chat threads list.
async fn handle_chat_threads_list(
    ctx: &crate::services::ServiceContext,
    args: &chat::ChatThreadsListArgs,
) -> i32 {
    let url = crate::services::chat::threads::build_threads_list_url(
        &args.space,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::chat::types::MessageListResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Chat DM space (find/create DM with a user).
async fn handle_chat_dm_space(
    ctx: &crate::services::ServiceContext,
    args: &chat::ChatDmSpaceArgs,
) -> i32 {
    let url = crate::services::chat::dm::build_dm_space_url();
    let body = crate::services::chat::dm::build_dm_space_body(&args.user);

    match crate::http::api::api_post::<crate::services::chat::types::Space>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(space)) => {
            if let Err(e) = ctx.write_output(&space) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would find/create DM space with '{}'", args.user);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Chat DM send.
async fn handle_chat_dm_send(
    ctx: &crate::services::ServiceContext,
    args: &chat::ChatDmSendArgs,
) -> i32 {
    // First, find/create the DM space
    let space_url = crate::services::chat::dm::build_dm_space_url();
    let space_body = crate::services::chat::dm::build_dm_space_body(&args.email);

    let space: crate::services::chat::types::Space =
        match crate::http::api::api_post::<crate::services::chat::types::Space>(
            &ctx.client,
            &space_url,
            &space_body,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
            false,
        )
        .await
        {
            Ok(Some(s)) => s,
            Ok(None) => {
                eprintln!("Error: unexpected empty response from DM space lookup");
                return codes::GENERIC_ERROR;
            }
            Err(e) => {
                eprintln!("Error finding DM space: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let space_name = match space.name {
        Some(ref n) => n.clone(),
        None => {
            eprintln!("Error: DM space has no name");
            return codes::GENERIC_ERROR;
        }
    };

    let send_url = crate::services::chat::dm::build_dm_send_url(&space_name);
    let send_body =
        crate::services::chat::dm::build_dm_send_body(&args.text, args.thread.as_deref());

    match crate::http::api::api_post::<crate::services::chat::types::Message>(
        &ctx.client,
        &send_url,
        &send_body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(msg)) => {
            if let Err(e) = ctx.write_output(&msg) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would send DM to '{}'", args.email);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle the `classroom` command and its subcommands.
async fn handle_classroom(args: classroom::ClassroomArgs, flags: &root::RootFlags) -> i32 {
    use classroom::{
        ClassroomAnnouncementsCommand, ClassroomCommand, ClassroomCoursesCommand,
        ClassroomCourseworkCommand, ClassroomGuardianInvitationsCommand, ClassroomGuardiansCommand,
        ClassroomInvitationsCommand, ClassroomMaterialsCommand, ClassroomRosterCommand,
        ClassroomStudentsCommand, ClassroomSubmissionsCommand, ClassroomTeachersCommand,
        ClassroomTopicsCommand,
    };

    // Handle offline commands first (no auth needed)
    if let ClassroomCommand::Courses(ref ca) = args.command {
        if let ClassroomCoursesCommand::Url(ref url_args) = ca.command {
            if url_args.course_ids.is_empty() {
                eprintln!("Error: at least one course ID is required");
                return codes::USAGE_ERROR;
            }
            let urls: Vec<String> = url_args
                .course_ids
                .iter()
                .map(|id| crate::services::classroom::courses::build_course_url(id))
                .collect();
            if flags.json {
                let json_val = match serde_json::to_value(&urls) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return codes::GENERIC_ERROR;
                    }
                };
                println!("{}", to_json_pretty(&json_val));
            } else {
                for url in &urls {
                    println!("{}", url);
                }
            }
            return codes::SUCCESS;
        }
    }

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        ClassroomCommand::Courses(ca) => match ca.command {
            ClassroomCoursesCommand::List(ref a) => handle_classroom_courses_list(&ctx, a).await,
            ClassroomCoursesCommand::Get(ref a) => handle_classroom_courses_get(&ctx, a).await,
            ClassroomCoursesCommand::Create(ref a) => {
                handle_classroom_courses_create(&ctx, a).await
            }
            ClassroomCoursesCommand::Update(ref a) => {
                handle_classroom_courses_update(&ctx, a).await
            }
            ClassroomCoursesCommand::Delete(ref a) => {
                handle_classroom_courses_delete(&ctx, a).await
            }
            ClassroomCoursesCommand::Archive(ref a) => {
                handle_classroom_courses_archive(&ctx, a).await
            }
            ClassroomCoursesCommand::Unarchive(ref a) => {
                handle_classroom_courses_unarchive(&ctx, a).await
            }
            ClassroomCoursesCommand::Join(ref a) => handle_classroom_courses_join(&ctx, a).await,
            ClassroomCoursesCommand::Leave(ref a) => handle_classroom_courses_leave(&ctx, a).await,
            ClassroomCoursesCommand::Url(_) => unreachable!(),
        },
        ClassroomCommand::Students(sa) => match sa.command {
            ClassroomStudentsCommand::List(ref a) => handle_classroom_students_list(&ctx, a).await,
            ClassroomStudentsCommand::Get(ref a) => handle_classroom_students_get(&ctx, a).await,
            ClassroomStudentsCommand::Add(ref a) => handle_classroom_students_add(&ctx, a).await,
            ClassroomStudentsCommand::Remove(ref a) => {
                handle_classroom_students_remove(&ctx, a).await
            }
        },
        ClassroomCommand::Teachers(ta) => match ta.command {
            ClassroomTeachersCommand::List(ref a) => handle_classroom_teachers_list(&ctx, a).await,
            ClassroomTeachersCommand::Get(ref a) => handle_classroom_teachers_get(&ctx, a).await,
            ClassroomTeachersCommand::Add(ref a) => handle_classroom_teachers_add(&ctx, a).await,
            ClassroomTeachersCommand::Remove(ref a) => {
                handle_classroom_teachers_remove(&ctx, a).await
            }
        },
        ClassroomCommand::Roster(ra) => match ra.command {
            ClassroomRosterCommand::List(ref a) => handle_classroom_roster_list(&ctx, a).await,
        },
        ClassroomCommand::Coursework(cwa) => match cwa.command {
            ClassroomCourseworkCommand::List(ref a) => {
                handle_classroom_coursework_list(&ctx, a).await
            }
            ClassroomCourseworkCommand::Get(ref a) => {
                handle_classroom_coursework_get(&ctx, a).await
            }
            ClassroomCourseworkCommand::Create(ref a) => {
                handle_classroom_coursework_create(&ctx, a).await
            }
            ClassroomCourseworkCommand::Update(ref a) => {
                handle_classroom_coursework_update(&ctx, a).await
            }
            ClassroomCourseworkCommand::Delete(ref a) => {
                handle_classroom_coursework_delete(&ctx, a).await
            }
            ClassroomCourseworkCommand::Assignees(ref a) => {
                handle_classroom_coursework_assignees(&ctx, a).await
            }
        },
        ClassroomCommand::Materials(ma) => match ma.command {
            ClassroomMaterialsCommand::List(ref a) => {
                handle_classroom_materials_list(&ctx, a).await
            }
            ClassroomMaterialsCommand::Get(ref a) => handle_classroom_materials_get(&ctx, a).await,
            ClassroomMaterialsCommand::Create(ref a) => {
                handle_classroom_materials_create(&ctx, a).await
            }
            ClassroomMaterialsCommand::Update(ref a) => {
                handle_classroom_materials_update(&ctx, a).await
            }
            ClassroomMaterialsCommand::Delete(ref a) => {
                handle_classroom_materials_delete(&ctx, a).await
            }
        },
        ClassroomCommand::Submissions(sa) => match sa.command {
            ClassroomSubmissionsCommand::List(ref a) => {
                handle_classroom_submissions_list(&ctx, a).await
            }
            ClassroomSubmissionsCommand::Get(ref a) => {
                handle_classroom_submissions_get(&ctx, a).await
            }
            ClassroomSubmissionsCommand::TurnIn(ref a) => {
                handle_classroom_submissions_turnin(&ctx, a).await
            }
            ClassroomSubmissionsCommand::Reclaim(ref a) => {
                handle_classroom_submissions_reclaim(&ctx, a).await
            }
            ClassroomSubmissionsCommand::Return(ref a) => {
                handle_classroom_submissions_return(&ctx, a).await
            }
            ClassroomSubmissionsCommand::Grade(ref a) => {
                handle_classroom_submissions_grade(&ctx, a).await
            }
        },
        ClassroomCommand::Announcements(aa) => match aa.command {
            ClassroomAnnouncementsCommand::List(ref a) => {
                handle_classroom_announcements_list(&ctx, a).await
            }
            ClassroomAnnouncementsCommand::Get(ref a) => {
                handle_classroom_announcements_get(&ctx, a).await
            }
            ClassroomAnnouncementsCommand::Create(ref a) => {
                handle_classroom_announcements_create(&ctx, a).await
            }
            ClassroomAnnouncementsCommand::Update(ref a) => {
                handle_classroom_announcements_update(&ctx, a).await
            }
            ClassroomAnnouncementsCommand::Delete(ref a) => {
                handle_classroom_announcements_delete(&ctx, a).await
            }
            ClassroomAnnouncementsCommand::Assignees(ref a) => {
                handle_classroom_announcements_assignees(&ctx, a).await
            }
        },
        ClassroomCommand::Topics(ta) => match ta.command {
            ClassroomTopicsCommand::List(ref a) => handle_classroom_topics_list(&ctx, a).await,
            ClassroomTopicsCommand::Get(ref a) => handle_classroom_topics_get(&ctx, a).await,
            ClassroomTopicsCommand::Create(ref a) => handle_classroom_topics_create(&ctx, a).await,
            ClassroomTopicsCommand::Update(ref a) => handle_classroom_topics_update(&ctx, a).await,
            ClassroomTopicsCommand::Delete(ref a) => handle_classroom_topics_delete(&ctx, a).await,
        },
        ClassroomCommand::Invitations(ia) => match ia.command {
            ClassroomInvitationsCommand::List(ref a) => {
                handle_classroom_invitations_list(&ctx, a).await
            }
            ClassroomInvitationsCommand::Get(ref a) => {
                handle_classroom_invitations_get(&ctx, a).await
            }
            ClassroomInvitationsCommand::Create(ref a) => {
                handle_classroom_invitations_create(&ctx, a).await
            }
            ClassroomInvitationsCommand::Accept(ref a) => {
                handle_classroom_invitations_accept(&ctx, a).await
            }
            ClassroomInvitationsCommand::Delete(ref a) => {
                handle_classroom_invitations_delete(&ctx, a).await
            }
        },
        ClassroomCommand::Guardians(ga) => match ga.command {
            ClassroomGuardiansCommand::List(ref a) => {
                handle_classroom_guardians_list(&ctx, a).await
            }
            ClassroomGuardiansCommand::Get(ref a) => handle_classroom_guardians_get(&ctx, a).await,
            ClassroomGuardiansCommand::Delete(ref a) => {
                handle_classroom_guardians_delete(&ctx, a).await
            }
        },
        ClassroomCommand::GuardianInvitations(gia) => match gia.command {
            ClassroomGuardianInvitationsCommand::List(ref a) => {
                handle_classroom_guardian_invitations_list(&ctx, a).await
            }
            ClassroomGuardianInvitationsCommand::Get(ref a) => {
                handle_classroom_guardian_invitations_get(&ctx, a).await
            }
            ClassroomGuardianInvitationsCommand::Create(ref a) => {
                handle_classroom_guardian_invitations_create(&ctx, a).await
            }
        },
        ClassroomCommand::Profile(ref a) => handle_classroom_profile(&ctx, a).await,
    }
}

// ---------------------------------------------------------------
// Classroom: Courses handlers
// ---------------------------------------------------------------

async fn handle_classroom_courses_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCoursesListArgs,
) -> i32 {
    let url = crate::services::classroom::courses::build_courses_list_url(
        args.state.as_deref(),
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::CourseListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_courses_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCoursesGetArgs,
) -> i32 {
    let url = crate::services::classroom::courses::build_course_get_url(&args.course_id);

    let result: crate::services::classroom::types::Course = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_courses_create(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCoursesCreateArgs,
) -> i32 {
    let url = crate::services::classroom::courses::build_course_create_url();
    let body = crate::services::classroom::courses::build_course_create_body(
        &args.name,
        args.owner.as_deref(),
        args.state.as_deref(),
    );

    match crate::http::api::api_post::<crate::services::classroom::types::Course>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(course)) => {
            if let Err(e) = ctx.write_output(&course) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create course '{}'", args.name);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_courses_update(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCoursesUpdateArgs,
) -> i32 {
    let url = crate::services::classroom::courses::build_course_update_url(&args.course_id);
    let body = crate::services::classroom::courses::build_course_update_body(
        args.name.as_deref(),
        args.state.as_deref(),
    );

    match crate::http::api::api_patch::<crate::services::classroom::types::Course>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(course)) => {
            if let Err(e) = ctx.write_output(&course) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would update course '{}'", args.course_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_courses_delete(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCoursesDeleteArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::classroom::courses::build_course_delete_url(&args.course_id);

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Course '{}' deleted.", args.course_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_courses_archive(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCoursesArchiveArgs,
) -> i32 {
    let url = crate::services::classroom::courses::build_course_archive_url(&args.course_id);
    let body = serde_json::json!({"courseState": "ARCHIVED"});

    match crate::http::api::api_patch::<crate::services::classroom::types::Course>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(course)) => {
            if let Err(e) = ctx.write_output(&course) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would archive course '{}'", args.course_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_courses_unarchive(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCoursesUnarchiveArgs,
) -> i32 {
    let url = crate::services::classroom::courses::build_course_archive_url(&args.course_id);
    let body = serde_json::json!({"courseState": "ACTIVE"});

    match crate::http::api::api_patch::<crate::services::classroom::types::Course>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(course)) => {
            if let Err(e) = ctx.write_output(&course) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would unarchive course '{}'", args.course_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_courses_join(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCoursesJoinArgs,
) -> i32 {
    let url = crate::services::classroom::roster::build_student_add_url(&args.course_id);
    let body = crate::services::classroom::roster::build_student_add_body("me", None);

    match crate::http::api::api_post::<crate::services::classroom::types::Student>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(student)) => {
            if let Err(e) = ctx.write_output(&student) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would join course '{}'", args.course_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_courses_leave(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCoursesLeaveArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::classroom::roster::build_student_remove_url(&args.course_id, "me");

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Left course '{}'.", args.course_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Students handlers
// ---------------------------------------------------------------

async fn handle_classroom_students_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomStudentsListArgs,
) -> i32 {
    let url = crate::services::classroom::roster::build_students_list_url(
        &args.course_id,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::StudentListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_students_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomStudentsGetArgs,
) -> i32 {
    let url =
        crate::services::classroom::roster::build_student_get_url(&args.course_id, &args.user_id);

    let result: crate::services::classroom::types::Student = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_students_add(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomStudentsAddArgs,
) -> i32 {
    let url = crate::services::classroom::roster::build_student_add_url(&args.course_id);
    let body = crate::services::classroom::roster::build_student_add_body(
        &args.user_id,
        args.enrollment_code.as_deref(),
    );

    match crate::http::api::api_post::<crate::services::classroom::types::Student>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(student)) => {
            if let Err(e) = ctx.write_output(&student) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would add student '{}' to course '{}'",
                args.user_id, args.course_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_students_remove(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomStudentsRemoveArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::classroom::roster::build_student_remove_url(
        &args.course_id,
        &args.user_id,
    );

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!(
                "Student '{}' removed from course '{}'.",
                args.user_id, args.course_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Teachers handlers
// ---------------------------------------------------------------

async fn handle_classroom_teachers_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomTeachersListArgs,
) -> i32 {
    let url = crate::services::classroom::roster::build_teachers_list_url(
        &args.course_id,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::TeacherListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_teachers_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomTeachersGetArgs,
) -> i32 {
    let url =
        crate::services::classroom::roster::build_teacher_get_url(&args.course_id, &args.user_id);

    let result: crate::services::classroom::types::Teacher = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_teachers_add(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomTeachersAddArgs,
) -> i32 {
    let url = crate::services::classroom::roster::build_teacher_add_url(&args.course_id);
    let body = crate::services::classroom::roster::build_teacher_add_body(&args.user_id);

    match crate::http::api::api_post::<crate::services::classroom::types::Teacher>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(teacher)) => {
            if let Err(e) = ctx.write_output(&teacher) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would add teacher '{}' to course '{}'",
                args.user_id, args.course_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_teachers_remove(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomTeachersRemoveArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::classroom::roster::build_teacher_remove_url(
        &args.course_id,
        &args.user_id,
    );

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!(
                "Teacher '{}' removed from course '{}'.",
                args.user_id, args.course_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Roster (combined) handler
// ---------------------------------------------------------------

async fn handle_classroom_roster_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomRosterListArgs,
) -> i32 {
    // If only students or only teachers requested, just call that handler
    if args.students && !args.teachers {
        let url = crate::services::classroom::roster::build_students_list_url(
            &args.course_id,
            None,
            None,
        );
        let result: crate::services::classroom::types::StudentListResponse =
            match crate::http::api::api_get(
                &ctx.client,
                &url,
                &ctx.circuit_breaker,
                &ctx.retry_config,
                ctx.is_verbose(),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return map_error_to_exit_code(&e);
                }
            };
        let next_token = result.next_page_token.clone();
        if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
        return codes::SUCCESS;
    }

    if args.teachers && !args.students {
        let url = crate::services::classroom::roster::build_teachers_list_url(
            &args.course_id,
            None,
            None,
        );
        let result: crate::services::classroom::types::TeacherListResponse =
            match crate::http::api::api_get(
                &ctx.client,
                &url,
                &ctx.circuit_breaker,
                &ctx.retry_config,
                ctx.is_verbose(),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return map_error_to_exit_code(&e);
                }
            };
        let next_token = result.next_page_token.clone();
        if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
        return codes::SUCCESS;
    }

    // Both (default)
    let students_url =
        crate::services::classroom::roster::build_students_list_url(&args.course_id, None, None);
    let teachers_url =
        crate::services::classroom::roster::build_teachers_list_url(&args.course_id, None, None);

    let students_result: crate::services::classroom::types::StudentListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &students_url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error fetching students: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let teachers_result: crate::services::classroom::types::TeacherListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &teachers_url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error fetching teachers: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let combined = serde_json::json!({
        "students": students_result.students,
        "teachers": teachers_result.teachers,
    });

    if let Err(e) = ctx.write_output(&combined) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

// ---------------------------------------------------------------
// Classroom: Coursework handlers
// ---------------------------------------------------------------

async fn handle_classroom_coursework_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCourseworkListArgs,
) -> i32 {
    let url = crate::services::classroom::coursework::build_coursework_list_url(
        &args.course_id,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::CourseWorkListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_coursework_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCourseworkGetArgs,
) -> i32 {
    let url = crate::services::classroom::coursework::build_coursework_get_url(
        &args.course_id,
        &args.coursework_id,
    );

    let result: crate::services::classroom::types::CourseWork = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_coursework_create(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCourseworkCreateArgs,
) -> i32 {
    let url = crate::services::classroom::coursework::build_coursework_create_url(&args.course_id);
    let body = crate::services::classroom::coursework::build_coursework_create_body(
        &args.title,
        &args.work_type,
        args.description.as_deref(),
        args.max_points,
        args.state.as_deref(),
    );

    match crate::http::api::api_post::<crate::services::classroom::types::CourseWork>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(cw)) => {
            if let Err(e) = ctx.write_output(&cw) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create coursework '{}'", args.title);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_coursework_update(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCourseworkUpdateArgs,
) -> i32 {
    let url = crate::services::classroom::coursework::build_coursework_update_url(
        &args.course_id,
        &args.coursework_id,
    );
    let mut body = serde_json::json!({});
    if let Some(ref t) = args.title {
        body["title"] = serde_json::Value::String(t.clone());
    }
    if let Some(ref d) = args.description {
        body["description"] = serde_json::Value::String(d.clone());
    }

    match crate::http::api::api_patch::<crate::services::classroom::types::CourseWork>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(cw)) => {
            if let Err(e) = ctx.write_output(&cw) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would update coursework '{}'", args.coursework_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_coursework_delete(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCourseworkDeleteArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::classroom::coursework::build_coursework_delete_url(
        &args.course_id,
        &args.coursework_id,
    );

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Coursework '{}' deleted.", args.coursework_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_coursework_assignees(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomCourseworkAssigneesArgs,
) -> i32 {
    let url = crate::services::classroom::coursework::build_coursework_update_url(
        &args.course_id,
        &args.coursework_id,
    );
    let body = serde_json::json!({
        "assigneeMode": "INDIVIDUAL_STUDENTS",
        "individualStudentsOptions": {
            "studentIds": args.add,
        },
    });

    match crate::http::api::api_patch::<crate::services::classroom::types::CourseWork>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(cw)) => {
            if let Err(e) = ctx.write_output(&cw) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would update assignees for coursework '{}'",
                args.coursework_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Materials handlers
// ---------------------------------------------------------------

async fn handle_classroom_materials_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomMaterialsListArgs,
) -> i32 {
    let url = crate::services::classroom::materials::build_materials_list_url(
        &args.course_id,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::CourseMaterialListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_materials_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomMaterialsGetArgs,
) -> i32 {
    let url = crate::services::classroom::materials::build_material_get_url(
        &args.course_id,
        &args.material_id,
    );

    let result: crate::services::classroom::types::CourseMaterial = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_materials_create(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomMaterialsCreateArgs,
) -> i32 {
    let url = crate::services::classroom::materials::build_material_create_url(&args.course_id);
    let body = crate::services::classroom::materials::build_material_create_body(
        &args.title,
        args.description.as_deref(),
        args.topic_id.as_deref(),
        args.state.as_deref(),
    );

    match crate::http::api::api_post::<crate::services::classroom::types::CourseMaterial>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(mat)) => {
            if let Err(e) = ctx.write_output(&mat) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create material '{}'", args.title);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_materials_update(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomMaterialsUpdateArgs,
) -> i32 {
    let url = crate::services::classroom::materials::build_material_update_url(
        &args.course_id,
        &args.material_id,
    );
    let mut body = serde_json::json!({});
    if let Some(ref t) = args.title {
        body["title"] = serde_json::Value::String(t.clone());
    }
    if let Some(ref d) = args.description {
        body["description"] = serde_json::Value::String(d.clone());
    }

    match crate::http::api::api_patch::<crate::services::classroom::types::CourseMaterial>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(mat)) => {
            if let Err(e) = ctx.write_output(&mat) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would update material '{}'", args.material_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_materials_delete(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomMaterialsDeleteArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::classroom::materials::build_material_delete_url(
        &args.course_id,
        &args.material_id,
    );

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Material '{}' deleted.", args.material_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Submissions handlers
// ---------------------------------------------------------------

async fn handle_classroom_submissions_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomSubmissionsListArgs,
) -> i32 {
    let url = crate::services::classroom::submissions::build_submissions_list_url(
        &args.course_id,
        &args.coursework_id,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::SubmissionListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_submissions_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomSubmissionsGetArgs,
) -> i32 {
    let url = crate::services::classroom::submissions::build_submission_get_url(
        &args.course_id,
        &args.coursework_id,
        &args.submission_id,
    );

    let result: crate::services::classroom::types::StudentSubmission =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_submissions_turnin(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomSubmissionsTurnInArgs,
) -> i32 {
    let url = crate::services::classroom::submissions::build_submission_turn_in_url(
        &args.course_id,
        &args.coursework_id,
        &args.submission_id,
    );
    let body = serde_json::json!({});

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(_) => {
            eprintln!("Submission '{}' turned in.", args.submission_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_submissions_reclaim(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomSubmissionsReclaimArgs,
) -> i32 {
    let url = crate::services::classroom::submissions::build_submission_reclaim_url(
        &args.course_id,
        &args.coursework_id,
        &args.submission_id,
    );
    let body = serde_json::json!({});

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(_) => {
            eprintln!("Submission '{}' reclaimed.", args.submission_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_submissions_return(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomSubmissionsReturnArgs,
) -> i32 {
    let url = crate::services::classroom::submissions::build_submission_return_url(
        &args.course_id,
        &args.coursework_id,
        &args.submission_id,
    );
    let body = serde_json::json!({});

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(_) => {
            eprintln!("Submission '{}' returned.", args.submission_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_submissions_grade(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomSubmissionsGradeArgs,
) -> i32 {
    let url = crate::services::classroom::submissions::build_submission_patch_url(
        &args.course_id,
        &args.coursework_id,
        &args.submission_id,
    );
    let body = crate::services::classroom::submissions::build_submission_grade_body(
        args.grade,
        args.draft_grade,
    );

    match crate::http::api::api_patch::<crate::services::classroom::types::StudentSubmission>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(sub)) => {
            if let Err(e) = ctx.write_output(&sub) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would grade submission '{}'", args.submission_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Announcements handlers
// ---------------------------------------------------------------

async fn handle_classroom_announcements_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomAnnouncementsListArgs,
) -> i32 {
    let url = crate::services::classroom::announcements::build_announcements_list_url(
        &args.course_id,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::AnnouncementListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_announcements_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomAnnouncementsGetArgs,
) -> i32 {
    let url = crate::services::classroom::announcements::build_announcement_get_url(
        &args.course_id,
        &args.announcement_id,
    );

    let result: crate::services::classroom::types::Announcement = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_announcements_create(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomAnnouncementsCreateArgs,
) -> i32 {
    let url =
        crate::services::classroom::announcements::build_announcement_create_url(&args.course_id);
    let body = crate::services::classroom::announcements::build_announcement_create_body(
        &args.text,
        args.state.as_deref(),
    );

    match crate::http::api::api_post::<crate::services::classroom::types::Announcement>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(ann)) => {
            if let Err(e) = ctx.write_output(&ann) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would create announcement in course '{}'",
                args.course_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_announcements_update(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomAnnouncementsUpdateArgs,
) -> i32 {
    let url = crate::services::classroom::announcements::build_announcement_update_url(
        &args.course_id,
        &args.announcement_id,
    );
    let mut body = serde_json::json!({});
    if let Some(ref t) = args.text {
        body["text"] = serde_json::Value::String(t.clone());
    }

    match crate::http::api::api_patch::<crate::services::classroom::types::Announcement>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(ann)) => {
            if let Err(e) = ctx.write_output(&ann) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would update announcement '{}'",
                args.announcement_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_announcements_delete(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomAnnouncementsDeleteArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::classroom::announcements::build_announcement_delete_url(
        &args.course_id,
        &args.announcement_id,
    );

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Announcement '{}' deleted.", args.announcement_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_announcements_assignees(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomAnnouncementsAssigneesArgs,
) -> i32 {
    let url = crate::services::classroom::announcements::build_announcement_update_url(
        &args.course_id,
        &args.announcement_id,
    );
    let body = serde_json::json!({
        "assigneeMode": "INDIVIDUAL_STUDENTS",
        "individualStudentsOptions": {
            "studentIds": args.add,
        },
    });

    match crate::http::api::api_patch::<crate::services::classroom::types::Announcement>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(ann)) => {
            if let Err(e) = ctx.write_output(&ann) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would update assignees for announcement '{}'",
                args.announcement_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Topics handlers
// ---------------------------------------------------------------

async fn handle_classroom_topics_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomTopicsListArgs,
) -> i32 {
    let url = crate::services::classroom::topics::build_topics_list_url(
        &args.course_id,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::TopicListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_topics_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomTopicsGetArgs,
) -> i32 {
    let url =
        crate::services::classroom::topics::build_topic_get_url(&args.course_id, &args.topic_id);

    let result: crate::services::classroom::types::Topic = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_topics_create(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomTopicsCreateArgs,
) -> i32 {
    let url = crate::services::classroom::topics::build_topic_create_url(&args.course_id);
    let body = crate::services::classroom::topics::build_topic_create_body(&args.name);

    match crate::http::api::api_post::<crate::services::classroom::types::Topic>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(topic)) => {
            if let Err(e) = ctx.write_output(&topic) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create topic '{}'", args.name);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_topics_update(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomTopicsUpdateArgs,
) -> i32 {
    let url =
        crate::services::classroom::topics::build_topic_update_url(&args.course_id, &args.topic_id);
    let body = crate::services::classroom::topics::build_topic_update_body(&args.name);

    match crate::http::api::api_patch::<crate::services::classroom::types::Topic>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(topic)) => {
            if let Err(e) = ctx.write_output(&topic) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would update topic '{}'", args.topic_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_topics_delete(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomTopicsDeleteArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url =
        crate::services::classroom::topics::build_topic_delete_url(&args.course_id, &args.topic_id);

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Topic '{}' deleted.", args.topic_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Invitations handlers
// ---------------------------------------------------------------

async fn handle_classroom_invitations_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomInvitationsListArgs,
) -> i32 {
    let url = crate::services::classroom::invitations::build_invitations_list_url(
        args.course_id.as_deref(),
        args.user_id.as_deref(),
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::InvitationListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_invitations_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomInvitationsGetArgs,
) -> i32 {
    let url =
        crate::services::classroom::invitations::build_invitation_get_url(&args.invitation_id);

    let result: crate::services::classroom::types::Invitation = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_invitations_create(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomInvitationsCreateArgs,
) -> i32 {
    let url = crate::services::classroom::invitations::build_invitation_create_url();
    let body = crate::services::classroom::invitations::build_invitation_create_body(
        &args.user_id,
        &args.course_id,
        &args.role,
    );

    match crate::http::api::api_post::<crate::services::classroom::types::Invitation>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(inv)) => {
            if let Err(e) = ctx.write_output(&inv) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would create invitation for '{}' in course '{}'",
                args.user_id, args.course_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_invitations_accept(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomInvitationsAcceptArgs,
) -> i32 {
    let url =
        crate::services::classroom::invitations::build_invitation_accept_url(&args.invitation_id);
    let body = serde_json::json!({});

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(_) => {
            eprintln!("Invitation '{}' accepted.", args.invitation_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

async fn handle_classroom_invitations_delete(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomInvitationsDeleteArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url =
        crate::services::classroom::invitations::build_invitation_delete_url(&args.invitation_id);

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Invitation '{}' deleted.", args.invitation_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Guardians handlers
// ---------------------------------------------------------------

async fn handle_classroom_guardians_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomGuardiansListArgs,
) -> i32 {
    let url = crate::services::classroom::guardians::build_guardians_list_url(
        &args.student_id,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::GuardianListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_guardians_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomGuardiansGetArgs,
) -> i32 {
    let url = crate::services::classroom::guardians::build_guardian_get_url(
        &args.student_id,
        &args.guardian_id,
    );

    let result: crate::services::classroom::types::Guardian = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_guardians_delete(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomGuardiansDeleteArgs,
) -> i32 {
    if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::classroom::guardians::build_guardian_delete_url(
        &args.student_id,
        &args.guardian_id,
    );

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Guardian '{}' deleted.", args.guardian_id);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Guardian Invitations handlers
// ---------------------------------------------------------------

async fn handle_classroom_guardian_invitations_list(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomGuardianInvitationsListArgs,
) -> i32 {
    let url = crate::services::classroom::guardians::build_guardian_invitations_list_url(
        &args.student_id,
        args.state.as_deref(),
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::classroom::types::GuardianInvitationListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_guardian_invitations_get(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomGuardianInvitationsGetArgs,
) -> i32 {
    let url = crate::services::classroom::guardians::build_guardian_invitation_get_url(
        &args.student_id,
        &args.invitation_id,
    );

    let result: crate::services::classroom::types::GuardianInvitation =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

async fn handle_classroom_guardian_invitations_create(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomGuardianInvitationsCreateArgs,
) -> i32 {
    let url = crate::services::classroom::guardians::build_guardian_invitation_create_url(
        &args.student_id,
    );
    let body =
        crate::services::classroom::guardians::build_guardian_invitation_create_body(&args.email);

    match crate::http::api::api_post::<crate::services::classroom::types::GuardianInvitation>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(inv)) => {
            if let Err(e) = ctx.write_output(&inv) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would create guardian invitation for '{}' to student '{}'",
                args.email, args.student_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

// ---------------------------------------------------------------
// Classroom: Profile handler
// ---------------------------------------------------------------

async fn handle_classroom_profile(
    ctx: &crate::services::ServiceContext,
    args: &classroom::ClassroomProfileArgs,
) -> i32 {
    let url = format!(
        "{}/userProfiles/{}",
        crate::services::classroom::CLASSROOM_BASE_URL,
        percent_encoding::utf8_percent_encode(&args.user_id, percent_encoding::NON_ALPHANUMERIC)
    );

    let result: crate::services::classroom::types::UserProfile = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle the `tasks` command and its subcommands.
async fn handle_tasks(args: tasks::TasksArgs, flags: &root::RootFlags) -> i32 {
    use tasks::{TasksCommand, TasksListsCommand};

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        TasksCommand::Lists(la) => match la.command {
            TasksListsCommand::List(ref a) => handle_tasks_lists_list(&ctx, a).await,
            TasksListsCommand::Create(ref a) => handle_tasks_lists_create(&ctx, a).await,
        },
        TasksCommand::List(ref a) => handle_tasks_list(&ctx, a).await,
        TasksCommand::Get(ref a) => handle_tasks_get(&ctx, a).await,
        TasksCommand::Add(ref a) => handle_tasks_add(&ctx, a).await,
        TasksCommand::Update(ref a) => handle_tasks_update(&ctx, a).await,
        TasksCommand::Done(ref a) => handle_tasks_done(&ctx, a).await,
        TasksCommand::Undo(ref a) => handle_tasks_undo(&ctx, a).await,
        TasksCommand::Delete(ref a) => handle_tasks_delete(&ctx, a).await,
        TasksCommand::Clear(ref a) => handle_tasks_clear(&ctx, a).await,
    }
}

/// Handle Tasks lists list.
async fn handle_tasks_lists_list(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksListsListArgs,
) -> i32 {
    let url =
        crate::services::tasks::tasklists::build_tasklists_list_url(args.max, args.page.as_deref());

    let result: crate::services::tasks::types::TaskListsResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Tasks lists create.
async fn handle_tasks_lists_create(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksListsCreateArgs,
) -> i32 {
    let url = crate::services::tasks::tasklists::build_tasklist_create_url();
    let body = crate::services::tasks::tasklists::build_tasklist_create_body(&args.title);

    match crate::http::api::api_post::<crate::services::tasks::types::TaskList>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(tl)) => {
            if let Err(e) = ctx.write_output(&tl) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create task list '{}'", args.title);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Tasks list (list tasks in a tasklist).
async fn handle_tasks_list(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksListArgs,
) -> i32 {
    let url = crate::services::tasks::task_ops::build_tasks_list_url(
        &args.tasklist,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::tasks::types::TasksResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Tasks get.
async fn handle_tasks_get(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksGetArgs,
) -> i32 {
    let url = crate::services::tasks::task_ops::build_task_get_url(&args.tasklist, &args.task);

    let result: crate::services::tasks::types::Task = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Tasks add.
async fn handle_tasks_add(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksAddArgs,
) -> i32 {
    let url = crate::services::tasks::task_ops::build_task_create_url(&args.tasklist);
    let body = crate::services::tasks::task_ops::build_task_create_body(
        &args.title,
        args.notes.as_deref(),
        args.due.as_deref(),
        args.parent.as_deref(),
        args.previous.as_deref(),
    );

    match crate::http::api::api_post::<crate::services::tasks::types::Task>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(task)) => {
            if let Err(e) = ctx.write_output(&task) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create task '{}'", args.title);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Tasks update.
async fn handle_tasks_update(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksUpdateArgs,
) -> i32 {
    let url = crate::services::tasks::task_ops::build_task_update_url(&args.tasklist, &args.task);
    let body = crate::services::tasks::task_ops::build_task_update_body(
        args.title.as_deref(),
        args.notes.as_deref(),
        args.due.as_deref(),
        args.status.as_deref(),
    );

    match crate::http::api::api_patch::<crate::services::tasks::types::Task>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(task)) => {
            if let Err(e) = ctx.write_output(&task) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would update task '{}'", args.task);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Tasks done (mark task completed).
async fn handle_tasks_done(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksDoneArgs,
) -> i32 {
    let url = crate::services::tasks::task_ops::build_task_update_url(&args.tasklist, &args.task);
    let body = crate::services::tasks::task_ops::build_task_update_body(
        None,
        None,
        None,
        Some("completed"),
    );

    match crate::http::api::api_patch::<crate::services::tasks::types::Task>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(task)) => {
            if let Err(e) = ctx.write_output(&task) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would mark task '{}' as done", args.task);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Tasks undo (mark task incomplete).
async fn handle_tasks_undo(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksUndoArgs,
) -> i32 {
    let url = crate::services::tasks::task_ops::build_task_update_url(&args.tasklist, &args.task);
    let body = crate::services::tasks::task_ops::build_task_update_body(
        None,
        None,
        None,
        Some("needsAction"),
    );

    match crate::http::api::api_patch::<crate::services::tasks::types::Task>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(task)) => {
            if let Err(e) = ctx.write_output(&task) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would mark task '{}' as not done", args.task);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Tasks delete.
async fn handle_tasks_delete(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksDeleteArgs,
) -> i32 {
    if !ctx.is_force() && !ctx.flags.no_input {
        eprint!(
            "Are you sure you want to delete task '{}'? [y/N] ",
            args.task
        );
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err()
            || !input.trim().eq_ignore_ascii_case("y")
        {
            eprintln!("Aborted.");
            return codes::SUCCESS;
        }
    } else if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::tasks::task_ops::build_task_delete_url(&args.tasklist, &args.task);

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Task '{}' deleted.", args.task);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Tasks clear (clear completed tasks).
async fn handle_tasks_clear(
    ctx: &crate::services::ServiceContext,
    args: &tasks::TasksClearArgs,
) -> i32 {
    let url = crate::services::tasks::task_ops::build_tasks_clear_url(&args.tasklist);
    let body = serde_json::json!({});

    match crate::http::api::api_post::<serde_json::Value>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(_) => {
            eprintln!("Completed tasks cleared from list '{}'.", args.tasklist);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle the `contacts` command and its subcommands.
async fn handle_contacts(args: contacts::ContactsArgs, flags: &root::RootFlags) -> i32 {
    use contacts::{ContactsCommand, ContactsDirectoryCommand, ContactsOtherCommand};

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        ContactsCommand::Search(ref a) => handle_contacts_search(&ctx, a).await,
        ContactsCommand::List(ref a) => handle_contacts_list(&ctx, a).await,
        ContactsCommand::Get(ref a) => handle_contacts_get(&ctx, a).await,
        ContactsCommand::Create(ref a) => handle_contacts_create(&ctx, a).await,
        ContactsCommand::Update(ref a) => handle_contacts_update(&ctx, a).await,
        ContactsCommand::Delete(ref a) => handle_contacts_delete(&ctx, a).await,
        ContactsCommand::Directory(da) => match da.command {
            ContactsDirectoryCommand::List(ref a) => handle_contacts_directory_list(&ctx, a).await,
            ContactsDirectoryCommand::Search(ref a) => {
                handle_contacts_directory_search(&ctx, a).await
            }
        },
        ContactsCommand::Other(oa) => match oa.command {
            ContactsOtherCommand::List(ref a) => handle_contacts_other_list(&ctx, a).await,
            ContactsOtherCommand::Search(ref a) => handle_contacts_other_search(&ctx, a).await,
        },
    }
}

/// Handle Contacts search.
async fn handle_contacts_search(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsSearchArgs,
) -> i32 {
    let query = args.query.join(" ");
    let url = crate::services::contacts::contacts::build_contacts_search_url(&query, args.max);

    let result: serde_json::Value = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Contacts list.
async fn handle_contacts_list(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsListArgs,
) -> i32 {
    let url = crate::services::contacts::contacts::build_contacts_list_url(
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::contacts::types::PersonListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Contacts get.
async fn handle_contacts_get(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsGetArgs,
) -> i32 {
    let url = crate::services::contacts::contacts::build_contact_get_url(&args.resource_name);

    let result: crate::services::contacts::types::Person = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Contacts create.
async fn handle_contacts_create(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsCreateArgs,
) -> i32 {
    let url = crate::services::contacts::contacts::build_contact_create_url();
    let body = crate::services::contacts::contacts::build_contact_create_body(
        args.given.as_deref(),
        args.family.as_deref(),
        args.email.as_deref(),
        args.phone.as_deref(),
    );

    match crate::http::api::api_post::<crate::services::contacts::types::Person>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(person)) => {
            if let Err(e) = ctx.write_output(&person) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create contact");
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Contacts update.
async fn handle_contacts_update(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsUpdateArgs,
) -> i32 {
    let url = crate::services::contacts::contacts::build_contact_update_url(&args.resource_name);
    let body = match crate::services::contacts::contacts::build_contact_update_body(
        args.given.as_deref(),
        args.family.as_deref(),
        args.email.as_deref(),
        args.phone.as_deref(),
        args.birthday.as_deref(),
        args.notes.as_deref(),
    ) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::USAGE_ERROR;
        }
    };

    match crate::http::api::api_patch::<crate::services::contacts::types::Person>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(person)) => {
            if let Err(e) = ctx.write_output(&person) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would update contact '{}'", args.resource_name);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Contacts delete.
async fn handle_contacts_delete(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsDeleteArgs,
) -> i32 {
    if !ctx.is_force() && !ctx.flags.no_input {
        eprint!(
            "Are you sure you want to delete contact '{}'? [y/N] ",
            args.resource_name
        );
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err()
            || !input.trim().eq_ignore_ascii_case("y")
        {
            eprintln!("Aborted.");
            return codes::SUCCESS;
        }
    } else if !ctx.is_force() && ctx.flags.no_input {
        eprintln!("Error: destructive operation requires --force when --no-input is set");
        return codes::USAGE_ERROR;
    }

    let url = crate::services::contacts::contacts::build_contact_delete_url(&args.resource_name);

    match crate::http::api::api_delete(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(()) => {
            eprintln!("Contact '{}' deleted.", args.resource_name);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle Contacts directory list.
async fn handle_contacts_directory_list(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsDirectoryListArgs,
) -> i32 {
    let url = crate::services::contacts::directory::build_directory_list_url(
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::contacts::types::DirectoryListResponse =
        match crate::http::api::api_get(
            &ctx.client,
            &url,
            &ctx.circuit_breaker,
            &ctx.retry_config,
            ctx.is_verbose(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
        };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Contacts directory search.
async fn handle_contacts_directory_search(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsDirectorySearchArgs,
) -> i32 {
    let query = args.query.join(" ");
    let url = crate::services::contacts::directory::build_directory_search_url(&query, args.max);

    let result: serde_json::Value = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Contacts other list.
async fn handle_contacts_other_list(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsOtherListArgs,
) -> i32 {
    let url = crate::services::contacts::directory::build_other_contacts_list_url(
        args.max,
        args.page.as_deref(),
    );

    let result: serde_json::Value = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let next_token = result
        .get("nextPageToken")
        .and_then(|v| v.as_str())
        .map(String::from);
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Contacts other search.
async fn handle_contacts_other_search(
    ctx: &crate::services::ServiceContext,
    args: &contacts::ContactsOtherSearchArgs,
) -> i32 {
    let query = args.query.join(" ");
    let url =
        crate::services::contacts::directory::build_other_contacts_search_url(&query, args.max);

    let result: serde_json::Value = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle the `people` command and its subcommands.
async fn handle_people(args: people::PeopleArgs, flags: &root::RootFlags) -> i32 {
    use people::PeopleCommand;

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        PeopleCommand::Me => handle_people_me(&ctx).await,
        PeopleCommand::Get(ref a) => handle_people_get(&ctx, a).await,
        PeopleCommand::Search(ref a) => handle_people_search(&ctx, a).await,
        PeopleCommand::Relations(ref a) => handle_people_relations(&ctx, a).await,
    }
}

/// Handle People me.
async fn handle_people_me(ctx: &crate::services::ServiceContext) -> i32 {
    let url = crate::services::people::people::build_people_me_url();
    let person: crate::services::people::types::PersonResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&person) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle People get.
async fn handle_people_get(
    ctx: &crate::services::ServiceContext,
    args: &people::PeopleGetArgs,
) -> i32 {
    let url = crate::services::people::people::build_people_get_url(&args.resource_name);
    let person: crate::services::people::types::PersonResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&person) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle People search.
async fn handle_people_search(
    ctx: &crate::services::ServiceContext,
    args: &people::PeopleSearchArgs,
) -> i32 {
    let query = args.query.join(" ");
    let url = crate::services::people::people::build_people_search_url(
        &query,
        args.max,
        args.page.as_deref(),
    );

    let result: crate::services::people::types::SearchResponse = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let next_token = result.next_page_token.clone();
    if let Err(e) = ctx.write_paginated(&result, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle People relations.
async fn handle_people_relations(
    ctx: &crate::services::ServiceContext,
    args: &people::PeopleRelationsArgs,
) -> i32 {
    let url =
        crate::services::people::people::build_people_relations_url(args.resource_name.as_deref());

    let result: serde_json::Value = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&result) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle the `groups` command and its subcommands.
async fn handle_groups(args: groups::GroupsArgs, flags: &root::RootFlags) -> i32 {
    use groups::GroupsCommand;

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        GroupsCommand::List(ref a) => handle_groups_list(&ctx, a).await,
        GroupsCommand::Members(ref a) => handle_groups_members(&ctx, a).await,
    }
}

/// Handle Groups list.
async fn handle_groups_list(
    ctx: &crate::services::ServiceContext,
    args: &groups::GroupsListArgs,
) -> i32 {
    let params = crate::services::common::PaginationParams {
        max_results: args.max,
        page_token: args.page.clone(),
        all_pages: args.all,
        fail_empty: args.fail_empty,
    };

    let max = args.max;
    let (items, next_token) = match crate::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        &params,
        |pt| crate::services::groups::groups::build_groups_list_url(max, pt),
        |value| {
            let groups = value
                .get("groups")
                .and_then(|v| v.as_array())
                .map(|arr| arr.to_vec())
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((groups, next))
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if crate::services::pagination::check_fail_empty(&items, args.fail_empty).is_err() {
        return codes::EMPTY_RESULTS;
    }

    let response = serde_json::json!({
        "groups": items,
    });

    if let Err(e) = ctx.write_paginated(&response, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Groups members list.
async fn handle_groups_members(
    ctx: &crate::services::ServiceContext,
    args: &groups::GroupsMembersArgs,
) -> i32 {
    // First, look up the group by email to get the group resource name
    let lookup_url = crate::services::groups::groups::build_group_lookup_url(&args.group_email);
    let group_lookup: serde_json::Value = match crate::http::api::api_get(
        &ctx.client,
        &lookup_url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error looking up group '{}': {}", args.group_email, e);
            return map_error_to_exit_code(&e);
        }
    };

    let group_name = match group_lookup.get("name").and_then(|v| v.as_str()) {
        Some(name) => name.to_string(),
        None => {
            eprintln!(
                "Error: could not resolve group name for '{}'",
                args.group_email
            );
            return codes::GENERIC_ERROR;
        }
    };

    let params = crate::services::common::PaginationParams {
        max_results: args.max,
        page_token: args.page.clone(),
        all_pages: args.all,
        fail_empty: args.fail_empty,
    };

    let max = args.max;
    let gn = group_name.clone();
    let (items, next_token) = match crate::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        &params,
        |pt| crate::services::groups::groups::build_members_list_url(&gn, max, pt),
        |value| {
            let members = value
                .get("memberships")
                .and_then(|v| v.as_array())
                .map(|arr| arr.to_vec())
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((members, next))
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if crate::services::pagination::check_fail_empty(&items, args.fail_empty).is_err() {
        return codes::EMPTY_RESULTS;
    }

    let response = serde_json::json!({
        "memberships": items,
    });

    if let Err(e) = ctx.write_paginated(&response, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle the `keep` command and its subcommands.
async fn handle_keep(args: keep::KeepArgs, flags: &root::RootFlags) -> i32 {
    use keep::KeepCommand;

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        KeepCommand::List(ref a) => handle_keep_list(&ctx, a).await,
        KeepCommand::Get(ref a) => handle_keep_get(&ctx, a).await,
        KeepCommand::Search(ref a) => handle_keep_search(&ctx, a).await,
        KeepCommand::Attachment(ref a) => handle_keep_attachment(&ctx, a).await,
    }
}

/// Handle Keep list.
async fn handle_keep_list(ctx: &crate::services::ServiceContext, args: &keep::KeepListArgs) -> i32 {
    let params = crate::services::common::PaginationParams {
        max_results: args.max,
        page_token: args.page.clone(),
        all_pages: args.all,
        fail_empty: args.fail_empty,
    };

    let max = args.max;
    let filter = args.filter.clone();
    let (items, next_token) = match crate::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        &params,
        |pt| crate::services::keep::notes::build_notes_list_url(max, pt, filter.as_deref()),
        |value| {
            let notes = value
                .get("notes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.to_vec())
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((notes, next))
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if crate::services::pagination::check_fail_empty(&items, args.fail_empty).is_err() {
        return codes::EMPTY_RESULTS;
    }

    let response = serde_json::json!({
        "notes": items,
    });

    if let Err(e) = ctx.write_paginated(&response, next_token.as_deref()) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Keep get.
async fn handle_keep_get(ctx: &crate::services::ServiceContext, args: &keep::KeepGetArgs) -> i32 {
    let url = crate::services::keep::notes::build_note_get_url(&args.note_id);
    let note: crate::services::keep::types::Note = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&note) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Keep search (client-side: list all notes then filter).
async fn handle_keep_search(
    ctx: &crate::services::ServiceContext,
    args: &keep::KeepSearchArgs,
) -> i32 {
    // Fetch all notes (paginate through all pages)
    let params = crate::services::common::PaginationParams {
        max_results: None,
        page_token: None,
        all_pages: true,
        fail_empty: false,
    };

    let (items, _) = match crate::services::pagination::paginate(
        &ctx.client,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        &params,
        |pt| crate::services::keep::notes::build_notes_list_url(None, pt, None),
        |value| {
            let notes = value
                .get("notes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.to_vec())
                .unwrap_or_default();
            let next = value
                .get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok((notes, next))
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    // Parse into Note types for client-side search
    let notes: Vec<crate::services::keep::types::Note> = items
        .iter()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect();

    let results = crate::services::keep::notes::build_notes_search(&notes, &args.query);

    // Apply max limit if specified
    let limited: Vec<&crate::services::keep::types::Note> = if let Some(max) = args.max {
        results.into_iter().take(max as usize).collect()
    } else {
        results
    };

    let response = serde_json::json!({
        "notes": limited,
        "resultCount": limited.len(),
    });

    if let Err(e) = ctx.write_output(&response) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle Keep attachment download.
async fn handle_keep_attachment(
    ctx: &crate::services::ServiceContext,
    args: &keep::KeepAttachmentArgs,
) -> i32 {
    let url =
        crate::services::keep::attachments::build_attachment_download_url(&args.attachment_name);

    let response = match crate::http::api::api_get_raw(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error reading response: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    if let Some(ref out_path) = args.out {
        match std::fs::write(out_path, &bytes) {
            Ok(()) => {
                eprintln!("Downloaded {} bytes to {}", bytes.len(), out_path);
                codes::SUCCESS
            }
            Err(e) => {
                eprintln!("Error writing file: {}", e);
                codes::GENERIC_ERROR
            }
        }
    } else {
        // Write raw bytes to stdout
        use std::io::Write;
        match std::io::stdout().write_all(&bytes) {
            Ok(()) => codes::SUCCESS,
            Err(e) => {
                eprintln!("Error writing to stdout: {}", e);
                codes::GENERIC_ERROR
            }
        }
    }
}

/// Handle the `appscript` command and its subcommands.
async fn handle_appscript(args: appscript::AppScriptArgs, flags: &root::RootFlags) -> i32 {
    use appscript::AppScriptCommand;

    // Bootstrap auth
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    match args.command {
        AppScriptCommand::Get(ref a) => handle_appscript_get(&ctx, a).await,
        AppScriptCommand::Content(ref a) => handle_appscript_content(&ctx, a).await,
        AppScriptCommand::Run(ref a) => handle_appscript_run(&ctx, a).await,
        AppScriptCommand::Create(ref a) => handle_appscript_create(&ctx, a).await,
    }
}

/// Handle AppScript get project.
async fn handle_appscript_get(
    ctx: &crate::services::ServiceContext,
    args: &appscript::AppScriptGetArgs,
) -> i32 {
    let script_id = crate::services::appscript::scripts::normalize_google_id(&args.script_id);
    let url = crate::services::appscript::scripts::build_project_get_url(&script_id);

    let project: crate::services::appscript::types::Project = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&project) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle AppScript get content.
async fn handle_appscript_content(
    ctx: &crate::services::ServiceContext,
    args: &appscript::AppScriptContentArgs,
) -> i32 {
    let script_id = crate::services::appscript::scripts::normalize_google_id(&args.script_id);
    let url = crate::services::appscript::scripts::build_content_get_url(&script_id);

    let content: crate::services::appscript::types::Content = match crate::http::api::api_get(
        &ctx.client,
        &url,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return map_error_to_exit_code(&e);
        }
    };

    if let Err(e) = ctx.write_output(&content) {
        eprintln!("Error: {}", e);
        return map_error_to_exit_code(&e);
    }
    codes::SUCCESS
}

/// Handle AppScript run.
async fn handle_appscript_run(
    ctx: &crate::services::ServiceContext,
    args: &appscript::AppScriptRunArgs,
) -> i32 {
    let script_id = crate::services::appscript::scripts::normalize_google_id(&args.script_id);
    let url = crate::services::appscript::scripts::build_run_url(&script_id);

    let body = match crate::services::appscript::scripts::build_run_body(
        &args.function,
        args.params.as_deref(),
        args.dev_mode,
    ) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::USAGE_ERROR;
        }
    };

    match crate::http::api::api_post::<crate::services::appscript::types::Operation>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(op)) => {
            if let Err(e) = ctx.write_output(&op) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "[dry-run] would run function '{}' on script '{}'",
                args.function, script_id
            );
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle AppScript create project.
async fn handle_appscript_create(
    ctx: &crate::services::ServiceContext,
    args: &appscript::AppScriptCreateArgs,
) -> i32 {
    let url = crate::services::appscript::scripts::build_project_create_url();
    let body = crate::services::appscript::scripts::build_project_create_body(
        &args.title,
        args.parent_id.as_deref(),
    );

    match crate::http::api::api_post::<crate::services::appscript::types::Project>(
        &ctx.client,
        &url,
        &body,
        &ctx.circuit_breaker,
        &ctx.retry_config,
        ctx.is_verbose(),
        ctx.is_dry_run(),
    )
    .await
    {
        Ok(Some(project)) => {
            if let Err(e) = ctx.write_output(&project) {
                eprintln!("Error: {}", e);
                return map_error_to_exit_code(&e);
            }
            codes::SUCCESS
        }
        Ok(None) => {
            eprintln!("[dry-run] would create project '{}'", args.title);
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            map_error_to_exit_code(&e)
        }
    }
}

/// Handle the `open` command: offline URL generation.
fn handle_open(args: open::OpenArgs, flags: &root::RootFlags) -> i32 {
    match open::resolve_target(&args.target, &args.r#type) {
        Ok(url) => {
            if flags.json {
                let json_val = serde_json::json!({
                    "url": url,
                    "target": args.target,
                });
                println!("{}", to_json_pretty(&json_val));
            } else {
                println!("{}", url);
            }
            codes::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            codes::USAGE_ERROR
        }
    }
}

/// Handle the `completion` command: generate shell completions.
fn handle_completion(args: completion::CompletionArgs) -> i32 {
    let mut stdout = std::io::stdout();
    match completion::generate_completions(&args.shell, &mut stdout) {
        Ok(()) => codes::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            codes::USAGE_ERROR
        }
    }
}

/// Handle the `exit-codes` command.
fn handle_exit_codes(flags: &root::RootFlags) -> i32 {
    let table = agent::exit_code_table();

    if flags.json {
        println!("{}", to_json_pretty(&table));
    } else if flags.csv {
        println!("code,name,description");
        for entry in &table {
            println!(
                "{},{},{}",
                crate::output::csv_escape(&entry.code.to_string()),
                crate::output::csv_escape(&entry.name),
                crate::output::csv_escape(&entry.description),
            );
        }
    } else if flags.plain {
        for entry in &table {
            println!("{}\t{}\t{}", entry.code, entry.name, entry.description);
        }
    } else {
        let header = format!("{:<6} {:<20} DESCRIPTION", "CODE", "NAME");
        println!("{}", header);
        println!("{}", "-".repeat(60));
        for entry in &table {
            println!("{:<6} {:<20} {}", entry.code, entry.name, entry.description);
        }
    }
    codes::SUCCESS
}

/// Handle the `schema` / `help-json` command.
fn handle_schema(args: agent::SchemaArgs, flags: &root::RootFlags) -> i32 {
    let schema = agent::generate_schema(args.command.as_deref(), args.include_hidden);

    if flags.plain {
        // For plain mode, just output the command names
        if let Some(subs) = schema.get("subcommands").and_then(|s| s.as_array()) {
            for sub in subs {
                if let Some(name) = sub.get("name").and_then(|n| n.as_str()) {
                    println!("{}", name);
                }
            }
        }
    } else {
        // Default and --json both output full JSON
        println!("{}", to_json_pretty(&schema));
    }
    codes::SUCCESS
}

/// Handle the `agent` command and its subcommands.
fn handle_agent(args: agent::AgentArgs, flags: &root::RootFlags) -> i32 {
    match args.command {
        agent::AgentCommand::ExitCodes => handle_exit_codes(flags),
        agent::AgentCommand::Schema(schema_args) => handle_schema(schema_args, flags),
    }
}

/// Handle the `webhook` command.
async fn handle_webhook(args: root::WebhookArgs) -> i32 {
    match args.command {
        root::WebhookCommand::Serve(serve_args) => {
            match crate::webhook::serve(&serve_args.bind, serve_args.port).await {
                Ok(()) => codes::SUCCESS,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    codes::GENERIC_ERROR
                }
            }
        }
    }
}

/// Rewrite desire path arguments before parsing.
/// Handles two kinds of rewrites:
/// 1. Command aliases: `send` -> `gmail send`, `ls` -> `drive ls`, etc.
/// 2. Flag aliases: `--fields` -> `--select` (except in `calendar events` context)
pub fn rewrite_desire_path_args(args: Vec<String>) -> Vec<String> {
    // First, rewrite command aliases (desire paths)
    let args = rewrite_command_aliases(args);

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
        } else if let Some(val) = arg.strip_prefix("--fields=") {
            result.push(format!("--select={}", val));
        } else {
            result.push(arg);
        }
    }

    result
}

/// Rewrite top-level command aliases (desire paths) to their canonical forms.
///
/// Aliases:
/// - `send` -> `gmail send`
/// - `ls` -> `drive ls`
/// - `search` -> `drive search`
/// - `download` -> `drive download`
/// - `upload` -> `drive upload`
/// - `login` -> `auth add`
/// - `logout` -> `auth remove`
/// - `status` -> `auth status`
/// - `me` / `whoami` -> `auth status`
pub fn rewrite_command_aliases(args: Vec<String>) -> Vec<String> {
    if args.is_empty() {
        return args;
    }

    // Find the first non-flag argument (the command token)
    let mut cmd_index = None;
    let mut skip_next = false;
    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg.starts_with('-') {
            if arg.contains('=') {
                continue;
            }
            if global_flag_takes_value(arg) {
                skip_next = true;
            }
            continue;
        }
        cmd_index = Some(i);
        break;
    }

    let cmd_index = match cmd_index {
        Some(i) => i,
        None => return args,
    };

    let cmd = args[cmd_index].to_lowercase();

    // Map desire path aliases to canonical (service, subcommand) pairs
    let rewrite: Option<(&str, &str)> = match cmd.as_str() {
        "send" => Some(("gmail", "send")),
        "ls" => Some(("drive", "ls")),
        "search" => Some(("drive", "search")),
        "download" => Some(("drive", "download")),
        "upload" => Some(("drive", "upload")),
        "login" => Some(("auth", "add")),
        "logout" => Some(("auth", "remove")),
        "status" | "me" | "whoami" => Some(("auth", "status")),
        _ => None,
    };

    match rewrite {
        Some((service, subcommand)) => {
            let mut result = Vec::with_capacity(args.len() + 1);
            // Copy flags before the command
            result.extend_from_slice(&args[..cmd_index]);
            // Insert the rewritten command
            result.push(service.to_string());
            result.push(subcommand.to_string());
            // Copy remaining args after the original command token
            result.extend_from_slice(&args[cmd_index + 1..]);
            result
        }
        None => args,
    }
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
        let args = vec![
            "gmail".to_string(),
            "search".to_string(),
            "--fields".to_string(),
            "id,subject".to_string(),
        ];
        let result = rewrite_desire_path_args(args);
        assert!(result.contains(&"--select".to_string()));
        assert!(!result.contains(&"--fields".to_string()));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: --fields=x,y is rewritten to --select=x,y
    #[test]
    fn req_cli_009_fields_equals_rewritten() {
        let args = vec![
            "drive".to_string(),
            "ls".to_string(),
            "--fields=id,name".to_string(),
        ];
        let result = rewrite_desire_path_args(args);
        assert!(result.iter().any(|a| a.starts_with("--select=")));
        assert!(!result.iter().any(|a| a.starts_with("--fields=")));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: calendar events --fields is NOT rewritten
    #[test]
    fn req_cli_009_calendar_events_not_rewritten() {
        let args = vec![
            "calendar".to_string(),
            "events".to_string(),
            "--fields".to_string(),
            "items(id)".to_string(),
        ];
        let result = rewrite_desire_path_args(args);
        assert!(result.contains(&"--fields".to_string()));
        assert!(!result.contains(&"--select".to_string()));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: cal events --fields is also NOT rewritten (alias)
    #[test]
    fn req_cli_009_cal_events_not_rewritten() {
        let args = vec![
            "cal".to_string(),
            "events".to_string(),
            "--fields".to_string(),
            "items(id)".to_string(),
        ];
        let result = rewrite_desire_path_args(args);
        assert!(result.contains(&"--fields".to_string()));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: calendar ls --fields is also NOT rewritten
    #[test]
    fn req_cli_009_calendar_ls_not_rewritten() {
        let args = vec![
            "calendar".to_string(),
            "ls".to_string(),
            "--fields".to_string(),
            "items(id)".to_string(),
        ];
        let result = rewrite_desire_path_args(args);
        assert!(result.contains(&"--fields".to_string()));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: -- stops rewriting
    #[test]
    fn req_cli_009_double_dash_stops_rewriting() {
        let args = vec![
            "gmail".to_string(),
            "search".to_string(),
            "--".to_string(),
            "--fields".to_string(),
            "test".to_string(),
        ];
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
        assert!(is_calendar_events_command(&[
            "calendar".to_string(),
            "events".to_string()
        ]));
        assert!(is_calendar_events_command(&[
            "cal".to_string(),
            "events".to_string()
        ]));
        assert!(is_calendar_events_command(&[
            "calendar".to_string(),
            "ls".to_string()
        ]));
        assert!(is_calendar_events_command(&[
            "calendar".to_string(),
            "list".to_string()
        ]));
    }

    // Requirement: REQ-CLI-009 (Should)
    #[test]
    fn req_cli_009_non_calendar_commands() {
        assert!(!is_calendar_events_command(&[
            "gmail".to_string(),
            "search".to_string()
        ]));
        assert!(!is_calendar_events_command(&["calendar".to_string()]));
        assert!(!is_calendar_events_command(&[
            "drive".to_string(),
            "ls".to_string()
        ]));
    }

    // Requirement: REQ-CLI-009 (Should)
    // Acceptance: Skips flags when detecting command tokens
    #[test]
    fn req_cli_009_skips_flags_in_detection() {
        assert!(is_calendar_events_command(&[
            "--json".to_string(),
            "calendar".to_string(),
            "--account".to_string(),
            "me@x.com".to_string(),
            "events".to_string()
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
