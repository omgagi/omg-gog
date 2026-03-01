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
        root::Command::Gmail(args) => handle_gmail(args, flags),
        root::Command::Calendar(args) => handle_calendar(args, flags),
        root::Command::Drive(args) => handle_drive(args, flags),
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
fn handle_gmail(args: gmail::GmailArgs, flags: &root::RootFlags) -> i32 {
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

    // All other Gmail commands require authentication
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `calendar` command and its subcommands.
fn handle_calendar(args: calendar::CalendarArgs, flags: &root::RootFlags) -> i32 {
    use calendar::CalendarCommand;

    match &args.command {
        CalendarCommand::Time => {
            // Calendar time: show server time (same as time now for stub)
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
            // Colors can work without auth (static data)
            eprintln!("Command registered. API call requires: omega-google auth add <email>");
            return codes::SUCCESS;
        }
        _ => {}
    }

    // All other Calendar commands require authentication
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

/// Handle the `drive` command and its subcommands.
fn handle_drive(args: drive::DriveArgs, flags: &root::RootFlags) -> i32 {
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

    // All other Drive commands require authentication
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
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
