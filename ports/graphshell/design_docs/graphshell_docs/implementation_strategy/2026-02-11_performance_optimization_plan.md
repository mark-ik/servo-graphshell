# Performance Optimization Plan (2026-02-11)

## Performance Optimization Plan
- Goal: 500 nodes at 45 FPS, 1000 nodes at 30+ FPS.
- Scope: Rendering, physics, and interaction only.
- Dependencies: profiling tools, egui draw pipeline.
- Phase 1: Profile
  - Capture frame timing and hot paths.
  - Establish baseline metrics for 100/500/1000 nodes.
- Phase 2: Render improvements
  - Batch node rendering where possible.
  - Add viewport culling for off-screen nodes.
- Phase 3: Physics tuning
  - Benchmark spatial queries and adjust cell sizes.
  - Add early-exit for near-stable graphs.
- Phase 4: LOD and clustering
  - Optional clustering when zoomed out.
  - Simplify labels at distance.
- Validation tests
  - Meets FPS targets at each node count.
  - No interaction stalls during pan/zoom.
- Outputs
  - Performance report and tuned settings.

## Findings
- 

## Progress
- 2026-02-11: Plan created.
