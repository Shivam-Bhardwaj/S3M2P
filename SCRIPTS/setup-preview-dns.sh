#!/bin/bash
# Setup Cloudflare DNS for preview and production deployments
# Creates CNAME records for:
#   - p.{project}.too.foo -> preview-{project}.pages.dev (preview)
#   - {project}.too.foo -> {project}-too-foo.pages.dev (production)
#
# Usage: ./setup-preview-dns.sh [--production] [--preview] [--all]
#
# Options:
#   --production  Set up production DNS records only
#   --preview     Set up preview DNS records only
#   --all         Set up both (default)
#
# Prerequisites:
# - CLOUDFLARE_ZONE_ID environment variable set (for too.foo zone)
# - CLOUDFLARE_API_TOKEN environment variable set

set -e

ZONE_NAME="too.foo"

# Parse arguments
SETUP_PREVIEW=true
SETUP_PRODUCTION=true

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --production) SETUP_PREVIEW=false; SETUP_PRODUCTION=true ;;
        --preview) SETUP_PREVIEW=true; SETUP_PRODUCTION=false ;;
        --all) SETUP_PREVIEW=true; SETUP_PRODUCTION=true ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
    shift
done

# Projects and their Cloudflare Pages project names (for production)
declare -A PROJECT_PAGES=(
    ["helios"]="helios-too-foo"
    ["autocrate"]="autocrate-too-foo"
    ["chladni"]="chladni-too-foo"
    ["portfolio"]="portfolio-too-foo"
    ["blog"]="blog-too-foo"
    ["mcad"]="mcad-too-foo"
    ["ecad"]="ecad-too-foo"
)

# Projects that need preview DNS (same list)
PROJECTS=(
    "helios"
    "autocrate"
    "chladni"
    "portfolio"
    "blog"
    "mcad"
    "ecad"
)

echo "Setting up DNS for $ZONE_NAME"
echo "========================================"
echo "Preview: $SETUP_PREVIEW | Production: $SETUP_PRODUCTION"

# Check for required tools
if ! command -v curl &> /dev/null; then
    echo "Error: curl not found."
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "Error: jq not found. Install with: sudo apt install jq"
    exit 1
fi

# Check environment variables
if [ -z "$CLOUDFLARE_ZONE_ID" ] || [ -z "$CLOUDFLARE_API_TOKEN" ]; then
    echo ""
    echo "Missing required environment variables. To set up DNS:"
    echo ""
    echo "1. Go to Cloudflare Dashboard > too.foo > Overview"
    echo "2. Copy the Zone ID from the right sidebar"
    echo "3. Go to My Profile > API Tokens > Create Token"
    echo "   (Use 'Edit zone DNS' template for the too.foo zone)"
    echo ""
    echo "4. Run:"
    echo "   export CLOUDFLARE_ZONE_ID=<your-zone-id>"
    echo "   export CLOUDFLARE_API_TOKEN=<your-api-token>"
    echo ""
    echo "Then run this script again."
    echo ""
    echo "Alternatively, manually add these CNAME records in Cloudflare Dashboard:"
    echo ""
    if [ "$SETUP_PREVIEW" = true ]; then
        echo "Preview DNS:"
        for project in "${PROJECTS[@]}"; do
            echo "  p.$project -> preview-$project.pages.dev"
        done
    fi
    if [ "$SETUP_PRODUCTION" = true ]; then
        echo ""
        echo "Production DNS:"
        for project in "${PROJECTS[@]}"; do
            pages_project="${PROJECT_PAGES[$project]}"
            echo "  $project -> $pages_project.pages.dev"
        done
    fi
    exit 0
fi

# Helper function to create/update DNS record
create_dns_record() {
    local subdomain=$1
    local target=$2

    echo "Creating: $subdomain.$ZONE_NAME -> $target"

    # Check if record exists
    EXISTING=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones/$CLOUDFLARE_ZONE_ID/dns_records?name=$subdomain.$ZONE_NAME" \
        -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
        -H "Content-Type: application/json" | jq -r '.result[0].id // empty')

    if [ -n "$EXISTING" ]; then
        echo "  Record exists, updating..."
        curl -s -X PUT "https://api.cloudflare.com/client/v4/zones/$CLOUDFLARE_ZONE_ID/dns_records/$EXISTING" \
            -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
            -H "Content-Type: application/json" \
            --data "{\"type\":\"CNAME\",\"name\":\"$subdomain\",\"content\":\"$target\",\"proxied\":true}" | jq -r '.success'
    else
        echo "  Creating new record..."
        curl -s -X POST "https://api.cloudflare.com/client/v4/zones/$CLOUDFLARE_ZONE_ID/dns_records" \
            -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
            -H "Content-Type: application/json" \
            --data "{\"type\":\"CNAME\",\"name\":\"$subdomain\",\"content\":\"$target\",\"proxied\":true}" | jq -r '.success'
    fi
}

# Create preview DNS records
if [ "$SETUP_PREVIEW" = true ]; then
    echo ""
    echo "Creating PREVIEW DNS records..."
    echo ""

    for project in "${PROJECTS[@]}"; do
        create_dns_record "p.$project" "preview-$project.pages.dev"
    done
fi

# Create production DNS records
if [ "$SETUP_PRODUCTION" = true ]; then
    echo ""
    echo "Creating PRODUCTION DNS records..."
    echo ""

    for project in "${PROJECTS[@]}"; do
        pages_project="${PROJECT_PAGES[$project]}"
        create_dns_record "$project" "$pages_project.pages.dev"
    done
fi

echo ""
echo "DNS setup complete!"
echo ""

if [ "$SETUP_PREVIEW" = true ]; then
    echo "Preview URLs:"
    for project in "${PROJECTS[@]}"; do
        echo "  https://p.$project.too.foo"
    done
fi

if [ "$SETUP_PRODUCTION" = true ]; then
    echo ""
    echo "Production URLs:"
    for project in "${PROJECTS[@]}"; do
        echo "  https://$project.too.foo"
    done
fi
