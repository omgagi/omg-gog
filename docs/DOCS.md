# DOCS.md -- omega-google Documentation

> Master index of all user-facing and developer documentation for omega-google.

## Documentation Files

| File | Description | Audience |
|------|-------------|----------|
| [developer-guide.md](developer-guide.md) | Developer setup, build instructions, module guide, testing strategy | Developers |
| [command-reference.md](command-reference.md) | CLI command tree with all flags and subcommands | Users, Agents |

## Quick Links

- **Specs**: See `specs/SPECS.md` for technical specifications
- **Requirements**: See `specs/omega-google-requirements.md` for 229 formal requirements
- **Architecture**: See `specs/omega-google-architecture.md` for system design

## Project Overview

omega-google is a Rust CLI for 15 Google Workspace services. It replaces the Go tool `gogcli` with idiomatic Rust, using raw REST API calls, OS keyring credential storage, and JSON-first output for scripting and LLM agent integration.
