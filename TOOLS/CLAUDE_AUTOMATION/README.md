# Claude Code Automation System

Fully automated GitHub issue resolution with conversational AI agents.

## Quick Start

### 1. Start the Daemon

```bash
# Enable and start the systemd service
systemctl --user enable claude-automation
systemctl --user start claude-automation

# Check status
systemctl --user status claude-automation

# View logs
journalctl --user -u claude-automation -f
```

### 2. Create an Issue

1. Go to GitHub and create an issue
2. Use a project template (auto-adds `project:xxx` label)
3. Add label: `claude-auto`
4. Within 60 seconds, the daemon will detect and start working

### 3. Talk Naturally in Comments

Just comment like you're talking to a teammate:
- "Can you add dark mode?"
- "The button is too small"
- "Show me the code for X"
- "Ship it!"

The agents will respond within 5-60 seconds (depends on activity level).

---

## Architecture

```
GitHub Issue (you create)
    ↓
GitHub Actions (posts CLAUDE_TRIGGER)
    ↓
Local Daemon (polls every 5-60s)
    ↓
Planner (Opus) → Creates implementation plan
    ↓
Executor (Sonnet) → Implements and iterates
    ↓
Preview Branch → Auto-deploys to issue-123-project.too.foo
    ↓
You review and comment → Executor responds
    ↓
"Ship it!" → Creates PR to main
```

---

## Components

### 1. GitHub MCP Server
**Location:** `mcp-server/`
**Purpose:** Provides GitHub API tools to Claude agents
**Tools:**
- `github_issue_read` - Read issue details
- `github_issue_comment` - Post comments
- `github_issue_comments` - List comments
- `github_pr_create` - Create pull requests
- `github_ci_status` - Check CI status

**Start:**
```bash
cd mcp-server
npm install
npm run build
```

### 2. Rust Daemon
**Location:** `src/`
**Purpose:** Polls GitHub and spawns Claude agents
**Features:**
- Adaptive polling (5s / 15s / 60s based on activity)
- Smart agent routing (Planner vs Executor)
- Worktree management
- SQLite conversation tracking
- Budget enforcement

**Build:**
```bash
cargo build --release -p claude-automation
```

**Binary:** `/home/curious/S3M2P/target/release/claude-automation`

### 3. Agents
**Location:** `.claude/agents/`

**Planner (Opus):**
- Deep analysis and architecture
- Initial planning on new issues
- Re-planning on major changes
- Cost: ~$0.08 per use

**Executor (Sonnet):**
- Fast implementation
- Quick iterations
- Responds to feedback
- Cost: ~$0.02 per use

---

## Configuration

### Main Config
**File:** `config.toml`

Key settings:
```toml
[daemon]
poll_interval_idle_secs = 60       # Low activity
poll_interval_active_secs = 15     # Normal work
poll_interval_very_active_secs = 5  # Rapid iteration

[github]
auto_label = "claude-auto"  # Trigger label

[limits]
max_cost_per_issue_usd = 5.00
daily_automation_limit = 20

[worktree]
base_path = "/home/curious/worktrees/auto"
```

### MCP Server Config
**File:** `.claude/settings.json`

Registered automatically - uses `GITHUB_TOKEN` from environment.

---

## Usage Examples

### Simple Feature Request

```markdown
Title: Add pause button to HELIOS
Labels: project:helios, claude-auto

Add a pause/resume button to the timeline controls.
```

**What Happens:**
1. GitHub Actions posts CLAUDE_TRIGGER
2. Daemon detects within 60s
3. Planner (Opus) analyzes and posts plan
4. Executor (Sonnet) implements
5. Preview deployed to: `issue-123-helios.too.foo`
6. You comment: "Looks good, ship it!"
7. Executor creates PR

**Time:** ~5 minutes
**Cost:** ~$0.20

### Complex Feature with Iteration

```markdown
Title: Add dark mode to WELCOME
Labels: project:welcome, claude-auto

Add dark mode toggle with smooth transitions.
```

**Conversation:**
```
Planner: "📋 I'll use CSS variables... [plan details]"
Executor: "✅ Done! Preview: issue-124-welcome.too.foo"

You: "Can the toggle be a moon icon?"
Executor: "🔧 Changed to moon icon. Preview updated!"

You: "Make it bigger"
Executor: "✅ 20% larger. Preview updated!"

You: "Perfect! Ship it"
Executor: "🎉 PR #457 created!"
```

**Time:** ~10 minutes
**Cost:** ~$0.25 (1 Planner + 3 Executor iterations)

---

## Adaptive Polling Behavior

The daemon adjusts its polling rate based on your activity:

| State | Interval | When |
|-------|----------|------|
| **Idle** | 60s | No activity for 10+ minutes |
| **Active** | 15s | You commented in last 10 min |
| **Very Active** | 5s | Rapid back-and-forth (< 2 min) |

**Example:**
```
09:00 - Daemon idle (60s polling)
09:05 - You create issue → Detected within 60s
09:06 - Switches to Active (15s polling)
09:08 - You comment → Detected within 15s
09:08:05 - Switches to VeryActive (5s polling)
09:10 - You comment → Detected within 5s (feels instant!)
09:20 - No activity for 10min → Drops to Idle (60s)
```

---

## Monitoring

### Check Daemon Status
```bash
systemctl --user status claude-automation
```

### View Logs
```bash
# Live logs
journalctl --user -u claude-automation -f

# Or from file
tail -f ~/.claude/automation-daemon.log
```

### Database Queries
```bash
sqlite3 ~/.claude/automation.db "SELECT * FROM automations ORDER BY started_at DESC LIMIT 5"
```

---

## Cost Tracking

### Per Issue (Typical)
- Planner (Opus): $0.08
- Executor (Sonnet): $0.02-0.10 (depends on iterations)
- **Average:** $0.20/issue

### Monthly (20 issues)
- Total: ~$4-6/month
- Time saved: ~60 hours
- **ROI:** Massive!

---

## Troubleshooting

### Daemon not responding?
```bash
# Check if running
systemctl --user status claude-automation

# Restart
systemctl --user restart claude-automation

# Check logs for errors
journalctl --user -u claude-automation --since "5 minutes ago"
```

### Agent not posting?
- Check GITHUB_TOKEN is set in environment
- Verify MCP server is running: `ps aux | grep "mcp-server"`
- Check `.claude/settings.json` has correct paths

### Preview not deploying?
- Check GitHub Actions tab for workflow runs
- Verify CLOUDFLARE_API_TOKEN secret is set
- Ensure preview branch was created: `git branch -r | grep preview`

---

## Development

### Run Daemon Locally (Debug)
```bash
GITHUB_TOKEN=ghp_xxx cargo run -p claude-automation
```

### Test MCP Server
```bash
cd mcp-server
GITHUB_TOKEN=ghp_xxx npm start
```

### Manual Agent Test
```bash
# Test Planner
claude --agent planner --model opus --env ISSUE_NUMBER=123

# Test Executor
claude --agent executor --model sonnet --env ISSUE_NUMBER=123
```

---

## File Locations

```
S3M2P/
├── TOOLS/CLAUDE_AUTOMATION/
│   ├── mcp-server/              # GitHub API integration
│   ├── src/                     # Rust daemon
│   ├── config.toml              # Configuration
│   └── README.md                # This file
│
├── .claude/
│   ├── agents/
│   │   ├── planner.md           # Opus agent
│   │   └── executor.md          # Sonnet agent
│   ├── settings.json            # MCP registration
│   └── automation.db            # SQLite state
│
├── .github/workflows/
│   ├── claude-automation.yml    # Trigger workflow
│   ├── preview-deploy.yml       # Auto-deploy previews
│   └── preview-cleanup.yml      # Cleanup old previews
│
└── ~/.config/systemd/user/
    └── claude-automation.service  # Daemon service
```

---

## Next Steps

1. **Enable the daemon:**
   ```bash
   systemctl --user enable claude-automation
   systemctl --user start claude-automation
   ```

2. **Set GitHub token:**
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export GITHUB_TOKEN="ghp_your_token_here"
   ```

3. **Create a test issue:**
   - Label it with `claude-auto`
   - Watch the magic happen!

4. **Monitor the daemon:**
   ```bash
   journalctl --user -u claude-automation -f
   ```

---

## Support

- **Logs:** `~/.claude/automation-daemon.log`
- **Database:** `~/.claude/automation.db`
- **Config:** `TOOLS/CLAUDE_AUTOMATION/config.toml`

For issues, check the logs first!
