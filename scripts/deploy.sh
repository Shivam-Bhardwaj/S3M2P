#!/bin/bash
# Deploy script for too.foo projects
# Usage: ./scripts/deploy.sh [project] [--publish]
#
# Projects: all, toofoo, helios, chladni, blog, autocrate, portfolio
# --publish: Actually deploy to Cloudflare (otherwise just builds)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Project configurations
declare -A PROJECTS=(
    ["toofoo"]="too.foo:too.foo"
    ["helios"]="helios:helios.too.foo"
    ["chladni"]="chladni:chladni.too.foo"
    ["blog"]="blog:blog.too.foo"
    ["autocrate"]="autocrate:autocrate.too.foo"
    ["portfolio"]="portfolio:portfolio.too.foo"
)

# Parse arguments
PROJECT="${1:-all}"
PUBLISH=false
if [[ "$2" == "--publish" ]] || [[ "$1" == "--publish" ]]; then
    PUBLISH=true
fi

log() {
    echo -e "${CYAN}[deploy]${NC} $1"
}

success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[!]${NC} $1"
}

error() {
    echo -e "${RED}[✗]${NC} $1"
    exit 1
}

build_project() {
    local name=$1
    local dir=$2
    local domain=$3

    log "Building $name..."

    cd "$ROOT_DIR/$dir"

    if [[ ! -f "index.html" ]]; then
        error "No index.html found in $dir"
    fi

    # Build with trunk
    trunk build --release

    if [[ -d "dist" ]]; then
        success "Built $name -> $dir/dist/"
    else
        error "Build failed for $name"
    fi

    cd "$ROOT_DIR"
}

publish_project() {
    local name=$1
    local dir=$2
    local domain=$3

    log "Publishing $name to $domain..."

    cd "$ROOT_DIR/$dir"

    if [[ ! -d "dist" ]]; then
        error "No dist folder. Run build first."
    fi

    # Cloudflare Pages deploy via wrangler
    # Requires: npm install -g wrangler && wrangler login
    if command -v wrangler &> /dev/null; then
        wrangler pages deploy dist --project-name="${name}" --branch=main
        success "Published $name to https://$domain"
    else
        warn "wrangler not installed. Install with: npm install -g wrangler"
        warn "Then run: wrangler login"
        warn "Manual deploy: upload $dir/dist to Cloudflare Pages dashboard"
    fi

    cd "$ROOT_DIR"
}

build_all() {
    log "Building all WASM projects..."

    for key in "${!PROJECTS[@]}"; do
        IFS=':' read -r dir domain <<< "${PROJECTS[$key]}"
        build_project "$key" "$dir" "$domain"
    done

    success "All projects built!"
}

publish_all() {
    log "Publishing all projects..."

    for key in "${!PROJECTS[@]}"; do
        IFS=':' read -r dir domain <<< "${PROJECTS[$key]}"
        publish_project "$key" "$dir" "$domain"
    done

    success "All projects published!"
}

# Main
echo ""
echo -e "${CYAN}═══════════════════════════════════════${NC}"
echo -e "${CYAN}       too.foo Deployment Script       ${NC}"
echo -e "${CYAN}═══════════════════════════════════════${NC}"
echo ""

if [[ "$PROJECT" == "all" ]]; then
    build_all
    if $PUBLISH; then
        publish_all
    fi
elif [[ -v "PROJECTS[$PROJECT]" ]]; then
    IFS=':' read -r dir domain <<< "${PROJECTS[$PROJECT]}"
    build_project "$PROJECT" "$dir" "$domain"
    if $PUBLISH; then
        publish_project "$PROJECT" "$dir" "$domain"
    fi
else
    echo "Usage: $0 [project] [--publish]"
    echo ""
    echo "Projects:"
    echo "  all        - Build/deploy all projects"
    echo "  toofoo     - Main landing page (too.foo)"
    echo "  helios     - Solar system simulation (helios.too.foo)"
    echo "  chladni    - Wave patterns (chladni.too.foo)"
    echo "  blog       - Blog engine (blog.too.foo)"
    echo "  autocrate  - Crate generator (autocrate.too.foo)"
    echo "  portfolio  - Interactive demos (portfolio.too.foo)"
    echo ""
    echo "Options:"
    echo "  --publish  - Deploy to Cloudflare Pages after building"
    echo ""
    echo "Examples:"
    echo "  $0 all              # Build all projects"
    echo "  $0 blog --publish   # Build and deploy blog"
    echo "  $0 all --publish    # Build and deploy everything"
    exit 1
fi

echo ""
log "Done!"
