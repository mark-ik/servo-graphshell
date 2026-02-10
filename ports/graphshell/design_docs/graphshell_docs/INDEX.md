# Graphshell Design Documentation Index

**Last Updated**: February 10, 2026  
**Status**: Foundation complete (~3,500 LOC), Servo integration sprint in progress

---

## 🎯 Essential Reading Order

### Phase 0: Understanding Current Status (Read These First)
1. **[README.md](README.md)** (10 min)
   - Project vision: spatial browser on Servo
   - Quick start: build & run commands
   - Implementation status summary (what works, what's stubbed, what's not started)
   
2. **[ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md)** ⭐ **Required** (20 min)
   - What foundational code exists (~3,500 LOC breakdown)
   - Architecture decisions with rationale (SlotMap, spatial hash, worker thread, egui)
   - Critical gaps: Servo integration partial, persistence stubbed
   - Diagnostic mode concept (Engine Inspector, Phase 3+)
   
3. **[IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)** (30 min)
   - Feature-driven plan (11 feature targets: Servo integration, thumbnails, persistence, zoom, clipping, diagnostics, P2P)
   - Validation tests and success criteria for each target
   - Tech stack: redb/fjall/rkyv for persistence, nucleo for search, tracing for diagnostics
   - **Reference during feature implementation**

### Phase 1: Detailed Specs
4. **[GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)** (15 min)
   - Browser behavior specification
   - How graph UI works as browser (navigation model, view toggle, lifecycle management)
   
5. **[BUILD.md](BUILD.md)** (10 min)
   - Windows 11 build setup (Rust 1.91+, Python, MozillaBuild)
   - Platform-specific instructions

---

## 📚 Document Map (DOC_POLICY Categories)

### Entry Points
- **[README.md](README.md)** — Project overview and current status
- **[INDEX.md](INDEX.md)** — This map

### 1. Technical Architecture
| Document | Purpose | Read When |
|----------|---------|-----------|
| **[ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md)** | Foundation code, architecture decisions, gaps | Second (Required) |
| **[technical_architecture/SERVO_INTEGRATION_BRIEF.md](technical_architecture/SERVO_INTEGRATION_BRIEF.md)** | Servo webview integration architecture, APIs, implementation steps | Before starting Servo integration |
| **[BUILD.md](BUILD.md)** | Platform build instructions | Before first build |
| **[QUICKSTART.md](QUICKSTART.md)** | Quick reference for building | When you forget commands |

### 2. Design
| Document | Purpose | Read When |
|----------|---------|-----------|
| **[GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)** | Browser behavior spec | Implementing navigation/views |
| **[DESIGN_OVERVIEW_AND_PLAN.md](DESIGN_OVERVIEW_AND_PLAN.md)** | UI/UX design overview | Visual design work |

### 3. Features
- **No dedicated feature briefs yet.** Use [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) until features warrant separate briefs.

### 4. Tests
- **No test specifications yet.** Create under a `tests/` category when validation docs exist.

### 5. Implementation Strategy
| Document | Purpose | Read When |
|----------|---------|-----------|
| **[IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)** | Feature targets, validation criteria, tech stack | Before implementing features |

### 6. Phase 3+ Research (P2P/Tokenization)
| Document | Purpose | Phase |
|----------|---------|-------|
| **[verse_docs/VERSE.md](../verse_docs/VERSE.md)** | Tokenization research | Phase 3+ |
| **[verse_docs/GRAPHSHELL_P2P_COLLABORATION.md](../verse_docs/GRAPHSHELL_P2P_COLLABORATION.md)** | P2P collaboration patterns | Phase 3+ |
| **[verse_docs/SEARCH_FINDINGS_SUMMARY.md](../verse_docs/SEARCH_FINDINGS_SUMMARY.md)** | Verse research scan | Phase 3+ |

### 7. Archive
- **[archive_docs/](../archive_docs/)** — Superseded analyses, historical reference, checkpoint snapshots

---

## ✨ Status Snapshot

- **Foundation implemented** (~3,500 LOC). Details in [ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md).
- **Priority features**: Servo integration, thumbnails + favicon fallback, persistence, zoom, center camera. Details in [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md).
- **Servo integration guide**: [technical_architecture/SERVO_INTEGRATION_BRIEF.md](technical_architecture/SERVO_INTEGRATION_BRIEF.md).

---

## 🔗 Quick Links

- **Entry Point**: [README.md](README.md)
- **Architecture**: [ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md) ⭐ Required reading
- **Implementation Plan**: [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)
- **Browser Behavior**: [GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)
- **Build Setup**: [BUILD.md](BUILD.md)

---

**Last Updated**: February 10, 2026  
**Status**: Foundation complete, Servo integration sprint in progress  
**Next Feature**: Servo webview integration (create/destroy webviews, navigation hooks, lifecycle binding)



