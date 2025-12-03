# Antimony Labs (SBL) Workflow

## Project Identity
- **Antimony Labs (SBL)**: The overarching project/website.
- **too.foo**: The URL for the landing page (SIM/TOOFOO).
- **S3M2P**: The repository name.

## Issue Tracking
- **GitHub Issues**: The single source of truth for all tasks.
- **No Local TODOs**: If it's not an issue, it doesn't exist.

## Development Workflow
### 1. Start Work
**Option A: Manual / Claude Code**
Always start by picking an issue and creating a worktree.
```bash
work <issue_id>
# Example: work 42
```

**Option B: Agent-Assisted (Gemini/LLM)**
Paste the GitHub issue description or URL into the agent chat. The agent will:
1. Run `./SCRIPTS/worktree.sh create <id>`
2. Set up the environment.

This command:
- Creates a git worktree: `~/worktrees/<project>-issue-<id>`
- Fetches issue details.
- Opens a dedicated terminal layout (System Monitor, Dev Server, Claude Code).

### 2. Run Dev Server
Start the development server for the specific project you are working on.
```bash
run <project>
# Example: run toofoo
# Example: run mcad
```
- **Context Aware**: Run this *inside* your worktree.
- **Auto-Port**: Automatically finds an available port.

### 3. Commit & Push
- **Branch Name**: `issue-<id>` (created automatically by `work`).
- **Preview**: Push to `preview/issue-<id>` for auto-deployment.
  ```bash
  git push origin HEAD:preview/issue-<id>
  ```
- **PR**: Create a Pull Request when ready.
  ```bash
  gh pr create
  ```

## Worktree Best Practices
- **Keep Main Clean**: Never work directly on `main` in the root `S3M2P` directory.
- **Simultaneous Work**: You can have multiple worktrees open for different issues.
- **Cleanup**: Remove worktrees when the issue is closed.
  ```bash
  git worktree remove ~/worktrees/issue-<id>
  ```

## Tools
- **Claude Code**: Use the CLI for all development tasks.
- **VS Code**: Use for code editing and debugging (if needed).
- **Chrome**: Use for testing web apps.
- **Web Command**: Use `web` to open GitHub pages.
  - `web issue`: Open current issue.
  - `web pr`: Open current PR.
  - `web branch`: Open current branch code.
  - `web repo`: Open repository.
