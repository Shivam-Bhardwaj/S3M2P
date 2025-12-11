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
  auto       Auto-select scope based on git diff vs origin/main
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

  auto)
    echo "== Auto-scoped checks (git diff vs origin/main) =="

    # Ensure we have origin/main locally.
    git fetch -q origin main || true

    base="$(git merge-base HEAD origin/main 2>/dev/null || true)"
    if [[ -z "$base" ]]; then
      echo "[auto] Could not determine merge-base vs origin/main; falling back to 'dna' scope."
      exec "$0" dna
    fi

    mapfile -t files < <(git diff --name-only "${base}..HEAD")
    if [[ ${#files[@]} -eq 0 ]]; then
      echo "[auto] No changes detected vs origin/main; nothing to test."
      exit 0
    fi

    docs_only=true
    autocrate_scope=false
    core_scope=false

    for f in "${files[@]}"; do
      # Anything non-doc breaks docs-only.
      if [[ "$f" != *.md && "$f" != "LICENSE" ]]; then
        docs_only=false
      fi

      # Autocrate and its exporters/viewer.
      if [[ "$f" == DNA/src/autocrate/* ]] || \
         [[ "$f" == DNA/src/export/* ]] || \
         [[ "$f" == TOOLS/AUTOCRATE/* ]] || \
         [[ "$f" == TOOLS/CORE/AUTOCRATE_ENGINE/* ]] || \
         [[ "$f" == TOOLS/CORE/EXPORT_ENGINE/* ]]; then
        autocrate_scope=true
      fi

      # Core algorithm surface area.
      if [[ "$f" == DNA/* ]] || \
         [[ "$f" == TOOLS/CORE/* ]] || \
         [[ "$f" == Cargo.toml ]] || \
         [[ "$f" == Cargo.lock ]]; then
        core_scope=true
      fi
    done

    echo "[auto] Changed files:"
    for f in "${files[@]}"; do echo "  - $f"; done

    if $docs_only; then
      echo "[auto] Docs-only change; skipping tests."
      exit 0
    fi

    # Decision:
    # - If core changed outside the autocrate surface, run broader DNA checks.
    # - If only autocrate surface changed, run fast + WASM build.
    if $core_scope && ! $autocrate_scope; then
      echo "[auto] Scope: core (DNA/CORE/Cargo) changed → running broader checks"
      cargo check --workspace
      cargo test -p dna --lib
      exit 0
    fi

    if $autocrate_scope; then
      echo "[auto] Scope: autocrate → running fast checks + engine tests + trunk build"
      "$0" fast
      cargo test -p autocrate-engine -p export-engine
      (cd TOOLS/AUTOCRATE && NO_COLOR=true trunk build --release)
      exit 0
    fi

    echo "[auto] Scope: other (not mapped) → running safe default cargo check"
    cargo check --workspace
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


