# Architecture: OMEGA Integration

## Scope

OMEGA_STORES_DIR credential unification, Gmail/Calendar/Drive push notification watch commands, and webhook serve (testing). Covers 24 requirements (REQ-OI-001 through REQ-OI-026).

## Overview

This feature adds four capabilities to omega-google:

1. **OMEGA store credential backend** -- reads credentials from `$OMEGA_STORES_DIR/google.json` instead of the normal keyring/file stores, enabling OMEGA to manage credentials centrally.
2. **Gmail watch handlers** -- wires the existing CLI args and URL builders to actual API calls for start/stop/status.
3. **Calendar/Drive watch commands** -- new CLI subcommands and service handlers for push notification registration via Google's webhook API.
4. **Webhook serve** -- a minimal HTTP server for testing push notification delivery locally.

```
 OMEGA_STORES_DIR/google.json
         |
  +------v-----------+
  | OmegaStoreCred   |  <-- new CredentialStore impl
  | (src/auth/       |
  |  omega_store.rs) |
  +------+-----------+
         |
  credential_store_factory() -- priority check before keyring/file logic
         |
  +------v-----------+     +-------------------+
  | bootstrap_       |---->| Watch handlers    |
  | service_context  |     | (gmail/cal/drive) |
  +------------------+     +--------+----------+
                                    |
                            +-------v-------+
                            | api_post /    |
                            | api_get       |
                            | (existing)    |
                            +---------------+

  +-------------------+
  | webhook serve     |  <-- standalone HTTP server (no ServiceContext)
  | (src/webhook/)    |
  +-------------------+
```

## Modules

### Module 1: auth/omega_store -- OMEGA Store Credential Backend

- **Responsibility**: Read/write credentials from `$OMEGA_STORES_DIR/google.json`; implement the `CredentialStore` trait for the OMEGA store format.
- **File**: `src/auth/omega_store.rs` (new file)
- **Public interface**:
  ```rust
  /// JSON schema for $OMEGA_STORES_DIR/google.json
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct OmegaStoreData {
      pub version: u32,
      pub client_id: String,
      pub client_secret: String,
      pub refresh_token: String,
      pub email: String,
  }

  pub struct OmegaStoreCredentialStore {
      path: PathBuf,  // full path to google.json
  }

  impl OmegaStoreCredentialStore {
      pub fn new(stores_dir: &str) -> anyhow::Result<Self>;
      pub fn read_store_data(&self) -> anyhow::Result<OmegaStoreData>;
      pub fn client_credentials(&self) -> anyhow::Result<ClientCredentials>;
  }

  impl CredentialStore for OmegaStoreCredentialStore {
      // get_token: reads google.json, constructs TokenData
      // set_token: atomic write back (only refresh_token updated)
      // delete_token: returns error (not permitted)
      // list_tokens: single-element vec
      // keys: single key
      // get_default_account: email from google.json
      // set_default_account: no-op
  }

  /// Check if OMEGA_STORES_DIR is set and contains google.json
  pub fn is_omega_store_active() -> bool;
  ```
- **Dependencies**: `crate::auth::{CredentialStore, TokenData, Service}`, `crate::config::ClientCredentials`, `serde`, `std::fs`
- **Implementation order**: 1 (foundation for all other modules)

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| google.json missing | OMEGA_STORES_DIR set but file absent | `std::fs::read_to_string` returns `NotFound` | Error with actionable message: "OMEGA_STORES_DIR is set but {path}/google.json not found" | All commands fail; user must create the file |
| Malformed JSON | Syntax error in google.json | `serde_json::from_str` returns `Err` | Error: "OMEGA_STORES_DIR is set but {path}/google.json is not valid JSON" | Same as above |
| Missing field | JSON valid but lacks required key | Custom validation after parse | Error: "...missing required field: {field}" | Same as above |
| Write conflict | Concurrent process writes google.json | Atomic write (tmp + rename) | Last writer wins; no corruption | At worst, one refresh token is stale; next refresh will fix it |
| Permission denied | File/dir not readable | `std::fs::read_to_string` error | Propagate OS error | User must fix permissions |

#### Security Considerations
- **Trust boundary**: `google.json` path comes from env var (trusted -- set by OMEGA). File contents are trusted (OMEGA manages them).
- **Sensitive data**: `client_secret` and `refresh_token` are confidential. File must be 0600. Debug impl must redact token values.
- **Attack surface**: A malicious `OMEGA_STORES_DIR` could point to an arbitrary file. Mitigated by: we only read the specific expected JSON schema, no path traversal.
- **Mitigations**: Atomic writes (tmp+rename) prevent corruption. 0600 permissions on Unix. No logging of token values.

#### Performance Budget
- **Latency target**: < 5ms for `read_store_data()` (single file read + JSON parse)
- **Memory budget**: < 1KB (small JSON file)
- **Complexity target**: O(1) for all operations (single file, single account)

### Module 2: auth/keyring + services/mod (integration points)

- **Responsibility**: Wire the OMEGA store into the existing credential factory and service bootstrap.
- **Files**: `src/auth/keyring.rs` (modify), `src/services/mod.rs` (modify), `src/auth/mod.rs` (modify `pub mod`)
- **Public interface changes**:
  ```rust
  // In credential_store_factory():
  // NEW: check OMEGA_STORES_DIR BEFORE the backend variable
  pub fn credential_store_factory(
      config: &ConfigFile,
  ) -> anyhow::Result<Box<dyn CredentialStore>> {
      // ---- NEW BLOCK (top of function) ----
      if let Ok(stores_dir) = std::env::var("OMEGA_STORES_DIR") {
          return Ok(Box::new(
              OmegaStoreCredentialStore::new(&stores_dir)?
          ));
      }
      // ---- existing logic unchanged ----
  }

  // In bootstrap_service_context():
  // Step 6 (refresh): detect OMEGA store and extract ClientCredentials
  // from the store instead of calling read_client_credentials()
  ```
- **Dependencies**: `crate::auth::omega_store::OmegaStoreCredentialStore`
- **Implementation order**: 1 (same milestone as Module 1)

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| OMEGA store + keyring_backend conflict | User sets both OMEGA_STORES_DIR and GOG_KEYRING_BACKEND | N/A -- OMEGA_STORES_DIR takes priority | OMEGA store wins; keyring_backend ignored | None (intended behavior) |
| Token refresh fails | Invalid client_id/secret in google.json | HTTP 401 from Google token endpoint | Error propagated; user must fix google.json | Command fails; credential file may need updating |

### Module 3: services/gmail/watch -- Gmail Watch Handlers

- **Responsibility**: Implement `watch_start`, `watch_stop`, `watch_status` handler functions that call the Gmail API using existing URL builders and `api_post`/`api_get` helpers.
- **File**: `src/services/gmail/watch.rs` (extend existing file)
- **Public interface**:
  ```rust
  // Response types (add to types.rs or define in watch.rs)
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct GmailWatchResponse {
      pub history_id: u64,
      pub expiration: String,  // milliseconds since epoch as string
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct GmailWatchRequest {
      pub topic_name: String,
      #[serde(skip_serializing_if = "Vec::is_empty")]
      pub label_ids: Vec<String>,
  }

  // Existing URL builders (already present):
  pub fn build_watch_start_url() -> String;
  pub fn build_watch_stop_url() -> String;

  // NEW handler functions:
  pub async fn watch_start(
      ctx: &ServiceContext,
      topic: &str,
      label_ids: &[String],
  ) -> anyhow::Result<Option<GmailWatchResponse>>;

  pub async fn watch_stop(
      ctx: &ServiceContext,
  ) -> anyhow::Result<()>;

  // build_profile_url() for watch status
  pub fn build_profile_url() -> String;
  ```
- **Dependencies**: `crate::http::api::{api_post, api_post_empty, api_get}`, `crate::services::ServiceContext`, `crate::services::gmail::types::GMAIL_BASE_URL`
- **Implementation order**: 2

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Permission denied (403) | Pub/Sub topic missing publisher role for `gmail-api-push@system.gserviceaccount.com` | HTTP 403 response | Print troubleshooting hint about Pub/Sub IAM | Watch not registered |
| Invalid topic (400) | Topic name doesn't exist or wrong format | HTTP 400 response | Print: "Ensure topic exists in format projects/{project}/topics/{topic}" | Watch not registered |
| Already watching | Duplicate watch start call | Success (Google returns new expiration) | No issue -- idempotent | None |

#### Performance Budget
- **Latency target**: Network-bound (< 2s p99, depends on Google API)
- **Memory budget**: < 1KB per response

### Module 4: services/calendar/watch -- Calendar Watch Handlers

- **Responsibility**: Implement Calendar push notification watch start/stop/status.
- **File**: `src/services/calendar/watch.rs` (new file)
- **Public interface**:
  ```rust
  use crate::services::calendar::types::CALENDAR_BASE_URL;

  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct WatchChannelRequest {
      pub id: String,        // UUID v4
      #[serde(rename = "type")]
      pub channel_type: String,  // "web_hook"
      pub address: String,   // callback URL
      #[serde(skip_serializing_if = "Option::is_none")]
      pub params: Option<WatchParams>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct WatchParams {
      pub ttl: String,  // seconds, e.g. "604800"
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct WatchChannelResponse {
      pub id: String,
      pub resource_id: String,
      pub expiration: String,  // milliseconds since epoch
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct ChannelStopRequest {
      pub id: String,
      pub resource_id: String,
  }

  pub fn build_calendar_watch_url(calendar_id: &str) -> String;
  pub fn build_calendar_stop_url() -> String;

  pub async fn watch_start(
      ctx: &ServiceContext,
      calendar_id: &str,
      callback_url: &str,
  ) -> anyhow::Result<Option<WatchChannelResponse>>;

  pub async fn watch_stop(
      ctx: &ServiceContext,
      channel_id: &str,
      resource_id: &str,
  ) -> anyhow::Result<()>;
  ```
- **Dependencies**: `crate::http::api::{api_post, api_post_empty}`, `uuid`, `crate::services::ServiceContext`
- **Implementation order**: 3

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Domain not verified (403) | Callback URL domain not verified in Google Search Console | HTTP 403 | Print: "Callback URL must be HTTPS with domain verified in Google Search Console" | Watch not registered |
| Invalid channel stop (404) | Channel ID or resource ID doesn't match | HTTP 404 | Print: "Channel not found. It may have already expired." | No impact (channel already gone) |

### Module 5: services/drive/watch -- Drive Watch Handlers

- **Responsibility**: Implement Drive push notification watch start/stop/status.
- **File**: `src/services/drive/watch.rs` (new file)
- **Public interface**:
  ```rust
  use crate::services::drive::types::DRIVE_BASE_URL;

  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct StartPageTokenResponse {
      pub start_page_token: String,
  }

  // Reuses WatchChannelRequest, WatchChannelResponse, ChannelStopRequest
  // from calendar::watch (or define common types in services/common.rs)

  pub fn build_start_page_token_url() -> String;
  pub fn build_changes_watch_url(page_token: &str) -> String;
  pub fn build_drive_stop_url() -> String;

  pub async fn watch_start(
      ctx: &ServiceContext,
      callback_url: &str,
  ) -> anyhow::Result<Option<(WatchChannelResponse, String)>>;
  // Returns (channel_response, start_page_token)

  pub async fn watch_stop(
      ctx: &ServiceContext,
      channel_id: &str,
      resource_id: &str,
  ) -> anyhow::Result<()>;

  pub async fn get_start_page_token(
      ctx: &ServiceContext,
  ) -> anyhow::Result<StartPageTokenResponse>;
  ```
- **Dependencies**: `crate::http::api::{api_post, api_post_empty, api_get}`, `uuid`, `crate::services::ServiceContext`
- **Implementation order**: 3 (same milestone as calendar watch)

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Domain not verified (403) | Same as Calendar | HTTP 403 | Same message | Watch not registered |
| Invalid page token (400) | Stale or malformed token | HTTP 400 | Print: "Page token invalid. Re-run 'drive watch start' to get a fresh token." | Watch not registered |

### Module 6: webhook -- Webhook Serve

- **Responsibility**: Minimal HTTP server that receives and logs Google push notification webhooks for testing.
- **File**: `src/webhook/mod.rs` (new module at top level, not under services/)
- **Public interface**:
  ```rust
  pub async fn serve(bind: &str, port: u16) -> anyhow::Result<()>;
  ```
- **Dependencies**: `tokio`, `hyper` (HTTP/1.1 server only -- lighter than axum)
- **Implementation order**: 4

#### Design Decision: hyper vs axum

Use **hyper** directly (v1.x) with `hyper-util` for the HTTP listener. Rationale:
- This server has 4 routes and no middleware. axum would add ~15 transitive dependencies for no benefit.
- hyper is already a transitive dependency of reqwest, so it adds zero new crate downloads.
- The entire server is ~100 lines of code.

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Port in use | Another process on the same port | `TcpListener::bind` error | Print: "Failed to bind to {bind}:{port}: Address already in use" | Server doesn't start |
| Invalid body | Non-JSON or empty POST body | Graceful handling | Log raw body bytes; still respond 200 | No impact (testing tool) |
| Ctrl+C | User interrupt | `tokio::signal::ctrl_c()` | Graceful shutdown | Clean exit |

#### Security Considerations
- **Trust boundary**: This is a **testing-only** tool. It accepts all POST requests without authentication. It is NOT intended for production webhook receipt.
- **Mitigations**: Binds to user-chosen interface (default 0.0.0.0). No state mutation. No file writes. Stdout-only output.

#### Performance Budget
- **Latency target**: < 1ms response time (just logs and returns 200)
- **Memory budget**: < 10MB RSS
- **Throughput target**: Sufficient for testing (tens of req/s)

### Module 7: CLI Wiring

- **Responsibility**: Add CLI subcommands for calendar watch, drive watch, and webhook serve; wire all watch/webhook dispatchers in `cli/mod.rs`.
- **Files**: `src/cli/calendar.rs` (modify), `src/cli/drive.rs` (modify), `src/cli/root.rs` (modify), `src/cli/mod.rs` (modify)
- **Public interface changes**:
  ```rust
  // calendar.rs -- add to CalendarCommand enum:
  /// Manage calendar push notification watch
  Watch(CalendarWatchArgs),

  #[derive(Args, Debug)]
  pub struct CalendarWatchArgs {
      #[command(subcommand)]
      pub command: CalendarWatchCommand,
  }

  #[derive(Subcommand, Debug)]
  pub enum CalendarWatchCommand {
      /// Start watching calendar events
      Start(CalendarWatchStartArgs),
      /// Stop watching
      Stop(CalendarWatchStopArgs),
      /// Show watch status
      Status,
  }

  #[derive(Args, Debug)]
  pub struct CalendarWatchStartArgs {
      /// Callback URL (must be HTTPS)
      #[arg(long)]
      pub callback_url: String,
      /// Calendar ID (default: "primary")
      #[arg(long, default_value = "primary")]
      pub calendar: String,
  }

  #[derive(Args, Debug)]
  pub struct CalendarWatchStopArgs {
      /// Channel ID from watch start response
      #[arg(long)]
      pub channel_id: String,
      /// Resource ID from watch start response
      #[arg(long)]
      pub resource_id: String,
  }

  // drive.rs -- add to DriveCommand enum:
  /// Manage drive push notification watch
  Watch(DriveWatchArgs),

  #[derive(Args, Debug)]
  pub struct DriveWatchArgs {
      #[command(subcommand)]
      pub command: DriveWatchCommand,
  }

  #[derive(Subcommand, Debug)]
  pub enum DriveWatchCommand {
      /// Start watching drive changes
      Start(DriveWatchStartArgs),
      /// Stop watching
      Stop(DriveWatchStopArgs),
      /// Show watch status
      Status,
  }

  #[derive(Args, Debug)]
  pub struct DriveWatchStartArgs {
      /// Callback URL (must be HTTPS)
      #[arg(long)]
      pub callback_url: String,
  }

  #[derive(Args, Debug)]
  pub struct DriveWatchStopArgs {
      /// Channel ID from watch start response
      #[arg(long)]
      pub channel_id: String,
      /// Resource ID from watch start response
      #[arg(long)]
      pub resource_id: String,
  }

  // root.rs -- add to Command enum:
  /// Webhook utilities (serve, test)
  Webhook(WebhookArgs),

  #[derive(Args, Debug)]
  pub struct WebhookArgs {
      #[command(subcommand)]
      pub command: WebhookCommand,
  }

  #[derive(Subcommand, Debug)]
  pub enum WebhookCommand {
      /// Start a local webhook receiver for testing
      Serve(WebhookServeArgs),
  }

  #[derive(Args, Debug)]
  pub struct WebhookServeArgs {
      /// Port to listen on
      #[arg(long, default_value = "8765")]
      pub port: u16,
      /// Address to bind to
      #[arg(long, default_value = "0.0.0.0")]
      pub bind: String,
  }
  ```
- **Dependencies**: All watch/webhook service modules
- **Implementation order**: Split across milestones (M2 for gmail, M3 for cal/drive, M4 for webhook)

## Shared Types: Watch Channel Types

Calendar and Drive watch use identical request/response schemas (`WatchChannelRequest`, `WatchChannelResponse`, `ChannelStopRequest`). To avoid duplication:

- Define these shared types in `src/services/common.rs` (or a new `src/services/watch_types.rs`)
- Both `calendar::watch` and `drive::watch` import from there
- Gmail watch has a **different** response schema (historyId + expiration), so it keeps its own types

## Milestones

| ID | Name | Scope (Modules) | Scope (Requirements) | Est. Size | Dependencies |
|----|------|-----------------|---------------------|-----------|-------------|
| OI-M1 | OMEGA Store Credentials | auth/omega_store, auth/keyring (factory patch), services/mod (bootstrap patch) | REQ-OI-001 to REQ-OI-007 | M | None |
| OI-M2 | Gmail Watch Commands | services/gmail/watch (handlers), cli/mod (gmail watch dispatch) | REQ-OI-010 to REQ-OI-013 | S | OI-M1 |
| OI-M3 | Calendar + Drive Watch | services/calendar/watch, services/drive/watch, services/common (watch types), cli/calendar, cli/drive, Cargo.toml (uuid) | REQ-OI-014 to REQ-OI-019, REQ-OI-023, REQ-OI-025, REQ-OI-026 | M | OI-M1 |
| OI-M4 | Webhook Serve | webhook/mod, cli/root (Webhook variant), cli/mod (webhook dispatch), Cargo.toml (hyper) | REQ-OI-020 to REQ-OI-022, REQ-OI-024 | S | None (independent) |

### Milestone Details

**OI-M1: OMEGA Store Credentials**
- Create `src/auth/omega_store.rs` with `OmegaStoreCredentialStore`
- Patch `credential_store_factory()` in `src/auth/keyring.rs` (add OMEGA_STORES_DIR check at top)
- Patch `bootstrap_service_context()` in `src/services/mod.rs` (extract ClientCredentials from OMEGA store)
- Add `pub mod omega_store;` to `src/auth/mod.rs`
- Update `auth status` handler in `src/cli/mod.rs` to show OMEGA store mode
- 3 files changed, 1 file created = 2 effective modules (M)

**OI-M2: Gmail Watch Commands**
- Extend `src/services/gmail/watch.rs` with handler functions (watch_start, watch_stop, watch_status)
- Add response/request types to `src/services/gmail/watch.rs` (or `types.rs`)
- Wire `GmailCommand::Watch` dispatch in `src/cli/mod.rs`
- Remove `Renew` variant from `GmailWatchCommand` (not in spec)
- 2 files changed = small (S)

**OI-M3: Calendar + Drive Watch**
- Create `src/services/calendar/watch.rs` + add `pub mod watch;` to calendar's mod.rs
- Create `src/services/drive/watch.rs` + add `pub mod watch;` to drive's mod.rs
- Add shared watch types to `src/services/common.rs`
- Add `Watch` variant to `CalendarCommand` and `DriveCommand` in their respective cli files
- Wire dispatch in `src/cli/mod.rs`
- Add `uuid = { version = "1", features = ["v4"] }` to `Cargo.toml`
- 3 new files, 5 files modified = 3 effective modules (M)

**OI-M4: Webhook Serve**
- Create `src/webhook/mod.rs` (new top-level module)
- Add `pub mod webhook;` to `src/main.rs` or `src/lib.rs`
- Add `Webhook` variant to `Command` enum in `src/cli/root.rs`
- Wire dispatch in `src/cli/mod.rs`
- Add `hyper = { version = "1", features = ["server", "http1"] }` and `hyper-util = { version = "0.1", features = ["tokio"] }` and `http-body-util = "0.1"` to `Cargo.toml`
- 1 new file, 4 files modified = 2 effective modules (S)

## Failure Modes (system-level)

| Scenario | Affected Modules | Detection | Recovery Strategy | Degraded Behavior |
|----------|-----------------|-----------|-------------------|-------------------|
| OMEGA_STORES_DIR set but google.json missing | auth/omega_store | File read error | Actionable error message with path | All commands fail |
| Google API 401 during watch | gmail/calendar/drive watch | HTTP 401 status | Trigger token refresh, retry once | If refresh fails, error with re-auth guidance |
| Google API 403 on watch start | gmail/calendar/drive watch | HTTP 403 status | Print specific troubleshooting (Pub/Sub IAM or domain verification) | Watch not registered |
| Token refresh writes stale refresh_token | auth/omega_store | N/A (race condition) | Next command triggers fresh refresh | Brief window of stale token; self-healing |
| Webhook port conflict | webhook | Bind error | Print port-in-use message, suggest --port | Server doesn't start |

## Security Model

### Trust Boundaries
- **OMEGA_STORES_DIR path**: Trusted (set by OMEGA, not user input from network)
- **google.json contents**: Trusted (OMEGA writes it; we validate schema defensively)
- **Google API responses**: Semi-trusted (validated by serde deserialization)
- **Webhook POST bodies**: Untrusted (any HTTP client can send requests to the test server)

### Data Classification
| Data | Classification | Storage | Access Control |
|------|---------------|---------|---------------|
| client_secret | Secret | google.json (0600) | OMEGA process only |
| refresh_token | Secret | google.json (0600) | OMEGA process only |
| access_token | Confidential | In-memory only | Process lifetime |
| watch channel IDs | Internal | Printed to stdout (ephemeral) | User only |
| webhook bodies | Internal | Printed to stdout (ephemeral) | User only |

### Attack Surface
- **OMEGA store file read**: Low risk -- path from env var, schema-validated parse
- **Webhook server**: Medium risk -- accepts unauthenticated POSTs. Mitigated by: testing-only, no side effects, no state mutation, no file writes.

## Graceful Degradation

| Dependency | Normal Behavior | Degraded Behavior | User Impact |
|-----------|----------------|-------------------|-------------|
| google.json | Full credential access | Error with actionable fix message | Must fix file before any command works |
| Gmail push API | Watch registered | Print troubleshooting hints | Must fix Pub/Sub permissions manually |
| Calendar/Drive push API | Watch registered | Print domain verification hints | Must verify domain in Search Console |
| Webhook port | Server starts on requested port | Suggest alternative port | Must choose different port |

## Performance Budgets

| Operation | Latency (p50) | Latency (p99) | Memory | Notes |
|-----------|---------------|---------------|--------|-------|
| OMEGA store read | < 1ms | < 5ms | < 1KB | Single file read |
| OMEGA store write (atomic) | < 5ms | < 20ms | < 1KB | tmp + rename |
| Watch start (any service) | < 500ms | < 2s | < 4KB | Network-bound |
| Watch stop (any service) | < 500ms | < 2s | < 1KB | Network-bound |
| Webhook request handling | < 1ms | < 5ms | < 64KB | Log + 200 response |

## Data Flow

### OMEGA Store Credential Flow
```
1. CLI invocation
2. credential_store_factory() checks OMEGA_STORES_DIR env
3. If set: OmegaStoreCredentialStore::new(stores_dir)
4. bootstrap_service_context() calls store.get_token()
   -> reads google.json, constructs TokenData
5. If refresh needed:
   a. store.client_credentials() -> extract ClientCredentials from google.json
   b. refresh_access_token() -> get new access_token
   c. store.set_token() -> atomic write refresh_token back to google.json
6. Build authenticated HTTP client with access_token
```

### Watch Registration Flow (Calendar/Drive)
```
1. User: omg-gog calendar watch start --callback-url https://example.com/hook
2. CLI: parse CalendarWatchStartArgs
3. Handler: generate UUID v4 for channel ID
4. Handler: build WatchChannelRequest { id, type: "web_hook", address, params: {ttl} }
5. Handler: api_post(url, body) -> WatchChannelResponse
6. Handler: print channel ID, resource ID, expiration
7. Handler: print note about HTTPS domain verification
```

### Watch Registration Flow (Gmail)
```
1. User: omg-gog gmail watch start --topic projects/myproject/topics/gmail
2. CLI: parse GmailWatchStartArgs
3. Handler: build GmailWatchRequest { topicName, labelIds }
4. Handler: api_post(url, body) -> GmailWatchResponse
5. Handler: print historyId, expiration datetime
6. Handler: print note about Pub/Sub IAM
```

## Design Decisions

| Decision | Alternatives Considered | Justification |
|----------|------------------------|---------------|
| New file `omega_store.rs` vs extend `keyring.rs` | Adding to keyring.rs | keyring.rs is already 500+ lines with 3 implementations. Separate file follows single-responsibility and matches the pattern of other auth modules. |
| OMEGA_STORES_DIR check in factory (top priority) | Separate init path | Factory is the single creation point for all CredentialStore instances. Adding a priority check at the top is minimal change and ensures OMEGA store always wins. |
| hyper over axum for webhook | axum, warp, actix-web | 4 routes, no middleware, ~100 lines. hyper is already a transitive dep of reqwest. axum adds 15+ deps for no benefit. |
| Shared watch types in common.rs | Duplicate in each module | Calendar and Drive use identical Google Channel API schema. DRY. |
| UUID v4 for channel IDs | User-provided IDs, sequential IDs | Google expects unique channel IDs. UUID v4 is standard, collision-free, no state needed. |
| auth status shows OMEGA mode | Silent operation | Users need to verify which credential source is active. Explicit is better than implicit. |

## External Dependencies

- `uuid = { version = "1", features = ["v4"] }` -- Generate channel IDs for Calendar/Drive watch (OI-M3)
- `hyper = { version = "1", features = ["server", "http1"] }` -- HTTP server for webhook serve (OI-M4)
- `hyper-util = { version = "0.1", features = ["tokio"] }` -- Tokio integration for hyper server (OI-M4)
- `http-body-util = "0.1"` -- Body utilities for hyper (OI-M4)

## Requirement Traceability

| Requirement ID | Priority | Architecture Section | Module(s) | File(s) |
|---------------|----------|---------------------|-----------|---------|
| REQ-OI-001 | Must | Module 1: auth/omega_store | OmegaStoreCredentialStore | `src/auth/omega_store.rs` |
| REQ-OI-002 | Must | Module 1: auth/omega_store | CredentialStore impl | `src/auth/omega_store.rs` |
| REQ-OI-003 | Must | Module 2: integration points | credential_store_factory | `src/auth/keyring.rs` |
| REQ-OI-004 | Must | Module 2: integration points | bootstrap_service_context | `src/services/mod.rs` |
| REQ-OI-005 | Must | Module 1: auth/omega_store | atomic write | `src/auth/omega_store.rs` |
| REQ-OI-006 | Must | Module 7: CLI wiring | auth status handler | `src/cli/mod.rs` |
| REQ-OI-007 | Should | Module 1: auth/omega_store | error messages | `src/auth/omega_store.rs` |
| REQ-OI-010 | Must | Module 3: gmail watch | watch_start | `src/services/gmail/watch.rs`, `src/cli/mod.rs` |
| REQ-OI-011 | Must | Module 3: gmail watch | watch_stop | `src/services/gmail/watch.rs`, `src/cli/mod.rs` |
| REQ-OI-012 | Must | Module 3: gmail watch | watch_status (profile) | `src/services/gmail/watch.rs`, `src/cli/mod.rs` |
| REQ-OI-013 | Should | Module 3: gmail watch | Pub/Sub hints | `src/cli/mod.rs` |
| REQ-OI-014 | Must | Module 4: calendar watch | watch_start | `src/services/calendar/watch.rs`, `src/cli/mod.rs` |
| REQ-OI-015 | Must | Module 4: calendar watch | watch_stop | `src/services/calendar/watch.rs`, `src/cli/mod.rs` |
| REQ-OI-016 | Must | Module 4: calendar watch | watch_status (info) | `src/cli/mod.rs` |
| REQ-OI-017 | Must | Module 5: drive watch | watch_start | `src/services/drive/watch.rs`, `src/cli/mod.rs` |
| REQ-OI-018 | Must | Module 5: drive watch | watch_stop | `src/services/drive/watch.rs`, `src/cli/mod.rs` |
| REQ-OI-019 | Must | Module 5: drive watch | watch_status (page token) | `src/services/drive/watch.rs`, `src/cli/mod.rs` |
| REQ-OI-020 | Must | Module 6: webhook | serve | `src/webhook/mod.rs`, `src/cli/mod.rs` |
| REQ-OI-021 | Should | Module 6: webhook | startup banner | `src/webhook/mod.rs` |
| REQ-OI-022 | Should | Module 6: webhook | method filtering | `src/webhook/mod.rs` |
| REQ-OI-023 | Must | External Dependencies | uuid crate | `Cargo.toml` |
| REQ-OI-024 | Must | External Dependencies | hyper crate | `Cargo.toml` |
| REQ-OI-025 | Must | Module 7: CLI wiring | CLI structure | `src/cli/calendar.rs`, `src/cli/drive.rs`, `src/cli/root.rs` |
| REQ-OI-026 | Must | Modules 3/4/5: watch types | response serde types | `src/services/gmail/watch.rs`, `src/services/common.rs` |
