# GRAPHSHELL P2P COLLABORATION

**Purpose**: Detailed specification for P2P synchronization, decentralized collaboration, and secure graph sharing. This is Phase 3+ research and implementation guidance.

---

## Design Principle: Local-First, Sync Second

Graphshell should:
1. Work perfectly offline (single user)
2. Support optional sync (P2P, decentralized)
3. No mandatory server (unlike Notion, Google Docs)
4. Handle conflicts gracefully (show UI when needed)

---

## 1. Sync Architecture Overview

```
Peer A (Local)               Peer B (Local)
┌──────────────┐             ┌──────────────┐
│Graph Storage │             │Graph Storage │
│+ Op Log      │             │+ Op Log      │
└──────────────┘             └──────────────┘
      ▲                           ▲
      │ [1] Sync Request          │
      └──────────────────────────┬┘
                                 │
         [2] Send OperationLog   │
         [3] Receive OperationLog│
                                 │
                         Sync Engine
                         (Merge Ops)
                                 │
      ┌──────────────────────────┘
      │
      ▼ [4] Apply to local state
   Graph Updated
```

**Key insight**: Like Obsidian Sync or git, not like Google Docs.
- No real-time character-level merging
- Operations are batched (node creation, edge deletion, etc.)
- Merges happen at command level
- Conflicts shown to user if needed

---

## 2. Command-Based Mutation

**All graph changes are Commands** (idempotent, replayable, syncable):

```rust
pub trait Command: Serialize + Clone {
    fn execute(&self, graph: &mut Graph) -> Result<()>;
    fn inverse(&self) -> Box<dyn Command>;
    fn timestamp(&self) -> Timestamp;
    fn peer_id(&self) -> PeerId;
}

pub enum GraphCommand {
    CreateNode(CreateNodeCommand),
    DeleteNode(DeleteNodeCommand),
    UpdateNodeMetadata(UpdateNodeMetadataCommand),
    CreateEdge(CreateEdgeCommand),
    DeleteEdge(DeleteEdgeCommand),
    MoveNode(MoveNodeCommand),
}

// Example command
pub struct CreateNodeCommand {
    node_id: NodeKey,
    url: Url,
    title: String,
    position: Point2D<f32>,
    timestamp: Timestamp,
    peer_id: PeerId,
}
```

**Benefit**: Every mutation is:
- Durable (saved to operation log before executing)
- Syncable (can be sent to peers)
- Undoable (inverse command is available)
- Replayable (crash recovery: replay all ops from disk)

---

## 3. Operation Log & Durability

```rust
pub struct OperationLog {
    operations: Vec<LogEntry>,
    file_path: PathBuf,
}

pub struct LogEntry {
    command: Box<dyn Command>,
    sequence_number: u64,
    synced_to_peers: Vec<PeerId>,    // Which peers have this op
}

impl OperationLog {
    pub fn append(&mut self, cmd: Box<dyn Command>) -> Result<()> {
        let entry = LogEntry {
            command: cmd.clone(),
            sequence_number: self.operations.len() as u64,
            synced_to_peers: vec![],
        };
        
        // 1. Persist to disk first (durability)
        self.save_entry_to_file(&entry)?;
        
        // 2. Then add to memory
        self.operations.push(entry);
        
        Ok(())
    }
    
    pub fn unsynced_ops(&self) -> Vec<(u64, &Box<dyn Command>)> {
        self.operations.iter()
            .enumerate()
            .map(|(i, entry)| (i as u64, &entry.command))
            .collect()
    }
    
    pub fn mark_synced(&mut self, peer_id: PeerId, up_to_seq: u64) {
        for i in 0..=up_to_seq as usize {
            self.operations[i].synced_to_peers.push(peer_id);
        }
    }
}
```

**File format** (append-only log):
```
JSON Lines (JSONL), one command per line
[timestamp] [peer_id] [seq_num] {command json}
1706000000 A 0 {"type":"CreateNode","node_id":"abc123",...}
1706000100 A 1 {"type":"UpdateNodeMetadata","node_id":"abc123","title":"New Title"}
1706000200 B 0 {"type":"CreateNode","node_id":"def456",...}
```

**Size**: ~500 bytes per operation → 1000 operations ≈ 500KB (acceptable)

---

## 4. Version Vectors for Causality

**Problem**: How do you know if two operations are concurrent or sequential?

**Solution**: Version vector per peer

```rust
pub struct VersionVector {
    clocks: HashMap<PeerId, u64>,
}

impl VersionVector {
    pub fn increment(&mut self, peer_id: PeerId) {
        *self.clocks.entry(peer_id).or_insert(0) += 1;
    }
    
    pub fn happened_before(&self, other: &VersionVector) -> bool {
        // self < other: All clocks ≤, at least one <
        self.clocks.iter().all(|(peer, clock)| {
            clock <= other.clocks.get(peer).copied().unwrap_or(0)
        }) && self.clocks != &other.clocks
    }
    
    pub fn concurrent(&self, other: &VersionVector) -> bool {
        !self.happened_before(other) && !other.happened_before(self)
    }
}

// Apply version vector to each node
pub struct Node {
    id: NodeKey,
    url: Url,
    version_vector: VersionVector,  // Track when this was last modified
    created_by: PeerId,
    last_modified_by: PeerId,
    last_modified_timestamp: Timestamp,
}
```

---

## 5. Merge Strategies

**When two peers make concurrent changes, how do you resolve?**

### Strategy 1: Last-Write-Wins (LWW)

**Simple**: Whoever has the later timestamp wins.

```rust
pub fn merge_node_metadata(local: &mut Node, remote: &Node) {
    if remote.last_modified_timestamp > local.last_modified_timestamp {
        local.title = remote.title.clone();
        local.notes = remote.notes.clone();
        local.last_modified_by = remote.last_modified_by;
        local.version_vector = remote.version_vector.clone();
    }
}
```

### Strategy 2: Version-Vector Ordering

**Better**: Use version vectors to determine causality.

```rust
pub fn merge_node_metadata(local: &mut Node, remote: &Node) {
    if remote.version_vector.happened_before(&local.version_vector) {
        return;  // Local is newer, keep it
    } else if local.version_vector.happened_before(&remote.version_vector) {
        *local = remote.clone();  // Remote is newer, take it
    } else {
        // Concurrent: conflict!
        return Err(Conflict {
            local_version: local.version_vector.clone(),
            remote_version: remote.version_vector.clone(),
        });
    }
}
```

### Strategy 3: CRDT-Inspired Merge for Positions

**For physics state** (node positions), use commutative operations:

```rust
pub struct CRDTPosition {
    last_update_by: PeerId,
    last_update_at: Timestamp,
    position: Point2D<f32>,
    version_vector: VersionVector,
}

pub fn merge_positions(local: &mut CRDTPosition, remote: &CRDTPosition) {
    if remote.version_vector.happened_before(&local.version_vector) {
        return;  // Local is newer
    } else if local.version_vector.happened_before(&remote.version_vector) {
        *local = remote.clone();  // Remote is newer
    } else {
        // Concurrent: average the positions (physics will converge anyway)
        local.position = Point2D {
            x: (local.position.x + remote.position.x) / 2.0,
            y: (local.position.y + remote.position.y) / 2.0,
        };
        local.version_vector.merge_with(&remote.version_vector);
    }
}
```

### Strategy 4: User Resolution for Topology

**For important changes** (node deletion, edge creation), ask user:

```rust
pub enum MergeConflict {
    NodeDeleted {
        node_id: NodeKey,
        deleted_by: PeerId,
        deleted_at: Timestamp,
        edges_exist: bool,  // User added edges after deletion?
    },
    EdgeConflict {
        from: NodeKey,
        to: NodeKey,
        local_exists: bool,
        remote_exists: bool,
    },
}

pub fn show_conflict_ui(conflict: &MergeConflict) -> ConflictResolution {
    // Show UI to user: "Peer B deleted node X, but you have edges to it. Keep or delete?"
    // Returns: Keep / Delete / Merge
}
```

---

## 6. Sync Protocol

**When two peers connect, how do they exchange state?**

```
Peer A                          Peer B
┌──────────┐                    ┌──────────┐
│Graph v5  │                    │Graph v3  │
│Op Log:   │                    │Op Log:   │
│ 0-4      │                    │ 0-2      │
└──────────┘                    └──────────┘
      │                              │
      │──[1] Pull Request ─────────>│
      │    "I am at v5, seq 4"      │
      │                             │
      │<─[2] State: ops 3-4 ────────│
      │    (ops A did but B didn't) │
      │                             │
      │──[3] State: ops 0-2 ───────>│
      │    (ops B did but A didn't) │
      │                             │
      │  [4] Apply ops 3-4 locally  │
      │<─────────────────────────────│
      │                             │
      │      [5] Apply ops 0-2      │
      │      Graph now synced!      │
```

**Protocol**:

```rust
pub struct SyncRequest {
    peer_id: PeerId,
    our_seq: u64,           // "We have up to seq 4"
}

pub struct SyncResponse {
    peer_id: PeerId,
    their_seq: u64,         // "We have up to seq 2"
    ops: Vec<Box<dyn Command>>,  // "Here are ops 3-4"
}

impl P2PSync {
    pub fn pull(&self, peer: &PeerId) -> Result<SyncResponse> {
        let req = SyncRequest {
            peer_id: self.our_id,
            our_seq: self.local_log.len() as u64,
        };
        
        let resp = peer.send_sync_request(&req).await?;
        
        // Apply received ops
        for op in &resp.ops {
            self.graph.execute(op)?;
            self.local_log.mark_synced(*peer, op.timestamp());
        }
        
        Ok(resp)
    }
    
    pub fn push_to(&self, peer: &PeerId, up_to_seq: u64) -> Result<()> {
        let ops = self.local_log.ops_since(peer, up_to_seq);
        peer.send_ops(&ops).await?;
        Ok(())
    }
}
```

---

## 7. Conflict Scenarios & Resolution

### Scenario A: Concurrent Node Creation

```
Peer A: Create node https://example.com at (100, 100)
Peer B: Create node https://example.com at (200, 200)
→ Merge: Keep both? They have same URL but different positions.
```

**Resolution**:
- Both nodes are unique (different node_ids)
- Position difference is OK (they'll be at different spots)
- Create edge between them? (optional, could be manual)

### Scenario B: Concurrent Node Deletion

```
Peer A: Delete node X
Peer B: Add edge to node X
→ Merge: Should B's edge be deleted too? Or recreate node?
```

**Resolution (Option 1: Ghost nodes)**:
- Keep node X as "ghost" (invisible but edges preserved)
- B's edge is still valid (points to ghost)

**Resolution (Option 2: Show conflict)**:
- "Peer A deleted node X, but you added edge to it. Keep X or delete edge?"
- User chooses

### Scenario C: Concurrent Metadata Edit

```
Peer A: Update node X title to "React Docs"
Peer B: Update node X title to "React API Ref"
→ Merge: Which title wins?
```

**Resolution**:
- Use version vectors + last-write-wins
- Or: Show conflict UI if truly concurrent
- Or: Store both versions (version history)

### Scenario D: Concurrent Position Update (Physics)

```
Peer A: Physics moves node X to (150, 150)
Peer B: Physics moves node X to (160, 160)
→ Merge: Average to (155, 155) + continue physics
```

**Resolution**:
- Average positions (they'll naturally converge to stable state)
- Physics engine handles it

---

## 8. Network Transport

**How are synced ops transmitted?**

### Option A: Direct P2P (Peer A ↔ Peer B)

Requires:
- Static IP or NAT traversal (STUN, TURN)
- Firewall holes
- Complex for NAT/mobile

**Use case**: Same network, direct connection.

**Implementation**:
```rust
pub struct DirectP2P {
    local_addr: SocketAddr,
    peers: HashMap<PeerId, SocketAddr>,
}

impl DirectP2P {
    pub async fn broadcast_ops(&self, ops: &[Box<dyn Command>]) {
        for (peer_id, addr) in &self.peers {
            self.send_to(addr, ops).await;
        }
    }
}
```

### Option B: Relay Server (Peer A → Relay → Peer B)

Simpler:
- Relay runs on static IP (e.g., AWS, DigitalOcean)
- Peers connect to relay (outbound TCP/WebSocket)
- Relay forwards ops
- No NAT issues

**Use case**: Internet, mobile, different networks.

**Implementation**:
```rust
pub struct RelayedP2P {
    relay_addr: String,  // e.g., "sync.graphshell-network.org:8080"
    group_id: String,    // "my-research-group"
}

impl RelayedP2P {
    pub async fn connect_to_relay(&self) {
        let ws = WebSocket::connect(&self.relay_addr).await?;
        ws.send(AuthMessage {
            group_id: self.group_id.clone(),
            peer_id: self.peer_id,
            signature: sign_with_group_key(),
        }).await?;
    }
    
    pub async fn broadcast_ops(&self, ops: &[Box<dyn Command>]) {
        let msg = SyncMessage {
            from: self.peer_id,
            to: None,  // Broadcast to group
            ops,
        };
        self.relay_connection.send(msg).await?;
    }
}
```

**Recommendation for Phase 3**: Start with relay (easier), add direct P2P later if needed.

### Option C: Cloud Sync (Peer A → Cloud → Peer B)

Like Obsidian Sync:
- Centralized cloud service
- Peers sync to cloud
- Cloud syncs between peers
- Cloud provides conflict resolution, versioning

**Pros**: Simple, reliable, can add server-side features (search indexing, analysis)  
**Cons**: Server required, privacy concerns, costs

**Not recommended for MVP** (defeats P2P goal), but possible for Phase 4+.

---

## 9. Security & Trust Model

### Trust Model 1: Full Trust (Friends Only)

**Assumption**: All peers in group are trusted (friends, teammates).  
**Key exchange**: Pre-shared key or out-of-band exchange (email, phone).

```rust
pub struct TrustedGroupSync {
    group_id: String,
    shared_key: Vec<u8>,  // 32 bytes, AES-256
    peers: Vec<PeerId>,
}

impl TrustedGroupSync {
    pub fn encrypt_message(&self, msg: &[u8]) -> Vec<u8> {
        encrypt_aes_gcm(msg, &self.shared_key)
    }
    
    pub fn decrypt_message(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        decrypt_aes_gcm(encrypted, &self.shared_key)
    }
}
```

**Setup**: Two peers agree on group ID + shared key, connect to relay with both.

### Trust Model 2: Zero Trust (Strangers/Public)

**Assumption**: Peers may be adversarial; verify authenticity.  
**Mechanism**: Digital signatures (Ed25519) + public key distribution.

```rust
pub struct SignedCommand {
    command: Box<dyn Command>,
    signature: Vec<u8>,        // Sign(peer_private_key, command)
    peer_public_key: Vec<u8>,  // For verification
}

impl SignedCommand {
    pub fn verify(&self, trusted_keys: &[Vec<u8>]) -> bool {
        let is_trusted = trusted_keys.contains(&self.peer_public_key);
        let is_valid = verify_signature(&self.signature, &self.command, &self.peer_public_key);
        is_trusted && is_valid
    }
}
```

**Setup**: Each peer has (public_key, private_key). Public keys shared via DHT, DNS, or manual entry.

**Trade-off**:
- Zero trust: More secure, heavier computation, slower
- Full trust: Faster, requires secure out-of-band setup

**Recommendation**: Start with full trust (Phase 3). Add zero trust later if needed.

---

## 10. Data Format & Portability

**Export/Import format for sharing graphs**:

```rust
pub struct PortableGraph {
    version: String,           // "1.0"
    created_at: Timestamp,
    app_version: String,       // "graphshell-0.2.0"
    
    nodes: Vec<PortableNode>,
    edges: Vec<PortableEdge>,
    
    operation_log: Vec<PortableCommand>,  // For rebuild
}

pub struct PortableNode {
    id: String,               // UUID, stable across exports
    url: String,
    title: String,
    notes: String,
    tags: Vec<String>,
    
    position: (f32, f32),     // Absolute position
    created_at: Timestamp,
    created_by: String,       // Peer name/id
    
    metadata: PortableMetadata,
}

pub struct PortableMetadata {
    favicon: Option<String>,  // Base64 PNG
    thumbnail: Option<String>, // Base64 JPEG
    description: String,
}
```

**Export formats**:
- **JSON**: Human-readable, editable
- **JSON + gzip**: Compressed for email
- **HTML**: Interactive, sharable link
- **Markdown**: For Obsidian/Roam compat

**Import compatibility**:
- bookmarks.html (Firefox)
- Pocket export
- Notion export
- Obsidian markdown

**Phase 2/3 tasks**:
- [ ] JSON export/import
- [ ] HTML interactive export
- [ ] bookmarks.html importer
- [ ] Notion/Obsidian converters

---

## 11. Phase 3 Roadmap (P2P)

🔴 **Build**:
- [ ] Sync protocol (pull/push)
- [ ] Merge strategies (LWW + CRDT)
- [ ] Conflict UI (user resolution)
- [ ] Network transport (direct P2P or relay)
- [ ] Signature verification (if zero-trust model)
- [ ] Multi-peer testing

---

## Related

- Browser behavior spec: [GRAPHSHELL_AS_BROWSER.md](../GRAPHSHELL_AS_BROWSER.md)
- Architecture decisions and milestones: [../ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md), [../IMPLEMENTATION_ROADMAP.md](../IMPLEMENTATION_ROADMAP.md)
- Verse research and tokenization: [VERSE.md](VERSE.md)
