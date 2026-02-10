# Design Docs Search Findings Summary

Comprehensive analysis of design documentation across Graphshell project. Searched: `design_docs/` and `design_docs/archive_docs/` folders.

Note: This file now lives in `verse_docs/`. Some links still assume the `design_docs/` root; add `../` if needed.

**Date**: February 4, 2026  
**Files Analyzed**: 27 markdown files across main and archive directories

---

## 1. P2P Synchronization, Collaborative Features, Shared Graph Updates

### Current Plan Status: **Deferred to Phase 3+**

**Key Files:**
- [ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md#L510-L533) (Section 21: Future Architecture)
- [IMPLEMENTATION_ROADMAP.md](../IMPLEMENTATION_ROADMAP.md#L518)
- [VERSE.md](VERSE.md)

**Findings:**

**Phase 1-2: Local-only (MVP)**
- MVP prioritizes local persistence with incremental saves (every 30 seconds)
- No P2P/sync implementation in Phase 1
- Architecture explicitly designed for future modularity

**Phase 3+: Planned Optional Modules**
From [ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md#L510):
```
Phase 2+: Optional modules
  - Local sync (file-based)
  - P2P sync (YaCy-style, Syncthing-like)
  - Distributed storage (IPFS, Arweave)
  - Token system (only if needed for incentives)
```

**Trait-Based Design for Future Sync:**
```rust
pub trait SyncBackend {
    fn push(&self, graph: &Graph) -> Result<()>;
    fn pull(&self) -> Result<Graph>;
    fn merge(&self, local: &Graph, remote: &Graph) -> Result<Graph>;
}

impl SyncBackend for LocalFilesystem { }
// Later: impl SyncBackend for P2PSync { }
```

**Shared Graph Updates Strategy:**
- Graph changes tracked via **dirty tracking** (only changed nodes/edges written)
- Session history: Keep last 10 full snapshots
- On open: Load latest session + replay unsaved deltas
- No CRDT/OT (operational transformation) currently planned for MVP

**Rationale (from [PROJECT_PHILOSOPHY.md](../archive_docs/PROJECT_PHILOSOPHY.md)):**
> "Local-first storage, with optional P2P sync later"  
> "Personal/Local-First, Not Collaborative-First"

- MVP focus: Spatial UX for single user
- Sync needed only if P2P proven useful in Phase 3
- Personal use case (one user, one machine) is primary in MVP

---

## 2. Firefox Architecture Decisions: Process Isolation, Multiprocess, Content Process Management

### Heavy Influence on Graphshell Architecture

**Key Files:**
- [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L491-L509) (Section 20: Process Isolation)
- [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L9-L100)
- [INDEX.md](INDEX.md#L72-L85)

**Findings:**

**Process Isolation Pattern (from Firefox):**
From [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L491):
```
Graph UI runs in compositor thread (trusted)
Webviews run in sandboxed Servo processes (untrusted)
```

**Launch Commands:**
```bash
cargo run --release -- -M -S
# -M: Multiprocess (Servo spawns content processes)
# -S: Sandbox (gaol, seccomp applied to content)
```

**Origin-Based Process Management (Servo's Model):**
From [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L54):
> "Firefox's approach (kill unused processes) more efficient than fixed pool + serialization"  
> "No serialization latency (create/destroy processes, not serialize DOM)"

**Process Lifecycle:**
1. User creates node from origin A → Servo spawns process for A
2. User closes all nodes from origin A → Servo kills process for A
3. User creates node from origin A again → New process spawns
4. **No webview reuse, no serialization, no dormant processes**

**Why This Matters:**
- Origin-grouped nodes map naturally to origin-grouped processes
- Firefox validates this pattern at scale (used in production)
- Servo's `-M` flag already implements this; no custom implementation needed
- Memory: Processes killed when all nodes for that origin close

**Alternative Rejected:**
- Fixed pool (simpler, but wasteful)
- Webview pooling with serialization (adds latency)

**Servo's Advantage Over Chrome Approach:**
- Servo's origin grouping is proven (Firefox uses it)
- No explicit DOM serialization needed
- Better payoff despite higher complexity upfront

---

## 3. CRDT (Conflict-free Replicated Data Type) or Operational Transformation Approaches

### Current Plan: **NOT PLANNED for MVP**

**Key Files:**
- [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L510-L533) (Modularity for future sync)
- [PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md#L100-L180)

**Findings:**

**Explicit Non-Use in MVP:**
From [PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md#L170):
> "No need for crdt/conflict resolution in MVP"
> "Personal use case (one user, one machine) is primary"

**Why CRDTs Deferred:**
- Designed for single-user, local-first workflow
- P2P sync (Phase 3+) would need conflict resolution
- Dependency injection via `SyncBackend` trait allows CRDT implementation later
- Current focus: Reliable local persistence with version control

**Conflict Resolution Strategy (for future P2P):**
The architecture stub shows:
```rust
pub trait SyncBackend {
    fn merge(&self, local: &Graph, remote: &Graph) -> Result<Graph>;
}
```
- Merge function signature prepared but not implemented
- Implementations could use: CRDTs, OT, last-write-wins, or custom resolution
- Decision deferred to Phase 3 when collaboration needs are clear

**Version Control Instead of CRDT:**
Current approach for MVP:
- Session snapshots: Keep last 10 full versions
- Dirty tracking: Only changed data persisted
- Replay semantics: Unsaved deltas replayed on load
- Simple, local, effective for single user

---

## 4. Anytype, Obsidian, Notion, OneNote, Google Docs, Office Architectures

### Current Plan: **NO DIRECT ARCHITECTURAL BORROWING, PARTIAL FEATURE INSPIRATION**

**Key Files:**
- [PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md) (Feature set comparisons)
- [COMPREHENSIVE_SYNTHESIS.md](archive_docs/COMPREHENSIVE_SYNTHESIS.md)

**Findings:**

**NOT Mentioned in Design Docs:**
No explicit architectural analysis of these products. No CRDT/sync patterns borrowed.

**Implied Lessons (Inferred from Graphshell Philosophy):**

**Different Problem Space:**
- **Notion/Obsidian**: Block-based notetaking, collaborative editing
- **Graphshell**: Spatial browser, knowledge graph visualization
- **Google Docs/Office**: Real-time collaborative editing
- **Graphshell MVP**: Single-user, local-first, graph-centric

**Feature Inspiration (from [PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md)):**
- Session management (like browser tabs, but explicit "sessions")
- DOM inspector (similar to Obsidian web clipper)
- Export formats: JSON, PNG, interactive HTML (similar to Obsidian)
- Sidebar option (optional tabs, for users preferring traditional interface)

**What Graphshell Explicitly Rejects:**
- Real-time collaborative editing (Phase 3+ only)
- Block-based nested structure (graph-first instead)
- Commercial sync infrastructure (P2P if implemented)

**Recommended Future Analysis (Phase 2-3):**
- Study Obsidian's local-first sync approach
- Review how Notion handles schema migrations
- Consider OneNote's conflict-free merge strategies if P2P implemented

---

## 5. YaCy-Style Decentralized Search

### Current Plan: **Explicitly Mentioned for Phase 3+**

**Key Files:**
- [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L518) (P2P sync options)
- [verse_docs/VERSE.md](verse_docs/VERSE.md) (Phase 3 tokenization)

**Findings:**

**Direct Reference:**
From [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L518):
```
Phase 2+: Optional modules
  - P2P sync (YaCy-style, Syncthing-like)
```

**YaCy Model Consideration:**
YaCy = peer-to-peer search engine where users share search indices.

**Graphshell's Potential Application:**
- Phase 3+: Optional P2P search across shared graphs
- Users could seed indexed graph fragments
- Other users discover knowledge through DHT (distributed hash table)
- Decentralized alternative to centralized search

**Not Detailed Yet:**
- No technical spec for YaCy integration
- Would be part of larger P2P sync infrastructure
- Paired with token incentives (Phase 3 research)

**Related Concept: Verse Indexers**
From [verse_docs/VERSE.md](verse_docs/VERSE.md):
```
Peer roles:
- Indexers/deduplicators: dedupe and index reports for efficient queries
```

This aligns with YaCy's distributed indexing model.

---

## 6. DOM Serialization/Deserialization Approaches

### Current Plan: **EXPLICITLY AVOIDED**

**Key Files:**
- [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L55, #L221)
- [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L54-L67)

**Findings:**

**Explicit Non-Implementation:**
From [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L221):
> "No explicit DOM serialization (too complex, Servo handles it)"

From [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L55):
> "No serialization latency (create/destroy processes, not serialize DOM)"

**Why Serialization Rejected:**
1. **Complexity**: DOM state is complex to serialize/deserialize reliably
2. **Servo Responsibility**: Servo manages DOM; no need to duplicate
3. **Process Model**: Kill/spawn processes instead of serializing state
4. **Latency**: Serialization causes ~150ms UI lag spikes (unacceptable)

**What IS Serialized:**
Only graph metadata serialized to JSON:
```json
{
  "nodes": [
    {
      "id": "node123",
      "url": "https://example.com",
      "title": "Page Title",
      "favicon": "...",
      "tags": ["research", "example"],
      "created_at": "2025-02-04T...",
      "metadata": { ... }
    }
  ],
  "edges": [...]
}
```

**Webview State Handling:**
- Each origin's webview is a separate Servo process
- Process created on demand, destroyed when not needed
- No attempt to preserve/restore DOM state
- User refreshes page if needed (fast, within 1-2 seconds)

**Alternative Considered and Rejected:**
- Fixed pool with dormant processes (serialization overhead)
- Process pooling with state serialization (latency spike)

---

## 7. Untrusted Data Handling in Graph Nodes

### Current Plan: **SANITIZATION + SERVO SANDBOXING**

**Key Files:**
- [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L461-L478) (Section 19)
- [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L868)

**Findings:**

**Decision from [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L461):**
> "Sanitize user-visible data. Validate URLs. Trust Servo for webview sandboxing."

**Implementation Strategy:**

**Input Sanitization (Compositor):**
```rust
fn sanitize_label(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.".contains(*c))
        .collect()
}

fn validate_url(url: &str) -> Result<Url> {
    let parsed = Url::parse(url)?;
    match parsed.scheme() {
        "http" | "https" | "file" => Ok(parsed),
        _ => Err(InvalidScheme),
    }
}
```

**Threats Addressed:**

1. **Node Labels from Untrusted Sources:**
   - Page title from `<title>` tag: Sanitize before display
   - Open Graph metadata: Sanitize before display
   - User-entered tags: Validate input

2. **URLs:**
   - Only http, https, file schemes allowed
   - Other schemes (javascript:, data:) rejected
   - URL parsing via robust library (Url crate)

3. **Webview Content:**
   - Runs in sandboxed Servo process (gaol, seccomp)
   - Process isolation prevents escape to compositor
   - Can't directly manipulate graph nodes
   - Sandboxing handled by Servo, not reimplemented

**Attack Surface:**
- Compositor displays user-visible data (sanitized)
- Webview processes can't access compositor memory
- Graph data protected by process boundary

**No Direct DOM Inspection:**
- Don't manually parse/serialize DOM
- Use Servo's safe interfaces only
- Avoid raw HTML parsing

---

## 8. Export/Import Formats and Interoperability

### Current Plan: **PHASE 2 FEATURE, BASIC SUPPORT**

**Key Files:**
- [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L348, #L539, #L606)
- [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)

**Findings:**

**Planned Export Formats:**
From [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L606):
```
- [ ] Export options (PNG, SVG, JSON)
```

**Phase 2 Milestones:**

| Format | Phase | Status | Purpose |
|--------|-------|--------|---------|
| **JSON** | 2 | Planned | Graph persistence, interoperability |
| **PNG** | 2 | Planned | Static visualization, sharing |
| **SVG** | 2 | Planned | Vector export, scalable |
| **Interactive HTML** | 3+ | Research | Standalone graphs with embedded webviews |

**JSON Format Details:**
From [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L738):
```
Serialization: serde_json
Standard, readable
```

**Save Performance Target:**
From [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L630):
```
Serialization: 10K graph < 500ms
```

**Import Capabilities:**

From [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L496):
```
- [ ] Import from Chrome/Firefox bookmarks.html
```

**Future Interoperability (Phase 3+):**
- Node-level sharing: `graphshell://node?id=abc123&title=...&url=...&tags=...`
- Standalone JSON cards for individual nodes
- Interactive HTML export (complex; deferred)

**Alternative Approaches (Deferred):**
- OPML export (for outliner interop)
- RDF/semantic web formats
- Markdown graph notation

---

## 9. Session/Browsing History Storage Concepts

### Current Plan: **PHASE 2 FEATURE, PARTIALLY SPECIFIED**

**Key Files:**
- [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L158-L180) (Section 6: Persistence)
- [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L291, #L529-L550)
- [PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md#L180-L230)

**Findings:**

**Session Persistence Strategy:**
From [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L160):
> "Incremental saves with version control and session history"

**Storage Structure:**
```
~/.config/graphshell-graph/
├── sessions/
│   ├── current.json           # Latest full session
│   ├── session_1707000000.json
│   ├── session_1707000900.json
│   └── ... (up to 10 versions)
├── preferences.toml
├── keybinds.toml
└── theme.toml
```

**Auto-Save Mechanism:**
From [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md#L165):
```
- Auto-save every 30 seconds (configurable)
- Dirty tracking: Only changed nodes/edges written
- Session history: Keep last 10 full snapshots
- On close: Write complete session
- On open: Load latest session + replay unsaved deltas
```

**Browsing History Types (from [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L277)):**
```
Edge types: Hyperlink (blue), Bookmark (green), History (gray), Manual (red)
```

**Phase 2 Session Features:**
From [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md#L545-L548):
```
- [ ] Session history:
  - Track visited nodes (like browser history)
  - Search history
  - Clear history
```

**Phase 3+ Concepts (from [PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md#L220)):**
```
Option A: Each session is a separate graph file
Option B: One graph with timestamps; can rewind/replay
Option C: Like browser history; can collapse old branches
```

**Not Yet Specified:**
- Session naming/tagging
- Cross-session search
- Session comparison (diff two sessions)
- History visualization timeline

---

## 10. Ghost Nodes Concept

### Current Plan: **PHASE 2+ OPTIONAL FEATURE, LOW PRIORITY**

**Key Files:**
- [COMPREHENSIVE_SYNTHESIS.md](archive_docs/COMPREHENSIVE_SYNTHESIS.md#L207-L226)
- [PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md#L100-L130, #L275-L290)

**Findings:**

**Definition:**
From [PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md#L100):
> "Use ghost nodes to preserve structure when removing items"

**Concept Explanation:**
From [COMPREHENSIVE_SYNTHESIS.md](archive_docs/COMPREHENSIVE_SYNTHESIS.md#L210):
> "When you delete a node, keep the edges visible (as 'ghost edges'), but dim/style them differently"

**Use Case:**
From [PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md#L285):
> "Knowledge organization; you remove a page but want to remember it was related to others"

**Implementation Recommendation (Phase 2):**
From [COMPREHENSIVE_SYNTHESIS.md](archive_docs/COMPREHENSIVE_SYNTHESIS.md#L220):
```
- Optional feature: "Show ghost connections" toggle
- When node deleted, create "GhostEdge" (visual only, no target)
- Render as dashed/faded line
- Can be turned off in settings
```

**Data Structure Addition:**
```rust
pub enum Edge {
    Normal { from: NodeKey, to: NodeKey, ty: EdgeType },
    Ghost { from: NodeKey, to: TombstonedNodeId, ty: EdgeType },
}
```

**Visual Representation:**
- Dashed lines (vs solid for normal edges)
- Reduced opacity/gray color
- Optional: Toggle in settings to hide/show

**Status in MVP:**
- **NOT in Phase 1** (MVP)
- **Proposed for Phase 2** (low priority)
- **Rationale**: Nice-to-have; doesn't block core functionality
- **Complexity**: Adds ~100 lines of code; straightforward

**Related Features (Deferred):**
- Tombstoning (mark nodes as deleted but keep metadata)
- Undo/redo for deletions
- Ghost node visualization in history timeline

---

## Summary Table: All Topics

| Topic | Phase | Status | Key File | Quote/Reference |
|-------|-------|--------|----------|-----------------|
| **P2P Sync** | 3+ | Deferred | ARCHITECTURE_DECISIONS.md | "Design for modularity, but don't implement P2P/sync in MVP" |
| **Collaborative Editing** | 3+ | Deferred | PROJECT_PHILOSOPHY.md | "Real-time sync deferred" |
| **CRDT/OT** | 3+ | Not Planned | PROJECT_PHILOSOPHY.md | "No need for crdt/conflict resolution in MVP" |
| **Firefox Patterns** | 1 | Implemented | ARCHITECTURE_DECISIONS.md | "Leverage Servo's origin-grouped multiprocess" |
| **Process Isolation** | 1 | Core Design | ARCHITECTURE_DECISIONS.md | "Compositor separate from content processes" |
| **YaCy Search** | 3+ | Proposed | ARCHITECTURE_DECISIONS.md | "P2P sync (YaCy-style, Syncthing-like)" |
| **DOM Serialization** | N/A | Rejected | ARCHITECTURE_DECISIONS.md | "No explicit DOM serialization (too complex)" |
| **Untrusted Data** | 1 | Core Design | ARCHITECTURE_DECISIONS.md | "Sanitize labels, validate URLs, trust Servo sandboxing" |
| **Export/Import** | 2 | Planned | IMPLEMENTATION_ROADMAP.md | "Export as PNG/JSON; import from browser bookmarks" |
| **Sessions/History** | 1-2 | Partial | ARCHITECTURE_DECISIONS.md | "Keep last 10 full snapshots, incremental saves" |
| **Ghost Nodes** | 2+ | Optional | COMPREHENSIVE_SYNTHESIS.md | "Use ghost edges to preserve structure when deleting" |

---

## Architecture Philosophy Summary

**Core Principles (from PROJECT_PHILOSOPHY.md):**
1. **Learning-first**: Ship early, iterate based on use
2. **Local-first**: MVP = single-user, local persistence
3. **Sense-making**: Built for research/knowledge organization
4. **Modularity**: Phase 1 sets foundation for P2P/sync later
5. **Optionality**: Multiple view modes, export formats, physics presets

**What's **NOT** in MVP:**
- Collaborative editing / real-time sync
- CRDTs or conflict resolution
- DOM serialization / process pooling
- YaCy-style distributed search
- 3D graph visualization
- Token/economic system

**What **IS** in MVP:**
- Force-directed graph visualization
- Origin-grouped multiprocess (Servo's `-M` flag)
- Process isolation (compositor vs content)
- Local persistence with version history (last 10 snapshots)
- Input sanitization + Servo sandboxing
- Basic search (prefix match, upgrade to fuzzy in Week 3)
- Basic export (JSON; PNG/SVG in Phase 2)

---

## Recommendations for Further Research

1. **CRDT Implementation** (If P2P sync needed in Phase 3):
   - Study Yjs, Automerge, or similar Rust implementations
   - CRDTs suited for collaborative graph editing

2. **YaCy/Decentralized Search** (If Phase 3 P2P enabled):
   - Review YaCy protocol (DHT-based peer indexing)
   - Design schema for graph fragment sharing

3. **Obsidian Sync Pattern** (For local-first inspiration):
   - Obsidian's vault system (local folder + optional sync)
   - Useful for understanding feature parity

4. **Notion/OneNote Architectures** (For feature inspiration):
   - Block-based nesting (orthogonal to Graphshell's graph model)
   - Conflict resolution strategies for future reference

5. **Webview Pool Optimization** (Phase 2):
   - Monitor if process creation/destruction adds latency
   - Consider Verse's IPC-based helper process model if bottleneck found

---

**Document Generated**: 2025-02-04  
**Total Design Docs Reviewed**: 27 files  
**Search Queries**: 10 comprehensive semantic searches + 15 targeted grep searches
