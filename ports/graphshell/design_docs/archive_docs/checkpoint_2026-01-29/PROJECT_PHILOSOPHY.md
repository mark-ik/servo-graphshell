# Archived: Project Philosophy & Vision

This philosophy document has been archived. The original is preserved in the archive folder for historical context.

Archived copy: [archive_docs/PROJECT_PHILOSOPHY.md](archive_docs/PROJECT_PHILOSOPHY.md)

For the current, consolidated project overview and roadmap see [README.md](README.md).

### Learning-First, Shipping-Second

> The goal is to learn, explore, and see what feels natural—not to compete with mainstream browsers.

**Implication for migration plan**:
- Ship early, iterate based on lived experience
- User research = using it yourself for a month
- Don't over-engineer for hypothetical users
- Embrace rough edges; they teach you something

### Problem Space: Relationship Visualization

**Origin**: RimWorld mod that visualized relationships as a graph. Made the user wonder why tabs (which also form relationships) couldn't be graphs.

**Core insight**: Browsing *already* has implicit structure (links, history, related pages). A graph visualization just makes that structure explicit.

**Why it matters**:
- This is fundamentally about **sense-making** and **knowledge organization**
- Not about speed, not about features
- For workflows like research, notetaking, cross-site synthesis

---

## Feature Set from Vision (Not All in Current Plan)

### Phase 1 Features (Currently in Migration Plan) ✅

| Feature | Vision | Current Plan | Status |
|---------|--------|---|---|
| Force-directed graph canvas | ✅ Core | ✅ MVP | GO |
| WASD/mouse reticule controls | ✅ Core | ✅ MVP | GO |
| Node = webpage | ✅ Core | ✅ MVP | GO |
| Edge = relationship | ✅ Core | ✅ MVP | GO |
| Save/load graph | ✅ Core | ✅ Phase 2 | OK |
| 2D canvas | ✅ Core | ✅ MVP | GO |
| Search/filter | ✅ Core | ✅ MVP | GO |
| Multi-window (detail/split) | Implicit | ✅ MVP | GO |

### Phase 2 Features (Recognized but Not Detailed) 🔄

| Feature | Vision | Current Plan | Status |
|---------|--------|---|---|
| 3D modes | Optional | Phase 2 | Planned |
| DOM inspector + extraction | ✅ Detailed | Phase 2 | Planned |
| Export (JSON, PNG, HTML) | ✅ Detailed | Phase 2 | Planned |
| Clustering by relatedness | ✅ Detailed | Phase 2 | Planned |
| Share nodes as URLs + metadata | ✅ Detailed | Phase 2 | Not explicit |
| Sidebar window manager (optional tabs) | ✅ Detailed | Not explicit | Missing |
| Ghost nodes (preserve structure) | ✅ Detailed | Not explicit | Missing |
| Level-of-detail rendering | ✅ Detailed | Phase 2 (implicit) | Planned |

### Phase 3+ Features (Deferred) 🔮

| Feature | Vision | Current Plan | Status |
|---------|--------|---|---|
| Persistent sessions / sessions library | ✅ Detailed | Not mentioned | Missing |
| Collaborative/shared graphs | ✅ Explored | Phase 3 (P2P) | Partial |
| Real-time sync | ✅ Explored | Phase 3 | Deferred |
| IPFS integration | ✅ Mentioned | Not mentioned | Future |
| Tokenized contributions | ✅ Mentioned | Phase 3 research | Aligned |
| Decentralized search (YaCy-inspired) | ✅ Mentioned | Not mentioned | Future |
| Desire paths / semantic suggestions | ✅ Mentioned | Not mentioned | Future |

---

## Vision → Current Plan Alignment

### What's Aligned ✅

**Core principles are sound**:
- Force-directed graph as primary interface ✅
- Spatial/ergonomic controls (WASD) ✅
- Graph as knowledge tool, not browser replacement ✅
- Optional, experimental features (P2P, tokenization) ✅
- Servo as rendering engine ✅
- Personal/learning-first mindset ✅

**MVP scope respects vision**:
- Not trying to replace tabs (MVP allows detail window with tabs)
- Not competing on features (MVP is minimal chrome, no bookmarks/history UI)
- Focused on core graph interaction ✅

### What's Missing from Migration Plan 🔴

1. **Sessions/Browsing History Storage**
   - Vision mentions: "Save, delete, or share historical sessions"
   - Current plan: Persistence of graph structure (Phase 2), not sessions
   - Gap: How do you save distinct browsing contexts? Folder-like collections?

2. **Ghost Nodes (Preserve Structure)**
   - Vision: "Use ghost nodes to preserve structure when removing items"
   - Current plan: Delete removes node entirely
   - Gap: When you delete a node, do edges stay visible (ghost edge)?

3. **Sidebar Window Manager (Optional Tabs)**
   - Vision: "Optional sidebar window manager for people who still want tabs"
   - Current plan: Detail window is primary; no sidebar option
   - Gap: Could add togglable sidebar showing all open nodes as list

4. **Share Individual Nodes as URLs + Metadata**
   - Vision: "Share individual nodes as URLs with metadata"
   - Current plan: Export graph (Phase 2), not individual node sharing
   - Gap: How to create shareable link to one node with its content/metadata?

5. **Interactive HTML Export**
   - Vision: "Export graphs as JSON, PNG, or interactive HTML"
   - Current plan: JSON export (Phase 2), not interactive HTML
   - Gap: Could generate standalone HTML with graph visualization + embedded webviews (Phase 2+)

6. **DOM Inspector + Element Extraction**
   - Vision: "DOM/element inspector that lets you pull elements out into the graph"
   - Current plan: DOM extraction mentioned (Phase 2); inspector not mentioned
   - Gap: Inspector UI for selecting/extracting DOM elements

7. **Collaborative/Real-time Sync**
   - Vision: "Real-time shared sessions" and "Asynchronous editing like Obsidian"
   - Current plan: P2P sync (Phase 3); no mention of real-time or async editing
   - Gap: Could be post-Phase 3; design model for conflict resolution

---

## Design Implications from Vision

### 1. **Personal/Local-First, Not Collaborative-First**

**Vision states**: "Local‑first storage, with optional P2P sync later"

**Current migration plan**: P2P in Phase 3

**Implication**: 
- MVP should prioritize local persistence (save/load graph reliably)
- Sync is nice-to-have, not core
- No need for crdt/conflict resolution in MVP
- Personal use case (one user, one machine) is primary

**Recommendation**: 
- Phase 2 should include robust local persistence (JSON + schema migration)
- Phase 3 can explore P2P without disrupting Phase 2 architecture

---

### 2. **Rough Edges Are OK; Learning is the Goal**

**Vision states**: "Embrace rough edges; they teach you something"

**Current migration plan**: 8-week MVP with polish

**Implication**:
- MVP doesn't need perfect UX
- Can ship with limitations (e.g., no history UI, basic export)
- User feedback from your own use is more valuable than external polish
- Phase 2+ refines based on what you learned using Phase 1

**Recommendation**:
- Focus Phase 1 on core graph interaction; accept rough edges elsewhere
- Plan for Phase 1.5 (1–2 week reflection/feedback period) before Phase 2
- Document what you learned; let that guide Phase 2 priorities

---

### 3. **Workflows: Research, Notetaking, Sense-Making**

**Vision states**: "Ideal for notetaking or cross‑site research"

**Current migration plan**: Generic "knowledge management"

**Implication**:
- Test MVP on **real workflows** (your own research, notetaking)
- Features that support these workflows are high-priority
- Performance features (LOD, clustering, filtering) matter for sense-making
- Search/tagging should support thinking, not just retrieval

**Recommendation**:
- During Phase 1, use Graphshell for your own research/notetaking for 2–4 weeks
- Identify what's missing; prioritize Phase 2 based on your actual use
- Document workflows in README; helps future users find similar use cases

---

### 4. **Optionality and Flexibility**

**Vision emphasizes options/flexibility**:
- "optional sidebar window manager for people who still want tabs"
- "2D for dense maps or low‑power devices; 3D for spatial depth"
- "persist graphs, or start fresh each session"
- "Export as JSON, PNG, or interactive HTML"

**Current migration plan**: Single path per phase

**Implication**:
- MVP can start with one mode (force-directed 2D)
- But architecture should allow optionality to be added later
- Example: Detail window should be optional (sidebar alternative in Phase 2)
- Settings should expose physics presets, appearance options, etc.

**Recommendation**:
- Phase 1: Ship one coherent path (2D, detail windows, game controls)
- Phase 2: Add optionality (3D toggle, sidebar option, multiple presets)
- Phase 3: Full customization layer (theming, layout modes, export formats)

---

## Missing/Underspecified Features from Vision

### 1. Sessions/Browsing History Management

**Vision idea**: "Start fresh each session or keep a persistent graph. Save, delete, or share historical sessions."

**Current plan**: Graph persists, but no concept of "sessions"

**Design question**: What is a "session"?
- Option A: Each session is a separate graph file (Graphshell_session_2025-01-28.json)
- Option B: One graph, with timestamps on nodes; can rewind/replay
- Option C: Like browser history; can collapse old branches

**Recommendation for Phase 2**:
- Add menu option: "New session" (creates fresh graph, auto-saves previous)
- Or: "Session history" view (tree of past graphs; can reload)
- Implement as folder of JSON files: `~/.graphshell/sessions/`

---

### 2. Ghost Nodes (Preserve Structure)

**Vision idea**: "Use ghost nodes to preserve structure when removing items"

**Explanation**: When you delete a node, keep the edges visible (as "ghost edges"), but dim/style them differently. This preserves the knowledge that "X was connected to Y" even after removing X.

**Use case**: Knowledge organization; you remove a page but want to remember it was related to others.

**Recommendation for Phase 2**:
- Optional feature: "Show ghost connections" toggle
- When node deleted, create "GhostEdge" (visual only, no target)
- Render as dashed/faded line
- Can be turned off in settings

---

### 3. Sidebar Window Manager (Optional Tabs)

**Vision idea**: "Optional sidebar window manager for people who still want tabs"

**Current plan**: Detail window on click; no sidebar option

**Design**: Add toggle in settings/menu
- OFF (default): Detail windows open (current behavior)
- ON: Sidebar on right shows list of open nodes; click to open/close

**Recommendation for Phase 2**:
- Add "View" menu option: "Show sidebar"
- Sidebar lists all open nodes as rows
- Click to focus; right-click to close
- Makes it easy for users coming from tab browsers

---

### 4. Share Nodes as URLs + Metadata

**Vision idea**: "Share individual nodes as URLs with metadata"

**Explanation**: Generate a link like `graphshell://node/abc123?title=Example&url=...&tags=...` that, when clicked, loads that node into the graph with metadata.

**Use case**: Share research findings with collaborators; they can load your node into their graph.

**Recommendation for Phase 3**:
- Node detail window has "Share" button
- Generates URL or QR code
- Link format: `graphshell://node?id=abc123&title=...&url=...&metadata=...` (URL-encoded)
- Or simpler: Export node as JSON, share file

---

### 5. Interactive HTML Export

**Vision idea**: "Export graphs as JSON, PNG, or interactive HTML"

**Current plan**: JSON (Phase 2), PNG (Phase 2); no interactive HTML

**Explanation**: Generate a standalone HTML file with:
- Graph visualization (canvas/webgl)
- Force simulation (wasm or JS port)
- Clickable nodes (load webpage or show metadata)
- Works offline

**Recommendation for Phase 3**:
- Use wasm/wgpu for export? Or JavaScript force simulator?
- Complex; defer to Phase 3+
- MVP: JSON + PNG sufficient

---

### 6. DOM Inspector + Element Extraction UI

**Vision idea**: "DOM/element inspector that lets you pull elements out into the graph"

**Explanation**: Right-click on a webpage element; select "Extract to graph". Creates new node with that element's content.

**Use case**: Research notetaking; pull relevant passages from multiple sites, organize in graph.

**Current plan**: DOM extraction (Phase 2); no inspector UI

**Recommendation for Phase 2**:
- Right-click page element → "Extract as note"
- Creates new node with element HTML/screenshot
- Or: Highlight tool (Ctrl+H) to select and extract

---

## Updating Migration Plan: Philosophy Section

The current [GRAPH_BROWSER_MIGRATION.md](GRAPH_BROWSER_MIGRATION.md) should have a **Philosophy & Vision** section at the top explaining:

1. **What Graphshell Is Not**
   - Not a browser competitor
   - Not meant to replace Chrome/Firefox for all users
   - Not optimized for speed/features

2. **What Graphshell Is**
   - A personal experiment in spatial thinking
   - A knowledge/sense-making tool for research/notetaking
   - An exploration of alternative UI metaphors
   - A learning project (priorities: understanding, then users, then polish)

3. **Success Metrics**
   - Does it help you think differently?
   - Do you use it for your own work?
   - Does it reveal interesting insights about browsing patterns?

4. **Design Principles**
   - Local-first, sync optional
   - Optionality and flexibility valued
   - Rough edges are ok; learning is the goal
   - Game-like, ergonomic controls
   - Visual representation of structure

---

## Revised Phase Priorities (Based on Vision)

### Phase 1 (Weeks 1–8): Core Spatial Browsing

**Priorities** (in order):
1. Force-directed graph + physics
2. WASD + mouse controls
3. Node launching (detail window)
4. Save/load graph (JSON)
5. Search/filter
6. Multi-window snapping (optional detail windows)

**Not in Phase 1** (can live without):
- History UI
- Bookmarks
- Metadata rich UI
- Export formats
- 3D

---

### Phase 1.5 (1–2 weeks): Reflection & Feedback

**Activity**: Use Graphshell for your own research/notetaking for 2–4 weeks
**Deliverable**: Document what works, what's missing, what's surprising
**Outcome**: Prioritize Phase 2 based on lived experience

---

### Phase 2 (Weeks 9–16): Sense-Making Features

**Priorities** (in order):
1. Persistence: Sessions/history (save/load past graphs)
2. DOM extraction + inspector
3. Clustering/grouping by relatedness
4. Export (JSON, PNG; interactive HTML is nice-to-have)
5. Filtering/tagging
6. 3D toggle
7. Sidebar window manager (optional tabs)
8. Level-of-detail rendering (prevent clutter)

---

### Phase 3 (Weeks 17–24): Sharing & Ecosystem

**Priorities** (in order):
1. P2P sync (async, not real-time)
2. Share nodes as URLs/metadata
3. Tokenization research
4. Collaborative/real-time (maybe; depends on Phase 2 learnings)

---

## Summary: Vision → Implementation

| Vision Aspect | Current Plan | Recommendation |
|---|---|---|
| **Personal experiment** | Implicit | Add Philosophy section to docs |
| **Local-first** | Phase 3 (P2P optional) | Prioritize Phase 2 persistence |
| **Sense-making/research** | Implicit | Use for real work; let that drive Phase 2 |
| **Optionality** | Basic | Phase 2: Add multiple presets/modes |
| **Sessions/history** | Not explicit | Add to Phase 2 priorities |
| **Ghost nodes** | Not mentioned | Phase 2 feature (low-priority) |
| **Sidebar tabs option** | Not mentioned | Phase 2 feature (easy win) |
| **Share node URLs** | Not mentioned | Phase 3 feature |
| **Interactive HTML** | Not mentioned | Phase 3+ feature |
| **DOM inspector** | Phase 2 | Detail in Phase 2 spec |
| **Collaborative** | Phase 3 (P2P) | Async before real-time |

**No major conflicts.** The migration plan respects the vision. Main additions:
- Philosophy/principles documentation
- Phase 1.5 reflection period
- Session/history management in Phase 2
- Optional sidebar tabs in Phase 2
- Clearer success criteria (learning + lived experience)

