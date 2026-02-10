# Architecture Decisions

This document captures the rationale behind key architectural choices for Graphshell.

## 1. View Toggle: Full-Screen with Resizable Split-View

**Decision:** Graph and detail view toggle via full-screen. Default layout when both visible: 60% document, 40% graph.

**Rationale:**
- Simple mental model (one active view)
- Better use of screen real estate in full-screen mode
- Resizable split-view avoids mini window manager complexity
- Cluster strip is a linear projection of the active subgraph (same nodes, different lens)
- Pinned cluster entries show visual indicator on graph nodes

**Implementation:**
- Toggle view with home button (left of omnibar)
- Resizable divider via mouse drag
- Remember user's preferred split ratio in preferences

**Alternative considered:** Floating popup window (more flexible, but adds dragging/z-order/minimize complexity).

---

## 2. Rendering Architecture: Dual-Pipeline Design

**Decision (Option A, egui-first):** Graph UI rendered via egui, web content rendered via Servo's WebRender. Two separate rendering pipelines composited in servoshell's shell layer. No WebRender-native graph UI in Phase 1-3.

**Rationale:**
- **egui** provides immediate-mode UI perfect for dynamic graph interactions (drag, pan, zoom)
- **WebRender** is Servo's GPU-accelerated renderer for web content (pages, images, text)
- **No unified rendering path in MVP**: egui translates to its own GPU primitives, WebRender uses display lists
- **Composition happens in shell layer**: servoshell (not Servo's compositor) orchestrates both
- **Future optimization possible**: Port graph renderer to WebRender-native for single pipeline (Phase 4+)

**Current Architecture:**
```
┌─────────────────────────────────────┐
│  Servoshell (Shell Layer)           │
│  ┌──────────────┐  ┌──────────────┐ │
│  │ egui         │  │ Servo        │ │
│  │ Graph UI     │  │ WebViews     │ │
│  │ (immediate)  │  │ (WebRender)  │ │
│  └──────────────┘  └──────────────┘ │
│         │                 │          │
│         └────► GPU ◄──────┘          │
└─────────────────────────────────────┘
```

**Why not unified WebRender for everything:**
- egui integration is mature and fast in servoshell (egui 0.33.3)
- WebRender-native UI would require custom primitives, more complexity
- MVP benefits from egui's rapid iteration and built-in widgets
- Unification is optimization, not architectural requirement

**Performance implications:**
- Two GPU command buffers (egui + WebRender)
- Acceptable overhead: < 1ms per frame on modern GPUs
- egui's retained-mode tessellation caches unchanged shapes
- Only dynamic elements (node positions during physics) re-tessellate

**Alternative considered:** WebRender-only (more consistent, but much higher initial dev cost).

---

## 2a. Servo Dependencies & Breakage Budget

**Decision:** Treat Servo as a moving dependency. Explicitly track required features and budget time for upstream changes each phase.

**Required Servo features (non-negotiable):**
- servoshell shell layer + egui integration
- WebRender surfaces and WebViewCollection pattern
- `EmbedderMsg` / `EmbedderEvent` and `ServoDelegate` / `WebViewDelegate` hooks
- `-M` multiprocess flag and origin grouping
- `ipc-channel` for internal Servo IPC (no custom bypass)

**Optional (nice-to-have):**
- Sandboxing (`-S` / gaol)
- Advanced accessibility (likely needs upstream work)

**Breakage budget:**
- Reserve 10-15% of each phase for Servo API changes and integration fixes
- Pin Servo revision at each milestone; rebase monthly
- Maintain a thin adapter layer between Graphshell and servoshell to localize changes

---

## 3. Edge Rendering: Bezier Curves → Bundled Edges

**Decision:** Start with quadratic Bezier curves (Week 1). Switch to bundled edges (Week 9) if visual clutter detected.

**Rationale:**
- Bezier: O(n) render cost, simple implementation, good for < 500 edges
- Bundled: ~200ms preprocessing (one-time), then ~16ms render, scales to 5000+ edges
- Bundled edges reveal graph structure (power-law clustering, hierarchies)
- Graceful upgrade path: if graph feels cluttered at Week 6, switch to bundled at Week 9

**Edge types and styles:**
- Color differentiates type (user-selectable)
- Line style (solid, dotted, bold, marker) provides colorblind-safe redundancy
- Types: Hyperlink (blue, solid), Bookmark (green, solid+marker), History (gray, dotted), Manual (red, bold)

**Implementation notes:**
- Control points for Bezier: midpoint of edge, pushed perpendicular to line
- Bundled edges: Use FDEB algorithm, implement via `D3-force` approach or custom
- Preview: Week 6 user testing will reveal if bundled edges are needed

---

## 4. Webview Management: Origin-Grouped + Lightweight Reuse Pool

**Decision:** Keep Servo's origin-grouped process model, but add a small reuse pool and memory-pressure reaper. Avoid large fixed pools.

**Rationale:**
- Servo's `-M` flag already manages processes per origin
- Origin-grouped nodes map naturally to origin-grouped processes
- Small reuse pool reduces latency when reopening recent origins
- LRU eviction prevents unbounded process churn
- Integrates cleanly with Active/Warm/Cold lifecycle (see 4a)

**Reuse Pool (small, focused):**
- Keep up to 2-4 recently used origin processes in a "warm" state
- If a node from that origin is reopened within a short TTL (30-60s), reuse the process
- If TTL expires or memory pressure hits, terminate oldest warm processes

**Process lifecycle:**
1. User creates node from origin A → Servo spawns process for A (Active)
2. Node demoted to Warm → snapshot thumbnail; process may stay warm if pool has capacity
3. Pool full or TTL expired → terminate oldest warm process (LRU)
4. Reopen origin A within TTL → reuse warm process; otherwise spawn new

**Memory pressure handler:**
- If free RAM < 500MB: demote warm processes first, then unpinned Active nodes
- If free RAM < 200MB: keep only focused Active node + pinned actives

**Alternative considered:** No pooling (fast churn, poor UX) or large fixed pool (wasteful at scale).

---

## 4a. Node Lifecycle: Active/Warm/Cold States

**Decision:** Implement three-tier node lifecycle to manage resource constraints. Hard cap of 20 active (live) webviews.

**Rationale:**
- **Active nodes** consume significant resources (Servo process, GPU memory, JS execution)
- Without lifecycle management, 100+ node graphs become infeasible
- Thumbnail-based "Warm" nodes provide visual continuity without process overhead
- Cold nodes are pure metadata, supporting unlimited graph size

**Node States:**

| State | Description | Resource Cost | Representation |
|-------|-------------|---------------|----------------|
| **Active** | Live Servo webview process | ~50-100MB RAM, GPU texture, CPU for JS/layout | Real-time rendering |
| **Warm** | Static thumbnail + serialized state | ~2-5MB RAM (texture only) | Last-known screenshot |
| **Cold** | URL + metadata only | ~1KB RAM | Placeholder icon + title |

**Promotion Rules (Cold/Warm → Active):**
1. User focuses node (click, Enter key)
2. Node enters viewport center (camera proximity < 200px)
3. User explicitly pins node as "always active"
4. Manual promotion via context menu

**Demotion Rules (Active → Warm → Cold):**
1. **Active → Warm:** When 20-node cap reached, demote least-recently-used (LRU) active node
2. **Warm → Cold:** Under memory pressure (< 500MB free RAM), demote oldest Warm nodes
3. User can manually demote via context menu
4. Pinned nodes never auto-demote

**Memory Reaper:**
- Runs every 5 seconds
- Checks system RAM via `sysinfo` crate
- If < 500MB free: Demote oldest Warm nodes to Cold (up to 50% of Warm nodes)
- If < 200MB free: Force-demote all unpinned Active nodes except focused node

**Thumbnail Pipeline:**
- Active node generates thumbnail on demotion (offscreen render to texture)
- Thumbnail stored in node state (JPEG, 400x300px, ~100KB)
- Warm nodes display last-known thumbnail with "stale" indicator if > 1 hour old
- User can force thumbnail refresh via context menu

**Implementation:**
```rust
pub enum NodeState {
    Active { webview: ServoWebView, last_interaction: Instant },
    Warm { thumbnail: Image, url: Url, scroll_pos: Point2D<f32>, timestamp: Instant },
    Cold { url: Url },
}

pub struct Node {
    id: NodeKey,
    state: NodeState,
    position: Point2D<f32>,
    velocity: Vector2D<f32>,
    pinned: bool,
    metadata: NodeMetadata,
}

const MAX_ACTIVE_NODES: usize = 20;
```

**Alternative considered:** Fixed 10-node cap (too restrictive), no lifecycle (infeasible at scale).

---

## 5. Physics: Grid-Based Repulsion with Worker Thread

**Decision:** Week 1: Grid-based spatial hashing for force calculation. Physics runs on dedicated worker thread. Week 6+: Profile and evaluate `kiddo` kd-tree if needed.

**Rationale:**
- **Grid-based repulsion (Week 1)**: O(n) average case for 200–500 nodes. Simple, fast, controls interaction feel.
- **Worker thread**: Offloads physics from egui/UI thread. Position updates sent via `crossbeam` channel every frame.
- **Staged upgrade path**: Start simple, measure Week 6, adopt `kiddo` or custom Barnes-Hut only if profiling shows repulsion is bottleneck.
- **Real-time interaction**: Worker thread + bounded update rate ensures responsive drag, pan, and hover without jank.

**Architecture:**

```
Main UI Thread (egui)          Physics Worker Thread
├─ Handle input              ├─ Spatial hash grid
├─ Render graph              ├─ Calculate forces (repulsion/attraction)
├─ Check lifecycle events    ├─ Integrate velocity → position
└─ Receive position updates◄─┴─ Send updated positions via channel
    (non-blocking)              (every physics_dt, rate-limited)
```

**Grid-Based Repulsion Algorithm:**

```rust
pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<NodeKey>>,
}

fn calculate_forces(graph: &mut Graph, grid: &SpatialGrid) {
    for node_key in graph.nodes.keys() {
        let node = &graph.nodes[node_key];
        let cell = (node.position.x / grid.cell_size) as i32;
        
        // Check current + 8 adjacent cells
        for dx in -1..=1 {
            for dy in -1..=1 {
                let neighbor_cell = (cell.0 + dx, cell.1 + dy);
                if let Some(neighbors) = grid.cells.get(&neighbor_cell) {
                    for &neighbor_key in neighbors {
                        if neighbor_key != node_key {
                            let repulsion = calculate_repulsion(node, &graph.nodes[neighbor_key]);
                            graph.nodes[node_key].velocity += repulsion * dt;
                        }
                    }
                }
            }
        }
        // Also apply attraction to connected edges
        for &edge_key in &graph.outgoing[&node_key] {
            let target_key = graph.edges[edge_key].to;
            let attraction = calculate_attraction(node, &graph.nodes[target_key]);
            graph.nodes[node_key].velocity += attraction * dt;
        }
    }
}
```

**Parameters:**
- **Damping**: 0.08 (default, user-adjustable)
- **Velocity threshold (world-space)**: Scale-normalized to graph bounds. Pause when avg velocity < 0.001 × graph_diagonal / frame_time
- **Grid cell size**: viewport_diagonal / 4 (dynamic, updates when camera zooms)
- **Stabilization timeout**: 5 seconds of low velocity → auto-pause
- **Physics dt**: 1/120 (fixed timestep, decoupled from render framerate)
- **Update rate**: Max 60 position updates/sec to UI (throttle to avoid channel saturation)

**Threading Implementation:**

```rust
use crossbeam::channel::{bounded, Sender, Receiver};
use std::thread;

pub struct PhysicsWorker {
    tx: Sender<PhysicsUpdate>,
    terminate: Arc<AtomicBool>,
}

impl PhysicsWorker {
    pub fn spawn(mut graph: Graph) -> (Self, Receiver<PhysicsUpdate>) {
        let (tx, rx) = bounded(2); // Small channel to avoid stale updates
        let terminate = Arc::new(AtomicBool::new(false));
        let terminate_clone = terminate.clone();

        thread::spawn(move || {
            let mut last_update = Instant::now();
            loop {
                if terminate_clone.load(Ordering::Relaxed) {
                    break;
                }
                
                // Run physics step
                physics.step(&mut graph, 1.0 / 120.0);
                
                // Rate-limit updates: max 60/sec
                if last_update.elapsed() > Duration::from_millis(16) {
                    let update = PhysicsUpdate {
                        positions: graph.nodes.iter().map(|(k, n)| (k, n.position)).collect(),
                    };
                    let _ = tx.try_send(update); // Non-blocking; drop if UI thread is slow
                    last_update = Instant::now();
                }
            }
        });

        (PhysicsWorker { tx, terminate }, rx)
    }

    pub fn terminate(&self) {
        self.terminate.store(true, Ordering::Relaxed);
    }
}
```

**Auto-Pause & Stability:**
- **Velocity threshold is scale-normalized**: `threshold = 0.001 * graph_diagonal / viewport_height`
  - Ensures pause behavior is consistent across zoom levels
  - Threshold recalculated when camera bounds change
- **Selected/dragged nodes**: Physics pauses immediately for focused node; other nodes continue until stabilization timeout
- **Pinned nodes**: Always have zero velocity, never contribute repulsion
- **Velocity clamping**: If any node velocity > 10× typical max, clamp to prevent explosion

**Worst-case handling:**
- **Oscillation**: Damping (0.08) prevents perpetual back-and-forth
- **Explosion**: Clamp velocity magnitude if > safety threshold (5× expected max)
- **Non-convergence**: 5-second timeout forces pause even if velocity oscillates slightly
- **Clustered nodes**: Grid degrades to O(n²) if 100+ nodes in one cell; Week 6 profiling will catch this

**Week 6+ Evaluation:**
If physics is CPU bottleneck (>5ms per frame at 500 nodes):
1. Measure grid cell utilization (are many cells empty? are some overloaded?)
2. Evaluate `kiddo` kd-tree substitute: ~200-line integration, better neighbor queries
3. If still insufficient: custom quadtree Barnes-Hut (Week 9+)

**Week 9+ Optional Upgrade (if needed):**
- Swap grid-based for `kiddo` or custom quadtree
- Same channel/worker thread architecture, different force calculation
- No UI changes required

---

## 6. Data Structures: Graph Representation

**Decision:** Adjacency list with separate in/out edges, metadata in-struct.

**Rationale:**
- SlotMap for stable node handles: `SlotMap<NodeKey, Node>`
- Adjacency list (in + out) for O(1) neighbor queries
- In-struct metadata: < 1M nodes expected, simpler code
- Separate adjacency prevents O(n) scans for physics forces

**Structures:**
```rust
pub struct Graph {
    nodes: SlotMap<NodeKey, Node>,
    edges: SlotMap<EdgeKey, Edge>,
    outgoing: HashMap<NodeKey, Vec<EdgeKey>>,  // Springs calculation
    incoming: HashMap<NodeKey, Vec<EdgeKey>>,  // Cluster strip grouping
}

pub struct Node {
    id: NodeKey,
    url: Url,
    position: Point2D<f32>,
    velocity: Vector2D<f32>,
    pinned: bool,
    metadata: NodeMetadata,
}

pub struct NodeMetadata {
    title: String,
    favicon: Option<Image>,
    thumbnail: Option<Image>,
    tags: Vec<String>,
    color: Option<Color>,
    notes: String,
    created_at: DateTime<Utc>,
    last_visited: DateTime<Utc>,
    visit_count: u32,
}

pub enum EdgeType {
    Hyperlink,
    Bookmark,
    History,
    Manual,
}

pub struct Edge {
    from: NodeKey,
    to: NodeKey,
    ty: EdgeType,
    weight: f32,
}
```

**Edge type purposes:**
- Hyperlink: Discovered via page scraping
- Bookmark: User manually saved
- History: Navigation sequence
- Manual: User-drawn connections

---

## 7. Persistence: Snapshot + Append-Only Log + Crash Recovery

**Decision:** Formal persistence model: immutable snapshots + append-only command log + deterministic recovery.

**Rationale:**
- **Snapshots** (full graph state): Taken every 30 seconds or after 20 commands, whichever comes first
- **Append-only log**: Every user action (add node, move edge, pin, etc.) logged before execution
- **Recover from crash**: Replay last snapshot + log entries since snapshot
- **Session history**: Keep last 5 snapshots + their logs (e.g., 1 hour of history)
- **Fast writes**: Only diff written to log, not full state ("delta encoding")

**Storage structure:**
```
~/.config/graphshell/
├── sessions/
│   ├── session_2025-02-06T10.00.00Z.snapshot.json    # Full state (immutable)
│   ├── session_2025-02-06T10.00.00Z.log               # Command log since snapshot
│   ├── session_2025-02-06T10.30.00Z.snapshot.json
│   ├── session_2025-02-06T10.30.00Z.log
│   ├── session_2025-02-06T11.00.00Z.snapshot.json
│   └── ... (up to 5 snapshots + logs)
├── preferences.toml
├── keybinds.toml
└── theme.toml
```

**Snapshot Format (JSON, compressed):**
```json
{
  "version": 1,
  "timestamp": "2025-02-06T10:00:00Z",
  "graph": {
    "nodes": [
      { "id": "node_0", "url": "https://example.com", "state": "Warm", "position": [100, 200], "pinned": false }
    ],
    "edges": [
      { "id": "edge_0", "from": "node_0", "to": "node_1", "type": "Hyperlink", "weight": 1.0 }
    ]
  },
  "metadata": { "node_count": 1, "edge_count": 1 }
}
```

**Command Log Format (append-only, one JSON per line):**
```json
{"timestamp": "2025-02-06T10:00:05Z", "command": "AddNode", "url": "https://example.org", "position": [150, 250]}
{"timestamp": "2025-02-06T10:00:10Z", "command": "AddEdge", "from": "node_0", "to": "node_1", "type": "Manual"}
{"timestamp": "2025-02-06T10:00:15Z", "command": "MoveNode", "id": "node_0", "position": [120, 180]}
```

**Recovery Process:**
1. Find latest snapshot file
2. Deserialize snapshot into in-memory graph
3. Read command log since snapshot timestamp
4. Replay commands in order (deterministic: same input → same final state)
5. Validate graph integrity (all nodes/edges present, no dangling refs)
6. Resume with recovered state

**Write Strategy:**
- **On every user action**: Log command immediately (< 1ms, single line append)
- **Every 30 seconds or 20 commands**: Write full snapshot (compress with gzip, < 100KB for typical graphs)
- **Non-blocking**: Writes happen on separate thread, don't block UI
- **Durability**: `fsync()` after snapshot writes, `fsync()` on shutdown

**Crash Recovery:**
- **Unclean shutdown** (crash/power loss): Log might have half-written line
  - On recovery: skip malformed last line, replay up to last valid command
  - User only loses ~1 second of work (since last log entry)
- **Snapshot corruption**: Fall back to previous snapshot + its log
- **Both corrupted**: Warn user, offer to load 2 snapshots back

**Cleanup Policy:**
- Keep 5 most recent (snapshot, log) pairs
- Delete older sessions automatically
- User can export snapshot at any time for backup

**Implementation:**
```rust
pub struct Persistence {
    snapshot_dir: PathBuf,
    current_log: File,
    command_count: usize,
    last_snapshot: Instant,
}

impl Persistence {
    pub fn log_command(&mut self, cmd: &Command) -> io::Result<()> {
        let json = serde_json::to_string(&cmd)?;
        writeln!(self.current_log, "{}", json)?;
        self.command_count += 1;
        
        // Snapshot if needed
        if self.command_count >= 20 || self.last_snapshot.elapsed() > Duration::from_secs(30) {
            self.write_snapshot()?;
            self.command_count = 0;
            self.last_snapshot = Instant::now();
        }
        Ok(())
    }
    
    pub fn recover_from_crash() -> io::Result<Graph> {
        let snapshot = load_latest_snapshot()?;
        let log = load_log_since(&snapshot.timestamp)?;
        let mut graph = snapshot.into_graph();
        for cmd in log {
            apply_command(&mut graph, &cmd)?;
        }
        Ok(graph)
    }
}
```

**Alternative considered:** Git-style version control (overkill for single-user graph). Simpler linear history sufficient.

---

## 8. Search: Fuzzy-First, Full-Text Later

**Decision:** Start with fuzzy search across title/url/tags using `fuzzy-matcher` (Phase 1). Add full-text indexing (Phase 2) and semantic search (Phase 3) if needed.

**Rationale:**
- Prefix-only is too weak for real use; fuzzy matching improves recall
- `fuzzy-matcher` (SkimMatcherV2) is lightweight and fast for 10K nodes
- Full-text search is heavier; defer until UX validated

**Search result visualization:**
- Highlight matching nodes in graph (brighter color)
- List results in omnibar dropdown
- Minimap shows matches as bright dots
- Focus camera on first match (optional, configurable)

**Roadmap:**
- Phase 1: `fuzzy-matcher` on title/url/tags + prefix fallback
- Phase 2: Optional full-text index (e.g., `tantivy`) for page content
- Phase 3: Semantic search (optional) using embeddings

**Alternative considered:** Full-text search on day one (too heavy for MVP).

---

## 8a. Extensions: Minimal, Safe, and Graph-Centric

**Decision:** Define a v0 extension API in Phase 3 with a small, safe surface area.

**Scope (v0):**
- Manifest-based extensions with explicit permissions
- Read-only graph queries by default; write access via explicit permission
- Event hooks: node opened, node created, edge created, selection changed
- Custom commands in omnibar (`/ext command`)

**Manifest (v0, JSON):**
```json
{
    "id": "com.example.graph-tools",
    "name": "Graph Tools",
    "version": "0.1.0",
    "description": "Quick graph utilities",
    "entry": "main.wasm",
    "permissions": ["graph.read", "graph.write", "selection.read", "commands.register"],
    "commands": [
        { "id": "ext.cluster", "title": "Cluster Selection" }
    ]
}
```

**Permissions (initial set):**
- `graph.read`: read nodes/edges/metadata
- `graph.write`: create/update/delete nodes/edges
- `selection.read`: read current selection and camera
- `commands.register`: register omnibar commands
- `file.export`: export data to user-selected location
- `file.import`: import user-selected file

**API surface (v0):**
- `graph.query(filter) -> Vec<Node>`
- `graph.add_node(url, position) -> NodeId`
- `graph.add_edge(from, to, type) -> EdgeId`
- `graph.update_node(id, patch)`
- `selection.get() -> Vec<NodeId>`
- `events.on("node_opened" | "node_created" | "edge_created" | "selection_changed", handler)`

**Execution model:**
- Prefer WASM sandbox (wasmtime) for untrusted extensions
- Allow optional native Rust plugins for trusted/local-only extensions
- Resource limits: CPU time per tick, max memory per extension, no network by default

**Not in v0:**
- Arbitrary UI injection into webviews
- Cross-origin scraping without user consent

**Alternative considered:** No extensions (limits long-term adoption).

---

## 8b. Import/Export: First-Class Interchange

**Decision:** Provide JSON-based graph interchange and common import/export paths early.

**Core formats:**
- Export graph as versioned JSON (schema in repo, backward compatible)
- Export view as PNG/SVG (presentation sharing)
- Import browser bookmarks (Netscape bookmarks.html)
- Export edges/nodes as CSV for analysis

**Graph JSON schema (v0):**
```json
{
    "schema_version": 1,
    "metadata": {
        "title": "Research Map",
        "created_at": "2026-02-06T10:00:00Z",
        "updated_at": "2026-02-06T11:00:00Z"
    },
    "nodes": [
        {
            "id": "node_0",
            "url": "https://example.com",
            "title": "Example",
            "position": [100, 200],
            "pinned": false,
            "tags": ["research"],
            "color": "#44AAFF"
        }
    ],
    "edges": [
        { "id": "edge_0", "from": "node_0", "to": "node_1", "type": "Hyperlink" }
    ],
    "anchors": [
        { "id": "anchor_0", "nodes": ["node_0", "node_1"], "center": [120, 180], "pinned": true }
    ],
    "regions": [
        { "id": "region_0", "name": "Research", "bounds": [0, 0, 800, 600], "color": "#FFF2B2" }
    ]
}
```

**Compatibility rules:**
- Schema is append-only; unknown fields are ignored
- Migrations are one-way with a stored `schema_version`
- Export always uses latest schema version

**Rationale:**
- Graphs are sense-making artifacts; sharing is core, not optional
- Importing bookmarks lowers onboarding friction
- Structured export enables tooling around Graphshell

---

## 9. Memory Management: Graceful Degradation

**Decision:** Monitor available RAM. Suspend background webviews if memory pressure detected.

**Rationale:**
- Chrome approach: Suspend background webviews when < 10% free RAM
- Firefox approach: Less aggressive, defer to OS
- For Graphshell: Use hybrid (monitor, but trust Servo's memory management)
- Servo already handles memory pressure for webviews

**Implementation:**
- Check available RAM every 5 seconds
- If < 500MB available: Suspect oldest unused webview process
- Suspend = pause JavaScript, freeze DOM, free GPU memory
- Resume = re-spawn process (costs 2-3 seconds, acceptable)

**No explicit DOM serialization** (too complex, Servo handles it).

---

## 10. Layout Algorithms: Force-Directed as Foundation

**Decision:** Implement force-directed layout with tunable presets for different topologies.

**Rationale:**
- Force-directed: Works for most graph types, proven algorithm
- Presets allow adaptation: Social (weak repulsion) vs Dense (strong repulsion) vs Hierarchical (level-based)
- User-selectable layout mode (Phase 2): Force-directed, Hierarchical, Circular, Grid, Timeline
- One physics engine, multiple force configurations

**Layout presets (configurable):**
```rust
pub enum LayoutPreset {
    ForceDirected { repulsion: f32, attraction: f32, damping: f32 },
    Hierarchical { level_separation: f32, node_separation: f32 },
    Circular { radius: f32 },
    Grid { cell_size: f32 },
    Timeline { axis: Axis, scale: f32 },
}
```

**User switches layouts:** Physics re-initialize with preset values, nodes animate to new positions.

---

## 11. Camera & Navigation

**Decision:** Bounded zoom with auto-center and minimap (Phase 2).

**Rationale:**
- Min zoom: 0.1 (see 10x larger area)
- Max zoom: 10.0 (see 10x smaller area)
- Auto-center: Average node position, auto-zoom to fit all nodes
- Keybind: 'C' for center
- Minimap: Shows all nodes, highlights viewport, click to navigate

**Alternative considered:** Infinite zoom (confusing at extremes, unnecessary).

---

## 11a. Spatial Stability: Pinning, Anchors, and Named Regions

**Decision:** First-class features for spatial organization. Users can pin nodes, anchor groups, and create named layout regions. Phase 1 (Week 1-8).

**Rationale:**
- Graph UIs are cognitively overloaded; visual stability reduces cognitive load
- Without anchors, users lose spatial context as graph updates
- Pinning allows "landmarks" for navigation
- Named regions let users organize subgraphs thematically

**Pinning (Week 1):**
- **User action**: Double-click node or Ctrl+P to pin
- **Behavior**: Pinned node has zero velocity, ignores repulsion/attraction, stays user-positioned
- **Visual**: Pinned nodes show anchor icon, slightly different color (e.g., gold border)
- **Implementation**: `node.pinned: bool` in Node struct. Physics skips pinned nodes.
- **Persistence**: Pinned state saved in command log ("PinNode" command)

**Anchors/Groups (Week 2-3):**
- **Selection-based**: User selects N nodes (Ctrl+A or drag-select), then Ctrl+G to create anchor
- **Anchor behavior**: Nodes in anchor maintain relative positions; anchor center can be pinned
- **Visual**: Dashed convex hull around anchored nodes, shared color, label
- **Persistence**: AnchorGroup stored as node set + center position
- **Example use**: Cluster related pages (e.g., all docs from docs.rs under one anchor)

**Named Regions (Week 3-4):**
- **User action**: Draw rectangle on canvas, assign name ("Research", "Todo", etc.)
- **Behavior**: Purely visual; regions are categories, not force-modified
- **Visual**: Transparent background fill with label, user-customizable colors
- **Snapshot**: Regions include name, bounds, color. Survive reload.
- **Example use**: Organize graph by project or topic

**Implementation:**
```rust
pub enum SpatialElement {
    PinnedNode(NodeKey),
    Anchor {
        id: AnchorKey,
        nodes: Vec<NodeKey>,
        center: Point2D<f32>,
        pinned: bool,
    },
    Region {
        id: RegionKey,
        name: String,
        bounds: Rect,
        color: Color,
    },
}

pub struct Node {
    // ...
    pinned: bool,
    anchor_id: Option<AnchorKey>,
}
```

**Interaction model:**
- **Pinned nodes**: User drags → stays in place, physics resumes other nodes
- **Anchored nodes**: User drags group → entire anchor moves, internal layout preserved
- **Regions**: Non-interactive boundaries; user can resize/relabel via context menu

**Physics interaction:**
- Pinned nodes: Skip all force calculations
- Anchored nodes: Calculate internal forces, but anchor center doesn't move (unless anchor itself is pinned)
- Regions: Ignored by physics; purely visual organization

**Week 1 minimum:** Pinning only (high ROI, low complexity)
**Week 2-3 add:** Anchors (requires selection + grouping UI)
**Week 3-4 add:** Named regions (requires rectangle drawing + persistence)

**Alternative considered:** No spatial stability; rely on physics only (leads to cognitive overload).

---

## 12. Accessibility: Keyboard-First Design

**Decision:** Full keyboard navigation. Screen reader support. Color + style for colorblindness. Accessibility is non-optional, even if upstream Servo work is required.

**Rationale:**
- Keyboard: Arrow keys for navigation, Enter to open, Escape to close
- Screen reader: Graph structure announced, nodes labeled
- Color blind: Edge types use color + line style (solid, dotted, bold)
- High contrast mode: User-selectable theme
- Reduced motion: Disable physics animations if `prefers-reduced-motion` set

**Testing:** Week 1.5 (validation) includes 1-2 screen reader users.

**Alternative considered:** Accessibility as Phase 4 afterthought (too late, design decisions lock you in).

---

## 13. Keybinds: Conflict Resolution & Defaults

**Decision:** Smart conflict resolution. ~30 default keybinds. All overrideable.

**Rationale:**
- Conflict: If user binds key to action B, action A rebinds to its previous binding (if available) or default
- Defaults: Common browser keybinds (Ctrl+T, Ctrl+W, etc.) but configurable
- User-selectable: Settings UI for all keybinds
- Inverse mapping: `HashMap<Action, KeyCombination>` to detect conflicts

**Key categories:**
- Navigation (pan, zoom, center): WASD, Ctrl+Scroll, 'C'
- Editing (add, delete, select): 'N', Delete, Ctrl+A
- Undo/Redo: Ctrl+Z, Ctrl+Y
- File (save, open, new): Ctrl+S, Ctrl+O, Ctrl+N
- Sidebar (bookmarks, downloads, tags): Ctrl+B, Ctrl+J, Ctrl+;
- Search: Ctrl+F
- Settings & help: Ctrl+,, F1

**Alternative considered:** Non-configurable defaults (simpler, less user control).

---

## 14. Testing: Performance Acceptance Criteria (Tiered)

**Decision:** Tiered performance targets instead of single "60fps @ 1000 nodes" claim. Staged validation at Week 6, Week 9.

**Rationale:**
- **Single aggressive target fails projects**: Impossible to hit uniformly, leads to corner-case debugging late in cycle
- **Tiered targets**: Realistic scaling + planned profiling dates
- **Performance is a feature trade-off**: Higher node counts require algorithmic changes (grid → kiddo → Barnes-Hut)
- **Week 6 profiling gate**: Decide if grid-based is sufficient or need upgrade

**Performance Targets (MVP: Week 1-8):**
| Node Count | Target FPS | Acceptable | Reach By |
|------------|-----------|-----------|----------|
| 200 | 60 | Mandatory | Week 2 |
| 500 | 45 | Mandatory | Week 4 |
| 1000 | 30+ (usable) | Validation | Week 6 |
| 10,000 | 10-15 | Not required, optional | Week 9+ |

**Definitions:**
- **Mandatory**: MVP will not ship without this
- **Validation**: Good-to-have, validates architecture, may trigger algorithm upgrade
- **Usable**: > 20 fps, graph interaction smooth enough for user testing
- **Reach by**: If not achieved by date, escalate algorithm choice at sprint review

**Memory Targets:**
- < 100 MB for 200 nodes + 2 webviews (baseline)
- < 300 MB for 500 nodes + 2 webviews
- < 1 GB for 1000 nodes + 5 webviews (if reached)

**Benchmark structure:**
```rust
#[bench]
fn bench_physics_200_nodes(b: &mut Bencher) {
    let graph = generate_random_graph(200, 0.05);
    b.iter(|| physics.step(&graph, 1.0/60.0));
}

#[bench]
fn bench_physics_500_nodes(b: &mut Bencher) {
    let graph = generate_random_graph(500, 0.05);
    b.iter(|| physics.step(&graph, 1.0/60.0));
}

#[bench]
fn bench_physics_1000_nodes(b: &mut Bencher) {
    let graph = generate_random_graph(1000, 0.05);
    b.iter(|| physics.step(&graph, 1.0/60.0));
}
```

**Profiling Strategy (Week 6):**
1. Run benches on target hardware (Windows, medium gaming PC)
2. If 1000 nodes < 30fps, profile to find bottleneck:
   - Physics step time? (→ upgrade to `kiddo`)
   - Egui rendering? (→ optimize graph draw calls)
   - Memory bandwidth? (→ reduce node state, use pooling)
3. Decision: Continue with grid-based, or commit to kd-tree + custom forces
4. Document decision + rationale for Phase 2+

**Measured separately:**
- Physics step time
- Graph rendering time
- Total frame time
- Memory usage per node
- GC pause time (if relevant)

---

## 15. Property-Based Testing

**Decision:** Use `proptest` for invariant validation.

**Rationale:**
- Physics always stabilizes (velocity → 0)
- Position never NaN or explode
- Graph operations don't corrupt structure
- Random topologies don't break physics

**Example:**
```rust
proptest! {
    #[test]
    fn physics_stabilizes(
        nodes in 10..1000usize,
        edge_ratio in 0.01f32..0.1,
    ) {
        let mut graph = generate_graph(nodes, edge_ratio);
        physics.step(&graph, 1.0/60.0) for 600 frames;
        assert!(avg_velocity < 0.001);
    }
}
```

---

## 16. Visual Regression Testing

**Decision:** Pixel-diff tests for graph rendering.

**Rationale:**
- Render graph with 100 nodes → compare to golden image
- Detect regressions: Node positions, edge rendering, styling
- Allow small diffs (anti-aliasing, RNG)

**Implementation:**
```rust
#[test]
fn visual_regression_100_nodes() {
    let graph = load_fixture("100_nodes_10_origins.json");
    let pixels = render_graph(&graph, &camera, 800, 600);
    let reference = load_image("golden/100_nodes.png");
    let diff = pixel_diff(&pixels, &reference);
    assert!(diff < 0.02);  // < 2% diff
}
```

---

## 17. Graph Rendering: Retained-Mode Tessellation

**Decision:** Use egui's retained tessellation for static graph elements. Only re-tessellate dynamic elements.

**Rationale:**
- **egui caches tessellation**: Shape primitives compiled to vertex buffers, reused across frames
- **Static elements** (edges, labels, icons): Tessellate once, cache until graph topology changes
- **Dynamic elements** (node positions during physics): Re-tessellate every frame
- **Selective invalidation**: Track which nodes moved, only update those vertex buffers
- **Memory efficient**: Cached meshes ~10KB per 100 nodes

**Implementation:**
```rust
pub struct GraphRenderer {
    edge_mesh_cache: HashMap<EdgeKey, Mesh>,
    static_mesh_dirty: bool,
}

fn render_frame(&mut self, ui: &mut egui::Ui, graph: &Graph, physics_active: bool) {
    if self.static_mesh_dirty {
        // Rebuild edge meshes (topology changed)
        self.rebuild_edge_meshes(graph);
        self.static_mesh_dirty = false;
    }
    
    // Always render node positions (may have moved)
    for node in &graph.nodes {
        ui.put(node.position, self.get_node_widget(node));
    }
}
```

**Note:** This is egui-specific, not WebRender display lists (see Section 2 for rendering architecture).

---

## 18. Undo/Redo: Command Pattern

**Decision:** Command pattern with undo/redo stacks. Session history for disaster recovery.

**Rationale:**
- Every user action is a Command (undoable, redoable)
- Stack-based: Undo pops from undo_stack, pushes to redo_stack
- Persistence: Save snapshots every 10 commands to disk
- Recovery: Last 10 snapshots retrievable if needed

**Alternative considered:** Version control (Git-style) - overkill for this.

---

## 19. Async Loading: Non-Blocking Metadata

**Decision:** Background thread fetches metadata. UI shows loading spinner, doesn't block interaction.

**Rationale:**
- Favicon fetch: ~100ms per node
- Page title scrape: ~500ms per node
- Thumbnail generation: ~2s per node
- Can't block UI for these
- User shouldn't wait for background fetches to open a node

**Implementation:**
```rust
// Background thread
spawn_thread(|| {
    for (node_key, url) in nodes_needing_metadata {
        let title = fetch_title(url);
        tx.send(MetadataUpdate::TitleLoaded(node_key, title));
    }
});

// Main thread (non-blocking)
while let Ok(update) = rx.try_recv() {
    match update {
        MetadataUpdate::TitleLoaded(key, title) => {
            graph.node_mut(key).metadata.title = title;
        }
    }
}
```

**Loading state:** Node shows spinner while loading, no interaction blocked.

---

## 20. Untrusted Data: Input Validation & Sanitization

**Decision:** Sanitize user-visible data. Validate URLs. Trust Servo for webview sandboxing.

**Rationale:**
- Node labels from untrusted sources (page title): Sanitize before display
- URLs: Validate scheme (http, https, file only)
- Metadata from open graph tags: Sanitize before display
- Webview content: Run in sandboxed Servo process (trusted architecture)
- No direct DOM inspection (use Servo's safe interfaces)

**Implementation:**
```rust
fn sanitize_label(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.".contains(*c))
        .collect()
}

fn validate_url(url: &str) -> Result<Url> {
    let parsed = Url::parse(url)?;
    match parsed.scheme() {
        "http" | "https" | "file" => Ok(parsed),
        _ => Err(InvalidScheme),
    }
}
```

---

## 21. Process Isolation: Shell Layer Separate from Content

**Decision:** Graph UI runs in servoshell's shell layer (trusted). Webviews run in sandboxed Servo content processes (untrusted).

**Rationale:**
- **Shell layer** = main servoshell process (egui UI, graph data, user input)
- **Content processes** = Sergio's sandboxed processes (gaol on macOS/Linux, seccomp on Linux)
- **Crash isolation**: Content process crash doesn't affect shell or other webviews
- **Untrusted data isolation**: URLs, page titles sanitized before display in shell layer
- **No direct DOM access**: Shell communicates with content via Servo's IPC (ipc-channel)

**Architecture:**
```
┌─────────────────────────────────────────┐
│  Shell Layer (Trusted)                  │
│  - Graph UI (egui)                      │
│  - User input handling                  │
│  - Graph persistence                    │
│  - Sanitization/validation              │
└─────────────────┬───────────────────────┘
                  │ IPC (ipc-channel)
        ┌─────────┴─────────┬─────────────┐
        │                   │             │
  ┌─────▼──────┐     ┌─────▼──────┐     │
  │ Content    │     │ Content    │    ...
  │ Process 1  │     │ Process 2  │
  │ (Sandboxed)│     │ (Sandboxed)│
  └────────────┘     └────────────┘
```

**Launch:**
```bash
cargo run --release -- -M -S
# -M: Multiprocess (Servo spawns content processes)
# -S: Sandbox (gaol, seccomp applied to content)
```

**Note:** "Compositor" in Servo refers to WebRender's internal compositor, not the shell layer.

---

## 22. Future Architecture: Modularity for P2P/Sync

**Decision:** Design for modularity, but don't implement P2P/sync in MVP.

**Rationale:**
- Phase 1: Local-only (MVP)
- Phase 2+: Optional modules
  - Local sync (file-based)
  - P2P sync (YaCy-style, Syncthing-like)
  - Distributed storage (IPFS, Arweave)
  - Token system (only if needed for incentives)
- Trait-based design allows swappable backends

**Example:**
```rust
pub trait SyncBackend {
    fn push(&self, graph: &Graph) -> Result<()>;
    fn pull(&self) -> Result<Graph>;
    fn merge(&self, local: &Graph, remote: &Graph) -> Result<Graph>;
}

impl SyncBackend for LocalFilesystem { }
// Later: impl SyncBackend for P2PSync { }
```

---

## 23. 3D Graph View: Deferred

**Decision:** 3D is not a priority. Can be added as Phase 4+ optional feature.

**Rationale:**
- Force-directed 2D is sufficient for MVP
- 3D adds complexity without clear benefit for web browsing
- If implemented: Would be optional layout mode, not default
- Library: `three-rs` or similar if ever needed

---

## 24. VERSE Tokenization: Deferred

**Decision:** Tokenization is Phase 3+ research. Not part of MVP.

**Rationale:**
- MVP focus: Spatial UX for single user
- Phase 3: Only if P2P sync is proven useful
- Tokens needed only if incentivizing storage contributions
- Ecosystem question, not core feature

**Architecture supports it:** Modular design allows token system later without major refactors.

---

## Summary: MVP Critical Path

**Weeks 1-2:** Architecture study, Servo understanding, origin-based process plan
**Weeks 3-5:** Graph model, physics, adjacency list, metadata in-struct
**Weeks 6-8:** UI, keybinds, search, validation, sanitization
**Week 9 (Validation):** User testing - does spatial UX work?

**If validation succeeds:** Phase 2 (performance, advanced features)
**If validation fails:** Fallback to linear cluster strip, graph as optional view

---

## Key Principles

1. **Simplicity first:** MVP should be understandable, not feature-complete
2. **Leverage Servo:** Don't reimplement what Servo already provides (multiprocess, rendering, sandboxing)
3. **UX validation early:** Week 9 determines if spatial graph is actually usable
4. **Graceful degradation:** Memory pressure, physics stability, edge clutter all have fallbacks
5. **Accessibility-first:** Keyboard navigation and screen reader support from Week 1
6. **Testability:** Performance, property-based, visual regression all measured
7. **Modularity:** Future P2P/sync possible without architecture change
