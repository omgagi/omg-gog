# OMEGA Integration Requirements

> Requirements specification for unifying credentials via OMEGA_STORES_DIR and adding Gmail/Calendar/Drive push notification watch commands.

**Scope:** auth module, config module, CLI (gmail/calendar/drive/webhook), services (gmail/calendar/drive), new webhook server module.

**Impact Analysis:**
- `src/auth/keyring.rs` — `credential_store_factory()` needs a new branch for OMEGA store mode
- `src/services/mod.rs` — `bootstrap_service_context()` must detect OMEGA store and extract `ClientCredentials` from `google.json` instead of calling `read_client_credentials()`
- `src/cli/gmail.rs` — Gmail watch CLI args already exist (lines 598-624); handler wiring needed
- `src/services/gmail/watch.rs` — URL builders exist; handler functions needed
- `src/cli/calendar.rs` — New `Watch` variant in `CalendarCommand`
- `src/cli/drive.rs` — New `Watch` variant in `DriveCommand`
- `src/cli/root.rs` — New `Webhook` variant in `Command` enum
- `Cargo.toml` — New dependencies: `uuid`, `hyper`/`axum` (for webhook serve)

---

## 1. OMEGA_STORES_DIR Credential Unification

### REQ-OI-001 (Must): Read credentials from OMEGA store
**User Story:** As OMEGA, I want omg-gog to read credentials from `$OMEGA_STORES_DIR/google.json` so I don't need to sync credentials between two locations.

**Acceptance Criteria:**
1. When `OMEGA_STORES_DIR` env var is set, read `client_id`, `client_secret`, `refresh_token`, `email` from `$OMEGA_STORES_DIR/google.json`
2. Use the `email` field as the current account (bypass normal account resolution)
3. Skip keyring entirely — no OS keyring or file-based credential store access
4. If the file doesn't exist: error with "OMEGA_STORES_DIR is set but $OMEGA_STORES_DIR/google.json is missing or invalid"
5. If the file exists but is malformed (missing fields, bad JSON): same error message

**OMEGA google.json schema:**
```json
{
  "version": 1,
  "client_id": "424288504335-xxx.apps.googleusercontent.com",
  "client_secret": "GOCSPX-xxx",
  "refresh_token": "1//03xxx",
  "email": "user@gmail.com"
}
```

### REQ-OI-002 (Must): OmegaStoreCredentialStore implements CredentialStore
**User Story:** As a developer, I want a new `CredentialStore` implementation that reads/writes the OMEGA store format.

**Acceptance Criteria:**
1. New struct `OmegaStoreCredentialStore` in `src/auth/keyring.rs` (or new file `src/auth/omega_store.rs`)
2. Implements all `CredentialStore` trait methods
3. `get_token()` reads from `google.json` and constructs `TokenData`
4. `set_token()` writes back only the `refresh_token` field (preserves `version`, `client_id`, `client_secret`, `email`)
5. `list_tokens()` returns a single-element vec with the OMEGA store token
6. `keys()` returns a single key
7. `get_default_account()` returns the `email` from `google.json`
8. `set_default_account()` is a no-op (OMEGA store has only one account)
9. `delete_token()` returns an error ("cannot delete OMEGA store token via CLI")

### REQ-OI-003 (Must): credential_store_factory detects OMEGA_STORES_DIR
**User Story:** As OMEGA, I want the credential store to automatically use my store when OMEGA_STORES_DIR is set.

**Acceptance Criteria:**
1. `credential_store_factory()` checks for `OMEGA_STORES_DIR` env var **before** all other backend logic
2. If set, returns `Box<OmegaStoreCredentialStore>` — bypasses keyring_backend config entirely
3. If not set, falls through to existing auto/file/keychain logic unchanged

### REQ-OI-004 (Must): bootstrap_service_context supports OMEGA store credentials
**User Story:** As OMEGA, I want token refresh to work with OMEGA store credentials.

**Acceptance Criteria:**
1. When OMEGA store is active, `bootstrap_service_context()` extracts `ClientCredentials` from `google.json` instead of calling `read_client_credentials(client_name)`
2. Token refresh uses these extracted credentials
3. After refresh, updated `refresh_token` is written back to `google.json` via `OmegaStoreCredentialStore::set_token()`
4. All other bootstrap steps (HTTP client, output mode, UI) work unchanged

### REQ-OI-005 (Must): Atomic write for OMEGA store updates
**User Story:** As OMEGA, I want my google.json to not get corrupted during concurrent access.

**Acceptance Criteria:**
1. Writes to `google.json` use the same atomic write pattern as existing config (tmp file + rename)
2. File permissions set to 0600 on Unix
3. All existing fields in the JSON are preserved (only `refresh_token` is updated)

### REQ-OI-006 (Must): auth status shows OMEGA store mode
**User Story:** As OMEGA, I want `omg-gog auth status` to show that credentials come from the OMEGA store.

**Acceptance Criteria:**
1. When OMEGA_STORES_DIR is active, `auth status` prints: "Source: OMEGA store ($OMEGA_STORES_DIR/google.json)"
2. Shows the email from the store
3. Shows whether an access token is cached

### REQ-OI-007 (Should): OMEGA store error messages are actionable
**Acceptance Criteria:**
1. Missing file: "OMEGA_STORES_DIR is set but {path}/google.json not found"
2. Invalid JSON: "OMEGA_STORES_DIR is set but {path}/google.json is not valid JSON"
3. Missing fields: "OMEGA_STORES_DIR is set but {path}/google.json missing required field: {field}"

---

## 2. Gmail Watch Commands

### REQ-OI-010 (Must): gmail watch start
**User Story:** As OMEGA, I want to register a Gmail push notification watch so I get notified when the inbox changes.

**Acceptance Criteria:**
1. Command: `omg-gog gmail watch start --topic <topic-name> [--label-ids <ids>]`
2. Calls `POST https://gmail.googleapis.com/gmail/v1/users/me/watch`
3. Request body: `{"topicName": "<topic>", "labelIds": ["INBOX"]}` (default) or user-specified label IDs
4. Prints: watch registered, expiration datetime (human-readable), historyId
5. Supports `--json` output with `historyId` and `expiration` fields
6. Respects `--dry-run` (logs request, does not call API)

### REQ-OI-011 (Must): gmail watch stop
**Acceptance Criteria:**
1. Command: `omg-gog gmail watch stop`
2. Calls `POST https://gmail.googleapis.com/gmail/v1/users/me/stop`
3. No request body
4. Prints: "Gmail watch stopped"
5. Respects `--dry-run`

### REQ-OI-012 (Must): gmail watch status
**Acceptance Criteria:**
1. Command: `omg-gog gmail watch status`
2. Calls `GET https://gmail.googleapis.com/gmail/v1/users/me/profile`
3. Prints: current historyId, emailAddress
4. Prints note: "Gmail watch status cannot be queried directly (API limitation)"
5. Supports `--json` output

### REQ-OI-013 (Should): Gmail watch start prints Pub/Sub prerequisites
**Acceptance Criteria:**
1. On successful watch registration, print a note: "Ensure gmail-api-push@system.gserviceaccount.com has Pub/Sub Publisher role on the topic"
2. On error, include troubleshooting hint

---

## 3. Calendar Watch Commands

### REQ-OI-014 (Must): calendar watch start
**User Story:** As OMEGA, I want to register Calendar push notifications to my webhook endpoint.

**Acceptance Criteria:**
1. Command: `omg-gog calendar watch start --callback-url <url> [--calendar <id>]`
2. `--calendar` defaults to "primary"
3. Generates a UUID v4 for the channel ID
4. Calls `POST https://www.googleapis.com/calendar/v3/calendars/{calendarId}/events/watch`
5. Request body: `{"id": "<uuid>", "type": "web_hook", "address": "<callback-url>", "params": {"ttl": "604800"}}`
6. Prints: channel ID, resource ID, expiration datetime
7. Prints note: "Callback URL must be HTTPS with domain verified in Google Search Console"
8. Supports `--json` / `--dry-run`

### REQ-OI-015 (Must): calendar watch stop
**Acceptance Criteria:**
1. Command: `omg-gog calendar watch stop --channel-id <id> --resource-id <id>`
2. Calls `POST https://www.googleapis.com/calendar/v3/channels/stop`
3. Body: `{"id": "<channel-id>", "resourceId": "<resource-id>"}`
4. Both parameters are required
5. Prints: "Calendar watch stopped"
6. Respects `--dry-run`

### REQ-OI-016 (Must): calendar watch status
**Acceptance Criteria:**
1. Command: `omg-gog calendar watch status`
2. Prints: "Calendar watch status cannot be queried (API limitation). Run 'watch start' to renew."
3. No API call needed

---

## 4. Drive Watch Commands

### REQ-OI-017 (Must): drive watch start
**User Story:** As OMEGA, I want to register Drive push notifications for file changes.

**Acceptance Criteria:**
1. Command: `omg-gog drive watch start --callback-url <url>`
2. First calls `GET https://www.googleapis.com/drive/v3/changes/startPageToken` to get current page token
3. Generates UUID v4 for channel ID
4. Calls `POST https://www.googleapis.com/drive/v3/changes/watch?pageToken=<token>`
5. Request body: `{"id": "<uuid>", "type": "web_hook", "address": "<callback-url>", "params": {"ttl": "604800"}}`
6. Prints: channel ID, resource ID, expiration datetime, start page token
7. Supports `--json` / `--dry-run`

### REQ-OI-018 (Must): drive watch stop
**Acceptance Criteria:**
1. Command: `omg-gog drive watch stop --channel-id <id> --resource-id <id>`
2. Calls `POST https://www.googleapis.com/drive/v3/channels/stop`
3. Body: `{"id": "<channel-id>", "resourceId": "<resource-id>"}`
4. Both parameters required
5. Prints: "Drive watch stopped"
6. Respects `--dry-run`

### REQ-OI-019 (Must): drive watch status
**Acceptance Criteria:**
1. Command: `omg-gog drive watch status`
2. Calls `GET https://www.googleapis.com/drive/v3/changes/startPageToken`
3. Prints: current start page token
4. Prints note: "Drive watch status cannot be queried (API limitation)"
5. Supports `--json`

---

## 5. Webhook Serve (Testing)

### REQ-OI-020 (Must): webhook serve command
**User Story:** As a developer, I want a local webhook receiver to test push notification registration.

**Acceptance Criteria:**
1. Command: `omg-gog webhook serve [--port <port>] [--bind <addr>]`
2. Default: `--port 8765 --bind 0.0.0.0`
3. Listens on `POST /webhook/google/gmail`, `POST /webhook/google/calendar`, `POST /webhook/google/drive`
4. Also listens on `POST /` as catch-all
5. For each request: prints timestamp, path, X-Goog-* headers, body (as JSON if valid) to stdout
6. Responds 200 OK to all valid POSTs
7. Ctrl+C to stop gracefully

### REQ-OI-021 (Should): webhook serve prints startup banner
**Acceptance Criteria:**
1. On start, prints: "Listening on {bind}:{port}"
2. Lists the registered routes

### REQ-OI-022 (Should): webhook serve rejects non-POST
**Acceptance Criteria:**
1. GET requests return 405 Method Not Allowed
2. Other methods return 405

---

## 6. Infrastructure Requirements

### REQ-OI-023 (Must): Add uuid dependency
**Acceptance Criteria:**
1. Add `uuid = { version = "1", features = ["v4"] }` to Cargo.toml
2. Used for generating channel IDs in Calendar/Drive watch

### REQ-OI-024 (Must): Add HTTP server dependency for webhook serve
**Acceptance Criteria:**
1. Add minimal HTTP server crate (e.g., `axum` or `hyper`) to Cargo.toml
2. Used only by `webhook serve` command
3. Prefer lightweight option to minimize binary size impact

### REQ-OI-025 (Must): CLI structure for watch subcommands
**Acceptance Criteria:**
1. Gmail: uses existing `GmailWatchCommand` enum — update `Renew` to match spec (remove if not needed)
2. Calendar: add `Watch(CalendarWatchArgs)` variant to `CalendarCommand` with `Start/Stop/Status` subcommands
3. Drive: add `Watch(DriveWatchArgs)` variant to `DriveCommand` with `Start/Stop/Status` subcommands
4. Root: add `Webhook(WebhookArgs)` variant to `Command` enum with `Serve` subcommand

### REQ-OI-026 (Must): Watch response types
**Acceptance Criteria:**
1. Gmail: `WatchResponse { historyId: u64, expiration: String }` (from Google API)
2. Calendar/Drive: `WatchChannelResponse { id: String, resourceId: String, expiration: String }`
3. All types derive `Serialize`/`Deserialize` with `#[serde(rename_all = "camelCase")]`

---

## Out of Scope (Won't)

1. Receiving push notifications in omg-gog (OMEGA's server does this)
2. Auto-renewal of watches (caller's responsibility)
3. Pub/Sub topic creation or IAM setup
4. Google Search Console domain verification
5. Multi-account watch management
6. Watch state persistence between CLI invocations
7. OMEGA_STORES_DIR supporting multiple accounts
8. OMEGA_STORES_DIR supporting multiple Google services files

---

## Traceability Matrix

| Req ID | Priority | Module | Test Coverage | Architecture Section | Implementation Module | Milestone |
|--------|----------|--------|---------------|---------------------|-----------------------|-----------|
| REQ-OI-001 | Must | auth/omega_store | req_oi_001_* (14 tests) + security_* + failure_* | Module 1: auth/omega_store -- OmegaStoreCredentialStore | OmegaStoreCredentialStore @ src/auth/omega_store.rs | OI-M1 |
| REQ-OI-002 | Must | auth/omega_store | req_oi_002_* (11 tests) | Module 1: auth/omega_store -- CredentialStore impl | CredentialStore impl @ src/auth/omega_store.rs | OI-M1 |
| REQ-OI-003 | Must | auth/keyring | req_oi_003_* (5 tests) | Module 2: integration points -- credential_store_factory | credential_store_factory @ src/auth/keyring.rs | OI-M1 |
| REQ-OI-004 | Must | services/mod | req_oi_004_* (3 tests) | Module 2: integration points -- bootstrap_service_context | bootstrap_service_context @ src/services/mod.rs, client_credentials @ src/auth/omega_store.rs | OI-M1 |
| REQ-OI-005 | Must | auth/omega_store | req_oi_005_* (6 tests) | Module 1: auth/omega_store -- atomic write | atomic_write @ src/auth/omega_store.rs | OI-M1 |
| REQ-OI-006 | Must | cli/mod (auth) | req_oi_006_* (3 tests) | Module 7: CLI wiring -- auth status handler | is_omega_store_active @ src/auth/omega_store.rs | OI-M1 |
| REQ-OI-007 | Should | auth/omega_store | req_oi_007_* (3 tests) | Module 1: auth/omega_store -- error messages | OmegaStoreCredentialStore::new @ src/auth/omega_store.rs | OI-M1 |
| REQ-OI-010 | Must | services/gmail/watch, cli/mod | TBD | Module 3: gmail watch -- watch_start | OI-M2 |
| REQ-OI-011 | Must | services/gmail/watch, cli/mod | TBD | Module 3: gmail watch -- watch_stop | OI-M2 |
| REQ-OI-012 | Must | services/gmail/watch, cli/mod | TBD | Module 3: gmail watch -- watch_status | OI-M2 |
| REQ-OI-013 | Should | cli/mod | TBD | Module 3: gmail watch -- Pub/Sub hints | OI-M2 |
| REQ-OI-014 | Must | services/calendar/watch, cli/mod | TBD | Module 4: calendar watch -- watch_start | OI-M3 |
| REQ-OI-015 | Must | services/calendar/watch, cli/mod | TBD | Module 4: calendar watch -- watch_stop | OI-M3 |
| REQ-OI-016 | Must | cli/mod | TBD | Module 4: calendar watch -- watch_status | OI-M3 |
| REQ-OI-017 | Must | services/drive/watch, cli/mod | TBD | Module 5: drive watch -- watch_start | OI-M3 |
| REQ-OI-018 | Must | services/drive/watch, cli/mod | TBD | Module 5: drive watch -- watch_stop | OI-M3 |
| REQ-OI-019 | Must | services/drive/watch, cli/mod | TBD | Module 5: drive watch -- watch_status | OI-M3 |
| REQ-OI-020 | Must | webhook/mod, cli/mod | TBD | Module 6: webhook -- serve | OI-M4 |
| REQ-OI-021 | Should | webhook/mod | TBD | Module 6: webhook -- startup banner | OI-M4 |
| REQ-OI-022 | Should | webhook/mod | TBD | Module 6: webhook -- method filtering | OI-M4 |
| REQ-OI-023 | Must | Cargo.toml | TBD | External Dependencies -- uuid crate | OI-M3 |
| REQ-OI-024 | Must | Cargo.toml | TBD | External Dependencies -- hyper crate | OI-M4 |
| REQ-OI-025 | Must | cli/* | TBD | Module 7: CLI wiring -- CLI structure | OI-M2/M3/M4 |
| REQ-OI-026 | Must | services/*/watch | TBD | Modules 3/4/5 + Shared Types -- response serde types | OI-M2/M3 |
