# Archived Documents

This directory contains superseded, historical, and exploratory documents that are not used in active development. They are kept for reference but are **not canonical** and may contain outdated information.

## Purpose

Per [DOC_POLICY.md](../DOC_POLICY.md), `archive_docs/` houses notes and information no longer actively relevant to Graphshell or Verse development. When a feature or research direction is deprecated, document it here. If later adopted again, move relevant documentation back to the appropriate active docs folder.

## Archived Documents (Feb 2026)

These docs are superseded by [graphshell_docs/IMPLEMENTATION_ROADMAP.md](../graphshell_docs/IMPLEMENTATION_ROADMAP.md):

### Strategic & Architectural Analysis
- **GRAPH_INTERFACE.md** — Canonical spec for graph UI (details now in implementation roadmap)
- **GRAPH_BROWSER_MIGRATION.md** — Migration strategy (superseded by Phase 1-4 roadmap)
- **PROJECT_PHILOSOPHY.md** — Vision document (vision now explicit in roadmap goals)
- **COMPREHENSIVE_SYNTHESIS.md** — Abstract synthesis (too theoretical; details in roadmap)
- **ARCHITECTURE_MODULAR_ANALYSIS.md** — Over-engineered trait architecture proposal
- **INTERFACE_EVOLUTION.md** — Speculative design evolution (replaced by concrete GRAPHSHELL_AS_BROWSER.md)

### Build & Setup Documentation
- **BUILD_SETUP_SUMMARY.md** — Build setup notes (consolidated into WINDOWS_BUILD.md)
- **SERVO_MIGRATION_SUMMARY.md** — Servo migration analysis (replaced by SERVOSHELL_VS_GRAPHSHELL decision)
- **SETUP_CHECKLIST.md** — Old setup checklist (replaced by WEEK1_CHECKLIST.md)
- **SKETCH_NOTES_INTEGRATION.md** — Integration sketches and scratch notes

### Documentation Index & Reference
- **DOCS_INDEX.md** — Old documentation index (replaced by README.md and INDEX.md)
- **QUICK_REFERENCE.md** — Old quick reference (replaced by QUICKSTART.md)
- **README2.md** — Duplicate README (consolidated into main README.md)

### Verse Research
- **VERSE_REFERENCE_ANALYSIS.md** — Analysis of external VERSE tokenization research (historical reference)

## Archival Policy

Per [DOC_POLICY.md](../DOC_POLICY.md):
- Archive docs are kept for historical reference
- They should not be kept up-to-date with active development unless directly relevant
- If a feature in an archived doc is adopted again, move the relevant documentation back to active docs
- Archive docs do not compete for organization, so don't remove them unless actively redundant with active docs

## Contributing

Instead of adding to archived docs:
1. Check if information belongs in [graphshell_docs/README.md](../graphshell_docs/README.md) or [verse_docs/README.md](../verse_docs/README.md)
2. If it's architectural, add to the relevant category subdirectory
3. Only add to archive if documenting deprecated features or historical decisions
