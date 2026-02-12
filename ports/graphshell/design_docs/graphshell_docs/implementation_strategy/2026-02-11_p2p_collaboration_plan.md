# P2P Collaboration Plan (2026-02-11)

## P2P Collaboration Plan
- Goal: Optional sync of graphs across peers, local-first by default.
- Scope: Sync backend, op log, merge strategy, and minimal UI.
- Dependencies: command log format, peer discovery, Verse specs.
- Phase 1: Backend abstraction
  - Define `SyncBackend` trait and local filesystem impl.
  - Reuse existing op log for sync payloads.
- Phase 2: Protocol
  - Implement pull/push exchange with version vectors.
  - Add conflict detection for concurrent edits.
- Phase 3: Merge rules
  - Implement deterministic merge for metadata and positions.
  - Provide conflict UI for topology changes.
- Phase 4: UX and settings
  - Add sync status panel and peer list.
  - Provide opt-in toggles and privacy guidance.
- Validation tests
  - Two peers converge after disconnected edits.
  - Conflicts are surfaced and resolvable.
- Outputs
  - Sync backend module and protocol docs.

## Findings
- 

## Progress
- 2026-02-11: Plan created.
