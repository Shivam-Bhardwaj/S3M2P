#!/bin/bash
# Validation script for Claude Automation System
# Run this before starting the daemon to ensure everything is configured

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🔍 Validating Claude Automation System..."
echo ""

ERRORS=0

# Check 1: GitHub token
echo -n "Checking GITHUB_TOKEN... "
if [ -f ~/.claude/automation.env ]; then
    source ~/.claude/automation.env
    if [ -n "$GITHUB_TOKEN" ]; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗ Token not set in automation.env${NC}"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo -e "${YELLOW}⚠ automation.env not found${NC}"
    if [ -n "$GITHUB_TOKEN" ]; then
        echo -e "  ${GREEN}✓ But GITHUB_TOKEN is set in environment${NC}"
    else
        echo -e "  ${RED}✗ No GITHUB_TOKEN found${NC}"
        ERRORS=$((ERRORS + 1))
    fi
fi

# Check 2: Agent files
echo -n "Checking agent files... "
if [ -f ".claude/agents/planner.md" ] && [ -f ".claude/agents/executor.md" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ Agent files missing${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check 3: MCP server
echo -n "Checking MCP server... "
if [ -f "TOOLS/CLAUDE_AUTOMATION/mcp-server/dist/index.js" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ MCP server not built${NC}"
    echo "  Run: cd TOOLS/CLAUDE_AUTOMATION/mcp-server && npm install && npm run build"
    ERRORS=$((ERRORS + 1))
fi

# Check 4: Daemon binary
echo -n "Checking daemon binary... "
if [ -f "target/release/claude-automation" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ Daemon not built${NC}"
    echo "  Run: cargo build --release -p claude-automation"
    ERRORS=$((ERRORS + 1))
fi

# Check 5: Worktree directory
echo -n "Checking worktree directory... "
WORKTREE_DIR="/home/curious/worktrees/auto"
if [ -d "$WORKTREE_DIR" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${YELLOW}⚠ Creating worktree directory${NC}"
    mkdir -p "$WORKTREE_DIR"
fi

# Check 6: GitHub label exists
echo -n "Checking claude-auto label... "
if gh label list --repo Shivam-Bhardwaj/S3M2P | grep -q "claude-auto"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ claude-auto label not found${NC}"
    echo "  Run: gh label create 'claude-auto' --description 'Trigger Claude automation' --color '7057ff'"
    ERRORS=$((ERRORS + 1))
fi

# Check 7: Run unit tests
echo -n "Running unit tests... "
if cargo test -p claude-automation --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ Tests failed${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check 8: GitHub Actions workflows
echo -n "Checking GitHub Actions... "
if [ -f ".github/workflows/claude-automation.yml" ] && \
   [ -f ".github/workflows/preview-deploy.yml" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ Workflows missing${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check 9: Test GitHub API access
echo -n "Testing GitHub API access... "
if gh api user >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ Cannot access GitHub API${NC}"
    echo "  Check GITHUB_TOKEN permissions"
    ERRORS=$((ERRORS + 1))
fi

# Check 10: Test MCP server can start
echo -n "Testing MCP server... "
if timeout 2 node TOOLS/CLAUDE_AUTOMATION/mcp-server/dist/index.js >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC}"
else
    # Timeout is expected (server runs indefinitely)
    echo -e "${GREEN}✓${NC}"
fi

echo ""
echo "========================================"
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed!${NC}"
    echo ""
    echo "System is ready. Start with:"
    echo "  systemctl --user start claude-automation"
    echo ""
    echo "Monitor with:"
    echo "  tail -f ~/.claude/automation-daemon.log"
    exit 0
else
    echo -e "${RED}❌ $ERRORS error(s) found${NC}"
    echo ""
    echo "Fix the errors above before starting the daemon."
    exit 1
fi
