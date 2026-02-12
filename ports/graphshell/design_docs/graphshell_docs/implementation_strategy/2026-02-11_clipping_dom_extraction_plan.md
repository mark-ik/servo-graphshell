# Clipping DOM Extraction Plan (2026-02-11)

## Clipping DOM Extraction Plan
- Goal: Clip a DOM element into its own node with snapshot and metadata.
- Scope: Servo DOM access, context menu, node creation.
- Dependencies: Servo inspector APIs, element selection, screenshot capture.
- Phase 1: DOM selection
  - Enable right-click element selection in webview.
  - Capture DOM path or selector for rehydration.
- Phase 2: Snapshot
  - Capture element bounding box and render to image.
  - Store HTML snippet or text summary.
- Phase 3: Node creation
  - Create node with clip metadata and link to source.
  - Add edge from source node to clip node.
- Phase 4: Refresh logic
  - Optional refresh when source page changes.
- Validation tests
  - Clip text and images reliably.
  - Clips persist across restart.
- Outputs
  - Clip metadata schema and renderer updates.

## Findings
- 

## Progress
- 2026-02-11: Plan created.
