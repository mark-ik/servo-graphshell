Now I have a thorough understanding of the entire project. Here is a comprehensive analysis of everything needed to implement the full vision as outlined in the design docs.

---

## Analysis: What's Needed to Fully Implement Graphshell

I've read all the design docs and source code. Here's a structured breakdown of everything that needs to be done, organized by the three areas you asked about: **diagnostic mode**, **full servo integration** (no placeholders), and **key priorities**.

---

### 1. Full Servo Integration (Eliminate Placeholder Nodes)

The current state uses 5 hardcoded demo nodes (`app.rs:97-133`) with no actual web rendering. The `webview_to_node` / `node_to_webview` HashMaps exist but are never populated. Here's what needs to happen:

**A. Webview Lifecycle Binding** (the core gap)

- **Create webviews for real pages**: Wire up `WebViewManager::create()` from libservo to actually spawn browser contexts when nodes are created. Currently `GraphBrowserApp` has mapping methods (`map_webview_to_node`, `unmap_webview` at `app.rs:245-268`) but nothing calls them.
- **Load URLs via Servo**: Hook into `WindowMethods::load_url()` so that when a node is created with a URL, a real Servo webview navigates to that page.
- **Destroy webviews on node deletion**: Tie webview destruction to node lifecycle transitions (Active/Warm -> Cold). The `demote_node_to_cold` method (`app.rs:299-310`) cleans up mappings but doesn't actually destroy the Servo webview.
- **Warm pool management**: Implement the 1 Active / 2-4 Warm / rest Cold invariant. The lifecycle enum exists (`graph/mod.rs:64-74`) but no transition logic enforces pool limits.

**B. Navigation Event Hooks**

- **Intercept link clicks**: When a user clicks a link in the Servo webview, graphshell needs to intercept it, create a new node (or navigate to existing one via `graph.get_node_by_url()`), and create a Hyperlink edge. The `SERVO_INTEGRATION_BRIEF.md` outlines a `WindowEventType` enum for this but none of it is implemented.
- **Same-origin vs cross-origin**: Per `GRAPHSHELL_AS_BROWSER.md:178-204`, same-origin navigations should update the current node in-place; cross-origin should create a new node.
- **Title/favicon metadata**: Servo's script process emits title updates and favicon data. These need to be forwarded to the graph layer to update `Node::title` (currently just set to the URL string at `graph/mod.rs:154`).

**C. Detail View Rendering with Real Webviews**

- **Show webview in Detail view**: When `View::Detail(NodeKey)` is active, the corresponding Servo webview needs to be composited into the egui panel. Currently `render_graph` (`render/mod.rs:14-98`) only draws the graph - there's no detail view rendering at all.
- **Input forwarding**: Mouse/keyboard events need to go to Servo when in Detail view instead of to the graph. The input handler (`input/mod.rs:45-115`) always operates on the graph with no branching for view mode.
- **Split view**: The `SplitViewConfig` (`app.rs:26-41`) has `enabled` and `detail_ratio` fields but they aren't used in rendering.

**D. Nodes = Tabs Equivalence**

- **Per-node history stack**: `GRAPHSHELL_AS_BROWSER.md:86-128` specifies `history_stack` and `forward_stack` per DetailView for Back/Forward navigation. None of this exists.
- **Omnibar/address bar**: No URL entry UI exists. The design calls for an omnibar that handles both graph search and direct URL navigation (`GRAPHSHELL_AS_BROWSER.md:306-328`).
- **Bookmarks**: The `BookmarkManager` struct specified in the design (`GRAPHSHELL_AS_BROWSER.md:217-248`) is not implemented.
- **Downloads**: The `DownloadManager` (`GRAPHSHELL_AS_BROWSER.md:257-295`) is not implemented.

---

### 2. Physics Configurable at Runtime

The physics engine (`physics/mod.rs`) has `PhysicsConfig` with hardcoded defaults (`repulsion_strength: 5000.0`, `spring_strength: 0.1`, `damping: 0.92`, `spring_rest_length: 100.0`). What's needed:

- **Runtime parameter UI**: An egui panel or sidebar where users can adjust repulsion, spring strength, damping, rest length, and velocity threshold with sliders. No such UI exists.
- **Propagation to worker thread**: Changes need to be sent to the physics worker via a new `PhysicsCommand::UpdateConfig(PhysicsConfig)` variant. The worker (`physics/worker.rs`) currently only accepts `UpdateGraph`, `Step`, `Toggle`, `Pause`, `Resume`, `UpdateViewport`, and `Shutdown`.
- **Preset system**: The PROJECT_DESCRIPTION mentions "mods" - using others' physics parameters. This requires serialization of `PhysicsConfig` and a load/save mechanism.
- **Per-node/per-edge rules**: The vision document mentions "rule-based node motility" and custom node/edge types affecting physics behavior. Currently all nodes get the same forces.

---

### 3. Diagnostic Mode (Engine Inspector)

This is Feature Target 10, a Phase 3 feature. Here's the full scope from `ARCHITECTURAL_OVERVIEW.md:235-273` and `IMPLEMENTATION_ROADMAP.md:352-368`:

**A. Servo Instrumentation**

- Add `tracing::span!()` calls at thread/channel boundaries in Servo's core components:
  - `components/constellation/` - the thread coordinator
  - `components/script/` - JavaScript engine threads
  - `components/layout/` - layout engine threads
  - IPC channels between these components
- This is invasive - it requires modifying upstream Servo code in `components/`.

**B. Telemetry Collection Layer**

- Subscribe to `tracing` spans from within the graphshell layer
- Collect events: thread creation/destruction, message sends/receives, latency measurements, backpressure indicators
- Buffer telemetry data and build it into a graph structure

**C. Engine Graph Builder**

- Map `ThreadId` -> `Node` (each Servo thread/component becomes a graph node)
- Map `Channel` -> `Edge` (IPC channels become edges, colored by latency/load)
- Track message counts, latencies, and backpressure as edge weights
- This is a fundamentally different graph than the browsing graph - different data source, different semantics

**D. Mode Toggle & UI**

- Hotkey `Ctrl+Shift+D` to switch between browsing graph and engine graph
- Reuse the same graph rendering infrastructure (`render/mod.rs`) with different data
- Possibly split-screen: browsing graph left, engine graph right
- Support SVG export for performance reports (via `visgraph` crate)

**E. Extension Framework**

- Telemetry plugins for custom metrics/visualizations
- Memory profiler view, JS profiler view as additional diagnostic modes
- This is the most speculative part - depends on how the base diagnostic mode works

---

### 4. Key Priorities (Ordered by the Roadmap)

The design docs define a clear critical path. Here's the priority ordering:

**Milestone 1 (M1): Browsable Graph** - The MVP

| Priority | Feature | Current State | Effort |
|----------|---------|--------------|--------|
| **P0** | Servo webview integration | Mapping stubs only, 5 demo nodes | Largest single task |
| **P1** | Thumbnail & favicon rendering | Lifecycle-colored circles only | Depends on webview integration |
| **P2** | Graph persistence | `persistence.rs` is empty stubs | Critical for usability |
| **P3** | Camera zoom integration | `Camera::zoom()` exists, not applied to rendering | Rendering at `render/mod.rs` ignores `camera.zoom` entirely |
| **P4** | Center camera | Keyboard binding exists at `input/mod.rs:25-27`, algorithm missing | Small task |

**Milestone 2 (M2): Usable Browser**

| Priority | Feature | Current State |
|----------|---------|--------------|
| **P5** | Search & filtering (nucleo) | Not started |
| **P6** | Bookmarks/history import | Not started |
| **P7** | Performance optimization (500 nodes @ 45fps) | No profiling done |

**Milestone 3 (M3): Advanced**

| Priority | Feature | Current State |
|----------|---------|--------------|
| **P8** | Clipping (DOM extraction) | Not started |
| **P9** | Diagnostic mode | Not started (requires Servo instrumentation) |
| **P10** | P2P collaboration (Verse) | Research docs only |

---

### 5. Specific Code-Level Gaps

Here are the concrete missing pieces by file:

- **`app.rs`**: No webview creation/destruction logic. No `on_link_clicked()`. No `on_node_selected()` that creates webviews. The `init_demo_graph()` needs to be replaced with real navigation.
- **`render/mod.rs`**: No camera transform applied (zoom is ignored). No Detail view rendering. No thumbnail/favicon rendering. No split view.
- **`input/mod.rs`**: No view-mode branching (graph input vs webview input). No zoom handling (mouse wheel). Center camera (`C` key) is a TODO comment.
- **`input/camera.rs`**: Structure complete but never integrated into the rendering pipeline or input pipeline.
- **`graph/persistence.rs`**: Completely empty stubs. Needs fjall (log) + redb (snapshots) + rkyv (serialization).
- **`physics/worker.rs`**: No `UpdateConfig` command for runtime physics changes.
- **No event_loop integration**: Servo's event loop events aren't being forwarded to the graph layer at all.

---

### Summary

The foundation (~3,500 LOC) is solid: graph data structures, physics simulation, basic rendering, and input handling all work. But the project is essentially a standalone graph visualization tool right now, not a browser. The single largest body of work is **Servo webview integration** - it touches every module and is the prerequisite for thumbnails, navigation, detail view, and eventually diagnostic mode. The diagnostic mode specifically requires upstream Servo instrumentation with `tracing`, a telemetry collection layer, a separate graph builder for engine internals, and a mode-toggle UI - all of which depend on having a working Servo integration first.

ok, let's break this down into little tasks and do them one by one. what's the first five?

Based on the design docs and current code, here are the first five tasks - ordered so each is self-contained and builds toward the larger goals:

| # | Task | Why First | Scope |
|---|------|-----------|-------|
| 1 | **Camera zoom integration** | `Camera::zoom()` exists but rendering ignores it. Unblocks natural graph interaction. | Wire mouse wheel -> camera, apply zoom transform in `render/mod.rs` (~40 lines) |
| 2 | **Center camera (C key)** | Binding exists as a TODO comment. Finishes the camera system. | Calculate bounding box centroid, set target position/zoom (~30 lines) |
| 3 | **Runtime physics config UI** | Physics params are hardcoded. Adds an egui slider panel + `UpdateConfig` command to worker. | New egui panel + one new `PhysicsCommand` variant (~80 lines) |
| 4 | **Detail view rendering scaffold** | Currently only graph view renders. Establishes the node=tab duality. | When `View::Detail`, render a panel with node info (URL, title, lifecycle) instead of graph (~60 lines) |
| 5 | **Split view rendering** | `SplitViewConfig` exists with `enabled` and `detail_ratio` but is never used in rendering. | Partition the viewport so graph and detail coexist side-by-side (~50 lines) |

These are all self-contained, don't require Servo webview plumbing yet, and build the UI foundation that everything else (webview compositing, diagnostic mode toggle, etc.) will plug into.

start with task 1 - wiring up camera zoom.
