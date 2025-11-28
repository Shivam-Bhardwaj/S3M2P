# Project Status

Show the current status of the S3M2P monorepo.

## Instructions

1. **List all worktrees**:
   ```bash
   git worktree list
   ```

2. **Show current branch and changes**:
   ```bash
   git status --short
   git log --oneline -5
   ```

3. **Check for open issues assigned to current user** (if any):
   ```bash
   gh issue list --assignee @me --state open
   ```

4. **Check open PRs**:
   ```bash
   gh pr list --state open
   ```

5. **Show build status** of each project:
   ```bash
   cargo check --workspace 2>&1 | tail -5
   ```

## Output Format

```
## S3M2P Status

### Current Context
- **Directory:** [path]
- **Branch:** [branch name]
- **Issue:** #[number] (if in issue worktree)

### Active Worktrees
| Path | Branch | Issue |
|------|--------|-------|
| ... | ... | ... |

### Open Issues (assigned to you)
- #X: [title] (project)
- ...

### Open PRs
- #X: [title] → main
- ...

### Build Status
- core: OK
- helios: OK
- ...
```
