#!/bin/bash
# S3M2P Unified Setup Script
# Installs dependencies and configures development environment

set -e

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
source "$SCRIPT_DIR/config.sh"

echo "🛠️  S3M2P Dev Environment Setup"
echo "=============================="

# 1. Rust & Toolchain
echo "🦀 Checking Rust toolchain..."
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo "Rust is installed."
fi

rustup target add wasm32-unknown-unknown

# 2. Trunk (Wasm Bundler)
echo "📦 Checking Trunk..."
if ! command -v trunk &> /dev/null; then
    echo "Installing Trunk..."
    cargo install --locked trunk
else
    echo "Trunk is installed."
fi

# 3. Node.js (for deploying)
echo "🌐 Checking Node.js (needed for deployment)..."
if ! command -v node &> /dev/null; then
    echo "⚠️  Node.js not found. Please install it manually for 'dev deploy' to work."
    echo "   See: https://nodejs.org/"
fi

# 4. Git Hooks
echo "🪝  Setting up Git Hooks..."
git config core.hooksPath .githooks
echo "Git hooks configured."

# 5. Local DNS (Optional)
echo ""
echo "🌍 Setup Local DNS (too.foo -> localhost)?"
echo "   This requires sudo privileges."
read -p "   Run detailed hostname setup? [y/N] " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -f "$SCRIPT_DIR/setup-hosts-local.sh" ]; then
        "$SCRIPT_DIR/setup-hosts-local.sh"
    else
        echo "setup-hosts-local.sh not found, skipping."
    fi
fi

echo ""
echo "✅ Setup Complete!"
echo "Run './dev' to start the dashboard."
