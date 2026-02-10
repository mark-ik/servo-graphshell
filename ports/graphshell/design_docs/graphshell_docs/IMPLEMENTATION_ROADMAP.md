# Graphshell Implementation Roadmap

**Document Type**: Feature-driven implementation plan  
**Organization**: By feature targets with validation tests (not calendar time)  
**Last Updated**: February 10, 2026  
**Priority Focus**: Top 5 features for Phase 1: Core Browsing Graph (Servo integration → thumbnails → persistence → zoom → camera)  

---

## Current State

**Foundation Complete** (~3,500 LOC):
- ✅ Graph data structures (SlotMap, Node, Edge, lifecycle)
- ✅ Force-directed physics engine (spatial hash, springs, damping)
- ✅ Physics worker thread (crossbeam channels, non-blocking)
- ✅ egui rendering (nodes, edges, labels, lifecycle colors)
- ✅ Mouse/keyboard input (drag, select, toggle physics/view)
- ✅ Camera structure (pan, zoom, smooth interpolation)
- ✅ View model (Graph/Detail toggle, split-view config)

**Critical Gaps**:
- ❌ Servo webview integration (create/destroy, navigation, thumbnails)
- ❌ Graph persistence (save/load, crash recovery)
- ❌ Camera zoom rendering (not applied to egui transform)

**Status**: Foundation ready for Servo integration sprint.

---

## Phase 1: Core Browsing Graph

### 🔥 Priority: Top 5 Features for Browsable Graph

These five features enable the core MVP: **users can browse real websites in a spatial graph that persists and feels responsive**.

| # | Feature | Duration | Why First |
|---|---------|----------|-----------|
| 1 | Servo Webview Integration | ~2 weeks | Can't browse without actual webviews |
| 2 | Thumbnail & Favicon Rendering | ~1 week | Spatial memory depends on visual recognition |
| 3 | Graph Persistence | ~1 week | Users lose all work on crash (critical) |
| 4 | Camera Zoom Integration | ~3 days | Half-implemented, completes user interaction |
| 5 | Center Camera | ~1 day | Finishes camera control (C key handler) |

**After these 5**: Search (Feature 6), Bookmarks import (Feature 7), Performance (Feature 8), Clipping (Feature 9), Diagnostics (Feature 10), P2P (Feature 11)

---

### Feature Target 1: Servo Webview Integration

**Goal**: Users can browse real websites, and each page becomes a node in the graph.

**Context**: Current demo uses 5 static nodes. Need to integrate Servo's webview management to create/destroy nodes based on actual browsing.

**Tasks**:
1. Study servoshell's WebViewManager and WindowMethods API
2. Create WebViewId for each Active node (lifecycle management)
3. Hook navigation events to create new nodes on link clicks
4. Implement node-to-webview lifetime binding (destroy webview when node becomes Cold)
5. Show/hide webviews based on View state (Detail vs. Graph)
6. Handle multiple webviews (1 Active, pool of 2-4 Warm, rest Cold)

**Validation Tests**:
- [ ] Load https://example.com → creates first node
- [ ] Click link in webview → creates second node + edge (Hyperlink type)
- [ ] Switch to Graph view → webview hidden, graph visible
- [ ] Double-click node → Detail view, correct webview shown
- [ ] Close node (delete) → webview destroyed, no crashes
- [ ] 10 nodes open → only 1-5 webviews in memory (lifecycle working)

**Outputs**:
- Modified `app.rs`: webview creation/destruction logic
- Modified `desktop/event_loop.rs`: navigation event hooks
- Integration tests demonstrating webview-node lifecycle

**Success Criteria**:
- Can browse real websites and see graph grow organically
- Memory usage scales with lifecycle (not unbounded)
- No webview leaks after deleting nodes

---

### Feature Target 2: Thumbnail & Favicon Rendering

**Goal**: Nodes show recognizable visuals: page thumbnail (best), site favicon (fallback), or lifecycle color (final fallback).

**Context**: Currently rendering circles with lifecycle colors only. Spatial memory benefits from site/page recognition.

**Rendering Priority** (tier-based approach):
1. **Page thumbnail** (256x192) — Full page preview, best spatial memory aid, Active/Warm nodes
2. **Site favicon** (32x32) — Standard browser favicon, lightweight, works for all lifecycle states
3. **Colored circle** — Lifecycle color only (current state, final fallback)

**Tasks**:
1. Favicon fetch & caching:
   - Fetch `/favicon.ico` or parse `<link rel="icon">` from page metadata
   - Cache favicon in Node struct (Option<egui::TextureHandle>)
   - Fetch asynchronously, non-blocking (parallel with page load)

2. Thumbnail capture:
   - Capture webview frame after page load (use `webrender::screen_capture` or `glow::Context::read_pixels()`)
   - Downscale to 256x192 using `image::imageops::resize()`
   - Store in Node struct (Option<egui::TextureHandle>)

3. Node rendering logic in `render/mod.rs`:
   ```rust
   fn render_node(node: &Node) {
       if let Some(thumbnail) = &node.thumbnail {
           // Render 256x192 thumbnail as rounded rect
       } else if let Some(favicon) = &node.favicon {
           // Render 32x32 favicon centered
       } else {
           // Render colored circle (lifecycle color)
       }
   }
   ```

4. Cache management to avoid re-captures

**Validation Tests**:
- [ ] Load page → favicon appears immediately (cached within 500ms)
- [ ] Favicon is recognizable (GitHub octocat, Google colors, etc.)
- [ ] Load page → thumbnail appears within 2 seconds (doesn't block UI)
- [ ] Thumbnail matches page content (visually recognizable)
- [ ] Cold node → shows favicon (no thumbnail capture)
- [ ] Node becomes Active → thumbnail updates if URL changed
- [ ] 50 nodes with mixed thumbnails/favicons → render at 60fps (performance check)
- [ ] Favicon fetch fails gracefully → falls back to circle

**Outputs**:
- Favicon fetch code in `app.rs` (async loader)
- Thumbnail capture code in `app.rs` or `render/mod.rs`
- Updated Node struct with `thumbnail: Option<egui::TextureHandle>` and `favicon: Option<egui::TextureHandle>`
- Rendering logic in `render/mod.rs` with three-tier fallback
- Visual regression tests comparing renders to expected output

**Success Criteria**:
- Favicons available immediately on most sites (80%+ within 500ms)
- Thumbnails recognizable within 2 seconds (90%+ sites load that fast)
- No performance degradation (<10% fps drop even with 100 mixed nodes)
- Graceful fallback when favicon/thumbnail unavailable (circle always renders)

---

### Feature Target 3: Graph Persistence (Snapshots + Log)

**Goal**: Save graph on shutdown, restore on startup, survive crashes with minimal data loss.

**Context**: `graph/persistence.rs` has empty stubs. Critical for usability (users lose all work on crash).

**Tasks**:
1. Choose persistence crates: **fjall** (append log), **redb** (snapshots), **rkyv** (serialization)
2. Implement append-only log: write every mutation (add_node, add_edge, delete_node, update_position)
3. Implement periodic snapshots (every 5 minutes or on clean shutdown)
4. Implement startup recovery: load latest snapshot + replay log since snapshot
5. Handle corrupted log gracefully (truncate to last valid entry)
6. Add LogEntry enum (AddNode, AddEdge, DeleteNode, UpdatePosition, UpdateMetadata)
7. Test crash scenarios (mid-write, corrupted file, missing snapshot)

**Validation Tests**:
- [ ] Browse 10 pages → shutdown → restart → all 10 nodes restored
- [ ] Browse 5 pages → simulate crash (kill -9) → restart → 5 nodes restored (or 4-5 if crash mid-mutation)
- [ ] Corrupted log file → startup succeeds, restores to last valid state
- [ ] No snapshot file → creates new graph (empty start)
- [ ] 1000 operations (adds/deletes) → log size < 10MB

**Outputs**:
- Full implementation of `graph/persistence.rs` (~300 lines)
- Log format specification (document LogEntry enum)
- Integration tests for crash recovery scenarios

**Success Criteria**:
- Zero data loss on clean shutdown
- <1 second of operations lost on crash (acceptable trade-off)
- Startup time <500ms for typical graph size (50-200 nodes)

---

### Feature Target 4: Camera Zoom Integration

**Goal**: Mouse wheel zooms in/out, graph scales accordingly, smooth interpolation.

**Context**: Camera::zoom() exists but not applied to egui rendering.

**Tasks**:
1. Hook mouse wheel events in `input/mod.rs` (call `Camera::zoom(wheel_delta)`)
2. Apply zoom transform in `render/mod.rs`: `screen_pos = (world_pos - camera.offset) * camera.zoom`
3. Test zoom range (0.1x to 10x, clamped)
4. Smooth interpolation already implemented (camera.update(dt) lerps target_zoom → zoom)
5. Center zoom on cursor position (not on graph origin)

**Validation Tests**:
- [ ] Scroll up → graph zooms in, centered on cursor
- [ ] Scroll down → graph zooms out, centered on cursor
- [ ] Zoom range clamps at 0.1x (very zoomed out) and 10x (very zoomed in)
- [ ] Zoom is smooth (no jitter, lerp working)
- [ ] Zoom + pan → coordinates remain consistent

**Outputs**:
- Modified `input/mod.rs`: wheel event handling
- Modified `render/mod.rs`: apply zoom transform to all painter calls
- Unit tests for coordinate transform correctness

**Success Criteria**:
- Zoom feels natural (similar to map apps)
- No coordinate drift or jitter
- Performance: 200 nodes @ 60fps even at 10x zoom

---

### Feature Target 5: Center Camera on Graph

**Goal**: Press `C` key → camera smoothly moves to show all nodes (auto-fit).

**Context**: Keyboard binding exists, algorithm missing.

**Tasks**:
1. Calculate graph bounding box (min/max x/y across all nodes)
2. Calculate center point: `(max + min) / 2`
3. Set `Camera::target_position` to center point
4. Optional: adjust zoom to fit all nodes in viewport (calculate required zoom based on bounding box diagonal vs viewport diagonal)
5. Smooth interpolation already handles animation (Camera::update())

**Validation Tests**:
- [ ] Press C → camera moves to show all nodes
- [ ] Empty graph → C does nothing (or moves to origin)
- [ ] Widely spread graph (1000px span) → C zooms out to fit
- [ ] Press C multiple times → always returns to same center (deterministic)

**Outputs**:
- Implementation in `input/mod.rs` or `camera.rs` (~30 lines)
- Unit test verifying centroid calculation for various graph shapes

**Success Criteria**:
- Always shows entire graph after pressing C
- Smooth animation (no instant snap)
- Works for graphs of any size (10 nodes or 1000 nodes)

---

## Phase 2: Usability & Polish

### Feature Target 6: Search & Filtering

**Goal**: Press Ctrl+F → search bar appears → type URL/title → matching nodes highlighted.

**Tasks**:
1. Add search bar UI (egui TextEdit)
2. Implement fuzzy string matching (use **nucleo** crate, fzf-like scoring)
3. Highlight matching nodes (different color or pulsing border)
4. Support filtering: show only matching nodes, hide others
5. Navigate search results: Up/Down arrows cycle through matches

**Validation Tests**:
- [ ] Search "example" → nodes with "example" in URL or title highlighted
- [ ] Fuzzy matching works: "gthub" matches "github.com"
- [ ] Empty search → all nodes visible (no filter)
- [ ] Search + filter mode → only matching nodes shown
- [ ] Esc clears search

**Outputs**:
- Search UI in `render/mod.rs` or new `search.rs` module
- Integration with nucleo crate
- Keybind for Ctrl+F

**Success Criteria**:
- Find nodes quickly in large graphs (200+ nodes)
- Fuzzy matching feels intuitive (fzf-like behavior)
- No performance degradation during search

---

### Feature Target 7: Bookmarks & History Import

**Goal**: Import browser bookmarks/history → seed initial graph structure.

**Tasks**:
1. Parse Firefox/Chrome bookmark format (JSON/HTML)
2. Create node for each bookmark
3. Create edges based on folder structure (parent-child)
4. Parse browser history database (SQLite)
5. Create nodes for recent history entries
6. Create edges based on referrer chains

**Validation Tests**:
- [ ] Import 100 bookmarks → creates 100 nodes
- [ ] Bookmark folders become node clusters (edges connecting folder contents)
- [ ] Import history → creates nodes for frequently visited sites
- [ ] Duplicate URLs merged (same node for bookmark + history entry)

**Outputs**:
- Import UI (file picker or auto-detect browser data directories)
- Parser for bookmark/history formats
- Migration guide for users switching from traditional browsers

**Success Criteria**:
- Successfully imports bookmarks from major browsers
- Graph structure reflects bookmark organization
- No data loss during import

---

### Feature Target 8: Performance Optimization

**Goal**: 500 nodes @ 45fps, 1000 nodes @ 30+fps (usable), smooth interaction.

**Tasks**:
1. Profile rendering: identify bottlenecks (egui Painter calls)
2. Batch node rendering (single draw call for all circles)
3. Cull off-screen nodes (simple rect test, skip if out of viewport)
4. LOD (level-of-detail): cluster distant nodes when zoomed out
5. Measure physics simulation time (ensure <16ms per frame)
6. Optimize spatial hash grid (tune cell size, benchmark neighbor queries)

**Validation Tests**:
- [ ] 500 nodes → 45fps maintained during pan/zoom
- [ ] 1000 nodes → 30+fps, usable (slight lag acceptable)
- [ ] Profile confirms no single hotspot >50% frame time
- [ ] Memory usage scales linearly (no leaks)

**Outputs**:
- Performance benchmarks (flamegraphs, timing data)
- Optimized rendering code
- LOD system (if needed)

**Success Criteria**:
- Meets performance targets without fallback UI
- Smooth interaction even with large graphs
- If targets not met: implement fallback (cluster strip view)

---

## Phase 3: Advanced Features

### Feature Target 9: Clipping (DOM Element Extraction)

**Goal**: Right-click element in webpage → "Clip to Graph" → element becomes independent node.

**Tasks**:
1. Expose Servo's DOM inspector API to GraphShell
2. Implement right-click context menu in webview
3. Capture clicked element's rendered output (screenshot or HTML snapshot)
4. Create new node with clipped content
5. Store DOM path or HTML in node metadata (for updating)

**Success Criteria**:
- Can clip images, text blocks, or entire sections
- Clipped nodes remain linked to source page
- Updates if source page modified (optional)

---

### Feature Target 10: Diagnostic/Engine Inspector Mode

**Goal**: Toggle mode to visualize Servo's internal architecture (Constellation, threads, IPC channels).

**Tasks**:
1. Instrument Servo with `tracing::span!()` at thread/channel boundaries
2. Collect trace events in GraphShell layer
3. Build dynamic graph: ThreadId → Node, Channel → Edge
4. Visualize message counts, latencies, backpressure as edge weights/colors
5. Implement mode toggle (Ctrl+Shift+D)
6. Support export as SVG for performance reports

**Success Criteria**:
- Developers can see real-time thread activity
- Users understand what browser is doing (transparency/education)
- Identifies performance bottlenecks visually

---

### Feature Target 11: P2P Collaboration (Verse)

**Goal**: Share graph with peers, real-time co-browsing, permissions-based access.

**Tasks**:
- (Deferred to design_docs/verse_docs/)
- Requires: IPFS integration, tokenization, CRDTs for sync

---

## Implementation Notes

**Dependency Recommendations** (from checkpoint analyses):

| Feature | Recommended Crate | Why |
|---------|-------------------|-----|
| Persistence snapshots | **redb** | Pure Rust KV store, ACID, faster than sled |
| Persistence log | **fjall** | LSM-tree append log, ACID, write-optimized |
| Serialization | **rkyv** | Zero-copy, fastest Rust serialization |
| Search (fuzzy) | **nucleo** | By helix-editor, fzf-like, 6x faster than fuzzy-matcher |
| Search (full-text) | **tantivy** | Lucene equivalent, BM25, for future content indexing |
| Easing (camera) | **simple_easing** or egui's built-in | Pure functions, keyframe is abandoned |

**Avoid**:
- `sled` — Stuck in beta since 2021, known corruption bugs
- `keyframe` — Abandoned (no activity since 2022)
- `RocksDB` — Massive C++ dependency, overkill

**Build Time Tracking**:
- Clean build: ~15-30 min (depends on machine)
- Incremental: ~30s-2min (typical code change)
- Release build: +20% time vs debug

---

## Success Milestones

**M1: Browsable Graph** (Feature Targets 1-5)
- Can browse real websites
- Graph persists across sessions
- Basic navigation (zoom, pan, center)
- Thumbnails for spatial recognition

**M2: Usable Browser** (Feature Targets 6-8)
- Search/filter works
- Performance acceptable (500 nodes)
- Bookmarks import seeded graph

**M3: Advanced** (Feature Targets 9-11)
- Clipping extracts DOM elements
- Diagnostic mode visualizes engine
- P2P collaboration (if Verse phase reached)

**Validation Gates**:
- After M1: User testing with real browsing workflows
- After M2: Performance benchmarks vs traditional browsers (memory, speed)
- After M3: Community feedback, public release decision

---

## Risk Mitigation

**Risk**: Servo integration harder than expected (API unstable, threading issues)  
**Mitigation**: Start with single webview, expand gradually; fallback to embedded browser (WebView2/WebKitGTK) if Servo proves too difficult

**Risk**: Performance targets not met (500 nodes too slow)  
**Mitigation**: Implement LOD (level-of-detail clustering), fallback to timeline strip + graph as optional view

**Risk**: User adoption low (spatial UX confusing)  
**Mitigation**: Extensive user testing after M1, willingness to pivot UI based on feedback

**Risk**: Persistence bugs cause data loss  
**Mitigation**: Comprehensive crash recovery tests, backup strategy (snapshots stored in multiple locations)

---

## References

- **Checkpoint Analyses**: `archive_docs/checkpoint_2026-02-09/`
- **Project Vision**: `PROJECT_DESCRIPTION.md`
- **Architecture**: `ARCHITECTURAL_OVERVIEW.md`
- **Code**: `c:\Users\mark_\Code\servo\ports\graphshell\`
