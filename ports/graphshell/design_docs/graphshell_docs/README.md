# Graphshell Design Documentation

Graphshell is a spatial browser that represents webpages as nodes in a force-directed graph. Built on Servo, it provides a research tool for sense-making and exploratory workflows.

## Quick Start

**Build & Run**:
```bash
# Clone (if not already)
git clone https://github.com/servo/servo
cd servo
git checkout graphshell  # Fork by mark-ik

# Build (requires Python 3.8+, Rust 1.91+)
./mach build --release -p graphshell

# Run (currently demo with 5 static nodes)
./mach run -p graphshell
```

For detailed setup, see **[BUILD.md](BUILD.md)**. For implementation status, see **[ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md)**.

---

## Project Description

Graphshell is a spatial browser built on Servo's engine. Webpages are nodes in a force-directed graph, links are visible edges, and navigation happens by traversing the graph spatially. Users build mental maps of web structure through physical layout.

**Vision**: Enable serendipitous discovery, preserve browsing context, reduce tab chaos.

**Current Status**: Foundation complete (~3,500 LOC), Servo integration in progress.

---

## Document Index

### 🚀 Start Here (Required Reading)
1. **[ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md)** ⭐ **Read first** — What's implemented, what's stubbed, architecture decisions
2. **[IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)** ⭐ Feature-driven plan (not calendar-based)
3. **[GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)** — How graph UI works as browser (behavior spec)

### Core Documentation
- **[BUILD.md](BUILD.md)** — Windows build setup (Rust 1.91+, Python, MozillaBuild)
- **[QUICKSTART.md](QUICKSTART.md)** — Quick reference for building
- **[DESIGN_OVERVIEW_AND_PLAN.md](DESIGN_OVERVIEW_AND_PLAN.md)** — UI/UX design overview
- **[technical_architecture/SERVO_INTEGRATION_BRIEF.md](technical_architecture/SERVO_INTEGRATION_BRIEF.md)** — Servo webview integration brief (Feature 1)
- **[INDEX.md](INDEX.md)** — Complete documentation map

### Archived Planning Docs
> **Note**: Calendar-based plans are archived. Current plan lives in IMPLEMENTATION_ROADMAP.md.

- **[archive_docs/](../archive_docs/)** — Archived planning and checkpoint materials

### Cross-Project
- **[verse_docs/README.md](../verse_docs/README.md)** — Verse documentation (P2P/tokenization research, Phase 3+)
- **[archive_docs/](../archive_docs/)** — Superseded analyses, historical reference

---

## Implementation Status

**Last Updated**: February 10, 2026  
**Codebase**: `c:\Users\mark_\Code\servo\ports\graphshell\` (~3,500 LOC)  
**Commits**: 6 to graphshell port (initial: ff79d737e21, recent: ccb1f70d4da)

### ✅ Foundation Complete

**Graph Data Structures** (`graph/mod.rs`, 271 lines):
- SlotMap-based storage (stable NodeKey/EdgeKey handles)
- Node properties: url, title, position, velocity, is_selected, is_pinned, lifecycle (Active/Warm/Cold)
- Edge types: Hyperlink, Bookmark, History, Manual with visual styles
- Adjacency lists (in_edges, out_edges)

**Physics Engine** (`physics/mod.rs`, 209 lines):
- Force-directed layout: Hooke's law springs + n-body repulsion
- Spatial hash grid for O(n) neighbor queries (not O(n²))
- Auto-pause on convergence (velocity threshold + delay)
- Configuration: repulsion 5000, spring 0.1, damping 0.92

**Physics Worker** (`physics/worker.rs`, 150 lines):
- Background thread using crossbeam_channel
- Non-blocking: graph cloned only on structure changes
- Commands: UpdateGraph, Step, Toggle, Pause, Resume
- Responses: NodePositions (HashMap) sent to main thread

**egui Rendering** (`render/mod.rs`, 145 lines):
- Nodes as colored circles (size/color by lifecycle)
- Edges as line segments (color/style by EdgeType)
- Selection highlighting (yellow), pinned border (red)
- Labels with font_id for readability

**Mouse & Keyboard Input** (`input/mod.rs`, ~150 lines):
- Click select, Shift multi-select
- Drag nodes (sets is_pinned, pauses physics)
- Pan graph (move all nodes)
- Double-click focus (Detail view)
- Keyboard: T toggle physics, Home/Esc toggle view, C center camera (TODO)

**Camera** (`input/camera.rs`, ~60 lines):
- Structure: position, zoom, target_position, target_zoom
- Smooth interpolation (lerp factor 10.0 * dt)
- zoom() method exists

**Application State** (`app.rs`, 332 lines):
- View enum (Graph/Detail)
- GraphBrowserApp with selection, focus, physics updates
- Bidirectional HashMap<WebViewId, NodeKey> mappings (structure only)
- Demo: init_demo_graph() creates 5 static nodes

### ⚠️ Partial (Exists But Not Integrated)

**Servo Webview Integration**:
- Mapping structures defined (HashMap<WebViewId, NodeKey>)
- Lifecycle structs exist
- **Missing**: Actual webview creation, navigation hooks, thumbnail capture

**Camera Zoom**:
- Camera::zoom() method exists
- **Missing**: egui rendering doesn't apply zoom transform (screen_pos calculation)

### ❌ Stubbed (Empty Functions)

**Graph Persistence** (`graph/persistence.rs`, ~40 lines):
- save_snapshot() and load_graph() are empty
- "TODO: Implement in Week 6" comments
- No serialization, no crash recovery

### 🚫 Not Started

- Thumbnails & favicons (Circle placeholders used)
- Clipping (DOM element extraction)
- Search/filtering
- Bookmarks/history import
- Export formats (SVG, DOT, JSON)
- Multiprocess isolation
- Sandboxing
- Extensions
- P2P collaboration (Verse, Phase 3+)
- Diagnostic/Engine Inspector mode (Phase 3+)

**See [ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md) for details.**

---

## Tech Stack

| Component | Technology | Status | Why |
|-----------|-----------|--------|-----|
| Language | Rust 1.91+ | ✅ | Performance, safety, Servo compatibility |
| Browser Engine | Servo (libservo) | ⚠️ Structures only | Modern, multiprocess, WebRender |
| UI Framework | egui 0.31 | ✅ | Immediate mode, fast iteration, composable |
| Graph Storage | SlotMap 1.0 | ✅ | Stable handles, O(1) lookup, generational keys |
| Physics | Custom | ✅ | O(n) with spatial hash, springs + repulsion |
| Worker Thread | crossbeam_channel | ✅ | Non-blocking physics, 60fps target |
| Geometry | euclid 0.22 | ✅ | 2D math (Point2D, Vector2D, Rect) |

**Planned (Not Yet Integrated)**:
- Persistence: redb (snapshots), fjall (append log), rkyv (serialization)
- Search: nucleo (fuzzy, fzf-like), tantivy (full-text, Phase 2)
- Tracing: tracing crate (for Engine Inspector, Phase 3+)

---

## Current Phase: Servo Integration Sprint

**Goal**: Make nodes represent real webpages, not static demo data.

**Priority** (from IMPLEMENTATION_ROADMAP.md):
1. Feature Target 1: Servo Webview Integration (create/destroy webviews, navigation hooks)
2. Feature Target 2: Thumbnail & Favicon Rendering (actual page previews + favicon fallback)
3. Feature Target 3: Graph Persistence (snapshots + log, crash recovery)
4. Feature Target 4: Camera Zoom Integration (apply transform to rendering)
5. Feature Target 5: Center Camera on Graph (implement C key handler)

**Success Criteria**:
- Can browse real websites (https://example.com)
- Graph persists across sessions (no data loss)
- Thumbnails recognizable (spatial memory aid)
- Zoom/pan feels natural (60fps interaction)

---

## Build Info

**Platform**: Windows 11 (primary development)
**Requirements**:
- Rust 1.91.0+ (via `rust-toolchain.toml`)
- Python 3.8+ (mach wrapper)
- MozillaBuild (C++ toolchain for Servo dependencies)

**Build Times**:
- Clean build: ~15-30 min (depends on machine)
- Incremental: ~30s-2min (typical code change)
- Release build: +20% time vs debug

**Run Commands**:
```bash
# Debug build + run
./mach run -p graphshell

# Release build
./mach build --release -p graphshell
./mach run --release -p graphshell

# Build only (no run)
./mach build -p graphshell
```

See **[BUILD.md](BUILD.md)** for detailed setup.

---

## References

**Graphshell Repositories**
- [graphshell on GitHub](https://github.com/servo/servo) — Main repository (Servo)
- [servo on GitHub](https://github.com/servo/servo) — Servo browser engine

**External Resources**
- [Servo Documentation](https://book.servo.org/)
- [egui Documentation](https://docs.rs/egui/)
- [WebRender](https://github.com/servo/webrender) — GPU renderer
