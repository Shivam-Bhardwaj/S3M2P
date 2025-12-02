# Testing Procedures for Claude Automation

## Pre-Flight Validation

**Always run before starting the daemon:**

```bash
cd /home/curious/S3M2P
./TOOLS/CLAUDE_AUTOMATION/validate.sh
```

This checks:
- ✅ GitHub token configured
- ✅ Agent files exist
- ✅ MCP server built
- ✅ Daemon binary compiled
- ✅ Worktree directory exists
- ✅ `claude-auto` label exists
- ✅ Unit tests pass
- ✅ GitHub Actions workflows present
- ✅ GitHub API accessible
- ✅ MCP server can start

---

## Unit Tests

**Run all unit tests:**
```bash
cargo test -p claude-automation
```

**Test specific modules:**
```bash
cargo test -p claude-automation agent_router  # Agent routing logic
cargo test -p claude-automation state          # Database operations
```

**Current test coverage:**
- `agent_router.rs`: 4 tests ✅
- `state.rs`: 0 tests ⚠️
- `github.rs`: 0 tests ⚠️
- `worktree.rs`: 0 tests ⚠️

---

## Manual Integration Tests

### Test 1: Basic Issue Detection

```bash
# Start daemon
systemctl --user start claude-automation
tail -f ~/.claude/automation-daemon.log

# In another terminal/browser:
# 1. Create issue with claude-auto label
# 2. Manually post CLAUDE_TRIGGER comment
# 3. Watch logs for detection

# Expected output:
# INFO Found 1 new issue(s) with trigger
# INFO New issue #X: Title - spawning Planner (Opus)
# INFO Worktree created successfully
```

### Test 2: Agent Router Decision

```bash
# Test executor keywords
gh issue comment {NUMBER} --body "implement this"
# Watch logs - should see: "Spawning Executor (Sonnet)"

# Test planner keywords
gh issue comment {NUMBER} --body "let's rethink this"
# Watch logs - should see: "Spawning Planner (Opus)"
```

### Test 3: Parallel Issues

```bash
# Create 2 issues with claude-auto
# Post CLAUDE_TRIGGER on both
# Watch daemon handle both simultaneously

# Check worktrees:
ls -la /home/curious/worktrees/auto/

# Should see:
# project1-X/
# project2-Y/
```

### Test 4: PR Comment Monitoring

```bash
# After PR is created
gh pr comment {NUMBER} --body "fix the tests"

# Watch logs - should see:
# INFO PR #X (for issue #Y): 1 new comment(s)
# INFO Spawning Executor (Sonnet) for PR #X feedback
```

---

## Component Testing

### Test MCP Server Standalone

```bash
cd TOOLS/CLAUDE_AUTOMATION/mcp-server
GITHUB_TOKEN=$GITHUB_TOKEN npm start

# In another terminal:
# Test with claude CLI
claude --mcp-config '{"github-automation": {"command": "node", "args": ["dist/index.js"]}}' \
       "Use github_issue_read(1) to read issue #1"
```

### Test Agent Definitions

```bash
# Test Planner
cd /home/curious/worktrees/auto/test-worktree
claude --model opus \
       --append-system-prompt "$(cat /home/curious/S3M2P/.claude/agents/planner.md | sed -n '/^---$/,/^---$/!p')" \
       "Analyze issue #1"

# Test Executor
claude --model sonnet \
       --append-system-prompt "$(cat /home/curious/S3M2P/.claude/agents/executor.md | sed -n '/^---$/,/^---$/!p')" \
       "Implement the plan"
```

### Test Worktree Creation

```bash
# Manual worktree test
cd /home/curious/S3M2P
git worktree add -b preview/issue-999 /home/curious/worktrees/auto/test-999 main

# Verify
git worktree list | grep test-999

# Cleanup
git worktree remove /home/curious/worktrees/auto/test-999
```

---

## Smoke Tests

### Smoke Test: End-to-End

1. **Create test issue:**
   - Title: "[helios] Test smoke"
   - Description: "Add a console.log statement"
   - Labels: `project:helios`, `claude-auto`

2. **Post trigger:**
   ```bash
   gh issue comment {NUMBER} --body "🤖 **CLAUDE_TRIGGER** [test]"
   ```

3. **Watch automation:**
   ```bash
   tail -f ~/.claude/automation-daemon.log
   ```

4. **Expected behavior:**
   - Planner posts plan within 60s
   - Executor implements within 2 min
   - PR created within 5 min
   - Preview deployed within 10 min

5. **Validate:**
   - Check worktree exists
   - Check branch exists
   - Check commits present
   - Check PR created
   - Check CI running

---

## Debugging Tools

### Check Database State

```bash
# Install sqlite3 if needed
sudo apt-get install sqlite3

# Query automations
sqlite3 ~/.claude/automation.db "
SELECT issue_number, status, has_plan, started_at
FROM automations
ORDER BY started_at DESC
LIMIT 5;
"

# Query conversation history
sqlite3 ~/.claude/automation.db "
SELECT issue_number, author, substr(body, 1, 50)
FROM conversation_history
WHERE issue_number = 2
ORDER BY created_at DESC;
"
```

### Check Active Processes

```bash
# Find running Claude processes
ps aux | grep claude

# Check daemon
systemctl --user status claude-automation

# Kill stuck processes
pkill -f "claude.*issue"
```

### Inspect Worktrees

```bash
# List all worktrees
git worktree list

# Check specific worktree
cd /home/curious/worktrees/auto/{project}-{number}
git status
git log --oneline -5
```

---

## Performance Tests

### Test Adaptive Polling

```bash
# Start daemon
systemctl --user start claude-automation

# Watch state transitions
tail -f ~/.claude/automation-daemon.log | grep "Polling\|Activity\|Switching"

# Expected:
# - Starts in Idle (60s)
# - Switches to Active on activity (15s)
# - Switches to VeryActive on rapid comments (5s)
# - Drops back to Idle after 10 min silence
```

### Test Concurrent Issues

```bash
# Create 3 issues simultaneously
for i in 5 6 7; do
  gh issue create --title "[helios] Test $i" --body "Test" --label claude-auto,project:helios
  gh issue comment $i --body "🤖 **CLAUDE_TRIGGER** [test]"
done

# Watch logs - should handle all 3
tail -f ~/.claude/automation-daemon.log | grep "New issue"
```

---

## Known Issues & Workarounds

### Issue: GitHub Actions trigger not posting

**Symptom:** Issues labeled with `claude-auto` don't get CLAUDE_TRIGGER comment

**Workaround:**
```bash
gh issue comment {NUMBER} --body "🤖 **CLAUDE_TRIGGER** [manual]"
```

**Fix:** Debug `.github/workflows/claude-automation.yml`

### Issue: Executor doesn't post to GitHub

**Symptom:** Executor runs but responses appear in logs, not on GitHub

**Workaround:** Check logs for Executor output

**Fix:** Change from `--print` mode to interactive mode with MCP tools

### Issue: Comment loops

**Symptom:** Agent keeps responding to its own comments

**Workaround:** Close the issue to stop loop

**Fix:** Improve comment filtering in `github.rs`

---

## Success Criteria

Before considering the system production-ready:

### Core Functionality
- [ ] Issue detection works 100% of time
- [ ] Planner posts plans to GitHub
- [ ] Executor posts updates to GitHub
- [ ] PR creation works
- [ ] Preview deployment works
- [ ] CI integration works

### Reliability
- [ ] No comment loops
- [ ] Proper turn-based behavior
- [ ] Handles errors gracefully
- [ ] Budget limits enforced
- [ ] Cleanup works correctly

### Testing
- [ ] Unit test coverage >80%
- [ ] Integration tests pass
- [ ] Smoke tests pass
- [ ] Can handle 3 concurrent issues
- [ ] Runs for 24h without crashes

---

## Next Steps for Production

1. ✅ Add unit tests for agent router (DONE)
2. ⚠️ Add unit tests for other modules
3. ⚠️ Fix Executor GitHub posting
4. ⚠️ Fix GitHub Actions trigger
5. ⚠️ Add error recovery
6. ⚠️ Add monitoring dashboard
7. ⚠️ Run 24h soak test

**Current status:** MVP working, needs hardening for production.
