# MCPC (Multi-Component Platform Compiler)

The Cargo for distributed agent & service platforms. MCPC is a delightful, declarative orchestrator that turns a single `mcp.spec.json` into a complete, incrementally-updatable, polyglot-ready distributed system with best-in-class Developer Experience.

## Features

- **Interactive Visual Editor**: Launch the sleek, Tauri-based GUI dashboard to visually design your system. Drag-and-drop node dependencies, configure functions on the fly, and auto-sync changes directly to your JSON spec.
- **Topological Incremental Builds**: A robust DAG-based incremental build system using intelligent content hashing. Only rebuilds what changed!
- **Declarative Spec**: Single source of truth (`mcp.spec.json`) to define your entire service mesh.
- **Modern Minimalist Aesthetics**: Professional-grade developer dashboard powered by React Flow, Vite, and Tailwind v4.

## Quick Start

### 1. The CLI

Run standard compiler commands right from your terminal:

```bash
# (Install from source or crates.io once published)
cargo install --path .
mcpc build
```

**Commands:**
- `mcpc build [--remote <URL>] [--dry-run]` - Builds the workspace natively.
- `mcpc validate` - Validates the structural integrity of your spec.
- `mcpc run` - Bootstraps the execution environment.
- `mcpc clean` - Removes the generated workspace.

### 2. The GUI Dashboard

Launch the fully interactive visual IDE to manage your spec graphically:

```bash
cd mcpc-gui
npm install
npm run tauri dev
```

**GUI Features:**
- **Visual Spec Editor**: Bi-directional data sync with `mcp.spec.json`.
- **Drag & Drop Pathways**: Physically draw connections between nodes to wire up architecture dependencies.
- **Native Backend Binding**: Fire compiler `build` and `validate` actions natively through the dashboard with zero latency.
- **Minimap & Custom Layouts**: Seamlessly navigate large architectures with minimaps and horizontal/vertical orientations.

---
Built with ❤️ for the automata ecosystem.