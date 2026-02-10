# Archived: Sketch Notes Integration

This sketch notes document has been archived. See the archived copy for the original full sketches and detailed UI ideas.

Archived copy: [archive_docs/SKETCH_NOTES_INTEGRATION.md](archive_docs/SKETCH_NOTES_INTEGRATION.md)

For actionable UI/interaction decisions, refer to [GRAPH_INTERFACE.md](GRAPH_INTERFACE.md) and [README.md](README.md).
- OR generally geometric (gridlike layout)
- **Best for**: Organizing known collections, building intentional structures
- **Use case**: Organizing bookmarks, creating knowledge base
- **Addition**: Not in current plan; should be Phase 2 optional toggle

#### **Gas Mode** (Phase 2+)
- Nodes motile, grouped by force ranking (general > direct > historical)
- Moved data does NOT clump, but selected sets can still clump
- **Best for**: Spreading out relationships, seeing all connections
- **Use case**: Analyzing highly connected nodes
- **Addition**: Advanced Phase 2+ feature

**Action**: Add "Physics Presets" settings (Liquid/Solid/Gas) to Phase 1.5 settings dialog. Default to Liquid; others toggle in advanced settings.

---

### 2. **Zoom Thresholds with Content-Based LOD** 🔥 **NEW**

The sketches define hierarchical zoom levels:

```
Zoom In (closest)   → Individual pages (nodes = URLs)
Zoom Out (medium)   → Site domains (group pages by domain)
Zoom Out (far)      → Related domain folders (group domains by category/relation)
```

**Current plan** has:
- Generic node visibility culling
- No content-aware zoom behavior
- No automatic grouping at different zoom levels

**Sketches add**:
- **Automatic aggregation** — As you zoom out, nodes merge into domain groups
- **Zoom-aware rendering** — At far zoom, show "folder" nodes containing domains
- **Intelligent grouping** — Domain folders group by relatedness (DNS zones, category, connection strength)

**Action**: 
- Add to Phase 2 LOD system
- Implement domain grouping algorithm
- Auto-collapse/expand nodes when crossing zoom thresholds
- Store hierarchical node structure (page → domain → folder level)

---

### 3. **Cost/Attention Model for Tab Management** 🔥 **NEW**

The sketches introduce a **soft cost system**:

- Opening/maintaining tabs consumes "attention" (configurable cap based on screen space)
- Closing a tab partially refunds the cost (soft cap, scales down)
- Cost is like "square footage" in an apartment: affects how "cramped" you feel
- Dead tabs (closed) are gone; history view shows them separately
- This creates **intentional friction** around tab proliferation

**Current plan** has:
- No tab cost/limit
- Simple add/remove nodes
- No attention economy model

**Sketches add**:
- Soft cap on active nodes (default: based on screen diagonal or user pref)
- "Cramped" visual feedback when approaching cap
- Cost refund on close (incentivizes deliberate curation)
- Mental model: "Your online home has limited floor space"

**Action**: 
- Defer to Phase 2 (UX research needed on whether this is desired)
- Could be optional "Strict Mode" toggle
- Implementation: Track "node cost" per node; sum checked against `hard_cap` (advisory) and `soft_cap` (visual warning)

---

### 4. **Online Fingerprint / Home Concept** 🔥 **NEW**

The sketches suggest nodes represent your "online home":
- Your graph is your personal web presence fingerprint
- Graph expansion/contraction reflects your attention and interests
- Can be monetized (attention tokens, ads, etc.)
- Extensible across platforms (Firefox, Chromium, Brave, etc.)

**Current plan** has:
- Graph as personal knowledge base
- No explicit "fingerprint" or monetization layer
- Single-app focus (Graphshell only)

**Sketches add**:
- Graph as **identity** (your browsing "home")
- Cross-platform extension potential
- Data monetization/token economy layer

**Action**:
- Recognize this as Phase 3+ narrative/design goal
- Document in GRAPH_BROWSER_MIGRATION.md as long-term vision
- Phase 3 research: How to present user's graph as shareable "fingerprint"?
- No MVP changes needed

---

### 5. **Multiple Organizational Views (Grid, Tree, Organic, Crystal, Cloud)** 🔥 **NEW**

Sketches reference distinct **layout/organizational modes**:

| Mode | Description | Current Plan |
|------|---|---|
| **Grid** | Nodes in cartesian grid (X, Y) | No |
| **Tree** | Hierarchical tree structure | Phase 2 (implicit) |
| **Organic (Molecule)** | Force-directed clusters, organic shapes | ✅ MVP (default) |
| **Crystal** | Repeating/geometric patterns (cubic, etc.) | No (relates to Solid physics) |
| **Cloud** | Loosely grouped, high dispersion | No |

**Current plan** assumes:
- Force-directed only (organic/molecule mode)
- Phase 2 adds 3D (but still force-directed)

**Sketches add**:
- User choice of organizational algorithm
- Each mode has different physics/layout engine
- Switchable per session

**Action**:
- Phase 2 feature: Add layout mode toggle (Organic/Grid/Tree buttons)
- Implement alternative algorithms:
  - **Grid**: Snap nodes to nearest grid cell
  - **Tree**: Hierarchical layout (Sugiyama/layer-by-layer)
  - **Crystal**: Repeating lattice (cubic/hexagonal)
- Can reuse existing physics engine; just constrain node placement

---

### 6. **History View as Branching Structure** 🔥 **NEW**

Sketches propose **View 2 (History)** showing:
- Branching timeline of navigation (mind map-like)
- If on tab X and open A, B, C → they branch from X in the representation
- Node size grows with connection count (weighted by historical reference)
- Delete chains of events
- Snowflake/tree aesthetic

**Current plan** has:
- Graph persistence (save/load)
- Phase 2 imports history
- No branching history view

**Sketches add**:
- Explicit **History View** (separate from Map/Home)
- Tree representation of navigation flow
- Visual editing of history (delete branches)
- Analytics on your own data (see your traversal patterns)

**Action**:
- Phase 2 feature: Add History View mode
- Trigger: Menu → View → History (or button)
- Reconstruct navigation tree from edge timestamps
- Implement tree layout algorithm
- Add delete-branch UI
- Track: Which node did you navigate FROM? (source → target timestamp order)

---

### 7. **Build Mode / Marketplace with Staking** 🔥 **NEW**

Sketches propose **View 3 (Map)** as a "build mode":
- Create/assemble chunks (collections of sites/tabs)
- Stake coins to validate links
- Share chunks as NFTs or marketplace items
- Earn coins for validated contributions
- Crawl pages to find related content

**Current plan** has:
- Phase 3 P2P sync + tokenization research
- No explicit "build mode" or marketplace

**Sketches add**:
- Marketplace paradigm (sell/buy collections)
- Staking as validation mechanism
- Chunk/collection-as-NFT concept
- Curation incentives

**Action**:
- Phase 3+ research: This is the ecosystem monetization layer
- Document in Phase 3 as "Tokenization & Marketplace" subsection
- Key questions:
  - How do we define "chunks" (collections)? 
  - How do we prevent spam/validation attacks?
  - Who stakes coins initially (bootstrap problem)?
  - Can we use reputation instead of coin stake?

---

## Feature Mapping to Current Migration Plan

### Phase 1 (MVP) Additions

✅ **Already in plan:**
- Force-directed graph (Liquid physics)
- WASD controls
- Search
- Multi-window snapping

🔄 **Should add:**
- Physics presets toggle (Liquid / Solid / Gas) — Settings dialog checkbox
  - Start with Liquid only; Solid/Gas as non-blocking Phase 2 prep
  - Impact: Small (just tuning constants + optional constraint system)

### Phase 2 (Personal Features) Additions

✅ **Already in plan:**
- Clustering
- 3D visualization
- DOM extraction
- Export

🔄 **Should add:**
- **Zoom-aware LOD with domain grouping**
  - Auto-collapse pages into domains as you zoom out
  - Implement hierarchical node structure (page/domain/folder levels)
  - Effort: 2–3 weeks (adds complexity to node model)

- **History View (branching tree visualization)**
  - Separate view mode showing navigation timeline
  - Tree layout algorithm (Sugiyama or similar)
  - Delete-chain interaction
  - Effort: 2 weeks (new layout algorithm + view UI)

- **Layout mode toggle (Grid / Tree / Organic)**
  - Radio button in settings or toolbar
  - Grid: Snap to cartesian grid
  - Tree: Hierarchical layout
  - Organic: Current force-directed (default)
  - Effort: 2 weeks (implement 2 new layout algorithms)

### Phase 3 (Network & Ecosystem) Additions

✅ **Already in plan:**
- P2P sync
- Tokenization research
- Shared graphs

🔄 **Should add:**
- **Cost/Attention model** (optional "Strict Mode")
  - Soft cap on active nodes
  - Cost refund on close
  - Visual "cramped" feedback
  - Effort: 1–2 weeks (config + UI feedback)
  - Note: UX research needed; may not be desired by all users

- **Marketplace / Build Mode**
  - Create/share chunks (collections)
  - Staking validation mechanism
  - NFT/token model
  - Effort: Major; likely 4–6 weeks of research + prototyping

- **Graph as Online Fingerprint**
  - Narrative/branding around user's personal web identity
  - How to present graph as shareable profile?
  - Extension to other browsers?
  - Effort: Design + research; 2–3 weeks

---

## Detailed Feature Additions by Phase

### Phase 1 → 1.5: Physics Presets

**In Settings dialog, add:**
```
Physics Mode:
  ☑ Liquid (default)    [Nodes motile, force-based]
  ☐ Solid              [Nodes immobile, user-placed]
  ☐ Gas                [Nodes motile, spread-out clustering]
  
Physics Constants (Liquid mode):
  Spring constant (k_s):    [slider] 0.1 ... 1.0
  Repulsion constant (k_r): [slider] 0.5 ... 2.0
  Damping:                  [slider] 0.1 ... 0.9
```

**Implementation**:
- Store selected mode in config
- In physics update loop:
  - **Liquid**: Apply spring + repulsion (current)
  - **Solid**: Skip position updates; snap to grid or user-placed locations
  - **Gas**: Apply repulsion > general > historical force ranking (reverse of liquid)

---

### Phase 2: Zoom-Based Domain Grouping

**Goal**: As user zooms out, auto-aggregate pages into domain nodes; domain nodes into folder nodes.

**Data structure**:
```rust
pub enum NodeLevel {
    Page(URL),      // Finest detail: individual URL
    Domain(Domain), // Medium: group of pages
    Folder(Category), // Coarse: group of domains
}

pub struct Node {
    id: NodeId,
    level: NodeLevel,
    children: Vec<NodeId>, // Child nodes at finer level
    parent: Option<NodeId>, // Parent node at coarser level
    ...
}
```

**Zoom thresholds**:
- Zoom > 1.5x: Show pages + domains
- 0.5x < Zoom < 1.5x: Show pages + domains
- Zoom < 0.5x: Show domains + folders

**Auto-grouping algorithm**:
- Cluster pages by domain (same origin)
- Cluster domains by relatedness (shared category, shared edges, DNS proximity)

**Rendering**: 
- Draw aggregated node at intermediate zoom
- Expand to show children when zoom threshold crossed

---

### Phase 2: History View

**New view mode**: History (accessible from View menu or button)

**Representation**:
- Reconstruct navigation flow from edge timestamps
- Build tree: each node → edges in chronological order
- Branching: if A → B, A → C, A → D all in session, show B, C, D as children of A

**Tree layout algorithm**:
- Use Sugiyama algorithm or hierarchical layout
- Place root at top; children fan out below
- Vertical position = chronological order
- Horizontal position = graph x-coordinate (preserve spatial sense)

**Interactions**:
- Click edge → Delete that navigation step
- Click node → Select all reachable nodes from that point
- Toggle: Show only certain time ranges (date picker)

---

### Phase 2: Layout Modes

**Settings or toolbar:**
```
Layout Mode:
  ◉ Organic (Force-directed) - Default, natural clustering
  ◯ Grid - Cartesian alignment
  ◯ Tree - Hierarchical layout
```

**Organic** (current):
- Force-directed, Verlet integration

**Grid**:
- Snap each node to nearest grid cell
- Maintain edge connections visually (curved lines, not straight)

**Tree**:
- Requires defining parent-child relationships (can use first/strongest edge as parent)
- Hierarchical layout: parents above children, columns aligned

---

## Integration Points with Current Plan

### 1. Settings Dialog (Phase 1.5)
Add sections:
- **Physics**: Mode (Liquid/Solid/Gas), constants
- **Appearance**: Node size, edge style, node shapes
- **Interaction**: Cost model (optional), attention cap

### 2. View Menu (Phase 2)
Add options:
- **Map** (current graph view)
- **History** (branching tree view)
- **Layout**: Organic / Grid / Tree (submenu)

### 3. Detail Window (MVP+)
Tabs in detail window could also respect layout mode:
- **Organic**: Tab order = chronological (current)
- **Tree**: Tab order = tree hierarchy (if defined)
- **Grid**: Tab order = spatial (left-to-right)

---

## Questions & Design Decisions

### 1. Should Physics Presets Be in MVP or Phase 2?
**Recommendation**: Phase 2 non-blocking
- MVP: Liquid physics only; solid foundation
- Phase 2: Add toggle for Solid/Gas as "advanced" features
- Reason: Keeps MVP focused; modes are useful but not essential

### 2. Cost/Attention Model: Feature or Anti-Pattern?
**Sketches suggest** it creates intentional friction; prevents "tab sprawl"
**Questions**:
- Do users want to be constrained by soft cap?
- Is this a feature or a limitation?
- Could be optional "Strict Mode" (on/off toggle)
**Recommendation**: Defer to Phase 3; user research needed

### 3. History View: Separate Mode or Overlay?
**Sketches suggest** separate view (View 2)
**Options**:
- A) Separate mode (current + map + history)
- B) Overlay on graph (show history edges/paths differently)
- C) Sidebar panel (third option)
**Recommendation**: (A) Separate view mode; cleaner UX

### 4. Layout Modes: How to Define Tree Parent-Child?
**Sketches don't specify**
**Options**:
- First/oldest edge from node = parent
- Strongest edge = parent
- User-defined parent (right-click menu)
**Recommendation**: User-defined (most flexible); first edge as fallback

### 5. Domain Grouping: Automatic or Manual?
**Sketches suggest** automatic (grouped by domain at medium zoom)
**Trade-offs**:
- Auto: Less user work; can be surprising
- Manual: More control; more UI complexity
**Recommendation**: Auto grouping + manual override (right-click: "don't group this domain")

---

## Phase-by-Phase Summary

### Phase 1: MVP (Weeks 1–8) ✅
- Force-directed graph (Liquid physics)
- WASD controls
- Multi-window snapping
- Detail window with connection tabs
- Search omnibar
- *(No changes from current plan)*

### Phase 1.5: Settings / Polish (Weeks 8–9) 🔄
- **Add**: Physics presets (Liquid/Solid/Gas) toggle in settings
- **Add**: Node size, edge style, appearance settings

### Phase 2: Personal Features (Weeks 9–16) 🔄
- Clustering (current)
- Filtering (current)
- Persistence (current)
- **Add**: Zoom-aware domain grouping (LOD)
- **Add**: History View (branching tree visualization)
- **Add**: Layout modes (Grid / Tree / Organic)
- **Add**: Export formats (current)
- **Add**: 3D visualization (current)
- **Add**: DOM extraction (current)

### Phase 3: Network & Ecosystem (Weeks 17–24) 🔄
- P2P sync (current)
- Tokenization research (current)
- **Add**: Cost/Attention model (optional "Strict Mode")
- **Add**: Marketplace / Build mode with staking
- **Add**: Graph as "online fingerprint" narrative
- **Add**: Cross-platform extension research (Firefox, Chromium, Brave)

---

## Summary

The sketch notes provide **sophisticated UX/design patterns** that strengthen the current migration plan:

1. **Physics presets** add customization (low effort, Phase 1.5)
2. **Zoom-based LOD with domain grouping** makes exploration more intuitive (medium effort, Phase 2)
3. **History View** provides an alternate perspective on your browsing (medium effort, Phase 2)
4. **Layout modes** empower users to organize their web differently (medium effort, Phase 2)
5. **Cost/Attention model** adds intentional friction (research needed; Phase 3+)
6. **Marketplace/staking** brings the ecosystem vision into focus (Phase 3+)
7. **Online fingerprint concept** provides compelling narrative (Phase 3 branding)

**All additions are complementary; none require rework of MVP.** The core force-directed graph architecture holds up well against these enhancements.

