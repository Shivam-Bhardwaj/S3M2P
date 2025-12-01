#!/bin/bash
# Sets up git hooks for local development
# Run this once after cloning the repo

set -e

echo "=== Setting up git hooks ==="

# Point git to our hooks directory
git config core.hooksPath .githooks

echo "Git hooks installed!"
echo ""
echo "Hooks enabled:"
echo "  - pre-commit: runs 'cargo fmt --check' and 'cargo clippy'"
echo "  - pre-push: runs 'cargo test --workspace'"
echo ""
echo "To bypass hooks temporarily, use --no-verify flag"
echo "To disable hooks, run: git config --unset core.hooksPath"
