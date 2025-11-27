#!/bin/bash
set -e

SERVER_IP="144.126.145.3"
USER="root"
PASS="asasasas"

# Check if sshpass is installed
if ! command -v sshpass &> /dev/null; then
    echo "sshpass could not be found, installing..."
    sudo apt-get install -y sshpass
fi

echo "Deploying to $SERVER_IP..."

# 1. Copy setup script
echo "Copying setup script..."
sshpass -p "$PASS" scp -o StrictHostKeyChecking=no storage-server/setup_server.sh $USER@$SERVER_IP:/root/

# 2. Copy Code
echo "Copying code (storage-server + core)..."
# Create directory first
sshpass -p "$PASS" ssh -o StrictHostKeyChecking=no $USER@$SERVER_IP "mkdir -p /root/storage-server /root/antimony-core"

# Copy files - excluding target directories to save time/bandwidth
echo "Copying storage-server..."
sshpass -p "$PASS" rsync -av --exclude 'target' --exclude '.git' storage-server/ $USER@$SERVER_IP:/root/storage-server/

echo "Copying antimony-core..."
sshpass -p "$PASS" rsync -av --exclude 'target' --exclude '.git' antimony-core/ $USER@$SERVER_IP:/root/antimony-core/

# 3. Run Setup
echo "Running setup on server..."
sshpass -p "$PASS" ssh $USER@$SERVER_IP "chmod +x /root/setup_server.sh && /root/setup_server.sh"

echo "Deployment Complete."
