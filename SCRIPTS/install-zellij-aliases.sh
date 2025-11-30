#!/bin/bash
# Install zellij aliases
# Run once: ./SCRIPTS/install-zellij-aliases.sh

BASHRC=~/.bashrc

echo "Installing zellij aliases..."

# Add to .bashrc if not already there
if ! grep -q "# Zellij aliases" "$BASHRC"; then
    cat >> "$BASHRC" << 'EOF'

# Zellij aliases
alias z='~/S3M2P/SCRIPTS/zellij-session.sh'
alias Z='~/S3M2P/SCRIPTS/zellij-session.sh'
alias zls='zellij list-sessions'
alias za='zellij attach'
alias zk='zellij kill-session'

# Quick session starters
alias dev='~/S3M2P/SCRIPTS/zellij-session.sh toofoo'
alias DEV='~/S3M2P/SCRIPTS/zellij-session.sh full'
EOF
    echo "✓ Added zellij aliases to ~/.bashrc"
else
    echo "✓ Zellij aliases already installed"
fi

echo ""
echo "Installation complete!"
echo ""
echo "Reload with: source ~/.bashrc"
echo ""
echo "Then use:"
echo "  dev          # Start too.foo dev session"
echo "  DEV          # Start full stack session"
echo "  z toofoo     # Start too.foo session"
echo "  z full       # Start full stack"
echo "  zls          # List sessions"
echo "  za           # Attach to session"
echo "  zk <name>    # Kill session"
