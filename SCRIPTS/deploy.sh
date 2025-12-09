#!/bin/bash
# Deploy script for too.foo projects
# Usage: ./SCRIPTS/deploy.sh [project] [--publish]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/config.sh"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Cloudflare Pages Project Names (Mapping from Config Keys)
declare -A PAGES_PROJECTS=(
    ["welcome"]="too-foo"
    ["helios"]="helios-too-foo"
    ["blog"]="blog-too-foo"
    ["chladni"]="chladni-too-foo"
    ["sensors"]="sensors-too-foo"
    ["autocrate"]="autocrate-too-foo"
    ["crm"]="crm-too-foo"
    ["pll"]="pll-too-foo"
    ["coming_soon"]="coming-soon-too-foo"
)

# Static projects that don't need trunk build
declare -A STATIC_PROJECTS=(
    ["coming_soon"]=1
    ["arch"]=1
)

# Parse arguments
PROJECT="${1:-all}"
PUBLISH=false
if [[ "$2" == "--publish" ]] || [[ "$1" == "--publish" ]]; then
    PUBLISH=true
fi

log() { echo -e "${CYAN}[deploy]${NC} $1"; }
success() { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; exit 1; }

build_project() {
    local name=$1
    local dir=${PROJECT_DIRS[$name]}
    
    if [[ -z "$dir" ]]; then
        error "Unknown project: $name"
    fi

    log "Building $name..."
    cd "$REPO_ROOT/$dir"

    if [[ ! -f "index.html" ]]; then
        error "No index.html found in $dir"
    fi

    if [[ -v "STATIC_PROJECTS[$name]" ]]; then
        log "Static project - no build needed"
        success "Ready $name -> $dir/"
        return
    fi

    trunk build --release
    if [[ -d "dist" ]]; then
        success "Built $name -> $dir/dist/"
    else
        error "Build failed for $name"
    fi
}

publish_project() {
    local name=$1
    local dir=${PROJECT_DIRS[$name]}
    local pages_project=${PAGES_PROJECTS[$name]}

    if [[ -z "$pages_project" ]]; then
        warn "No Cloudflare project mapped for '$name', skipping publish."
        return
    fi

    log "Publishing $name to Cloudflare ($pages_project)..."
    cd "$REPO_ROOT/$dir"

    local deploy_dir="dist"
    if [[ -v "STATIC_PROJECTS[$name]" ]]; then
        deploy_dir="."
    fi

    if [[ "$deploy_dir" == "dist" ]] && [[ ! -d "dist" ]]; then
        error "No dist folder. Run build first."
    fi

    if command -v wrangler &> /dev/null; then
        wrangler pages deploy "$deploy_dir" --project-name="${pages_project}" --branch=main --commit-dirty=true
        success "Published $name"
    else
        warn "wrangler not found. Install generic tools first."
    fi
}

build_all() {
    log "Building all projects..."
    for key in "${!PROJECT_DIRS[@]}"; do
        # Only build things we have mapped to Cloudflare (or known deployables)
        # Verify if it has a PAGES mapping or is in our deploy list
        if [[ -v "PAGES_PROJECTS[$key]" ]]; then
            build_project "$key"
        fi
    done
}

publish_all() {
    log "Publishing all projects..."
    for key in "${!PROJECT_DIRS[@]}"; do
        if [[ -v "PAGES_PROJECTS[$key]" ]]; then
            publish_project "$key"
        fi
    done
}

# Main Dispatch
if [[ "$PROJECT" == "all" ]]; then
    build_all
    if $PUBLISH; then publish_all; fi
elif [[ -v "PROJECT_DIRS[$PROJECT]" ]]; then
    build_project "$PROJECT"
    if $PUBLISH; then publish_project "$PROJECT"; fi
else
    echo "Usage: $0 [project] [--publish]"
    echo "Available projects: ${!PROJECT_DIRS[@]}"
    exit 1
fi
