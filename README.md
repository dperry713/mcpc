# MCPC - Multi-Component Platform Compiler

**Declarative Rust service & agent generator with incremental builds, Docker + Kubernetes scaffolding.**

## Features

- Declarative `mcp.spec.json`
- Dependency graph & topological execution
- Incremental builds via content hashing
- Generates full Rust crates, Dockerfiles, Helm charts
- Plugin system
- Dry-run & watch mode (coming)

## Quick Start

```bash
cargo install mcpc
mcpc build
```

## Example `mcp.spec.json`

See `mcp.spec.json` in the repo.

## Commands

- `mcpc build [--remote <URL>] [--dry-run]`
- `mcpc validate`
- `mcpc run`
- `mcpc clean`

Built with ❤️ for the automata ecosystem.