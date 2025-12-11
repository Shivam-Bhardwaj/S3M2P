# Metadata Sanity Check Agent

You are performing a comprehensive metadata sanity check on the S3M2P codebase. This command analyzes 1570+ source files, generates accurate PURPOSE descriptions using code analysis, updates file headers, and validates architecture.

## Parameters
Parse command-line arguments:
- `--target <path>`: Process specific file/directory (default: all source files)
- `--skip-validation`: Skip architecture validation phase
- `--dry-run`: Preview changes without writing files
- `--verbose`: Show detailed progress and analysis

## Phase 1: Discovery & Planning (5-10 seconds)

### 1.1 Scan Source Files
Discover all source files in the codebase:

**Directories to scan**:
- DNA/, TOOLS/, SIMULATION/, LEARN/, HELIOS/, WELCOME/, BLOG/, ARCH/, SCRIPTS/

**Include extensions**:
- `.rs`, `.js`, `.ts`, `.html`, `.css`, `.md`, `.sh`, `.toml`

**Exclude paths** (use Glob to filter):
- `**/target/**`
- `**/node_modules/**`
- `**/dist/**`
- `**/.git/**`
- `**/assets/**`
- `**/images/**`
- Binary files (Cargo.lock, package-lock.json)

### 1.2 Group and Prioritize
Group files by directory and determine processing order:

**Processing order** (DNA first, projects last):
1. DNA/ (foundation) - ~102 files
2. */CORE/* (engines) - ~300 files
3. SIMULATION/ - ~150 files
4. TOOLS/ - ~400 files
5. LEARN/ - ~350 files
6. HELIOS/, WELCOME/, BLOG/, ARCH/ - ~250 files
7. SCRIPTS/ - ~20 files

### 1.3 Display Discovery Summary
Show table:
```
| Directory    | Files | Priority |
|--------------|-------|----------|
| DNA          | 102   | 1        |
| TOOLS/CORE   | 300   | 2        |
| SIMULATION   | 150   | 3        |
| TOOLS        | 400   | 4        |
| LEARN        | 350   | 5        |
| Projects     | 250   | 6        |
| SCRIPTS      | 20    | 7        |
| **Total**    | 1570  |          |
```

If `--target` specified, filter to only matching files.

## Phase 2: Analysis & Update (20-40 minutes for all files)

For each file in priority order:

### 2.1 Read and Analyze File

**Read file content**:
```
content = Read(file_path)
extension = path.extname(file_path)
relative_path = path.relative(workspace_root, file_path)
```

**Detect comment style**:
- `.rs` → `//!` (Rust doc comment)
- `.js/.ts` → `/**  */` (JSDoc)
- `.html` → `<!-- -->` (HTML comment)
- `.sh` → `#` (Shell comment)
- `.toml` → `#` (TOML comment)
- `.css` → `/* */` (CSS comment)
- `.md` → `---` YAML frontmatter

### 2.2 Code Structure Analysis

**For Rust files (`.rs`)**:
Extract using regex patterns:
- `pub fn \w+` → Count public functions
- `pub struct \w+` → Extract public types
- `pub enum \w+` → Extract public enums
- `use [\w:]+` → Extract imports
- `mod \w+` → Count modules
- `#\[wasm_bindgen\]` → Detect WASM entry
- `fn main\(\)` → Detect binary entry
- `#\[cfg\(test\)\]` → Count tests

**For JavaScript/TypeScript (`.js`, `.ts`)**:
- `export (function|const|class)` → Extract exports
- `import .* from` → Extract imports
- `export default` → Find main export

**For HTML (`.html`)**:
- `<title>` → Extract page title
- `<canvas id="` → Detect canvas app
- `<script.*wasm` → Detect WASM app

**Build structure object**:
```json
{
  "exports": ["BoidArena", "SpatialGrid", "Boid"],
  "imports": ["std::cell::RefCell", "glam::Vec3"],
  "main_entry": "fn main()" | "wasm_bindgen::start" | "N/A",
  "counts": {
    "public_functions": 12,
    "public_types": 5,
    "modules": 3,
    "tests": 2
  }
}
```

### 2.3 Infer Layer from Path

Use path-based rules:
- `DNA/*` → "DNA (foundation)"
- `DNA/src/physics/*` → "DNA → PHYSICS"
- `*/CORE/*` → "CORE → {engine_name}"
- `SIMULATION/*` → "SIMULATION → {project}"
- `TOOLS/*` → "TOOLS → {tool}"
- `LEARN/*` → "LEARN → {topic}"
- `HELIOS/*` → "HELIOS (simulation)"
- `WELCOME/*` → "WELCOME (landing page)"
- `ARCH/*` → "ARCH (architecture explorer)"
- `SCRIPTS/*` → "SCRIPTS (build automation)"

### 2.4 Find Similar Files

Look for 2-3 well-documented files in the same directory with existing PURPOSE headers:
```
same_dir_files = Glob(`${dir}/*.${ext}`)
similar = same_dir_files
  .filter(f => f has PURPOSE header with ═══)
  .slice(0, 3)
  .map(f => ({path: f.path, purpose: extract_purpose(f)}))
```

### 2.5 Generate PURPOSE Description

**Build analysis context**:
```markdown
File Analysis for: {relative_path}

File Type: {extension}
Layer: {inferred_layer}

Code Structure:
- Exports: {exports.join(", ") || "N/A"}
- Imports: {imports.slice(0,5).join(", ")}...
- Main Entry: {main_entry}
- Contains: {counts.public_functions} functions, {counts.public_types} types, {counts.modules} modules

Similar files in this directory:
{similar_files.map(f => `- ${f.path}: "${f.purpose}"`).join("\n")}

File Content (first 5000 characters):
{content.slice(0, 5000)}
```

**Generate PURPOSE using analysis**:

Use the code analysis to intelligently generate PURPOSE descriptions:

1. **For files with clear exports**:
   - If exports types (struct, enum, class): "Defines {TypeNames} {domain context}"
   - If exports functions: "Implements {functionality} {algorithm details}"
   - If both: "Defines {types} and implements {functions}"

2. **For files with main entry**:
   - If `fn main()`: "CLI entry point for {functionality}"
   - If `wasm_bindgen`: "WASM entry point {purpose}"
   - If HTML with canvas: "Interactive {app_type} with {features}"

3. **For module files (mod.rs, index.js)**:
   - "Module exports: {list_of_exports}"

4. **Match style of similar files**:
   - Use similar vocabulary and structure
   - Keep length consistent (1-2 sentences)
   - Use active voice verbs: Defines, Implements, Renders, Calculates, etc.

5. **Add domain context when helpful**:
   - Physics files: mention algorithms (RK4, Euler, SPH, etc.)
   - Math files: mention mathematical concepts
   - Simulation files: describe phenomena being simulated

**Example generated PURPOSE descriptions**:
- `DNA/src/data/arena.rs` → "Generic fixed-capacity arena allocator with generational indices for safe entity references"
- `HELIOS/src/render.rs` → "Canvas 2D rendering engine for solar system visualization with starfield and orbit paths"
- `TOOLS/AUTOCRATE/src/lib.rs` → "WASM entry point for shipping crate calculator with dimension optimization"
- `DNA/src/physics/orbital/kepler.rs` → "Implements Kepler's laws for orbital mechanics including period and true anomaly calculations"

### 2.6 Build Metadata Header

Generate header with appropriate comment syntax:

**For Rust (`.rs`)**:
```rust
//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: {filename} | {relative_path}
//! PURPOSE: {generated_purpose}
//! MODIFIED: {current_date_YYYY-MM-DD}
//! LAYER: {inferred_layer}
//! ═══════════════════════════════════════════════════════════════════════════════
```

**For JavaScript/TypeScript**:
```javascript
/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * FILE: {filename} | {relative_path}
 * PURPOSE: {generated_purpose}
 * MODIFIED: {current_date_YYYY-MM-DD}
 * ═══════════════════════════════════════════════════════════════════════════════
 */
```

**For HTML**:
```html
<!--
═══════════════════════════════════════════════════════════════════════════════
FILE: {filename} | {relative_path}
PURPOSE: {generated_purpose}
MODIFIED: {current_date_YYYY-MM-DD}
═══════════════════════════════════════════════════════════════════════════════
-->
```

### 2.7 Update File

**Check for existing header**:
```
has_header = content.includes("═══════════")
```

**If header exists (UPDATE)**:
- Find header boundaries (from first `═══` to second closing `═══`)
- Replace entire header block with new header
- Preserve rest of file content

**If no header (INSERT)**:
- Check for shebang (`#!/`) - preserve at top
- Check for HTML `<!DOCTYPE>` - insert after
- Otherwise insert header at very top of file
- Add blank line after header

**Write file** (unless `--dry-run`):
```
if dry_run:
  print "Would update: {relative_path}"
  print "  New PURPOSE: {generated_purpose}"
else:
  Write(file_path, updated_content)
  print "[{current}/{total}] Updated {relative_path}"
```

### 2.8 Progress Reporting

Every 50 files, show progress:
```
Progress: 50/1570 (3.2%)
API calls: 50
Estimated cost: $0.001
Time elapsed: 2 min
Estimated remaining: 60 min
```

Continue until all files processed.

## Phase 3: Regenerate Architecture Data (10-20 seconds)

### 3.1 Regenerate db.json
Run the documentation scanner:
```bash
node SCRIPTS/scan_docs.js
```

**Verify**:
- File exists: `ARCH/src/db.json`
- File size > 100KB
- Contains 1570+ entries
- Valid JSON (parse check)

### 3.2 Regenerate workspace_data.json
Run the workspace analyzer:
```bash
node SCRIPTS/generate_workspace_data.js
```

**Verify**:
- File exists: `ARCH/src/workspace_data.json`
- Contains ~60 crates
- Valid JSON (parse check)
- All workspace members included

Show:
```
✓ Regenerated db.json (1570 files)
✓ Regenerated workspace_data.json (60 crates)
```

## Phase 4: Architecture Validation (30-60 seconds)

Only if `--skip-validation` is NOT set:

### 4.1 Scan DNA for Domain Code
Search DNA directory for domain-specific patterns:
```
Grep DNA/ for patterns:
- "heliosphere"
- "solar_wind"
- "parker"
- "termination_shock"
- (other domain keywords)
```

**Report violations**:
```
⚠ DNA violations found:
- DNA/src/heliosphere.rs (contains solar wind logic)
→ Should move to HELIOS/src/
```

### 4.2 Check CORE Dependencies
For each CORE crate in workspace_data.json:
```
core_crates = workspace_data.crates.filter(c => c.layer == "Core")
for crate in core_crates:
  non_dna_deps = crate.dependencies.filter(d => !is_dna(d))
  if non_dna_deps.length > 0:
    warn("${crate.name} depends on non-DNA crates: ${non_dna_deps}")
```

### 4.3 Find Orphan Engines
Check which CORE engines have no consumers:
```
for core_crate in core_crates:
  consumers = workspace_data.crates.filter(c =>
    c.dependencies.includes(core_crate.name)
  )
  if consumers.length == 0:
    warn("Orphan engine: ${core_crate.name}")
```

**Report**:
```
Architecture Validation:
✓ DNA is clean (no domain-specific code)
✓ CORE dependencies are valid (DNA only)
⚠ 2 orphan engines found:
  - cad-engine (no consumers)
  - export-engine (no consumers)
```

## Phase 5: Final Report

Generate comprehensive markdown report:

````markdown
# Metadata Sanity Check Report
*Generated: {current_datetime}*

## Summary
- **Files scanned**: 1570
- **Files updated**: 1342
- **Files skipped**: 228 (already current)
- **Total runtime**: 47 minutes
- **Total cost**: $0.012

## Processing by Directory

| Directory       | Files | Updated | Skipped | API Calls |
|-----------------|-------|---------|---------|-----------|
| DNA             | 102   | 89      | 13      | 89        |
| TOOLS/CORE      | 300   | 267     | 33      | 267       |
| SIMULATION/CORE | 45    | 40      | 5       | 40        |
| SIMULATION      | 105   | 92      | 13      | 92        |
| TOOLS           | 400   | 356     | 44      | 356       |
| LEARN           | 350   | 301     | 49      | 301       |
| HELIOS          | 98    | 85      | 13      | 85        |
| WELCOME         | 67    | 52      | 15      | 52        |
| BLOG            | 45    | 34      | 11      | 34        |
| ARCH            | 38    | 15      | 23      | 15        |
| SCRIPTS         | 20    | 11      | 9       | 11        |

## Architecture Data

✓ **db.json** regenerated successfully
  - 1570 file entries
  - All PURPOSE descriptions updated
  - File size: 487 KB

✓ **workspace_data.json** regenerated successfully
  - 60 workspace crates
  - Dependency graph validated
  - File size: 12 KB

## Architecture Validation

✓ **DNA layer is clean**
  - No domain-specific code found
  - All code is generic/foundational

✓ **CORE dependencies are valid**
  - All CORE engines depend only on DNA
  - No circular dependencies detected

⚠ **2 orphan CORE engines found**:
  - `cad-engine` (no projects using it yet)
  - `export-engine` (no projects using it yet)

  *Note: These engines may be work-in-progress or future features*

## Recommendations

1. ✅ Review generated PURPOSE descriptions (spot-check 10-20 files)
2. ✅ Rebuild ARCH visualization:
   ```bash
   cd ARCH && trunk build index.html
   ```
3. ✅ Test ARCH locally:
   ```bash
   trunk serve ARCH/index.html --port 8087
   ```
4. ✅ Deploy to production:
   ```bash
   ./SCRIPTS/deploy.sh arch --publish
   ```
5. ⚠ Consider adding TOOLS/CAD and TOOLS/SPICE to use cad-engine and export-engine

## Next Steps

- [ ] Commit metadata updates:
  ```bash
  git add .
  git commit -m "feat: comprehensive metadata update via /sanity-check

  - Updated PURPOSE descriptions for 1342 files
  - Regenerated db.json and workspace_data.json
  - Architecture validation passed
  - Ready for ARCH deployment"
  ```

- [ ] Push to remote:
  ```bash
  git push origin main
  ```

- [ ] Deploy ARCH to arch.too.foo:
  ```bash
  ./SCRIPTS/deploy.sh arch --publish
  ```

## Files Modified

*Full list available in git status*

Key directories with updates:
- DNA/: 89 files
- TOOLS/: 356 files
- SIMULATION/: 132 files
- LEARN/: 301 files
- Projects: 239 files
- ARCH/src/db.json (regenerated)
- ARCH/src/workspace_data.json (regenerated)

---

**Sanity check complete!** ✨

The codebase now has accurate, AI-generated PURPOSE descriptions for all source files. The ARCH visualization will display this metadata when rebuilt and deployed.
````

Display this report to the user.

## Error Handling

If any phase fails:

1. **File read/write errors**: Skip file, add to error list, continue
2. **JSON validation errors**: Report which file failed, show error
3. **Script execution errors**: Show stderr, suggest fixes
4. **API errors**: Retry once, then skip file

At end, show error summary if any errors occurred:
```
⚠ Errors encountered: 3
- Failed to read: TOOLS/foo.rs (permission denied)
- Invalid JSON in: config.json (unexpected token)
- API timeout: DNA/complex.rs (retried and succeeded)
```

## Dry-Run Mode

If `--dry-run` flag is set:
- Do NOT write any files
- Do NOT run scan_docs.js or generate_workspace_data.js
- DO show what would be changed:
  ```
  Would update: DNA/src/lib.rs
    Current PURPOSE: "Foundation library root"
    New PURPOSE: "Foundation library root - physics, math, world, data structures"

  Would update: ARCH/src/lib.rs
    Current PURPOSE: "Interactive canvas-based architecture explorer"
    New PURPOSE: "Terminal-style file-level architecture explorer with complete drill-down"
  ```

## Completion

After all phases complete:
1. Show final report (above)
2. If NOT dry-run, suggest git commit command
3. Remind to rebuild and deploy ARCH
4. Exit successfully

Total expected runtime: **45-50 minutes** for full codebase.
Total expected cost: **~$0.015** with prompt caching.
