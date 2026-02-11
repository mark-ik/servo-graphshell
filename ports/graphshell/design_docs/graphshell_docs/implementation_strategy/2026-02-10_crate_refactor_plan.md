# Crate Refactor Plan: Replacing Custom Code with Ecosystem Solutions

**Date Started:** 2026-02-10
**Status:** Planning
**Goal:** Replace custom graph/rendering/layout code with mature Rust crates to reduce maintenance burden and enable focus on Servo integration

---

## Crate Refactor Plan

### Phase 1: Graph Visualization Core (HIGHEST IMPACT)
**Target:** Replace custom graph rendering and interaction with `egui_graphs`

**Rationale:**
- `egui_graphs` is purpose-built for interactive graph visualization in egui
- Would eliminate ~60% of custom code in `render/mod.rs` and `input/mod.rs`
- Provides professional-grade interaction (selection, dragging, zoom, pan) out of the box
- Active maintenance and good API

**Tasks:**
1. Add `egui_graphs` dependency to `Cargo.toml`
2. Evaluate `egui_graphs` API compatibility with current `Graph` structure
3. Create adapter layer between `Graph` (SlotMap-based) and `egui_graphs`
4. Migrate rendering logic from `render/mod.rs` to `egui_graphs`
5. Migrate input handling from `input/mod.rs` to `egui_graphs` interaction handlers
6. Preserve existing features: node lifecycle colors, selection, camera transforms
7. Run tests to verify behavior unchanged

**Validation Tests:**
- Graph renders with correct node/edge positions
- Selection (single and multi-select) works
- Dragging nodes updates positions
- Camera zoom and pan work correctly
- Physics simulation still runs and updates positions
- Double-click to focus still works

**Outputs:**
- Simplified `render/mod.rs` (egui_graphs wrapper)
- Simplified `input/mod.rs` (delegate to egui_graphs)
- Adapter module: `graph/egui_adapter.rs`

**Success Criteria:**
- All 84 existing tests pass
- Visual parity with current rendering
- Reduced LOC in rendering/input modules by 50%+

---

### Phase 2: Graph Data Structure (MEDIUM IMPACT)
**Target:** Integrate `petgraph` as a projection layer for algorithms

**Rationale:**
- Current SlotMap approach works well for runtime operations
- `petgraph` provides graph algorithms (pathfinding, centrality, clustering)
- Analysis recommends using it as a "projection" - convert SlotMap → petgraph for analysis, apply results back
- Enables future features: connected components, shortest paths, hub detection

**Tasks:**
1. Add `petgraph` dependency with `serde-1` feature
2. Create conversion: `Graph` → `petgraph::stable_graph::StableGraph`
3. Create conversion: `StableGraph` results → `Graph` node/edge updates
4. Add graph algorithm wrappers in `graph/algorithms.rs`
5. Expose algorithms via `GraphBrowserApp` API

**Validation Tests:**
- Round-trip conversion preserves graph structure
- Pathfinding returns correct shortest paths
- Connected components correctly identifies clusters
- Performance: conversion overhead < 5ms for 100-node graph

**Outputs:**
- `graph/algorithms.rs` module with algorithm wrappers
- `graph/petgraph_adapter.rs` for conversions
- New graph algorithm APIs on `Graph` type

**Success Criteria:**
- Can compute shortest path between any two nodes
- Can identify disconnected subgraphs
- Can detect hub nodes (high centrality)

---

### Phase 3: Split View (HIGH UX IMPACT)
**Target:** Replace exclusive view toggle with `egui_tiles` tiled layout

**Rationale:**
- Current approach: graph OR detail view (exclusive)
- `egui_tiles` enables: graph AND detail view (simultaneous)
- Professional tiling: drag-to-resize, tabs, docking, persistence
- Dramatically improves UX for spatial browsing

**Tasks:**
1. Add `egui_tiles` dependency
2. Replace `View` enum (Graph/Detail) with `egui_tiles::Tree` layout
3. Migrate graph rendering into tile pane
4. Migrate webview rendering into tile pane
5. Preserve keyboard shortcuts (Home to toggle layout)
6. Add tile persistence (save/restore split ratios)

**Validation Tests:**
- Graph and detail view render simultaneously
- Split ratio can be dragged
- Layout persists across restarts
- Physics still runs when graph pane visible
- Webview lifecycle works correctly in split mode

**Outputs:**
- Reworked `desktop/gui.rs` layout using `egui_tiles`
- Layout persistence in `app::SplitViewConfig`
- Updated keyboard shortcuts documentation

**Success Criteria:**
- Can view graph + webpage simultaneously
- Can resize split with mouse drag
- Layout configuration saved/restored
- No framebuffer bleed-through issues

---

### Phase 4: Spatial Optimization (PERFORMANCE)
**Target:** Replace HashMap spatial grid with `kiddo` KD-tree

**Rationale:**
- Current: `HashMap<(i32,i32), Vec<NodeKey>>` (grid-based)
- `kiddo`: Cache-friendly KD-tree, optimized for per-frame rebuild
- Better for non-uniform node distributions
- Enables radius queries for physics (currently hardcoded 300.0 distance)

**Tasks:**
1. Add `kiddo` dependency
2. Replace `SpatialGrid` with `KdTree<f32, NodeKey, 2>`
3. Update physics `step()` to rebuild KD-tree each frame
4. Replace fixed 300.0 radius with configurable physics param
5. Benchmark: compare HashMap vs KD-tree performance

**Validation Tests:**
- Physics behavior unchanged (positions converge identically)
- Neighbor queries return same nodes as before
- Performance: step time < 1ms for 100-node graph
- Memory: KD-tree uses < 2x memory of HashMap

**Outputs:**
- `physics/spatial_kdtree.rs` replacing `spatial_hash.rs`
- Benchmark results in `design_docs/tests/`

**Success Criteria:**
- Physics simulation 10%+ faster for graphs > 50 nodes
- All physics tests pass unchanged

---

### Future Phases (Not Immediate Priority)

**Phase 5: Better Force-Directed Layout**
- `forceatlas2` for Barnes-Hut O(N log N) repulsion
- Better handling of hub-and-spoke topologies
- Only after Servo integration, as demo graph doesn't need this

**Phase 6: Persistence**
- `fjall` (append-only log) + `rkyv` (zero-copy snapshots)
- Critical for production, but lower priority than Servo integration
- See Claude ANALYSIS Phase 1 for detailed architecture

---

## Findings

### Crate Evaluation Matrix

| Crate | Version | Maturity | Maintenance | Servo Compatibility | Notes |
|-------|---------|----------|-------------|---------------------|-------|
| `egui_graphs` | 0.21.0 | Stable | Active (2024) | ✅ egui-compatible | Purpose-built for graph viz |
| `petgraph` | 0.6.5 | Mature | Active | ✅ Pure Rust | 9M+ downloads, de facto standard |
| `egui_tiles` | 0.13.0 | Stable | Active (2025) | ✅ egui plugin | By emilk (egui author) |
| `kiddo` | 4.2.1 | Stable | Active | ✅ Pure Rust | rkyv-compatible, cache-friendly |
| `forceatlas2` | 0.3.0 | Beta | Moderate | ✅ Pure Rust | Barnes-Hut quadtree |

### Trade-offs

**Keeping Custom Code:**
- ✅ Full control over behavior
- ✅ No external API constraints
- ❌ High maintenance burden
- ❌ Missing ecosystem algorithms
- ❌ Reinventing the wheel

**Using Crates:**
- ✅ Battle-tested implementations
- ✅ Access to graph algorithms
- ✅ Professional UX (egui_graphs, egui_tiles)
- ✅ Reduced LOC to maintain
- ❌ Potential API mismatches
- ❌ Dependency on external updates

**Decision:** Proceed with crate integration. Custom physics engine can coexist with `egui_graphs` rendering.

---

## Progress

### Session 1: 2026-02-10

**Completed:**
- ✅ Read Claude ANALYSIS document
- ✅ Read DOC_POLICY
- ✅ Identified crate candidates aligned with analysis
- ✅ Created this plan file per workflow documentation rule

**Next Steps:**
- [ ] User confirmation on which phase to start
- [ ] Add dependencies to Cargo.toml
- [ ] Begin Phase 1 implementation

**Questions for User:**
1. Should we start with Phase 1 (egui_graphs) for biggest impact?
2. Or Phase 3 (egui_tiles) for split view UX improvement first?
3. Or focus on Servo integration (Task 1 from earlier) before crate refactors?

**Test Results:**
- Current baseline: 84 tests passing

---

## Notes

- Custom physics engine is actually quite good (spatial hash grid optimization)
- No need to replace physics immediately - can keep it while using egui_graphs for rendering
- Camera smooth interpolation code is simple and works - no crate needed
- `egui_graphs` was not mentioned in Claude ANALYSIS but is a major opportunity
- Phase order can be flexible - independence between phases allows parallel work
