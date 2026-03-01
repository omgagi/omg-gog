# Requirements: omega-google Runtime Layer

## Scope

This document covers the **runtime layer** -- the connective tissue that turns omega-google from a 28K-line stub into a functional Google Workspace CLI. It spans:

### Domains/Modules Affected
- `src/auth/oauth.rs` -- OAuth code exchange, desktop/manual/remote flows
- `src/auth/token.rs` -- Token refresh, access token caching
- `src/auth/keyring.rs` -- OS keyring backend (new), credential store factory
- `src/auth/service_account.rs` -- JWT exchange
- `src/auth/mod.rs` -- TokenData struct extension (access_token, expires_at)
- `src/http/client.rs` -- Authenticated client construction from stored tokens
- `src/services/mod.rs` -- ServiceContext factory, auth bootstrap, generic API call helpers
- `src/services/common.rs` -- Pagination loop implementation
- `src/cli/mod.rs` -- All 15 service handler dispatch paths (currently stubs)
- `src/services/gmail/` -- Gmail handler wiring
- `src/services/calendar/` -- Calendar handler wiring
- `src/services/drive/` -- Drive handler wiring (including file upload/download)
- `src/services/*/` -- Remaining 12 services handler wiring

### Files NOT Affected (already working)
- `src/auth/scopes.rs` -- scope mappings complete
- `src/http/retry.rs` -- backoff logic complete
- `src/http/circuit_breaker.rs` -- circuit breaker complete
- `src/http/middleware.rs` -- retry middleware complete
- `src/output/` -- all formatters complete
- `src/cli/root.rs` -- all CLI flag definitions complete
- `src/error/` -- error types and exit codes complete

## Summary (plain language)

omega-google currently parses all CLI arguments, defines all types, builds all URLs, and formats all output -- but every service command prints "Command registered" and exits. This requirements document specifies the runtime plumbing that makes real Google API calls work: OAuth login flows, token storage and refresh, authenticated HTTP calls, paginated result fetching, file upload/download, and the wiring of the three core services (Gmail, Calendar, Drive). The remaining 12 services follow identical patterns and are lower priority.

## User Stories

- As a developer, I want to run `omega-google auth add` and complete an OAuth flow so that my Google account credentials are stored securely.
- As a developer, I want `omega-google gmail search "from:boss"` to actually search my Gmail and return real results, not a stub message.
- As a developer, I want `omega-google drive download <fileId>` to download the actual file to my local disk.
- As a developer, I want `omega-google calendar events` to show my real calendar events.
- As a headless server operator, I want `omega-google auth add --manual` to work without a local browser.
- As an automation engineer, I want `omega-google drive ls --all --json` to fetch all pages of results and output valid JSON.
- As a developer, I want expired tokens to be silently refreshed so I do not have to re-authenticate constantly.
- As a sysadmin, I want service account JWT auth to work so I can use domain-wide delegation for Keep and Groups.
- As a developer, I want `--verbose` to show me the actual HTTP requests being made for debugging.
- As a developer, I want `--dry-run` on mutating commands to show what would happen without executing.
- As a security-conscious user, I want my tokens stored in the OS keyring (macOS Keychain) rather than plain files.

## Requirements

### Area 1: OAuth and Auth Runtime

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-RT-001 | OAuth code exchange via raw reqwest POST to Google token endpoint | Must | - [ ] POST to `https://oauth2.googleapis.com/token` with grant_type=authorization_code<br>- [ ] Sends client_id, client_secret, code, redirect_uri as form-urlencoded body<br>- [ ] Deserializes response into existing `TokenResponse` struct<br>- [ ] Returns error with Google's error message on failure (400, 401) |
| REQ-RT-002 | Desktop OAuth flow with ephemeral local HTTP server | Must | - [ ] Starts a TCP listener on `127.0.0.1:0` (OS-assigned port)<br>- [ ] Uses the assigned port in redirect_uri: `http://127.0.0.1:<port>`<br>- [ ] Opens browser with `open` (macOS) / `xdg-open` (Linux) pointing to auth URL<br>- [ ] HTTP server accepts exactly one GET request on `/`<br>- [ ] Extracts `code` query parameter from the redirect<br>- [ ] Returns a user-friendly HTML "Success, you may close this tab" page<br>- [ ] Shuts down server after receiving the code<br>- [ ] Times out after 120 seconds with exit code 4 (auth required) |
| REQ-RT-003 | Manual OAuth flow (--manual): paste redirect URL | Must | - [ ] Prints auth URL to stderr with instructions<br>- [ ] Reads a line from stdin (the full redirect URL)<br>- [ ] Parses the `code` query parameter from the pasted URL<br>- [ ] Uses redirect_uri `urn:ietf:wg:oauth:2.0:oob` in the auth URL<br>- [ ] Falls back to manual flow if browser cannot be opened |
| REQ-RT-004 | Remote OAuth flow (--remote): two-step headless flow | Should | - [ ] `--remote --step 1` prints auth URL with a randomly generated state parameter<br>- [ ] State is cached in a temporary file in the config directory<br>- [ ] `--remote --step 2 --auth-url <url>` receives the redirect URL<br>- [ ] Validates state parameter matches cached state<br>- [ ] Exchanges code for tokens |
| REQ-RT-005 | Token refresh when access token expires | Must | - [ ] Calls `needs_refresh()` before each API call (checks `expires_at - 5min buffer`)<br>- [ ] POSTs to token endpoint with grant_type=refresh_token<br>- [ ] Updates stored access_token and expires_at in credential store<br>- [ ] If refresh fails with invalid_grant, returns exit code 4 with re-auth message<br>- [ ] Thread-safe: concurrent commands sharing a token do not race |
| REQ-RT-006 | Service account JWT exchange | Must | - [ ] POSTs to `https://oauth2.googleapis.com/token` with grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer<br>- [ ] Sends the JWT assertion built by existing `build_jwt_assertion()`<br>- [ ] Deserializes the access_token from the response<br>- [ ] Returns error with message on failure |
| REQ-RT-007 | Store access token alongside refresh token in credential store | Must | - [ ] Extend `TokenData` to include `access_token: Option<String>` and `expires_at: Option<chrono::DateTime<chrono::Utc>>`<br>- [ ] Update `serialize_token()` and `deserialize_token()` to handle new fields<br>- [ ] `needs_refresh()` uses `expires_at` field instead of `created_at` heuristic<br>- [ ] Backward compatible: old token data without these fields still deserializes |
| REQ-RT-008 | `auth add` full flow: OAuth + store token + set default | Must | - [ ] Loads client credentials from config dir<br>- [ ] Determines flow mode from flags (desktop/manual/remote)<br>- [ ] Collects requested services from `--services` flag (default: all user services)<br>- [ ] Computes scopes honoring `--readonly` and `--drive-scope`<br>- [ ] Performs OAuth flow and exchanges code for tokens<br>- [ ] Stores refresh token, access token, and metadata via CredentialStore<br>- [ ] Sets as default account if first account<br>- [ ] Prints success message with email and granted scopes |
| REQ-RT-009 | `auth remove` deletes stored token from credential store | Must | - [ ] Prompts for confirmation unless `--force`<br>- [ ] Deletes token from credential store<br>- [ ] Attempts to remove legacy key format as well<br>- [ ] Prints confirmation message |
| REQ-RT-010 | `auth status` shows current auth state | Must | - [ ] Shows config path, keyring backend in use, credential file status<br>- [ ] Shows current account (resolved from flag/env/default)<br>- [ ] Shows token state: email, client, services, scopes, created_at, refresh status<br>- [ ] JSON/plain/text output |
| REQ-RT-011 | `auth list` shows all stored accounts from credential store | Must | - [ ] Lists all tokens from `CredentialStore::list_tokens()`<br>- [ ] Shows email, client, services, created_at for each<br>- [ ] Indicates which is the default account<br>- [ ] JSON/plain/text output |
| REQ-RT-012 | `auth tokens delete` removes specific token | Must | - [ ] Deletes the token for the specified email from the credential store<br>- [ ] Uses the resolved client name |

### Area 2: Keyring and Credential Storage

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-RT-013 | OS keyring backend wrapping the `keyring` crate | Must | - [ ] `KeyringCredentialStore` implements `CredentialStore` trait<br>- [ ] Uses `keyring::Entry` with service name `omega-google`<br>- [ ] Key format: `token:<client>:<email>` for tokens, `default:<client>` for default account<br>- [ ] Works on macOS (Keychain) via `apple-native` feature<br>- [ ] Works on Linux (Secret Service/D-Bus) via `linux-native` feature<br>- [ ] Falls back gracefully if keyring unavailable (returns error, does not panic) |
| REQ-RT-014 | File-based fallback with AES-GCM encryption | Should | - [ ] Existing `FileCredentialStore` extended with optional AES-GCM encryption<br>- [ ] Password sourced from TTY prompt or `GOG_KEYRING_PASSWORD` env var<br>- [ ] Encryption uses AES-256-GCM with random nonce per entry<br>- [ ] Key derived from password using a KDF (e.g., PBKDF2 or Argon2 via existing `aes-gcm` crate)<br>- [ ] Unencrypted mode remains for testing/CI (when no password provided) |
| REQ-RT-015 | Credential store factory: auto/keychain/file selection | Must | - [ ] Reads `keyring_backend` from config and `GOG_KEYRING_BACKEND` env (env overrides config)<br>- [ ] `auto` (default): try OS keyring first, fall back to file on error<br>- [ ] `keychain` / `keyring`: force OS keyring, error if unavailable<br>- [ ] `file`: force file-based backend<br>- [ ] Factory returns `Box<dyn CredentialStore>` |
| REQ-RT-016 | Keyring timeout on Linux (5 seconds) | Should | - [ ] When accessing D-Bus keyring on Linux, wrap with a 5-second timeout<br>- [ ] On timeout, print hint: "Keyring timed out. Try GOG_KEYRING_BACKEND=file"<br>- [ ] Fall back to file backend in `auto` mode on timeout |

### Area 3: Service Execution Infrastructure

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-RT-017 | Auth bootstrap: resolve account, load token, refresh if needed, build client | Must | - [ ] Single function: `bootstrap_auth(flags, config, store) -> Result<(String, reqwest::Client)>`<br>- [ ] Calls `resolve_account()` to determine email<br>- [ ] Loads token from credential store<br>- [ ] If `needs_refresh()`, performs token refresh and updates store<br>- [ ] Calls `build_authenticated_client()` with the access token<br>- [ ] Returns the resolved email and authenticated client<br>- [ ] On failure: maps to appropriate OmegaError variant and exit code |
| REQ-RT-018 | ServiceContext factory with auth bootstrap | Must | - [ ] Constructs `ServiceContext` from RootFlags, authenticated client, and resolved email<br>- [ ] Resolves OutputMode from flags (json/plain/csv/text)<br>- [ ] Builds JsonTransform from --results-only and --select flags<br>- [ ] Constructs Ui from color mode<br>- [ ] Single entry point: `ServiceContext::from_flags(flags, client) -> Result<ServiceContext>` |
| REQ-RT-019 | Generic API call helper: GET with deserialization | Must | - [ ] `api_get<T: DeserializeOwned>(ctx, url) -> Result<T>`<br>- [ ] Builds RetryableRequest from URL<br>- [ ] Calls `execute_with_retry()` with shared circuit breaker<br>- [ ] Checks response status: 2xx deserializes body, 4xx/5xx maps to OmegaError<br>- [ ] Logs request/response to stderr if `--verbose` |
| REQ-RT-020 | Generic API call helper: POST/PUT/PATCH/DELETE with body | Must | - [ ] `api_post<T: DeserializeOwned>(ctx, url, body) -> Result<T>`<br>- [ ] Same pattern as GET but with JSON body serialization<br>- [ ] Content-Type: application/json header<br>- [ ] Variant for empty response (returns `()`) for DELETE operations |
| REQ-RT-021 | Single shared CircuitBreaker per CLI invocation | Must | - [ ] CircuitBreaker instance created once in main/execute<br>- [ ] Passed through ServiceContext to all API call helpers<br>- [ ] Arc-wrapped for thread safety |
| REQ-RT-022 | Async handler pattern for service commands | Must | - [ ] All service handler functions become `async fn` returning `Result<i32>` or `Result<()>`<br>- [ ] CLI dispatch in `execute()` awaits the handler<br>- [ ] Errors converted to exit codes via `exit_code_for()` |

### Area 4: Pagination

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-RT-023 | Generic pagination loop for nextPageToken pattern | Must | - [ ] `paginate<T, F>(ctx, initial_url, extract_fn) -> Result<Vec<T>>` where F returns (items, nextPageToken)<br>- [ ] Fetches first page with initial URL<br>- [ ] If nextPageToken present and `--all` flag set, appends pageToken to URL and fetches next page<br>- [ ] Accumulates all items into a single Vec<br>- [ ] Respects `--max` as per-page limit on each request<br>- [ ] Progress hint on stderr: "Fetching page N..." when not first page |
| REQ-RT-024 | Single-page mode (default, no --all) | Must | - [ ] Returns one page of results<br>- [ ] If nextPageToken exists, prints hint on stderr: "More results available. Use --all or --page <token>"<br>- [ ] Respects `--page <token>` to continue from a specific page |
| REQ-RT-025 | --fail-empty exits with code 3 on empty results | Must | - [ ] When `--fail-empty` flag is set and results are empty, return exit code 3 (EMPTY_RESULTS)<br>- [ ] Applies after pagination completes (all pages fetched if --all) |

### Area 5: File I/O

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-RT-026 | Drive file download: binary files | Must | - [ ] GET to `https://www.googleapis.com/drive/v3/files/<id>?alt=media`<br>- [ ] Streams response body to output file (not buffering entire file in memory)<br>- [ ] Default output filename from Drive file metadata name<br>- [ ] `--out` flag overrides output path<br>- [ ] Progress hint on stderr showing bytes downloaded |
| REQ-RT-027 | Drive file download: Google Workspace file export | Must | - [ ] For Google Docs/Sheets/Slides, use export endpoint: `files/<id>/export?mimeType=<mime>`<br>- [ ] `--format` flag maps to MIME type (pdf, docx, xlsx, pptx, csv, txt)<br>- [ ] Default format: PDF for all Google Workspace types<br>- [ ] Output filename gets appropriate extension |
| REQ-RT-028 | Drive file upload: simple (multipart) | Must | - [ ] POST to `https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart`<br>- [ ] Multipart body with metadata JSON part and file content part<br>- [ ] `--name` flag for filename (defaults to local filename)<br>- [ ] `--parent` flag for target folder ID<br>- [ ] `--convert` flag to convert to Google format on upload<br>- [ ] Returns file ID and web link |
| REQ-RT-029 | Drive file upload: resumable for large files | Should | - [ ] Files > 5MB use resumable upload protocol<br>- [ ] POST to initiate, then PUT chunks to the returned upload URI<br>- [ ] Progress reporting on stderr<br>- [ ] Resumable on network failure (re-PUT from last successful byte) |
| REQ-RT-030 | Gmail attachment download | Must | - [ ] GET attachment by message ID and attachment ID<br>- [ ] Base64url-decode the response data field<br>- [ ] Write binary data to output file<br>- [ ] `--out-dir` flag for output directory<br>- [ ] Filename from the attachment's filename field |
| REQ-RT-031 | Export to various formats (used by Docs, Sheets, Slides) | Should | - [ ] Shared export function in `services/` that any document service can use<br>- [ ] Maps format string to MIME type<br>- [ ] Uses Drive API export endpoint<br>- [ ] Writes to file with correct extension |

### Area 6: Core Service Handlers (Gmail, Calendar, Drive)

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-RT-032 | Gmail: search threads (`gmail search <query>`) | Must | - [ ] Auth bootstrap, build search URL via existing `build_thread_search_url()`<br>- [ ] API GET, deserialize `ThreadListResponse`<br>- [ ] Pagination via REQ-RT-023/024<br>- [ ] Output formatted via ServiceContext.write_output()<br>- [ ] Honors --max, --page, --all, --fail-empty |
| REQ-RT-033 | Gmail: search messages (`gmail messages search <query>`) | Must | - [ ] Uses `build_message_search_url()` with optional `include_body`<br>- [ ] Pagination support<br>- [ ] JSON/plain/text output |
| REQ-RT-034 | Gmail: get thread (`gmail thread get <id>`) | Must | - [ ] GET thread by ID with format=full<br>- [ ] Returns all messages in thread<br>- [ ] JSON/plain/text output |
| REQ-RT-035 | Gmail: get message (`gmail get <id>`) | Must | - [ ] GET message by ID<br>- [ ] Honors --format flag (full/metadata/raw)<br>- [ ] JSON/plain/text output |
| REQ-RT-036 | Gmail: send email (`gmail send`) | Must | - [ ] Builds MIME message via existing `build_mime_message()`<br>- [ ] Base64url-encodes and POSTs via `build_send_url()`<br>- [ ] Supports --to, --subject, --body, --body-html, --cc, --bcc, --reply-to-message-id, --attach<br>- [ ] Dry-run shows the composed message without sending |
| REQ-RT-037 | Gmail: labels CRUD (`gmail labels list/get/create/modify/delete`) | Must | - [ ] list: GET labels, output as list<br>- [ ] get: resolves label by name or ID<br>- [ ] create: POST new label<br>- [ ] modify: PATCH label<br>- [ ] delete: DELETE with confirmation prompt |
| REQ-RT-038 | Gmail: drafts CRUD (`gmail drafts list/get/create/update/send/delete`) | Should | - [ ] Full CRUD using existing URL builders<br>- [ ] send: sends a draft by ID<br>- [ ] Same compose flags as send command |
| REQ-RT-039 | Gmail: attachment download (`gmail attachment <msgId> <attachId>`) | Must | - [ ] Uses REQ-RT-030 attachment download logic |
| REQ-RT-040 | Gmail: thread modify (`gmail thread modify <id> --add/--remove`) | Must | - [ ] POST to modify endpoint with addLabelIds/removeLabelIds |
| REQ-RT-041 | Gmail: batch operations (`gmail batch delete/modify`) | Should | - [ ] Uses existing `build_batch_*` URL builders<br>- [ ] Batch modify/delete with message IDs |
| REQ-RT-042 | Gmail: history listing (`gmail history --since <historyId>`) | Should | - [ ] Uses existing history URL builder<br>- [ ] Pagination support |
| REQ-RT-043 | Gmail: settings (filters, forwarding, send-as, delegates, vacation) | Could | - [ ] Uses existing settings URL builders (10 builder functions)<br>- [ ] Each setting subcommand wired to its endpoint |
| REQ-RT-044 | Calendar: list events (`calendar events`) | Must | - [ ] Auth bootstrap with Calendar service<br>- [ ] Uses existing `build_events_list_url()`<br>- [ ] Supports --cal, --from, --to, --max, --query, --page, --all<br>- [ ] Flexible date/time parsing via existing time module<br>- [ ] Pagination support<br>- [ ] JSON/plain/text output |
| REQ-RT-045 | Calendar: get event (`calendar event <calId> <eventId>`) | Must | - [ ] GET single event by calendar ID and event ID<br>- [ ] JSON/plain/text output |
| REQ-RT-046 | Calendar: create event (`calendar create`) | Must | - [ ] POST to events endpoint via existing `build_event_create_url()`<br>- [ ] Supports --summary, --from, --to, --description, --location, --attendees, --all-day<br>- [ ] Dry-run shows event JSON without creating |
| REQ-RT-047 | Calendar: update event (`calendar update`) | Must | - [ ] PATCH to events endpoint<br>- [ ] All create fields as optional overrides |
| REQ-RT-048 | Calendar: delete event (`calendar delete`) | Must | - [ ] DELETE with confirmation prompt unless --force<br>- [ ] Dry-run support |
| REQ-RT-049 | Calendar: list calendars (`calendar calendars`) | Must | - [ ] GET calendar list<br>- [ ] JSON/plain/text output |
| REQ-RT-050 | Calendar: free/busy query (`calendar freebusy`) | Must | - [ ] POST to freeBusy endpoint via existing builder<br>- [ ] Multiple calendar IDs, --from, --to |
| REQ-RT-051 | Calendar: respond/RSVP (`calendar respond`) | Should | - [ ] PATCH event with attendee response status<br>- [ ] --send-updates flag |
| REQ-RT-052 | Calendar: search events (`calendar search`) | Should | - [ ] Cross-calendar search using existing builder<br>- [ ] Pagination support |
| REQ-RT-053 | Calendar: ACL listing (`calendar acl`) | Should | - [ ] GET ACL for a calendar<br>- [ ] JSON/plain/text output |
| REQ-RT-054 | Calendar: colors (`calendar colors`) | Should | - [ ] GET color definitions (static endpoint, can work without full auth context) |
| REQ-RT-055 | Drive: list files (`drive ls`) | Must | - [ ] Auth bootstrap with Drive service<br>- [ ] Uses existing `build_list_url()`<br>- [ ] Supports --parent, --max, --page, --query, --all, --all-drives<br>- [ ] Pagination support<br>- [ ] JSON/plain/text output |
| REQ-RT-056 | Drive: search files (`drive search <text>`) | Must | - [ ] Uses existing `build_search_url()`<br>- [ ] Supports --raw-query, --max, --page, --all-drives<br>- [ ] Pagination support |
| REQ-RT-057 | Drive: get file metadata (`drive get <fileId>`) | Must | - [ ] GET file metadata by ID<br>- [ ] JSON/plain/text output |
| REQ-RT-058 | Drive: download file (`drive download <fileId>`) | Must | - [ ] Uses REQ-RT-026 and REQ-RT-027 download logic<br>- [ ] Determines if Google Workspace file (needs export) or binary (direct download) |
| REQ-RT-059 | Drive: upload file (`drive upload <path>`) | Must | - [ ] Uses REQ-RT-028 upload logic<br>- [ ] Supports --name, --parent, --convert flags |
| REQ-RT-060 | Drive: create folder (`drive mkdir <name>`) | Must | - [ ] POST with mimeType=application/vnd.google-apps.folder<br>- [ ] --parent flag |
| REQ-RT-061 | Drive: delete file (`drive delete <fileId>`) | Must | - [ ] Trash (default) or permanent delete (--permanent)<br>- [ ] Confirmation prompt unless --force |
| REQ-RT-062 | Drive: move file (`drive move <fileId> --parent`) | Must | - [ ] PATCH to update parents |
| REQ-RT-063 | Drive: rename file (`drive rename <fileId> <name>`) | Must | - [ ] PATCH to update name |
| REQ-RT-064 | Drive: share file (`drive share <fileId>`) | Must | - [ ] POST permission via existing builder<br>- [ ] --to, --email, --domain, --role flags |
| REQ-RT-065 | Drive: list permissions (`drive permissions <fileId>`) | Must | - [ ] GET permissions list<br>- [ ] JSON/plain/text output |
| REQ-RT-066 | Drive: copy file (`drive copy <fileId>`) | Must | - [ ] POST to files/<id>/copy<br>- [ ] --name, --parent flags |
| REQ-RT-067 | Drive: list shared drives (`drive drives`) | Should | - [ ] Uses existing `build_drives_list_url()`<br>- [ ] Pagination support |
| REQ-RT-068 | Drive: comments CRUD (`drive comments`) | Could | - [ ] Uses existing comment URL builders<br>- [ ] list/get/create/resolve/delete |

### Area 7: Remaining 12 Service Handlers

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-RT-069 | Docs service handlers (export, create, copy, info, edit, comments) | Should | - [ ] Wire up all Docs subcommands using existing URL/body builders<br>- [ ] Export uses shared Drive export logic (REQ-RT-031) |
| REQ-RT-070 | Sheets service handlers (read, write, update, append, format, create) | Should | - [ ] Wire up all Sheets subcommands<br>- [ ] A1 notation parsing already exists |
| REQ-RT-071 | Slides service handlers (export, create, copy, slides ops, notes) | Should | - [ ] Wire up all Slides subcommands<br>- [ ] Export uses shared Drive export logic |
| REQ-RT-072 | Forms service handlers (get, create, responses) | Should | - [ ] Wire up Forms subcommands<br>- [ ] 3 URL builders exist |
| REQ-RT-073 | Chat service handlers (spaces, messages, DMs) | Should | - [ ] Wire up Chat subcommands<br>- [ ] Workspace-only API (will fail for consumer accounts with clear error) |
| REQ-RT-074 | Classroom service handlers (courses, roster, coursework, materials, submissions, announcements, topics, invitations, guardians) | Could | - [ ] Wire up all Classroom subcommands<br>- [ ] 61 URL builders exist across 9 modules |
| REQ-RT-075 | Tasks service handlers (tasklists, tasks CRUD, done/undo/clear) | Should | - [ ] Wire up all Tasks subcommands<br>- [ ] 11 URL builders exist |
| REQ-RT-076 | Contacts service handlers (search, list, get, create, update, delete, directory) | Should | - [ ] Wire up all Contacts subcommands<br>- [ ] 12 URL builders exist |
| REQ-RT-077 | People service handlers (me, get, search, relations) | Should | - [ ] Wire up People subcommands<br>- [ ] 4 URL builders exist |
| REQ-RT-078 | Groups service handlers (list, members) | Could | - [ ] Wire up Groups subcommands<br>- [ ] Workspace-only API<br>- [ ] 3 URL builders exist |
| REQ-RT-079 | Keep service handlers (list, get, search notes, download attachments) | Could | - [ ] Wire up Keep subcommands<br>- [ ] Requires service account auth (REQ-RT-006)<br>- [ ] 4 URL builders exist |
| REQ-RT-080 | Apps Script service handlers (create, get, content, run) | Could | - [ ] Wire up Apps Script subcommands<br>- [ ] 6 URL builders exist |

### Area 8: Verbose/Dry-Run Support

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-RT-081 | --verbose shows HTTP request/response details on stderr | Must | - [ ] Before each request: logs method, URL, headers (redacting Authorization bearer token value)<br>- [ ] After each response: logs status code, content-type, body size<br>- [ ] On retry: logs retry attempt number and delay<br>- [ ] All verbose output goes to stderr, never stdout |
| REQ-RT-082 | --dry-run for mutating commands | Must | - [ ] On POST/PUT/PATCH/DELETE: prints the request that would be made (method, URL, body) to stderr<br>- [ ] Does NOT execute the request<br>- [ ] Returns exit code 0<br>- [ ] Read-only commands (GET/list) execute normally even with --dry-run |

## Acceptance Criteria (detailed)

### REQ-RT-001: OAuth code exchange
- [ ] Given valid client credentials and authorization code, when `exchange_code()` is called, then it returns a `TokenResponse` with access_token and refresh_token
- [ ] Given an invalid authorization code, when `exchange_code()` is called, then it returns an error containing Google's error description
- [ ] Given network failure, when `exchange_code()` is called, then it returns an error (no retry for auth flows)
- [ ] The function uses raw `reqwest::Client::post()`, NOT the `oauth2` crate

### REQ-RT-002: Desktop OAuth flow
- [ ] Given credentials are configured and a browser is available, when `auth add` is run, then the browser opens, user completes consent, and the token is stored
- [ ] Given the user does not complete consent within 120 seconds, when the timeout elapses, then the command exits with code 4 and message "OAuth flow timed out after 120 seconds"
- [ ] Given the server is started, when checking the port, then `127.0.0.1` is used (never `0.0.0.0`)
- [ ] Given the redirect arrives, when the code is extracted, then query parameter parsing handles both `?code=` and `?error=` cases

### REQ-RT-005: Token refresh
- [ ] Given a token with `expires_at` in 3 minutes (within the 5-minute buffer), when an API call is made, then the token is refreshed before the call
- [ ] Given a token with `expires_at` in 10 minutes (outside the buffer), when an API call is made, then the existing access token is used
- [ ] Given the refresh fails with `invalid_grant`, when the error is returned, then the message includes "Re-authenticate with: omega-google auth add"
- [ ] Given a successful refresh, when the new token is received, then both access_token and expires_at are updated in the credential store

### REQ-RT-007: Access token caching in TokenData
- [ ] Given a newly exchanged token with access_token and expires_in=3600, when stored, then `expires_at` is set to `now + 3600 seconds`
- [ ] Given existing stored data without access_token/expires_at fields, when deserialized, then those fields are None (backward compatible)
- [ ] Given `needs_refresh()` is called with a token that has `expires_at`, then it checks `expires_at - 5min < now`
- [ ] Given `needs_refresh()` is called with a token that has no `expires_at`, then it falls back to the `created_at` heuristic

### REQ-RT-013: OS keyring backend
- [ ] Given macOS, when `KeyringCredentialStore` is used, then tokens are stored in macOS Keychain (verifiable via Keychain Access app)
- [ ] Given Linux with Secret Service running, when `KeyringCredentialStore` is used, then tokens are stored via D-Bus
- [ ] Given the `keyring` crate fails (e.g., no D-Bus session), when auto mode is active, then it falls back to file backend with a warning on stderr
- [ ] Given the store has tokens, when `list_tokens()` is called, then it iterates all keys matching `token:*` pattern

### REQ-RT-017: Auth bootstrap
- [ ] Given `--account user@example.com` and a stored token, when bootstrap runs, then the correct account's token is loaded
- [ ] Given no `--account` and one stored account, when bootstrap runs, then it auto-selects the single account
- [ ] Given no stored accounts, when bootstrap runs, then it returns exit code 4 with message "No authenticated accounts. Run: omega-google auth add"
- [ ] Given a token that needs refresh, when bootstrap runs, then the refresh happens transparently before returning the client

### REQ-RT-023: Pagination loop
- [ ] Given `--all` and a response with `nextPageToken`, when pagination runs, then subsequent pages are fetched until no more tokens
- [ ] Given `--all` and 5 pages of results, when pagination completes, then all items are accumulated into a single Vec and output together
- [ ] Given `--max 10` and `--all`, when pagination runs, then each page requests maxResults=10 but all pages are fetched
- [ ] Given `--page TOKEN123`, when the request is made, then `pageToken=TOKEN123` is included in the URL

### REQ-RT-026: Drive file download (binary)
- [ ] Given a binary file (PDF, image), when downloaded, then the file content matches the original
- [ ] Given `--out /tmp/output.pdf`, when downloaded, then the file is written to `/tmp/output.pdf`
- [ ] Given a large file (>100MB), when downloaded, then memory usage stays bounded (streaming, not buffering)

### REQ-RT-028: Drive file upload
- [ ] Given a local file, when uploaded with default settings, then the file appears in Drive root
- [ ] Given `--parent FOLDER_ID`, when uploaded, then the file appears in the specified folder
- [ ] Given `--convert`, when uploading a .docx file, then it is converted to Google Docs format

### REQ-RT-081: Verbose logging
- [ ] Given `--verbose`, when a GET request is made, then stderr shows: `> GET https://...` and `< 200 OK (1234 bytes)`
- [ ] Given `--verbose`, when the Authorization header is logged, then the token value is redacted: `Authorization: Bearer [REDACTED]`

### REQ-RT-082: Dry-run
- [ ] Given `--dry-run` and `gmail send --to user@example.com --subject "Test"`, when run, then stderr shows the request details but no email is sent
- [ ] Given `--dry-run` and `gmail search "test"`, when run, then the search executes normally (GET is not mutating)

## Impact Analysis

### Existing Code Affected

| File/Module | How It Is Affected | Risk |
|---|---|---|
| `src/auth/mod.rs` (TokenData struct) | Add `access_token` and `expires_at` fields | Medium -- must be backward compatible with existing serialized tokens |
| `src/auth/token.rs` (serialize/deserialize) | Extend to handle new TokenData fields | Low -- additive change |
| `src/auth/token.rs` (needs_refresh) | Change to check `expires_at` instead of `created_at` | Medium -- logic change, must handle None expires_at |
| `src/auth/oauth.rs` (exchange_code) | Replace stub with real HTTP POST | Low -- currently a bail, clean replacement |
| `src/auth/service_account.rs` (exchange_jwt) | Replace stub with real HTTP POST | Low -- same as above |
| `src/auth/keyring.rs` | Add `KeyringCredentialStore` alongside existing `FileCredentialStore` | Low -- additive, existing implementation untouched |
| `src/services/mod.rs` (ServiceContext) | Add `circuit_breaker` field, API call helpers | Medium -- changes struct used by all handlers |
| `src/cli/mod.rs` (all handlers) | Replace stubs with real async handler calls | High -- touches all 15 service dispatch paths |
| `src/http/client.rs` | No changes needed (build_authenticated_client already works) | None |
| `src/http/middleware.rs` | No changes needed (execute_with_retry already works) | None |

### What Breaks If This Changes

| Module/Function | What Happens | Mitigation |
|---|---|---|
| `TokenData` serialization | Old tokens without access_token/expires_at fields | Use `Option<>` fields with serde `skip_serializing_if` and default deserialization |
| `needs_refresh()` callers | Return value semantics change (now checks expires_at vs created_at) | Fallback behavior: if no expires_at, use existing created_at heuristic |
| All existing tests (1,357) | Should not break -- they test URL builders, types, and formatting, not runtime | Run full test suite after changes |
| `cli/mod.rs` handler functions | Signatures change from sync to async | Must update all `handle_*` call sites |

### Regression Risk Areas

| Area | Why It Might Break |
|---|---|
| Token serialization round-trip | New fields must not break old stored data |
| OAuth URL generation | Must not regress -- auth URLs already tested |
| Service URL builders | Handler wiring calls builders with real arguments -- parameter mismatches possible |
| Output formatting | Real API responses may have fields not covered by type definitions (mitigated by `serde(flatten)`) |
| Exit codes | New error paths must map to correct exit codes |
| Circuit breaker shared state | Must be thread-safe when shared across async tasks (already uses `Mutex`) |

## Traceability Matrix

| Requirement ID | Priority | Test IDs | Architecture Section | Implementation Module |
|---------------|----------|----------|---------------------|---------------------|
| REQ-RT-001 | Must | req_rt_001_exchange_code_function_exists, req_rt_001_exchange_code_posts_authorization_code, req_rt_001_token_response_deserialize_full, req_rt_001_token_response_deserialize_minimal, req_rt_001_exchange_code_400_invalid_grant, req_rt_001_exchange_code_401_invalid_client, req_rt_001_uses_reqwest_not_oauth2_crate, req_rt_001_edge_empty_code, req_rt_001_edge_code_with_special_chars, req_rt_001_edge_token_response_extra_fields, req_rt_001_edge_token_response_missing_access_token, req_rt_001_edge_token_response_missing_token_type, req_rt_001_security_token_url_is_google, req_rt_001_security_auth_url_is_google, req_rt_001_form_urlencoded_not_json_body | rt-arch Module 3: auth/oauth.rs | `src/auth/oauth.rs` |
| REQ-RT-002 | Must | req_rt_002_oauth_flow_result_has_code_field, req_rt_002_oauth_flow_result_has_redirect_uri_field, req_rt_002_oauth_flow_result_clone_and_debug, req_rt_002_desktop_flow_timeout_is_120_seconds, req_rt_002_run_oauth_flow_exists, req_rt_002_run_desktop_flow_exists, req_rt_002_security_localhost_only, req_rt_002_desktop_mode_dispatches_to_desktop_flow, req_rt_002_integration_desktop_flow_single_request (ignored), req_rt_002_integration_desktop_flow_timeout (ignored), req_rt_002_extract_code_valid, req_rt_002_extract_code_error_access_denied, req_rt_002_extract_code_error_with_description, req_rt_002_extract_code_missing_code, req_rt_002_extract_code_with_extra_params, req_rt_002_extract_code_malformed_url, req_rt_002_extract_code_special_chars, req_rt_002_extract_code_empty_code, req_rt_002_extract_code_no_query_string, req_rt_002_extract_code_fragment_only, req_rt_002_extract_code_https_url, req_rt_002_extract_code_very_long, req_rt_002_security_code_not_logged, req_rt_002_flow_mode_desktop_exists, req_rt_002_flow_mode_is_debug, req_rt_002_flow_mode_is_clone_copy, req_rt_002_run_oauth_flow_with_force_consent, req_rt_002_run_oauth_flow_multiple_services, req_rt_002_run_oauth_flow_empty_services, req_rt_002_failure_browser_launch, req_rt_002_failure_port_bind_documented, req_rt_002_oauth_flow_module_accessible, req_rt_002_extract_code_accessible, req_rt_002_flow_mode_accessible | rt-arch Module 4: auth/oauth_flow.rs (desktop) | `src/auth/oauth_flow.rs` |
| REQ-RT-003 | Must | req_rt_003_manual_redirect_uri, req_rt_003_run_manual_flow_exists, req_rt_003_manual_mode_dispatches_to_manual_flow, req_rt_003_extract_code_from_pasted_url, req_rt_003_extract_code_oob_url_format, req_rt_003_fallback_documented, req_rt_003_edge_user_pastes_code_directly, req_rt_003_edge_url_with_whitespace, req_rt_003_edge_url_with_unicode, req_rt_003_failure_invalid_redirect, req_rt_003_flow_mode_manual_exists | rt-arch Module 4: auth/oauth_flow.rs (manual) | `src/auth/oauth_flow.rs` |
| REQ-RT-004 | Should | req_rt_004_flow_mode_remote_exists | rt-arch Module 4: auth/oauth_flow.rs (remote) | `src/auth/oauth_flow.rs` |
| REQ-RT-005 | Must | req_rt_005_refresh_access_token_exists, req_rt_005_refresh_posts_to_token_endpoint, req_rt_005_token_response_deserialize_happy_path, req_rt_005_token_response_deserialize_no_refresh_token, req_rt_005_refresh_invalid_grant_error, req_rt_005_refresh_network_error, req_rt_005_refresh_empty_refresh_token, req_rt_005_security_token_url_hardcoded, req_rt_005_needs_refresh_within_five_min_buffer, req_rt_005_needs_refresh_outside_five_min_buffer | rt-arch Module 2: auth/token.rs (refresh_access_token) | `src/auth/token.rs` |
| REQ-RT-006 | Must | req_rt_006_exchange_jwt_function_exists, req_rt_006_exchange_jwt_posts_jwt_bearer_grant_type, req_rt_006_service_account_token_response_deserialize, req_rt_006_exchange_jwt_failure_returns_error, req_rt_006_jwt_claims_serialize_correctly, req_rt_006_jwt_claims_no_subject, req_rt_006_edge_empty_assertion, req_rt_006_service_account_key_deserialize, req_rt_006_edge_wrong_key_type, req_rt_006_edge_key_file_not_found, req_rt_006_edge_malformed_key_file, req_rt_006_edge_sa_token_response_extra_fields, req_rt_006_security_token_url, req_rt_006_security_private_key_not_in_error | rt-arch Module 6: auth/service_account.rs (exchange_jwt) | `src/auth/service_account.rs` |
| REQ-RT-007 | Must | req_rt_007_token_data_has_access_token_field, req_rt_007_token_data_has_expires_at_field, req_rt_007_token_data_new_fields_optional_none, req_rt_007_token_data_empty_access_token, req_rt_007_token_data_expires_at_in_past, req_rt_007_token_data_clone_preserves_new_fields, req_rt_007_token_data_debug_contains_access_token, req_rt_007_serialize_includes_access_token, req_rt_007_serialize_includes_expires_at, req_rt_007_serialize_omits_access_token_when_none, req_rt_007_serialize_omits_expires_at_when_none, req_rt_007_deserialize_reads_access_token, req_rt_007_deserialize_reads_expires_at, req_rt_007_deserialize_backward_compatible_no_new_fields, req_rt_007_roundtrip_with_new_fields, req_rt_007_roundtrip_without_new_fields, req_rt_007_needs_refresh_expires_at_within_buffer, req_rt_007_needs_refresh_expires_at_outside_buffer, req_rt_007_needs_refresh_expires_at_exactly_five_minutes, req_rt_007_needs_refresh_expires_at_already_expired, req_rt_007_needs_refresh_no_expires_at_fresh_token, req_rt_007_needs_refresh_no_expires_at_old_token, req_rt_007_old_token_without_new_fields_triggers_refresh, req_rt_007_edge_deserialize_empty_string, req_rt_007_edge_deserialize_invalid_json, req_rt_007_edge_deserialize_missing_email, req_rt_007_edge_access_token_special_chars, req_rt_007_edge_expires_at_invalid_format, req_rt_007_edge_expires_at_far_future | rt-arch Module 1: auth/mod.rs (TokenData), Module 2: auth/token.rs (needs_refresh) | `src/auth/mod.rs`, `src/auth/token.rs` |
| REQ-RT-008 | Must | req_rt_008_auth_add_dispatches, req_rt_008_auth_add_manual_dispatches, req_rt_008_auth_add_force_consent_dispatches, req_rt_008_auth_add_no_credentials_returns_error, req_rt_008_auth_subcommands_all_reachable, req_rt_008_auth_add_remote_flag_parsed | rt-arch Module 11: cli/mod.rs (handle_auth_add) | `src/cli/mod.rs` |
| REQ-RT-009 | Must | req_rt_009_auth_remove_dispatches, req_rt_009_auth_remove_missing_email_usage_error, req_rt_009_auth_remove_force_flag_parsed | rt-arch Module 11: cli/mod.rs (handle_auth_remove) | `src/cli/mod.rs` |
| REQ-RT-010 | Must | req_rt_010_auth_status_dispatches, req_rt_010_auth_status_json_dispatches, req_rt_010_auth_status_shows_info | rt-arch Module 11: cli/mod.rs (handle_auth_status) | `src/cli/mod.rs` |
| REQ-RT-011 | Must | req_rt_011_auth_list_dispatches, req_rt_011_auth_list_json_dispatches, req_rt_011_auth_list_empty_store | rt-arch Module 11: cli/mod.rs (handle_auth_list) | `src/cli/mod.rs` |
| REQ-RT-012 | Must | req_rt_012_auth_tokens_delete_dispatches, req_rt_012_auth_tokens_delete_missing_email_usage_error, req_rt_012_auth_tokens_list_dispatches | rt-arch Module 11: cli/mod.rs (handle_auth_tokens) | `src/cli/mod.rs` |
| REQ-RT-013 | Must | req_rt_013_keyring_credential_store_struct_exists, req_rt_013_keyring_implements_credential_store, req_rt_013_service_name_is_omega_google, req_rt_013_key_format, req_rt_013_graceful_failure_no_panic, req_rt_013_keyring_store_is_send_sync, req_rt_013_keyring_set_get_roundtrip (ignored), req_rt_013_keyring_delete (ignored), req_rt_013_keyring_list_tokens (ignored), req_rt_013_keyring_default_account (ignored), req_rt_013_edge_get_nonexistent_token, req_rt_013_edge_delete_nonexistent_token, req_rt_013_security_file_permissions, req_rt_013_failure_permission_denied | rt-arch Module 5: auth/keyring.rs (KeyringCredentialStore) | `src/auth/keyring.rs` |
| REQ-RT-014 | Should | (filled by test-writer) | rt-arch Module 5: auth/keyring.rs (encrypted file backend) | `src/auth/keyring.rs` |
| REQ-RT-015 | Must | req_rt_015_factory_function_exists, req_rt_015_factory_file_backend, req_rt_015_factory_keychain_backend, req_rt_015_factory_keyring_synonym, req_rt_015_factory_auto_backend, req_rt_015_factory_none_defaults_to_auto, req_rt_015_factory_returns_boxed_trait, req_rt_015_factory_env_overrides_config, req_rt_015_edge_unknown_backend, req_rt_015_edge_empty_backend_string, req_rt_015_file_store_set_get_delete_cycle, req_rt_015_file_store_list_tokens, req_rt_015_file_store_keys, req_rt_015_file_store_default_account, req_rt_015_file_store_empty_directory, req_rt_015_file_store_multiple_clients, req_rt_015_file_store_overwrite_token | rt-arch Module 5: auth/keyring.rs (credential_store_factory) | `src/auth/keyring.rs` |
| REQ-RT-016 | Should | (filled by test-writer) | rt-arch Module 5: auth/keyring.rs (timeout handling) | `src/auth/keyring.rs` |
| REQ-RT-017 | Must | (filled by test-writer) | rt-arch Module 9: services/mod.rs (bootstrap_service_context) | `src/services/mod.rs` |
| REQ-RT-018 | Must | (filled by test-writer) | rt-arch Module 9: services/mod.rs (ServiceContext) | `src/services/mod.rs` |
| REQ-RT-019 | Must | (filled by test-writer) | rt-arch Module 7: http/api.rs (api_get) | `src/http/api.rs` |
| REQ-RT-020 | Must | (filled by test-writer) | rt-arch Module 7: http/api.rs (api_post, api_patch, api_delete) | `src/http/api.rs` |
| REQ-RT-021 | Must | (filled by test-writer) | rt-arch Module 7: http/api.rs + Module 9: services/mod.rs (circuit_breaker field) | `src/http/api.rs`, `src/services/mod.rs` |
| REQ-RT-022 | Must | (filled by test-writer) | rt-arch Module 7: http/api.rs (check_response_status) | `src/http/api.rs` |
| REQ-RT-023 | Must | (filled by test-writer) | rt-arch Module 8: services/pagination.rs (paginate) | `src/services/pagination.rs` |
| REQ-RT-024 | Must | (filled by test-writer) | rt-arch Module 8: services/pagination.rs (fetch_page) | `src/services/pagination.rs` |
| REQ-RT-025 | Must | (filled by test-writer) | rt-arch Module 8: services/pagination.rs + Module 11: cli/mod.rs (fail_empty check) | `src/services/pagination.rs`, `src/cli/mod.rs` |
| REQ-RT-026 | Must | (filled by test-writer) | rt-arch Module 7: http/api.rs (api_get_raw) + Module 11: cli/mod.rs (drive download) | `src/http/api.rs`, `src/cli/mod.rs` |
| REQ-RT-027 | Must | (filled by test-writer) | rt-arch Module 10: services/export.rs (export_document) | `src/services/export.rs` |
| REQ-RT-028 | Must | (filled by test-writer) | rt-arch Module 7: http/api.rs (api_put_bytes) + Module 11: cli/mod.rs (drive upload) | `src/http/api.rs`, `src/cli/mod.rs` |
| REQ-RT-029 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (drive resumable upload) | `src/services/drive/files.rs` |
| REQ-RT-030 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (gmail attachment download) | `src/services/gmail/message.rs`, `src/cli/mod.rs` |
| REQ-RT-031 | Should | (filled by test-writer) | rt-arch Module 10: services/export.rs (shared export) | `src/services/export.rs` |
| REQ-RT-032 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_search) | `src/cli/mod.rs` |
| REQ-RT-033 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_message_search) | `src/cli/mod.rs` |
| REQ-RT-034 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_thread_get) | `src/cli/mod.rs` |
| REQ-RT-035 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_message_get) | `src/cli/mod.rs` |
| REQ-RT-036 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_send) | `src/cli/mod.rs` |
| REQ-RT-037 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_labels) | `src/cli/mod.rs` |
| REQ-RT-038 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_drafts) | `src/cli/mod.rs` |
| REQ-RT-039 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_modify) | `src/cli/mod.rs` |
| REQ-RT-040 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_trash) | `src/cli/mod.rs` |
| REQ-RT-041 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_batch) | `src/cli/mod.rs` |
| REQ-RT-042 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_history) | `src/cli/mod.rs` |
| REQ-RT-043 | Could | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_gmail_settings) | `src/cli/mod.rs` |
| REQ-RT-044 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_events_list) | `src/cli/mod.rs` |
| REQ-RT-045 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_events_get) | `src/cli/mod.rs` |
| REQ-RT-046 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_events_create) | `src/cli/mod.rs` |
| REQ-RT-047 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_events_update) | `src/cli/mod.rs` |
| REQ-RT-048 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_events_delete) | `src/cli/mod.rs` |
| REQ-RT-049 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_calendars_list) | `src/cli/mod.rs` |
| REQ-RT-050 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_freebusy) | `src/cli/mod.rs` |
| REQ-RT-051 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_respond) | `src/cli/mod.rs` |
| REQ-RT-052 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_search) | `src/cli/mod.rs` |
| REQ-RT-053 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_calendars_crud) | `src/cli/mod.rs` |
| REQ-RT-054 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_calendar_colors) | `src/cli/mod.rs` |
| REQ-RT-055 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_list) | `src/cli/mod.rs` |
| REQ-RT-056 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_search) | `src/cli/mod.rs` |
| REQ-RT-057 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_get) | `src/cli/mod.rs` |
| REQ-RT-058 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_download) | `src/cli/mod.rs` |
| REQ-RT-059 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_upload) | `src/cli/mod.rs` |
| REQ-RT-060 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_mkdir) | `src/cli/mod.rs` |
| REQ-RT-061 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_move) | `src/cli/mod.rs` |
| REQ-RT-062 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_copy) | `src/cli/mod.rs` |
| REQ-RT-063 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_trash) | `src/cli/mod.rs` |
| REQ-RT-064 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_permissions_list) | `src/cli/mod.rs` |
| REQ-RT-065 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_permissions_create) | `src/cli/mod.rs` |
| REQ-RT-066 | Must | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_info) | `src/cli/mod.rs` |
| REQ-RT-067 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_shared_drives) | `src/cli/mod.rs` |
| REQ-RT-068 | Could | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_drive_comments) | `src/cli/mod.rs` |
| REQ-RT-069 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_docs) | `src/cli/mod.rs`, `src/services/docs/` |
| REQ-RT-070 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_sheets) | `src/cli/mod.rs`, `src/services/sheets/` |
| REQ-RT-071 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_slides) | `src/cli/mod.rs`, `src/services/slides/` |
| REQ-RT-072 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_forms) | `src/cli/mod.rs`, `src/services/forms/` |
| REQ-RT-073 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_chat) | `src/cli/mod.rs`, `src/services/chat/` |
| REQ-RT-074 | Could | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_classroom) | `src/cli/mod.rs`, `src/services/classroom/` |
| REQ-RT-075 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_tasks) | `src/cli/mod.rs`, `src/services/tasks/` |
| REQ-RT-076 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_contacts) | `src/cli/mod.rs`, `src/services/contacts/` |
| REQ-RT-077 | Should | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_people) | `src/cli/mod.rs`, `src/services/people/` |
| REQ-RT-078 | Could | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_groups) | `src/cli/mod.rs`, `src/services/groups/` |
| REQ-RT-079 | Could | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_keep) | `src/cli/mod.rs`, `src/services/keep/` |
| REQ-RT-080 | Could | (filled by test-writer) | rt-arch Module 11: cli/mod.rs (handle_appscript) | `src/cli/mod.rs`, `src/services/appscript/` |
| REQ-RT-081 | Must | (filled by test-writer) | rt-arch Module 7: http/api.rs (verbose logging) | `src/http/api.rs` |
| REQ-RT-082 | Must | (filled by test-writer) | rt-arch Module 7: http/api.rs (dry-run guard) + Module 11: cli/mod.rs | `src/http/api.rs`, `src/cli/mod.rs` |

## Specs Drift Detected

| Spec File | What Is Outdated |
|---|---|
| `specs/omega-google-architecture.md` | Lists `src/auth/refresh.rs` as a separate module -- this file does not exist. Token refresh logic should be added to `src/auth/token.rs` or a new file, matching whatever the implementation chooses. |
| `specs/omega-google-architecture.md` | Lists `src/services/export.rs` as a shared export module -- this file does not exist yet. The architecture expects it; implementation should create it. |
| `specs/omega-google-architecture.md` | Lists `src/config/credentials.rs` as containing credential file parsing logic -- actual parsing lives in `src/config/mod.rs:126-135` and `src/config/credentials.rs` is a thin wrapper calling `credentials::parse_credentials()`. This is not a problem but the architecture doc implies more logic in credentials.rs than exists. |
| `specs/omega-google-requirements.md` | Requirements REQ-AUTH-003 through REQ-AUTH-020 describe auth commands as "Must" priority but none are implemented. This runtime document provides the implementation requirements that satisfy those existing requirements. |
| `specs/SPECS.md` | Milestones M1-M6 all show "Planned" status, but M1 and M2 scaffolding (CLI parsing, types, URL builders, output formatters) is substantially complete. Status should be updated. |

## Assumptions

| # | Assumption (technical) | Explanation (plain language) | Confirmed |
|---|----------------------|---------------------------|-----------|
| 1 | `reqwest` POST for token exchange, not `oauth2` crate | We use raw HTTP calls for token exchange to maintain full control, consistent with the "raw REST" philosophy | Yes (design decision) |
| 2 | Access tokens cached between invocations alongside refresh tokens | The token stored in keyring includes the access token so we do not need to refresh on every single command run | Yes (design decision) |
| 3 | `needs_refresh()` uses 5-minute buffer before expiry | Access tokens are refreshed when less than 5 minutes remain before expiry | Yes (design decision) |
| 4 | Pagination accumulates in memory for `--all` mode | All pages are collected into a Vec before output; no streaming output | Yes (design decision) |
| 5 | Desktop OAuth timeout is 120 seconds | User has 2 minutes to complete browser consent | Yes (design decision) |
| 6 | Single shared CircuitBreaker per invocation | All API calls within one CLI run share the same circuit breaker state | Yes (design decision) |
| 7 | Manual flow uses redirect URL paste, not raw code paste | User pastes the full redirect URL (not just the auth code) for `--manual` mode. The code is extracted from URL query parameters | Yes (design decision, OOB deprecated) |
| 8 | The `keyring` crate version 3.x with `apple-native` and `linux-native` features is sufficient for OS keyring access | No additional platform-specific code needed beyond what the keyring crate provides | No -- needs verification |
| 9 | Google API responses always include `nextPageToken` for paginated results | If no more pages exist, `nextPageToken` is absent or null | Yes (Google API standard behavior) |
| 10 | Files under 5MB can use simple multipart upload | Google's recommendation; resumable upload for larger files | Yes (Google API docs) |
| 11 | All existing 1,357 tests continue to pass after runtime changes | The runtime layer is additive; existing URL builder and type tests should not be affected | No -- must verify |
| 12 | The `aes-gcm` crate is used for file-based keyring encryption only, not for OS keyring | OS keyring handles its own encryption | Yes |

## Identified Risks

| Risk | Severity | Mitigation |
|---|---|---|
| OS keyring access fails silently on CI/headless environments | High | Auto-fallback to file backend in `auto` mode. Clear error message with suggested fix. |
| Google API error responses have inconsistent format across services | Medium | Existing `format_api_error()` handles the standard format. Use `serde(flatten)` extra fields as a catch-all. |
| Token refresh race condition when multiple concurrent omega-google processes run | Low | File-based credential store uses overwrite semantics. Worst case: one process uses a slightly stale token and gets a 401, triggering its own refresh. |
| Large Drive file download causes OOM | Medium | Use streaming download (reqwest response.bytes_stream()) instead of response.bytes(). |
| OAuth local server port conflicts | Low | Using port 0 (OS-assigned) eliminates conflicts. |
| `--all` pagination on huge result sets (100K+ items) | Medium | Acceptable per design decision. Users who need streaming should use `--page` manually. |
| Service account private key exposure in logs when `--verbose` | High | Never log request bodies for token exchange endpoints. Redact Authorization header values. |
| Breaking changes to TokenData serialization format | Medium | All new fields are `Option` with serde defaults. Old data deserializes cleanly. |

## Out of Scope (Won't)

| Item | Reason |
|---|---|
| Gmail watch/push notification webhook server (`gmail watch serve`) | Requires long-running server mode, not a single CLI invocation. Deferred to M6. |
| Email tracking pixel integration (`--track` on gmail send) | Requires Cloudflare Worker backend. Deferred to M6. |
| Shell completion generation | Already scaffolded, just needs wiring. Not part of runtime plumbing. |
| Agent mode / command allowlisting enforcement | M6 polish feature. |
| sedmat document editing engine | M3 feature, complex standalone module. |
| CSV output for service-specific commands | CSV formatters need service-specific column definitions. Deferred to per-service polishing. |
| Windows keyring support | Target macOS and Linux first. Windows is best-effort via the keyring crate. |
| `Ui::confirm()` actual stdin reading | Currently returns false. Full interactive confirmation prompt is a polish item. |

## Priority Summary

| Priority | Count | Items |
|---|---|---|
| Must | 52 | REQ-RT-001 through REQ-RT-003, 005-013, 015, 017-028, 030, 032-037, 039-040, 044-050, 055-066, 081-082 |
| Should | 22 | REQ-RT-004, 014, 016, 029, 031, 038, 041-042, 051-054, 067, 069-073, 075-077 |
| Could | 8 | REQ-RT-043, 068, 074, 078-080 |
| Won't | 7 | Gmail watch serve, email tracking, shell completion, agent mode, sedmat, CSV per-service, Windows keyring |

## Implementation Order Recommendation

The requirements should be implemented in this order due to dependencies:

1. **Phase 1 -- Auth Core** (REQ-RT-001, 007, 005, 006, 013, 015): Token exchange, TokenData extension, token refresh, OS keyring backend, credential store factory. Everything else depends on working auth.

2. **Phase 2 -- Auth Flows** (REQ-RT-002, 003, 008, 009, 010, 011, 012): Desktop/manual OAuth flows, auth add/remove/list/status commands. Users can now log in.

3. **Phase 3 -- Execution Infrastructure** (REQ-RT-017, 018, 019, 020, 021, 022, 023, 024, 025, 081, 082): Auth bootstrap, ServiceContext factory, API call helpers, pagination, verbose/dry-run. The generic execution path that all handlers use.

4. **Phase 4 -- Core Service Handlers** (REQ-RT-032-040, 044-050, 055-066): Gmail, Calendar, Drive handlers. The three most important services.

5. **Phase 5 -- File I/O** (REQ-RT-026, 027, 028, 030): Download/upload/export. Required for Drive download/upload commands to fully work.

6. **Phase 6 -- Extended Services** (REQ-RT-069-080): Docs, Sheets, Slides, Forms, Chat, Tasks, Contacts, People, Groups, Keep, Apps Script. All follow identical patterns established in Phase 4.

7. **Phase 7 -- Polish** (REQ-RT-004, 014, 016, 029, 031): Remote flow, encrypted file backend, keyring timeout, resumable upload, shared export module.