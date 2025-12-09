# Plan: Issue #46 - Bubble Text Readability Improvement

## Problem Statement

On mobile screens, bubbles are too small to identify without clicking. Users need visual labels that clearly indicate what each bubble represents.

## Requirements (from issue)

| Requirement | Value | Description |
|-------------|-------|-------------|
| Text size | 10% of bubble diameter | Curved text wrapping below bubble |
| Text gap | 2% of bubble diameter | Distance from bubble edge to text |
| Outer gap | 5% of (bubble + text) | Spacing between bubbles |
| Text position | Bottom arc | Symmetrical about the 6 o'clock point |
| Big circle | Contains all bubbles | Calculate bubble size based on this |
| No overlap | Required | Handle floating point precision |

## Current Implementation

```
constellation_size = available_min * 0.35
bubble_size = constellation_size * 0.12
orbit_radius = max(min_orbit, target_orbit)
```

- Labels appear below bubble on hover (opacity: 0 → 1)
- No always-visible text
- Positioned via CSS transform (rotate → translate → rotate)

## New Algorithm

### 1. Calculate Big Circle Radius

```
big_circle_radius = constellation_size / 2
```

### 2. Calculate Bubble Size (reverse from requirements)

Given N bubbles arranged in a circle, with:
- `r` = bubble radius
- `t` = text size = 0.10 × (2r) = 0.2r
- `g` = text gap = 0.02 × (2r) = 0.04r
- `m` = outer margin = 0.05 × (2r + t) = 0.05 × 2.2r = 0.11r
- `effective_radius` = r + g + t + m = r + 0.04r + 0.2r + 0.11r = 1.35r

For N bubbles in a circle with margin from big circle edge:
```
orbit_radius = big_circle_radius - effective_radius - margin_from_edge
margin_from_edge = 0.05 × (2 × effective_radius)
```

Minimum orbit to prevent overlap:
```
min_spacing = 2 × effective_radius
min_orbit = min_spacing / (2 × sin(π/N))
```

Solving for r:
```
effective_radius = 1.35r
orbit + effective_radius + margin = big_circle_radius
margin = 0.05 × 2 × 1.35r = 0.135r

orbit + 1.35r + 0.135r = big_circle_radius
orbit + 1.485r = big_circle_radius
```

Also: `orbit ≥ 1.35r × 2 / (2 × sin(π/N))`

### 3. Implementation Approach

**Phase 1: Layout Algorithm Update** (`main.rs`)
1. Create new `BubbleLayout` struct with all calculated values
2. Implement algorithm to solve for bubble_radius given:
   - `big_circle_radius` (constrained by viewport)
   - `N` bubbles
   - Spacing requirements (10%, 2%, 5%)
3. Add collision detection to verify no overlap
4. Round values to integers at the end to avoid floating point drift

**Phase 2: SVG Text Rendering** (`main.rs`)
1. Create SVG element per bubble with:
   - Circular `<path>` for text baseline (arc below bubble)
   - `<textPath>` referencing the path
   - Text centered using `startOffset="50%"` and `text-anchor="middle"`
2. Size SVG to encompass bubble + text + gap
3. Position SVG at same location as bubble

**Phase 3: CSS Updates** (`index.html`)
1. Style `.bubble-text-arc` for curved text appearance
2. Ensure text is always visible (not just on hover)
3. Maintain hover effects on bubble itself

## Detailed Implementation

### File Changes

#### 1. `WELCOME/src/main.rs`

**Add structs:**
```rust
/// All calculated layout values for the bubble constellation
struct BubbleLayout {
    big_circle_radius: f64,
    bubble_radius: f64,
    text_size: f64,        // 10% of diameter
    text_gap: f64,         // 2% of diameter
    outer_margin: f64,     // 5% of effective diameter
    effective_radius: f64, // bubble + gap + text + margin
    orbit_radius: f64,
}

impl BubbleLayout {
    fn calculate(viewport_min: f64, bubble_count: usize) -> Self {
        // Implementation here
    }
}
```

**Add SVG text rendering:**
```rust
fn create_bubble_text_svg(
    document: &Document,
    label: &str,
    layout: &BubbleLayout,
) -> web_sys::Element {
    // Create SVG with arc path and textPath
}
```

**Modify `render_bubbles()`:**
- Use new `BubbleLayout::calculate()`
- Call `create_bubble_text_svg()` for each bubble
- Position SVG alongside bubble element

#### 2. `WELCOME/index.html`

**Add CSS for curved text:**
```css
.bubble-text-arc {
    position: absolute;
    top: 50%;
    left: 50%;
    pointer-events: none;
    /* Size set by JS: width/height = effective_diameter */
}

.bubble-text-arc text {
    font-family: 'Rajdhani', sans-serif;
    font-size: var(--text-size);
    font-weight: 600;
    fill: rgba(255, 255, 255, 0.9);
    text-shadow: 0 0 5px rgba(0, 0, 0, 0.8);
    letter-spacing: 1px;
    text-transform: uppercase;
}
```

### SVG Arc Text Structure

```svg
<svg class="bubble-text-arc" width="..." height="..." viewBox="...">
  <defs>
    <path id="text-arc-{i}"
          d="M {x1} {y1} A {r} {r} 0 0 1 {x2} {y2}" />
  </defs>
  <text>
    <textPath href="#text-arc-{i}"
              startOffset="50%"
              text-anchor="middle">
      LABEL
    </textPath>
  </text>
</svg>
```

Arc parameters:
- Center at (cx, cy) = center of SVG
- Arc radius = bubble_radius + text_gap + text_size/2
- Arc spans ~120° centered at bottom (from -30° to +30° from 6 o'clock)

### Floating Point Precision

1. All calculations use f64 internally
2. Final pixel values rounded to nearest 0.5px
3. Collision check: `distance(b1, b2) > effective_radius_1 + effective_radius_2 + epsilon`
4. Epsilon = 0.5px for safety margin

### Testing Strategy

1. **Visual regression**: Screenshot at 375px, 768px, 1440px widths
2. **Overlap detection**: Console warning if any bubbles overlap
3. **Edge cases**: 1 bubble, 2 bubbles, 7 bubbles (HOME), many bubbles

## Task Breakdown

1. [ ] Create `BubbleLayout` struct and `calculate()` method
2. [ ] Implement collision detection helper
3. [ ] Create `create_bubble_text_svg()` function
4. [ ] Modify `render_bubbles()` to use new layout
5. [ ] Add CSS for `.bubble-text-arc`
6. [ ] Test on mobile viewport (375px)
7. [ ] Test on tablet viewport (768px)
8. [ ] Test on desktop viewport (1440px)
9. [ ] Verify no hover regressions
10. [ ] Validate with `trunk build`

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Text too small on mobile | Minimum text size of 10px |
| Overlapping bubbles | Collision detection + console warning |
| Performance regression | Minimize DOM elements, use CSS transforms |
| SVG text rendering differences | Test Chrome, Firefox, Safari |

## Out of Scope

- Changing bubble icons or colors
- Animation of text appearance
- Touch gesture improvements
- Other pages beyond WELCOME