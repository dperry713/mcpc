# MCPC (Multi-Component Platform Compiler)

The Cargo for distributed agent & service platforms. MCPC is a delightful, declarative orchestrator that turns a single `mcp.spec.json` into a complete, incrementally-updatable, polyglot-ready distributed system with best-in-class Developer Experience.

MCPC is built in Rust and comes with both a blazingly fast **CLI** and a highly polished **Interactive Visual Editor (GUI)** powered by Tauri, React, and Tailwind v4.

---

## 🌟 Key Features

- **Interactive Visual Editor**: Launch the sleek, Tauri-based GUI dashboard to visually design your system. Drag-and-drop node dependencies, configure functions on the fly, and auto-sync changes directly to your JSON spec.
- **Topological Incremental Builds**: A robust DAG-based incremental build system using intelligent content hashing. Only rebuilds what changed!
- **Declarative Spec**: Single source of truth (`mcp.spec.json`) to define your entire service mesh, plugins, tools, and agents.
- **Modern Minimalist Aesthetics**: Professional-grade developer dashboard powered by React Flow, Vite, and Tailwind v4.
- **Cross-Platform**: Natively supports Windows, macOS, and Linux.

---

## 💻 Prerequisites

To run MCPC (both the CLI and the GUI), ensure you have the following installed on your system:

1. **Rust & Cargo**: [Install via rustup](https://rustup.rs/) (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
2. **Node.js & npm**: [Install Node.js](https://nodejs.org/) (v18+ recommended).

### 🪟 Windows Specific Requirements
If you are running on Windows, Tauri requires the Microsoft C++ Build Tools and WebView2:
1. **Microsoft Visual Studio C++ Build Tools**: Download the [Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/). During installation, select **"Desktop development with C++"**.
2. **WebView2**: Windows 11 comes with this pre-installed. If you are on Windows 10, download and install the [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section).

---

## 🚀 Installation & Setup

Clone the repository:

```bash
git clone https://github.com/your-org/mcpc.git
cd mcpc
```

### 📦 Automated Installation Wizards

MCPC comes with native, automated wizards to compile, package, and register the orchestrator and GUI onto your system PATH:

#### 🪟 Windows (Batch / PowerShell / Native EXE)
Choose the method that fits your environment:
- **Double-Click Batch script**: Execute [install.bat](file:///c:/Users/d/automata/mcpc/install.bat) directly.
- **PowerShell Wizard**: Run `.\install.ps1` to scan prerequisites, compile release builds, and update environment paths.
- **Native Installer EXE**: Compile and run the standalone Rust binary:
  ```bash
  cargo build --release --bin install_wizard
  .\target\release\install_wizard.exe
  ```

#### 🍎 Linux & macOS (Bash)
Execute the installer script:
```bash
chmod +x install.sh
./install.sh
```

### 1. Manual CLI Installation

Alternatively, install the CLI globally using cargo:

```bash
cargo install --path .
```

Verify the installation:
```bash
mcpc --help
```

### 2. Manual GUI Dashboard Development

To run the Visual Editor dashboard in development mode:

```bash
cd mcpc-gui
npm install
npm run tauri dev
```
*(On first run, Cargo compiles the Tauri backend. Subsequent launches are nearly instant).*

### 🛡️ Enterprise Hardening Features
MCPC automatically embeds robust zero-trust policies inside every generated build:
- **gVisor Sandboxed Runtimes**: Container workloads are configured with `runtimeClassName: gvisor` to isolate host kernel access.
- **Least-Privilege Seccomp Profiles**: Automatically generates a custom `seccomp.json` inside each module to filter system calls and deny high-privilege operations like `ptrace` and `mount` by default.
- **Dynamic NetworkPolicies**: Automatically creates Kubernetes `NetworkPolicy` ingress/egress rules computed directly from the visual edge graph.
- **Bearer Token Auth & Reverse Proxying**: Gateway endpoints terminate TLS and perform cryptographic JWT signature and audience validation, acting as a secure reverse proxy router.
- **OpenTelemetry distributed tracing**: Decoupled trace logging exporting directly to OTLP collector endpoints.

---

## 📖 Usage Guide

### The Visual Editor (GUI)
When you launch the GUI (`npm run tauri dev`), you are greeted with a fully interactive Visual Graph Editor:
- **Draw Pathways**: Click and drag from a node's bottom handle to another node's top handle to wire up dependencies. This automatically updates your `mcp.spec.json`!
- **Remove Connections**: Click on any edge (connection line) and press `Backspace` to delete the dependency.
- **Configure Nodes**: Click on any module node to open the Editor Sidebar. From here, you can change the module's `type` (e.g., plugin, agent, tool) and add comma-separated `features`.
- **Navigation**: Use the Layout toggle (Horizontal/Vertical) and the Minimap to navigate complex distributed architectures.

### The CLI
If you prefer the terminal, the MCPC compiler is fully equipped to handle your workspace natively:

- **Build the Workspace**:
  ```bash
  mcpc build
  ```
  Parses your `mcp.spec.json`, resolves the dependency graph, and incrementally generates/compiles the required target folders in `automata-mcp/`.

- **Validate Spec**:
  ```bash
  mcpc validate
  ```
  Strictly checks your `mcp.spec.json` for circular dependencies, missing fields, and structural integrity.

- **Run Workspace**:
  ```bash
  mcpc run
  ```
  Bootstraps the execution environment for your generated platform.

- **Clean**:
  ```bash
  mcpc clean
  ```
  Removes the generated `automata-mcp` workspace.

- **Audit Compliance**:
  ```bash
  mcpc audit
  ```
  Generates a comprehensive JSON-formatted compliance report summarizing container hardening signatures, NetworkPolicies, and RuntimeClass mappings.

---

## 📄 The `mcp.spec.json` Format

MCPC relies on a single declarative JSON file. Here is an example of what it looks like:

```json
{
  "project": "my_platform",
  "modules": [
    {
      "name": "core_engine",
      "type": "plugin",
      "features": ["logging", "auth"],
      "dependencies": []
    },
    {
      "name": "http_api",
      "type": "agent",
      "features": ["routing"],
      "dependencies": ["core_engine"]
    }
  ]
}
```
*Note: Any modifications made in the Visual GUI Editor are automatically saved directly back to this file.*

---

Built with ❤️ for the automata ecosystem.