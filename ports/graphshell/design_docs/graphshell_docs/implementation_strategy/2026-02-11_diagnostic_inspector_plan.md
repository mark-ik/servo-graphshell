# Diagnostic Inspector Plan (2026-02-11)

## Diagnostic Inspector Plan
- Goal: Visualize Servo internals as a live graph for debugging.
- Scope: Tracing, data ingestion, and a dedicated UI mode.
- Dependencies: tracing spans, event collection, graph rendering.
- Phase 1: Instrumentation
  - Add tracing spans at thread and IPC boundaries.
  - Define event schema for message counts and latency.
- Phase 2: Data pipeline
  - Collect events and aggregate into a diagnostic graph.
  - Define node/edge types for threads and channels.
- Phase 3: UI mode
  - Add toggle to switch to diagnostic view.
  - Render weights and highlights for bottlenecks.
- Phase 4: Export
  - Support export to SVG or JSON for reports.
- Validation tests
  - Events appear during typical browsing session.
  - Diagnostic view does not impact main FPS.
- Outputs
  - Diagnostic mode implementation and docs.

## Findings
- 

## Progress
- 2026-02-11: Plan created.
