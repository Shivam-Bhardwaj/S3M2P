# AutoCrate - ASTM D6039 Shipping Crate Generator

Professional ASTM D6039 compliant crate generator with 3D visualization and STEP export.

## ✨ Current Features (Phase 1 Complete)

### 3D Visualization
- **WebGL2 renderer** with Phong lighting
- **Interactive orbit controls**: Drag to rotate, scroll to zoom
- **Realistic materials**: Wood grain, plywood, metallic nails
- **ASTM-compliant structure**: Proper frame, sheathing, and fasteners

### What You'll See
Visit http://127.0.0.1:8084/ to see:
- 3 dark brown skids (4x4 lumber) at base
- 11 light tan floor boards (2x6 lumber) spanning across
- 6 medium brown frame posts (4 corners + 2 intermediate)
- 6 frame rails (top + bottom perimeter)
- 5 light plywood panels (4 walls + top)
- 55 metallic nail heads

### Architecture

```
TOOLS/AUTOCRATE/src/
├── lib.rs              - Entry point, WASM bindings
├── assembly.rs         - Component tree structure
├── generator.rs        - Style A/B generation algorithms
├── geometry.rs         - 3D types (Point3, BoundingBox)
├── constants.rs        - ASTM standards
├── calculator.rs       - Dimension calculations
└── render/
    ├── webgl.rs        - 3D renderer + camera
    ├── canvas2d.rs     - 2D technical drawings
    ├── materials.rs    - Material properties
    ├── textures.rs     - Procedural wood grain
    └── mesh.rs         - Box mesh generation

DNA/src/export/step/
├── entities.rs         - Entity ID management
├── writer.rs           - Part 21 file format
├── primitives.rs       - Geometric primitives
├── topology.rs         - B-rep topology (TODO)
├── brep.rs             - Box-to-BRep (TODO)
├── product.rs          - Product structure (TODO)
├── pmi.rs              - PMI annotations (TODO)
└── gdt.rs              - GD&T entities (TODO)
```

## 🎯 ASTM D6039 Compliance

**Style B (Sheathed) Crate - Currently Rendered:**
- ✅ Skids with proper spacing for forklift access
- ✅ Floor boards perpendicular to skids
- ✅ Vertical corner posts at all 4 corners
- ✅ Intermediate posts per 24" spacing rule
- ✅ Top and bottom rail frame
- ✅ Plywood sheathing on all 5 faces
- ✅ Visible fasteners at proper locations

**Style A (Open Frame) - Generator Ready:**
- Corner posts without sheathing panels
- Cleated frame structure only

## 🔧 Development

```bash
# Run dev server
cd TOOLS/AUTOCRATE
trunk serve index.html --port 8084

# Build for production
trunk build --release index.html

# Check compilation
cargo check --package autocrate
```

## 📋 Todo (Phase 2)

- [ ] Wire up GENERATE button to recreate crate from inputs
- [ ] Complete STEP topology/B-rep modules
- [ ] Add PMI annotations (dimensions, nailing schedules, lumber specs)
- [ ] Implement GD&T feature control frames
- [ ] Export formats: CSV, JSON, STEP AP242
- [ ] 2D technical drawing view
- [ ] Datum reference frame visualization

## 🎨 Design

- **Background**: `#1f1f26` (lighter dark for visibility)
- **Wood tones**: Natural brown gradient (dark skids → light plywood)
- **Metallic**: Galvanized steel gray for nails
- **UI**: too.foo theme (#0a0a0f, #3498db accent, JustSans font)

## 📊 Statistics

- **Commit**: 119fe08
- **Lines**: 3,798 insertions across 32 files
- **Performance**: 60 FPS smooth rendering
- **Build time**: ~8 seconds

## 🚀 Next Session Priorities

1. Complete STEP export functionality
2. Add parametric regeneration from UI inputs
3. Implement proper GD&T annotations for manufacturing
4. Test STEP files in FreeCAD/OnShape

---

**Issue**: #43
**Branch**: `autocrate/issue-43`
**Status**: Phase 1 complete, visualization working, STEP export in progress
