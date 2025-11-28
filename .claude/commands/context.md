# Load Project Context

Automatically detect and load context for the current project.

## Instructions

1. **Determine current location**:
   ```bash
   pwd
   git branch --show-current
   ```

2. **Detect project from path or branch**:
   - If in worktree, extract project from branch name (e.g., `helios/issue-23` -> helios)
   - If in subdir, use that (e.g., `/S3M2P/helios/src` -> helios)
   - Otherwise, default to root context

3. **Load appropriate CLAUDE.md**:
   - Read the root `CLAUDE.md` for general context
   - Read the project-specific `<project>/CLAUDE.md` if working on a project

4. **Check for issue context**:
   - Extract issue number from branch if present
   - Fetch issue details: `gh issue view <number> --json title,body,labels`

5. **Present context summary**:

## Output Format

```
## Context Loaded

**Project:** [name]
**Branch:** [branch]
**Issue:** #[number] - [title] (if applicable)

### Project Overview
[Summary from project CLAUDE.md]

### Current Issue (if applicable)
[Issue description and acceptance criteria]

### Key Files
- [List relevant source files for this project]

Ready to work on [project].
```
