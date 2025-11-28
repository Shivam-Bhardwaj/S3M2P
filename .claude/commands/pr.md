# Create Pull Request

Create a pull request for the current branch/issue.

## Instructions

1. **Get current branch info**:
   ```bash
   git branch --show-current
   git log main..HEAD --oneline
   ```

2. **Extract issue number** from branch name (pattern: `project/issue-XX`)

3. **Run validation** first - ensure all checks pass

4. **Get issue details** for PR description:
   ```bash
   gh issue view [issue-number] --json title,body
   ```

5. **Generate PR**:
   - Title: Issue title (without project prefix)
   - Body: Summary of changes + link to issue
   - Labels: Same as issue labels

6. **Create PR using**:
   ```bash
   gh pr create --title "..." --body "..." --base main
   ```

## PR Body Template

```markdown
## Summary

[Brief description of changes]

Closes #[issue-number]

## Changes

- Change 1
- Change 2

## Testing

- [ ] cargo check passes
- [ ] cargo test passes
- [ ] trunk build succeeds
- [ ] Manual testing done

## Screenshots

[If UI changes, include before/after]
```

## Output

Report the PR URL when complete.
