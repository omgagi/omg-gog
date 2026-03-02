# Architect Progress: omega-google

## Date: 2026-03-01

## Status: COMPLETE

## Deliverables Created

1. **Architecture Spec**: `specs/omega-google-architecture.md`
   - Full project structure (directory/module layout)
   - 11 module definitions with public interfaces, dependencies, implementation order
   - Trait and struct definitions (Rust code) for all key interfaces
   - Data flow diagrams (command execution, token refresh)
   - Failure modes per module and system-level
   - Security model with trust boundaries, data classification, attack surface
   - Performance budgets per operation
   - Graceful degradation table
   - Design decision rationale (10 decisions)
   - External dependencies (17 runtime + 6 dev crates)
   - Complete Nix flake.nix structure
   - Milestone implementation guide (M1-M6 build order)
   - Full requirement traceability (229 requirements mapped to modules)

2. **SPECS.md index updated**: `specs/SPECS.md`
   - Added architecture spec entry

3. **DOCS.md index created**: `docs/DOCS.md`
   - Master documentation index

4. **Developer Guide**: `docs/developer-guide.md`
   - Setup instructions (Nix and non-Nix)
   - Module overview
   - Coding conventions (errors, API types, output, async, testing)
   - Guide for adding new services

5. **Command Reference**: `docs/command-reference.md`
   - Complete CLI command tree with all flags
   - Global flags table
   - All 15 services documented
   - Desire path aliases table
   - Exit codes table

6. **Traceability Matrix updated**: `specs/omega-google-requirements.md`
   - All 229 requirements now have Architecture Section and Implementation Module columns filled

## Key Design Decisions (Initial Architecture)

- Single binary (not Cargo workspace) for simplicity
- Raw REST API calls via reqwest (no generated clients)
- ServiceContext pattern for shared command handler dependencies
- serde(flatten) on all API response types for forward compatibility
- thiserror for typed library errors, anyhow for CLI-level chaining
- Keyring service name: `omega-google` (fresh namespace)
- Nix flake with crane for reproducible builds
- chrono + chrono-tz for flexible date/time parsing

---

## Runtime Layer Architecture: 2026-03-01

### Status: COMPLETE

### Deliverables Created

7. **Runtime Architecture Spec**: `specs/runtime-architecture.md`
   - 11 module definitions covering auth, HTTP, services, CLI layers
   - Detailed public interfaces with Rust function signatures
   - 4 new files designed: `auth/oauth_flow.rs`, `http/api.rs`, `services/pagination.rs`, `services/export.rs`
   - 11 modified files documented with exact change descriptions
   - Failure modes per module (40+ failure scenarios)
   - Security model: trust boundaries, data classification, attack surface, token redaction
   - Performance budgets: token refresh <2s, bootstrap <2.5s, streaming downloads
   - Graceful degradation: OS keyring fallback, circuit breaker, broken pipe handling
   - 7 milestones (RT-M1 through RT-M7) with dependency graph
   - Implementation order within each milestone
   - Complete traceability matrix: all 82 requirements mapped to architecture modules
   - Data flow diagrams: auth flow, service command flow, pagination flow, download flow
   - 9 design decisions with alternatives and justification

8. **SPECS.md index updated**: Added runtime-architecture.md entry and RT-M1 through RT-M7 sub-milestones

9. **Traceability matrix updated**: `specs/runtime-requirements.md`
   - All 82 requirements now have Architecture Section column filled with specific rt-arch module references

### Key Design Decisions (Runtime)

- Raw reqwest POST for token exchange (not oauth2 crate) for consistency with "raw REST" philosophy
- Cache access_token in credential store (saves 500ms-2s per command)
- OS-assigned port for OAuth server (eliminates port conflicts)
- Single CircuitBreaker per CLI invocation (Arc<CircuitBreaker> shared across all API calls)
- Pagination accumulates in memory (--page available for manual pagination of huge sets)
- Manual flow accepts redirect URL (not raw auth code) since Google deprecated OOB
- File-based key index alongside OS keyring (OS keyrings lack enumeration APIs)
- futures-util crate needed for streaming response body in downloads/exports
- bootstrap_service_context as single entry point for all authenticated commands
