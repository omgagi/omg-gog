# Feature Evaluation: Runtime Layer (OAuth, HTTP Execution, Pagination, File I/O, Token Refresh)

## Feature Description
Make omega-google functionally competitive with gogcli by implementing the runtime layer: real OAuth flows (browser + manual + remote), keyring token storage (macOS Keychain, Linux Secret Service, encrypted file fallback), actual HTTP calls to Google APIs, response handling, pagination execution, file upload/download, and transparent token refresh. This transforms omega-google from a skeleton that prints "Command registered" into a tool that actually talks to Google.

## Evaluation Summary

| Dimension | Score (1-5) | Assessment |
|-----------|-------------|------------|
| D1: Necessity | 5 | Without this, omega-google is a 28K-line no-op. Every service handler is a stub that prints "Command registered" and exits. |
| D2: Impact | 5 | Transforms the project from zero utility to a functional Google Workspace CLI. This is the entire value proposition. |
| D3: Complexity Cost | 2 | Cross-cutting change touching auth, HTTP, and all 15 service handler dispatch paths. Requires OAuth server, token refresh loop, pagination generics, multipart upload, and resumable download. |
| D4: Alternatives | 4 | No existing Rust CLI covers 15 Google Workspace services. google-apis-rs is in maintenance mode seeking a new maintainer. No alternative delivers this project's stated goal. |
| D5: Alignment | 5 | The Idea Brief's entire purpose is "a Rust reimplementation of gogcli." The runtime layer is not a tangent -- it IS the project. |
| D6: Risk | 3 | OAuth flows interact with external Google endpoints and OS keyrings. Token handling has security implications. However, the existing architecture (CredentialStore trait, RetryableRequest, CircuitBreaker) was designed for this. |
| D7: Timing | 4 | M1/M2 scaffolding is complete: CLI parsing, types, URL builders, body builders, output formatting, retry logic, circuit breaker. The runtime layer is the natural next step. Only minor concern: no existing test infrastructure for live API calls. |

**Feature Viability Score: 4.0 / 5.0**

```
FVS = (D1 + D2 + D5) x 2 + (D3 + D4 + D6 + D7)
    = (5 + 5 + 5) x 2 + (2 + 4 + 3 + 4)
    = 30 + 13
    = 43 / 10
    = 4.3
```

## Verdict: GO

This is the feature that gives the project a reason to exist. The 28,052 lines of Rust code currently written produce a binary that parses arguments, builds URLs, defines types, and then prints "Command registered" for every real operation. Without the runtime layer, the project has zero functional value to any user. The complexity cost is real (score 2), but the alternative is a permanently inert codebase.

## Detailed Analysis

### What Problem Does This Solve?

omega-google currently cannot perform any Google Workspace operation. Every service handler in `src/cli/mod.rs` (lines 568-708) terminates with `eprintln!("Command registered. API call requires: omega-google auth add <email>")`. The OAuth code exchange function in `src/auth/oauth.rs:55-61` explicitly bails with "OAuth code exchange not yet implemented." The service account JWT exchange in `src/auth/service_account.rs:72-76` does the same.

The project's stated goal (from the Idea Brief) is "feature parity with gogcli." gogcli is a production tool with 5,200+ stars that actually calls Google APIs. omega-google is a type-safe argument parser that does nothing.

### What Already Exists?

The codebase has substantial infrastructure already built for this exact purpose:

- **Auth module** (`src/auth/`): `CredentialStore` trait with `get_token`/`set_token`/`delete_token` (mod.rs:47-55), `FileCredentialStore` implementation with Unix permissions (keyring.rs), token serialization/deserialization (token.rs), OAuth URL generation (oauth.rs:28-51), `TokenResponse` struct (oauth.rs:64-71), `needs_refresh` logic (token.rs:75-78), service account JWT structure (service_account.rs), scope mappings for all 15 services (scopes.rs, 598 lines)
- **HTTP module** (`src/http/`): `build_authenticated_client` with Bearer token injection (client.rs:28-45), `execute_with_retry` middleware with circuit breaker integration (middleware.rs:35-108), exponential backoff with jitter (retry.rs), `CircuitBreaker` with configurable thresholds (circuit_breaker.rs, 250 lines), `RetryableRequest` with body replay (middleware.rs:14-31)
- **Service modules** (`src/services/`): URL builders for all endpoints (gmail/search.rs, drive/list.rs, calendar/events.rs, etc.), request body builders, response type definitions with serde derives, helper functions (query building, MIME construction, A1 notation parsing)
- **Output module** (`src/output/`): JSON/plain/text formatters, field selection via `--select`, results-only filtering
- **Dependencies** (`Cargo.toml`): reqwest with rustls-tls, oauth2 crate, keyring crate with apple-native and linux-native features, jsonwebtoken for service accounts, tokio with full features, aes-gcm for encryption

The gap is the connective tissue: nothing calls `build_authenticated_client` with a real token, nothing calls `execute_with_retry` with a real request, nothing loops through paginated responses, nothing handles file streams.

### Complexity Assessment

This is a high-complexity feature (score 2) with the following components:

1. **OAuth flow implementation** (3 modes): Desktop flow requires starting a local HTTP server on an ephemeral port, opening a browser, receiving the redirect callback, and exchanging the authorization code. Manual flow requires printing a URL and reading a pasted code. Remote flow adds a two-step device authorization. Estimated: 200-400 lines of new code in `src/auth/oauth.rs`.

2. **Token refresh**: The `needs_refresh` function exists but nothing calls it or performs the actual refresh HTTP request. Requires intercepting requests, checking token age, refreshing via POST to `TOKEN_URL`, and updating the keyring. Estimated: 50-100 lines, likely in a new `src/auth/refresh.rs` or added to `token.rs`.

3. **Service handler wiring**: All 15 service dispatch paths in `src/cli/mod.rs` must change from stubs to real flows: resolve account, get token, refresh if needed, build client, build request, execute with retry, deserialize response, format output. This is repetitive but touches every service. Estimated: substantial changes across `src/cli/mod.rs` (currently 1,069 lines) and possibly new per-service execution modules.

4. **Pagination execution**: URL builders already accept `page_token` parameters. The runtime needs a generic pagination loop: call API, check for `nextPageToken` in response, accumulate results, repeat. Estimated: 50-80 lines of generic pagination in `src/services/common.rs` or `src/http/`.

5. **File upload/download**: Drive upload requires multipart MIME construction. Drive download requires streaming response bodies to files with progress indication. Gmail attachment download is simpler (base64 decode). Estimated: 150-300 lines across Drive and Gmail service modules.

6. **OS keyring integration**: The `keyring` crate is already a dependency with platform features. The `FileCredentialStore` exists. An `OsKeyringStore` implementing `CredentialStore` needs to wrap the `keyring` crate. Estimated: 50-100 lines.

**Total estimated new code**: 500-1,100 lines of runtime logic, plus modifications to ~700 lines of existing stub handlers.

**Maintenance burden**: Moderate. OAuth flows and token refresh are stable patterns unlikely to change. The main ongoing cost is tracking Google API changes (response format changes, new error codes, scope changes), which is inherent to the project regardless.

### Risk Assessment

- **Security**: Token storage in the keyring must use correct permissions. The `FileCredentialStore` already sets `0o600` on Unix (keyring.rs:47-50). Refresh tokens must never leak to stdout -- token.rs:15-16 already documents this constraint. OAuth redirect server must bind only to localhost.
- **Breakage**: The existing 1,357 tests are all unit tests against URL builders, type serialization, and utility functions. None test live API calls. Adding the runtime layer should not break existing tests since it adds new code paths rather than modifying tested logic.
- **External dependency**: OAuth flows depend on Google's token endpoint availability. Token refresh depends on Google not revoking scopes. These are inherent to the project's purpose, not risks specific to this feature.
- **Scope risk**: The feature description is broad ("functionally competitive with gogcli"). If interpreted as "implement all 272 command handlers," it is a multi-month effort. If scoped to "implement the runtime plumbing so that individual commands can be wired up incrementally," it is a 1-2 week effort. **Recommendation: scope to the plumbing, not all commands.**

## Conditions
None -- feature approved for pipeline entry.

## Alternatives Considered

- **google-apis-rs (Byron/google-apis-rs)**: Generated Rust bindings for all Google APIs. In maintenance mode, seeking new maintainer. Generates per-API crates, not a unified CLI. Does not replace the need for omega-google's auth, output, and CLI layers. Could theoretically be used as a library dependency instead of raw reqwest calls, but the project's Idea Brief explicitly chose "raw REST API calls" for full control. Verdict: not a replacement.
- **google-workspace-apis crate**: Available on crates.io but minimal adoption. Does not provide a CLI. Would still require all the same auth, output, and dispatch plumbing. Verdict: not a replacement.
- **Continue using gogcli (Go)**: The existing Go tool works. The user could simply keep using it instead of reimplementing in Rust. However, the explicit project goal is a Rust reimplementation for the omega-tools ecosystem. The user has already invested 28K lines and 1,357 tests toward this goal. Verdict: technically viable but contradicts project intent.
- **Implement a minimal subset first**: Instead of targeting all 15 services, implement the runtime for Gmail, Calendar, and Drive only (M2 scope). This delivers 80% of practical value for approximately 40% of the work. **This is the recommended approach** and aligns with the Idea Brief's "MVP Scope" section.

## Recommendation

Proceed. This is the defining feature of the project -- without it, omega-google is a 28K-line argument parser with no functional output. The existing infrastructure (CredentialStore trait, HTTP middleware, retry/circuit-breaker, URL builders, type definitions) was explicitly designed as scaffolding for this runtime layer.

**Scope recommendation**: Structure the work as the Idea Brief already suggests -- implement the runtime plumbing (OAuth flows, token refresh, authenticated request execution, pagination, file I/O) as infrastructure, then wire up M2 services (Gmail, Calendar, Drive) first. The remaining 12 services can be wired incrementally using the same patterns.

**Key risks to surface to the Analyst**: (1) OAuth desktop flow requires a local HTTP server and browser interaction, which complicates testing. (2) The `keyring` crate's behavior varies across OS platforms -- test on macOS and Linux. (3) Pagination and file upload/download should be implemented as generic utilities in `src/services/common.rs` or `src/http/`, not duplicated per service.

## User Decision
[Awaiting user response: PROCEED / ABORT / MODIFY]
