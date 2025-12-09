#!/usr/bin/env bash
# S3M2P - Setup Hosts Aliases
# Adds entries to /etc/hosts for easy access to dev servers.
# Requires sudo.

set -e

# Define aliases
declare -A ALIASES=(
    ["welcome.too.foo"]="127.0.0.1"
    ["helios.too.foo"]="127.0.0.1"
    ["chladni.too.foo"]="127.0.0.1"
    ["sensors.too.foo"]="127.0.0.1"
    ["autocrate.too.foo"]="127.0.0.1"
    ["blog.too.foo"]="127.0.0.1"
    ["learn.too.foo"]="127.0.0.1"
    ["arch.too.foo"]="127.0.0.1"
    ["pll.too.foo"]="127.0.0.1"
    ["power.too.foo"]="127.0.0.1"
    ["ai.too.foo"]="127.0.0.1"
    ["ubuntu.too.foo"]="127.0.0.1"
    ["opencv.too.foo"]="127.0.0.1"
    ["arduino.too.foo"]="127.0.0.1"
    ["esp32.too.foo"]="127.0.0.1"
    ["swarm.too.foo"]="127.0.0.1"
    ["slam.too.foo"]="127.0.0.1"
    ["404.too.foo"]="127.0.0.1"
)

HOSTS_FILE="/etc/hosts"
BACKUP_FILE="/etc/hosts.bak.$(date +%s)"

echo "This script will add aliases to $HOSTS_FILE."
echo "You will need sudo privileges."
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "Please run with sudo: sudo $0"
   exit 1
fi

# Backup hosts file
echo "Backing up $HOSTS_FILE to $BACKUP_FILE..."
cp "$HOSTS_FILE" "$BACKUP_FILE"

echo "Adding aliases..."
for alias in "${!ALIASES[@]}"; do
    ip="${ALIASES[$alias]}"
    if grep -q "$alias" "$HOSTS_FILE"; then
        echo "  - $alias already exists"
    else
        echo "$ip $alias" >> "$HOSTS_FILE"
        echo "  + Added $alias"
    fi
done

echo ""
echo "Done! You can now access projects via:"
echo "  http://welcome.too.foo:8080"
echo "  http://helios.too.foo:8081"
echo "  etc."
echo ""
echo "Note: You still need to include the port number unless you set up a reverse proxy (e.g., nginx/caddy)."