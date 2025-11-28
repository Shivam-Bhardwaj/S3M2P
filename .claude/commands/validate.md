# Validate Changes

Run validation checks before committing changes.

## Instructions

1. **Determine which projects changed**:
   ```bash
   git diff --name-only HEAD
   git diff --name-only --staged
   ```

2. **Run project-specific validations**:

   ### For core changes:
   ```bash
   cd core && cargo check && cargo test
   ```

   ### For helios changes:
   ```bash
   cd helios && cargo check
   trunk build --release helios/index.html
   ```

   ### For too.foo changes:
   ```bash
   cd too.foo && cargo check
   trunk build --release too.foo/index.html
   ```

   ### For storage-server changes:
   ```bash
   cd storage-server && cargo check && cargo test
   ```

3. **Run workspace-wide check if multiple projects affected**:
   ```bash
   cargo check --workspace
   ```

4. **Run visual regression tests if UI changed**:
   ```bash
   npx playwright test
   ```

5. **Report results**:

## Output Format

```
## Validation Results

### Changed Projects
- [x] core (3 files)
- [ ] helios

### Checks
| Check | Status | Details |
|-------|--------|---------|
| cargo check | PASS | |
| cargo test | PASS | 12 tests |
| trunk build | PASS | |
| playwright | SKIP | No UI changes |

### Issues Found
[List any errors or warnings]

### Ready to Commit
[Yes/No - with reasons if No]
```
