#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# FILE: deploy_changed.sh | SCRIPTS/deploy_changed.sh
# PURPOSE: Determine which Cloudflare Pages projects to deploy based on git diff
# MODIFIED: 2025-12-11
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/config.sh"

base_sha="${1:-}"
head_sha="${2:-}"

usage() {
  cat <<'EOF'
Usage: ./SCRIPTS/deploy_changed.sh <BASE_SHA> <HEAD_SHA>

Outputs:
  Newline-delimited list of deployable project keys (from PAGES_PROJECTS).

Rules:
  - If core changed (DNA/**, TOOLS/CORE/**, Cargo.toml, Cargo.lock): deploy all non-static pages projects.
  - Else: deploy only projects whose PROJECT_DIRS folder changed.
EOF
}

if [[ -z "$base_sha" || -z "$head_sha" ]]; then
  usage
  exit 2
fi

# Gather changed files for this push range.
mapfile -t files < <(git diff --name-only "${base_sha}..${head_sha}" || true)

if [[ ${#files[@]} -eq 0 ]]; then
  exit 0
fi

core_changed=false
for f in "${files[@]}"; do
  if [[ "$f" == DNA/* ]] || \
     [[ "$f" == TOOLS/CORE/* ]] || \
     [[ "$f" == Cargo.toml ]] || \
     [[ "$f" == Cargo.lock ]]; then
    core_changed=true
    break
  fi
done

declare -A selected=()

if $core_changed; then
  # Deploy all non-static Pages projects (safe default).
  for key in "${!PAGES_PROJECTS[@]}"; do
    if [[ -v "STATIC_PROJECTS[$key]" ]]; then
      continue
    fi
    selected["$key"]=1
  done
else
  # Deploy only projects whose directory prefix changed.
  for key in "${!PAGES_PROJECTS[@]}"; do
    dir="${PROJECT_DIRS[$key]:-}"
    if [[ -z "$dir" ]]; then
      continue
    fi
    for f in "${files[@]}"; do
      if [[ "$f" == "$dir/"* ]] || [[ "$f" == "$dir" ]]; then
        selected["$key"]=1
        break
      fi
    done
  done
fi

if [[ ${#selected[@]} -eq 0 ]]; then
  exit 0
fi

# Stable output order.
for key in "${!selected[@]}"; do
  echo "$key"
done | sort


