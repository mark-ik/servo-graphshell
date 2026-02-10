# Graphshell Architectural Overview

**Last Updated**: February 9, 2026  
**Status**: Foundation implemented, Servo integration in progress  

---

## Project Vision

Graphshell is a **spatial browser** where webpages are nodes in a force-directed graph instead of tabs in a bar. Users navigate by seeing and interacting with the topology of their browsing session.

**Core Idea**: Replace linear history with spatial memory. Instead of "Back/Forward," you see where you came from and where pages link to.

---

## Current Implementation Status

### ✅ Foundation Complete (~3,500 LOC)

**Graph Core** (`graph/mod.rs`, 271 lines)
- `Graph`: SlotMap-based storage for stable node/edge handles
- `Node`: URL, title, position, velocity, selection state, lifecycle (Active/Warm/Cold)
- `Edge`: Connections with types (Hyperlink, Bookmark, History, Manual) and styles
- `NodeKey`/`EdgeKey`: Stable identifiers that survive deletions
- URL-to-NodeKey HashMap for O(1) lookup

**Physics Engine** (`physics/mod.rs`, 209 lines)
- Force-directed layout: repulsion (spatial hash O(n)), spring attraction (Hooke's law), damping
- Auto-pause on convergence (monitors max velocity, pauses after 5s below threshold)
- Configurable parameters: repulsion 5000.0, spring 0.1, damping 0.92, rest length 100px
- Spatial hash grid for efficient neighbor queries (~300px radius)

**Physics Worker** (`physics/worker.rs`, 150 lines)
- Background thread using `crossbeam_channel` for non-blocking simulation
- Commands: UpdateGraph, Step, Toggle, Pause, Resume, UpdateViewport, Shutdown
- Responses: NodePositions (HashMap updates), IsRunning status
- Runs at 60 FPS target, sends position updates back to main thread

**Rendering** (`render/mod.rs`, 145 lines)
- egui immediate-mode UI
- Draws nodes (circles, lifecycle-based colors/sizes) and edges (straight lines, typed/styled)
- Labels truncated to 20 chars below each node
- Selection highlighting (yellow), pinned nodes (red border)
- Dark background (RGB 20, 20, 25)

**Input** (`input/mod.rs`, ~150 lines)
- **Mouse**: Click select, Shift multi-select, drag nodes, pan graph (when not dragging node), double-click to focus (switch to Detail view)
- **Keyboard**: `T` toggle physics, `Home`/`Esc` toggle view, `C` center camera (TODO)
- Pauses physics while user is interacting (dragging)

**Camera** (`input/camera.rs`, ~60 lines)
- Pan and zoom with smooth interpolation (lerp factor 10.0 * dt)
- Structure complete, but **zoom not integrated** into egui rendering transform yet

**Application** (`app.rs`, 332 lines)
- View model: `View::Graph` or `View::Detail(NodeKey)`
- Split view config (enabled, detail_ratio 0.6)
- Bidirectional webview↔node mapping: `HashMap<WebViewId, NodeKey>` and inverse
- Selection management (single/multi), focus switching
- Demo graph initialization (5 static nodes)

### ⚠️ Partial / Needs Integration

**Servo Integration**
- Webview lifecycle structs defined (Active/Warm/Cold)
- Mapping structures exist
- **Missing**: Actual Servo webview creation, destruction, thumbnail capture, navigation hooks

**Camera Zoom**
- `Camera::zoom()` method exists and updates `target_zoom`
- **Missing**: Integration with egui coordinate transform (not applied in rendering yet)

**Center Camera**
- Keyboard binding exists (C key)
- **Missing**: Algorithm to calculate graph centroid and move camera

### ❌ Not Implemented

**Critical for MVP:**
1. **Graph Persistence** (`graph/persistence.rs` - empty stubs)
   - save_snapshot() and load_graph() marked "Week 6 milestone"
   - Need: Snapshot + append-only log strategy for crash recovery

2. **Servo Webview Integration**
   - Create webview for each node
   - Capture thumbnails after page load
   - Handle navigation events (create new nodes on link clicks)
   - Manage webview lifecycle (Active → Warm → Cold transitions)

3. **Thumbnail Display**
   - Render captured page thumbnails instead of colored circles
   - Fallback to colored circles for Cold nodes

**Planned Features** (from PROJECT_DESCRIPTION, all unimplemented):
- Clipping (DOM element extraction)
- Search/filtering
- Lasso zoning
- 2D/3D canvas switching
- Minimap
- Export (JSON, DOT, interactive HTML)
- Bookmarks/history import
- Undo/redo
- Mods/plugins
- P2P collaboration (Verse)

---

## Architecture Decisions

### Data Structures

**Why SlotMap?**
- Stable handles across deletions (NodeKey remains valid)
- O(1) access by key
- Memory efficient (no heap fragmentation from Vec holes)
- Better than `HashMap<UUID, Node>` (no hashing overhead, tighter packing)

**Why Separate Graph/Edge Storage?**
- Allows O(1) edge lookup by EdgeKey
- Enables edge metadata (style, color, creation time)
- Nodes store edge lists (in_edges, out_edges) for quick traversal

**Why URL-to-NodeKey HashMap?**
- Fast duplicate detection: "Does this URL already have a node?"
- Prevents creating duplicate nodes when user revisits pages

### Physics & Performance

**Why Spatial Hash?**
- Repulsion is O(n²) naive (check every pair)
- Spatial hash reduces to O(n) average case (only check nearby nodes within ~300px)
- Cell size = viewport_diagonal / 4.0 (auto-scales to screen size)

**Why Worker Thread?**
- Physics simulation can't block rendering (60 FPS requirement)
- Worker runs physics in parallel, sends position updates via channel
- Main thread applies updates only when available (doesn't wait)

**Why Auto-Pause?**
- Graph converges to stable state after ~5-10 seconds
- Unnecessary to keep simulating when max velocity < 0.001 px/frame
- Saves CPU, extends battery life
- Resumes on any interaction (drag, new node, toggle)

### Rendering & UI

**Why egui?**
- Immediate-mode: no state synchronization between data and UI
- Fast iteration: code changes reflect instantly
- Already integrated with Servo's servoshell (used for debug UI)
- Cross-platform (Win/Linux/macOS/Android/OpenHarmony) out of the box
- No HTML/CSS overhead for graph UI

**Why Straight Edges (Not Bundled)?**
- Simple to implement (Week 1-4)
- Readable for small graphs (<100 nodes)
- Upgrade to bundled edges if needed in Week 9 (performance validation gate)

**Why Lifecycle Colors?**
- Active (blue): User knows webview is running
- Warm (purple): Hint that process is alive but hidden
- Cold (gray): Visual cue that clicking will take time (needs to spawn process)

### View Model

**Why Graph/Detail Toggle (Not Split by Default)?**
- Maximizes graph space (most important UX)
- Users can enable split view in settings if desired (detail_ratio 0.6)
- Keyboard-driven workflow: Esc returns to graph, double-click focuses node

**Why Bidirectional Mapping?**
- Node → WebViewId: "Which webview renders this node?"
- WebViewId → NodeKey: "Which node owns this webview?" (for navigation events)
- Enables reuse: Warm nodes can have their webview reassigned to a different node

---

## Key Remaining Challenges

### 1. Servo Integration

**Problem**: Servo's API expects traditional tab-based UI.

**Solution**:
- Use servoshell's EventLoop + WindowMethods as foundation
- Create multiple WebViewId instances (one per Active node, pool of 2-4 for Warm)
- Hide/show webviews based on node focus (only render Active/focused node)
- Hook `send_window_event()` to detect navigation → create new nodes

**Open Questions**:
- How to capture thumbnails? (use `webrender::screen_capture` after layout completes)
- How to detect link clicks? (intercept navigation events, create edge + new node)
- How to manage memory with 50+ webviews? (use lifecycle: keep 1 Active, 3-5 Warm, rest Cold)

### 2. Graph Persistence

**Recommended Approach** (from checkpoint analyses):
- **fjall**: Append-only operation log (LSM-tree, ACID, pure Rust)
- **redb**: Snapshot storage (KV store, faster than sled, ACID transactions)
- **rkyv**: Zero-copy serialization (fastest, mmap-friendly)

**Architecture**:
```
[Runtime] --serialize--> [fjall log] (every mutation)
          --snapshot-->  [redb] (every 5 min or shutdown)
[Startup] --load snapshot--> --replay log--> [Recovered Graph]
```

### 3. Camera Zoom Rendering

**Problem**: `Camera::zoom()` updates `target_zoom` but egui rendering doesn't apply transform.

**Solution**: In `render_graph()`, wrap painter calls in coordinate transform:
```rust
let zoom = app.camera.zoom;
let offset = app.camera.position;
// Transform all positions: screen_pos = (world_pos - offset) * zoom
```

### 4. Performance Validation (Week 9 Gate)

**Target**: 200 nodes @ 60fps, 500 @ 45fps, 1000 @ 30+fps (usable)

**If Not Met**:
- Upgrade to bundled edges (reduce overdraw)
- LOD (level-of-detail): cluster distant nodes, expand on zoom
- Cull off-screen nodes (simple rect test)
- Batch rendering (draw all nodes in single call)

**Fallback Plan**:
- If spatial UX doesn't scale, pivot to "cluster strip" (groups along timeline) + graph as optional view

---

## Diagnostic Mode (Engine Inspector) - Phase 3+ Feature

**Concept** (from Gemini analysis): Visualize Servo's internal architecture (Constellation, threads, IPC channels) as a graph.

**Use Cases**:
- **Developers**: Debug Servo performance, identify bottlenecks, trace message flow
- **Users**: Understand browser internals, educational tool, transparency

**Architecture**:
- **Instrumentation**: Add `tracing` spans to Servo's constellation, script, layout threads
- **Graph Model**: Nodes = threads/components, Edges = IPC channels (color by latency/load)
- **UI**: Same graph rendering, different data source (switch mode with hotkey)
- **Extension Framework**: Telemetry plugins can add custom metrics/visualizations

**Technical Approach**:
1. Instrument Servo with `tracing::span!()` at thread/channel boundaries
2. Collect events in GraphShell layer (subscribe to tracing spans)
3. Build dynamic graph: `ThreadId → Node`, `Channel → Edge`
4. Visualize message counts, latencies, backpressure as edge weights/colors
5. Support snapshots for performance reports (export SVG via `visgraph`)

**Relationship to Browsing Graph**:
- **Separate mode** (toggle with hotkey, e.g., Ctrl+Shift+D)
- Different graph structure (not webpages, but engine components)
- Shares rendering/physics infrastructure (reuse graph UI code)
- Could run simultaneously: split-screen with browsing graph on left, engine graph on right

**Benefits**:
- Educational: See how a browser engine works in real-time
- Debugging: Identify slow threads, stuck processes, IPC bottlenecks
- Transparency: Users understand what their browser is doing
- Extensibility: Framework for other diagnostic plugins (memory profiler, JS profiler)

**Phase 3 Milestone** (Weeks 13-16):
- Add tracing instrumentation to key Servo components
- Implement mode toggle in GraphShell
- Create engine graph builder (ThreadId/Channel → Node/Edge)
- Demonstrate basic visualization (threads + message counts)

---

## References

**Codebase**:
- `ports/graphshell/` — Main implementation (~3,500 LOC)
- `ports/servoshell/` — Base shell (windowing, event loop, WebRender)
- `components/constellation/` — Servo's coordinator (upstream)

**Key Crates**:
- `slotmap` — Graph storage
- `euclid` — 2D geometry
- `egui` — Immediate-mode UI
- `crossbeam` — Physics worker channels
- `servo` — Browser engine (libservo)

**Checkpoint Analyses**:
- `archive_docs/checkpoint_2026-02-09/Claude ANALYSIS 2.9.26.md` — Codebase audit & recommendations
- `archive_docs/checkpoint_2026-02-09/Gemini Graphshell Analysis 2.9.26.md` — Engine visualization concept
- `archive_docs/checkpoint_2026-02-09/GRAPHSHELL_CHANGELOG.md` — Commit history
