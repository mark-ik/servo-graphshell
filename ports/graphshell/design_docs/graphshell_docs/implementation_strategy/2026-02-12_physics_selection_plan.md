# Physics Selection Plan (2026-02-12)

## Physics Selection Plan
- Goal: Replace custom physics/worker stack with egui_graphs force-directed layout and consolidate selection to one source of truth.
- Scope: Graphshell runtime state, render integration, and tests. No Servo core changes.
- Sequencing: Physics migration and selection consolidation happen before FT2 thumbnail completion.

### Feature Target 1: Physics Engine Migration
- Replace `LayoutRandom` usage with egui_graphs force-directed layout path.
- Remove custom worker-based simulation loop and graph cloning channel path.
- Keep pin semantics by adding a small pin-aware force algorithm wrapper/state if required.
- Preserve current user-visible controls:
  - physics toggle
  - config panel behavior
  - interaction pause/resume during drag

Validation tests:
- Existing physics/render interaction tests still pass after migration.
- No edge-force double application (regression against previous `out_neighbors` + `in_neighbors` behavior).
- Pinned nodes remain fixed under layout steps.

### Feature Target 2: Selection State Consolidation
- Remove duplicated state model:
  - `app.selected_nodes` collection
  - `node.is_selected` field
- Promote one canonical selection container in app state.
- Update graph adapter/rendering to compute selected visual state from canonical selection container.
- Remove full-graph O(n) selection clear path used on each single-select.

Validation tests:
- Single-select and multi-select behavior unchanged from user perspective.
- Selection survives graph interactions correctly (drag, focus, delete).
- App and render tests no longer rely on duplicated selected state.

### Feature Target 3: Documentation and Roadmap Alignment
- Update active docs to reflect:
  - egui_tiles migration completion,
  - physics migration as immediate next implementation task,
  - FT2 thumbnail completion directly after physics/selection work.

Validation tests:
- No active docs claim stale architecture (custom worker physics as settled design).
- "What's next" sections are consistent across roadmap and README.

## Findings
- Current custom physics still has known correctness/maintenance risks:
  - spring attraction applied over both `out_neighbors` and `in_neighbors`,
  - worker thread complexity (full graph clones, lifecycle management) for typical graph sizes.
- Current render path still uses `LayoutRandom` while external physics updates positions.
- Selection is currently duplicated (`selected_nodes` + per-node selected flag), increasing coherence risk and extra work per select operation.
- `egui_graphs` already provides a force-directed layout foundation and integrates directly with current graph rendering.
- This migration is not blocked by missing prerequisites in Servo or egui_tiles.

## Progress
- 2026-02-12: Plan created.
- 2026-02-12: Confirmed sequencing decision:
  - Physics migration and selection consolidation first,
  - then FT2 thumbnail completion,
  - then FT6 search/filter integration.

