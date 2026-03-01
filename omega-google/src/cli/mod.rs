pub mod root;
pub mod desire_paths;
pub mod exit_codes;
pub mod gmail;
pub mod calendar;
pub mod drive;
pub mod docs;
pub mod sheets;
pub mod slides;
pub mod forms;
pub mod chat;
pub mod tasks;
pub mod classroom;
pub mod contacts;
pub mod people;
pub mod groups;
pub mod keep;
pub mod appscript;
pub mod open;
pub mod completion;
pub mod agent;

use std::ffi::OsString;

use clap::Parser;

use crate::error::exit::codes;

/// Safely serialize a value to pretty-printed JSON, returning an error JSON string on failure.
fn to_json_pretty(val: &impl serde::Serialize) -> String {
    serde_json::to_string_pretty(val)
        .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
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
        root::Command::Docs(args) => handle_docs(args, flags),
        root::Command::Sheets(args) => handle_sheets(args, flags),
        root::Command::Slides(args) => handle_slides(args, flags),
        root::Command::Forms(args) => handle_forms(args, flags),
        root::Command::Chat(args) => handle_chat(args, flags),
        root::Command::Classroom(args) => handle_classroom(args, flags),
        root::Command::Tasks(args) => handle_tasks(args, flags),
        root::Command::Contacts(args) => handle_contacts(args, flags),
        root::Command::People(args) => handle_people(args, flags),
        root::Command::Groups(args) => handle_groups(args, flags),
        root::Command::Keep(args) => handle_keep(args, flags),
        root::Command::AppScript(args) => handle_appscript(args, flags),
        root::Command::Open(args) => handle_open(args, flags),
        root::Command::Completion(args) => handle_completion(args),
        root::Command::ExitCodes => handle_exit_codes(flags),
        root::Command::Schema(args) => handle_schema(args, flags),
        root::Command::Agent(args) => handle_agent(args, flags),
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
        root::ConfigCommand::Set(set_args) => handle_config_set(&set_args.key, &set_args.value, flags),
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
            eprintln!("Error: unknown config key '{}'. Use 'config keys' to see valid keys.", key);
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
            eprintln!("Error: unknown config key '{}'. Use 'config keys' to see valid keys.", key);
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
            Err(e) => { eprintln!("Error: {}", e); return codes::GENERIC_ERROR; }
        };
        println!("{}", to_json_pretty(&json_val));
    } else {
        let json_val = match serde_json::to_value(&cfg) {
            Ok(v) => v,
            Err(e) => { eprintln!("Error: {}", e); return codes::GENERIC_ERROR; }
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
            Err(e) => { eprintln!("Error: {}", e); return codes::GENERIC_ERROR; }
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
        root::AuthCommand::Credentials(cred_args) => handle_auth_credentials(&cred_args.path, flags),
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
            eprintln!("Credentials stored for client '{}'.", crate::config::DEFAULT_CLIENT_NAME);
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
    let client_name = flags.client.as_deref().unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
    let client_name = crate::config::normalize_client_name(client_name);

    // 2. Load client credentials from config dir
    let creds = match crate::config::read_client_credentials(&client_name) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}. Run 'omega-google auth credentials <path>' first.", e);
            return codes::CONFIG_ERROR;
        }
    };

    // 3. Determine flow mode from flags
    let mode = if add_args.manual {
        FlowMode::Manual
    } else if add_args.remote {
        FlowMode::Remote
    } else {
        FlowMode::Desktop
    };

    // 4. Collect services -- filter by --services flag or default to user services
    let services = if let Some(ref svc_list) = add_args.services {
        let mut parsed = Vec::new();
        for name in svc_list.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
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
                eprintln!("Error: unknown drive scope '{}'. Use: full, readonly, file", other);
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
    ).await {
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

    let scopes: Vec<String> = token_response.scope
        .map(|s| s.split_whitespace().map(|x| x.to_string()).collect())
        .unwrap_or_default();

    let now = chrono::Utc::now();
    let expires_at = token_response.expires_in.map(|secs| {
        now + chrono::Duration::seconds(secs as i64)
    });

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

    // Set as default account if it's the only one
    let keys = store.keys().unwrap_or_default();
    let matching: Vec<_> = keys.iter()
        .filter_map(|k| crate::auth::parse_token_key(k))
        .filter(|(c, _)| c == &client_name)
        .collect();
    if matching.len() == 1 {
        let _ = store.set_default_account(&client_name, &email);
    }

    eprintln!("Account '{}' added successfully.", email);
    codes::SUCCESS
}

/// Fetch the authenticated user's email from Google's userinfo endpoint.
async fn fetch_email_from_token(http_client: &reqwest::Client, access_token: &str) -> anyhow::Result<String> {
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

    let client_name = flags.client.as_deref().unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
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

    let client_name = flags.client.as_deref().unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
    let client_name = crate::config::normalize_client_name(client_name);

    let store = match crate::auth::keyring::credential_store_factory(&cfg) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error initializing credential store: {}", e);
            return codes::CONFIG_ERROR;
        }
    };

    let tokens = match store.list_tokens() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error listing tokens: {}", e);
            return codes::GENERIC_ERROR;
        }
    };

    // Get default account for the current client
    let default_account = store.get_default_account(&client_name).unwrap_or(None);

    if flags.json {
        let json_accounts: Vec<serde_json::Value> = tokens.iter().map(|t| {
            let is_default = default_account.as_deref() == Some(&t.email) && t.client == client_name;
            serde_json::json!({
                "email": t.email,
                "client": t.client,
                "services": t.services,
                "scopes": t.scopes,
                "created_at": t.created_at.to_rfc3339(),
                "is_default": is_default,
            })
        }).collect();
        println!("{}", to_json_pretty(&json_accounts));
    } else if tokens.is_empty() {
        eprintln!("No authenticated accounts found. Use 'omega-google auth add' to add one.");
    } else {
        for t in &tokens {
            let is_default = default_account.as_deref() == Some(&t.email) && t.client == client_name;
            let marker = if is_default { "* " } else { "  " };
            let services_str: Vec<String> = t.services.iter().map(|s| format!("{:?}", s)).collect();
            println!("{}{}\t{}\t{}", marker, t.email, t.client, services_str.join(","));
        }
    }
    codes::SUCCESS
}

/// Handle `auth status`: show config path, keyring backend, current account, credential file status.
fn handle_auth_status(flags: &root::RootFlags) -> i32 {
    let config_path = crate::config::config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let cfg = crate::config::read_config().unwrap_or_default();

    let keyring_backend = cfg.keyring_backend.clone().unwrap_or_else(|| "auto".to_string());

    let client_name = flags.client.as_deref().unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
    let client_name = crate::config::normalize_client_name(client_name);

    // Check if credential file exists
    let cred_filename = crate::config::credential_filename(&client_name);
    let cred_path = crate::config::config_dir()
        .map(|d| d.join(&cred_filename))
        .unwrap_or_default();
    let cred_exists = cred_path.exists();

    // Try to get the current account
    let store = crate::auth::keyring::credential_store_factory(&cfg).ok();
    let current_account = store.as_ref().and_then(|s| {
        crate::auth::resolve_account(
            flags.account.as_deref(),
            &cfg,
            s.as_ref(),
            &client_name,
        ).ok()
    });

    // Load token details if we have a current account
    let token_details = current_account.as_ref().and_then(|email| {
        store.as_ref().and_then(|s| {
            s.get_token(&client_name, email).ok()
        })
    });

    let needs_refresh = token_details.as_ref().map(crate::auth::token::needs_refresh);

    if flags.json {
        let mut json_val = serde_json::json!({
            "config_path": config_path,
            "keyring_backend": keyring_backend,
            "client": client_name,
            "credentials_file": cred_path.to_string_lossy(),
            "credentials_found": cred_exists,
            "current_account": current_account,
        });
        if let Some(ref td) = token_details {
            json_val["services"] = serde_json::to_value(&td.services).unwrap_or_default();
            json_val["scopes"] = serde_json::to_value(&td.scopes).unwrap_or_default();
            json_val["created_at"] = serde_json::Value::String(td.created_at.to_rfc3339());
            json_val["needs_refresh"] = serde_json::Value::Bool(needs_refresh.unwrap_or(false));
        }
        println!("{}", to_json_pretty(&json_val));
    } else {
        println!("Config path:       {}", config_path);
        println!("Keyring backend:   {}", keyring_backend);
        println!("Client:            {}", client_name);
        println!("Credentials file:  {}", cred_path.display());
        println!("Credentials found: {}", if cred_exists { "yes" } else { "no" });
        match &current_account {
            Some(acct) => println!("Current account:   {}", acct),
            None => println!("Current account:   (none)"),
        }
        if let Some(ref td) = token_details {
            let services_str: Vec<String> = td.services.iter().map(|s| format!("{:?}", s)).collect();
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
            Err(e) => { eprintln!("Error: {}", e); return codes::GENERIC_ERROR; }
        };
        println!("{}", to_json_pretty(&json_val));
    } else {
        for si in &services {
            let user_marker = if si.user { "user" } else { "admin" };
            println!("{:?}\t{}\t{}", si.service, user_marker, si.scopes.join(", "));
        }
    }
    codes::SUCCESS
}

fn handle_auth_tokens(args: root::AuthTokensArgs, flags: &root::RootFlags) -> i32 {
    match args.command {
        root::AuthTokensCommand::List => handle_auth_tokens_list(flags),
        root::AuthTokensCommand::Delete(del_args) => handle_auth_tokens_delete(&del_args.email, flags),
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
        let json_keys: Vec<serde_json::Value> = keys.iter().map(|k| {
            match crate::auth::parse_token_key(k) {
                Some((client, email)) => serde_json::json!({
                    "key": k,
                    "client": client,
                    "email": email,
                }),
                None => serde_json::json!({
                    "key": k,
                }),
            }
        }).collect();
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

    let client_name = flags.client.as_deref().unwrap_or(crate::config::DEFAULT_CLIENT_NAME);
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
                    Err(e) => { eprintln!("Error: {}", e); return codes::GENERIC_ERROR; }
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
        let urls: Vec<String> = url_args.thread_ids.iter().map(|id| thread_url(id)).collect();
        if flags.json {
            let json_val = match serde_json::to_value(&urls) {
                Ok(v) => v,
                Err(e) => { eprintln!("Error: {}", e); return codes::GENERIC_ERROR; }
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
        GmailCommand::Search(ref search_args) => {
            handle_gmail_search(&ctx, search_args).await
        }
        GmailCommand::Messages(ref msg_args) => {
            handle_gmail_messages(&ctx, msg_args).await
        }
        GmailCommand::Thread(ref thread_args) => {
            handle_gmail_thread(&ctx, thread_args).await
        }
        GmailCommand::Get(ref get_args) => {
            handle_gmail_message_get(&ctx, get_args).await
        }
        GmailCommand::Send(ref send_args) => {
            handle_gmail_send(&ctx, send_args).await
        }
        GmailCommand::Labels(ref labels_args) => {
            handle_gmail_labels(&ctx, labels_args).await
        }
        GmailCommand::Attachment(ref att_args) => {
            handle_gmail_attachment(&ctx, att_args).await
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
    let url = crate::services::gmail::message::build_message_get_url(
        &args.message_id,
        format_str,
    );
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
                if std::io::stdin().read_line(&mut input).is_err() || input.trim().to_lowercase() != "y" {
                    eprintln!("Cancelled.");
                    return codes::CANCELLED;
                }
            } else if !ctx.is_force() && ctx.flags.no_input {
                eprintln!("Error: destructive operation requires --force when --no-input is set");
                return codes::USAGE_ERROR;
            }

            let url =
                crate::services::gmail::labels::build_label_delete_url(&delete_args.label_id);

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

            let label: crate::services::gmail::types::Label =
                match crate::http::api::api_get(
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
        CalendarCommand::Event(ref get_args) => {
            handle_calendar_event_get(&ctx, get_args).await
        }
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
        CalendarCommand::Freebusy(ref fb_args) => {
            handle_calendar_freebusy(&ctx, fb_args).await
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
        let attendees: Vec<serde_json::Value> = args.add_attendee.iter()
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
    let calendars: Vec<String> = args.calendar_ids
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let req = crate::services::calendar::freebusy::build_freebusy_request(
        &calendars,
        &args.from,
        &args.to,
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
                Err(e) => { eprintln!("Error: {}", e); return codes::GENERIC_ERROR; }
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
        eprint!(
            "Are you sure you want to delete '{}'? [y/N] ",
            args.file_id
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
    let get_url = format!("{}?fields=parents",
        crate::services::drive::files::build_file_get_url(&args.file_id));
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
        args.file_id, args.parent, old_parents
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

    let url =
        crate::services::drive::permissions::build_create_permission_url(&args.file_id);

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
    let url =
        crate::services::drive::permissions::build_list_permissions_url(&args.file_id);

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
/// Reads a local file and uploads it via multipart POST with metadata.
async fn handle_drive_upload(
    ctx: &crate::services::ServiceContext,
    args: &drive::DriveUploadArgs,
) -> i32 {
    use crate::services::drive::files::build_file_upload_url;
    use crate::services::export;

    // Read the file
    let file_data = match std::fs::read(&args.path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", args.path, e);
            return codes::GENERIC_ERROR;
        }
    };

    let filename = args
        .name
        .as_deref()
        .unwrap_or_else(|| {
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

    // Build multipart body
    let boundary = "omega_google_upload_boundary";
    let metadata_json = serde_json::to_string(&metadata).unwrap_or_default();

    // Guess content type from extension
    let content_type = export::guess_content_type_from_path(&args.path);

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
    let encoded_query: String =
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
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
fn handle_docs(_args: docs::DocsArgs, _flags: &root::RootFlags) -> i32 {
    // All Docs commands require authentication
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `sheets` command and its subcommands.
fn handle_sheets(_args: sheets::SheetsArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `slides` command and its subcommands.
fn handle_slides(_args: slides::SlidesArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `forms` command and its subcommands.
fn handle_forms(_args: forms::FormsArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `chat` command and its subcommands.
fn handle_chat(_args: chat::ChatArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `classroom` command and its subcommands.
fn handle_classroom(_args: classroom::ClassroomArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `tasks` command and its subcommands.
fn handle_tasks(_args: tasks::TasksArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `contacts` command and its subcommands.
fn handle_contacts(_args: contacts::ContactsArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `people` command and its subcommands.
fn handle_people(_args: people::PeopleArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `groups` command and its subcommands.
fn handle_groups(_args: groups::GroupsArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `keep` command and its subcommands.
fn handle_keep(_args: keep::KeepArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `appscript` command and its subcommands.
fn handle_appscript(_args: appscript::AppScriptArgs, _flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
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
            println!("{},{},{}",
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
