# Validate Changes

Run validation checks before committing changes. These checks replace the cloud CI checks.

## Instructions

1. **Run full validation suite** (what CI used to run):
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace -- -D warnings
   cargo check --workspace
   cargo test --workspace
   ```

2. **Run security audit** (optional, for release):
   ```bash
   ./SCRIPTS/audit.sh
   ```

3. **Determine which projects changed**:
   ```bash
   git diff --name-only HEAD
   git diff --name-only --staged
   ```

4. **Run project-specific validations if needed**:

   ### For WASM projects (helios, too.foo):
   ```bash
   trunk build SIM/HELIOS/index.html
   trunk build WELCOME/index.html
   ```

5. **Run visual regression tests if UI changed**:
   ```bash
   npx playwright test
   ```

6. **Report results**:

## Output Format

```
## Validation Results

### Full Suite
| Check | Status | Details |
|-------|--------|---------|
| cargo fmt | PASS/FAIL | |
| cargo clippy | PASS/FAIL | N warnings |
| cargo check | PASS/FAIL | |
| cargo test | PASS/FAIL | N tests |

### Changed Projects
- [x] DNA (3 files)
- [ ] SIM/HELIOS

### Project-Specific
| Check | Status | Details |
|-------|--------|---------|
| trunk build | PASS/SKIP | |
| playwright | SKIP | No UI changes |

### Issues Found
[List any errors or warnings]

### Ready to Commit
[Yes/No - with reasons if No]
```
