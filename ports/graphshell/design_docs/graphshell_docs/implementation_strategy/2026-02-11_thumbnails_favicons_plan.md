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
- 

## Progress
- 2026-02-11: Plan created.
