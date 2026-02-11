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

# Run
./mach run -p graphshell
```

For detailed setup, see **[BUILD.md](BUILD.md)**. For implementation status, see **[ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md)**.

---

## Project Description

Graphshell is a spatial browser built on Servo's engine. Webpages are nodes in a force-directed graph, links are visible edges, and navigation happens by traversing the graph spatially. Users build mental maps of web structure through physical layout.

**Vision**: Enable serendipitous discovery, preserve browsing context, reduce tab chaos.

**Current Status**: Core browsing graph functional (~4,500 LOC). Servo integration, persistence, zoom/camera, physics all working. Thumbnails remaining for Phase 1 completion.

---

## Document Index

### Start Here (Required Reading)

1. **[ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md)** — What's implemented, architecture decisions, key crates
2. **[IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)** — Feature-driven plan (not calendar-based)
3. **[GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)** — How graph UI works as browser (behavior spec)

### Core Documentation

- **[BUILD.md](BUILD.md)** — Windows build setup (Rust 1.91+, Python, MozillaBuild)
- **[QUICKSTART.md](QUICKSTART.md)** — Quick reference for building
- **[technical_architecture/SERVO_INTEGRATION_BRIEF.md](technical_architecture/SERVO_INTEGRATION_BRIEF.md)** — Servo webview integration architecture
- **[INDEX.md](INDEX.md)** — Complete documentation map

### Archived Planning Docs

> **Note**: Calendar-based plans are archived. Current plan lives in IMPLEMENTATION_ROADMAP.md.

- **[archive_docs/](../archive_docs/)** — Archived planning and checkpoint materials

### Cross-Project

- **[verse_docs/README.md](../verse_docs/README.md)** — Verse documentation (P2P/tokenization research, Phase 3+)

---

## Implementation Status

**Last Updated**: February 11, 2026
**Codebase**: `ports/graphshell/` (~4,500 LOC in core modules)

### What Works

- **Real browsing**: Servo webviews integrated, navigation creates nodes + edges
- **Graph visualization**: egui_graphs GraphView with zoom/pan/drag/selection
- **Physics**: Force-directed layout with kiddo KD-tree, background worker thread
- **Persistence**: fjall log + redb snapshots + rkyv serialization, survives restarts
- **Keyboard shortcuts**: T (physics), C (fit), P (config panel), N (new node), Home/Esc (view toggle), Del (remove)

### What's Next

- **Thumbnails & favicons**: Nodes are colored circles — page previews would aid spatial memory
- **Search/filtering**: nucleo fuzzy search (Phase 2)
- **Split view**: egui_tiles for graph + detail simultaneously (Phase 2)

**See [ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md) for full details.**

---

## Tech Stack

| Component | Technology | Notes |
| --------- | ---------- | ----- |
| Language | Rust 1.91+ | Performance, safety, Servo compatibility |
| Browser Engine | Servo (libservo) | Webview lifecycle, navigation tracking |
| UI Framework | egui 0.31 | Immediate mode, integrated with Servo |
| Graph Storage | petgraph 0.8 (StableGraph) | Stable indices, algorithm ecosystem |
| Graph Visualization | egui_graphs 0.29 | GraphView widget, events, zoom/pan |
| Physics | Custom + kiddo 4.2 KD-tree | O(n) with spatial queries, springs + repulsion |
| Worker Thread | crossbeam_channel | Non-blocking physics, 60fps target |
| Persistence | fjall 3 + redb 2 + rkyv 0.8 | Append log + snapshots + zero-copy serialization |
| Geometry | euclid | 2D math (Point2D, Vector2D) |

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

### Graphshell Repositories

- [graphshell on GitHub](https://github.com/servo/servo) — Main repository (Servo)

### External Resources

- [Servo Documentation](https://book.servo.org/)
- [egui Documentation](https://docs.rs/egui/)
- [WebRender](https://github.com/servo/webrender) — GPU renderer
