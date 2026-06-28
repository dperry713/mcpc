#!/bin/bash
# MCPC Enterprise Multi-Platform Installation Wizard (Linux/macOS Bash)
# Automates compiling and registering the production orchestrator

set -e

# Visual Header
echo -e "\033[0;36m==========================================================\033[0m"
echo -e "\033[0;36m         🔒 MCPC ENTERPRISE INSTALLATION WIZARD           \033[0m"
echo -e "\033[0;36m==========================================================\033[0m"
echo -e "\033[0;90mTarget: Linux/macOS Production Release\033[0m"
echo ""

# 1. Prerequisite Verification
verify_tool() {
    local name=$1
    local cmd=$2
    echo -n "Checking for $name... "
    if command -v $cmd >/dev/null 2>&1; then
        echo -e "\033[0;32mFound.\033[0m"
        return 0
    else
        echo -e "\033[0;31mNOT FOUND.\033[0m Please install $name."
        return 1
    fi
}

prereqs=0
verify_tool "Rust/Cargo" "cargo" || prereqs=1
verify_tool "Node.js" "node" || prereqs=1
verify_tool "npm" "npm" || prereqs=1
verify_tool "Docker" "docker" || prereqs=1

if [ $prereqs -ne 0 ]; then
    echo -e "\n\033[0;31m[Error] Prerequisites check failed. Please install missing tools and try again.\033[0m"
    exit 1
fi

# 2. Compile Rust Backend
echo -e "\n\033[0;36m[1/4] Compiling Rust backend in Release Mode...\033[0m"
cargo build --release
echo -e "\033[0;32mRust backend successfully compiled.\033[0m"

# 3. Compile GUI wrapper
echo -e "\n\033[0;36m[2/4] Initializing React/Tauri GUI...\033[0m"
cd mcpc-gui
echo "Installing GUI dependencies..."
npm install
echo "Building GUI client..."
npm run build
cd ..
echo -e "\033[0;32mGUI successfully compiled.\033[0m"

# 4. Install Binary & Register Path
echo -e "\n\033[0;36m[3/4] Installing binary to user home directory...\033[0m"
INSTALL_DIR="$HOME/.mcpc/bin"
mkdir -p "$INSTALL_DIR"

cp target/release/mcpc "$INSTALL_DIR/mcpc"
chmod +x "$INSTALL_DIR/mcpc"
echo -e "\033[0;32mBinary copied to $INSTALL_DIR/mcpc\033[0m"

# 5. Add to PATH env
echo -e "\n\033[0;36m[4/4] Registering binary path in shell profile...\033[0m"
SHELL_PROFILE=""
if [[ "$SHELL" == *"zsh"* ]]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [[ "$SHELL" == *"bash"* ]]; then
    SHELL_PROFILE="$HOME/.bashrc"
else
    SHELL_PROFILE="$HOME/.profile"
fi

if [ -f "$SHELL_PROFILE" ]; then
    if ! grep -q "$INSTALL_DIR" "$SHELL_PROFILE"; then
        echo "export PATH=\$PATH:$INSTALL_DIR" >> "$SHELL_PROFILE"
        echo -e "\033[0;32mPath successfully registered in $SHELL_PROFILE. Please run 'source $SHELL_PROFILE' or restart your terminal.\033[0m"
    else
        echo -e "\033[0;90mPath is already registered.\033[0m"
    fi
else
    echo -e "\033[0;33mShell profile not found. Please manually append $INSTALL_DIR to your PATH.\033[0m"
fi

# 6. Bootstrap Directories
mkdir -p "$HOME/.mcpc/plugins"
mkdir -p "$HOME/.mcpc/cache"

echo -e "\n\033[0;36m==========================================================\033[0m"
echo -e "\033[0;32m🔒 MCPC INSTALLATION COMPLETED SUCCESSFULLY!\033[0m"
echo -e "\033[0;32mYou can now run 'mcpc --help' to get started.\033[0m"
echo -e "\033[0;36m==========================================================\033[0m"
