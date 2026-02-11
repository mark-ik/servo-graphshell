# GraphShell Development Memory

## Code Philosophy

**Principle: Favor Maintained Crates Over Custom Code**

When choosing between custom implementations and well-regarded ecosystem crates:
- ✅ **Prefer**: Maintained, well-documented crates compatible with Servo/GraphShell dependencies
- ✅ **Benefits**: Less maintenance burden, battle-tested code, access to algorithms
- ❌ **Avoid**: Custom implementations for problems already solved by the ecosystem
- ⚠️ **Exception**: When crate API conflicts with core architecture or adds significant complexity

**Examples Applied:**
- Replaced custom spatial hash (95 LOC) → `kiddo` KD-tree (per-frame rebuild optimized)
- Replaced custom graph rendering (371 LOC) → `egui_graphs` widget (includes navigation)
- Kept custom physics engine → No suitable force-directed crate for our needs yet
- Added `petgraph` projection layer → Access to graph algorithms without replacing SlotMap core

## Architecture Patterns

**Polling vs Events**
- Servo integration uses polling (`sync_webviews_to_graph()` every frame)
- Simpler than delegate pattern, no async complications
- Works when you have access to both data sources in update loop

**Browsing Path Model**
- Each navigation creates new node (not URL-based reuse)
- Graph reflects browsing history, not URL uniqueness
- Exception: Initial unmapped node reused for first webview

**Tab Lifecycle**
- Save node keys when entering graph view (once only, check condition carefully)
- Destroy webviews to reclaim memory
- Recreate webviews when returning to detail view

## Common Pitfalls

**Borrow Checker**
- Destructure fields before nested calls: `let Self { field_a, field_b, .. } = self;`
- Use `.as_ref()` when passing Options to avoid partial moves

**Frame-Based Conditions**
- Conditions that run every frame can cause bugs
- Example: Tab restoration ran every frame, cleared saved list on frame 2
- Fix: Use `list.is_empty() && other_exists()` to run exactly once

**Camera Zoom**
- Overlapping nodes: Don't zoom too aggressively (2.0x max, not 10.0x)
- Node size scales with zoom: At 10x, a 15px node becomes 150px

## Project-Specific

**File Paths**
- Plan files: `design_docs/graphshell_docs/implementation_strategy/`
- Always update plan files with session notes per DOC_POLICY

**Testing**
- 98 tests currently passing
- Always run full test suite after refactors
- Tests are in same file as code (Rust convention)

**Dependencies**
- egui v0.33.3, egui_graphs v0.29.0 (events feature enabled)
- petgraph v0.8.3 (serde-1 feature)
- kiddo (added in current session)
- Avoid: sled (beta/corruption), keyframe (abandoned), RocksDB (overkill)
