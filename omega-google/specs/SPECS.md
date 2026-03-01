# SPECS.md -- omega-google Technical Specifications

> Master index of all technical specification documents for omega-google.

## Specification Files

| File | Description | Status |
|------|-------------|--------|
| [omega-google-requirements.md](omega-google-requirements.md) | Full requirements specification for omega-google (229 requirements across 24 categories) | Active |
| [omega-google-architecture.md](omega-google-architecture.md) | Complete system architecture: module structure, interfaces, data flow, failure modes, security model, performance budgets, Nix build, traceability | Active |

## Project Overview

omega-google is a Rust reimplementation of gogcli: a CLI for 15 Google Workspace services with JSON-first output, multiple accounts, least-privilege OAuth, and OS keyring credential storage.

## Milestones

| Milestone | Scope | Status |
|-----------|-------|--------|
| M1 | Project scaffolding, Nix, auth, config, HTTP client, output formatting, UI | Planned |
| M2 | Core services: Gmail, Calendar, Drive + desire path aliases | Planned |
| M3 | Productivity services: Docs (incl. sedmat), Sheets, Slides, Forms | Planned |
| M4 | Collaboration services: Chat, Classroom, Tasks, Contacts, People | Planned |
| M5 | Admin/Workspace services: Groups, Keep, Apps Script | Planned |
| M6 | Polish: Tracking, completion, allowlisting, agent mode, full tests, CI | Planned |
