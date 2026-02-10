# Critical Analysis: Technical Problems, Gaps & Missing Pieces

**Date**: February 4, 2026  
**Scope**: Architecture validation, feature drift analysis, P2P collaboration planning

---

## EXECUTIVE SUMMARY

The current design is sound for MVP (Weeks 1-8), but has **critical gaps** for production and **deferred decisions** that will block Phase 2 if not addressed now. Key issues:

1. **P2P Collaboration**: Currently deferred to Phase 3, but architecture doesn't account for conflict resolution, version control, or merge strategies
2. **DOM Serialization**: Deliberately avoided, but P2P sync requires it (contradiction)
3. **Webview Lifecycle**: Origin-based approach untested at scale; no process memory pooling strategy
4. **Node Metadata Versioning**: No multi-version state tracking needed for sync
5. **Feature Drift**: Session management, ghost nodes, sidebar dropped from Phase 2 despite vision
6. **Firefox Integration Pattern**: Servo's `-M` flag copied, but Firefox's crash isolation strategy not fully integrated
7. **Browser Functionality**: Detail view lacks critical browser features (history, forward/back)
8. **Export/Import**: Phase 2 placeholder, but essential for data portability and collaboration setup
9. **Untrusted Data**: Sanitization adequate for single-user, insufficient for P2P verification
10. **Testing Gap**: No tests for distributed state merge, CRDTs, or conflict scenarios

---

## Merged Critical Review Summary (From CRITICAL_REVIEW_SUMMARY.md)

### The Bottom Line

Your current design is **85% correct for MVP**, but has **critical gaps for Phase 2/3** that need design work now:

#### What's Good ✅
- Servo foundation (multiprocess, sandboxing, rendering)
- Servoshell integration strategy
- Graph model (SlotMap, adjacency list)
- Physics engine approach (force-directed, auto-pause)
- MVP scope (Weeks 1-8)
- Keybind system (36 actions)

#### What's Missing 🔴
- **Command trait** (needed for P2P, undo, crash recovery)
- **Version vectors** (needed for P2P merge detection)
- **Operation log** (needed for durability, sync)
- **Sync protocol spec** (needed for Phase 3)
- **Browser feature spec** (detail view behavior unspecified)
- **Feature drift** (sessions, ghost nodes, sidebar dropped)

#### What's Untested ⚠️
- **Webview lifecycle at scale** (100+ nodes, 20+ origins)
- **Process crash handling** (automatic respawn?)
- **Memory pressure** (what happens at 2GB RAM limit?)

---

### Quick Actions Before Week 1

1. Design command trait (2 hours)
2. Add version vectors (1 hour)
3. Process monitoring stubs (2 hours)
4. SERVO_INTEGRATION_SPEC.md (3 hours)
5. EXPORT_FORMAT_SPEC.md (2 hours)

**Total time**: 10 hours (design-only)

---

### Reading Order

**Before Week 1**:
- Read [CRITICAL_ANALYSIS.md](CRITICAL_ANALYSIS.md) sections 1-6
- Read [GRAPHSHELL_AS_BROWSER.md](GRAPHSHELL_AS_BROWSER.md)
- Do the 10-hour prep work

**Week 1**:
- Read [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md) sections 1-12
- Read [FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md](FIREFOX_SERVO_GRAPHSHELL_ARCHITECTURE.md) Parts 3-5
- Skim [verse_docs/GRAPHSHELL_P2P_COLLABORATION.md](verse_docs/GRAPHSHELL_P2P_COLLABORATION.md)

---

## 1. CRITICAL ISSUE: P2P Collaboration Architecture Missing

### The Problem

**Decision**: P2P sync deferred to Phase 3+.  
**Reality**: Architecture doesn't support it.  
**Impact**: Phase 3 will require complete redesign of state management.

### Why It Matters Now

Current design assumes single-user local graph:
- **No version vectors** (needed to track causality in P2P)
- **No operation log** (needed to replay/merge changes)
- **No node versioning** (needed to detect concurrent edits)
- **Direct mutation pattern** (can't track why state changed)

Example scenario (Week 9, validation succeeds, Phase 2 launches):
```
User A: Deletes node X at time T1
User B: Adds edge to X at time T1.1 (A's deletion not yet synced)
→ Sync arrives
→ What happens to B's edge? Deleted? Conflict?
```

### Firefox's Approach (Relevant Pattern)

Firefox's multiprocess architecture is **read-heavy, write-serialized**:
- Content processes (webviews) are **read-only** for tab state
- Parent process (compositor) is **single writer** for all state
- IPC messages are **ordered, ACKed** (guarantees consistency)

**Key insight**: Firefox doesn't do P2P; it's centralized by design. But the ordering guarantees are relevant.

### Recommended Architecture (Phase 1, Design Only)

Defer implementation, but design now:

```rust
// Graph operations must be commandifiable
pub trait Command: Serialize + Deserialize {
    fn execute(&self, graph: &mut Graph) -> Result<()>;
    fn inverse(&self) -> Box<dyn Command>;
    fn timestamp(&self) -> u64;
    fn peer_id(&self) -> u64;
}

pub struct CreateNodeCommand {
    node_key: NodeKey,
    url: String,
    position: Point2D<f32>,
    timestamp: u64,
    peer_id: u64,
}

impl Command for CreateNodeCommand {
    fn execute(&self, graph: &mut Graph) -> Result<()> {
        // Create node, or skip if already exists
    }
    fn inverse(&self) -> Box<dyn Command> {
        Box::new(DeleteNodeCommand { key: self.node_key })
    }
}

// Operation log (durable, used for sync)
pub struct OperationLog {
    operations: Vec<Box<dyn Command>>,  // Ordered by timestamp
    last_synced: u64,
}

impl OperationLog {
    pub fn append(&mut self, cmd: Box<dyn Command>) {
        self.operations.push(cmd);
        self.save_to_disk();  // Durable
    }
    
    pub fn unsync_since(&self, peer_id: u64, last_ack: u64) -> Vec<Box<dyn Command>> {
        self.operations.iter()
            .filter(|op| op.timestamp() > last_ack)
            .cloned()
            .collect()
    }
}
```

**Benefits**:
- MVP doesn't implement P2P, but structure supports it
- Every user action is a `Command` (auditable, reversible, syncable)
- Phase 3 P2P adds merge/conflict resolution, not architecture rewrite

### Approach for Collaboration (When Implementing Phase 3)

**Pattern to research** (not implement now): **CRDT-based approach**

Different collaborative tools use different strategies:

| Tool | Strategy | Why |
|------|----------|-----|
| Google Docs | Operational Transform (OT) | Text-centric, needs character-level merging |
| Notion | Central server + local cache | Proprietary sync, not P2P |
| Obsidian Sync | Merkle trees + delta sync | File-centric (markdown), central coordinator |
| Anytype | CRDTs (Automerge-like) | Local-first, P2P, no server needed |
| Office 365 | CRDT-inspired + versioning | Multi-layer sync, conflict ribbon |
| Yjs | CRDT library | General-purpose, supports P2P |

**For Graphshell (graph-centric)**:

Hybrid approach likely best:
1. **Command-based log** (local): Every graph mutation is a Command
2. **Vector clock** per peer (causal consistency): Track "happened before" relationships
3. **CRDT for node positions** (physics state): Concurrent moves merge naturally
4. **Conflict resolution for topology** (edges/deletions): User intervention if simultaneous delete/edit

Example:
```rust
pub struct NodeWithVersionInfo {
    node: Node,
    version_vector: VersionVector,  // [Peer A: 15, Peer B: 23, ...]
    created_by: u64,
    last_modified_by: u64,
    last_modified_at: u64,
}

pub struct VersionVector(HashMap<u64, u64>);

impl VersionVector {
    pub fn happened_before(&self, other: &Self) -> bool {
        // Self's all version numbers <= other's
    }
    
    pub fn concurrent(&self, other: &Self) -> bool {
        // Neither happened before the other
    }
}
```

### Recommendation

**Phase 1 (Weeks 1-8)**:
- [ ] Design `Command` trait, don't implement P2P yet
- [ ] Add `OperationLog` infrastructure (durable, for crash recovery)
- [ ] Add `timestamp` and `peer_id` fields to all graph mutations
- [ ] Add version vectors to Node/Edge (for future merge detection)

**Phase 3 (when implementing P2P)**:
- Implement merge strategies for concurrent operations
- Use Automerge or Yjs CRDT library if complexity demands it
- Test with 2-3 peers simultaneously editing same graph

---

## 2. CRITICAL ISSUE: DOM Serialization Needed for P2P

### The Problem

**Decision**: "Don't serialize DOM; Servo handles it."  
**Reality for P2P**: Peer B needs to know what Peer A rendered, to merge node state.

### Why It Matters

Current design assumes webview state stays in Servo process:
- Node metadata: title, favicon, description → stored in graph
- DOM state: user's bookmarks on page, notes in input field → **NOT captured**

Scenario:
```
User A on https://example.com:
  - Highlights text: "Important concept"
  - Saves to Graphshell note field
  - Close node

User B opens same node on different peer:
  - Sees cached title/favicon (synced)
  - DOM is fresh (no state preserved)
  - User A's highlight/note lost
```

### Why Full Serialization Is Expensive

Current decision avoids this because:
- Serializing DOM tree = 100-500KB per page (uncompressed)
- Storage bloat: 1000 nodes × 200KB = 200MB
- Sync overhead: 200MB per peer sync

### Recommendation: Selective Serialization

Instead of full DOM, serialize only **user annotations**:

```rust
pub struct NodeAnnotations {
    user_notes: String,                      // User's notes field
    highlights: Vec<(Range, String)>,        // Text + why highlighted
    bookmarks: Vec<ElementSelector>,         // User's bookmarks on page
    form_state: HashMap<String, String>,     // Form field values
    scroll_position: (u32, u32),             // Where user scrolled to
}

impl NodeAnnotations {
    pub fn serialize(&self) -> Vec<u8> {
        // Compact binary format, ~1-10KB per node
        serde_json::to_vec(self).unwrap()
    }
    
    pub fn deserialize(data: &[u8]) -> Self {
        serde_json::from_slice(data).unwrap()
    }
}
```

**Size**: ~5KB per node (vs 200KB+ for full DOM)  
**Storage**: 1000 nodes × 5KB = 5MB (acceptable)  
**Sync**: Selective serialization in each mutation

**Phase 2 implementation**:
- [ ] Add `NodeAnnotations` struct to `Node`
- [ ] Capture user input in detail view (notes, highlights)
- [ ] Serialize on node close or auto-save
- [ ] Deserialize on node open, replay highlights/notes

---

## 3. CRITICAL ISSUE: Webview Lifecycle Untested at Scale

### The Problem

**Decision**: "Use Servo's origin-grouped process management directly."  
**Assumption**: Processes scale linearly (1 origin = 1 process, no overhead).  
**Reality**: Unknown until tested.

### Questions Not Answered in Design

1. **Memory per process**: How much RAM per origin? (Depends on page)
2. **Process creation latency**: How long to spawn new process? (Expected: 200-500ms)
3. **Shared memory**: Do multiple nodes from same origin share process memory? (Yes, designed this way)
4. **Process death**: When exactly does process kill happen? (When all nodes for origin close + grace period?)
5. **Grace period**: Is there one? How long? (Not specified)
6. **Cascade**: If 50 nodes from 50 origins open quickly, what happens? (CPU spike? OOM?)
7. **Swap**: Does system start using swap disk? (Degrades performance to single-digit fps)

### Firefox's Approach (Lessons Learned)

Firefox uses **process pooling** for this reason:
- Pre-spawns 4-8 content processes on startup
- Assigns origins to processes round-robin
- When demand exceeds pool, spawns temporary processes
- Reuses processes when possible (faster than re-spawn)

**Key insight**: Servo's origin-grouped model is elegant, but untested at browser scale.

### Recommendation: Add Safeguards

**Phase 1 (Design)**:
- [ ] Add memory pressure monitoring (like Decision #8, but detailed)
- [ ] Track process count vs system RAM
- [ ] Add telemetry: spawn time, memory per origin, swap usage

**Week 6 (User testing)**:
- [ ] Test with 100-500 nodes, 20-50 origins
- [ ] Measure process spawn latency
- [ ] Check swap usage under load
- [ ] Validate memory predictions

**If problems found**:
- Option A: Implement process pooling (pre-spawn + reuse)
- Option B: Cap open processes (kill LRU origin's process when > limit)
- Option C: Lazy load (don't materialize webview until user opens detail view)

**Fallback strategy (Phase 2)**:
```rust
pub struct ProcessPool {
    max_processes: usize,           // e.g., 32
    active: HashMap<Origin, ProcessHandle>,
    lru_queue: VecDeque<Origin>,    // For eviction
}

impl ProcessPool {
    pub fn get_or_spawn(&mut self, origin: &Origin) -> ProcessHandle {
        if let Some(handle) = self.active.get(origin) {
            self.lru_queue.move_to_back(origin);
            return handle.clone();
        }
        
        if self.active.len() >= self.max_processes {
            let victim = self.lru_queue.pop_front().unwrap();
            self.kill_process(&victim);
        }
        
        let handle = spawn_servo_process(origin);
        self.active.insert(origin.clone(), handle.clone());
        self.lru_queue.push_back(origin.clone());
        handle
    }
}
```

---

## 4. CRITICAL ISSUE: Node Metadata Versioning Missing

### The Problem

**Current design**: Node stores single version of metadata (title, favicon, description, notes).

**For P2P**: Need to track who changed what, when, and handle concurrent edits.

### Example Conflict Scenario

```
Peer A: Updates node title from "React Docs" to "React 18 Docs" at T1
Peer B: Updates same node title to "React API Reference" at T1 (concurrent)
→ Sync arrives
→ Which title wins? Last-write-wins? Conflict resolution UI?
```

### Recommendation: Version Vector Per Field

```rust
pub struct VersionedField<T> {
    value: T,
    version_vector: VersionVector,  // [Peer A: 5, Peer B: 3, ...]
    last_modified_by: PeerId,
    last_modified_at: Timestamp,
}

pub struct Node {
    id: NodeKey,
    url: VersionedField<Url>,
    title: VersionedField<String>,
    notes: VersionedField<String>,
    position: VersionedField<Point2D<f32>>,
    // ... other fields similarly versioned
}

// On merge from remote:
pub fn merge_node(&mut self, remote: &Node) {
    if remote.title.version_vector.happened_after(&self.title.version_vector) {
        self.title = remote.title.clone();  // Remote is newer
    } else if self.title.version_vector.happened_after(&remote.title.version_vector) {
        // Local is newer, keep it
    } else {
        // Concurrent edit: conflict
        // Option 1: Show conflict UI (user picks version)
        // Option 2: Last-write-wins
        // Option 3: Merge text (if collaborative editing)
    }
}
```

**Phase 1 implementation**: Not needed yet, but structure allows it.

---

## 5. FEATURE DRIFT: Critical Phase 2 Features Dropped

### Missing from Current Plan

#### Session Management (Dropped)

**Original vision**: "Save, delete, or share historical sessions"  
**Current plan**: Save graph state (monolithic), no sessions concept

**Problem**: User has 200-node graph. Wants to explore a tangent (add 50 new nodes), then revert.
- Current: Save/revert entire graph (tedious, all-or-nothing)
- With sessions: Each browse is a session; can branch/merge

**Recommendation**:
```rust
pub struct Session {
    id: SessionId,
    graph: Graph,
    name: String,
    created_at: Timestamp,
    last_modified_at: Timestamp,
    parent_session_id: Option<SessionId>,  // For branching
}

pub struct SessionLibrary {
    sessions: HashMap<SessionId, Session>,
    current: SessionId,
}

impl SessionLibrary {
    pub fn branch_current(&mut self, name: String) -> SessionId {
        let new_session = Session {
            graph: self.current.graph.clone(),
            parent_session_id: Some(self.current.id),
            ..
        };
        self.sessions.insert(new_session.id, new_session);
        new_session.id
    }
}
```

**Phase 2 task**: [ ] Implement session branching/merging

#### Ghost Nodes (Dropped)

**Original vision**: "Use ghost nodes to preserve structure when removing items"  
**Current plan**: Delete removes node entirely

**Problem**: User deletes node X, which has edges to 10 other nodes. Graph structure breaks (now 10 orphaned edges).

**Solution**: Ghost node—visual placeholder that preserves edges but marks deleted.

```rust
pub enum NodeState {
    Alive,
    Ghost,  // Deleted, but structure preserved
}

pub struct Node {
    state: NodeState,
    // ... other fields
}

// Rendering: Ghost nodes appear faded/dashed
fn render_node(node: &Node) {
    match node.state {
        NodeState::Alive => {
            // Normal circle, solid edges
        }
        NodeState::Ghost => {
            // Dashed circle, dashed edges
        }
    }
}
```

**Phase 2 task**: [ ] Add ghost node state, rendering, and merge logic

#### Sidebar (Dropped)

**Original vision**: "Optional sidebar window manager for people who still want tabs"  
**Current plan**: Detail view only, no sidebar alternative

**Problem**: Users coming from Firefox might prefer traditional tab list.

**Solution**: Optional sidebar toggle showing all open nodes.

```rust
pub enum ViewLayout {
    GraphOnly,
    SplitView { split_ratio: f32 },  // Current: 60/40 default
    SidebarLayout,                    // Future: sidebar on left, graph on right
}
```

**Phase 2 task**: [ ] Add sidebar layout option

### Why These Matter for P2P

**Sessions**: P2P sync must handle branched graphs merging back.  
**Ghost nodes**: P2P deletion must be reversible (ghost nodes track intent).  
**Sidebar**: Alternative UX helps non-technical users collaborate.

---

## 6. FIREFOX ARCHITECTURE: Crash Isolation & Multi-Origin Management

### What Firefox Does (And Servo Copies)

Firefox's multi-process architecture:

```
Main Process (Compositor)
  ├─ Content Process 1 (origin A)
  ├─ Content Process 2 (origin B)
  ├─ GPU Process
  └─ Network Process

Rules:
- Tabs are origin-grouped (one origin per process, ~4 tabs per process)
- Process crash isolates one origin
- Compositor never crashes from bad webview
- IPC is serialized, ordered
```

Servo copies this via `-M` flag. **Good**. But missing one piece:

### What Graphshell Is Missing: Process Crash Recovery

**Scenario**:
```
User has 3 nodes open:
- https://example.com/1
- https://example.com/2
- https://evil.com/pwn

Webview from evil.com crashes (RWA bug)
→ Servo kills process for evil.com origin
→ All nodes from evil.com become unresponsive
```

**Current handling**: User manually closes evil.com node? (Untested)

**Recommendation**: Add crash recovery handler

```rust
pub struct WebviewHandle {
    origin: Origin,
    process_id: ProcessId,
    is_alive: Arc<AtomicBool>,
}

impl WebviewHandle {
    pub fn on_process_crash(&mut self) {
        self.is_alive.store(false, Ordering::SeqCst);
        
        // Notify UI
        tx.send(Event::ProcessCrashed(self.origin.clone()));
        
        // Mark all nodes from this origin as "needs respawn"
        for node in graph.nodes_for_origin(&self.origin) {
            node.state = NodeState::Dead;  // New state
            node.retry_at = now() + Duration::secs(2);
        }
        
        // Auto-respawn on next interaction
    }
    
    pub fn respawn(&mut self) {
        self.process_id = spawn_servo_process(&self.origin);
        self.is_alive.store(true, Ordering::SeqCst);
        
        // Reload all dead nodes from origin
        self.reload_all_dead_nodes();
    }
}
```

**Phase 2 task**: [ ] Add process crash monitoring + auto-respawn

---

## 7. CRITICAL ISSUE: Detail View Lacks Browser Features

### Missing Features

Current "Detail View" is just a Servo window showing the page. Missing:

1. **History** (Back/Forward buttons)
   - Current: Clicking edge goes back (indirect)
   - Expected: Ctrl+[ / Ctrl+] or Back/Forward buttons

2. **Bookmarks** (save page as bookmark)
   - Current: Manual edge creation
   - Expected: Ctrl+B to save bookmark, bookmark icon in UI

3. **Address Bar** (navigate to arbitrary URL)
   - Current: Only access via nodes
   - Expected: Omnibar search + ability to paste URL

4. **Tab Management** (in detail view)
   - Current: Pinned tabs show connected nodes
   - Missing: Tab groups, muting, reload

### Recommendation

**Phase 1** (MVP): None of these needed (graph navigation sufficient)

**Phase 2** (Browser features):
- [ ] Implement proper browser chrome in detail view
  - Back/Forward buttons (leverage Servo's history)
  - Bookmarks button (populate from bookmark edge type)
  - Reload button (Servo API exists)
  - Address bar (but maybe optional; graph-based nav is primary)

---

## 8. EXPORT/IMPORT INFRASTRUCTURE: Essential for Collaboration

### Current Status

**Phase 2 placeholder**: "Export JSON, PNG, SVG"

**Problem**: No concrete spec for export format; essential for:
- Sharing graphs via email/file
- Interop with other tools (Obsidian, Notion, etc.)
- P2P sync (need portable format)

### Recommendation: Design Export Format Now

**Phase 1 (design, no implementation)**:

```rust
pub struct ExportFormat {
    version: String,           // "1.0"
    created_at: Timestamp,
    app_version: String,       // e.g., "graphshell-0.1.0"
    nodes: Vec<ExportNode>,
    edges: Vec<ExportEdge>,
}

pub struct ExportNode {
    id: String,               // UUID or stable hash
    url: String,
    title: String,
    favicon: Option<Base64>,  // PNG data
    notes: String,
    tags: Vec<String>,
    position: (f32, f32),     // Absolute, not physics-dependent
    created_at: Timestamp,
}

pub struct ExportEdge {
    from_id: String,
    to_id: String,
    edge_type: String,        // "hyperlink", "bookmark", etc.
    weight: f32,
}
```

**Format choices**:
- **JSON**: Human-readable, widely compatible, ~50KB per 100 nodes
- **JSON + gzip**: Compressed, smaller for email
- **YAML**: More human-friendly than JSON
- **CBOR**: Binary, compact (~30% smaller than JSON), less compatible

**Recommendation**: JSON primary, offer JSON+gzip + YAML export options

**Phase 2 implementation**:
- [ ] Export to JSON with full metadata
- [ ] Import from JSON (reconstruct graph)
- [ ] Export to HTML (interactive, sharable)

### Import Compatibility

**Bookmarks.html** (Firefox export):
```html
<DT><A HREF="https://example.com" ADD_DATE="..." TAGS="...">Title</A>
<DT><H3>Folder Name</H3>
```

**Phase 2 importer**:
- [ ] Read bookmarks.html
- [ ] Create node for each bookmark
- [ ] Create folder edge for hierarchy

---

## 9. UNTRUSTED DATA: Insufficient for Collaboration

### Current Strategy

**Decision**: "Sanitize user-visible data. Validate URLs. Trust Servo for webview sandboxing."

**Code**:
```rust
fn sanitize_label(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.".contains(*c))
        .collect()
}
```

**Problem for P2P**: Peer B receives sanitized title from Peer A, but can't verify authenticity.

### Additional Concerns

1. **No signature/hash verification**: If graph is synced over untrusted channel (email, P2P), how do you know it wasn't tampered?
2. **URL injection**: Peer A crafts malicious `javascript://` URL, sends to Peer B
3. **Metadata injection**: Peer A embeds exploit in "notes" field
4. **Edge misuse**: Peer A creates "false" edges to manipulate ranking/clustering

### Recommendation

**Phase 1**: Keep current approach (single user, local only).

**Phase 3 (P2P)**:
- [ ] Add signature verification (sign mutations with peer's key)
- [ ] Add URL scheme whitelist (http, https, file, feed; reject javascript://)
- [ ] Add content-hash of edges (detect tampering)
- [ ] Add provenance tracking (who created each node/edge?)

Example:
```rust
pub struct SignedNode {
    node: Node,
    signature: Vec<u8>,        // Sign(peer_key, node)
    peer_public_key: Vec<u8>,
}

impl SignedNode {
    pub fn verify(&self, trusted_keys: &[Vec<u8>]) -> bool {
        let is_trusted = trusted_keys.contains(&self.peer_public_key);
        let is_valid = verify_signature(&self.signature, &self.node, &self.peer_public_key);
        is_trusted && is_valid
    }
}
```

---

## 10. TESTING GAP: No Distributed State Testing

### Missing Tests

**Current**: Performance, property-based physics, visual regression.

**Missing**: Distributed scenarios (P2P, merging, conflicts).

### Recommendation

**Phase 1**: Design test structure (don't implement):

```rust
#[cfg(test)]
mod distributed_tests {
    use super::*;
    
    /// Simulate two peers editing same graph concurrently
    #[test]
    fn test_concurrent_node_creation() {
        let mut graph_a = Graph::new();
        let mut graph_b = graph_a.clone();
        
        // Peer A: Create node X
        let cmd_a = CreateNodeCommand { node_key: ..., timestamp: T1, peer_id: A };
        graph_a.execute(&cmd_a).unwrap();
        
        // Peer B: Create node Y (concurrent)
        let cmd_b = CreateNodeCommand { node_key: ..., timestamp: T1, peer_id: B };
        graph_b.execute(&cmd_b).unwrap();
        
        // Sync A's ops to B
        graph_b.execute(&cmd_a).unwrap();
        
        // Sync B's ops to A
        graph_a.execute(&cmd_b).unwrap();
        
        // Both should have both nodes
        assert_eq!(graph_a.nodes.len(), 2);
        assert_eq!(graph_b.nodes.len(), 2);
        assert_eq!(graph_a.nodes[0].id, graph_b.nodes[0].id);
    }
    
    /// Simulate concurrent edge edit and node deletion (conflict)
    #[test]
    fn test_concurrent_delete_and_edit_conflict() {
        // Peer A: Delete node X at T1
        // Peer B: Add edge to X at T1.1
        // → When merged, should have conflict marker or auto-resolve
    }
    
    /// Test CRDT merge of physics state (positions)
    #[test]
    fn test_position_crdt_merge() {
        // Two peers have different physics states for same node
        // Merge should produce stable layout (likely average or CRDT resolution)
    }
}
```

**Phase 2/3**: Implement these tests as P2P feature approaches.

---

## 11. SERVO CROSS-DOMAIN INTEGRATION: Gaps in Documentation

### How Graphshell Should Operate as a Web Browser

**Current understanding**:
- Servo renders webview via `-M -S` (multiprocess, sandbox)
- Graphshell creates nodes from URLs
- Detail view shows Servo's webview

**Missing details**:

1. **How does user navigate?**
   - User clicks link in webview → what happens?
   - Current plan: Servo opens new tab in process
   - For Graphshell: Create new node? Or reuse existing if URL matches?

2. **How is history tracked?**
   - Each node gets a history stack (back/forward per node)?
   - Or global history across all nodes?
   - How does it sync with graph edges?

3. **How do downloads work?**
   - Where does Servo download files?
   - How does Graphshell manage download state?
   - Currently: Download manager (Phase 2), but mechanism unclear

4. **How do JavaScript/DOM interactions map to graph?**
   - User fills form in webview, submits → new page loads
   - Current: New node? Or update existing node's content?
   - Spec needed.

### Recommendation

**Phase 1 (design document)**:
- [ ] Create SERVO_INTEGRATION.md specifying:
  - Link clicking behavior (create node? reuse?)
  - History stack per node vs global
  - Download flow
  - Form submission handling

**Example spec**:
```markdown
## Link Clicking

When user clicks link in detail view webview:
1. Link URL is extracted
2. If node with URL already exists → Switch to that node (reuse)
3. Else → Create new node at URL (position near current node)
4. Update history: current node → new node (manual edge)

## History

Per-node history stack (like browser):
- Node A visits: page1 → page2 → page3
- Back button goes A to page2
- Separate from graph edges (edges are intentional connections)
```

---

## 12. P2P COLLABORATION: Approaches from Existing Apps

### Sync Patterns Observed

| App | Approach | Strengths | Weaknesses |
|-----|----------|-----------|-----------|
| **Google Docs** | Operational Transform (OT) | Real-time, character-level | Server-required |
| **Obsidian** | Merkle trees + central sync | Simple, works well for markdown | Sync still centralized |
| **Notion** | CRDT-inspired (Automerge clone) | Works offline, P2P capable | Proprietary |
| **Anytype** | Full P2P CRDT (based on Automerge) | True P2P, no server needed | Experimental, complexity |
| **Office 365** | Differential sync + version vectors | Handles big docs | Microsoft-specific |
| **Yjs** | CRDT library | Works with any data structure | Adds dependency, learning curve |

### For Graphshell (Graph-Centric)

**Recommended approach: Hybrid CRDT + operational log**

Not pure CRDT (Automerge), not pure OT (Google Docs), but combination:

```rust
// 1. Command log (like OT, but for graph operations)
pub struct CommandLog {
    commands: Vec<GraphCommand>,  // Ordered
    last_synced: u64,
}

impl CommandLog {
    pub fn append(&mut self, cmd: GraphCommand) {
        self.commands.push(cmd);
        self.persist_to_disk();
    }
}

// 2. Version vectors per node (like CRDT, for causality)
pub struct VersionVector {
    clock: HashMap<PeerId, u64>,
}

impl VersionVector {
    pub fn increment(&mut self, peer_id: PeerId) {
        *self.clock.entry(peer_id).or_insert(0) += 1;
    }
    
    pub fn happened_before(&self, other: &Self) -> bool {
        // All clocks less than or equal, at least one strictly less
    }
}

// 3. Merge strategy for topology (edges, deletions)
pub enum MergeStrategy {
    LastWriteWins,      // Simple, deterministic
    VectorClockWins,    // Stronger: causal consistency
    UserIntervention,   // Show conflict UI, user picks
}
```

**Why this hybrid approach**:
- Command log provides auditability + crash recovery (like Notion)
- Version vectors prevent conflicts for independent changes (like CRDT)
- User intervention for true conflicts (edge deletions)
- Simpler than full CRDT, more P2P-capable than server-based sync

### Security Considerations for P2P

**Trust model** (choose one):

1. **Full trust**: All peers are friends, encrypt channel with shared key
2. **Zero trust**: Peers may be adversarial, sign every mutation
3. **Web-of-trust**: Peer A trusts B, B trusts C → transitively trust C

**For personal/research use**: Full trust (shared key per group)  
**For public collaboration**: Zero trust (sign all mutations, verify provenance)

Example (full trust):
```rust
pub struct P2PSession {
    group_id: String,           // e.g., "research-2026"
    shared_key: Vec<u8>,        // Pre-shared, 32 bytes
    peers: HashMap<PeerId, PeerAddress>,
}

impl P2PSession {
    pub fn encrypt_message(&self, msg: &[u8]) -> Vec<u8> {
        // AES-256-GCM with shared key
        encrypt_aes_gcm(msg, &self.shared_key)
    }
}
```

Example (zero trust):
```rust
pub struct SignedCommand {
    command: GraphCommand,
    signature: Vec<u8>,
    peer_public_key: Vec<u8>,
}

impl SignedCommand {
    pub fn verify(&self) -> bool {
        verify_signature(&self.signature, &self.command, &self.peer_public_key)
    }
}
```

---

## RECOMMENDATIONS SUMMARY

### Phase 1 (Weeks 1-8: MVP)
✅ Keep as planned, add these design-only tasks:
- [ ] Command trait + operation log (for crash recovery + future P2P)
- [ ] Version vector fields in Node/Edge (for future merge detection)
- [ ] Process monitoring infrastructure (for Phase 2 process pooling)
- [ ] SERVO_INTEGRATION.md spec (link clicking, history, downloads)

### Phase 2 (Weeks 9-12: Browser Features + Performance)
🟡 Add features dropped from current plan:
- [ ] Session branching/merging
- [ ] Ghost nodes (preserve structure)
- [ ] Sidebar layout (optional tab list)
- [ ] Export/import (JSON, HTML, bookmarks.html)
- [ ] Process pooling (if memory testing shows need)
- [ ] Crash recovery (process auto-respawn)

### Phase 3 (Weeks 13-16+: P2P Collaboration)
🔴 Design now, implement later:
- [ ] Merge strategies (CRDT-inspired)
- [ ] Signature verification (zero-trust model)
- [ ] Sync protocol (command log + vector clocks)
- [ ] Conflict UI (user resolution)
- [ ] Test suite for distributed scenarios

---

## PRIORITY ISSUES (Act Now)

1. **Design command trait** (Phase 1, week 1): Unblocks P2P later
2. **Process monitoring** (Phase 1, weeks 1-2): De-risks webview strategy
3. **SERVO_INTEGRATION spec** (Phase 1, week 1): Unblocks detail view implementation
4. **Export format spec** (Phase 1, week 2): Unblocks sharing/collaboration research

These 4 tasks take <10 hours total but prevent major reworks in Phase 2/3.
