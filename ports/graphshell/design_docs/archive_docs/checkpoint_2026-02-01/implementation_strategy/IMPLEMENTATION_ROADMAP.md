# Graphshell: Implementation Roadmap

## Vision
A spatial browser that represents webpages as nodes in a force-directed graph, built on Servo, designed for research and sense-making workflows.

## Foundation Decision
**Start with servoshell** (not Graphshell codebase). Reasons:
- Current with Servo main branch (no 30-60 hour update debt)
- Has multiprocess support built-in (`-M` flag)
- Already has egui integration
- Clean WebViewCollection pattern
- Active upstream maintenance

See [SERVOSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md](SERVOSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md) for full analysis.

---

## Servo Update Budget
Servo is a moving target. Budget time for integration churn.

- Reserve 10-15% of each phase for Servo API changes and integration fixes
- Pin a Servo revision per milestone; rebase monthly
- Keep a thin adapter layer around servoshell APIs to localize breakage

---

## Architecture: Single Crate, Feature Flags

### Servo's Existing Architecture

Servo already provides the foundation:
- **Immutable tree pipeline**: DOM → Style (Stylo) → Layout (Taffy) → Paint → WebRender Display List → Compositor
- **Multiprocess**: EventLoops (origin-grouped) run in separate processes via `-M` flag
- **Sandboxing**: gaol library provides macOS/Linux sandboxing via `-S` flag
- **Display lists**: WebRender consumes display lists, handles GPU rendering
- **IPC**: `ipc-channel` handles all cross-process communication
- **Threading**: Script/Layout/Paint threads run independently

**Your graph browser adds a layer on top:**
- Graph model (nodes/edges) → Physics (force-directed layout) → egui draw primitives
- Composited in servoshell's shell layer alongside WebRender surfaces
- Webviews managed via servoshell's `WebViewCollection` pattern

### Graph Browser Architecture

```
graphshell-graph/
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # Application state machine
│   ├── config/              # Configuration
│   │   ├── mod.rs           # Config loading/saving
│   │   └── keybinds.rs      # Keybind configuration
│   ├── graph/               # Graph data structures
│   │   ├── mod.rs           # Graph, Node, Edge
│   │   ├── spatial.rs       # SpatialGrid (hash grid) for O(n) avg queries
│   │   └── persistence.rs   # Snapshots + append-only log
│   ├── physics/             # Force-directed layout
│   │   ├── mod.rs           # PhysicsEngine
│   │   └── spatial_hash.rs  # O(n) force calculation
│   ├── render/              # Rendering with LOD
│   │   ├── mod.rs           # Renderer
│   │   ├── batch.rs         # Batched drawing
│   │   └── egui.rs          # egui backend
│   ├── browser/             # Servo integration
│   │   ├── mod.rs           # ProcessManager
│   │   ├── process_manager.rs # Lifecycle + lightweight reuse pool
│   │   └── servo.rs         # Servo-specific code
│   ├── input/               # Input handling
│   │   ├── mod.rs           # Event routing via keybinds
│   │   └── camera.rs        # Camera controller
│   ├── ui/                  # Browser chrome
│   │   ├── mod.rs           # UI state
│   │   ├── clusterbar.rs    # Cluster strip (linear projection of graph)
│   │   ├── omnibar.rs       # Search/navigation
│   │   └── sidebar.rs       # Bookmarks/downloads/settings
│   └── features/            # Browser features
│       ├── bookmarks.rs     # Bookmark manager
│       ├── downloads.rs     # Download manager
│       └── storage.rs       # Persistence layer
└── Cargo.toml

[features]
default = ["multiprocess"]
multiprocess = []  # Enable Servo's -M flag
```

**Design principles:**
- Single crate initially (split later if publishing components)
- Feature flags for conditional compilation
- SlotMap for stable node handles
- Spatial indexing for performance
- Active/Warm/Cold lifecycle + small reuse pool (not one-per-node)

---

## Phase 1A: Core Graph Browser (Weeks 1-4)

### Milestone 1.1: Servoshell Foundation (Week 1)
**Goal:** Get servoshell building and understand its architecture

**Tasks:**
- [ ] Fork/copy servoshell into `graphshell/`
- [ ] Verify builds: `cargo build --release`
- [ ] Run: `./target/release/graphshell https://example.com`
- [ ] Test multiprocess: `./target/release/graphshell -M https://example.com`
- [ ] Study Servo's architecture:
  - `components/constellation/pipeline.rs` - Pipeline abstraction (frame/window)
  - `components/constellation/event_loop.rs` - EventLoop spawning, multiprocess
  - `components/paint/paint.rs` - WebRender integration, display lists
  - `components/constellation/sandboxing.rs` - gaol sandboxing profiles
- [ ] Study servoshell key files:
  - `desktop/app.rs` - ApplicationHandler pattern
  - `window.rs` - ServoShellWindow
  - `running_app_state.rs` - WebViewCollection
  - `desktop/gui.rs` - egui integration

**Deliverable:** Working servoshell clone, documented understanding of Servo's layers

---

### Milestone 1.2: Headless Graph View (Week 2)
**Goal:** Replace servoshell's default UI with a graph canvas

**Tasks:**
- [ ] Create `graph/` module with basic structures:
  ```rust
  pub struct Node {
      pub id: NodeKey,
      pub url: String,
      pub position: Point2D<f32>,
      pub velocity: Vector2D<f32>,
  }
  
  pub struct Graph {
      nodes: SlotMap<NodeKey, Node>,
      edges: Vec<Edge>,
  }
  ```
- [ ] Add `app.rs` with view state:
  ```rust
  enum View {
      Graph,           // Default view
      Detail(NodeKey), // Focused node with cluster strip
  }
  ```
- [ ] Render hardcoded graph (5 nodes) with egui:
  - Circles for nodes
  - Lines for edges
  - No physics yet (static positions)
- [ ] Add headless screenshotting (offscreen render) for Warm node thumbnails
- [ ] Add basic camera (pan only, no zoom)
- [ ] Remove servoshell's default tab bar in Graph view

**Deliverable:** App opens to graph view showing 5 static nodes

---

### Milestone 1.3: Grid-Based Physics + Worker Thread (Week 3)
**Goal:** Nodes move according to forces without blocking UI

**Tasks:**
- [ ] Create `physics/` module:
  ```rust
  pub struct PhysicsEngine {
      repulsion_strength: f32,
      spring_strength: f32,
      damping: f32,
      grid: SpatialGrid,
  }
  
  impl PhysicsEngine {
      pub fn step(&mut self, graph: &mut Graph, dt: f32) {
          // Grid-based repulsion (O(n) average)
          // Hooke's law springs on edges
          // Velocity damping + integration
      }
  }
  ```
- [ ] Run physics on a dedicated worker thread
- [ ] Send position updates to UI via `crossbeam` channel
- [ ] Add scale-normalized velocity threshold for auto-pause
- [ ] Add UI controls:
  - Toggle physics on/off (T key)
  - Adjust damping/strength sliders
- [ ] Week 6 evaluation gate: if 1000 nodes < 30fps, consider `kiddo` or Barnes-Hut

**Deliverable:** Animated graph with smooth UI responsiveness

---

### Milestone 1.4: Node-Browser Integration + Keybinds (Week 4)
**Goal:** Configurable interaction with nodes, view toggle, resizable split-view

**Tasks:**
- [ ] Create keybind config system:
  ```rust
  // config/keybinds.rs
  pub struct KeybindConfig {
      pub node_focus: KeyAction,        // Default: DoubleClick or Enter
      pub node_select: KeyAction,       // Default: Click
      pub multi_select: KeyAction,      // Default: Shift+Click
      pub toggle_view: KeyAction,       // Default: Home button or Escape
      pub new_node: KeyAction,          // Default: N
      pub delete_node: KeyAction,       // Default: Delete
      pub center_graph: KeyAction,      // Default: C
      pub toggle_physics: KeyAction,    // Default: T
      // ... more keybinds (see ARCHITECTURE_DECISIONS.md)
  }
  
  impl Default for KeybindConfig {
      fn default() -> Self {
          // Sensible defaults, all user-overrideable
      }
  }
  ```
- [ ] Load/save keybinds: `~/.config/graphshell/keybinds.toml`
- [ ] Implement view toggle (graph ↔ detail):
  - Full-screen toggle (graph hides completely, detail takes full window)
  - OR resizable split-view (default: 60% detail, 40% graph)
  - Remember user's preferred split ratio
  - Home button (left of omnibar) toggles between them
- [ ] Implement origin-based webview management:
  - Use Servo's `-M` origin grouping for process isolation
  - Add Active/Warm/Cold lifecycle (cap Active at 20)
  - Warm nodes render last thumbnail; Cold nodes are metadata only
  - Lightweight reuse pool (2-4) for recently used origins (TTL 30-60s)
  - Memory reaper (sysinfo): demote Warm/Active on pressure
- [ ] Wire up default interactions:
  - Single click: Select node (highlight)
  - Double click or Enter: Open node → switch to detail view
  - Escape: Return to graph view
  - In detail view: Cluster strip shows connected nodes (linear projection)
  - Pinned clusters: Show pin icon on corresponding graph node

**Deliverable:** View toggle works smoothly, origin-based processes spawn/die as expected, keybinds are configurable

---

## Phase 1B: Usability + Persistence (Weeks 5-8)

### Milestone 1.5: Camera & Navigation (Week 5)
**Goal:** Smooth camera controls

**Tasks:**
- [ ] Implement `Camera`:
  ```rust
  pub struct Camera {
      position: Point2D<f32>,
      zoom: f32,
      target: Point2D<f32>,  // For smooth interpolation
  }
  
  impl Camera {
      pub fn world_to_screen(&self, pos: Point2D<f32>) -> Point2D<f32>;
      pub fn screen_to_world(&self, pos: Point2D<f32>) -> Point2D<f32>;
      pub fn smooth_move(&mut self, dt: f32);  // Lerp to target
  }
  ```
- [ ] Implement controls:
  - **WASD / Arrow keys:** Pan camera
  - **Mouse wheel:** Zoom
  - **Middle-mouse drag:** Pan
  - **Double-click node:** Center camera + switch to Detail view
- [ ] Add bounds (don't pan outside graph extent)
- [ ] Add smooth interpolation (ease in/out)

**Deliverable:** Comfortable navigation, feels polished

---

### Milestone 1.6: Advanced Interactions (Week 6)
**Goal:** Context menu, drag nodes, marquee select, edge type visualization

**Tasks:**
- [ ] Accessibility baseline:
  - Screen reader labels for nodes and selection state
  - Reduced motion toggle for physics
- [ ] Add multi-select patterns (via keybinds):
  - Shift+click: Multi-select
  - Drag: Marquee select (rubber band)
  - Click empty space: Deselect all
- [ ] Add node dragging:
  - Click+drag selected node: Move it (disable physics temporarily)
  - Release: Physics resumes
  - Pin mode: Right-click → "Pin" (disable physics permanently)
- [ ] Add context menu (right-click):
  - Open in new node
  - Delete node
  - Pin/Unpin
  - Create edge to...
  - Inspect (show URL, title, metadata)
  - Copy URL
- [ ] Add edge type colors (user-selectable):
  - Load from config: `~/.config/graphshell/preferences.toml`
  - Edge types: Hyperlink (blue), Bookmark (green), History (gray), Manual (red)
  - Line styles: Solid, dotted, bold, marker for colorblind accessibility
  - Settings UI (Phase 2): Color picker, preset themes (light, dark, colorblind-friendly)
- [ ] Add keybind editor UI:
  - Settings → Keybinds
  - Show current bindings
  - Click to rebind
  - Reset to defaults button
  - Smart conflict resolution: If user rebinds key A from action X to action Y, action X rebinds to its previous key or default

**Deliverable:** Rich interaction, customizable keybinds and edge colors, feels responsive

---

### Milestone 1.7: Persistence (Week 7)
**Goal:** Save and load graphs

**Tasks:**
- [ ] Implement snapshot + append-only log model:
  - Snapshot every 30s or 20 commands
  - Append log on every command (single-line JSON)
  - Recovery = last snapshot + log replay
- [ ] Implement JSON snapshot schema:
  ```json
  {
    "version": "1.0",
    "nodes": [
      {
        "id": "node-abc123",
        "url": "https://example.com",
        "position": {"x": 100, "y": 200},
        "pinned": false,
        "metadata": {
          "title": "Example Domain",
          "visited_at": "2026-02-03T10:00:00Z"
        }
      }
    ],
    "edges": [
      {"from": "node-abc123", "to": "node-def456", "type": "link"}
    ],
    "camera": {"position": {"x": 0, "y": 0}, "zoom": 1.0}
  }
  ```
- [ ] Add file operations:
  - `Ctrl+S`: Save snapshot now
  - `Ctrl+O`: Open graph
  - `Ctrl+N`: New graph
- [ ] Store in `~/.config/graphshell/sessions/`
- [ ] Add "Recent graphs" menu
- [ ] Validate recovery: skip malformed log line, warn user

**Deliverable:** Persistent workflow, can close and resume

---

### Milestone 1.8: Search & Filter (Week 8)
**Goal:** Omnibar for search and navigation

**Tasks:**
- [ ] Add omnibar (Ctrl+F or click top bar):
  ```
  [🔍 Search nodes, add URL, or command... ]
  ```
- [ ] Implement search modes:
  - Type URL → Create node + navigate there
  - Type text → Fuzzy filter by title/URL/tags (`fuzzy-matcher`)
  - Type `/command` → Execute command (e.g., `/physics off`)
- [ ] Filter display:
  - Matching nodes: Full opacity
  - Non-matching: 20% opacity or hidden
  - Highlight matches in node labels
- [ ] Add quick commands:
  - `/center` - Center camera on selected
  - `/cluster` - Run auto-clustering
  - `/export` - Export as PNG/JSON
  - `/settings` - Open settings

- [ ] Roadmap note:
  - Phase 2: Full-text indexing (optional, `tantivy`)
  - Phase 3: Semantic search (optional)

**Deliverable:** Fast navigation, feels like a power tool

---

## Phase 2: Performance & Multiprocess (Weeks 9-12)

### Milestone 2.1: Spatial Optimization (Week 9)
**Goal:** Reach tiered performance targets (200@60fps, 500@45fps, 1000@30+)

**Tasks:**
- [ ] Tune grid-based spatial hash (cell size, neighbor radius)
- [ ] Profile physics vs render time on target hardware
- [ ] If 1000 nodes < 30fps, evaluate `kiddo` or Barnes-Hut
- [ ] Add QuadTree for viewport culling:
  ```rust
  pub struct QuadTree<T> {
      // Only render nodes in viewport
  }
  ```
- [ ] Implement LOD (Level of Detail):
  - Zoom < 0.5: Nodes as 2px dots
  - Zoom 0.5-2.0: Nodes as circles
  - Zoom > 2.0: Nodes with favicons + labels
- [ ] Add performance monitoring:
  - FPS counter
  - Node count
  - Physics time
  - Render time

**Deliverable:** Tiered targets met or upgrade decision documented

---

### Milestone 2.2: Origin-Based Process Management (Week 10)
**Goal:** Leverage Servo's origin-grouped multiprocess (already implemented!)

**Tasks:**
- [ ] Verify multiprocess mode is working:
  ```bash
  cargo run --release -- -M -S https://example.com
  # -M: multiprocess mode (origin-grouped)
  # -S: sandbox mode (gaol on macOS/Linux)
  ```
- [ ] Verify origin grouping:
  - Create nodes from 3 origins (e.g., example.com, wikipedia.org, github.com)
  - Check process count: Should be 3 processes (one per origin)
  - Create more nodes from example.com: Still 1 process for example.com
  - Nodes from same origin share the same process (memory efficient)
- [ ] Implement lightweight reuse pool (2-4) with TTL for recently used origins
- [ ] Implement memory pressure handler:
  - Demote Warm nodes first, then unpinned Active nodes
  - Keep focused + pinned nodes active
- [ ] Test crash isolation:
  - Navigate one node to crash-test page
  - Webview process crashes, Servo respawns it
  - Graph UI survives (shell layer unaffected)
  - Other nodes from same origin may be affected (acceptable, all re-spawn)
- [ ] Study Servo's process lifecycle:
  - Read `constellation/event_loop.rs` to understand process spawning
  - Understand origin grouping (same-origin nodes share process)
  - Understand sandboxing profiles (gaol on macOS, seccomp on Linux)
- [ ] Add optional process monitoring UI (Phase 2):
  - Show which origins are running (e.g., "example.com (3 nodes)")
  - Display process memory usage
  - Allow manual restart (rare, for stuck processes)

**Deliverable:** Crash-resistant with origin-based process management via Servo's built-in system

**Key insight:** Servo's `-M` flag provides isolation, but Graphshell still needs lifecycle + reuse for UX.

---

### Milestone 2.3: Node Grouping (Week 11)
**Goal:** Visual clustering based on zoom/domain

**Tasks:**
- [ ] Implement zoom-based aggregation:
  - Zoom < 0.3: Group nodes by domain (e.g., "wikipedia.org (15 nodes)")
  - Zoom 0.3-0.8: Group by subdomain
  - Zoom > 0.8: Show individual nodes
- [ ] Add clustering visualization:
  - Group rendered as larger circle
  - Click to expand (zoom in + focus)
  - Number badge shows count
- [ ] Optional: Use petgraph for semantic clustering:
  ```rust
  #[cfg(feature = "algorithms")]
  pub fn auto_cluster(&self) -> Vec<Cluster> {
      let components = petgraph::algo::tarjan_scc(&self.graph);
      // Return strongly connected components
  }
  ```

**Deliverable:** Navigate thousands of nodes efficiently

---

### Milestone 2.4: Webview Optimization (Week 12)
**Goal:** Minimize memory with lifecycle + lightweight reuse

**Tasks:**
- [ ] Tune reuse pool:
  - Default: 2-4 warm origin processes
  - Strategy: Keep focused + neighbors Active; demote others to Warm/Cold
- [ ] Implement lazy loading:
  - Nodes start with "title + favicon only"
  - Webview created only when:
    - Node is focused (Detail view)
    - Node is neighbor of focused (preload)
    - User explicitly clicks "Load"
- [ ] Implement thumbnail atlas for Warm nodes to reduce GPU memory
- [ ] Add webview lifecycle UI:
  - Node badge shows state:
    - Gray: Not loaded
    - Yellow: Loading
    - Green: Loaded
    - Red: Crashed
  - Right-click → "Load/Unload/Reload"

**Deliverable:** Memory stable at tiered targets, Active/Warm/Cold behavior validated

---

### Milestone 2.5: Import/Export v0 (Week 12)
**Goal:** Enable graph interchange and onboarding from existing browsers

**Tasks:**
- [ ] Define versioned graph JSON schema (nodes, edges, anchors, regions)
- [ ] Export graph as versioned JSON (latest schema)
- [ ] Import graph JSON with compatibility rules (ignore unknown fields)
- [ ] Export current view as PNG/SVG
- [ ] Import bookmarks (Netscape bookmarks.html)
- [ ] Export nodes/edges as CSV for analysis
- [ ] Add UI hooks in omnibar: `/import` and `/export`
- [ ] Add validation and error reporting (schema mismatch, missing URLs)

**Deliverable:** Users can move data in and out of Graphshell

---

## Phase 3: Browser Features (Weeks 13-16)

### Milestone 3.1: Bookmark Manager (Week 13)
**Goal:** Manage bookmarks, import from browsers

**Tasks:**
- [ ] Create bookmark storage:
  ```rust
  pub struct Bookmark {
      title: String,
      url: Url,
      tags: Vec<String>,
      created_at: DateTime<Utc>,
  }
  ```
- [ ] Add bookmark UI:
  - Sidebar panel (Ctrl+B)
  - List view with search
  - Folder tree (optional)
  - Add current node (Ctrl+D)
  - Import from Chrome/Firefox bookmarks.html
- [ ] Persist to `~/.config/graphshell/bookmarks.json`
- [ ] Add to omnibar:
  - Type bookmark name → create node from bookmark

**Deliverable:** Bookmark workflow integrated with graph

---

### Milestone 3.2: Download Manager (Week 14)
**Goal:** Track downloads, show progress

**Tasks:**
- [ ] Listen for Servo download events:
  ```rust
  impl ServoDelegate for App {
      fn on_download(&self, url: &str, filename: &str) {
          self.downloads.add(Download::new(url, filename));
      }
  }
  ```
- [ ] Create downloads panel:
  - Sidebar tab (Ctrl+J)
  - List with progress bars
  - Actions: Pause/Resume/Cancel/Open
  - History of completed downloads
- [ ] Save location: `~/Downloads/graphshell/`
- [ ] Persist history: `~/.config/graphshell/downloads.json`

**Deliverable:** Modern browser download experience

---

### Milestone 3.3: Storage Manager (Week 15)
**Goal:** Manage graphs, history, cache

**Tasks:**
- [ ] Create storage panel (Settings → Storage):
  - Show disk usage:
    - Graphs: X MB
    - Cache: Y MB
    - Bookmarks/Downloads: Z MB
  - Clear buttons per category
  - Export/Import graphs
- [ ] Add graph library:
  - List all saved graphs
  - Preview thumbnail (screenshot of graph)
  - Rename/Delete/Duplicate
  - Open in new window
- [ ] Session history:
  - Track visited nodes (like browser history)
  - Search history
  - Clear history

**Deliverable:** Professional storage management

---

### Milestone 3.4: Document Type Support (Week 16)
**Goal:** Render more than just HTML

**Tasks:**
- [ ] Add document type detection:
  ```rust
  match content_type {
      "application/pdf" => open_pdf_viewer(url),
      "image/*" => open_image_viewer(url),
      "text/plain" => open_text_viewer(url),
      "video/*" => open_video_player(url),
      _ => open_in_servo(url),
  }
  ```
- [ ] Integrate viewers:
  - **PDF:** Use `pdf-rs` or external viewer
  - **Images:** Native image viewer with zoom
  - **Text:** Syntax-highlighted text editor
  - **Video:** GStreamer (Servo already has this)
  - **Markdown:** Render with `pulldown-cmark`
- [ ] Add preview in graph nodes:
  - PDF: First page thumbnail
  - Image: Thumbnail
  - Video: First frame

**Deliverable:** Universal document browser

---

### Milestone 3.5: Extension API v0 (Week 16)
**Goal:** Minimal, safe extension surface for graph workflows

**Tasks:**
- [ ] Define extension manifest schema (id, version, entry, permissions, commands)
- [ ] Implement permission checks for graph read/write and file import/export
- [ ] Provide graph query API (read-only by default)
- [ ] Allow extensions to add nodes/edges with explicit permission
- [ ] Register omnibar commands (`/ext ...`)
- [ ] Add event hooks: node opened, node created, edge created, selection changed
- [ ] Sandbox untrusted extensions via WASM (wasmtime) with CPU/memory limits
- [ ] Add extension loading UI (enable/disable, permission prompts)

**Deliverable:** Extensions can automate graph workflows without compromising safety

---

## Phase 4: Polish & Hardening (Weeks 17-24)

### Milestone 4.1: UI Polish (Weeks 17-18)
- [ ] Smooth animations (fade in/out, ease transitions)
- [ ] Dark/light theme toggle
- [ ] Customizable colors (nodes, edges, background)
- [ ] Keyboard shortcut cheatsheet (F1 or Ctrl+?)
  - Shows current keybinds
  - Searchable
  - Click to rebind
- [ ] Tooltips and hints for discoverable interactions
- [ ] Accessibility (screen reader support, high contrast mode, reduced motion)
- [ ] Minimap (Phase 2 if needed):
  - Shows all nodes in viewport
  - Highlights current viewport
  - Click to navigate
- [ ] Visual polish:
  - Node shadows, glow on selection
  - Edge glow on hover
  - Smooth transitions when toggling views

### Milestone 4.2: Advanced Features (Weeks 19-20)
- [ ] 3D graph view (optional, via WGPU)
- [ ] Export options (PNG, SVG, JSON)
- [ ] Share graph via URL (serialize to base64)
- [ ] Graph templates (research, reading list, project)
- [ ] Tag system (color-code by tags)

### Milestone 4.3: Testing & Benchmarks (Weeks 21-22)
- [ ] Unit tests for all modules:
  - Graph operations (add, delete, edges)
  - Physics stability (converges, no NaN)
  - Serialization (save/load roundtrip)
  - Keybind resolution (conflicts handled)
- [ ] Property-based tests (proptest):
  - Physics always stabilizes for any topology
  - Positions never NaN or explode
  - Graph operations preserve invariants
- [ ] Performance benchmarks:
  - 200 nodes, 60fps: Physics + render < 16ms
  - 500 nodes, 45fps: Physics + render < 22ms
  - 1000 nodes, 30fps: Physics + render < 33ms
  - Serialization: 10K graph < 500ms
- [ ] Visual regression tests:
  - Render golden graphs, compare pixel diff
  - Edge styles and node positioning
- [ ] User testing (Week 9 validation + final):
  - 5-10 target users
  - Can they understand spatial UX?
  - Do they prefer graph over a linear cluster strip?
  - Accessibility testing (1-2 screen reader users)

---

## Complete Keybind List

### Navigation (9)
| Action | Default | Context |
|--------|---------|----------|
| Pan up | Up arrow or W | Graph view |
| Pan down | Down arrow or S | Graph view |
| Pan left | Left arrow or A | Graph view |
| Pan right | Right arrow or D | Graph view |
| Zoom in | Ctrl+Scroll or + | Graph view |
| Zoom out | Ctrl+Scroll or - | Graph view |
| Center graph | C | Graph view |
| Next cluster | Ctrl+Tab | Detail view |
| Prev cluster | Ctrl+Shift+Tab | Detail view |

### Graph Editing (8)
| Action | Default | Context |
|--------|---------|----------|
| New node | N | Graph view |
| Delete selected | Delete | Graph view |
| Select all | Ctrl+A | Graph view |
| Deselect all | Escape / Click empty | Graph view |
| Toggle physics | T | Graph view |
| Pin/unpin selected | Ctrl+D | Graph view |
| Multi-select | Shift+Click | Graph view |
| Drag node | Click+drag | Graph view |

### Undo/Redo & File (6)
| Action | Default | Context |
|--------|---------|----------|
| Undo | Ctrl+Z | All |
| Redo | Ctrl+Y or Ctrl+Shift+Z | All |
| Save session | Ctrl+S | All |
| Open session | Ctrl+O | All |
| New session | Ctrl+N | All |
| Save as / Export | Ctrl+Shift+S | All |

### Search & Filter (3)
| Action | Default | Context |
|--------|---------|----------|
| Search/filter | Ctrl+F | Graph view |
| Toggle layout mode | Ctrl+/ | Graph view |
| Search history | Ctrl+` | Graph view |

### Sidebar & UI (6)
| Action | Default | Context |
|--------|---------|----------|
| Toggle bookmarks | Ctrl+B | All |
| Toggle downloads | Ctrl+J | All |
| Toggle tags sidebar | Ctrl+; | All |
| Settings | Ctrl+, | All |
| Help / Keybinds | F1 or Ctrl+? | All |
| Focus omnibar | Ctrl+L | All |

### Detail View / Webview (4)
| Action | Default | Context |
|--------|---------|----------|
| Reload page | Ctrl+R or F5 | Detail view |
| Hard reload | Ctrl+Shift+R | Detail view |
| Back | Alt+Left | Detail view |
| Forward | Alt+Right | Detail view |

### View Toggle (1)
| Action | Default | Context |
|--------|---------|----------|
| Graph ↔ Detail | Home / Escape | All |

**Total: ~36 default keybinds.** All user-overrideable in Settings.



| Metric | Target | Measurement |
|--------|--------|-------------|
| **Physics FPS** | 60fps @ 200 nodes, 45fps @ 500, 30fps @ 1000 | Grid-based hash, optional Barnes-Hut |
| **Render FPS** | 60fps @ 200 nodes, 45fps @ 500, 30fps @ 1000 | LOD + culling |
| **Startup time** | < 2 seconds | Lazy initialization |
| **Memory** | < 100MB @ 200 nodes, < 300MB @ 500 | Active/Warm/Cold + reuse pool |
| **Save time** | < 1 second for 10,000 nodes | Fast JSON serialization |

---

## Technology Stack

| Component | Technology | Why |
|-----------|-----------|-----|
| **Language** | Rust | Performance, safety |
| **Browser engine** | Servo | Modern, multiprocess, WebRender |
| **UI framework** | egui | Immediate mode, fast (already in servoshell) |
| **Windowing** | winit | Cross-platform (Servo uses this) |
| **GPU rendering** | WebRender | Servo's display list renderer |
| **Multiprocess** | ipc-channel | Servo's IPC system (built-in) |
| **Sandboxing** | gaol | Servo's sandbox library (macOS/Linux) |
| **Graph storage** | SlotMap | Stable handles, O(1) access |
| **Adjacency list** | HashMap | O(1) neighbor lookup for physics |
| **Spatial index** | SpatialGrid | O(n) average repulsion, viewport culling |
| **Physics** | Custom | Grid-based repulsion, Barnes-Hut if needed |
| **Search** | fuzzy-matcher (Phase 1) | Fast fuzzy search over title/url/tags |
| **Serialization** | serde_json | Standard, readable |
| **Algorithms** | petgraph (optional) | Clustering algorithms (Phase 2+) |

See [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md) for rationale on each choice.

---

## Development Workflow

```bash
# Initial setup
git clone https://github.com/servo/servo
cd servo/ports/servoshell
# Study the code

cd ~/graphshell
cargo init --bin
# Start implementing

# Daily development
cargo check          # Fast compile check
cargo run            # Run app
cargo test           # Run tests
cargo clippy         # Linting

# Before commit
cargo fmt            # Format code
cargo test           # Verify tests pass

# Release build
cargo build --release
./target/release/graphshell-graph
```

---

## Documentation Consolidation

**Keep these docs:**
- ✅ `IMPLEMENTATION_ROADMAP.md` (this file) - Concrete plan
- ✅ `SERVOSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md` - Foundation decision
- ✅ `GRAPHSHELL_SERVO_PARITY_ANALYSIS.md` - Historical reference
- ✅ `WINDOWS_BUILD.md` - Platform-specific setup

**Archive these (move to `archive_docs/`):**
- ⚠️ `GRAPH_BROWSER_MIGRATION.md` - Superseded by this roadmap
- ⚠️ `GRAPH_INTERFACE.md` - Details now in roadmap milestones
- ⚠️ `PROJECT_PHILOSOPHY.md` - Vision is clear, details archived
- ⚠️ `COMPREHENSIVE_SYNTHESIS.md` - Too abstract
- ⚠️ `ARCHITECTURE_MODULAR_ANALYSIS.md` - Over-engineered
- ⚠️ `INTERFACE_EVOLUTION.md` - Speculative
- ⚠️ `README2.md` - Duplicate
- ⚠️ `verse_docs/VERSE.md` - Phase 3+ research, not MVP

**Keep for reference:**
- ✅ `AGENTS.md` - Useful for AI assistance
- ✅ `README.md` - Update to point to this roadmap
- ✅ `README.md` - Includes project status
- ✅ `verse_docs/VERSE.md` - Future vision, not immediate

---

## Next Actions

1. **Week 1, Day 1:**
   - [ ] Copy servoshell to new repo
   - [ ] Verify it builds: `cargo build --release`
   - [ ] Test multiprocess: `cargo run --release -- -M https://example.com`
   - [ ] Study Servo's architecture:
     - Read `components/constellation/pipeline.rs` (Pipeline abstraction)
     - Read `components/constellation/event_loop.rs` (multiprocess spawning)
     - Read `components/paint/paint.rs` (WebRender integration)
   - [ ] Read and understand servoshell's `desktop/app.rs`

2. **Week 1, Day 2:**
   - [ ] Create `graph/mod.rs` with Node/Graph structs
   - [ ] Render 5 hardcoded nodes with egui

3. **Week 1, Day 3:**
   - [ ] Add camera pan (mouse drag)
   - [ ] Remove servoshell's tab bar

4. **Week 1, Day 4:**
   - [ ] Start physics engine
   - [ ] Implement repulsion between nodes

5. **Week 1, Day 5:**
   - [ ] Get physics animating at 60fps
   - [ ] Add physics toggle (T key)

**By end of Week 1:** You should see an animated graph of 5 nodes.

---

## Success Criteria

**MVP Success (Week 8):**
- Can create graph from browsing
- Smooth navigation (WASD, zoom)
- Double-click to view webpage
- Save/load graphs
- Search works
- Feels usable for daily work

**Production Success (Week 24):**
- Handles 1000 nodes at usable FPS; 10,000 is optional/stretch
- Multiprocess for stability
- Bookmarks/downloads integrated
- Can replace regular browser for research workflows
- Shared publicly, has users

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| **Physics too slow** | Grid-based hash + Week 6 profiling; upgrade to `kiddo` or Barnes-Hut if needed |
| **UI feels laggy** | egui proven in servoshell, LOD + display list diffing, 60fps achievable |
| **Servo integration hard** | Budget 10-15% per phase for Servo breakage; adapter layer + pinned revisions |
| **Memory blowup** | Active/Warm/Cold lifecycle + reuse pool + memory reaper on pressure |
| **Spatial UX unproven** | Week 9 validation: Test with 5-10 target users before scaling |
| **Multiprocess complexity** | Servo's `-M` flag provides this; no custom implementation needed |
| **Sandboxing complexity** | Servo's `-S` flag + gaol library handle this on macOS/Linux |
| **Feature creep** | Stick to roadmap, defer Phase 3+ features, explicitly deferred: 3D view, tokenization |
| **Burnout** | 8-week MVP is short; validation gates Phase 2 (forced reflection point) |
| **Edge rendering clutter** | Bezier curves sufficient for MVP; bundled edges available (Week 9) if needed |
| **Display list overhead** | Differential updates (only changed nodes) + separate static/dynamic caches |
| **Undo/redo complexity** | Command pattern is simple; session history provides disaster recovery |
| **Async loading lag** | Background thread for metadata; spinner UI, non-blocking |
| **Untrusted data exploit** | Input sanitization + URL validation; Servo's sandboxing for webviews |
| **Keybind conflicts** | Smart resolution: rebind conflicting action to previous or default binding |
| **Display list performance** | Can use WebRender primitives if egui is too slow |

---

## Questions to Validate During Phase 1B (Week 8-9)

After building MVP, before adding complexity:

1. **Does the graph UX actually work?**
  - Is it faster than a linear cluster strip?
   - Do you actually use it?

2. **What hurts?**
   - Too slow?
   - Missing features?
   - Awkward interactions?

3. **What's surprising?**
   - Unexpected use cases?
   - Better/worse than expected?

4. **Should you continue?**
   - If yes → proceed to Phase 2
   - If no → iterate on core UX first

---

This roadmap is concrete, actionable, and scoped for success. Each week has clear deliverables. Each milestone builds on the previous. By Week 8, you'll have a usable graph browser.
