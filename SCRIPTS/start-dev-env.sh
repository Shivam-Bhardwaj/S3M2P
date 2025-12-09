#!/usr/bin/env bash
# S3M2P - Start Development Environment
# Orchestrates DNS setup, Caddy, and Dev Servers.

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🚀 Starting S3M2P Development Environment..."
echo ""

# 1. Check/Setup Hosts
echo "--- [1/3] Checking Host Aliases ---"
if ! grep -q "welcome.local.too.foo" /etc/hosts; then
    echo "⚠️  Host aliases missing. Running setup-hosts-local.sh..."
    # This requires sudo, so it will prompt if needed
    sudo "$REPO_ROOT/SCRIPTS/setup-hosts-local.sh"
else
    echo "✅ Host aliases present."
fi
echo ""

# 2. Start Dev Servers
echo "--- [2/3] Starting Dev Servers ---"
"$REPO_ROOT/SCRIPTS/serve-all.sh"
echo ""

# 3. Start Caddy
echo "--- [3/3] Checking Caddy ---"
if pgrep -x "caddy" > /dev/null; then
    echo "✅ Caddy is already running."
    echo "   (If config changed, reload with: sudo caddy reload --config \"$REPO_ROOT/Caddyfile\")"
else
    echo "⚠️  Caddy is not running."
    echo "   Starting Caddy..."
    echo "   Please enter password for sudo if prompted (required for ports 80/443):"
    
    # Use 'caddy start' to run in background, or 'caddy run' to run in foreground.
    # Since serve-all.sh puts servers in background, we probably want caddy in background too 
    # so the script returns control to user.
    sudo caddy start --config "$REPO_ROOT/Caddyfile" --adapter caddyfile
    echo "✅ Caddy started."
fi

echo ""
echo "🎉 Environment Ready!"
echo "   Access projects at:"
echo "   - https://welcome.local.too.foo"
echo "   - https://helios.local.too.foo"
echo "   - https://blog.local.too.foo"
echo "   ..."
echo ""
echo "   Note: You may need to accept the self-signed certificate in your browser."