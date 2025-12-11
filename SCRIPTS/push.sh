#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# FILE: push.sh | SCRIPTS/push.sh
# PURPOSE: Scope-aware "push to main" wrapper (runs scoped tests then pushes)
# MODIFIED: 2025-12-11
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./SCRIPTS/push.sh [options]

Options:
  --remote <name>      Git remote (default: origin)
  --branch <name>      Target branch (default: main)
  --allow-dirty        Allow dirty working tree (default: false)
  --no-test            Skip tests (emergency escape hatch)
  -h, --help           Show help

Behavior:
  - Runs: ./SCRIPTS/test.sh auto
  - Then pushes to: <remote> <branch>
EOF
}

remote="origin"
branch="main"
allow_dirty=false
run_tests=true

while [[ $# -gt 0 ]]; do
  case "$1" in
    --remote)
      remote="${2:-}"; shift 2 ;;
    --branch)
      branch="${2:-}"; shift 2 ;;
    --allow-dirty)
      allow_dirty=true; shift ;;
    --no-test)
      run_tests=false; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "Unknown arg: $1"
      echo ""
      usage
      exit 2
      ;;
  esac
done

current_branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$current_branch" != "$branch" ]]; then
  echo "[push] Refusing: current branch is '$current_branch' but target is '$branch'."
  echo "[push] Checkout '$branch' first (or pass --branch to match)."
  exit 1
fi

if ! $allow_dirty; then
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "[push] Refusing: working tree is dirty. Commit/stash first or pass --allow-dirty."
    exit 1
  fi
fi

if $run_tests; then
  echo "[push] Running scoped tests..."
  ./SCRIPTS/test.sh auto
else
  echo "[push] WARNING: --no-test specified; skipping scoped tests."
fi

echo "[push] Pushing to ${remote} ${branch}..."
git push "$remote" "$branch"

echo "[push] Done."


