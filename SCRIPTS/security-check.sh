#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# FILE: security-check.sh | SCRIPTS/security-check.sh
# PURPOSE: Pre-commit hook to scan for secrets and PII
# MODIFIED: 2025-12-09
# ═══════════════════════════════════════════════════════════════════════════════
#
# Install as git hook:
#   ln -s ../../SCRIPTS/security-check.sh .git/hooks/pre-commit
#
# Or run manually:
#   ./SCRIPTS/security-check.sh

set -e

echo "🔍 Running security scan on staged files..."

# Build the security CLI if not already built
if [ ! -f "DNA/SECURITY_CLI/target/release/dna-security" ]; then
    echo "📦 Building security scanner..."
    cd DNA/SECURITY_CLI
    cargo build --release
    cd ../..
fi

# Run security check
DNA/SECURITY_CLI/target/release/dna-security check --fail-on-findings

exit_code=$?

if [ $exit_code -eq 0 ]; then
    echo "✅ Security check passed"
else
    echo ""
    echo "❌ Security check failed"
    echo "Fix the security issues above or use:"
    echo "  git commit --no-verify   (NOT RECOMMENDED)"
    exit 1
fi
