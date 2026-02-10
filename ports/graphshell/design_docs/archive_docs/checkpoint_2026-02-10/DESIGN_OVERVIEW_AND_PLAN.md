# Design Docs — Overview & Plan (Consolidated)

**Consolidation date**: February 4, 2026  
This file consolidates overview and planning materials. Original file sections are preserved verbatim under “Source” headings. Internal links inside source text may reference legacy filenames.

**DOC_POLICY Notice** (Feb 10, 2026): This file is a legacy consolidation and contains outdated, calendar-based plans and archived references. Treat it as historical context only. For current docs, use:
- **[README.md](README.md)** — Project overview and current status
- **[INDEX.md](INDEX.md)** — Documentation map
- **[ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md)** — Verified implementation status and architecture
- **[IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)** — Feature-driven roadmap

---

## Source: README.md

# Graphshell Design Documentation

Graphshell is a spatial browser that represents webpages as nodes in a force-directed graph. Research tool for sense-making and exploratory workflows.

## 🚀 Start Here

**[INDEX.md](INDEX.md)** — Complete documentation map and reading order  
**[implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md)** — 24-week implementation plan  
**[ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)** — Why we made each architectural choice  
**[implementation_strategy/WEEK1_CHECKLIST.md](implementation_strategy/WEEK1_CHECKLIST.md)** — Day-by-day tasks for Week 1 architecture study

## Foundation Decision (Feb 2026)

**Building on servoshell**, not the Graphshell codebase. Servoshell is current with Servo main, has multiprocess built-in, saves ~30-60 hours update work.

See [SERVOSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md](SERVOSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md) for analysis.

### What Servo Already Provides

Servo's architecture already handles most low-level concerns:
- ✅ **Immutable tree pipeline**: DOM → Style → Layout → Paint → WebRender
- ✅ **Multiprocess**: Origin-grouped processes via `-M` flag
- ✅ **Sandboxing**: gaol library (macOS/Linux) via `-S` flag  
- ✅ **Display lists**: WebRender for GPU rendering
- ✅ **IPC**: ipc-channel handles cross-process communication
- ✅ **Threading**: Script/Layout/Paint run independently

**Your graph browser adds a layer on top**, not a reimplementation.

## Active Documents

- **[CRITICAL_ANALYSIS.md](CRITICAL_ANALYSIS.md)** ⭐ **READ THIS FIRST** — Critical gaps + recommendations
- **[implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md)** ⭐ Week-by-week plan with milestones
- **[ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)** Detailed rationale for every architectural choice
- **[GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)** Browser behavior spec (graph-first UX)
- **[verse_docs/GRAPHSHELL_P2P_COLLABORATION.md](verse_docs/GRAPHSHELL_P2P_COLLABORATION.md)** P2P collaboration & decentralized sync (Verse)
- **[FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md](FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md)** Firefox patterns, Servo integration, cross-domain interactions
- **[SERVERSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md](SERVERSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md)** Foundation decision
- **[GRAPHSHELL_SERVO_PARITY_ANALYSIS.md](GRAPHSHELL_SERVO_PARITY_ANALYSIS.md)** Historical reference (30-60hr update estimate)
- **[BUILD.md](BUILD.md)** Platform-specific build setup
- **[AGENTS.md](AGENTS.md)** AI assistance guidance
- **[verse_docs/VERSE.md](verse_docs/VERSE.md)** Phase 3+ research (tokenization, not MVP)

## Archive (Move to `archive_docs/`)

These docs are superseded by [implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md):
- `GRAPH_INTERFACE.md` → Details now in roadmap milestones
- `GRAPH_BROWSER_MIGRATION.md` → Superseded by roadmap Phase 1-4
- `PROJECT_PHILOSOPHY.md` → Vision clear, archived
- `COMPREHENSIVE_SYNTHESIS.md` → Too abstract
- `ARCHITECTURE_MODULAR_ANALYSIS.md` → Over-engineered
- `INTERFACE_EVOLUTION.md` → Speculative
- `README2.md` → Duplicate

## Quick Summary

**Phase 1** (Weeks 1-8): Core graph browser - force-directed UI, Servo integration, search, persistence  
**Phase 2** (Weeks 9-12): Performance & multiprocess - spatial optimization, crash isolation  
**Phase 3** (Weeks 13-16): Browser features - bookmarks, downloads, document types  
**Phase 4** (Weeks 17-24): Polish & extensions - WASM compilation, public release  

See [implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md) for full details.

---

## Tech Stack

| Component | Technology | Why |
|-----------|-----------|-----|
| Language | Rust | Performance, safety |
| Browser engine | Servo | Modern, multiprocess, WebRender |
| UI framework | egui | Immediate mode, fast (in servoshell) |
| Multiprocess | ipc-channel | Servo's IPC (built-in) |
| Sandboxing | gaol | Servo's sandbox lib |
| Graph storage | SlotMap | Stable handles, O(1) |
| Physics | Custom | O(n) with spatial hash |

---

## Project Status (Merged from STATUS.md)

**Last Updated:** February 2026

### Current Phase: Documentation Consolidation ✅

#### 1. ✅ Foundation Decision
- **Decision:** Build on servoshell (not Graphshell codebase)
- **Rationale:** Current with Servo, multiprocess built-in, saves 30-60 hours
- **Documentation:** [SERVERSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md](SERVERSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md)

#### 2. ✅ Implementation Roadmap Created
- **File:** [implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md)
- **Scope:** 24-week concrete plan with weekly milestones

#### 3. ✅ Documentation Consolidated
- **Active docs:** Consolidated into the files listed above
- **Archived:** Superseded docs moved to [archive_docs/](archive_docs/)

---

## Next Steps (Week 1)

From [implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md) Milestone 1.1:

### Day 1: Servoshell Foundation
- [ ] Fork/copy servoshell into `graphshell-graph/`
- [ ] Verify builds: `cargo build --release`
- [ ] Run: `./target/release/graphshell-graph https://example.com`
- [ ] Study key files:
	- `desktop/app.rs` - ApplicationHandler pattern
	- `window.rs` - ServoShellWindow
	- `running_app_state.rs` - WebViewCollection
	- `desktop/gui.rs` - egui integration

### Day 2: Initial Graph View
- [ ] Create `graph/mod.rs` with Node/Graph structs
- [ ] Render 5 hardcoded nodes with egui

### Day 3: Camera Pan
- [ ] Add camera pan (mouse drag)
- [ ] Remove servoshell's default tab bar in graph view

### Day 4-5: Physics
- [ ] Implement basic force-directed layout
- [ ] Get 60fps animation working

**Deliverable:** By end of Week 1, animated graph of 5 nodes

---

## Source: INDEX.md

# Graphshell Design Documentation Complete ✅

**Status**: All architectural decisions documented and implementation roadmap ready.

---

## 🎯 Essential Reading Order

### Phase 0: Foundation (Read These First)
1. **[README.md](README.md)** (5 min)
   - Project vision
   - Why servershell (not Graphshell)
   - What Servo provides
   
2. **[ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)** (30 min)
   - 23 decision sections with detailed rationale
   - Core decisions: View toggle, edge rendering, webview management, physics, data structures
   - Advanced topics: Testing, accessibility, security, process isolation
   - **Read sections 1-5 before Week 1 starts**

3. **[implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md)** (20 min)
   - 24-week implementation plan
   - 8 milestones, success criteria, risk mitigation
   - Technology stack, keybind list, testing strategy
   - **Reference this constantly during implementation**

### Phase 1: Week 1 Start
4. **[implementation_strategy/WEEK1_CHECKLIST.md](implementation_strategy/WEEK1_CHECKLIST.md)** (overview, then use daily)
   - Day-by-day tasks for architecture study
   - Success gates and outputs
   - **Start Monday with this**

---

## 📚 Document Map

### Core Implementation Docs
| Document | Purpose | Size | Read When |
|----------|---------|------|-----------|
| [implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md) | Week-by-week plan with milestones | 30+ pages | Before coding each milestone |
| [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md) | Rationale for all major choices | 20+ pages | When uncertain about "why" |
| [implementation_strategy/WEEK1_CHECKLIST.md](implementation_strategy/WEEK1_CHECKLIST.md) | Day-by-day tasks for architecture study | 2 pages | Week 1, daily |

### Decision Reference
| Document | Purpose | Size | Read When |
|----------|---------|------|-----------|
| [SERVERSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md](SERVERSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md) | Why servershell (not Graphshell codebase) | 5 pages | Understanding foundation decision |
| [GRAPHSHELL_SERVO_PARITY_ANALYSIS.md](GRAPHSHELL_SERVO_PARITY_ANALYSIS.md) | Why Graphshell isn't viable (30-60 hr update cost) | 5 pages | Historical context only |
| [BUILD.md](BUILD.md) | Build setup for all platforms (Windows, macOS, Linux) | 8 pages | Before first build |

### Optional / Phase 3+
| Document | Purpose | Size |
|----------|---------|------|
| [verse_docs/VERSE.md](verse_docs/VERSE.md) | Tokenization and Phase 3+ research | 10 pages |
| [verse_docs/GRAPHSHELL_P2P_COLLABORATION.md](verse_docs/GRAPHSHELL_P2P_COLLABORATION.md) | P2P collaboration and decentralized sync | 30 pages |
| [verse_docs/SEARCH_FINDINGS_SUMMARY.md](verse_docs/SEARCH_FINDINGS_SUMMARY.md) | Verse-related research scan | 20 pages |
| [archive_docs/](archive_docs/) | Superseded design docs | — |

---

## 🚀 What's Decided

### ✅ Core Architecture
- **View Model**: Full-screen toggle with resizable 60/40 split
- **Edge Rendering**: Bezier curves Week 1, upgrade to bundled edges Week 9 if needed
- **Webview Management**: Origin-grouped + small reuse pool (2-4) + Active/Warm/Cold lifecycle
- **Physics**: Grid-based repulsion, worker thread, scale-normalized auto-pause
- **Data Structures**: SlotMap for nodes, adjacency list for O(1) neighbor lookup
- **Rendering**: egui + WebRender dual-pipeline, composited in servoshell shell layer

### ✅ Implementation Strategy
- **Technology**: Servo + egui + tokio + serde + slotmap + petgraph
- **Multiprocess**: Servo's `-M` flag (origin-grouped EventLoops)
- **Sandboxing**: Servo's `-S` flag (gaol on macOS/Linux)
- **IPC**: ipc-channel for cross-process communication

### ✅ Testing & Quality
- **Performance Targets**: 200@60fps, 500@45fps, 1000@30+ (usable)
- **Testing Approach**: Property-based physics tests, visual regression, performance benchmarks
- **Accessibility**: Keyboard-first from Week 1, colorblind-safe colors + line styles
- **Keybinds**: 36 default actions, fully overrideable, smart conflict resolution

### ✅ Risk Mitigation
- **Week 9 Validation Gate**: Is spatial UX viable? If no, pivot to cluster strip + graph optional
- **Untrusted Data**: Sandboxed webviews in Servo processes
- **Process Isolation**: Graph UI in shell layer, webviews isolated
- **Persistence**: Snapshot + append-only log + crash recovery

---

## 📋 Implementation Milestones

### Phase 1A: Core Graph Browser (Weeks 1-4)
- **M1.1**: Servo integration & node creation
- **M1.2**: Headless graph view + thumbnails
- **M1.3**: Grid-based physics + worker thread
- **M1.4**: Node-browser integration + keybinds

### Phase 1B: Usability + Persistence (Weeks 5-8)
- **M1.5**: Camera & navigation
- **M1.6**: Advanced interactions + accessibility baseline
- **M1.7**: Persistence (snapshot + log)
- **M1.8**: Search (fuzzy match)

### Phase 2: Performance & Multiprocess (Weeks 9-12)
- **M2.1**: Performance optimization
- **M2.2**: Bundled edge rendering (if needed)
- **M2.3**: Crash isolation & restart
- **M2.4**: Memory management
- **M2.5**: Import/Export v0

### Phase 3: Browser Features (Weeks 13-16)
- **M3.1**: Bookmarks
- **M3.2**: Downloads
- **M3.3**: Storage manager
- **M3.4**: Document type support
- **M3.5**: Extension API v0

### Phase 4: Polish & Hardening (Weeks 17-24)
- **M4.1**: Visual polish
- **M4.2**: Accessibility improvements
- **M4.3**: Testing & validation

---

## 🎓 Key Insights

### Why Servoshell
- Current with Servo main (no update burden)
- Already has egui 0.33.3 integration
- Multiprocess built-in (saves architecture work)
- Servo's `-M -S` flags handle complex concerns

### Why Origin-Based Webviews
- Servo already manages origin-grouped processes
- No serialization latency (processes created/destroyed, not pooled)
- Scales naturally (1000 origins = 1000 processes, not a UX problem)
- Firefox's approach validated this pattern

### Why No Webview Pool
- Fixed pools are wasteful (dormant webviews consume memory)
- Serialization adds latency (DOM state round-trips)
- Servo's origin grouping is proven at scale (Firefox uses it)

### Why Edge Rendering Progression
- Bezier curves: Simple (O(n)), scales to ~500 edges
- Bundled edges: Complex (FDEB algorithm), scales to 5000+ edges
- Week 9 validation gate: Only upgrade if visual clutter detected
- Graceful progression: Data structure changes, no architectural rework

### Why Physics Auto-Pause
- Physics simulation is expensive (~100μs per node)
- Auto-pause at 0.001 px/frame saves CPU when graph stabilizes
- Threshold tuned so user sees responsive interaction (velocity clamps immediately when dragged)

---

## ✨ What This Means

✅ **No architectural decisions pending**
- Every major choice is documented with rationale
- Alternatives considered for each decision
- Implementation details specified

✅ **Implementation roadmap is concrete**
- 8 detailed milestones over 24 weeks
- Success criteria for each phase
- Risk mitigation strategies

✅ **Week 1 is ready**
- Architecture study checklist
- Day-by-day tasks with outputs
- Success gates defined

✅ **You're ready to code**
- All design decisions made
- Technology stack chosen
- Implementation sequence clear

---

## 🔗 Quick Links

- **Start Coding**: [implementation_strategy/WEEK1_CHECKLIST.md](implementation_strategy/WEEK1_CHECKLIST.md)
- **Reference Architecture**: [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)
- **Implementation Plan**: [implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md)
- **Foundation Decision**: [SERVERSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md](SERVERSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md)
- **Build Setup**: [BUILD.md](BUILD.md)

---

**Last Updated**: February 2026
**Status**: Architecture validation complete, ready for implementation
**Next Step**: Monday — Start Week 1 architecture study with implementation_strategy/WEEK1_CHECKLIST.md

---

## Consolidated Reading Path (Merged from COMPLETE_DOCUMENTATION_MAP.md)

### 🔴 URGENT: Read Before Week 1 Starts (This Week)

1. **[CRITICAL_ANALYSIS.md](CRITICAL_ANALYSIS.md)** (30 min, sections 1-6)
   - 10 critical issues detailed
   - Recommendations for each
   - Why P2P architecture is missing now
   - Why webview lifecycle is untested

### ⭐ ESSENTIAL: Read Week 1 (Architecture Study)

1. **[ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)** (90 min)
   - Sections 1-12: Core decisions (physics, data structures, UI)
   - Sections 13-23: Advanced (testing, security, P2P)

2. **[GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)** (60 min)
   - How Graphshell operates as web browser

3. **[verse_docs/GRAPHSHELL_P2P_COLLABORATION.md](verse_docs/GRAPHSHELL_P2P_COLLABORATION.md)** (60 min)
   - P2P sync architecture (reference for Phase 3)

4. **[FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md](FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md)** (120 min)
   - Firefox & Servo multiprocess
   - Servo-Graphshell integration

### 🎯 REFERENCE: During Implementation

1. **[implementation_strategy/IMPLEMENTATION_ROADMAP.md](implementation_strategy/IMPLEMENTATION_ROADMAP.md)** (constant reference)
2. **[implementation_strategy/WEEK1_CHECKLIST.md](implementation_strategy/WEEK1_CHECKLIST.md)** (daily during Week 1)
3. **[CRITICAL_ANALYSIS.md](CRITICAL_ANALYSIS.md)** sections 7-10 (Phase 2 planning)

### 📚 OPTIONAL: Deep Reference

1. **[verse_docs/GRAPHSHELL_P2P_COLLABORATION.md](verse_docs/GRAPHSHELL_P2P_COLLABORATION.md)** (Phase 3 planning)
2. **[FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md](FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md)** Part 6 (if issues arise)
3. **[verse_docs/VERSE.md](verse_docs/VERSE.md)** (Phase 3+)

---

## 📊 Document Overview (Consolidated)

### Critical Analysis

| Document | Purpose | Size | Audience | Read When |
|----------|---------|------|----------|-----------|
| [CRITICAL_ANALYSIS.md](CRITICAL_ANALYSIS.md) | Deep dive: 10 issues + solutions | 31 KB | Technical lead | This week |
| [GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md) | Browser behavior spec | 18 KB | Week 1 team | Week 1 |
| [verse_docs/GRAPHSHELL_P2P_COLLABORATION.md](verse_docs/GRAPHSHELL_P2P_COLLABORATION.md) | P2P collaboration spec (Verse) | 30 KB | Phase 3 team | Phase 3 |
| [FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md](FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md) | Firefox lessons + Servo integration | 20 KB | Week 1-2 | Week 1 |

### Utilities & Reference

| Document | Purpose | Read When |
|----------|---------|-----------|
| [README.md](README.md) | Top-level overview + status | First pass |
| [INDEX.md](INDEX.md) | Reading map | Always |
| [AGENTS.md](AGENTS.md) | AI assistance guide | As needed |

---

## 🗂️ File Structure (Consolidated)

```
design_docs/
├── README.md
├── INDEX.md
├── CRITICAL_ANALYSIS.md
├── ARCHITECTURE_DECISIONS.md
├── implementation_strategy/
│  ├── IMPLEMENTATION_ROADMAP.md
│  ├── WEEK1_CHECKLIST.md
├── GRAPHSHELL_AS_BROWSER.md
├── FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md
├── SERVERSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md
├── GRAPHSHELL_SERVO_PARITY_ANALYSIS.md
├── BUILD.md
├── AGENTS.md
├── verse_docs/
│  ├── VERSE.md
│  ├── GRAPHSHELL_P2P_COLLABORATION.md
│  ├── SEARCH_FINDINGS_SUMMARY.md
│  └── archive/VERSE_REFERENCE_ANALYSIS.md
└── archive_docs/
```

---

## ✅ Completion Status (Consolidated)

### MVP Phase (Weeks 1-8)
- ✅ Architecture decisions made & documented
- ✅ Implementation roadmap detailed
- ✅ Technology stack chosen (Servo, egui, tokio)
- ⚠️ Design-phase items specified (command trait, version vectors, process monitoring)

### Phase 2 (Weeks 9-12)
- ✅ High-level features planned (performance, browser, exports)
- ⚠️ Feature drift identified (sessions, ghost nodes, sidebar)

### Phase 3+ (Weeks 13-16+)
- ✅ P2P architecture outlined (command log, version vectors, merge strategies)
- ⚠️ Security models explored (full trust, zero trust)

---

## Source: IMPLEMENTATION_ROADMAP.md

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
graphshell/
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
- Webview pooling (not one-per-node)

---

## Phase 1: Core Graph Browser (Weeks 1-8)

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
- [ ] Run physics at 60fps in Graph view
- [ ] Add UI controls:
  - Toggle physics on/off (T key)
  - Adjust damping/strength sliders

- [ ] Week 6 evaluation gate: if 1000 nodes < 30fps, consider `kiddo` or Barnes-Hut

**Deliverable:** Animated graph with working physics

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
- [ ] Load/save keybinds: `~/.config/graphshell-graph/keybinds.toml`
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
- [ ] Add multi-select patterns (via keybinds):
  - Shift+click: Multi-select
  - Drag: Marquee select (rubber band)
  - Click empty space: Deselect all
- [ ] Add node dragging:
  - Click+drag selected node: Move it (disable physics temporarily)
  - Release: Physics resumes
  - Pin mode: Right-click → "Pin" (disable physics permanently)
- [ ] Add context menu (right-click):
  - Navigate in new tab
  - Delete node
  - Pin/Unpin
  - Create edge to...
  - Inspect (show URL, title, metadata)
  - Copy URL
- [ ] Add edge type colors (user-selectable):
  - Load from config: `~/.config/graphshell-graph/preferences.toml`
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
- [ ] Implement JSON schema:
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
  - `Ctrl+S`: Save graph
  - `Ctrl+O`: Open graph
  - `Ctrl+N`: New graph
  - Auto-save every 30 seconds
- [ ] Store in `~/.config/graphshell-graph/graphs/`
- [ ] Add "Recent graphs" menu

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
  - Type text → Filter visible nodes by title/URL/tags
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

**Deliverable:** Fast navigation, feels like a power tool

---

## Phase 2: Performance & Multiprocess (Weeks 9-12)

### Milestone 2.1: Spatial Optimization (Week 9)
**Goal:** Handle 1000+ nodes at 60fps

**Tasks:**
- [ ] Replace O(n²) physics with spatial hash:
  ```rust
  impl PhysicsEngine {
      fn build_spatial_hash(&mut self, graph: &Graph) {
          // Divide space into 100x100 cells
          // Only check forces within nearby cells
      }
  }
  ```
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
**Goal:** Minimize memory with smart pooling

**Tasks:**
- [ ] Tune webview pool:
  - Default: 5 webviews
  - Setting: User can adjust (3-10)
  - Strategy: Keep focused + neighbors loaded
- [ ] Implement lazy loading:
  - Nodes start with "title + favicon only"
  - Webview created only when:
    - Node is focused (Detail view)
    - Node is neighbor of focused (preload)
    - User explicitly clicks "Load"
- [ ] Add webview lifecycle UI:
  - Node badge shows state:
    - Gray: Not loaded
    - Yellow: Loading
    - Green: Loaded
    - Red: Crashed
  - Right-click → "Load/Unload/Reload"

**Deliverable:** 10,000 nodes, 5MB memory (vs 5GB with one-per-node)

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
- [ ] Persist to `~/.config/graphshell-graph/bookmarks.json`
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
- [ ] Persist history: `~/.config/graphshell-graph/downloads.json`

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

## Phase 4: Polish & Extensions (Weeks 17-24)

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

### Milestone 3.5: Extension API v0 (Week 16)
- [ ] Define extension manifest + permissions model
- [ ] Provide graph query API (read-only by default)
- [ ] Allow extensions to add nodes/edges with explicit permission
- [ ] Register omnibar commands (`/ext ...`)
- [ ] Sandbox untrusted extensions via WASM (wasmtime)

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
| Prev tab | Ctrl+Shift+Tab | Detail view |

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
| **Physics FPS** | 60fps with 1,000 nodes | O(n) spatial hash |
| **Render FPS** | 60fps with 10,000 nodes | LOD + culling |
| **Startup time** | < 2 seconds | Lazy initialization |
| **Memory** | < 100MB for 1,000 nodes | Webview pooling |
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
| **Spatial index** | QuadTree | O(log n) queries, viewport culling |
| **Physics** | Custom | O(n) with spatial hash |
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

cd ~/graphshell-graph
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
  - [ ] Remove servoshell's default tab bar

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
- Handles 10,000+ nodes smoothly
- Multiprocess for stability
- Bookmarks/downloads integrated
- Can replace regular browser for research workflows
- Shared publicly, has users

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| **Physics too slow** | Spatial hash: O(n²) worst → O(n) average, cell size tuned, done by Week 9 |
| **UI feels laggy** | egui proven in servoshell, LOD + display list diffing, 60fps achievable |
| **Servo integration hard** | Servoshell proves it works; Servo's origin grouping eliminates webview pooling complexity |
| **Memory blowup** | Origin-based processes (not fixed pool); Servo kills unused processes; graceful degradation on pressure |
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

---

## Source: WEEK1_CHECKLIST.md

# Week 1: Architecture Study Checklist

**Goal**: Complete architecture study phase. By Friday, you'll have:
- SlotMap node structure defined
- Adjacency list data structures sketched
- Physics engine prototype (convergence validated)
- Servo multiprocess integration plan

## Day 1 (Mon): Read & Absorb

**Morning (2 hours)**
- [ ] Read ARCHITECTURE_DECISIONS.md sections 1-5 (View, Edge, Webview, Physics, Data Structures)
- [ ] Review IMPLEMENTATION_ROADMAP.md Milestone 1.1 details
- [ ] Skim Servo `constellation/pipeline.rs` (pipeline abstraction)
- [ ] Skim Servo `constellation/event_loop.rs` (origin grouping, multiprocess)

**Afternoon (2 hours)**
- [ ] Review servoshell `src/` structure
- [ ] Check egui 0.33.3 docs (window management, resizing)
- [ ] Identify how servoshell integrates with Servo
- [ ] **Output**: 1-page "Servo Integration Model" diagram (how servoshell talks to Servo)

**Success Criteria**: You can explain origin-grouped processes and why lifecycle + reuse pool are needed.

---

## Day 2 (Tue): Design Data Structures

**Morning (2 hours)**
- [ ] Design SlotMap node structure:
  ```rust
  struct Node {
    id: NodeKey,
    url: String,
    title: String,
    position: Vec2,     // Physics
    velocity: Vec2,     // Physics
    is_selected: bool,
    created_at: DateTime,
    // ... other fields
  }
  ```
- [ ] List all node fields needed for Phase 1 (no Phase 3+ extras)
- [ ] Verify SlotMap is right choice (stable handles across deletions)

**Afternoon (2 hours)**
- [ ] Design Edge structure:
  ```rust
  struct Edge {
    from: NodeKey,
    to: NodeKey,
    edge_type: EdgeType,  // Hyperlink, Bookmark, History, Manual
    color: Color,
    style: LineStyle,     // Solid, Dotted, Bold, Marker
    metadata: EdgeMetadata,
  }
  ```
- [ ] Design adjacency list:
  - `in_edges: Vec<EdgeKey>` (O(1) predecessors)
  - `out_edges: Vec<EdgeKey>` (O(1) successors)
- [ ] **Output**: `WEEK1_DATA_STRUCTURES.md` (node/edge/adjacency list definitions)

**Success Criteria**: Data structures fit in one screen, no loops, O(1) neighbor lookup proven.

---

## Day 3 (Wed): Physics Prototype

**Morning (2 hours)**
- [ ] Implement spatial hash:
  ```rust
  cell_size = viewport_diagonal / 4
  struct SpatialHash {
    cells: HashMap<(i32, i32), Vec<NodeKey>>,
  }
  ```
- [ ] Implement force calculation:
  - Repulsion between all nodes (O(n) brute force Week 1)
  - Attraction along edges
  - Damping: velocity *= 0.92 per frame

**Afternoon (2 hours)**
- [ ] Run convergence test:
  - 100 random nodes
  - Measure time to stabilize (velocity < 0.001 px/frame)
  - Target: ~60 frames at 60 fps = 1 second
- [ ] Test auto-pause:
  - Pause at 5 seconds of low velocity
  - Verify CPU drops to 0% when paused
  - **Output**: `WEEK1_PHYSICS_TEST.md` (convergence data, CPU measurements)

**Success Criteria**: Physics converges in < 2 seconds for 100 nodes. Auto-pause works.

---

## Day 4 (Thu): Servo Integration

**Morning (2 hours)**
- [ ] Sketch servoshell integration plan:
  - How egui window talks to Servo process
  - How to spawn Servo with `-M -S` flags (multiprocess + sandbox)
  - How to capture navigation events (user clicks a link)
- [ ] Read `servoshell/src/window.rs` or equivalent
- [ ] Check how WebViewCollection currently works

**Afternoon (2 hours)**
- [ ] Design graph node → webview binding:
  - Node creation: User types URL → spawn Servo process for origin
  - Navigation: User clicks link in webview → create new graph node
  - Process lifecycle: Close all nodes from origin → kill process
- [ ] **Output**: `WEEK1_SERVO_INTEGRATION.md` (spawning, event flow, lifecycle)

**Success Criteria**: Clear plan for how graph nodes will get webviews (origin-grouped, dynamic).

---

## Day 5 (Fri): Review & Validate

**Morning (2 hours)**
- [ ] Review all outputs:
  - WEEK1_DATA_STRUCTURES.md
  - WEEK1_PHYSICS_TEST.md
  - WEEK1_SERVO_INTEGRATION.md
- [ ] Cross-check against ARCHITECTURE_DECISIONS.md (sections 1-5)
- [ ] Identify any conflicts or gaps

**Afternoon (2 hours)**
- [ ] Plan Monday's code (Week 2):
  - Create Rust project structure
  - Stub out data structures (SlotMap, Edge, Adjacency list)
  - Stub out physics loop
  - Connect egui window to Servo
- [ ] **Output**: `WEEK1_SUMMARY.md` (architecture validated, ready to code)

**Success Criteria**: You have a clear plan for Week 2-3 implementation. No surprises.

---

## Outputs This Week

Save all outputs to `design_docs/week1/`:
- [ ] WEEK1_DATA_STRUCTURES.md (node/edge definitions)
- [ ] WEEK1_PHYSICS_TEST.md (convergence data, CPU measurements)
- [ ] WEEK1_SERVO_INTEGRATION.md (spawning, event flow, lifecycle)
- [ ] WEEK1_SUMMARY.md (architecture validated, Week 2 plan)

## Success Gate

**By Friday EOD**, you should be able to answer:
- [ ] How many fields does a Node need? (10-15 expected)
- [ ] Why SlotMap and not Vec? (Stable handles across deletions)
- [ ] How do you find all neighbors of a node in O(1)? (Adjacency list in-struct)
- [ ] What's the physics convergence time for 100 nodes? (Should be < 2 sec)
- [ ] How do you spawn a Servo process for a new origin? (Servo's EventLoop API)
- [ ] Why no webview pool? (Origin-based, Servo already manages it)

If you can answer all 6, you're ready for Week 2. 🎯

---

## Source: archive_docs/README.md (Archived docs index)

# Archived Design Documents

These documents are superseded by [../IMPLEMENTATION_ROADMAP.md](../IMPLEMENTATION_ROADMAP.md) (Feb 2026).

They represent earlier explorations and analysis that informed the final concrete plan. Kept for historical reference.

## Archived Feb 2026

### Strategic Analysis (Superseded)
- **GRAPH_INTERFACE.md** — Canonical spec for graph UI (now in roadmap milestones)
- **GRAPH_BROWSER_MIGRATION.md** — Migration roadmap (superseded by IMPLEMENTATION_ROADMAP)
- **PROJECT_PHILOSOPHY.md** — Vision document (vision now clear in roadmap)
- **COMPREHENSIVE_SYNTHESIS.md** — Abstract synthesis (too theoretical)
- **ARCHITECTURE_MODULAR_ANALYSIS.md** — Over-engineered trait architecture
- **INTERFACE_EVOLUTION.md** — Speculative design evolution
- **verse_docs/archive/VERSE_REFERENCE_ANALYSIS.md** — Analysis of VERSE tokenization paper

### Build & Setup (Superseded)
- **BUILD_SETUP_SUMMARY.md** — Build setup notes (now in WINDOWS_BUILD.md)
- **SERVO_MIGRATION_SUMMARY.md** — Servo migration notes (using servoshell now)
- **SETUP_CHECKLIST.md** — Setup checklist (now in roadmap Week 1)
- **SKETCH_NOTES_INTEGRATION.md** — Integration sketches

### Reference (Superseded)
- **DOCS_INDEX.md** — Old documentation index (now in README.md)
- **QUICK_REFERENCE.md** — Quick reference (now in IMPLEMENTATION_ROADMAP)
- **README2.md** — Duplicate README

## What's Still Active?

See [../README.md](../README.md) for active documentation:
- **IMPLEMENTATION_ROADMAP.md** — Concrete 24-week plan (start here)
- **SERVOSHELL_VS_GRAPHSHELL_STRATEGIC_ANALYSIS.md** — Foundation decision
- **GRAPHSHELL_SERVO_PARITY_ANALYSIS.md** — Historical reference (Graphshell update estimate)
- **WINDOWS_BUILD.md** — Platform build setup
- **AGENTS.md** — AI assistance guidance
- **verse_docs/VERSE.md** — Phase 3+ research (not MVP)
- **README.md** — Current progress tracking

---

## Source: archive_docs/README2.md (Archived duplicate README)

# Graphshell

Graphshell is an experimental browser built on [Servo](https://servo.org/), using Rust and WebRender for rendering.

## Current State

Graphshell is currently a **servoshell-based prototype** with support for:
- Multiple webviews and windows via Winit event loop
- WebRender-based rendering with Servo integration
- Download management
- Configuration system
- Clipboard support (via arboard)
- Keyboard and touch input handling

**Planned**: Graph-first interface with cluster strip in the detail view (see [design_docs/](design_docs/) for research and specifications).

## Quick Start

### Windows 11 Setup

**See [WINDOWS_BUILD.md](WINDOWS_BUILD.md) for detailed step-by-step instructions.**

TL;DR:
1. Install Visual Studio 2022 Build Tools with C++ workload
2. Install Python 3.8+, Perl, Git (add all to PATH)
3. Install Rust from https://rustup.rs/
4. Install MozillaBuild from https://wiki.mozilla.org/MozillaBuild
5. Open MozillaBuild Terminal: `C:\mozilla-build\start-shell.bat`
6. Build:
   ```bash
   cd /c/path/to/graphshell
   cargo build --release
   ./target/release/graphshell.exe
   ```

### Linux / macOS Setup

```bash
git clone https://github.com/markik/graphshell
cd graphshell
cargo build --release
./target/release/graphshell
```

### Alternative: Nix Shell (Linux/macOS)

If you have Nix installed:
```bash
nix-shell
cargo build --release
./target/release/graphshell
```

## Requirements

- **Rust** (see [rust-toolchain.toml](rust-toolchain.toml)) — Latest stable recommended
- **Platform tooling** for Servo builds:
  - **Windows 11**: MozillaBuild (see [Windows 11 Setup](#windows-11-setup) above)
  - **Linux**: `build-essential`, Python 3.8+, Perl
  - **macOS**: Xcode Command Line Tools
- **Python 3.8+** in PATH
- **Perl** in PATH (for Servo build scripts)

## Architecture

### Core Components

- **[src/main.rs](src/main.rs)**: Winit-based event loop with ApplicationHandler
- **[src/graphshell.rs](src/graphshell.rs)**: Main Graphshell struct integrating Servo constellation, compositor, and webview pool
- **[src/compositor.rs](src/compositor.rs)**: Rendering coordination with WebRender and display lists
- **[src/window.rs](src/window.rs)**: Window management and event handling
- **[src/tab.rs](src/tab.rs)**: Tab data structures
- **[src/webview/](src/webview/)**: WebView embedding and context menu handling
  - [context_menu.rs](src/webview/context_menu.rs): Right-click menu
  - [webview.rs](src/webview/webview.rs): WebView lifecycle management
  - [prompt.rs](src/webview/prompt.rs): Alert/prompt dialogs
- **[src/download.rs](src/download.rs)**: Download manager
- **[src/storage.rs](src/storage.rs)**: Persistence layer
- **[src/config.rs](src/config.rs)**: Configuration management
- **[src/keyboard.rs](src/keyboard.rs)**: Keyboard input handling
- **[src/touch.rs](src/touch.rs)**: Touch input handling
- **[src/rendering.rs](src/rendering.rs)**: Rendering utilities
- **[src/errors.rs](src/errors.rs)**: Error types

### Crates

- **graphshell** (library): Builder pattern and public API ([graphshell/src/main.rs](graphshell/src/main.rs) demonstrates usage)
- **graphshellview_messages**: IPC message types for webview communication
- **graphshellview_build**: Build support utilities

### Dependencies

Key external dependencies:
- **Servo**: constellation (tab/pipeline management), compositor, script, layout, canvas, webrender (from `servo/main` branch)
- **Winit**: Window creation and event loop
- **crossbeam**: Channel-based concurrency
- **ipc-channel**: Inter-process communication
- **arboard**: Clipboard access
- **serde**: Serialization/deserialization

## Troubleshooting

### Windows: "MozTools or MozillaBuild not found"
- Ensure you're running in **MozillaBuild Terminal** (`start-shell.bat`), not regular Command Prompt
- Verify MozillaBuild installation in `C:\mozilla-build`
- Check that Perl and Python are in your PATH within the MozillaBuild environment

### Windows: "Python not found"
- Install Python 3.8+ from https://www.python.org
- Add Python to system PATH
- Verify in MozillaBuild Terminal: `python --version`

### Compilation hangs
- Servo builds are large and may take 10–15 minutes on first build
- Subsequent builds are faster (incremental compilation)
- Monitor system resources; Servo uses significant RAM (8GB+ recommended)

## Design Documents

Research, specifications, and future roadmap:
- [design_docs/GRAPH_INTERFACE.md](design_docs/GRAPH_INTERFACE.md) — Interaction model for planned graph canvas
- [design_docs/GRAPH_BROWSER_MIGRATION.md](design_docs/GRAPH_BROWSER_MIGRATION.md) — Migration plan to graph-first UI
- [design_docs/PROJECT_PHILOSOPHY.md](design_docs/PROJECT_PHILOSOPHY.md) — Vision and design principles
- [design_docs/verse_docs/VERSE.md](design_docs/verse_docs/VERSE.md) — Phase 3+ tokenization and P2P research
- [design_docs/](design_docs/) — Full archive of research and specifications

## Contributing

See [.github/CONTRIBUTING.md](.github/CONTRIBUTING.md) and [.github/CODE_OF_CONDUCT.md](.github/CODE_OF_CONDUCT.md).

## License

Dual-licensed: MIT or Apache-2.0

## References

- [Servo Browser Engine](https://servo.org/)
- [WebRender](https://github.com/servo/webrender)
- [Winit](https://github.com/rust-windowing/winit)
- [Servo on Servo Wiki](https://github.com/servo/servo/wiki)

