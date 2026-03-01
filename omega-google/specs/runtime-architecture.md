# Architecture: omega-google Runtime Layer

## Scope

This architecture covers the **runtime layer** -- the modules that turn omega-google from a 28K-line scaffolding stub into a fully functional Google Workspace CLI. It spans authentication flows, token management, API call execution, pagination, file I/O, and the wiring of all 15 service handlers.

### Covered Domains
- Authentication (OAuth flows, token exchange, token refresh, credential storage)
- Execution infrastructure (API helpers, pagination, verbose logging, dry-run)
- Service handler dispatch (all 15 services: Gmail, Calendar, Drive, Docs, Sheets, Slides, Forms, Chat, Tasks, Classroom, Contacts, People, Groups, Keep, Apps Script)
- File I/O (streaming download, simple/resumable upload, export)

### Not Covered (already complete)
- CLI parsing (`cli/root.rs`, all `cli/<service>.rs`)
- Type definitions (all `services/<service>/types.rs`)
- URL builders (all `services/<service>/*.rs` builder functions)
- Output formatting (`output/`)
- HTTP retry and circuit breaker (`http/retry.rs`, `http/circuit_breaker.rs`, `http/middleware.rs`)
- Error types (`error/`)
- Config system (`config/`)

## Overview

```
                         +-------------------+
                         |   main.rs         |
                         |   cli::execute()  |
                         +--------+----------+
                                  |
                                  v
                    +-------------+-----------+
                    |  cli/mod.rs              |
                    |  dispatch_command()      |
                    |  (sync -> async bridge)  |
                    +--+----+----+----+----+--+
                       |    |    |    |    |
           +-----------+    |    |    |    +-----------+
           v                v    v    v                v
     handle_auth()    handle_gmail() ... handle_appscript()
           |                |
           v                v
  +--------+------+   +----+------------+
  | auth/          |   | bootstrap_auth()|
  | oauth_flow.rs  |   +----+-----------+
  | (Desktop/      |        |
  |  Manual/Remote)|        v
  +--------+------+   +----+-----------+
           |           | ServiceContext  |
           v           |   .client      |
  +--------+------+   |   .breaker     |
  | auth/oauth.rs  |   |   .retry_cfg   |
  | exchange_code()|   |   .flags       |
  +--------+------+   |   .output_mode  |
           |           +---+----+-------+
           v               |    |
  +--------+------+        |    |
  | CredentialStore|        |    |
  | (keyring/file) |        |    |
  +---------------+        v    v
                     +-----+----+-------+
                     | http/api.rs       |
                     | api_get/post/...  |
                     +---+----+---------+
                         |    |
                         v    v
                   +-----+----+---------+
                   | http/middleware.rs   |
                   | execute_with_retry  |
                   +----+----+----------+
                        |    |
                        v    v
                 +------+----+----------+
                 | http/circuit_breaker  |
                 | (existing, unchanged) |
                 +-----------+----------+
                             |
                             v
                   +---------+-------+
                   | Google REST APIs  |
                   +-----------------+
```

## Modules

### Module 1: auth/mod.rs (Modified)

- **Responsibility**: Central auth types and the CredentialStore trait. Extended with access_token caching.
- **Public interface**:
  ```rust
  // EXISTING (unchanged):
  pub trait CredentialStore: Send + Sync { ... }
  pub struct TokenData { ... }  // extended below
  pub fn resolve_account(...) -> Result<String>
  pub fn parse_token_key(key: &str) -> Option<(String, String)>
  pub fn token_key(client: &str, email: &str) -> String

  // MODIFIED TokenData -- add two optional fields:
  pub struct TokenData {
      pub client: String,
      pub email: String,
      pub services: Vec<Service>,
      pub scopes: Vec<String>,
      pub created_at: chrono::DateTime<chrono::Utc>,
      pub refresh_token: String,
      // NEW:
      pub access_token: Option<String>,
      pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
  }
  ```
- **Dependencies**: `serde`, `chrono`, `crate::config`
- **Error handling**: `anyhow::Result` throughout. New fields are `Option` for backward compatibility with stored tokens that lack them.
- **Key design decision**: `access_token` and `expires_at` are `Option<>` with `#[serde(skip_serializing_if = "Option::is_none")]` and `#[serde(default)]` so old tokens deserialize without error.

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Old token missing new fields | Deserialized from pre-runtime storage | `access_token.is_none()` | Treat as expired, trigger refresh | None visible to user |
| Token key format mismatch | Corrupted keyring data | `parse_token_key` returns `None` | Skip entry, log warning | Listed accounts may be incomplete |

#### Security Considerations
- **Sensitive data**: `access_token` and `refresh_token` must never appear in stdout, logs, or error messages
- **Mitigations**: `access_token` has `#[serde(skip_serializing_if = "Option::is_none")]`; the `serialize_token` function is the only path that serializes refresh_token, and it goes only to the credential store

#### Performance Budget
- Token deserialization: < 1ms (JSON parse of ~500 bytes)

---

### Module 2: auth/token.rs (Modified)

- **Responsibility**: Token serialization, deserialization, and refresh-check logic.
- **Public interface**:
  ```rust
  // EXISTING (modified):
  pub fn serialize_token(token: &TokenData) -> Result<String>
  pub fn deserialize_token(json_str: &str) -> Result<TokenData>
  pub fn needs_refresh(token: &TokenData) -> bool

  // NEW:
  pub async fn refresh_access_token(
      http_client: &reqwest::Client,
      creds: &ClientCredentials,
      refresh_token: &str,
  ) -> Result<TokenResponse>
  ```
- **Dependencies**: `serde_json`, `chrono`, `reqwest`, `crate::auth::oauth::TokenResponse`, `crate::config::ClientCredentials`
- **Error handling**: `anyhow::Result`. The `refresh_access_token` function returns the raw `TokenResponse` so the caller can update the credential store.

**Key logic changes**:
- `serialize_token`: Adds `access_token` and `expires_at` to the JSON if present.
- `deserialize_token`: Reads `access_token` and `expires_at` with defaults of `None`.
- `needs_refresh`: Now checks `expires_at` first. If `expires_at` is `Some`, refresh if fewer than 5 minutes remain. If `expires_at` is `None`, fall back to the existing `created_at + 55 minutes` heuristic.
- `refresh_access_token`: POSTs to `https://oauth2.googleapis.com/token` with `grant_type=refresh_token`, `refresh_token`, `client_id`, `client_secret`. Returns `TokenResponse`.

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Refresh token revoked | User revoked consent in Google Account | HTTP 400 `invalid_grant` | Clear stored token, exit code 4 with re-auth guidance | User must re-authenticate |
| Token endpoint unreachable | Network down | HTTP timeout/connection error | Exit code 5 with network error message | Command cannot proceed |
| Clock skew | System clock >5 min off | Refresh succeeds but `expires_at` is in the past immediately | Re-refresh on next call; no special handling needed | Possible extra refresh on each call |

#### Security Considerations
- **Trust boundary**: The token endpoint URL is hardcoded, not user-configurable
- **Sensitive data**: `refresh_token` appears in the POST body but never in logs

#### Performance Budget
- `needs_refresh`: < 1us (two datetime comparisons)
- `refresh_access_token`: < 2s network round-trip (must)

---

### Module 3: auth/oauth.rs (Modified)

- **Responsibility**: OAuth2 URL building and code exchange.
- **Public interface**:
  ```rust
  // EXISTING (unchanged):
  pub const AUTH_URL: &str = "...";
  pub const TOKEN_URL: &str = "...";
  pub fn build_auth_url(...) -> Result<String>

  // EXISTING (implemented, was stub):
  pub async fn exchange_code(
      http_client: &reqwest::Client,
      creds: &ClientCredentials,
      code: &str,
      redirect_uri: &str,
  ) -> Result<TokenResponse>

  // EXISTING (unchanged):
  pub struct TokenResponse { ... }
  ```
- **Dependencies**: `reqwest`, `url`, `serde`, `crate::config::ClientCredentials`
- **Error handling**: `anyhow::Result`. Checks HTTP status and parses Google error JSON on failure.

**Implementation of `exchange_code`**:
```rust
pub async fn exchange_code(
    http_client: &reqwest::Client,
    creds: &ClientCredentials,
    code: &str,
    redirect_uri: &str,
) -> anyhow::Result<TokenResponse> {
    let resp = http_client
        .post(TOKEN_URL)
        .form(&[
            ("code", code),
            ("client_id", &creds.client_id),
            ("client_secret", &creds.client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("{}", crate::error::api_error::format_api_error(status.as_u16(), &body));
    }

    let token: TokenResponse = resp.json().await?;
    Ok(token)
}
```

**Signature change**: Adds `http_client: &reqwest::Client` as first parameter (the stub had `_creds` -- the new version takes both the client and credentials explicitly). This is a breaking change to the stub signature but the stub was never called.

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Invalid code | User pasted wrong code, code expired | HTTP 400 `invalid_grant` | Display error with suggestion to retry | Auth add fails |
| Invalid client credentials | Wrong credentials.json installed | HTTP 401 | Display error pointing to `auth credentials` | Auth add fails |
| Network timeout | Endpoint unreachable | reqwest timeout | Display network error | Auth add fails |

#### Security Considerations
- **Trust boundary**: Authorization code comes from user input (pasted URL or redirect callback)
- **Mitigations**: Code is used exactly once and immediately; not logged

#### Performance Budget
- `exchange_code`: < 3s (single network round-trip to Google)

---

### Module 4: auth/oauth_flow.rs (New File)

- **Responsibility**: Desktop, manual, and remote OAuth flow orchestration. Local HTTP server for redirect capture. Browser launching.
- **Public interface**:
  ```rust
  /// Run the OAuth flow based on the selected mode.
  /// Returns the authorization code on success.
  pub async fn run_oauth_flow(
      creds: &ClientCredentials,
      services: &[Service],
      mode: FlowMode,
      force_consent: bool,
  ) -> anyhow::Result<OAuthFlowResult>

  /// Result of a successful OAuth flow.
  pub struct OAuthFlowResult {
      pub code: String,
      pub redirect_uri: String,
  }

  // Internal helpers (pub(crate) for testing):
  pub(crate) async fn run_desktop_flow(
      creds: &ClientCredentials,
      services: &[Service],
      force_consent: bool,
  ) -> Result<OAuthFlowResult>

  pub(crate) async fn run_manual_flow(
      creds: &ClientCredentials,
      services: &[Service],
      force_consent: bool,
  ) -> Result<OAuthFlowResult>

  pub(crate) async fn run_remote_flow(
      creds: &ClientCredentials,
      services: &[Service],
      force_consent: bool,
  ) -> Result<OAuthFlowResult>

  pub(crate) fn extract_code_from_url(url_str: &str) -> Result<String>
  ```
- **Dependencies**: `tokio::net::TcpListener`, `tokio::time::timeout`, `crate::auth::oauth`, `crate::auth::scopes`, `crate::ui`, `crossterm` (for terminal prompts)
- **Error handling**: `anyhow::Result`. Timeout after 120 seconds for desktop flow. Clear error messages for each failure mode.

**Desktop flow**:
1. Bind `TcpListener` to `127.0.0.1:0` (OS-assigned port)
2. Build auth URL with `redirect_uri = http://127.0.0.1:{port}`
3. Print URL to stderr, attempt `open::that()` or just tell user to open it
4. Accept one HTTP connection with 120s timeout
5. Parse the `?code=` query parameter from the GET request
6. Respond with a simple HTML "Success! You may close this tab." page
7. Return `OAuthFlowResult { code, redirect_uri }`

**Manual flow**:
1. Build auth URL with `redirect_uri = urn:ietf:wg:oauth:2.0:oob` (or a localhost placeholder)
2. Print the URL to stderr
3. Prompt user to paste the full redirect URL (not just the code)
4. Extract `?code=` from the pasted URL via `extract_code_from_url`
5. Return result

**Remote flow** (Should priority, RT-M7):
1. Build auth URL with an externally-reachable redirect URI
2. Print instructions for completing on a different machine
3. Poll or wait for the code

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Port bind failure | Another process on the port | `TcpListener::bind` error | Port 0 eliminates this; if bind fails, suggest `--manual` | Desktop flow unusable |
| Browser launch failure | Headless env, no display | `open::that` returns Err | Print URL to stderr with manual instructions | User copies URL manually |
| Timeout (120s) | User abandoned flow | `tokio::time::timeout` | Display timeout error, suggest retry | Auth add fails |
| Invalid redirect URL pasted | User error in manual mode | URL parse fails / no `code` param | Prompt again or error with guidance | Auth add fails |

#### Security Considerations
- **Trust boundary**: The redirect server MUST bind to `127.0.0.1` only, never `0.0.0.0`
- **Attack surface**: Local-only TCP listener; attacker on same machine could race to connect. Mitigated by PKCE (future enhancement) and the fact that the auth code is single-use.
- **Sensitive data**: Authorization code must not be logged even in verbose mode

#### Performance Budget
- Server startup: < 100ms
- Total flow: bounded by 120s timeout

---

### Module 5: auth/keyring.rs (Modified)

- **Responsibility**: Credential storage backends. Adds `KeyringCredentialStore` (OS keyring) and `credential_store_factory`.
- **Public interface**:
  ```rust
  // EXISTING (unchanged):
  pub struct FileCredentialStore { ... }
  impl CredentialStore for FileCredentialStore { ... }

  // NEW:
  pub struct KeyringCredentialStore;

  impl KeyringCredentialStore {
      pub fn new() -> Result<Self>
  }

  impl CredentialStore for KeyringCredentialStore { ... }

  /// Factory: build the appropriate CredentialStore based on config.
  /// Priority: config keyring_backend > auto-detect (try OS keyring, fall back to file)
  pub fn credential_store_factory(
      config: &ConfigFile,
  ) -> anyhow::Result<Box<dyn CredentialStore>>
  ```
- **Dependencies**: `keyring` crate (v3, apple-native + linux-native features), `crate::auth::token`, `crate::config`
- **Error handling**: The factory uses a try-then-fallback pattern. If `keyring_backend` is `"auto"` or absent, try OS keyring first; if it fails (e.g., no D-Bus session), fall back to file with a warning on stderr. If `keyring_backend` is `"file"`, use file directly. If `"keyring"`, use OS keyring and fail hard if it cannot be initialized.

**KeyringCredentialStore implementation**:
- Uses the `keyring` crate `Entry::new("omega-google", key)` where `key` is the token key (`token:<client>:<email>`)
- `get_token`: `entry.get_password()` -> `deserialize_token()`
- `set_token`: `serialize_token()` -> `entry.set_password()`
- `delete_token`: `entry.delete_credential()`
- `list_tokens`: Must iterate known keys. The OS keyring does not support enumeration natively. Solution: maintain a key index file (`$CONFIG_DIR/omega-google/keyring-index.json`) that lists all stored key names. Updated on set/delete.
- `keys`: Read the key index file.
- `get_default_account` / `set_default_account`: Store in `Entry::new("omega-google", "default:<client>")`

**credential_store_factory logic**:
```
match config.keyring_backend.as_deref() {
    Some("file") => FileCredentialStore::new(config_dir()?.join("tokens"))
    Some("keyring") => KeyringCredentialStore::new()? // fail hard
    Some("auto") | None => {
        match KeyringCredentialStore::new() {
            Ok(store) => store
            Err(e) => {
                eprintln!("Warning: OS keyring unavailable ({}), using file backend", e);
                FileCredentialStore::new(config_dir()?.join("tokens"))
            }
        }
    }
    Some(other) => bail!("unknown keyring_backend: {}", other)
}
```

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| OS keyring unavailable | No D-Bus session, locked keychain | `keyring::Entry::new` fails | Auto-mode falls back to file backend | Warning on stderr |
| Key index out of sync | Manual keyring editing | Index lists key that is deleted | Skip missing, log warning | list_tokens may miss entries |
| File permission denied | Config dir not writable | `std::fs::write` error | Propagate error with path info | Token storage fails |
| Keychain locked (macOS) | User hasn't unlocked keychain | `get_password` returns auth error | Prompt or error with guidance | Command blocked until unlock |

#### Security Considerations
- **Trust boundary**: OS keyring is managed by the OS security model (macOS Keychain, Linux Secret Service). File backend uses 0600 permissions.
- **Sensitive data**: All token data (including refresh_token, access_token) stored encrypted by the OS keyring or in 0600 files
- **Attack surface**: File backend tokens are readable by the user. OS keyring requires OS-level authentication for access.

#### Performance Budget
- `get_token`: < 50ms (OS keyring read)
- `set_token`: < 100ms (OS keyring write)
- `credential_store_factory`: < 200ms (includes OS keyring probe)

---

### Module 6: auth/service_account.rs (Modified)

- **Responsibility**: JWT-based service account authentication.
- **Public interface**:
  ```rust
  // EXISTING (unchanged):
  pub struct ServiceAccountKey { ... }
  pub struct JwtClaims { ... }
  pub fn load_service_account_key(path: &Path) -> Result<ServiceAccountKey>
  pub fn build_jwt_assertion(key: &ServiceAccountKey, scopes: &[String], subject: Option<&str>) -> Result<String>

  // EXISTING (implemented, was stub):
  pub async fn exchange_jwt(
      http_client: &reqwest::Client,
      assertion: &str,
  ) -> Result<ServiceAccountTokenResponse>

  // NEW:
  pub struct ServiceAccountTokenResponse {
      pub access_token: String,
      pub expires_in: u64,
      pub token_type: String,
  }
  ```
- **Dependencies**: `reqwest`, `jsonwebtoken`, `chrono`, `serde`
- **Error handling**: `anyhow::Result`. Parse Google token endpoint error on failure.

**Implementation of `exchange_jwt`**:
```rust
pub async fn exchange_jwt(
    http_client: &reqwest::Client,
    assertion: &str,
) -> anyhow::Result<ServiceAccountTokenResponse> {
    let resp = http_client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", assertion),
        ])
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("{}", crate::error::api_error::format_api_error(status.as_u16(), &body));
    }

    let token: ServiceAccountTokenResponse = resp.json().await?;
    Ok(token)
}
```

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Invalid private key | Corrupted key file | `jsonwebtoken::encode` fails | Error with "check service account key file" guidance | SA auth fails |
| Key file not found | Wrong path | `load_service_account_key` file not found | Error with expected path | SA auth fails |
| JWT exchange rejected | Wrong scopes, expired key | HTTP 400/401 from token endpoint | Display Google error message | SA auth fails |

#### Security Considerations
- **Sensitive data**: Service account private key MUST never be logged, even with `--verbose`. The assertion JWT (which contains no secrets in the payload, only a signature) can be logged in redacted form.
- **Mitigations**: No logging of request body for token endpoint POST

#### Performance Budget
- `build_jwt_assertion`: < 10ms (RSA signing)
- `exchange_jwt`: < 2s (network round-trip)

---

### Module 7: http/api.rs (New File)

- **Responsibility**: Generic API call helpers that combine authenticated HTTP client, retry middleware, circuit breaker, verbose logging, and error handling into a single call.
- **Public interface**:
  ```rust
  use crate::services::ServiceContext;

  /// GET a URL, deserialize JSON response.
  pub async fn api_get<T: DeserializeOwned>(
      ctx: &ServiceContext,
      url: &str,
  ) -> anyhow::Result<T>

  /// POST JSON body to a URL, deserialize JSON response.
  pub async fn api_post<T: DeserializeOwned>(
      ctx: &ServiceContext,
      url: &str,
      body: &impl Serialize,
  ) -> anyhow::Result<T>

  /// POST JSON body, return no parsed body (for 204 responses).
  pub async fn api_post_empty(
      ctx: &ServiceContext,
      url: &str,
      body: &impl Serialize,
  ) -> anyhow::Result<()>

  /// PATCH JSON body to a URL, deserialize JSON response.
  pub async fn api_patch<T: DeserializeOwned>(
      ctx: &ServiceContext,
      url: &str,
      body: &impl Serialize,
  ) -> anyhow::Result<T>

  /// DELETE a URL, return no parsed body.
  pub async fn api_delete(
      ctx: &ServiceContext,
      url: &str,
  ) -> anyhow::Result<()>

  /// PUT raw bytes (for file upload).
  pub async fn api_put_bytes<T: DeserializeOwned>(
      ctx: &ServiceContext,
      url: &str,
      content_type: &str,
      body: Vec<u8>,
  ) -> anyhow::Result<T>

  /// GET a URL, return raw response (for file download / streaming).
  pub async fn api_get_raw(
      ctx: &ServiceContext,
      url: &str,
  ) -> anyhow::Result<reqwest::Response>

  /// Check response status, format Google API error if 4xx/5xx.
  pub fn check_response_status(status: u16, body: &str) -> anyhow::Result<()>
  ```
- **Dependencies**: `crate::http::middleware`, `crate::http::RetryConfig`, `crate::http::circuit_breaker::CircuitBreaker`, `crate::error::api_error`, `serde`, `reqwest`
- **Error handling**: All functions check HTTP status codes. 4xx/5xx responses are parsed as Google API errors and returned as `anyhow::Error`. The error message includes the HTTP status and the extracted Google error message.

**Verbose logging**: When `ctx.flags.verbose` is true, each API call logs to stderr:
```
> GET https://gmail.googleapis.com/gmail/v1/users/me/threads?maxResults=20
< 200 OK (4523 bytes, 234ms)
```
The Authorization header is redacted: `Authorization: Bearer [REDACTED]`

**Dry-run**: When `ctx.flags.dry_run` is true, mutating methods (POST, PATCH, DELETE, PUT) log the request details and return early without executing. GET requests proceed normally (reads are not mutating).

**Implementation pattern** (for `api_get`):
```rust
pub async fn api_get<T: DeserializeOwned>(ctx: &ServiceContext, url: &str) -> anyhow::Result<T> {
    if ctx.flags.verbose {
        eprintln!("> GET {}", url);
    }

    let request = RetryableRequest::new(Method::GET, url.to_string(), None);
    let start = Instant::now();
    let response = execute_with_retry(&ctx.client, &request, &ctx.retry_config, &ctx.circuit_breaker).await?;
    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let body = response.text().await?;

    if ctx.flags.verbose {
        eprintln!("< {} ({} bytes, {}ms)", status, body.len(), elapsed.as_millis());
    }

    check_response_status(status, &body)?;
    let parsed: T = serde_json::from_str(&body)?;
    Ok(parsed)
}
```

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| JSON deserialization fails | API response doesn't match expected type | `serde_json::from_str` error | Return error with response body for debugging | Command fails |
| Circuit breaker open | Too many recent failures | `execute_with_retry` returns circuit breaker error | Error message suggests waiting 30s | All API calls blocked |
| Network timeout | Slow/dead network | reqwest timeout (300s default) | Error with timeout suggestion | Command fails |
| 401 Unauthorized | Token expired mid-execution | Status check | Error with "run auth add" guidance | Command fails |
| 403 Forbidden | Insufficient scopes | Status check | Error with "check service permissions" | Command fails |

#### Security Considerations
- **Sensitive data**: Bearer token redacted in verbose logs
- **Mitigations**: Logging function replaces `Bearer <token>` with `Bearer [REDACTED]`

#### Performance Budget
- Per-call overhead (excluding network): < 1ms
- Verbose logging overhead: < 0.5ms

---

### Module 8: services/pagination.rs (New File)

- **Responsibility**: Generic pagination loop that works with any Google API paginated endpoint.
- **Public interface**:
  ```rust
  use crate::services::ServiceContext;

  /// Paginate through all pages of a list endpoint.
  /// `url_fn` builds the URL for each page given an optional page token.
  /// `extract_fn` extracts items and next_page_token from the response JSON.
  pub async fn paginate<T>(
      ctx: &ServiceContext,
      url_fn: impl Fn(Option<&str>) -> String,
      extract_fn: impl Fn(serde_json::Value) -> anyhow::Result<(Vec<T>, Option<String>)>,
  ) -> anyhow::Result<Vec<T>>

  /// Paginate with a progress callback (for verbose mode).
  pub async fn paginate_with_progress<T>(
      ctx: &ServiceContext,
      url_fn: impl Fn(Option<&str>) -> String,
      extract_fn: impl Fn(serde_json::Value) -> anyhow::Result<(Vec<T>, Option<String>)>,
      progress_fn: impl Fn(usize, usize),  // (page_number, items_so_far)
  ) -> anyhow::Result<Vec<T>>

  /// Single-page fetch: returns items and optional next_page_token hint.
  /// Used when `--all` is not specified.
  pub async fn fetch_page<T: DeserializeOwned>(
      ctx: &ServiceContext,
      url: &str,
  ) -> anyhow::Result<(T, Option<String>)>
  where T: HasNextPageToken
  ```

  ```rust
  /// Trait for types that carry a nextPageToken.
  pub trait HasNextPageToken {
      fn next_page_token(&self) -> Option<&str>;
  }
  ```
- **Dependencies**: `crate::http::api`, `serde_json`, `crate::ui::progress`
- **Error handling**: Errors on any page are propagated immediately (fail-fast). No partial results returned on error.

**Pagination loop**:
```rust
pub async fn paginate<T>(ctx, url_fn, extract_fn) -> Result<Vec<T>> {
    let mut all_items = Vec::new();
    let mut page_token: Option<String> = None;
    let mut page_num = 0;

    loop {
        page_num += 1;
        let url = url_fn(page_token.as_deref());

        if ctx.flags.verbose {
            eprintln!("Fetching page {}...", page_num);
        }

        let response: serde_json::Value = api_get(ctx, &url).await?;
        let (items, next_token) = extract_fn(response)?;

        all_items.extend(items);
        page_token = next_token;

        if page_token.is_none() {
            break;
        }
    }

    Ok(all_items)
}
```

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Infinite pagination loop | API always returns nextPageToken | Guard: max 1000 pages | Break with warning | Bounded resource usage |
| Partial failure on page N | Network error mid-pagination | Error from `api_get` | Fail-fast, discard accumulated results | User retries from scratch |
| Empty items array with nextPageToken | Unusual API response | `extract_fn` returns empty vec | Continue to next page (valid scenario) | No data loss |

#### Performance Budget
- Per-page overhead: < 1ms (beyond API call time)
- Memory: O(total_items) -- all items accumulated. No streaming.

---

### Module 9: services/mod.rs (Modified)

- **Responsibility**: ServiceContext struct, auth bootstrap, and context factory.
- **Public interface**:
  ```rust
  // EXISTING (extended):
  pub struct ServiceContext {
      pub client: reqwest::Client,
      pub output_mode: OutputMode,
      pub json_transform: JsonTransform,
      pub ui: Ui,
      pub flags: RootFlags,
      // NEW:
      pub circuit_breaker: Arc<CircuitBreaker>,
      pub retry_config: RetryConfig,
      pub email: String,  // resolved account email
  }

  // EXISTING (unchanged):
  impl ServiceContext {
      pub fn write_output<T: Serialize>(&self, value: &T) -> Result<()>
      pub fn write_paginated<T: Serialize>(&self, value: &T, next_page_token: Option<&str>) -> Result<()>
      pub fn is_dry_run(&self) -> bool
      pub fn is_force(&self) -> bool
      pub fn account(&self) -> Option<&str>
  }

  // NEW:
  /// Bootstrap authentication and build a ServiceContext.
  /// 1. Load config
  /// 2. Build credential store (via factory)
  /// 3. Resolve account (flag > env > default > single)
  /// 4. Load token from store
  /// 5. Check if refresh needed, refresh if so
  /// 6. Build authenticated reqwest::Client
  /// 7. Build ServiceContext
  pub async fn bootstrap_service_context(
      flags: &RootFlags,
  ) -> anyhow::Result<ServiceContext>
  ```
- **Dependencies**: `crate::auth`, `crate::config`, `crate::http::client`, `crate::http::circuit_breaker`, `crate::http::RetryConfig`, `crate::output`
- **Error handling**: Bootstrap reports specific exit-code-worthy errors:
  - No credentials file: "Run: omega-google auth credentials <path>"
  - No stored accounts: "Run: omega-google auth add" (exit 4)
  - Token refresh failure: Network or revoked token errors
  - Ambiguous account: "Multiple accounts found. Use --account to specify."

**bootstrap_service_context implementation outline**:
```rust
pub async fn bootstrap_service_context(flags: &RootFlags) -> anyhow::Result<ServiceContext> {
    let config = crate::config::read_config()?;
    let client_name = crate::config::normalize_client_name(
        flags.client.as_deref().unwrap_or("default")
    );

    let store = crate::auth::keyring::credential_store_factory(&config)?;
    let email = crate::auth::resolve_account(
        flags.account.as_deref(), &config, store.as_ref(), &client_name
    )?;

    let mut token = store.get_token(&client_name, &email)?;

    // Refresh if needed
    if crate::auth::token::needs_refresh(&token) {
        let creds = crate::config::read_client_credentials(&client_name)?;
        let bare_client = crate::http::client::build_client()?;
        let resp = crate::auth::token::refresh_access_token(&bare_client, &creds, &token.refresh_token).await?;

        token.access_token = Some(resp.access_token.clone());
        token.expires_at = resp.expires_in.map(|secs|
            chrono::Utc::now() + chrono::Duration::seconds(secs as i64)
        );
        // Persist updated token
        store.set_token(&client_name, &email, &token)?;
    }

    let access_token = token.access_token
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("no access token available; run: omega-google auth add"))?;

    let authenticated_client = crate::http::client::build_authenticated_client(access_token)?;

    let is_tty = atty::is(atty::Stream::Stdout);  // or crossterm equivalent
    let output_mode = crate::output::resolve_mode_full(
        flags.json, flags.plain, flags.csv, is_tty
    )?;
    let json_transform = JsonTransform {
        results_only: flags.results_only,
        select: flags.select.as_deref()
            .map(|s| s.split(',').map(|f| f.trim().to_string()).collect())
            .unwrap_or_default(),
    };
    let ui = crate::ui::Ui::new(flags.verbose, !flags.no_input);

    Ok(ServiceContext {
        client: authenticated_client,
        output_mode,
        json_transform,
        ui,
        flags: flags.clone(),
        circuit_breaker: Arc::new(CircuitBreaker::new()),
        retry_config: RetryConfig::default(),
        email,
    })
}
```

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| No credential file | First-time user, hasn't run `auth credentials` | `read_client_credentials` fails | Exit 4 with clear guidance | All commands blocked |
| No stored accounts | User hasn't authenticated | `resolve_account` fails | Exit 4 with "run auth add" | All commands blocked |
| Ambiguous account | Multiple accounts, no --account flag, no default | `resolve_account` fails | Exit 2 with account list | User must specify |
| Token refresh fails | Network, revoked token | `refresh_access_token` error | Propagate with re-auth guidance | Command blocked |

#### Security Considerations
- **Trust boundary**: All external data (config files, keyring data) validated before use
- **Sensitive data**: access_token passed to `build_authenticated_client` only, never logged

#### Performance Budget
- Full bootstrap (cache hit, no refresh): < 100ms
- Full bootstrap (with refresh): < 2.5s

---

### Module 10: services/export.rs (New File)

- **Responsibility**: Shared export logic for Google Workspace documents (Docs/Sheets/Slides exported to PDF/text/etc via Drive export API).
- **Public interface**:
  ```rust
  use crate::services::ServiceContext;

  /// Export a Google Workspace document to a local file.
  pub async fn export_document(
      ctx: &ServiceContext,
      file_id: &str,
      mime_type: &str,
      output_path: &std::path::Path,
  ) -> anyhow::Result<u64>  // returns bytes written

  /// Supported export MIME types for each Google document type.
  pub fn export_mime_types(google_mime_type: &str) -> Vec<(&'static str, &'static str)>
  // Returns vec of (mime_type, file_extension) tuples
  ```
- **Dependencies**: `crate::http::api`, `tokio::fs`, `tokio::io`
- **Error handling**: 404 = file not found, 403 = no permission, 400 = unsupported export format.

**Implementation**:
```rust
pub async fn export_document(ctx, file_id, mime_type, output_path) -> Result<u64> {
    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}/export?mimeType={}",
        file_id,
        percent_encoding::utf8_percent_encode(mime_type, percent_encoding::NON_ALPHANUMERIC)
    );
    let response = api_get_raw(ctx, &url).await?;
    // Stream response body to file
    let mut file = tokio::fs::File::create(output_path).await?;
    let mut stream = response.bytes_stream();
    let mut total = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        total += chunk.len() as u64;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
    }
    Ok(total)
}
```

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Unsupported export format | Wrong mime_type for doc type | HTTP 400 | List supported formats in error | Export fails |
| Output path not writable | Permission denied | `File::create` error | Error with path info | Export fails |
| Large export OOM | Exporting huge spreadsheet | N/A -- streaming | Streaming prevents OOM | No memory issue |

#### Performance Budget
- Memory: Streaming, bounded by chunk size (8KB default)
- Throughput: Limited by network bandwidth

---

### Module 11: cli/mod.rs (Modified -- Major)

- **Responsibility**: Dispatch all commands to their handlers. Auth commands wired to OAuth flow. Service commands wired to async handlers via bootstrap.
- **Changes**:

**Handler signature change**: All service handlers become async and receive `ServiceContext` instead of raw `RootFlags`:

```rust
// BEFORE (stub):
fn handle_gmail(args: gmail::GmailArgs, flags: &root::RootFlags) -> i32 {
    eprintln!("Command registered. API call requires: omega-google auth add <email>");
    codes::SUCCESS
}

// AFTER (real):
async fn handle_gmail(args: gmail::GmailArgs, flags: &root::RootFlags) -> i32 {
    // Commands that don't need auth (url, etc.)
    if let GmailCommand::Url(url_args) = &args.command { ... return ...; }

    // Bootstrap auth for all other commands
    let ctx = match crate::services::bootstrap_service_context(flags).await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return codes::AUTH_ERROR;
        }
    };

    match args.command {
        GmailCommand::Search(search_args) => handle_gmail_search(&ctx, &search_args).await,
        GmailCommand::Threads(thread_args) => handle_gmail_threads(&ctx, &thread_args).await,
        GmailCommand::Send(send_args) => handle_gmail_send(&ctx, &send_args).await,
        // ... all subcommands
        _ => unreachable!(), // Url already handled above
    }
}
```

**Dispatch pattern**: The `dispatch_command` function signature changes from returning `i32` synchronously to `async fn dispatch_command(...) -> i32`. It is already called in an async context (the `execute` function is already `async`).

**Auth handler changes**:
```rust
root::AuthCommand::Add(add_args) => handle_auth_add(add_args, flags).await,
root::AuthCommand::Remove(remove_args) => handle_auth_remove(&remove_args.email, flags),
root::AuthCommand::Status => handle_auth_status(flags),
```

**`handle_auth_add` implementation**:
```rust
async fn handle_auth_add(args: root::AuthAddArgs, flags: &root::RootFlags) -> i32 {
    let config = match crate::config::read_config() { ... };
    let client_name = crate::config::normalize_client_name(
        flags.client.as_deref().unwrap_or("default")
    );
    let creds = match crate::config::read_client_credentials(&client_name) { ... };

    let mode = if args.manual { FlowMode::Manual }
               else if args.remote { FlowMode::Remote }
               else { FlowMode::Desktop };

    let services = crate::auth::user_services();

    // Run OAuth flow
    let flow_result = match run_oauth_flow(&creds, &services, mode, args.force_consent).await {
        Ok(r) => r,
        Err(e) => { eprintln!("Error: {}", e); return codes::AUTH_ERROR; }
    };

    // Exchange code for tokens
    let bare_client = crate::http::client::build_client().unwrap();
    let token_resp = match exchange_code(&bare_client, &creds, &flow_result.code, &flow_result.redirect_uri).await {
        Ok(r) => r,
        Err(e) => { eprintln!("Error: {}", e); return codes::AUTH_ERROR; }
    };

    // Extract email from userinfo endpoint
    let email = fetch_userinfo(&bare_client, &token_resp.access_token).await
        .unwrap_or_else(|_| "unknown@unknown".to_string());

    // Build TokenData and store
    let token_data = TokenData {
        client: client_name.clone(),
        email: email.clone(),
        services: services.clone(),
        scopes: scopes_for_manage(&services, &Default::default()).unwrap_or_default(),
        created_at: chrono::Utc::now(),
        refresh_token: token_resp.refresh_token.unwrap_or_default(),
        access_token: Some(token_resp.access_token),
        expires_at: token_resp.expires_in.map(|s| chrono::Utc::now() + chrono::Duration::seconds(s as i64)),
    };

    let store = credential_store_factory(&config).unwrap();
    store.set_token(&client_name, &email, &token_data).unwrap();
    store.set_default_account(&client_name, &email).unwrap();

    eprintln!("Authenticated as {}", email);
    codes::SUCCESS
}
```

**Service handler implementation pattern** (using Gmail search as example):
```rust
async fn handle_gmail_search(ctx: &ServiceContext, args: &GmailSearchArgs) -> i32 {
    let pagination = PaginationParams {
        max_results: args.max_results,
        page_token: args.page_token.clone(),
        all_pages: args.all,
        fail_empty: args.fail_empty,
    };

    if pagination.all_pages {
        // Paginate all pages
        let items = match paginate(
            ctx,
            |token| build_thread_search_url(&args.query, pagination.max_results, token),
            |json| {
                let resp: ThreadListResponse = serde_json::from_value(json)?;
                Ok((resp.threads, resp.next_page_token))
            },
        ).await {
            Ok(items) => items,
            Err(e) => { eprintln!("Error: {}", e); return codes::API_ERROR; }
        };

        if items.is_empty() && pagination.fail_empty {
            return codes::EMPTY_RESULT;
        }
        let _ = ctx.write_output(&items);
    } else {
        // Single page
        let url = build_thread_search_url(&args.query, pagination.max_results, pagination.page_token.as_deref());
        let resp: ThreadListResponse = match api_get(ctx, &url).await {
            Ok(r) => r,
            Err(e) => { eprintln!("Error: {}", e); return codes::API_ERROR; }
        };

        if resp.threads.is_empty() && pagination.fail_empty {
            return codes::EMPTY_RESULT;
        }
        let _ = ctx.write_paginated(&resp, resp.next_page_token.as_deref());
    }

    codes::SUCCESS
}
```

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Bootstrap fails | No auth, no credentials | bootstrap returns Err | Exit with appropriate code (4 for auth, 5 for network) | Handler never runs |
| API call fails | 4xx/5xx response | api_get returns Err | Display formatted error, return API_ERROR exit code | Command fails cleanly |
| Output write fails | stdout closed (broken pipe) | write_output returns Err | Silently exit with SUCCESS (Unix convention) | No crash |

#### Performance Budget
- Handler dispatch overhead: < 1ms
- Bootstrap: see Module 9 budget

---

## System-Level Failure Modes

| Scenario | Affected Modules | Detection | Recovery Strategy | Degraded Behavior |
|----------|-----------------|-----------|-------------------|-------------------|
| Google API completely down | All service handlers | Circuit breaker opens after 5 failures | Error message, suggest retry later | No API operations possible |
| Token revoked by user | bootstrap, all handlers | 401 on any API call or refresh failure | Exit code 4 with "run auth add" | Must re-authenticate |
| Config directory deleted | credential store, config | File read errors | Clear error with "run auth credentials" | Must reconfigure |
| Disk full | credential store (file), export, download | Write errors | Error with disk space message | Cannot store tokens or download files |
| OS keyring service crash | KeyringCredentialStore | keyring crate returns error | Auto-mode falls back to file | Transparent fallback |
| Multiple concurrent processes | credential store | Write race on token file | Last writer wins; losers refresh on next call | Occasional extra refresh |

## Security Model

### Trust Boundaries

| Boundary | Trusted Side | Untrusted Side | Protection |
|----------|-------------|----------------|------------|
| CLI arguments | Application code | User input | clap validation, argument parsing |
| Config files | Config module | Filesystem (user-writable) | JSON parsing with error handling |
| Credential store | Token management | Storage backend | 0600 file perms, OS keyring encryption |
| OAuth redirect | Local server | Browser/network | 127.0.0.1 binding only |
| Google API responses | HTTP client | Google servers | TLS 1.2+, response validation |
| Token endpoint | Auth module | Google OAuth | Hardcoded endpoint URL, TLS |

### Data Classification

| Data | Classification | Storage | Access Control |
|------|---------------|---------|---------------|
| OAuth client_id | Internal | Config dir file (0600) | File permissions |
| OAuth client_secret | Confidential | Config dir file (0600) | File permissions |
| Refresh token | Secret | OS keyring or file (0600) | OS keyring auth or file perms |
| Access token | Secret (short-lived) | OS keyring or file (0600), memory | Same as refresh; expires in 1h |
| Service account private key | Secret | Config dir file (0600) | File permissions; never logged |
| Authorization code | Secret (single-use) | Memory only | Not persisted; used immediately |
| API response data | Internal | Memory, stdout | User-controlled output |

### Attack Surface

| Surface | Risk | Mitigation |
|---------|------|------------|
| Local OAuth server | TOCTOU: attacker races to send code | Single-use code; 127.0.0.1 only; PKCE (future) |
| Token file on disk | Readable by same user | 0600 permissions; encrypted file backend (Should) |
| Verbose logging | Token leakage in logs | Bearer token redacted; SA keys never logged |
| Environment variables | GOG_ACCOUNT, GOG_CLIENT readable by processes | Standard Unix env security model |
| Redirect URL in manual flow | Phishing risk if user sends URL to attacker | URL contains only auth code, not credentials |

## Graceful Degradation

| Dependency | Normal Behavior | Degraded Behavior | User Impact |
|-----------|----------------|-------------------|-------------|
| OS keyring | Tokens in OS-encrypted storage | Auto-fallback to file (0600) | Warning on stderr; functional |
| Google OAuth endpoint | Token refresh in <2s | Retry once, then fail with guidance | User retries or waits |
| Google APIs | API calls succeed | Circuit breaker trips after 5 failures; 30s cooldown | "Service temporarily unavailable" |
| Browser (desktop flow) | Opens auth URL in browser | Falls back to manual URL display | User copies URL manually |
| Network | All operations work | Timeout after 300s | Error with network troubleshooting hint |
| stdout (broken pipe) | Output written | Silently exit | No crash; Unix convention |

## Performance Budgets

| Operation | Latency (p50) | Latency (p99) | Memory | Notes |
|-----------|---------------|---------------|--------|-------|
| Token refresh | 500ms | 2s | < 1KB | Single HTTP round-trip |
| Bootstrap (cached token) | 20ms | 100ms | < 10KB | File read + client build |
| Bootstrap (with refresh) | 600ms | 2.5s | < 10KB | Includes token refresh |
| Single API call (GET) | 200ms | 1s | Response size | Depends on API endpoint |
| Pagination (10 pages) | 2s | 10s | Sum of all pages | Linear in page count |
| File download (100MB) | 10s | 60s | < 1MB | Streaming; bounded memory |
| File upload (simple, <5MB) | 2s | 10s | File size | Single POST |
| File upload (resumable) | Varies | Varies | < 8MB | Chunked; 8MB chunks |
| OAuth desktop flow | 5s | 120s | < 1MB | Bounded by user action |

## Data Flow

### Authentication Flow
```
User runs `auth add`
  -> CLI parses flags
  -> Load config + client credentials
  -> run_oauth_flow() starts local server
  -> User completes consent in browser
  -> Server captures auth code
  -> exchange_code() POST to Google token endpoint
  -> Receive access_token + refresh_token
  -> fetch_userinfo() GET to Google userinfo endpoint
  -> Build TokenData with all fields
  -> credential_store.set_token() persists to keyring/file
  -> Print success message
```

### Service Command Flow
```
User runs `gmail search "test"`
  -> CLI parses flags + args
  -> dispatch_command() -> handle_gmail() -> handle_gmail_search()
  -> bootstrap_service_context():
     -> read_config()
     -> credential_store_factory()
     -> resolve_account()
     -> store.get_token()
     -> needs_refresh() -> maybe refresh_access_token()
     -> build_authenticated_client(access_token)
     -> ServiceContext { client, breaker, retry_config, ... }
  -> build_thread_search_url(query, max, page_token)
  -> api_get(ctx, url) -> execute_with_retry(client, request, config, breaker)
  -> Deserialize ThreadListResponse
  -> ctx.write_output(response)  (respects --json, --plain, --select, --results-only)
  -> Return exit code 0
```

### Pagination Flow
```
User runs `gmail search "test" --all`
  -> bootstrap_service_context()
  -> paginate(ctx, url_fn, extract_fn):
     -> page 1: url_fn(None) -> api_get -> extract items + nextPageToken
     -> page 2: url_fn(Some(token)) -> api_get -> extract items + nextPageToken
     -> page 3: url_fn(Some(token)) -> api_get -> extract items + None (done)
  -> Return Vec<all_items>
  -> ctx.write_output(all_items)
```

### File Download Flow
```
User runs `drive download FILE_ID --out /tmp/file.pdf`
  -> bootstrap_service_context()
  -> api_get_raw(ctx, download_url)
  -> Response has streaming body
  -> Open output file
  -> While chunks remain: read chunk, write to file
  -> Print "Downloaded N bytes to /tmp/file.pdf"
```

## Design Decisions

| Decision | Alternatives Considered | Justification |
|----------|------------------------|---------------|
| Raw reqwest POST for token exchange (not `oauth2` crate) | oauth2 crate flows | Consistent with "raw REST" philosophy; full control over error handling; oauth2 crate adds complexity for little benefit |
| Cache access_token in credential store | Refresh on every command | Saves 500ms-2s per command; access tokens valid for 1h |
| Pagination accumulates in memory | Streaming JSON output | Simpler; consistent output format; --page available for manual pagination of huge sets |
| Single CircuitBreaker per invocation | Per-service breakers, global persistent breaker | Single invocation is short-lived; no need for cross-invocation state; per-service adds complexity |
| OS-assigned port for OAuth server | Fixed port (8080, etc.) | Eliminates port conflicts; redirect_uri includes actual port |
| `anyhow::Result` throughout (not custom error enum) | Typed error enum | Runtime layer is high-level orchestration; anyhow context propagation is sufficient; typed errors in lower modules (existing) |
| Async handlers called from async dispatch | Separate async runtime per handler | Already in a tokio runtime from main(); no overhead |
| Manual flow accepts redirect URL (not raw code) | Paste just the auth code | OOB flow deprecated by Google; redirect URL extraction is more robust |
| File-based key index for OS keyring enumeration | Enumerate keyring directly | OS keyrings (macOS Keychain, Linux Secret Service) lack efficient enumeration APIs |

## External Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `reqwest` | 0.12 | HTTP client (existing) |
| `tokio` | 1 | Async runtime (existing) |
| `serde` + `serde_json` | 1 | Serialization (existing) |
| `chrono` | 0.4 | Date/time (existing) |
| `keyring` | 3 | OS keyring access (existing dep, now used) |
| `jsonwebtoken` | 9 | JWT signing for SA (existing) |
| `url` | 2 | URL parsing (existing) |
| `percent-encoding` | 2 | URL encoding (existing) |
| `base64` | 0.22 | Base64 encoding for MIME (existing) |
| `aes-gcm` | 0.10 | Encrypted file backend (existing dep, used in RT-M7) |
| `crossterm` | 0.28 | Terminal interaction for prompts (existing) |
| `anyhow` | 1 | Error handling (existing) |
| `futures-util` | 0.3 | **NEW** -- `StreamExt` for streaming response body in download/export |

**Note**: `futures-util` must be added to `Cargo.toml` for the `bytes_stream().next()` pattern used in streaming downloads.

## New Files

| File | Module | Responsibility |
|------|--------|---------------|
| `src/auth/oauth_flow.rs` | Module 4 | OAuth flow orchestration (desktop, manual, remote) |
| `src/http/api.rs` | Module 7 | Generic API call helpers (GET, POST, PATCH, DELETE, PUT, raw) |
| `src/services/pagination.rs` | Module 8 | Generic pagination loop |
| `src/services/export.rs` | Module 10 | Shared Drive export logic |

## Modified Files

| File | Changes |
|------|---------|
| `src/auth/mod.rs` | Add `access_token: Option<String>`, `expires_at: Option<DateTime<Utc>>` to `TokenData`; add `pub mod oauth_flow;` |
| `src/auth/token.rs` | Extend `serialize_token` / `deserialize_token` for new fields; update `needs_refresh` to check `expires_at`; add `refresh_access_token` |
| `src/auth/oauth.rs` | Implement `exchange_code` (replace stub); add `http_client` parameter |
| `src/auth/keyring.rs` | Add `KeyringCredentialStore`, `credential_store_factory` |
| `src/auth/service_account.rs` | Implement `exchange_jwt` (replace stub); add `http_client` parameter; add `ServiceAccountTokenResponse` |
| `src/http/mod.rs` | Add `pub mod api;` |
| `src/services/mod.rs` | Add `circuit_breaker`, `retry_config`, `email` fields to `ServiceContext`; add `bootstrap_service_context`; add `pub mod pagination;` and `pub mod export;` |
| `src/services/common.rs` | No changes needed (pagination params already defined; pagination loop in new module) |
| `src/cli/mod.rs` | Convert all `handle_*` service functions to async; implement auth add/remove/status/list; implement all 15 service handler dispatches |
| `src/lib.rs` | No changes needed (all modules already declared) |
| `Cargo.toml` | Add `futures-util = "0.3"` dependency |

## Milestones

| ID | Name | Scope (Files) | Scope (Requirements) | Dependencies | Est. LOC |
|----|------|---------------|---------------------|--------------|----------|
| RT-M1 | Auth Core | `auth/mod.rs`, `auth/token.rs`, `auth/oauth.rs`, `auth/service_account.rs`, `auth/keyring.rs` | REQ-RT-001, 005, 006, 007, 013, 015 | None | ~400 |
| RT-M2 | Auth Flows | `auth/oauth_flow.rs`, `cli/mod.rs` (auth handlers only) | REQ-RT-002, 003, 008, 009, 010, 011, 012 | RT-M1 | ~500 |
| RT-M3 | Execution Infrastructure | `http/api.rs`, `services/mod.rs`, `services/pagination.rs`, `Cargo.toml` | REQ-RT-017, 018, 019, 020, 021, 022, 023, 024, 025, 081, 082 | RT-M1 | ~600 |
| RT-M4 | Core Service Handlers | `cli/mod.rs` (Gmail, Calendar, Drive handlers) | REQ-RT-032-040, 044-050, 055-066 | RT-M3 | ~1200 |
| RT-M5 | File I/O | `services/drive/` (download/upload), `services/gmail/` (attachments), `services/export.rs` | REQ-RT-026, 027, 028, 030, 031 | RT-M3 | ~400 |
| RT-M6 | Extended Service Handlers | `cli/mod.rs` (remaining 12 service handlers) | REQ-RT-069-080 | RT-M4 | ~800 |
| RT-M7 | Polish | `auth/oauth_flow.rs` (remote), `auth/keyring.rs` (encrypted file), `services/drive/` (resumable upload) | REQ-RT-004, 014, 016, 029 | RT-M6 | ~300 |

### Milestone Dependency Graph
```
RT-M1 (Auth Core)
  |
  +---> RT-M2 (Auth Flows)
  |
  +---> RT-M3 (Execution Infrastructure)
           |
           +---> RT-M4 (Core Handlers)
           |       |
           |       +---> RT-M6 (Extended Handlers)
           |                |
           |                +---> RT-M7 (Polish)
           |
           +---> RT-M5 (File I/O)
```

### Implementation Order Within Each Milestone

**RT-M1: Auth Core**
1. Extend `TokenData` in `auth/mod.rs` (backward-compatible `Option` fields)
2. Update `serialize_token` / `deserialize_token` in `auth/token.rs`
3. Update `needs_refresh` in `auth/token.rs`
4. Implement `refresh_access_token` in `auth/token.rs`
5. Implement `exchange_code` in `auth/oauth.rs`
6. Implement `exchange_jwt` in `auth/service_account.rs`
7. Add `KeyringCredentialStore` to `auth/keyring.rs`
8. Add `credential_store_factory` to `auth/keyring.rs`

**RT-M2: Auth Flows**
1. Create `auth/oauth_flow.rs` with `extract_code_from_url`
2. Implement `run_manual_flow`
3. Implement `run_desktop_flow` (local TCP server + browser open)
4. Wire `handle_auth_add` in `cli/mod.rs`
5. Wire `handle_auth_remove` in `cli/mod.rs`
6. Wire `handle_auth_list` in `cli/mod.rs` (read from credential store)
7. Wire `handle_auth_status` in `cli/mod.rs`

**RT-M3: Execution Infrastructure**
1. Create `http/api.rs` with `api_get` (simplest helper)
2. Add `api_post`, `api_patch`, `api_delete`, `api_put_bytes`, `api_get_raw`
3. Add verbose logging to all API helpers
4. Add dry-run guard to mutating helpers
5. Create `services/pagination.rs` with `paginate` function
6. Add `bootstrap_service_context` to `services/mod.rs`
7. Extend `ServiceContext` with new fields

**RT-M4: Core Service Handlers**
1. Gmail: search, threads, messages, send, labels (most subcommands)
2. Calendar: events list/get/create/update/delete, calendars list, freebusy
3. Drive: list, search, get, move, copy, trash, permissions

**RT-M5: File I/O**
1. Drive download (streaming GET with file output)
2. Drive upload (simple multipart POST)
3. Gmail attachment download
4. Create `services/export.rs` for document export

**RT-M6: Extended Service Handlers**
- Docs, Sheets, Slides, Forms, Chat, Tasks, Classroom, Contacts, People, Groups, Keep, Apps Script
- Each follows the identical pattern: bootstrap -> match subcommand -> build URL -> api_get/post -> write_output

**RT-M7: Polish**
1. Remote OAuth flow (`run_remote_flow`)
2. Encrypted file backend for credential store
3. Keyring timeout handling
4. Resumable upload for large files (>5MB)

## Requirement Traceability

| Requirement ID | Priority | Architecture Section | Module(s) | File(s) |
|---------------|----------|---------------------|-----------|---------|
| REQ-RT-001 | Must | Module 3: auth/oauth | auth/oauth | `src/auth/oauth.rs` |
| REQ-RT-002 | Must | Module 4: auth/oauth_flow | auth/oauth_flow | `src/auth/oauth_flow.rs` |
| REQ-RT-003 | Must | Module 4: auth/oauth_flow | auth/oauth_flow | `src/auth/oauth_flow.rs` |
| REQ-RT-004 | Should | Module 4: auth/oauth_flow | auth/oauth_flow | `src/auth/oauth_flow.rs` |
| REQ-RT-005 | Must | Module 2: auth/token | auth/token | `src/auth/token.rs` |
| REQ-RT-006 | Must | Module 6: auth/service_account | auth/service_account | `src/auth/service_account.rs` |
| REQ-RT-007 | Must | Module 1: auth/mod, Module 2: auth/token | auth/mod, auth/token | `src/auth/mod.rs`, `src/auth/token.rs` |
| REQ-RT-008 | Must | Module 11: cli/mod | cli/mod (handle_auth_add) | `src/cli/mod.rs` |
| REQ-RT-009 | Must | Module 11: cli/mod | cli/mod (handle_auth_remove) | `src/cli/mod.rs` |
| REQ-RT-010 | Must | Module 11: cli/mod | cli/mod (handle_auth_list) | `src/cli/mod.rs` |
| REQ-RT-011 | Must | Module 11: cli/mod | cli/mod (handle_auth_status) | `src/cli/mod.rs` |
| REQ-RT-012 | Must | Module 11: cli/mod | cli/mod (handle_auth_tokens) | `src/cli/mod.rs` |
| REQ-RT-013 | Must | Module 5: auth/keyring | auth/keyring (KeyringCredentialStore) | `src/auth/keyring.rs` |
| REQ-RT-014 | Should | Module 5: auth/keyring | auth/keyring (encrypted file) | `src/auth/keyring.rs` |
| REQ-RT-015 | Must | Module 5: auth/keyring | auth/keyring (credential_store_factory) | `src/auth/keyring.rs` |
| REQ-RT-016 | Should | Module 5: auth/keyring | auth/keyring (timeout handling) | `src/auth/keyring.rs` |
| REQ-RT-017 | Must | Module 9: services/mod | services/mod (bootstrap_service_context) | `src/services/mod.rs` |
| REQ-RT-018 | Must | Module 9: services/mod | services/mod (ServiceContext) | `src/services/mod.rs` |
| REQ-RT-019 | Must | Module 7: http/api | http/api (api_get) | `src/http/api.rs` |
| REQ-RT-020 | Must | Module 7: http/api | http/api (api_post, api_patch, api_delete) | `src/http/api.rs` |
| REQ-RT-021 | Must | Module 7: http/api, Module 9: services/mod | http/api + services/mod | `src/http/api.rs`, `src/services/mod.rs` |
| REQ-RT-022 | Must | Module 7: http/api | http/api (error handling) | `src/http/api.rs` |
| REQ-RT-023 | Must | Module 8: services/pagination | services/pagination (paginate) | `src/services/pagination.rs` |
| REQ-RT-024 | Must | Module 8: services/pagination | services/pagination (single page) | `src/services/pagination.rs` |
| REQ-RT-025 | Must | Module 8: services/pagination | services/pagination (--fail-empty) | `src/services/pagination.rs` |
| REQ-RT-026 | Must | Module 10: services/export, cli/mod | drive handler | `src/services/drive/files.rs`, `src/cli/mod.rs` |
| REQ-RT-027 | Must | Module 10: services/export | drive export | `src/services/export.rs`, `src/services/drive/files.rs` |
| REQ-RT-028 | Must | Module 11: cli/mod | drive upload handler | `src/services/drive/files.rs`, `src/cli/mod.rs` |
| REQ-RT-029 | Should | Module 11: cli/mod | drive resumable upload | `src/services/drive/files.rs` |
| REQ-RT-030 | Must | Module 11: cli/mod | gmail attachment download | `src/services/gmail/message.rs`, `src/cli/mod.rs` |
| REQ-RT-031 | Should | Module 10: services/export | shared export | `src/services/export.rs` |
| REQ-RT-032 | Must | Module 11: cli/mod | gmail search handler | `src/cli/mod.rs` |
| REQ-RT-033 | Must | Module 11: cli/mod | gmail message search handler | `src/cli/mod.rs` |
| REQ-RT-034 | Must | Module 11: cli/mod | gmail thread get handler | `src/cli/mod.rs` |
| REQ-RT-035 | Must | Module 11: cli/mod | gmail message get handler | `src/cli/mod.rs` |
| REQ-RT-036 | Must | Module 11: cli/mod | gmail send handler | `src/cli/mod.rs` |
| REQ-RT-037 | Must | Module 11: cli/mod | gmail labels handler | `src/cli/mod.rs` |
| REQ-RT-038 | Should | Module 11: cli/mod | gmail drafts handler | `src/cli/mod.rs` |
| REQ-RT-039 | Must | Module 11: cli/mod | gmail modify handler | `src/cli/mod.rs` |
| REQ-RT-040 | Must | Module 11: cli/mod | gmail trash handler | `src/cli/mod.rs` |
| REQ-RT-041 | Should | Module 11: cli/mod | gmail batch handler | `src/cli/mod.rs` |
| REQ-RT-042 | Should | Module 11: cli/mod | gmail history handler | `src/cli/mod.rs` |
| REQ-RT-043 | Could | Module 11: cli/mod | gmail settings handler | `src/cli/mod.rs` |
| REQ-RT-044 | Must | Module 11: cli/mod | calendar events list handler | `src/cli/mod.rs` |
| REQ-RT-045 | Must | Module 11: cli/mod | calendar events get handler | `src/cli/mod.rs` |
| REQ-RT-046 | Must | Module 11: cli/mod | calendar events create handler | `src/cli/mod.rs` |
| REQ-RT-047 | Must | Module 11: cli/mod | calendar events update handler | `src/cli/mod.rs` |
| REQ-RT-048 | Must | Module 11: cli/mod | calendar events delete handler | `src/cli/mod.rs` |
| REQ-RT-049 | Must | Module 11: cli/mod | calendar calendars list handler | `src/cli/mod.rs` |
| REQ-RT-050 | Must | Module 11: cli/mod | calendar freebusy handler | `src/cli/mod.rs` |
| REQ-RT-051 | Should | Module 11: cli/mod | calendar respond handler | `src/cli/mod.rs` |
| REQ-RT-052 | Should | Module 11: cli/mod | calendar search handler | `src/cli/mod.rs` |
| REQ-RT-053 | Should | Module 11: cli/mod | calendar calendars CRUD handler | `src/cli/mod.rs` |
| REQ-RT-054 | Should | Module 11: cli/mod | calendar colors handler | `src/cli/mod.rs` |
| REQ-RT-055 | Must | Module 11: cli/mod | drive list handler | `src/cli/mod.rs` |
| REQ-RT-056 | Must | Module 11: cli/mod | drive search handler | `src/cli/mod.rs` |
| REQ-RT-057 | Must | Module 11: cli/mod | drive get handler | `src/cli/mod.rs` |
| REQ-RT-058 | Must | Module 11: cli/mod | drive download handler | `src/cli/mod.rs` |
| REQ-RT-059 | Must | Module 11: cli/mod | drive upload handler | `src/cli/mod.rs` |
| REQ-RT-060 | Must | Module 11: cli/mod | drive mkdir handler | `src/cli/mod.rs` |
| REQ-RT-061 | Must | Module 11: cli/mod | drive move handler | `src/cli/mod.rs` |
| REQ-RT-062 | Must | Module 11: cli/mod | drive copy handler | `src/cli/mod.rs` |
| REQ-RT-063 | Must | Module 11: cli/mod | drive trash handler | `src/cli/mod.rs` |
| REQ-RT-064 | Must | Module 11: cli/mod | drive permissions list handler | `src/cli/mod.rs` |
| REQ-RT-065 | Must | Module 11: cli/mod | drive permissions create handler | `src/cli/mod.rs` |
| REQ-RT-066 | Must | Module 11: cli/mod | drive info handler | `src/cli/mod.rs` |
| REQ-RT-067 | Should | Module 11: cli/mod | drive shared drives handler | `src/cli/mod.rs` |
| REQ-RT-068 | Could | Module 11: cli/mod | drive comments handler | `src/cli/mod.rs` |
| REQ-RT-069 | Should | Module 11: cli/mod | docs handler | `src/cli/mod.rs` |
| REQ-RT-070 | Should | Module 11: cli/mod | sheets handler | `src/cli/mod.rs` |
| REQ-RT-071 | Should | Module 11: cli/mod | slides handler | `src/cli/mod.rs` |
| REQ-RT-072 | Should | Module 11: cli/mod | forms handler | `src/cli/mod.rs` |
| REQ-RT-073 | Should | Module 11: cli/mod | chat handler | `src/cli/mod.rs` |
| REQ-RT-074 | Could | Module 11: cli/mod | classroom handler | `src/cli/mod.rs` |
| REQ-RT-075 | Should | Module 11: cli/mod | tasks handler | `src/cli/mod.rs` |
| REQ-RT-076 | Should | Module 11: cli/mod | contacts handler | `src/cli/mod.rs` |
| REQ-RT-077 | Should | Module 11: cli/mod | people handler | `src/cli/mod.rs` |
| REQ-RT-078 | Could | Module 11: cli/mod | groups handler | `src/cli/mod.rs` |
| REQ-RT-079 | Could | Module 11: cli/mod | keep handler | `src/cli/mod.rs` |
| REQ-RT-080 | Could | Module 11: cli/mod | appscript handler | `src/cli/mod.rs` |
| REQ-RT-081 | Must | Module 7: http/api | http/api (verbose logging) | `src/http/api.rs` |
| REQ-RT-082 | Must | Module 7: http/api, Module 11: cli/mod | http/api (dry-run), cli/mod | `src/http/api.rs`, `src/cli/mod.rs` |
