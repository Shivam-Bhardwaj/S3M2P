#!/bin/bash
# Super simple dev launcher
# Just starts zellij with helpful message

echo "========================================="
echo "  S3M2P Development Session"
echo "========================================="
echo ""
echo "Starting zellij..."
echo ""
echo "Quick commands:"
echo "  run toofoo    - Start too.foo server"
echo "  run website   - Start personal site"
echo "  run list      - See all projects"
echo ""
echo "Zellij keys:"
echo "  Alt+t         - New tab"
echo "  Alt+n         - New pane"
echo "  Alt+d         - Detach (keeps running)"
echo "  Alt+1/2/3     - Switch tabs"
echo ""
echo "Press Enter to start..."
read

exec zellij attach s3m2p --create
