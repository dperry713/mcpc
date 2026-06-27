# MCP Compiler (mcpc)

The MCP Compiler (`mcpc`) is a powerful, deterministic tool for generating and orchestrating cloud-native backend architectures from a declarative specification. It compiles a single `mcp.spec.json` into distributed rust microservices, complete with infrastructure-as-code manifests.

## Features
- **DAG Compilation**: Resolves dependencies using a Directed Acyclic Graph to ensure accurate build order.
- **Incremental Caching**: Only rebuilds modules that have changed, based on deep hashing of AST definitions.
- **Distributed Execution**: Built-in `worker` daemon allows compilation jobs to be dispatched to remote build nodes.
- **Infrastructure-as-Code**: Dynamically generates `Dockerfile`, Helm Charts, and `docker-compose.yml`.
- **Plugin System**: JSON-RPC plugin architecture for hooking into validation and compilation phases.

## Installation
Ensure you have Rust and Cargo installed, then build:
```bash
cargo build --release
```

## Quick Start

### 1. Define your Spec
Create a `mcp.spec.json` file in your root directory:
```json
{
  "name": "my-mcp-cluster",
  "modules": [
    {
      "name": "control-plane",
      "type": "default"
    },
    {
      "name": "gateway",
      "type": "api",
      "dependencies": ["control-plane"]
    }
  ]
}
```

### 2. Build the Workspace
```bash
mcpc build
```
This generates an `automata-mcp` directory containing the rust crates configured as a cargo workspace, `Dockerfile` and helm charts for each module, and a unified `docker-compose.yml`.

### 3. Run the Cluster
```bash
cd automata-mcp
docker compose up --build
```

## CLI Usage
`mcpc` provides the following commands:
- `mcpc build [--remote <URL>]`: Compiles the MCP specifications into rust modules. Optionally dispatches builds to a remote worker.
- `mcpc validate`: Statically analyzes the spec for circular dependencies or schema violations.
- `mcpc clean`: Clears the generated `automata-mcp` and caches.
- `mcpc worker`: Starts a remote builder node on port 50051.
- `mcpc run`: Orchestrates the final run command.

## Architecture & Schemas

### `mcp.spec.json` Schema
Defines the input specifications:
```json
{
  "name": "project_name",
  "modules": [
    {
      "name": "module_name",
      "type": "api | worker | default | agent",
      "entry": "src/main.rs",
      "features": ["feature1", "feature2"],
      "dependencies": ["other_module"]
    }
  ]
}
```

### Manifest output (`manifest.json`)
The build outputs a trace of its execution:
```json
{
  "built_modules": ["control-plane", "gateway"],
  "skipped_modules": [],
  "timestamp": "2026-06-27T12:00:00Z"
}
```

### Incremental Cache (`.mcpc/cache.json`)
`mcpc` hashes module specifications to avoid unnecessary builds:
```json
{
  "gateway": "79b4a11f2a36b361",
  "control-plane": "14f2e519c925b41"
}
```

