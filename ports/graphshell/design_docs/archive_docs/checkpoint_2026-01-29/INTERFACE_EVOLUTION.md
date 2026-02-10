# Archived: Interface Evolution

This interface evolution note has been archived. The original document is available in the archive folder.

Archived copy: [archive_docs/INTERFACE_EVOLUTION.md](archive_docs/INTERFACE_EVOLUTION.md)

For the active interaction model and UI decision summary, see [GRAPH_INTERFACE.md](GRAPH_INTERFACE.md) and [README.md](README.md).

| Feature | Status | Notes |
|---------|--------|-------|
| **Graph-based spatial browsing** | ✅ MVP | Force-directed nodes + edges, force physics |
| **Map view as primary interface** | ✅ MVP | Graph canvas opens on app launch |
| **Tabs as connection list** | ✅ MVP | Tabs in detail window show edges to/from node, ordered chronologically |
| **Multi-window support** | ✅ MVP | Windows 11 snap-like layouts, side-by-side browsing |
| **Personal history view** | ✅ Phase 2 | Can reconstruct graph from browser history, personal OPTE-like view |
| **Search/filtering** | ✅ MVP | Live search, filter by title/URL/tags |
| **Clustering & grouping** | ✅ Phase 2 | Detect connected components, collapse/expand |

### 🔄 Deferred to Phase 3+ (Network & Blockchain)

| Feature | Old Plan | Current Plan | Timeline |
|---------|----------|--------------|----------|
| **Blockchain wayback machine** | Full feature | Research + prototype | Phase 3+ |
| **Sharing active tabs/webs** | P2P sync | Simple P2P + event-based merge | Phase 3 research |
| **Live collaborative browsing** | Sandbox instance of others' webs | Future network layer | Phase 4+ |
| **OPTE-style visualization** | 3D/5D archive overlay | Simpler: 3D toggle (Phase 2) | Phase 2–3 |
| **Time as scrollable dimension** | 5D (x, y, z, time, validation) | Time as edge metadata only | Phase 2+ |
| **Consensus validation** | Two-tier (blocks + relationships) | Research: event-based validation | Phase 3 |
| **Domain-level abstraction** | Primary grouping unit | Implicit (clustering by domain) | Phase 2–3 |
| **IP/location positioning** | User localization (where is "here"?) | Not in current plan | Future |

---

## Detailed Breakdown

### Older Vision: Full Ecosystem

The older notes sketched an ambitious vision:
1. **Personal history** as a personal OPTE (Internet Map-like visualization)
2. **Shared webs** — Access other people's active tab collections as sandboxed instances
3. **Blockchain validation** — Prior compiled versions of sites via wayback, validated by consensus
4. **Multi-dimensional** — Space (x, y, z) + time (scroll history) + validation strength
5. **Consensus organization** — Sites validated and organized by community consensus, not just traffic
6. **Domain-centric** — Group sites by domain (simpler relationships than all-pairs)

### Current Plan: Pragmatic MVP → Ecosystem

**Phase 1 (Weeks 1–8)**: Single-user force-directed graph
- Create/navigate nodes (URLs)
- Edges are simple connections (chronological, user-created)
- Detail windows show adjacent nodes as tabs
- Multi-window snapping for side-by-side exploration
- Save/load graph as JSON

**Phase 2 (Weeks 9–16)**: Personal features
- Persist graph to disk + bookmarks import
- Clustering (auto-detect groups)
- Filtering (by domain, date, tags)
- 3D visualization (optional)
- DOM extraction (capture page snippets)
- Export (JSON, HTML, PNG)

**Phase 3 (Weeks 17–24)**: Network & ecosystem research
- P2P sync model (event-based: node added/removed/updated)
- Shared graph merge (simple decentralized sync)
- Tokenization research (user data ownership, compensation)
- Validation model (how to trust shared graphs?)

---

## What Changed, Why

### 1. **Blockchain Wayback Machine → Deferred**
**Old**: Access prior validated versions of sites compiled from a blockchain
**Now**: Phase 3+ research; MVP focuses on current-state browsing

**Rationale**: 
- Blockchain adds complexity (validation, chain management, consensus)
- Current version capture (DOM extraction in Phase 2) covers snapshot use case
- Full wayback requires upstream validation infrastructure

### 2. **Sharing Active Webs → Simple P2P (Phase 3)**
**Old**: Click a person's shared web → live sandbox of their browsing
**Now**: Phase 3 P2P sync → can share graph structure + validate edges

**Rationale**:
- Sandbox instance requires container tech + ongoing sync
- MVP: Personal graphs only (no sharing)
- Phase 3: Simple event-based sync (graph structure, edge timestamps)
- Future: Layer in validation/sandbox after P2P stabilizes

### 3. **Time as Dimension → Edge Metadata**
**Old**: 5D visualization (x, y, z, time, validation); scroll through history
**Now**: Time stored as `created_at` on edges; 3D optional in Phase 2

**Rationale**:
- 5D is hard to visualize and implement
- Timestamp on edges enables chronological tab ordering (MVP requirement)
- 3D (Phase 2) provides optional depth; time through history becomes "navigate back in graph"
- Can revisit true time-dimension visualization in Phase 4+

### 4. **Consensus Validation → Research Phase**
**Old**: Two-tier validation (blocks + site relationships); consensus from community
**Now**: Phase 3 research; MVP has no validation (single-user)

**Rationale**:
- Consensus requires network + incentives
- MVP is local-only; can't validate without peers
- Phase 3 explores event-based validation (edge strength = how many users saw connection)
- Avoid premature blockchain commitment; MVP proves graph model first

### 5. **Domain-Level Abstraction → Implicit Clustering**
**Old**: Primary grouping: domains as zones; domains relate to other domains
**Now**: Nodes are URLs; Phase 2 auto-detects clusters by connectivity

**Rationale**:
- Site-level granularity more flexible (can mix domains in same cluster)
- Clustering algorithm (connected components) provides domain-like grouping automatically
- Can surface domain grouping in Phase 2 if useful

### 6. **User Localization → Not in Current Plan**
**Old**: "Where is 'here' online?" — IP-based, IRL location, site connections
**Now**: No positioning; graph position is physics-driven, not location-based

**Rationale**:
- User location (IP, physical) adds complexity without clear UX benefit for MVP
- Graph layout driven by relationships, not geography
- Could revisit for Phase 3+ (e.g., "nearby" peers in P2P network)

---

## Key Deferral Points

| Complexity | MVP Avoids | Phase 3+ Adds |
|------------|-----------|--------------|
| **Consensus** | None; single user | Event-based validation + stakes |
| **Sync** | No sharing | P2P merge of graphs |
| **Archive** | Current snapshots | Blockchain wayback? |
| **Visualization** | 2D + optional 3D | 5D (time scrolling), OPTE overlay |
| **Geography** | None | IP/location positioning |
| **Sandboxing** | N/A | Live collaborative web instances |

---

## Alignment with Vision

### What's NOT Changing
- **Graph is primary interface** ✅ (unchanged)
- **Spatial/force-directed layout** ✅ (core MVP)
- **Tabs reflect graph structure** ✅ (connection list model)
- **Personal knowledge management** ✅ (history import, clustering)
- **Eventual ecosystem play** ✅ (Phase 3+ P2P + tokenization)

### What's Being Pragmatized
- **Blockchain validator** → Local timestamp-based edge validation
- **Shared sandbox webs** → Simple P2P graph sync
- **5D visualization** → 3D toggle + time metadata
- **Consensus mechanism** → Event-based (revisit in Phase 3)
- **Domain zones** → Implicit clustering algorithm

---

## Recommended Phase 3+ Research Topics

After MVP is solid (Phase 1–2), investigate:

1. **Validation model**: How do we trust edges in a shared graph?
   - Options: Majority vote, stake-weighted, timestamp-based, user reputation
   - Blockchain vs. decentralized append-only log vs. event sourcing

2. **Sandbox & live sharing**: How to give access to someone's live web?
   - Options: WebContainer API, Docker, virtual browser, HTML archive + snapshots

3. **Wayback integration**: Can we leverage existing wayback machines + blockchain?
   - OPTE-like (visual archive), but simpler
   - Or build blockchain archive incrementally as users visit sites?

4. **Tokenization**: How do we compensate contributors/validators?
   - Data marketplaces, governance tokens, reputation systems
   - Avoid speculation; focus on legitimate user incentives

5. **Time dimension**: Revisit after Phase 3 P2P is working
   - Could allow "history scrubbing" (view graph at point in time)
   - Or explicit version control per graph snapshot

---

## Summary for Development

**Start with Phase 1 MVP exactly as documented** — it's sound architecture that doesn't preclude the bigger vision. The deferred features (blockchain, sharing, consensus) are additive layers that can be built on top of a solid single-user graph foundation.

Avoid premature implementation of:
- Blockchain validation (wait for clear use case in Phase 3)
- Sandbox instances (wait for P2P + user demand)
- 5D visualization (3D + metadata is sufficient for MVP)

**The MVP proves the core insight**: force-directed graphs work for spatial browsing. Once that's solid, the network + ecosystem features become tractable engineering problems rather than architectural risks.

