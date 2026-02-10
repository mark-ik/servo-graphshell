# Graphshell Codebase Changelog

**Repository**: servo (https://github.com/servo/servo)  
**Branch**: graphshell  
**Build Status**: Buildable (tested Feb 9, 2026)  
**Crate**: graphshell (ports/graphshell/)  

---

## Commit History

### Recent Commits (Most Recent First)

| Commit | Date | Author | Message |
|--------|------|--------|---------|
| `159e9e50a8a` | 2026-02-09 | (upstream) | Replace abandoned keyframe crate with ezing/simple_easing |
| `8f444cfa5ee` | 2026-02-09 | (upstream) | Add codebase analysis and roadmap for graphshell |
| `4eb1e7d619e` | 2026-02-08 23:54:00 -0500 | Mark-ik | Replaced branch README with project description; plan to make README with build guide |
| `50191d28b44` | 2026-02-08 23:45:14 -0500 | Mark-ik | Added project description and updated dependencies |
| `ccb1f70d4da` | 2026-02-08 22:05:03 -0500 | Mark-ik | Implement webview lifecycle management for graphshell |
| `855471aa057` | 2026-02-08 09:12:37 -0500 | Mark-ik | Trying to fix webview visibility on graph |
| `af9013d4046` | (date) | (author) | Merge branch 'servo:main' into graphshell |
| `a749991ac6b` | (date) | (author) | Merge branch 'graphshell' of https://github.com/mark-ik/servo into graphshell |
| `8e42e07949f` | (date) | (author) | Update README with graphshell branch information |
| `ccb1f70d4da` | (date) | (author) | Implement webview lifecycle management for graphshell |
| `ff79d737e21` | (date) | (author) | Initial verso-graph browser port |

**Total Commits Touching ports/graphshell**: 6  
**Branch Started**: verso-graph initial port (commit ff79d737e21)  
**Primary Developer**: Mark-ik

---

## Codebase Structure

### Root Level Files

```
ports/graphshell/
├── main.rs                 # Entry point (delegates to graphshell::main())
├── lib.rs                  # Library root
├── app.rs                  # Application state & view management (332 lines)
├── build.rs                # Build script
├── Cargo.toml              # Package manifest
├── parser.rs               # URL/argument parsing
├── crash_handler.rs        # Crash handlers & debugging
├── panic_hook.rs           # Panic hook setup
├── running_app_state.rs    # Runtime state tracking
├── prefs.rs                # Preferences system
├── resources.rs            # Resource loading
├── test.rs                 # Test utilities
└── webdriver.rs            # WebDriver protocol support
```

### Core Modules

#### 1. **graph/** - Graph data structures
```
graph/
├── mod.rs                  # Graph container, Node, Edge definitions (271 lines)
├── persistence.rs          # Graph save/load (JSON serialization)
└── spatial.rs              # Spatial queries & spatial hash grid
```

**Key Types**:
- `Graph`: Main graph container using SlotMap for node storage
- `Node`: Webpage node with position, velocity, metadata
- `Edge`: Connection between nodes (hyperlink, bookmark, history, manual)
- `NodeKey`, `EdgeKey`: Stable identifiers (survive deletions)

**Node Properties**:
- `url`: Full URL of webpage
- `title`: Page title (fallback to URL)
- `position`: Point2D in graph space
- `velocity`: Vector2D for physics simulation
- `is_selected`: Selection state
- `is_pinned`: Lock node position (prevents physics movement)
- `created_at`: Timestamp
- `last_visited`: Activity tracking

#### 2. **physics/** - Force-directed layout engine
```
physics/
├── mod.rs                  # Physics engine core (209 lines)
├── spatial_hash.rs         # Spatial hash grid for O(n) repulsion
└── worker.rs               # Threaded physics simulation worker
```

**Key Components**:
- **PhysicsEngine**: Local physics queries
- **PhysicsWorker**: Runs physics on separate thread
- **PhysicsConfig**: Tunable simulation parameters
  - `repulsion_strength`: 5000.0 (default)
  - `spring_strength`: 0.1
  - `damping`: 0.92
  - `spring_rest_length`: 100.0 pixels
  - `velocity_threshold`: 0.001 px/frame (auto-pause)
  - `pause_delay`: 5.0 seconds

**Algorithm**:
- Spatial hash grid for O(n) average-case repulsion forces
- Hooke's law springs on edges
- Velocity damping for convergence
- Auto-pause on convergence

#### 3. **render/** - Graphics rendering
```
render/
└── mod.rs                  # Egui-based graph visualization
```

Status: Placeholder/minimal implementation

#### 4. **input/** - Input handling
Handles user interaction (keyboard, mouse, touch)

#### 5. **desktop/** - Desktop platform support
Platform-specific windowing & integration

#### 6. **platform/** - Platform abstractions
Cross-platform code for Windows/macOS/Linux

#### 7. **egl/** - EGL rendering backend
OpenGL/EGL integration (used by servoshell)

#### 8. **config/** - Configuration
Application configuration system

---

## Application Architecture

### View Model (`app.rs`)

```rust
pub enum View {
    Graph,            // Force-directed graph view
    Detail(NodeKey),  // Detail/tab view focused on a specific node
}

pub struct SplitViewConfig {
    enabled: bool,         // Split view toggle
    detail_ratio: f32,     // Split ratio (0.0-1.0, default 0.6)
}

pub struct GraphBrowserApp {
    graph: Graph,                                    // Graph structure
    physics: PhysicsEngine,                          // Local physics queries
    physics_worker: Option<PhysicsWorker>,           // Worker thread
    view: View,                                      // Current view
    split_config: SplitViewConfig,                   // UI layout
    selected_nodes: Vec<NodeKey>,                    // Selection state
    webview_to_node: HashMap<WebViewId, NodeKey>,   // Tab ↔ Node mapping
    node_to_webview: HashMap<NodeKey, WebViewId>,
    is_interacting: bool,                           // Drag/pan state
}
```

**Key Features**:
- Bidirectional mapping between Servo webviews (tabs) and graph nodes
- Split-view support (graph + detail simultaneously)
- Multi-select node support
- Physics simulation on separate thread
- Interactive state tracking (for input handling)

### Demo Graph (`app.rs::init_demo_graph()`)
- Initializes 5 static nodes for testing
- Demonstrates graph structure before loading real content

---

## Dependencies

### Key Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| `libservo` | Browser engine | ✅ Integrated |
| `euclid` | 2D geometry (Point2D, Vector2D) | ✅ Active |
| `slotmap` | Efficient node/edge storage | ✅ Active |
| `egui` | Immediate-mode GUI for graph rendering | ✅ Active |
| `serde` | Serialization (persistence) | ✅ Active |
| `log` | Logging framework | ✅ Active |
| `winit` | Cross-platform windowing | ✅ Active |

### Features

Default features (from Cargo.toml):
```toml
default = [
    "gamepad",                      # Gamepad support
    "libservo/clipboard",           # Clipboard integration
    "js_jit",                       # JavaScript JIT compilation
    "max_log_level",                # Release-mode logging
    "webgpu",                       # WebGPU support
    "webxr"                         # WebXR support
]
```

Optional features:
- `crown`: Extended features
- `debugmozjs`: SpiderMonkey debugging
- `jitspew`: JIT compiler logging
- `js_backtrace`: JavaScript stack traces
- `media-gstreamer`: GStreamer media backend

---

## Current Implementation Status

### ✅ Implemented

1. **Graph Data Structures**
   - Node creation, deletion, properties
   - Edge creation between nodes
   - SlotMap-based storage (efficient, handles deletion)

2. **Physics Engine**
   - Force-directed layout (repulsion + springs)
   - Spatial hashing for performance
   - Multithreaded physics worker
   - Configurable parameters
   - Auto-pause on convergence

3. **View Management**
   - Graph view (force-directed visualization)
   - Detail/tab view toggle
   - Split-view support (graph + detail)
   - Node selection (single & multi-select)
   - Pinning (lock node position)

4. **Webview Integration**
   - Bidirectional mapping (tabs ↔ nodes)
   - Webview lifecycle management
   - Multi-webview support

5. **Serialization**
   - Graph persistence (save/load)
   - JSON export format

6. **Input Handling**
   - Basic input pipeline setup

### 🚧 In Progress / Partial

1. **Egui Rendering**
   - Placeholder render/mod.rs
   - Full graph visualization GUI needed

2. **Visibility & UI**
   - Active work on webview visibility in graph view
   - Recent commit (855471aa057): "trying to fix webview visibility on graph"

3. **Platform Integration**
   - Desktop platform support (Windows primary)
   - WIP: macOS/Linux (see design roadmap)

### ❌ Not Yet Started

1. **User Interaction**
   - Graph panning/zooming
   - Node dragging
   - Edge creation in UI
   - Search/filtering

2. **Advanced Features**
   - Clipping (DOM element extraction)
   - Custom rules/topologies
   - 2D/3D canvas switching
   - Mods/plugins system

3. **Persistence**
   - Session save/restore
   - Graph export formats
   - Bookmarks/history import

4. **P2P/Verse**
   - All P2P features deferred to Phase 3+

---

## Build Information

### Package Metadata

```toml
[package]
name = "graphshell"
version = "0.0.1"
edition = "2021"

[lib]
name = "graphshell"
path = "lib.rs"

[[bin]]
name = "graphshell"
path = "main.rs"
```

**Windows Metadata**:
- FileDescription: "GraphShell Browser"
- ProductName: "GraphShell Browser"
- LegalCopyright: "© The Verso Project Developers"
- OriginalFilename: "graphshell.exe"

### Build Requirements

- Rust 1.91.0+
- Windows 11 (primary development platform)
- MozillaBuild (for Windows)
- Python 3.8+

### Build Commands

```bash
# Full debug build
./mach build graphshell

# Release build
./mach build -r graphshell
./mach build --release graphshell

# Run
./target/release/graphshell https://example.com

# Check compilation
cargo check -p graphshell
```

---

## Known Issues & Notes

### Webview Visibility
**Status**: WIP (commit 855471aa057, 2026-02-08)
- Webview rendering in graph view needs work
- Recent commits addressing visibility issues

### Physics Parameters
Default tuning may need adjustment for different screen sizes:
- `repulsion_strength: 5000.0` — Strong repulsion
- `spring_rest_length: 100.0` — Ideal distance between connected nodes
- `damping: 0.92` — High damping for quick convergence
- Consider viewport-relative scaling (see viewport_diagonal)

### Code Quality
- Well-structured modules with clear separation
- Good use of Rust type safety (NodeKey, EdgeKey)
- Physics worker pattern prevents UI blocking
- Documentation comments on major structures

---

## Next Steps (Per Design Roadmap)

**Phase 1 (Weeks 1-8)**: Core graph UI, Servo integration
- 🚧 Graph structure: DONE
- 🚧 Physics engine: DONE
- 🚧 View management: DONE
- 🚧 Webview integration: Partially done (visibility issues)
- ❌ Egui rendering: Needs completion
- ❌ Input handling: Needs completion

**Phase 2 (Weeks 9-12)**: Performance, multiprocess
- Full physics optimization
- Spatial hash improvements
- Crash isolation testing

**Phase 3 (Weeks 13-16)**: Browser features
- Bookmarks
- Downloads
- Protocol handlers

**Phase 4 (Weeks 17-24)**: Polish, extensions
- Extension system
- Public release

---

## Repository Metadata

- **Default Branch**: main (Servo upstream)
- **Active Branch**: graphshell
- **Merges from main**: Regular (6 merges visible)
- **Last Merge**: 2f54aa61988 (Feb 2026)
- **Upstream Sync**: Current with Servo main (~Feb 9, 2026)

---

## Contributors

- **Mark-ik**: Primary developer (project owner)
- **Servo Team**: Upstream maintenance & merges

---

**Last Updated**: February 9, 2026  
**Build Status**: ✅ Builds successfully  
**Test Status**: 🚧 In development (Phase 1)
