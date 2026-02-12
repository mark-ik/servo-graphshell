# Thumbnails And Favicons Plan (2026-02-11)

## Thumbnails And Favicons Plan
- Goal: Show recognizable node visuals using thumbnail > favicon > color fallback.
- Scope: Graphshell node rendering and data pipeline only; no external sync.
- Dependencies: Servo capture hook, egui textures, persistence updates.
- Phase 1: Data model
  - Add optional `thumbnail` and `favicon` handles to the node model.
  - Add metadata for source URL and last update timestamp.
- Phase 2: Favicon pipeline
  - Parse `<link rel="icon">` and fallback to `/favicon.ico`.
  - Fetch asynchronously, decode to RGBA, resize to 32x32.
  - Cache per origin and persist to disk.
- Phase 3: Thumbnail pipeline
  - Capture after load idle; downscale to 256x192.
  - Store as texture and persist compressed bytes.
  - Evict or refresh based on node lifecycle.
- Phase 4: Rendering
  - Render thumbnail if present; else favicon; else colored circle.
  - Add subtle border to indicate Active/Warm/Cold state.
- Validation tests
  - Favicon loads in under 500ms for common sites.
  - Thumbnail appears within 2 seconds without UI hitching.
  - Mixed 100-node graph maintains target FPS.
- Outputs
  - Capture and fetch pipeline in app/render modules.
  - Persistence schema update for favicon/thumbnail bytes.
  - Visual regression snapshots for sample URLs.

## Findings
- Existing tab favicon plumbing already existed (`window.take_pending_favicon_loads` + tab texture cache).
- The missing path was graph-node integration and persistence.
- `egui_graphs` supports custom node drawing via `DisplayNode`; this allows favicon rendering in graph view without changing graph interaction handling.
- Snapshot persistence is the right first persistence step for favicon bytes (log entries are not required for this slice).

## Progress
- 2026-02-11: Plan created.
- 2026-02-12: Implemented favicon vertical slice:
  - Added favicon fields to node model (`favicon_rgba`, `favicon_width`, `favicon_height`).
  - Added snapshot persistence support for favicon data (`PersistedNode` extended).
  - Added custom graph node shape that renders favicon textures when available (falls back to colored circle).
  - Wired favicon ingestion to persist bytes on mapped graph nodes from pending favicon loads.
  - Added tests for favicon snapshot/schema roundtrip.
