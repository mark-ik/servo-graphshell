# Persistence & Camera Zoom Implementation

**Date:** 2026-02-11
**Status:** Complete

## Feature 1: Graph Persistence

### Architecture

```
[Runtime Graph] --log_mutation()--> [fjall append log] (every mutation)
[Runtime Graph] --take_snapshot()--> [rkyv bytes → redb KV] (every 5 min)
[Startup] --load redb snapshot--> --replay fjall log--> [Recovered Graph]
```

### Crates

- **fjall 3** — Append-only LSM log for mutation journaling (pure Rust, ACID)
- **redb 3** — Embedded KV store for periodic snapshots (pure Rust, ACID)
- **rkyv 0.8** — Zero-copy serialization for fast snapshot read/write

### Key Design Decision: URL-based Identity

SlotMap keys (`NodeKey`, `EdgeKey`) contain internal version+index pairs that change across sessions. Persisted data uses URL strings as stable node identity. Edges identified by `(from_url, to_url, edge_type)` tuple.

### Persisted Data (Graph Structure Only)

**Nodes:** url, title, position (x, y), is_pinned
**Edges:** from_url, to_url, edge_type (Hyperlink | History)

Not persisted (resets on load): velocity, is_selected, lifecycle, in_edges/out_edges (rebuilt from edges), physics config, view state.

### Storage Location
`dirs::config_dir()/graphshell/graphs/` — platform-agnostic:

- Windows: `%APPDATA%\graphshell\graphs\`
- Linux: `~/.config/graphshell/graphs/`
- macOS: `~/Library/Application Support/graphshell/graphs/`

### Crash Recovery Flow
1. Check for `snapshots.redb` — if missing, start with empty graph
2. Load latest snapshot (copy to `AlignedVec` first — see implementation notes)
3. Rebuild Graph from snapshot (add_node, add_edge for each entry)
4. Open fjall log, replay entries after snapshot
5. Return recovered graph (periodic snapshot compacts log on next interval)

### Files

- `persistence/mod.rs` — `GraphStore`: fjall log + redb snapshots, recovery
- `persistence/types.rs` — rkyv-serializable types: `GraphSnapshot`, `LogEntry`, `PersistedNode`, `PersistedEdge`
- `graph/mod.rs` — `Graph::to_snapshot()`, `Graph::from_snapshot()`
- `app.rs` — `persistence: Option<GraphStore>`, mutation logging, periodic snapshots
- `desktop/gui.rs` — Log edges/titles, periodic snapshot check, skip initial node if recovered

### Mutation Logging Integration

| Call site | LogEntry variant |
|-----------|-----------------|
| `app.add_node_and_sync()` | `AddNode { url, position_x, position_y }` |
| `gui.sync_webviews_to_graph()` after `add_edge()` | `AddEdge { from_url, to_url, edge_type }` |
| `gui.sync_webviews_to_graph()` on title change | `UpdateNodeTitle { url, title }` |

### Implementation Notes

**rkyv alignment with redb:** Bytes returned by redb are NOT aligned to rkyv's required alignment (8+). `rkyv::access()` and `rkyv::from_bytes()` both fail with "unaligned pointer" errors. Fix: copy to `rkyv::util::AlignedVec::<16>::new()` via `extend_from_slice()` before calling `rkyv::from_bytes()`.

**fjall v3 API:** `Database::builder(&path).open()`, `db.keyspace("name", || opts)` (closure, not value), `Iter` yields `Guard` directly with `guard.into_inner() -> Result<(key, value)>`. Must keep `Database` alive as a struct field since `Keyspace` borrows from it.

**rkyv 0.8 archived types:** Archived `f32` is `f32_le` (little-endian), requires `.into()` for conversion. Add `#[rkyv(derive(Debug, PartialEq))]` to get traits on archived enum variants.

### Future Extensibility

- `LogEntry` enum grows new variants without breaking existing logs
- redb supports multiple tables — later add session state, browser state
- Eventually integrate with Servo's own storage layer for per-node browser data

### Tests (19 tests)

**persistence/types.rs (7):** rkyv roundtrip for each type variant
**graph/mod.rs (3):** snapshot roundtrip, empty graph, edge type preservation
**persistence/mod.rs (6):** empty startup, log+recover, snapshot+recover, snapshot+log recovery, duplicate URL idempotency, title update
**app.rs (5):** camera defaults, clamp within range, clamp below min, clamp above max, clamp at boundaries

---

## Feature 2: Camera Zoom Clamping

### Problem
egui_graphs has no built-in zoom min/max bounds. `MetadataFrame.zoom` multiplies unclamped on each mouse wheel event.

### Solution: Post-Frame Clamp
After `GraphView` renders each frame, read `MetadataFrame` from egui's persisted data store, clamp `zoom` to [0.1, 10.0], write back if changed. Works for all zoom sources (mouse wheel, fit-to-screen, etc.).

### Camera Struct (`app.rs`)

```rust
pub struct Camera {
    pub zoom_min: f32,   // 0.1
    pub zoom_max: f32,   // 10.0
    pub current_zoom: f32, // tracked via Event::Zoom + MetadataFrame read
}
```

### Integration Points

- `render/mod.rs::clamp_zoom()` — post-frame read/clamp/write of `MetadataFrame` via `ctx.data_mut()`
- `render/mod.rs::process_events()` — `Event::Zoom` handler updates `camera.current_zoom`
- `render/mod.rs::draw_graph_info()` — displays `Zoom: {:.1}x` in overlay
- `egui_graphs::MetadataFrame` stored at `Id::new("egui_graphs_metadata_")`
