# Start Work on Issue

You are starting work on GitHub issue #$ARGUMENTS.

## Instructions

1. **Fetch issue details** using `gh issue view $ARGUMENTS --json title,body,labels`

2. **Determine the project** from the issue labels (`project:xxx`) or title prefix (`[project]`)

3. **Check if worktree exists** for this issue:
   ```bash
   ./scripts/worktree.sh goto $ARGUMENTS
   ```

4. **If no worktree**, create one:
   ```bash
   ./scripts/worktree.sh create $ARGUMENTS
   ```
   Then inform the user to `cd` to that directory and restart Claude.

5. **If worktree exists** (we're in it), analyze the issue:
   - Parse the description and acceptance criteria
   - Identify affected files/modules
   - Create a TodoWrite plan for implementation

6. **Load project context** by reading the project-specific CLAUDE.md:
   - `core/CLAUDE.md` for core issues
   - `helios/CLAUDE.md` for helios issues
   - etc.

7. **Present your implementation plan** to the user before starting work.

## Output Format

```
## Issue #$ARGUMENTS: [Title]

**Project:** [project name]
**Type:** [bug/feature/etc]
**Priority:** [P0-P3]

### Summary
[Brief summary of what needs to be done]

### Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

### Implementation Plan
1. Step 1
2. Step 2
...

Ready to proceed?
```
