#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# FILE: test.sh | SCRIPTS/test.sh
# PURPOSE: Smart test runner (fast, scoped, full) for local development
# MODIFIED: 2025-12-11
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

scope="${1:-fast}"

usage() {
  cat <<'EOF'
Usage: ./SCRIPTS/test.sh [scope]

Scopes:
  fast       Fast feedback for AutoCrate work (default)
  autocrate  Alias for fast
  dna        Run all DNA tests
  full       Full workspace tests (+ Playwright)

Examples:
  ./SCRIPTS/test.sh fast
  ./SCRIPTS/test.sh dna
  ./SCRIPTS/test.sh full
EOF
}

case "$scope" in
  -h|--help|help)
    usage
    exit 0
    ;;

  fast|autocrate)
    echo "== Fast checks (autocrate) =="
    cargo check -p dna -p autocrate-engine -p autocrate
    cargo test -p dna autocrate::
    ;;

  dna)
    echo "== DNA tests =="
    cargo test -p dna
    ;;

  full)
    echo "== Full workspace tests =="
    cargo test --workspace --exclude hw
    echo "== Playwright =="
    npx playwright test
    ;;

  *)
    echo "Unknown scope: $scope"
    echo ""
    usage
    exit 2
    ;;
esac


