# GRAPHSHELL AS A WEB BROWSER

**Purpose**: Detailed specification for how Graphshell operates as a functional web browser (graph-first UX with a Servo-backed detail view).

**Document Type**: Behavior specification (not implementation status)  
**Status**: Foundation implemented (~3,500 LOC), Servo integration in progress  
**See**: [ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md) for actual code status

---

## Design Principle: Graph-First, Cluster-Second

Unlike Firefox (tabs are primary), Graphshell inverts the relationship:
- **Primary interface**: Force-directed graph of webpages
- **Secondary interface**: Detail view + cluster strip (linear projection of the graph)
- **Implicit history**: Graph edges track navigation/relationships
- **Browser history**: Per-node history stack (like browser back/forward)

---

## 1. Navigation Model

### Link Clicking in Webview

**Scenario**: User is in detail view of node A (viewing https://example.com/page1), clicks a link to https://example.com/page2

**Recommended behavior**:

```rust
pub enum LinkClickBehavior {
    /// Reuse existing node if URL already open
    ReuseIfExists,
    
    /// Create new node for every link
    AlwaysNew,
    
    /// Create new only if different domain
    NewPerDomain,
}

// Graphshell should implement: ReuseIfExists (like a cluster strip entry, but for graph)

impl DetailView {
    pub fn on_link_clicked(&mut self, url: &Url) -> Action {
        if let Some(existing_node) = self.graph.find_node_by_url(url) {
            // Node already exists in graph
            return Action::SwitchToNode(existing_node.id);
        } else {
            // New URL: create node near current node
            let new_node = Node {
                url: url.clone(),
                position: self.current_node.position + random_offset(),
                title: "[Loading...]".to_string(),
                velocity: Vector2D::zero(),
            };
            
            // Create edge from current → new (manual type, tracks navigation)
            let edge = Edge {
                from: self.current_node.id,
                to: new_node.id,
                edge_type: EdgeType::Manual,  // User navigated here
                weight: 1.0,
            };
            
            self.graph.add_node(new_node.clone());
            self.graph.add_edge(edge);
            
            // Fetch metadata in background
            self.spawn_metadata_fetch_async(new_node.id, url);
            
            return Action::SwitchToNode(new_node.id);
        }
    }
}
```

**Result**: Graph grows organically as user browses. Clicking same link multiple times reuses node (like cluster strip entries).

### Back/Forward Navigation

**Scenario**: User is in detail view, wants to go back to previous page

**Expected**: Ctrl+[ (back), Ctrl+] (forward) buttons or navigation UI

**Recommended implementation**:

```rust
pub struct DetailView {
    current_node: NodeKey,
    history_stack: Vec<NodeKey>,      // Back stack
    forward_stack: Vec<NodeKey>,       // Forward stack
}

impl DetailView {
    pub fn go_back(&mut self) -> Result<()> {
        if let Some(prev_node) = self.history_stack.pop() {
            self.forward_stack.push(self.current_node);
            self.current_node = prev_node;
            self.render();
            Ok(())
        } else {
            Err("No history".into())
        }
    }
    
    pub fn go_forward(&mut self) -> Result<()> {
        if let Some(next_node) = self.forward_stack.pop() {
            self.history_stack.push(self.current_node);
            self.current_node = next_node;
            self.render();
            Ok(())
        } else {
            Err("No forward history".into())
        }
    }
    
    pub fn switch_to_node(&mut self, target: NodeKey) {
        self.history_stack.push(self.current_node);
        self.forward_stack.clear();  // Clear forward on new navigation
        self.current_node = target;
    }
}
```

**Key insight**: Per-node history is separate from graph edges.
- **Edges**: Intentional relationships (what you're studying)
- **History stack**: Browsing sequence (chronological)

### Global Browser History

**Scenario**: User wants to see all pages visited (across all nodes)

**Recommended**:

```rust
pub struct GlobalHistory {
    entries: Vec<HistoryEntry>,  // Ordered by timestamp
}

pub struct HistoryEntry {
    url: Url,
    title: String,
    timestamp: Timestamp,
    node_id: NodeKey,
    duration_on_page: Duration,  // How long user spent on page
    referrer: Option<Url>,       // Page they came from
}

impl GlobalHistory {
    pub fn most_visited(&self) -> Vec<&HistoryEntry> {
        // Return entries grouped by URL, sorted by visit count
    }
    
    pub fn recent(&self, limit: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(limit).collect()
    }
}
```

**UI**: History sidebar (Phase 2) shows timeline, filterable by date/domain.

---

## 2. Form State & Page Interactions

**Problem**: User fills form on page, submits, gets result page. What should Graphshell do?

**Scenario**:
- Node A = search.example.com with search form
- User types query, submits
- New page (search results) loads
- Is this a new node or update to existing node?

**Recommended behavior: Update existing node**

```rust
impl DetailView {
    pub fn on_page_loaded(&mut self, new_url: &Url, title: &str) {
        let old_url = &self.current_node.url;
        
        if same_origin(old_url, new_url) {
            // Same origin: update in-place (form submission, SPA navigation)
            self.graph.node_mut(self.current_node.id).url = new_url.clone();
            self.graph.node_mut(self.current_node.id).title = title.to_string();
        } else {
            // Different origin: create new node
            let new_node = self.graph.add_node(Node {
                url: new_url.clone(),
                title: title.to_string(),
                position: self.current_node.position + offset,
                ..
            });
            self.graph.add_edge(Edge {
                from: self.current_node.id,
                to: new_node.id,
                edge_type: EdgeType::Manual,
            });
            self.switch_to_node(new_node.id);
        }
    }
}
```

**Result**: Form submission on same origin updates node (cleaner graph). Cross-origin submission creates new node.

---

## 3. Bookmarks Integration

**Current**: Manual edge creation

**Expected**: Browser-like bookmark UI

**Recommended**:

```rust
pub struct BookmarkManager {
    bookmarks: HashMap<NodeKey, Bookmark>,
}

pub struct Bookmark {
    node_id: NodeKey,
    added_at: Timestamp,
    folder: Option<String>,       // "Research/React", "Tools", etc.
    tags: Vec<String>,
}

impl DetailView {
    pub fn bookmark_current(&mut self) -> Result<()> {
        self.bookmarks.insert(self.current_node.id, Bookmark {
            node_id: self.current_node.id,
            added_at: now(),
            folder: Some("Inbox".to_string()),
            tags: vec![],
        });
        
        // Create bookmark edge for visualization
        // (optional, could also hide in UI)
        
        Ok(())
    }
}
```

**UI**:
- Ctrl+B toggles bookmark for current node (adds Bookmark icon)
- Sidebar (Phase 2) shows bookmark folders
- Import bookmarks.html from Firefox

---

## 4. Downloads & Files

**Scenario**: User downloads a file from a webpage

**Recommended**:

```rust
pub struct Download {
    id: DownloadId,
    source_node_id: NodeKey,
    url: Url,
    filename: String,
    size: u64,
    progress: f32,            // 0.0 to 1.0
    state: DownloadState,
}

pub enum DownloadState {
    Pending,
    InProgress,
    Completed(PathBuf),
    Failed(String),
    Paused,
}

pub struct DownloadManager {
    downloads: HashMap<DownloadId, Download>,
}

impl DetailView {
    pub fn on_download_started(&mut self, url: &Url, filename: &str) {
        let dl = Download {
            id: DownloadId::new(),
            source_node_id: self.current_node.id,
            url: url.clone(),
            filename: filename.to_string(),
            size: 0,
            progress: 0.0,
            state: DownloadState::Pending,
        };
        self.downloads.insert(dl.id, dl);
    }
}
```

**UI**: Downloads sidebar (Phase 2) shows in-progress + completed downloads.

---

## 5. Search & Address Bar

**Current**: Omnibar (graph-based search)

**Missing**: Ability to navigate directly to URL

**Recommended**:

```rust
pub enum OmnibarMode {
    SearchGraph,        // Default: search nodes by title/tags
    NavigateToUrl,      // "http://example.com" → create/switch node
}

impl Omnibar {
    pub fn on_input(&mut self, input: &str) {
        if input.starts_with("http://") || input.starts_with("https://") {
            // URL navigation
            let url = Url::parse(input).unwrap();
            self.emit_event(OmnibarEvent::NavigateToUrl(url));
        } else {
            // Graph search
            let results = self.search_graph(input);
            self.emit_event(OmnibarEvent::SearchResults(results));
        }
    }
}
```

**UX**: Omnibar works for both search (graph) and navigation (URL).

---

## Summary: How Graphshell Differs from Traditional Browsers

| Feature | Firefox | Graphshell |
|---------|---------|-------|
| **Primary UI** | Tab bar | Force-directed graph + cluster strip |
| **Navigation** | Click link → new tab | Click link → new/existing node |
| **History** | Browser history | Graph edges + per-node stack |
| **Bookmarks** | Bookmark folder tree | Nodes tagged as bookmarks |
| **Data portability** | HTML bookmarks | JSON, HTML, markdown |

**Core difference**: Graphshell treats navigation as graph construction, not sequential lists.

---

## Related

- P2P collaboration and decentralized sync: [verse_docs/GRAPHSHELL_P2P_COLLABORATION.md](verse_docs/GRAPHSHELL_P2P_COLLABORATION.md)
- Architecture decisions and implementation milestones: [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md), [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)
