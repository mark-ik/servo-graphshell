# Graph Persistence Plan (2026-02-11)

## Graph Persistence Plan
- Goal: Ensure crash-safe, fast recovery of graphs across sessions.
- Scope: fjall log, redb snapshots, rkyv serialization.
- Dependencies: Existing persistence modules, node schema stability.
- Phase 1: Audit and verify
  - Review snapshot cadence and log replay ordering.
  - Confirm URL-based identity remains stable across reload.
- Phase 2: Test hardening
  - Add tests for log replay after partial write.
  - Add tests for schema evolution with backward compatibility.
- Phase 3: Performance
  - Benchmark save and load on 1k, 10k node graphs.
  - Tune snapshot interval and log compaction strategy.
- Phase 4: Diagnostics
  - Add counters for last snapshot time and log length.
  - Provide warnings if log growth exceeds threshold.
- Validation tests
  - Simulated crash during write restores to last durable state.
  - Load time under target for 10k nodes.
- Outputs
  - Expanded test suite and benchmark notes.
  - Optional config for snapshot cadence.

## Findings
- 

## Progress
- 2026-02-11: Plan created.
