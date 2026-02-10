# GraphShell Codebase Analysis & Roadmap

## Current State Summary

GraphShell is a **spatial browser prototype** (~3,958 LOC) built on Servo that visualizes browsing sessions as a force-directed node graph instead of traditional tabs. It uses **egui** for immediate-mode UI, **SlotMap** for stable graph handles, and a custom physics engine running on a background thread via crossbeam channels.

### What Works Today

| Area | Status |
|------|--------|
| Graph core (nodes, edges, SlotMap storage) | Implemented |
| Force-directed physics (spatial hash, springs, damping) | Implemented |
| Physics worker thread (non-blocking simulation) | Implemented |
| egui rendering (nodes, edges, labels, overlay) | Implemented |
| Mouse input (select, multi-select, drag, pan) | Implemented |
| Keyboard controls (toggle physics, toggle view) | Implemented |
| Graph/Detail view switching | Implemented |
| Webview-to-Node bidirectional mapping | Implemented |
| Node lifecycle (Active/Warm/Cold) | Implemented |
| CLI argument parsing & preferences | Implemented |
| Keybind configuration (TOML) | Implemented |
| Cross-platform (Win/Linux/macOS/Android/OpenHarmony) | Implemented |
| Demo graph (5 static nodes + edges) | Implemented |
| URL-to-NodeKey fast lookup | Implemented |

### What's Missing / Stubbed

| Area | Status | Priority |
|------|--------|----------|
| Graph persistence (save/load) | Stubbed (empty functions) | **Critical** |
| Camera zoom | Stub only | High |
| Center camera on graph | TODO | High |
| Thumbnail capture for nodes | Not started | High |
| Search (history, bookmarks) | Not started | Medium |
| Graph algorithms (clusters, centrality) | Not started | Medium |
| Export (DOT, JSON, etc.) | Not started | Medium |
| Undo/redo for graph edits | Not started | Medium |
| Smooth camera animations | Not started | Medium |
| Advanced spatial indexing | Basic HashMap grid only | Medium |
| P2P co-op browsing (Verse) | Conceptual only | Long-term |
| 3D graph visualization | Not started | Long-term |
| Minimap | Not started | Low |
| Lasso zoning | Not started | Low |
| Mods/plugins system | Not started | Low |

### Test Coverage Gaps

Only 14 test functions exist, covering URL parsing and spatial grid basics.
**Untested:** physics simulation, node interaction, view toggling, webview-node mapping,
rendering, keybinds, physics worker thread communication, graph mutations.

---

## Recommended Roadmap

### Phase 1: Core Stability (Foundation)

**1. Graph Persistence**
The `save_snapshot()` and `load_graph()` stubs in `graph/persistence.rs` are the most critical gap.

| Crate | Role | Why |
|-------|------|-----|
| **fjall** | Append-only operation log | LSM-tree based, pure Rust, ACID, designed for write-heavy workloads. Matches the planned append-only log pattern. |
| **redb** | Snapshot storage | Pure Rust embedded KV store, ACID transactions, savepoints for snapshot/rollback. Outperforms sled. |
| **rkyv** | Zero-copy serialization | Fastest Rust serialization. Read graph data directly from memory-mapped files without deserialization. |

*Avoid sled (stuck in beta since 2021). Avoid RocksDB (massive C++ dependency).*

Architecture:
```
[Runtime Graph] --serialize--> [fjall append log] (every mutation)
                --snapshot-->  [redb/rkyv snapshot] (every N minutes)
[Startup] --load snapshot--> --replay log--> [Recovered Graph]
```

**2. Camera & Navigation**
- Implement `zoom()` in `input/camera.rs`
- Implement "center camera on graph" (TODO in `input/mod.rs:26`)
- Add smooth transitions with **ezing** or **simple_easing** crates (pure easing functions), or use egui's built-in `ctx.animate_value_with_time()`
- Note: `keyframe` crate is abandoned (no activity since 2022) — avoid it

**3. Tests**
Add tests for: physics force calculations, graph mutations (add/remove node/edge),
webview-node mapping consistency, physics worker command/response cycle.

---

### Phase 2: Usability Features

**4. Thumbnail Capture**
Use Servo's built-in `webrender::screen_capture` module + `glow::Context::read_pixels()`
after Servo renders a page, then downscale with `image::imageops::resize()`.
Both `glow` and `image` are already in the dependency tree — no new crates needed.

**5. Search**

| Crate | Role | Why |
|-------|------|-----|
| **tantivy** | Persistent full-text search index | Rust equivalent of Lucene, BM25 ranking, fuzzy queries, 2x faster than Lucene. Index pages on load. |
| **nucleo** | Real-time fuzzy matching in UI | By helix-editor team, fzf-like scoring, 6x faster than fuzzy-matcher. For the search bar. |

**6. Undo/Redo**

| Crate | Role | Why |
|-------|------|-----|
| **undo** | Command-pattern undo/redo | Supports both linear (Record) and tree-based (History) undo. Implement `Edit` trait per graph operation. |

Pattern:
```rust
enum GraphEdit {
    AddNode { id: NodeKey, url: String, position: Vec2 },
    RemoveNode { id: NodeKey, /* stashed data */ },
    AddEdge { from: NodeKey, to: NodeKey },
    MoveNode { id: NodeKey, old_pos: Vec2, new_pos: Vec2 },
}
```

**7. Improved Spatial Indexing**

| Crate | Role | Why |
|-------|------|-----|
| **kiddo** | KD-tree for physics neighbor queries | Cache-friendly packed layout, supports per-frame rebuild, rkyv-compatible. |
| **rstar** | R-tree for viewport/click queries | Range queries over bounding boxes, good for "which nodes are visible?" and hit-testing. |

Replace the current `HashMap<(i32,i32), Vec<NodeKey>>` spatial grid with kiddo for physics
and rstar for viewport culling.

---

### Phase 3: Intelligence & Export

**8. Graph Algorithms**

| Crate | Role | Why |
|-------|------|-----|
| **petgraph** | Core graph algorithms | De facto standard (9M+ downloads). Dijkstra, BFS, DFS, connected components, topo sort, DOT export. |
| **graphrs** | Community detection, centrality | Louvain/Leiden clustering, betweenness/closeness/eigenvector centrality. For finding page clusters and hub nodes. |

Use petgraph as a "projection" layer: periodically convert your SlotMap graph to a
petgraph `StableGraph` for analysis, then apply results back (cluster colors, layout hints).

**9. Export**

| Format | Approach |
|--------|----------|
| DOT (Graphviz) | `petgraph::dot::Dot` (built-in) or **graphviz-rust** for richer output |
| JSON | `serde_json` (already a dependency) with petgraph's `serde-1` feature |
| JSON-LD | **json-ld** crate for semantic web / knowledge graph interop |
| GEXF | **quick-xml** (write GEXF XML manually, format is simple) |
| CSV | **csv** crate (BurntSushi, well-maintained) |
| SVG | **graphviz-rust** can invoke Graphviz to produce SVG |

**10. Graph Layout Alternatives**

| Crate | Role | Why |
|-------|------|-----|
| **forceatlas2** | Better force-directed layout | Barnes-Hut O(N log N) repulsion via quadtree. Designed for network viz, handles hub-and-spoke well. |
| **layout-rs** | Hierarchical/layered layouts | Parses DOT, produces Sugiyama-style layouts. Good alternative view mode. |

---

### Phase 4: Decentralized / P2P (Long-term — "Verse")

**11. P2P Co-op Browsing**

| Crate | Role | Why |
|-------|------|-----|
| **iroh** | Core P2P transport | QUIC-based, dial by public key, automatic NAT traversal + hole-punching. Higher success rate than libp2p. |
| **iroh-gossip** | Broadcast graph changes | Pub/sub overlay for real-time co-op graph updates. |
| **iroh-docs** | Shared graph state (CRDTs) | Eventually-consistent KV store for shared browsing sessions. |
| **iroh-blobs** | Share thumbnails/snapshots | BLAKE3 content-addressed blob transfer. |

*Use libp2p only if you need Kademlia DHT or GossipSub at scale.*

---

## 29 TODOs/FIXMEs in the Codebase

### Critical
- `graph/persistence.rs:30` — `save_snapshot()` → empty
- `graph/persistence.rs:36` — `load_graph()` → empty

### Input/Camera
- `input/mod.rs:26` — Center camera on graph
- `input/camera.rs` — zoom() incomplete

### Desktop Platform
- `desktop/headed_window.rs:498` — Cursor restore after dialog
- `desktop/headed_window.rs:649` — Tab key handling
- `desktop/headed_window.rs:797` — Screen space calculation (subtract system UI)
- `desktop/headed_window.rs:850` — Dialog animation workaround
- `desktop/headed_window.rs:1063,1072` — Toolbar height for positioning
- `desktop/headless_window.rs:149` — `unimplemented!()` GlWindow

### Desktop GUI
- `desktop/gui.rs:370` — Fullscreen phishing mitigation
- `desktop/gui.rs:782` — AccessKit tree forwarding
- `desktop/dialog.rs:488,555,614` — Dialog alignment & backdrop

### Mobile/EGL
- `egl/app.rs:262,401,619` — Multi-window, stop-load, VSync
- `egl/ohos/mod.rs:463,557` — Window/webview creation, multi-window

### Other
- `desktop/protocols/resource.rs:102` — Referrer checking
- `lib.rs:179` — HiTrace extension cost
- `prefs.rs:148` — macOS config_dir
- `panic_hook.rs:44` — Servo options dependency

---

## Crate Summary Table

| Category | Primary Pick | Secondary | Notes |
|----------|-------------|-----------|-------|
| Persistence | **fjall** + **rkyv** | **redb** | Append log + zero-copy snapshots |
| Graph Algorithms | **petgraph** | **graphrs** | Core algos + community detection |
| P2P Networking | **iroh** | iroh-gossip/docs/blobs | Full P2P stack |
| Thumbnails | Servo `screen_capture` + `image` | — | Already in deps |
| Spatial Indexing | **kiddo** | **rstar** | KD-tree + R-tree |
| Graph Layout | **forceatlas2** | **layout-rs** | Barnes-Hut + hierarchical |
| Animation | **ezing** / **simple_easing** | egui built-ins | Pure easing functions |
| Search | **tantivy** | **nucleo** | Full-text + fuzzy |
| Export | **petgraph** DOT/serde | **graphviz-rust**, **csv** | Multiple formats |
| Undo/Redo | **undo** | — | Command pattern |
