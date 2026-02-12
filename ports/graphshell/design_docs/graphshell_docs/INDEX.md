# Graphshell Design Documentation Index

**Last Updated**: February 12, 2026
**Status**: Core browsing graph functional with egui_tiles runtime complete; physics/selection simplification is next

---

## Essential Reading Order

### Phase 0: Understanding Current Status (Read These First)

1. **[README.md](README.md)** (10 min)
   - Project vision: spatial browser on Servo
   - Quick start: build & run commands
   - Implementation status summary (what works, what's next)

2. **[DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md)** (15 min) **← Start here for development**
   - Quick orientation for new contributors and AI assistants
   - Essential commands, common patterns, debugging
   - Current work status and troubleshooting

3. **[ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md)** — Required (20 min)
   - Foundation code breakdown (~7,000 LOC)
   - Architecture decisions with rationale (petgraph StableGraph, egui_graphs, kiddo KD-tree, fjall+redb+rkyv persistence)
   - Webview lifecycle management and frame execution order
   - Key crates and their roles

4. **[CODEBASE_MAP.md](CODEBASE_MAP.md)** — Reference (30 min)
   - Detailed module breakdown with line counts
   - Test distribution across modules
   - Data flow diagrams and hot paths
   - File size reference for context window planning

5. **[IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)** (30 min)
   - Feature-driven plan (11 feature targets)
   - egui_tiles migration complete
   - Next block: physics/selection simplification, then FT2 thumbnails
   - Validation tests and success criteria for each target

### Phase 1: Detailed Specs

1. **[GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)** (15 min)
   - Browser behavior specification
   - How graph UI works as browser (navigation model, view toggle, lifecycle management)

2. **[BUILD.md](BUILD.md)** (10 min)
   - Windows 11 build setup (Rust 1.91+, Python, MozillaBuild)
   - Platform-specific instructions

---

## Document Map (DOC_POLICY Categories)

### Entry Points

- **[README.md](README.md)** — Project overview and current status
- **[INDEX.md](INDEX.md)** — This map

### 1. Technical Architecture

| Document | Purpose | Read When |
| -------- | ------- | --------- |
| **[DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md)** | Quick orientation, commands, patterns, debugging | First (for contributors/AI) |
| **[CODEBASE_MAP.md](CODEBASE_MAP.md)** | Detailed module map, test distribution, data flow | Reference during development |
| **[ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md)** | Foundation code, architecture decisions | Second (Required) |
| **[technical_architecture/SERVO_INTEGRATION_BRIEF.md](technical_architecture/SERVO_INTEGRATION_BRIEF.md)** | Servo webview integration architecture | Reference for webview work |
| **[BUILD.md](BUILD.md)** | Platform build instructions | Before first build |
| **[QUICKSTART.md](QUICKSTART.md)** | Quick reference for building | When you forget commands |

### 2. Design

| Document | Purpose | Read When |
| -------- | ------- | --------- |
| **[GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)** | Browser behavior spec | Implementing navigation/views |

### 3. Features

- **No dedicated feature briefs yet.** Use [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) until features warrant separate briefs.

### 4. Tests

- **No test specifications yet.** Create under a `tests/` category when validation docs exist.

### 5. Implementation Strategy

| Document | Purpose | Read When |
| -------- | ------- | --------- |
| **[IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)** | Feature targets, validation criteria, tech stack | Before implementing features |
| **[implementation_strategy/2026-02-11_phase1_refinement_plan.md](implementation_strategy/2026-02-11_phase1_refinement_plan.md)** | **Active:** Phase 1 refinement (11 steps) | Current work reference |
| **[implementation_strategy/2026-02-12_persistence_ux_plan.md](implementation_strategy/2026-02-12_persistence_ux_plan.md)** | Persistence reset UX plan | Reference (completed) |
| **[implementation_strategy/2026-02-12_physics_selection_plan.md](implementation_strategy/2026-02-12_physics_selection_plan.md)** | Physics migration + selection consolidation plan | Current work reference |

**Architecture Plans**

- **[implementation_strategy/2026-02-12_architecture_reconciliation.md](implementation_strategy/2026-02-12_architecture_reconciliation.md)** — **Required reading:** Reconciles two architectural proposals, defines egui_tiles approach
- **[implementation_strategy/2026-02-12_servoshell_inheritance_analysis.md](implementation_strategy/2026-02-12_servoshell_inheritance_analysis.md)** — What to keep/replace from servoshell, egui_tiles integration plan
- **[implementation_strategy/2026-02-12_egui_tiles_implementation_guide.md](implementation_strategy/2026-02-12_egui_tiles_implementation_guide.md)** — **Step-by-step guide:** Concrete API calls, code examples, 7-phase migration plan

**Feature Plans (by target)**

- **[implementation_strategy/2026-02-11_thumbnails_favicons_plan.md](implementation_strategy/2026-02-11_thumbnails_favicons_plan.md)** — Feature Target 2
- **[implementation_strategy/2026-02-11_graph_persistence_plan.md](implementation_strategy/2026-02-11_graph_persistence_plan.md)** — Feature Target 3
- **[implementation_strategy/2026-02-11_camera_zoom_plan.md](implementation_strategy/2026-02-11_camera_zoom_plan.md)** — Feature Target 4
- **[implementation_strategy/2026-02-11_center_camera_plan.md](implementation_strategy/2026-02-11_center_camera_plan.md)** — Feature Target 5
- **[implementation_strategy/2026-02-11_search_filtering_plan.md](implementation_strategy/2026-02-11_search_filtering_plan.md)** — Feature Target 6
- **[implementation_strategy/2026-02-11_bookmarks_history_import_plan.md](implementation_strategy/2026-02-11_bookmarks_history_import_plan.md)** — Feature Target 7
- **[implementation_strategy/2026-02-11_performance_optimization_plan.md](implementation_strategy/2026-02-11_performance_optimization_plan.md)** — Feature Target 8
- **[implementation_strategy/2026-02-11_clipping_dom_extraction_plan.md](implementation_strategy/2026-02-11_clipping_dom_extraction_plan.md)** — Feature Target 9
- **[implementation_strategy/2026-02-11_diagnostic_inspector_plan.md](implementation_strategy/2026-02-11_diagnostic_inspector_plan.md)** — Feature Target 10
- **[implementation_strategy/2026-02-11_p2p_collaboration_plan.md](implementation_strategy/2026-02-11_p2p_collaboration_plan.md)** — Feature Target 11

### 6. Phase 3+ Research (P2P/Tokenization)

| Document | Purpose | Phase |
| -------- | ------- | ----- |
| **[verse_docs/VERSE.md](../verse_docs/VERSE.md)** | Tokenization research | Phase 3+ |
| **[verse_docs/GRAPHSHELL_P2P_COLLABORATION.md](../verse_docs/GRAPHSHELL_P2P_COLLABORATION.md)** | P2P collaboration patterns | Phase 3+ |
| **[verse_docs/SEARCH_FINDINGS_SUMMARY.md](../verse_docs/SEARCH_FINDINGS_SUMMARY.md)** | Verse research scan | Phase 3+ |

### 7. Archive

- **[archive_docs/](../archive_docs/)** — Superseded analyses, historical reference, checkpoint snapshots
- **[archive_docs/alternative_approaches/](../archive_docs/alternative_approaches/)** — Alternative architectural proposals not pursued
  - [2026-02-12_unified_architecture_plan_ARCHIVED.md](../archive_docs/alternative_approaches/2026-02-12_unified_architecture_plan_ARCHIVED.md) — Continuous zoom approach (not pursued)

---

## Status Snapshot

- **Core browsing graph functional** (~7,000 LOC total, ~1,300 core graph+physics). Details in [ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md).
- **Phase 1 progress**: egui_tiles migration complete; favicon vertical slice complete; thumbnail pipeline still pending.
- **Refinement**: 11/11 steps complete. See [phase1_refinement_plan.md](implementation_strategy/2026-02-11_phase1_refinement_plan.md).
- **Tests**: 155 passing
- **Tech stack**: petgraph + egui_graphs + kiddo + fjall + redb + rkyv. Details in [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md).

---

## Quick Links

- **Entry Point**: [README.md](README.md)
- **Architecture**: [ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md) — Required reading
- **Implementation Plan**: [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)
- **Browser Behavior**: [GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)
- **Build Setup**: [BUILD.md](BUILD.md)
