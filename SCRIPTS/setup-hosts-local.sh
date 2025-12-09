#!/usr/bin/env bash
# S3M2P - Setup Hosts Aliases (Local Dev)
# Adds entries to /etc/hosts for easy access to dev servers using *.local.too.foo
# Use this if NetworkManager/dnsmasq setup fails.
# Requires sudo.

set -e

# Define aliases
declare -A ALIASES=(
    ["welcome.local.too.foo"]="127.0.0.1"
    ["helios.local.too.foo"]="127.0.0.1"
    ["chladni.local.too.foo"]="127.0.0.1"
    ["sensors.local.too.foo"]="127.0.0.1"
    ["autocrate.local.too.foo"]="127.0.0.1"
    ["blog.local.too.foo"]="127.0.0.1"
    ["learn.local.too.foo"]="127.0.0.1"
    ["arch.local.too.foo"]="127.0.0.1"
    ["pll.local.too.foo"]="127.0.0.1"
    ["power.local.too.foo"]="127.0.0.1"
    ["ai.local.too.foo"]="127.0.0.1"
    ["ubuntu.local.too.foo"]="127.0.0.1"
    ["opencv.local.too.foo"]="127.0.0.1"
    ["arduino.local.too.foo"]="127.0.0.1"
    ["esp32.local.too.foo"]="127.0.0.1"
    ["swarm.local.too.foo"]="127.0.0.1"
    ["slam.local.too.foo"]="127.0.0.1"
    ["404.local.too.foo"]="127.0.0.1"
)

HOSTS_FILE="/etc/hosts"
BACKUP_FILE="/etc/hosts.bak.local.$(date +%s)"

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
echo "  http://welcome.local.too.foo"
echo "  http://helios.local.too.foo"
echo "  etc."
echo ""
echo "Make sure Caddy is running: caddy run"