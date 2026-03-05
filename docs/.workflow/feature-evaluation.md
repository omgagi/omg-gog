# Feature Evaluation: omg-gog OMEGA Integration

## Feature Description
Five changes to integrate omg-gog with the OMEGA personal AI agent ecosystem:

1. **OMEGA_STORES_DIR env var** -- When set, read all credential data (OAuth client credentials, refresh tokens, access tokens) from `$OMEGA_STORES_DIR/google.json` instead of the default `~/.config/omega-google/` + OS keyring. No behavior change when unset.
2. **Gmail watch start/stop/status** -- New subcommands to register/manage Gmail push notification watches via Google's `users.watch()` API. omg-gog only registers the watch; OMEGA's server receives the notifications.
3. **Calendar watch start/stop/status** -- Same pattern for Google Calendar `events.watch()` push notifications to a callback URL.
4. **Drive watch start/stop/status** -- Same pattern for Google Drive `changes.watch()` push notifications for file changes.
5. **Webhook serve (testing)** -- Minimal HTTP server to receive and print Google push notifications locally for development/testing.

## Evaluation Summary

| Dimension | Score (1-5) | Assessment |
|-----------|-------------|------------|
| D1: Necessity | 4 | Credential duplication between `~/.config/omega-google/` and `~/.omega/stores/google.json` is a real operational problem for the single developer. Watch commands enable real-time OMEGA awareness of email/calendar/drive changes -- a genuine capability gap. |
| D2: Impact | 4 | Transforms omg-gog from a standalone CLI into an integrated OMEGA skill with real-time event awareness. This is a qualitative capability upgrade for the OMEGA agent system. |
| D3: Complexity Cost | 4 | Change 1 is a small, isolated modification to `config/mod.rs` and `auth/keyring.rs`. Changes 2-4 follow an identical pattern (Gmail watch scaffolding already exists at `src/services/gmail/watch.rs` and `src/cli/gmail.rs:598-624`). Change 5 requires a new HTTP server dependency but is isolated. |
| D4: Alternatives | 4 | No existing tool provides a CLI interface for Google push notification registration. Credential unification could be done with symlinks, but that is fragile and error-prone. The OMEGA_STORES_DIR approach is the clean solution. |
| D5: Alignment | 5 | The Idea Brief (line 1) states the tool is for "LLM/agent tool use." The MEMORY.md confirms "OMEGA is the developer's personal AI agent that already uses omg-gog." The project was built to be an OMEGA skill -- this formalizes that relationship. |
| D6: Risk | 4 | Change 1 is behind an env var (zero risk to default behavior). Changes 2-4 are new additive commands that cannot break existing functionality. Change 5 is a testing utility. The `CredentialStore` trait in `src/auth/mod.rs:48-63` already abstracts credential backends, making the OMEGA store a clean new implementation. |
| D7: Timing | 3 | The runtime layer milestones (RT-M1 through RT-M7 in `specs/SPECS.md`) are all still "Planned." The watch commands need a working authenticated HTTP client to actually call Google APIs. Change 1 (OMEGA_STORES_DIR) can land immediately. Changes 2-5 depend on the runtime layer being functional for the relevant services. |

**Feature Viability Score: 4.0 / 5.0**

```
FVS = (D1 + D2 + D5) x 2 + (D3 + D4 + D6 + D7)
    = (4 + 4 + 5) x 2 + (4 + 4 + 4 + 3)
    = 26 + 15
    = 41 / 10
    = 4.1
```

## Verdict: GO

This feature formalizes the integration between omg-gog and its primary consumer (the OMEGA agent). It solves a real credential management problem and adds a genuinely new capability (push notification watch registration) that no alternative tool provides. The complexity is well-contained: Change 1 is trivial, Changes 2-4 follow an established pattern already scaffolded in the codebase, and Change 5 is an optional testing convenience.

## Detailed Analysis

### What Problem Does This Solve?

**Credential duplication (Change 1):** Currently, OMEGA stores its Google credentials at `~/.omega/stores/google.json` while omg-gog expects them at `~/.config/omega-google/` with tokens in the OS keyring. When OMEGA spawns omg-gog as a subprocess, the credentials must exist in both locations. This is a real operational pain point: token refreshes in one location are invisible to the other, and manual synchronization is error-prone.

**Push notification registration (Changes 2-4):** OMEGA's server at `omgagi.ai` needs to receive real-time notifications from Google when emails arrive, calendar events change, or files are modified. Google's push notification system requires explicit API calls to register a watch with a callback URL and topic/channel ID. Currently there is no CLI interface to manage these registrations. Without this, OMEGA must poll Google APIs on a schedule, which is both slower and more API-quota-intensive than push notifications.

**Testing (Change 5):** When developing and debugging watch registrations, the developer needs to verify that Google is actually sending notifications. A minimal local webhook receiver that prints incoming notifications to stdout eliminates the need to deploy to the production server for testing.

### What Already Exists?

**Gmail watch is already scaffolded.** The CLI structure at `/Users/isudoajl/ownCloud/Projects/omega-cortex/omega-tools/omg-gog/src/cli/gmail.rs:598-624` defines `GmailWatchCommand` with `Start`, `Status`, `Renew`, and `Stop` variants. The service layer at `/Users/isudoajl/ownCloud/Projects/omega-cortex/omega-tools/omg-gog/src/services/gmail/watch.rs` has `build_watch_start_url()` and `build_watch_stop_url()` functions. The requirement `REQ-GMAIL-012` in `specs/omega-google-requirements.md:152` already specifies Gmail watch with start/status/renew/stop/serve.

**Calendar and Drive have no watch scaffolding.** Neither `/Users/isudoajl/ownCloud/Projects/omega-cortex/omega-tools/omg-gog/src/cli/calendar.rs` nor `/Users/isudoajl/ownCloud/Projects/omega-cortex/omega-tools/omg-gog/src/cli/drive.rs` contain any watch-related commands. The service layers (`src/services/calendar/` and `src/services/drive/`) have no watch modules.

**The credential abstraction is ready for extension.** The `CredentialStore` trait at `/Users/isudoajl/ownCloud/Projects/omega-cortex/omega-tools/omg-gog/src/auth/mod.rs:48-63` defines `get_token`, `set_token`, `delete_token`, `list_tokens`, `keys`, `get_default_account`, and `set_default_account`. A new `OmegaStoreCredentialStore` struct reading from `$OMEGA_STORES_DIR/google.json` would implement this trait cleanly.

**The config module already supports env var overrides.** `/Users/isudoajl/ownCloud/Projects/omega-cortex/omega-tools/omg-gog/src/config/mod.rs:47-49` checks `OMEGA_GOOGLE_CONFIG_DIR` env var for config directory override. Adding `OMEGA_STORES_DIR` follows the same pattern.

**The `serve` concept is deferred in current plans.** `specs/runtime-requirements.md:403` explicitly notes "Gmail watch serve requires long-running server mode, deferred to M6."

### Complexity Assessment

**Change 1 (OMEGA_STORES_DIR):** Requires a new `CredentialStore` implementation that reads/writes a single JSON file. The file format must contain: OAuth client credentials (client_id, client_secret), refresh token, access token, expiration, email, and scopes. Estimated: ~80-120 lines for the new store implementation, ~10 lines to check the env var and select the store backend. Touches: `src/auth/mod.rs` or a new `src/auth/omega_store.rs`, plus the credential store selection logic (likely in bootstrap/init code). **Low complexity, isolated change.**

**Changes 2-4 (Watch commands):** Each watch registration requires: (a) CLI arg structs with callback URL, channel ID/token, and optional filters; (b) URL builder functions for the Google API endpoints; (c) request body builder; (d) response type definitions. Gmail already has (a) and partial (b). Calendar and Drive need all four. However, Google's push notification API uses the same `Channel` resource structure across all three services (id, type, address, token, expiration). A shared `WatchChannel` type in `src/services/common.rs` could serve all three. Estimated: ~150-200 lines of new code per service (CLI args, URL builders, types), with ~50 lines of shared channel types. **Moderate complexity, repetitive pattern.**

**Change 5 (Webhook serve):** Requires an HTTP server that listens on a port and prints incoming POST bodies. The project already depends on `tokio` with full features. Adding a minimal HTTP listener (using `tokio::net::TcpListener` or adding `axum`/`warp` as an optional dependency) is straightforward. Estimated: ~100-150 lines. This could be behind a feature flag to avoid adding the HTTP server dependency to the default build. **Low-moderate complexity, fully isolated.**

**Maintenance burden:** Low. Google's push notification API has been stable since 2015. The OMEGA_STORES_DIR integration is a simple env var check. The watch commands are thin wrappers around well-documented Google API endpoints. The webhook serve is a development utility unlikely to need frequent updates.

### Risk Assessment

- **Zero risk to existing behavior:** Change 1 is behind an env var that defaults to unset. Changes 2-5 are additive new commands. No existing code paths are modified.
- **Security consideration:** The `OMEGA_STORES_DIR` store will hold credentials in a single JSON file. It must enforce `0600` permissions, matching the existing `FileCredentialStore` pattern at `src/auth/keyring.rs:47-50`. The webhook serve (Change 5) should bind to `127.0.0.1` only by default.
- **Dependency risk:** Change 5 may require a new HTTP server dependency (e.g., `axum`, `warp`, or `hyper`). This increases the dependency tree. However, since the project already depends on `reqwest` which pulls in `hyper`, the actual new dependency weight is minimal.
- **Timing dependency:** Changes 2-4 require a working authenticated HTTP client to make real API calls. Per `specs/SPECS.md`, the runtime milestones (RT-M1 through RT-M7) are all "Planned." The watch URL builders and CLI args can be scaffolded now, but the commands will not function until the runtime layer is operational. This is not a blocker -- it follows the same pattern as the existing codebase where M1-M6 scaffolding was built before the runtime.

## Conditions
None -- feature approved for pipeline entry.

## Alternatives Considered

- **Symlinks for credential unification**: Symlink `~/.config/omega-google/` to `~/.omega/stores/`. Fragile: token format differs (keyring stores tokens as JSON in OS secret storage, not files). Does not solve the structural mismatch. Verdict: workaround, not a solution.
- **Google Cloud Pub/Sub directly**: Instead of watch commands in omg-gog, OMEGA could use the Google Cloud Pub/Sub API directly. However, the `users.watch()` Gmail API and `events.watch()` Calendar API are the registration endpoints -- Pub/Sub is just the transport. Someone still needs to call the registration API. Verdict: complementary, not alternative.
- **Polling instead of push**: OMEGA could poll Gmail/Calendar/Drive on a schedule. Works but wastes API quota, introduces latency (minimum 1-minute polling intervals to stay within quotas vs. near-instant push), and is the less capable approach. Verdict: inferior fallback.
- **Build only Change 1 now, defer Changes 2-5**: Since the runtime layer is not yet functional, only the OMEGA_STORES_DIR change can be immediately useful. The watch commands could be deferred until RT-M4 (Core Service Handlers) is complete. This is a valid phasing strategy but the user has explicitly proposed all five as a package with ordered implementation (Change 1 first). Verdict: valid phasing, but the user's proposed order already accounts for this.

## Recommendation

Proceed with all five changes. The implementation order proposed by the user is sound: Change 1 (OMEGA_STORES_DIR) first as a standalone foundation that delivers immediate value, then Changes 2-4 (watch commands) as a repeating pattern that can be scaffolded now and wired to real API calls when the runtime layer lands, then Change 5 (webhook serve) as a testing convenience.

The Analyst should note:
- Change 1 should implement the `CredentialStore` trait from `src/auth/mod.rs:48-63` to ensure seamless integration with the existing auth infrastructure.
- Changes 2-4 should share a common `WatchChannel` type since Google uses the same channel resource across Gmail, Calendar, and Drive push notification APIs.
- The Gmail watch already has CLI scaffolding (`src/cli/gmail.rs:598-624`) and URL builders (`src/services/gmail/watch.rs`) that should be extended, not replaced.
- Calendar and Drive watch commands need to be added to their respective CLI enum types (`CalendarCommand` in `src/cli/calendar.rs` and `DriveCommand` in `src/cli/drive.rs`).
- Change 5 should bind to `127.0.0.1` only by default and should consider being behind a Cargo feature flag to keep the default binary lean.

## User Decision
[Awaiting user response: PROCEED / ABORT / MODIFY]
