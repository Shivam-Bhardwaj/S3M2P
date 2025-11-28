# S3M2P Development Workflow

Professional workflow for parallel development using git worktrees and Claude Code.

## Overview

```
                    ┌─────────────────┐
                    │   GitHub Issue  │
                    │  [project] #23  │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │ ./scripts/      │
                    │ worktree.sh     │
                    │ create 23       │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
    ┌─────────▼───────┐ ┌────▼────┐ ┌──────▼──────┐
    │ S3M2P-helios-23 │ │ S3M2P   │ │ S3M2P-core-5│
    │ (worktree)      │ │ (main)  │ │ (worktree)  │
    └─────────────────┘ └─────────┘ └─────────────┘
```

## Workflow Steps

### 1. Create an Issue

Go to GitHub → Issues → New Issue → Select template:
- `[core] Issue` - For core simulation engine
- `[helios] Issue` - For helios visualization
- `[too.foo] Issue` - For too.foo visualization
- `[storage-server] Issue` - For backend
- `[infra] Issue` - For CI/CD, tooling, cross-project

The template ensures:
- Project is labeled (`project:xxx`)
- Acceptance criteria defined
- Priority set

### 2. Start Work with Claude Code

```bash
cd ~/Desktop/S3M2P
claude

# In Claude Code:
/work 23
```

This command will:
1. Fetch issue #23 from GitHub
2. Determine project from labels/title
3. Create worktree at `../S3M2P-<project>-23/`
4. Create branch `<project>/issue-23`
5. Show you the implementation plan

Then:
```bash
cd ../S3M2P-helios-23
claude
```

### 3. Develop in Isolation

Each worktree is a complete copy:
- Independent `node_modules/` (if needed)
- Shared `.git/` (via worktree)
- Can run separate dev servers

```bash
# In worktree
trunk serve helios/index.html  # Port 8080

# In another terminal, different worktree
cd ../S3M2P-toofoo-15
trunk serve too.foo/index.html --port 8081
```

### 4. Validate Before Commit

```bash
# In Claude Code:
/validate
```

This runs:
- `cargo check` on affected crates
- `cargo test` on test crates
- `trunk build` for WASM crates
- `playwright test` if UI changed

### 5. Create Pull Request

```bash
# In Claude Code:
/pr
```

This:
- Pushes branch to origin
- Creates PR linking to issue
- Includes test plan

### 6. Clean Up After Merge

```bash
# From main repo
./scripts/worktree.sh remove 23
```

Or clean all merged worktrees:
```bash
./scripts/worktree.sh clean
```

## Claude Code Commands

| Command | Description |
|---------|-------------|
| `/work <issue>` | Start work on issue (creates worktree) |
| `/validate` | Run validation checks |
| `/pr` | Create pull request |
| `/status` | Show repo status, worktrees, open issues |

## Worktree Script Commands

```bash
./scripts/worktree.sh create <issue>   # Create worktree for issue
./scripts/worktree.sh list             # List all worktrees
./scripts/worktree.sh goto <issue>     # Print path to worktree
./scripts/worktree.sh remove <issue>   # Remove worktree
./scripts/worktree.sh clean            # Remove prunable worktrees
```

## Best Practices

### Issue Hygiene
- One issue = one feature/fix
- Keep scope small (1-3 day max)
- Link related issues
- Update status as you work

### Branch Naming
Automatic: `<project>/issue-<number>`
- `helios/issue-23`
- `core/issue-5`
- `infra/issue-100`

### Commit Messages
```
[project] Short description

Longer explanation if needed.

Closes #23
```

### Code Review Checklist
- [ ] Tests pass (`/validate`)
- [ ] No new warnings
- [ ] CLAUDE.md updated if API changed
- [ ] Screenshots for UI changes

## Parallel Work Example

Working on 3 issues simultaneously:

```bash
# Terminal 1: Helios feature
cd ~/Desktop/S3M2P-helios-23
claude
# "I'm working on issue 23 - add constellation overlay"

# Terminal 2: Core bug fix
cd ~/Desktop/S3M2P-core-5
claude
# "I'm fixing issue 5 - NaN in flocking forces"

# Terminal 3: too.foo enhancement
cd ~/Desktop/S3M2P-toofoo-15
claude
# "I'm implementing issue 15 - season indicators"
```

Each Claude session has:
- Isolated file context
- Project-specific CLAUDE.md loaded
- Independent git branch

## Troubleshooting

### Worktree creation fails
```bash
# Check if branch exists
git branch -a | grep issue-23

# Manual worktree creation
git worktree add ../S3M2P-helios-23 -b helios/issue-23
```

### gh command not authenticated
```bash
gh auth login
```

### Merge conflicts
```bash
# In worktree
git fetch origin main
git rebase origin/main
# Resolve conflicts
git rebase --continue
```

### Cleaning stale worktrees
```bash
git worktree prune
git worktree list  # Verify cleaned
```
