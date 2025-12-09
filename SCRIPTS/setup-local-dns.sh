#!/usr/bin/env bash
# S3M2P - Setup Local DNS (dnsmasq)
# Configures *.local.too.foo to point to 127.0.0.1 using NetworkManager's dnsmasq.
# This avoids conflict with the actual production domain (too.foo).

set -e

if [[ $EUID -ne 0 ]]; then
   echo "Please run with sudo: sudo $0"
   exit 1
fi

echo "Setting up local DNS for *.local.too.foo..."

# Check if NetworkManager is used
if command -v NetworkManager >/dev/null; then
    echo "Detected NetworkManager."
    
    # Create dnsmasq config for NetworkManager
    echo "address=/local.too.foo/127.0.0.1" > /etc/NetworkManager/dnsmasq.d/local.too.foo.conf
    
    echo "Restarting NetworkManager..."
    systemctl restart NetworkManager
    
    echo "Done! *.local.too.foo should now resolve to 127.0.0.1"
else
    echo "NetworkManager not found. Please use SCRIPTS/setup-hosts.sh instead."
fi