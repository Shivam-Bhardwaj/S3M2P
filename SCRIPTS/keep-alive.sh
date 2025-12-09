#!/usr/bin/env bash
# S3M2P - Keep Alive Monitor
# Ensures dev servers stay running. Run this in a background terminal or tmux session.

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVE_SCRIPT="$REPO_ROOT/SCRIPTS/serve-all.sh"

echo "Starting Keep-Alive Monitor..."
echo "Press Ctrl+C to stop."

while true; do
    # Check if any trunk processes are running
    if ! pgrep -x "trunk" > /dev/null; then
        echo "[$(date)] No trunk processes found. Restarting all servers..."
        "$SERVE_SCRIPT"
    else
        # Optional: Check specific ports if needed, but simple process check is usually enough
        # for a dev environment.
        :
    fi
    
    # Check every 60 seconds
    sleep 60
done