#!/bin/bash
# Deploy script for too.foo projects
# Usage: ./scripts/deploy.sh [project] [--publish]
#
# Projects: all, welcome, helios, chladni, blog, autocrate, portfolio
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

# Project configurations - format: "path:domain:pages-project"
# pages-project must match the names used in CI (.github/workflows/deploy.yml)
declare -A PROJECTS=(
    # L1 Bubbles
    ["welcome"]="WELCOME:too.foo:too-foo"
    ["helios"]="HELIOS:helios.too.foo:helios-too-foo"
    ["blog"]="BLOG:blog.too.foo:blog-too-foo"

    # Simulations
    ["chladni"]="SIMULATIONS/CHLADNI:chladni.too.foo:chladni-too-foo"

    # Tools
    ["sensors"]="TOOLS/SENSORS:sensors.too.foo:sensors-too-foo"
    ["autocrate"]="TOOLS/AUTOCRATE:autocrate.too.foo:autocrate-too-foo"
    ["crm"]="TOOLS/CRM:crm.too.foo:crm-too-foo"
    ["pll"]="TOOLS/PLL:pll.too.foo:pll-too-foo"
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
    local pages_project=$4

    log "Publishing $name to $domain (project: $pages_project)..."

    cd "$ROOT_DIR/$dir"

    if [[ ! -d "dist" ]]; then
        error "No dist folder. Run build first."
    fi

    # Cloudflare Pages deploy via wrangler
    # Requires: npm install -g wrangler && wrangler login
    if command -v wrangler &> /dev/null; then
        wrangler pages deploy dist --project-name="${pages_project}" --branch=main
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
        IFS=':' read -r dir domain pages_project <<< "${PROJECTS[$key]}"
        build_project "$key" "$dir" "$domain"
    done

    success "All projects built!"
}

publish_all() {
    log "Publishing all projects..."

    for key in "${!PROJECTS[@]}"; do
        IFS=':' read -r dir domain pages_project <<< "${PROJECTS[$key]}"
        publish_project "$key" "$dir" "$domain" "$pages_project"
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
    IFS=':' read -r dir domain pages_project <<< "${PROJECTS[$PROJECT]}"
    build_project "$PROJECT" "$dir" "$domain"
    if $PUBLISH; then
        publish_project "$PROJECT" "$dir" "$domain" "$pages_project"
    fi
else
    echo "Usage: $0 [project] [--publish]"
    echo ""
    echo "Projects:"
    echo "  all        - Build/deploy all projects"
    echo "  welcome    - Landing page      -> WELCOME (too.foo)"
    echo "  helios     - Solar system      -> HELIOS (helios.too.foo)"
    echo "  chladni    - Wave patterns     -> SIMULATIONS/CHLADNI (chladni.too.foo)"
    echo "  blog       - Blog engine       -> BLOG (blog.too.foo)"
    echo "  sensors    - Sensor test       -> TOOLS/SENSORS (sensors.too.foo)"
    echo "  autocrate  - Crate generator   -> TOOLS/AUTOCRATE (autocrate.too.foo)"
    echo "  crm        - CRM (coming soon) -> TOOLS/CRM (crm.too.foo)"
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
