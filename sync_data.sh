#!/bin/bash
set -e

# Configuration
REMOTE_USER="root"
REMOTE_HOST="144.126.145.3"
REMOTE_PATH="/var/data/helios"
LOCAL_DATA_DIR="./data"

# 1. Run Simulation (Optional)
if [ "$1" == "--generate" ]; then
    echo "Running Simulation..."
    cd simulation-cli
    cargo run --release -- generate --count 500000 --output ../data
    cd ..
fi

# 2. Sync Data
echo "Syncing data to $REMOTE_HOST..."
# Using rsync for differential transfer
# -a: archive mode
# -v: verbose
# -z: compress
# --delete: remove files on remote that don't exist locally
rsync -avz --progress $LOCAL_DATA_DIR/ $REMOTE_USER@$REMOTE_HOST:$REMOTE_PATH/

echo "Data Sync Complete."

