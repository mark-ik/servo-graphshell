# Archived: Graph Browser Migration

This migration plan has been archived. See the archived copy below for the original full migration notes.

Archived copy: [archive_docs/GRAPH_BROWSER_MIGRATION.md](archive_docs/GRAPH_BROWSER_MIGRATION.md)

For the current consolidated interaction and UI details, see [GRAPH_INTERFACE.md](GRAPH_INTERFACE.md). For high-level project overview, see [README.md](README.md).

## Updated implementation strategy (supersedes details below)

The high-level UX and data model in this document still apply, but the **implementation approach is now more conservative**:

- **Single application crate for Phase 1**: keep graph, camera, UI, and Servo integration in one crate, but design `graph` and related modules so they do not depend on Servo or UI types. Extracting `graphshell-graph-core` into a separate crate is a Phase 2+ refactor once the MVP is stable.
- **Reuse existing compositor/WebRender**: the Phase 1 graph canvas is rendered using the existing Servo/WebRender compositor. A dedicated `wgpu` renderer and fully pluggable backends are Phase 2+ work.
- **Small webview pool, not one-per-node**: nodes always exist as URL/metadata in the graph. A small pool of Servo webviews is created lazily and rebound to focused/visible nodes rather than allocating a webview for every node.
- **Simpler window layout in Phase 1**: the MVP uses a single main window with a graph canvas plus a docked detail panel (top/bottom or left/right). The "Windows 11 snap-like" multi-window layout is a Phase 2 feature.
- **Pragmatic physics targets**: Phase 1 tunes the force-directed layout for roughly 20–50 nodes without quadtree/spatial partitioning; heavier optimizations are introduced only when profiling shows they are needed.

The detailed phase breakdowns and module lists below should be read as **aspirational** and are subject to these updated constraints.

- **Servo integration**: Rendering engines per node (already proven with tabs)
- **Config/CLI** system: argument parsing, resource loading
- **Window management**: glutin/winit event loop, keyboard/mouse input
- **Download/storage**: persistence layer, file management
- **Core types**: WebViewId, PipelineId (Servo identifiers)

### What Changes 🔄
| Current | New |
|---------|-----|
| Main window with tab bar | Graph canvas + detail window with connection tabs |
| Tab selection = document navigation | Node click opens detail window; tabs show connections |
| Linear tab ordering | Graph canvas layout (force-directed); tab order = chronological |
| Tab menu bar | Detail window header (node title); connection tabs (edges) |
| Browser chrome (URL bar, buttons) | Heads-up display (HUD) on graph canvas |
| Single focused webview | Node-focused webview in detail window |
| Tab coordinates fixed | Node coordinates physics-driven on canvas |
| Tab metadata minimal | Edge metadata rich (created_at, type, weight) |

### What's New ✨
- **Physics engine**: Spring forces, repulsion, damping
- **Camera system**: Pan/zoom with WASD + mouse
- **Graph operations**: Add/remove nodes, create edges, grouping
- **Metadata**: Node-specific title, favicon, scroll position
- **Serialization**: Save/load graph structure as JSON
- **3D rendering**: Optional stacked or full 3D canvas
- **Graph chrome**: Menu bar, toolbar, HUD, search, stats panel, settings

---

## Main Graph UI Layout

```
┌──────────────────────────────────────────────────────────────┐
│ [Search: _________________] [⚙]  [≡]                         │ ← Chrome (omnibar, settings, menus)
├──────────────────────────────────────────────────────────────┤
│                                                               │
│                   GRAPH CANVAS (Home View)                   │ ← Primary view
│                   Force-directed nodes & edges               │   Opens on app launch
│                   (WASD pan, scroll zoom)                    │   Graph structure affects all UI
│                   Click node → snap detail window            │
│                                                               │
│              [FPS: 60] [Nodes: 12] [Edges: 8]                │ ← Status bar overlay
│              [Zoom: 1.0x]                                    │
└──────────────────────────────────────────────────────────────┘
```

**Multi-window layout example** (Windows 11 Snap-like):
```
┌──────────────────────────────────────────────────────────────┐
│ [Search: _______________] [⚙] [≡]                           │
├──────────────────┬──────────────────────────────────────────┤
│                  │                                           │
│  GRAPH CANVAS    │  DETAIL WINDOW (Node A)                  │
│  (Primary)       │  ┌─────────────────────────┐             │
│                  │  │ A | B | C |             │ ← Tabs      │
│                  │  ├─────────────────────────┤ (edges from │
│                  │  │ (Webview content)       │  graph)     │
│                  │  │                         │             │
│                  │  └─────────────────────────┘             │
│                  │                                           │
└──────────────────┴──────────────────────────────────────────┘
```

### Chrome (Minimal, Top Bar)
- **Search omnibar**: Live search/filter nodes by title, URL, tags, metadata
  - Search results highlight nodes on canvas or in open detail windows
  - Quick nav: type URL or node name, press Enter to create node or navigate
- **Settings button [⚙]**: Opens settings dialog
  - Physics tuning (damping, spring constant, repulsion)
  - Detail window behavior (auto-snap, layout mode, tab ordering)
  - Chrome visibility (show/hide HUD, minimap)
  - Graph appearance (node size, edge colors, icons)
  - Persistence (auto-save interval)
- **Menu button [≡]**: Opens dropdown menu (evolves as project grows)
  - Phase 1: File (New, Open, Save, Export), Help (Keybinds, About)
  - Phase 2: Edit (Undo, Redo, Select All), View (Zoom, Reset, 3D)
  - Phase 3: Tools (Physics analysis, Clustering), Network (Sync, Share)

### Detail Window
- **Tabs at top**: List edges (connections) to/from the focused node, ordered chronologically
  - **Tab labels**: Adjacent node titles + favicons
  - **Tabs change based on graph structure**: If you delete an edge, tab disappears; if you add one, new tab appears
  - **Tab clicking**: Navigate to adjacent node (detail window updates in-place or slides content)
- **Content area**: Servo webview rendering node's URL/content

### Status Bar (Optional HUD overlay, bottom-left)
- **FPS**: Frames per second
- **Node count**: Total nodes
- **Edge count**: Total edges
- **Zoom level**: Current camera zoom
- (Optional) **Selected node name**, **Camera position**

### Window Snapping & Multi-Window Management
- **All windows are child windows** of the main application (not independent floating windows)
- **Snap layouts** (Windows 11-like):
  - Click node in graph → Detail window appears (snaps to right side by default)
  - Click another node → Can open second detail window (snaps to right side split 50/50, or stacked below)
  - Manual resize/reposition: Drag window edges to resize or drag title to reposition within snap grid
  - Click "Snap" control on window header to cycle through snap positions (left 50%, right 50%, center, etc.)
- **Keyboard shortcuts** for window management:
  - **Win + Left/Right arrow**: Snap detail window to left/right (Windows 11 convention)
  - **Ctrl + W**: Close detail window
  - **Ctrl + Tab**: Cycle between open detail windows
- **Window hierarchy**: 
  - Main graph window is always visible (can't be closed)
  - Detail windows are children; closing main app closes all windows
  - Detail windows are maximizable/minimizable but always within main window bounds

### Graph as Home Page (Navigation Model)
- **Opening app**: Shows graph canvas by default
- **Graph is the primary view**: All navigation starts here; detail windows are contextual explorations
- **Graph structure drives UI logic**:
  - **Tabs in detail window reflect edges** from the focused node
  - **Tab order is chronological** (oldest connection first) per edge `created_at`
  - **Tab colors/icons** can indicate edge type (visited, bookmarked, clicked, auto-linked, etc.)
  - **Hover node on canvas** → Show preview of connected nodes (Phase 2)
  - **Search results highlight path** from current node to result (Phase 2)
- **Back navigation**: Close detail window → Return to graph-centric view; graph state persists
- **Context menu** on canvas reflects graph operations (add node, create edge, group, etc.)

---

## Phase 1: MVP (Force-Directed Graph Canvas)

**Goal**: Replace tab strip with a force-directed graph. Load URLs into nodes. Move camera around. Maintain feature parity with current tab browser for basic browsing.

**Scope**: Graph engine + canvas rendering + basic interaction (single monolithic crate)

**Estimated effort**: 6–8 weeks (core developer)

**Architecture Note**: Phase 1 is a **single `graphshell` application crate** with graph, physics, rendering, and UI code all coexisting. We design internal modules (graph, camera, physics) to be engine/UI agnostic and maintainable, but **do not extract into separate crates yet**. This keeps the build simple, avoids IPC complexity, and lets us iterate rapidly on the graph UX. Extraction to `graphshell-graph-core`, `graphshell-renderer`, etc. happens in Phase 2/3 once the core design is stable.

### 1.1: Design Graph Engine (1–2 weeks)

**What**: Core data structures and physics simulation. Keep it simple: Verlet integration, no quadtree (Phase 2).

**Deliverables**:
- `src/graph/mod.rs`: Module organization
- `src/graph/node.rs`: Node struct (id, position, velocity, webview_id, metadata)
- `src/graph/edge.rs`: Edge struct (source, target, type, weight)
- `src/graph/graph.rs`: Graph container and query API
- `src/graph/physics.rs`: Force calculations (spring, repulsion, damping)

**Key decisions**:
- Use simple Verlet integration for physics (position-based, stable)
- **No spatial partitioning Phase 1**: O(n²) brute-force forces. Target ~50 nodes max; optimize to quadtree in Phase 2
- Force model: `F_attraction = k_s * (d - L_0)` and `F_repulsion = k_r / d^2`
- Tune k_s, k_r, damping interactively

**Acceptance criteria**:
- Graph with 50 nodes runs physics tick at 60 FPS
- Nodes converge to stable layout in <2 seconds
- No panics or memory leaks under stress tests (50 node addition/removal)

### 1.2: Implement Core Data Structures (1 week)

**What**: Node, Edge, and Graph types with mutation/query API.

```rust
// Pseudo sketch
pub struct Node {
    pub id: NodeId,
    pub position: Vec2,
    pub velocity: Vec2,
    pub webview_id: WebViewId,
    pub title: String,
    pub pinned: bool, // Frozen node (don't move)
}

pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: EdgeType, // Historical, User, ClickPath, etc.
    pub created_at: Timestamp, // For chronological ordering in detail window tabs
    pub weight: f32, // Strength of connection
}

pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,
    // ...
}

impl Graph {
    pub fn add_node(&mut self, webview_id: WebViewId, title: String) -> NodeId;
    pub fn remove_node(&mut self, id: NodeId) -> Option<Node>;
    pub fn add_edge(&mut self, source: NodeId, target: NodeId, ty: EdgeType) -> EdgeId;
    pub fn update_physics(&mut self, dt: f32);
}
```

**Tests**: Unit tests for add/remove, edge cases (disconnected nodes, cycles)

### 1.3: Adapt Rendering (2 weeks)

**What**: Replace tab canvas with graph canvas. Render nodes as boxes with Servo content inside. Add basic chrome UI.

**Current flow**:
1. Tab UI draws tab strip
2. On tab click, show corresponding webview
3. Webview renders via Servo

**New flow**:
1. Graph canvas draws nodes at (x, y)
2. Small webview pool (2–4 reused webviews) renders focused/visible nodes
3. Servo output composited as offscreen textures
4. Composite textures onto node quads in graph view
5. Chrome (menu, toolbar, HUD) rendered as 2D overlays
6. On node focus, allocate webview from pool, load URL, bind to node

**Deliverables**:
- `src/rendering/canvas.rs`: Canvas abstraction (2D backend first)
  - `pub trait CanvasBackend { fn draw_quad(...); fn draw_line(...); fn render_text(...); }`
  - `pub struct Canvas2D { /* WebRender integration */ }`
- `src/rendering/graph_renderer.rs`: Graph-specific drawing
  - Draw nodes (circle/rect) with labels and icons
  - Draw edges as lines (straight, no fancy curves Phase 1)
  - No LOD culling Phase 1 (add in Phase 2)
- `src/ui/chrome.rs`: Menu bar, toolbar, status bar, HUD rendering
  - Menu bar (text + click detection)
  - Toolbar buttons (+ Node, Pause/Play, Reset, Search box)
  - Status bar (FPS, node/edge count, zoom level)
  - Optional HUD info overlay
- Adapt `src/compositor.rs` to feed Servo output into node textures and composite chrome on top
- **Webview pool in `src/graphshell.rs`**: Pool of 2–4 shared webviews
  - `fn focus_node(node_id) → binds webview from pool, loads node URL`
  - `fn unfocus_node(node_id) → returns webview to pool`
  - Pool automatically renders focused/nearby nodes; others render as static thumbnails (Phase 2)

**Key integration point**:
- Servo renders to offscreen surface → Node stores texture handle
- Canvas composites node quads + edges + chrome into main framebuffer
- Pool model: No one-webview-per-node (scales to infinity); focus pool reuse (practical browsing)

**Acceptance criteria**:
- 50-node graph renders at 60 FPS on typical laptop
- Node textures update when webview content changes
- Edges render without z-fighting
- Menu bar clickable (File, Edit, View, Tools, Help)
- Toolbar buttons functional (Pause, Reset, Search shows/hides results)
- Status bar displays correct FPS, node count, zoom level

### 1.4: Camera System (1 week)

**What**: Pan and zoom controls. Game-like camera.

**Input mapping**:
- **WASD**: Pan (A = left, D = right, W = up, S = down)
- **Mouse wheel** or **Q/E**: Zoom in/out
- **Mouse move**: Show reticule (crosshair)
- **Mouse click**: Select node at cursor (raycast)

**Deliverables**:
- `src/camera.rs`: Camera struct (position, zoom, viewport)
  - `fn pan(&mut self, dx: f32, dy: f32)`
  - `fn zoom(&mut self, factor: f32)`
  - `fn screen_to_world(&self, x: i32, y: i32) -> Vec2` (for picking)
- Update rendering to apply camera transform (view matrix)
- Hook keyboard/mouse into camera input

**Acceptance criteria**:
- Smooth panning and zooming
- No stuttering or jank
- Zoom limits prevent getting stuck (e.g., min 0.1, max 10.0)

### 1.5: Graph Interaction & Detail Window (1 week)

**What**: Click nodes to open detail window with connection tabs; drag nodes on canvas; navigate via edges.

**Graph Canvas Interactions**:
- **Click node**: Open detail window (simple docked pane, not floating)
- **Drag node**: Pin temporarily, move it, resume physics on release
- **Right-click node**: Context menu (delete, pin, create edge, etc.)
- **Right-click edge**: Context menu (remove, change type, inspect)
- **Hotkeys**: (e.g., N = new node, D = delete selected, T = toggle pause physics, ESC = close detail window)

**Detail Window**:
- **Header**: Node title, close button, simple snap control (dropdown: right 50%, bottom 30%)
- **Connection tabs**: Each tab represents an edge (to/from this node)
  - **Ordered chronologically**: oldest edge first
  - **Tab label**: destination node title + optional favicon
  - **Tab icons**: Can indicate edge type (visited, bookmarked, etc.)
- **Content area**: Servo webview rendering node content (webview from pool, bound to this node)
- **Tab click**: Navigate to adjacent node (detail window updates in-place; graph canvas highlight shifts)
- **Tab close button** (optional): Remove edge from graph

**Window Layout System**:
- **Simple docked pane** (not Windows 11-style snapping):
  - Default: Graph takes left 70%, detail window takes right 30% (vertically split)
  - Alternative: Graph takes top 70%, detail window below takes 30% (horizontally split)
  - One detail window at a time (Phase 2: multi-pane support)
  - Manual resize: Drag divider between windows to adjust split ratio
- **Snap control dropdown**: Click dropdown in detail window header to choose layout
  - Options: Right 30%, Right 50%, Bottom 30%, Bottom 50%
- **Keyboard shortcuts**:
  - **Ctrl + W**: Close detail window
  - **Alt + Right/Down arrows**: Cycle snap positions
- **Constraint**: All windows stay within main application bounds; no floating windows

**Deliverables**:
- `src/input/picker.rs`: Raycast/picking logic
- `src/window.rs`: Window manager with simple docked layout logic
  - `WindowLayout` enum: RightSplit(ratio), BottomSplit(ratio)
  - Resize/close operations (no multi-pane grid Phase 1)
- `src/graph/interaction.rs`: Node/edge selection and manipulation
- `src/detail_window.rs`: Detail window logic, tab rendering, traversal, snap controls
- `src/ui/window_manager.rs`: Master window layout orchestration (coordinate graph canvas + detail pane)

**Acceptance criteria**:
- Click node opens detail window (docked right by default)
- Detail window header shows snap dropdown; clicking changes layout (right/bottom split)
- Connection tabs appear, sorted chronologically
- Tab click navigates to adjacent node (detail window content updates)
- Drag detail window divider resizes both panels smoothly
- Keyboard shortcuts work (Ctrl+W close, Alt+arrow cycle snaps)
- All windows are within main app bounds

### 1.6: Graph Construction & Seeding (1–2 weeks)

**What**: Build initial graph from bookmarks/history or start empty. Helper to add nodes/edges.

**Deliverables**:
- `src/graph/builder.rs`: Builder API
  ```rust
  pub struct GraphBuilder { ... }
  impl GraphBuilder {
      pub fn new() -> Self;
      pub fn with_node(self, url: &str) -> Self;
      pub fn build(self) -> Graph;
      pub fn from_bookmarks(path: &Path) -> Result<Self>;
      pub fn from_history(path: &Path) -> Result<Self>;
  }
  ```
- Implement parsers for bookmark files (HTML, JSON, etc.)
- Initial layout: circular or random placement, then let physics settle

**Acceptance criteria**:
- Import 50 bookmarks into graph nodes
- Graph settles to stable state in <5 seconds
- All nodes visible and no overlaps

### 1.7: Node Lifecycle & Webview Pool Management (1 week)

**What**: Manage a small pool of Servo webviews reused across focused nodes. Nodes exist as graph data without always having active webviews.

**Design**:
- **Graph exists independently** of webviews. Nodes hold metadata (URL, title, position) but not webview_id.
- **Webview pool** in `src/graphshell.rs`: 2–4 reused webviews (configurable)
- **On node focus**: Request webview from pool, load node URL, bind to node
- **On node unfocus**: Return webview to pool; save scroll position/form state to node metadata
- **Rendering**:
  - Focused/visible nodes: Render live via webview from pool
  - Unfocused/distant nodes: Render static thumbnail (cached during visit) or placeholder
  - Pool automatically prioritizes rendering focused node + nearby nodes if pool size allows

**Deliverables**:
- Modify `src/graphshell.rs` to manage graph-wide webview pool
- Each Node holds `url: String, metadata: NodeMetadata` (not webview_id)
- Pool struct with queue: `WebviewPool { available: Vec<WebviewId>, focused: HashMap<NodeId, WebviewId> }`
- `fn focus_node(node_id) → allocate webview from pool, load URL`
- `fn unfocus_node(node_id) → save state, return webview to pool`
- Update constellation message routing to map webview_id → node_id temporarily during focus
- Thumbnail caching for unfocused nodes (Phase 2: progressive updates)

**Acceptance criteria**:
- Create/destroy 50 nodes without leaking memory
- Focus 2 nodes sequentially; webview pool reuses same webview_ids
- Scroll position preserved when switching focus to another node, then back
- No message routing errors during webview reuse

### 1.8: MVP Validation (1 week)

**What**: Integration test. Load a small graph, interact, verify behavior.

**Test scenario**:
1. Start graphshell with empty graph
2. Right-click canvas → "New node" → Type URL (e.g., example.com) → Node A created
3. Repeat step 2 twice more → Nodes B, C on canvas
4. Drag nodes around → Physics updates layout
5. Right-click between A and B → "Create edge" → Edge created with timestamp
6. Click node A → Detail window opens showing A's content with connection tabs (B shown as tab)
7. Click tab for B → Detail window updates to show B's content; webview from pool rebinds to node B; canvas shows B highlighted
8. Click back to tab for A → Webview returns to pool, reallocates for A; scroll position preserved
9. Close detail window → Canvas visible again
10. Delete node C via context menu
11. Save/load graph → JSON persists structure + edge timestamps + node scroll positions

**Acceptance criteria**:
- All interactions work as expected
- No crashes, memory leaks, or hangs
- Performance remains >30 FPS with 50 nodes (physics off or reduced)
- Webview pool reuse works without routing errors
- Detail window and connection tabs work seamlessly
- Edge timestamps are preserved and tabs ordered correctly
- Scroll position preserved on node refocus

---

## Phase 2: Architecture Extraction & Advanced Features (Weeks 9–16)

### 2.1: Modularization (Extract `graphshell-graph-core`)
- Once Phase 1 design is stable, extract graph engine, physics, and camera into a separate `graphshell-graph-core` crate
- `graphshell-graph-core` exposes:
  - `GraphEngine`: Graph, Node, Edge, Physics, Camera types
  - `GraphRenderer` trait: Pluggable renderer (WebRender, wgpu, etc.)
  - `CanvasBackend` trait: Pluggable 2D drawing
- `graphshell` crate becomes UI orchestration + Servo integration on top of `graphshell-graph-core`
- **Timing**: Extract after MVP features are solid; avoid premature modularization

### 2.2: Optimization & Persistence
- **Quadtree spatial partitioning**: Scale physics to 500+ nodes
- **Persistence**: JSON serialization of graph structure, node metadata, viewport state
- **Load from bookmarks**: Import Firefox/Chrome/Edge bookmarks as initial graph
- **Load from history**: Reconstruct graph from browser history file

### 2.3: Grouping & Hierarchy
- Detect clusters (connected components) automatically
- Collapse/expand groups as single nodes
- Visual feedback (color, icons)

### 2.4: Filtering & Search
- Search bar: "Find nodes by title/URL"
- Filters: by domain, date added, tag
- Toggle visibility of subsets

### 2.5: Multi-Pane Layout
- Support multiple detail windows (Phase 1 limit: one)
- Split remaining space (e.g., 2-way or 3-way grid)
- Window snapping controls (Windows 11-like snap positions)
- Keyboard shortcuts for multi-pane navigation

### 2.6: Export & Sharing
- **JSON export**: Full graph data
- **Interactive HTML**: Standalone HTML file with node cards and clickable edges
- **PNG/SVG**: Visual snapshot
- **Node URL**: Share individual node as URL with metadata card

### 2.7: 3D Visualization
- **Stacked 3D**: Nodes arranged in depth layers, non-rotatable camera
- **Full 3D**: Arbitrary rotation, perspective camera
- Preserve edge connections and relative positions

### 2.8: DOM Extraction
- Right-click element on page → "Extract as node"
- New node created with element content and source reference
- Metadata stored (source URL, element selector, timestamp)

### 2.9: Node Metadata & Sidebar
- Title, favicon, creation date, last visited, tags
- Scroll position / form state
- Metadata panel in sidebar when node selected
- Quick access to node operations (pin, delete, add edge)

---

## Phase 3: Network & Ecosystem (Weeks 17–24)

### 3.1: P2P Sync (Research)
- Event-based sync model (node added/removed/updated)
- Decentralized graph merge
- Identity/signing for attributed contributions

### 3.2: Tokenization & Data Marketplace (Research)
- User data ownership model
- Opt-in sharing with privacy controls
- Compensation mechanisms

---

## Module Reorganization

### Current Structure
```
src/
  graphshell.rs          (main event loop)
  compositor.rs     (rendering)
  window.rs         (window management)
  tab.rs            (tabs)
  webview/          (webview/embedding)
  storage.rs        (persistence)
  download.rs       (downloads)
  config.rs         (config)
```

### New Structure (post-migration)
```
src/
  graphshell.rs          (main event loop) - MODIFIED
  compositor.rs     (rendering) - MODIFIED
  window.rs         (window mgmt) - MODIFIED (now manages detail window + connection tabs)
  tab.rs            (REPURPOSED - now represents connection/edges in detail window, not main tabs)
  
  graph/            (NEW)
    mod.rs
    node.rs         (Node struct)
    edge.rs         (Edge struct)
    graph.rs        (Graph container)
    physics.rs      (Physics engine)
    builder.rs      (Graph construction)
    interaction.rs  (User interactions)
  
  rendering/        (NEW)
    mod.rs
    canvas.rs       (CanvasBackend trait)
    graph_renderer.rs (Graph drawing)
  
  ui/               (NEW)
    mod.rs
    chrome.rs       (Menu bar, toolbar, HUD, status bar)
    menu.rs         (Menu structure and event handling)
    search.rs       (Search/filter UI)
  
  detail_window.rs  (NEW - Detail window + connection tabs management)
  camera.rs         (NEW - Camera control)
  
  input/            (NEW)
    mod.rs
    picker.rs       (Ray casting / picking)
  
  webview/          (MODIFIED - manage pool, not individual tabs)
  storage.rs        (MODIFIED - graph serialization + edge timestamps)
  download.rs       (unchanged)
  config.rs         (MODIFIED - new graph-specific options + detail window config + chrome visibility)
```

---

## Key Design Decisions

### 1. **Physics Model**
- **Choice**: Verlet integration (position-based)
- **Rationale**: Stable, simple, efficient; easy to add constraints
- **Alternative considered**: Force-based (Euler); more standard but less stable

### 2. **Rendering Backend**
- **Phase 1**: 2D only (WebRender)
- **Phase 2**: 3D (extend webrender or use wgpu)
- **Rationale**: 2D is MVP; 3D is nice-to-have for exploration

### 3. **Node Selection / Focus**
- **Approach**: Single active node shown in detail window
- **Detail window model**:
  - Canvas shows all nodes; one is focused (highlighted)
  - Click node → detail window opens
  - Detail window shows node content + connection tabs
  - Clicking connection tab updates the active node
  - Close detail window to return to canvas-only view
- **Optional Phase 2 enhancements**: Multiple open detail windows (tabbed), hover previews

### 4. **Webview Management**
- **Current**: One or a few webviews per tab
- **New**: One webview per graph node (potentially dozens)
- **Challenge**: Servo overhead (memory, CPU)
- **Solution**: 
  - Lazy loading (create webview only when node visible or clicked)
  - Offscreen rendering (render to texture, not screen)
  - Unload distant nodes (GC strategy)

### 5. **Edge Representation**
- **Data**: `(source_id, target_id, edge_type, weight, created_at, metadata)`
- **Timestamp**: Edges store creation time for chronological ordering in detail window tabs
- **Edge types**: Historical (clicked from A→B), Related (detected via ML?), User-created
- **Rendering**: Straight line, bezier, or force-repelled curves on canvas
- **In detail window**: Tab list sorted by `created_at`; oldest first

---

## Quick Start: MVP Sequence

1. **Week 1**: Design graph engine + implement Node/Edge/Graph structs (add `created_at` to Edge)
2. **Week 2**: Add physics update loop; test with 5 stationary nodes
3. **Week 3**: Canvas renderer; draw nodes/edges; basic camera
4. **Week 4**: Keyboard input for camera; mouse picking for selection
5. **Week 5**: Right-click menu; add/delete node ops; detail window shell
6. **Week 6**: Integrate Servo; one webview per node; load URLs in detail window
7. **Week 7**: Connection tabs in detail window; chronological edge sorting; tab navigation
8. **Week 8**: MVP validation; polish; bug fixes; test detail window + tab switching

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Servo multi-webview overhead (CPU/memory) | Slow/crash with >20 nodes | Lazy load, offscreen render, GC distant nodes |
| Physics instability (exploding forces) | Jank, aesthetic issues | Tune constants, cap force magnitude, test |
| Complex refactoring breaks existing features | Regressions | Keep tab browser working in parallel (Phase 1); full cutover after MVP validated |
| Camera controls feel unintuitive | UX confusion | Play-test with users; iterate based on feedback |
| Graph serialization format becomes stale | Data loss on upgrade | Version format; migration code; docs |

---

## Success Criteria

### Phase 1 (MVP)
- [ ] Graph with 10+ nodes renders smoothly (60 FPS)
- [ ] WASD + mouse navigation works intuitively
- [ ] Click node → load and view URL
- [ ] Drag node → physics pauses, resumes on release
- [ ] Graph persists to JSON; can reload
- [ ] No crashes or memory leaks (8-hour soak test)

### Phase 2
- [ ] Import bookmarks / history
- [ ] Export to interactive HTML
- [ ] 3D visualization toggleable
- [ ] DOM element extraction
- [ ] ~50 nodes render without lag

### Phase 3
- [ ] P2P sync MVP (2-3 node graph sharing)
- [ ] Tokenization prototype (analytics + compensation model)

---

## Timeline Estimate

| Phase | Effort | Calendar |
|-------|--------|----------|
| Phase 1 (MVP) | 6–8 weeks | 2–3 months |
| Phase 2 (Features) | 6–8 weeks | 3–4 months |
| Phase 3 (Network) | 4–6 weeks | 2–3 months |
| **Total** | **16–22 weeks** | **~9 months** |

*Assumes 1 full-time senior engineer. Scales with team size.*

---

## Recommended Starting Point

**Begin with Phase 1, Week 1: Graph Engine Design**

1. Create `src/graph/` module structure
2. Define Node, Edge, Graph traits and types
3. Write unit tests for graph operations
4. Sketch physics engine interface (no implementation yet)
5. Document design decisions in comments

Then proceed sequentially through Phase 1 to MVP (Week 8).

---

## References

- **Force-directed layout**: Fruchterman-Reingold algorithm (classic reference)
- **Physics simulation**: Verlet integration (Jakobsen 2001, GDC)
- **Game-like UI**: Quake/Half-Life HUD paradigm
- **Spatial browsing**: Internet Map (http://internet-map.net/)
- **Knowledge graphs**: Obsidian, Roam Research (inspiration for graph UX)

