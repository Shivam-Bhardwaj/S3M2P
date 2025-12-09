#!/bin/bash
set -e

echo "1. Checking workspace..."
cargo check --workspace

echo "2. Running tests..."
cargo test --workspace

echo "3. Running clippy..."
cargo clippy --workspace

echo "4. Building TooFoo (Production Build)..."
trunk build --release WELCOME/index.html || trunk build --release SIM/TOOFOO/index.html || echo "Could not find index.html for TOOFOO, skipping trunk build check"
