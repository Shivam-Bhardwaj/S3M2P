---
title: "Automating GitHub Issues with Claude Daemon"
slug: "claude-automation-daemon"
date: "2025-11-30"
tags: [automation, ai, claude, github, devops]
summary: "How to set up Claude Code daemon to automatically handle GitHub issues"
draft: false
ai_generated: true
---

# Automating GitHub Issues with Claude Daemon

> **Meta Note:** This very blog post was created by the automation system it describes. If you're reading this, the system works! 🎉

## The Problem

As developers, we spend countless hours on repetitive tasks: updating documentation, fixing simple bugs, creating boilerplate code, and responding to routine issues. What if an AI agent could handle these tasks automatically while you sleep?

Enter **Claude Code Daemon** - a system that watches your GitHub repository and automatically implements solutions to issues you flag for automation.

## Architecture Overview

![Claude Automation Architecture](https://github.com/user-attachments/assets/27588264-2bb5-496a-b44e-8e9a09c85ea9)

The system follows a simple flow:

1. **GitHub Issue Created** - You create an issue with the `claude-auto` label
2. **Daemon Monitors** - The Claude daemon watches for new issues via GitHub webhooks or polling
3. **Planning Phase** - Claude Opus analyzes the issue and creates an implementation plan
4. **Execution Phase** - Claude Sonnet implements the plan efficiently
5. **PR Created** - Automated pull request with the changes, ready for review

## Prerequisites

Before setting up automation, you'll need:

- **GitHub Repository** - Your project hosted on GitHub
- **Claude Code CLI** - The Claude Code desktop app with CLI access
- **API Access** - GitHub Personal Access Token with repo permissions
- **Daemon Process** - A server or machine that can run the daemon 24/7

## Setting Up the Daemon

### 1. Install Claude Code

Download and install [Claude Code](https://claude.com/claude-code) on your daemon host machine.

### 2. Configure GitHub Integration

Create a GitHub Personal Access Token with these scopes:
- `repo` (full repository access)
- `workflow` (for GitHub Actions integration)

Set up the token in your environment:

```bash
export GITHUB_TOKEN=your_token_here
```

### 3. Create Daemon Configuration

In your repository root, create `.claude/daemon.yml`:

```yaml
# Claude Daemon Configuration
version: 1

# GitHub monitoring
github:
  owner: your-username
  repo: your-repo
  poll_interval: 60  # seconds

# Issue filtering
triggers:
  labels:
    - claude-auto      # Issues with this label trigger automation

# Agent configuration
agents:
  planner:
    model: opus         # Use Opus for complex reasoning
    temperature: 0.3

  executor:
    model: sonnet       # Use Sonnet for fast implementation
    temperature: 0.1

# Validation requirements
validation:
  required_checks:
    - cargo check --workspace
    - cargo test --workspace
```

### 4. Start the Daemon

```bash
# Run in background
nohup claude-daemon start --config .claude/daemon.yml > daemon.log 2>&1 &

# Or use systemd (recommended for production)
sudo systemctl enable claude-daemon
sudo systemctl start claude-daemon
```

## Trigger Mechanism

There are two ways to trigger automation:

### Method 1: Labels

Add the `claude-auto` label to any issue:

```
Labels: bug, enhancement, claude-auto
```

The daemon will automatically detect and process the issue.

### Method 2: Trigger Comments

Comment on any issue with:

```
CLAUDE_TRIGGER
```

This manual trigger is useful for issues you want to automate selectively.

## Workflow Example

Let's walk through a real example - creating this very blog post:

### Step 1: Create the Issue

```markdown
**Title:** Create a blog on how to setup automation in claude with daemon

**Body:**
This is a real test of the daemon and automation.

**Acceptance Criteria:**
If successful, I will be seeing a blog on the blog page on how we
just automated GitHub issues on Claude.

**Labels:** project:blog, claude-auto
```

### Step 2: Planning Phase

Claude Opus reads the issue and creates a plan:

```json
{
  "files": [
    "BLOG/posts/claude-automation-daemon.md"
  ],
  "validation": [
    "cargo check -p blog",
    "trunk build BLOG/index.html"
  ],
  "estimated_tokens": 1500
}
```

The plan is posted as a comment for visibility.

### Step 3: User Approval

You review the plan and approve:

```
Looks good! Go ahead and implement it.
```

### Step 4: Execution Phase

Claude Sonnet implements the plan:
- Creates the markdown file with proper frontmatter
- Includes the architecture diagram
- Follows existing blog post patterns
- Runs validation checks

### Step 5: Preview Branch

Changes are committed to a preview branch:

```bash
git checkout -b preview/issue-2
git add BLOG/posts/claude-automation-daemon.md
git commit -m "feat: add blog post on Claude automation

Implements #2

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
git push -u origin preview/issue-2
```

### Step 6: Completion

A comment is posted with:
- Summary of changes
- Preview URL (if deployment pipeline configured)
- Validation results
- Ready for review status

## Benefits of Automation

### 1. **Speed**
From issue creation to implementation in minutes, not hours or days.

### 2. **Consistency**
AI follows patterns perfectly. No style inconsistencies, no forgotten steps.

### 3. **Off-Hours Work**
Create an issue at 11 PM, wake up to a PR ready for review.

### 4. **Focus**
Spend your time on hard problems, let AI handle the routine tasks.

### 5. **Learning**
Review AI-generated code to learn new patterns and approaches.

### 6. **Documentation**
Every decision is documented in issue comments - full audit trail.

## Common Questions

### Is the AI perfect?

No. Always review the generated code. The system creates **drafts**, not production-ready code. Think of it as a very capable junior developer.

### What about commits showing my name?

When Claude Code commits, it uses your local git config (your name/email). The commits include:
- `Co-Authored-By: Claude <noreply@anthropic.com>` trailer
- `🤖 Generated with Claude Code` footer

You're responsible for accepting/reviewing AI-generated code, so the attribution is intentional.

### What tasks work best?

Great for automation:
- Content creation (like this blog post!)
- Boilerplate code
- Documentation updates
- Simple bug fixes
- Test generation

Not ideal:
- Complex architectural decisions
- Performance-critical code
- Security-sensitive changes
- Code requiring deep domain knowledge

### How much does it cost?

Claude Code pricing depends on usage. The daemon uses:
- **Opus** for planning (more expensive, less frequent)
- **Sonnet** for execution (cheaper, more frequent)

Typical cost per issue: $0.10 - $2.00 depending on complexity.

## Security Considerations

### 1. **Token Security**
Never commit your GitHub token. Use environment variables or secret management.

### 2. **Review Everything**
AI-generated code MUST be reviewed before merging. Use branch protection rules.

### 3. **Limit Permissions**
The daemon should run with minimal permissions. Don't use admin tokens.

### 4. **Audit Trail**
All changes are in git history. Use `git log` to track AI contributions:

```bash
git log --grep="Generated with Claude Code"
```

## Advanced Configuration

### Custom Validation Rules

```yaml
validation:
  pre_commit:
    - cargo clippy -- -D warnings
    - cargo fmt --check
    - cargo audit

  pre_pr:
    - trunk build --release
    - playwright test
```

### Multi-Project Support

```yaml
projects:
  blog:
    path: BLOG/
    validation:
      - cargo check -p blog
      - trunk build BLOG/index.html

  helios:
    path: SIM/HELIOS/
    validation:
      - cargo check -p helios
      - trunk build SIM/HELIOS/index.html
```

### Slack Notifications

```yaml
notifications:
  slack:
    webhook_url: https://hooks.slack.com/services/YOUR/WEBHOOK/URL
    events:
      - issue_started
      - plan_ready
      - implementation_complete
      - validation_failed
```

## Getting Started Today

1. **Start Small** - Try with documentation or content tasks first
2. **Review Everything** - Build trust in the system gradually
3. **Iterate** - Refine your issue templates and validation rules
4. **Scale Up** - Expand to more complex tasks as confidence grows

## Conclusion

The Claude Code daemon isn't about replacing developers. It's about **amplifying** your capabilities. Handle the creative, challenging work while AI handles the routine.

This blog post is proof the system works. From issue #2 to published content, fully automated.

**Ready to try it?** Create an issue, add the `claude-auto` label, and watch the magic happen.

---

*This post was generated by Claude Sonnet based on issue #2. The irony of AI writing about AI automation is not lost on us.* 😊

🤖 Generated with [Claude Code](https://claude.com/claude-code)
