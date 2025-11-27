#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting Server Setup...${NC}"

# 1. Update System
echo -e "${GREEN}Updating system packages...${NC}"
apt-get update && apt-get upgrade -y

# 2. Install Dependencies
echo -e "${GREEN}Installing dependencies (curl, git, build-essential)...${NC}"
apt-get install -y curl git build-essential ufw fail2ban

# 3. Install Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${GREEN}Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# 4. Setup Firewall
echo -e "${GREEN}Configuring Firewall...${NC}"
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 3000/tcp  # API Port
ufw --force enable

# 5. Create Data Directory
echo -e "${GREEN}Creating data directory...${NC}"
mkdir -p /var/data/helios
chown -R $USER:$USER /var/data/helios

# 6. Setup Service (Systemd)
echo -e "${GREEN}Creating systemd service...${NC}"
cat <<EOF > /etc/systemd/system/helios-storage.service
[Unit]
Description=Helios Storage Server
After=network.target

[Service]
User=root
WorkingDirectory=/root/storage-server
ExecStart=/root/.cargo/bin/cargo run --release
Environment="DATA_DIR=/var/data/helios"
Restart=always

[Install]
WantedBy=multi-user.target
EOF

echo -e "${BLUE}Setup Complete!${NC}"
echo "To deploy the code:"
echo "1. Copy the 'storage-server' and 'antimony-core' folders to /root/"
echo "2. Run 'systemctl enable --now helios-storage'"

