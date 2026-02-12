# Bookmarks And History Import Plan (2026-02-11)

## Bookmarks And History Import Plan
- Goal: Seed graph from browser bookmarks and history data.
- Scope: Import UI and parsers; graph merge logic.
- Dependencies: file picker, JSON/HTML parsing, SQLite access.
- Phase 1: Bookmarks import
  - Parse Firefox/Chrome bookmarks export formats.
  - Create nodes and folder-based edges.
- Phase 2: History import
  - Read browser history SQLite databases.
  - Create nodes from recent visits with referrer edges.
- Phase 3: Dedup and merge
  - Merge nodes by URL with conflict rules.
  - Preserve tags and folder labels as metadata.
- Phase 4: UX flow
  - Provide import wizard and progress feedback.
- Validation tests
  - Import 100+ bookmarks with structure preserved.
  - Dedup avoids duplicate nodes for same URL.
- Outputs
  - Import commands and parsers.
  - Migration guide for users.

## Findings
- 

## Progress
- 2026-02-11: Plan created.
