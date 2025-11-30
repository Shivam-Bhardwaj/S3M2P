#!/bin/bash
# Install simple development aliases
# Run once: ./SCRIPTS/install-simple-aliases.sh

BASHRC=~/.bashrc

echo "Installing simple dev aliases..."

# Remove old zellij aliases if they exist
sed -i '/# Zellij aliases/,/^$/d' "$BASHRC" 2>/dev/null

# Add simple aliases
cat >> "$BASHRC" << 'EOF'

# too.foo Development
alias mon='btop'                              # System monitor
alias work='~/S3M2P/SCRIPTS/work-on-issue.sh' # Work on GitHub issue

# Quick shortcuts
alias s3m2p='cd ~/S3M2P'
alias preview='git push -u origin preview/$(git branch --show-current)'
EOF

echo "✓ Simple aliases installed"
echo ""
echo "Reload: source ~/.bashrc"
echo ""
echo "Usage:"
echo "  work 42        # Work on issue #42 (creates worktree)"
echo "  run toofoo     # Start too.foo server"
echo "  mon            # System monitor (btop)"
echo "  s3m2p          # cd to S3M2P"
echo "  preview        # Push current branch to preview"
