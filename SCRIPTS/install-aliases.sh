#!/bin/bash
# Install run script aliases and completion
# Run this once: ./SCRIPTS/install-aliases.sh

BASHRC=~/.bashrc
SCRIPT_DIR="$HOME/S3M2P/SCRIPTS"

echo "Installing run script aliases..."

# Add to .bashrc if not already there
if ! grep -q "# S3M2P run script" "$BASHRC"; then
    cat >> "$BASHRC" << 'EOF'

# S3M2P run script
alias run='~/S3M2P/SCRIPTS/run.sh'
alias RUN='~/S3M2P/SCRIPTS/run.sh'  # Case insensitive
alias work='~/S3M2P/SCRIPTS/work-on-issue.sh'
alias WORK='~/S3M2P/SCRIPTS/work-on-issue.sh'
alias issue='gh issue create'
alias web='~/S3M2P/SCRIPTS/web.sh'
alias WEB='~/S3M2P/SCRIPTS/web.sh'
source ~/S3M2P/SCRIPTS/run-completion.bash
EOF
    echo "✓ Added aliases to ~/.bashrc"
else
    echo "✓ Aliases already installed"
fi

# Make scripts executable
chmod +x "$SCRIPT_DIR/run.sh"
chmod +x "$SCRIPT_DIR/run-completion.bash"

echo ""
echo "Installation complete!"
echo ""
echo "Run one of these commands to activate:"
echo "  source ~/.bashrc"
echo "  OR restart your terminal"
echo ""
echo "Then use:"
echo "  run toofoo"
echo "  run helios"
echo "  run mcad"
echo "  run list      # See all projects"
