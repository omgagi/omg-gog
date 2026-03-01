# SPECS.md -- omega-google Technical Specifications

> Master index of all technical specification documents for omega-google.

## Specification Files

| File | Description | Status |
|------|-------------|--------|
| [omega-google-requirements.md](omega-google-requirements.md) | Full requirements specification for omega-google (229 requirements across 24 categories) | Active |
| [omega-google-architecture.md](omega-google-architecture.md) | Complete system architecture: module structure, interfaces, data flow, failure modes, security model, performance budgets, Nix build, traceability | Active |
| [runtime-requirements.md](runtime-requirements.md) | Runtime layer requirements: OAuth flows, token refresh, keyring storage, service execution infrastructure, pagination, file I/O, core service handlers (82 requirements) | Active |
| [runtime-architecture.md](runtime-architecture.md) | Runtime layer architecture: module design, interfaces, failure modes, security model, performance budgets, milestones (RT-M1 through RT-M7), traceability for all 82 runtime requirements | Active |

## Project Overview

omega-google is a Rust reimplementation of gogcli: a CLI for 15 Google Workspace services with JSON-first output, multiple accounts, least-privilege OAuth, and OS keyring credential storage.

## Milestones

| Milestone | Scope | Status |
|-----------|-------|--------|
| M1 | Project scaffolding, Nix, auth, config, HTTP client, output formatting, UI | Complete (scaffolding) |
| M2 | Core services: Gmail, Calendar, Drive + desire path aliases | Complete (scaffolding) |
| M3 | Productivity services: Docs (incl. sedmat), Sheets, Slides, Forms | Complete (scaffolding) |
| M4 | Collaboration services: Chat, Classroom, Tasks, Contacts, People | Complete (scaffolding) |
| M5 | Admin/Workspace services: Groups, Keep, Apps Script | Complete (scaffolding) |
| M6 | Polish: Tracking, completion, allowlisting, agent mode, full tests, CI | Complete (scaffolding) |
| RT | Runtime layer: OAuth flows, token refresh, service execution, pagination, file I/O | Active |
| RT-M1 | Auth Core: token exchange, TokenData extension, refresh, keyring backend, credential factory | Planned |
| RT-M2 | Auth Flows: desktop/manual OAuth, auth add/remove/list/status CLI handlers | Planned |
| RT-M3 | Execution Infrastructure: API helpers, pagination, bootstrap, verbose/dry-run | Planned |
| RT-M4 | Core Service Handlers: Gmail, Calendar, Drive handler wiring | Planned |
| RT-M5 | File I/O: streaming download, simple upload, attachment download, export | Planned |
| RT-M6 | Extended Service Handlers: remaining 12 services | Planned |
| RT-M7 | Polish: remote OAuth, encrypted file backend, resumable upload | Planned |
