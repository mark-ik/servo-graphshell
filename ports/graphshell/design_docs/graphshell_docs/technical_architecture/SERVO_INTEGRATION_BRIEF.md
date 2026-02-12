# Servo Integration Brief

**Purpose**: Integrate Servo's webview system into Graphshell's graph browsing model  
**Status**: Implemented in core graph browsing (Feature Target 1 complete)  
**Scope**: Node ↔ Webview lifecycle binding, navigation hooks, multi-webview management  
**References**: [IMPLEMENTATION_ROADMAP.md](../IMPLEMENTATION_ROADMAP.md) Feature Target 1, [GRAPHSHELL_AS_BROWSER.md](../GRAPHSHELL_AS_BROWSER.md)

---

## Current State

**Graphshell Foundation** (ports/graphshell/):
- Graph structures use petgraph `StableGraph` with URL/title/position metadata
- Application state (`app.rs`) manages WebViewId ↔ NodeKey mappings
- Webview lifecycle and navigation tracking implemented in `desktop/gui.rs`
- Graph view destroys webviews to avoid framebuffer bleed-through

**Servo Capabilities** (libservo):
- WebViewManager for creating/destroying browsing contexts
- WindowMethods trait for DOM operations and event handling
- WebRender for compositing multiple render targets
- ipc-channel for cross-process communication (if using origin-grouped processes)
- Navigation hooks via WindowMethods::load_url()

---

## Integration Model

### Node ↔ Webview Lifecycle

```
Node Creation (User clicks link)
    ↓
Create WebViewId in Servo
    ↓
Establish HashMap mapping (WebViewId ↔ NodeKey)
    ↓
Load URL via WindowMethods::load_url()
    ↓
Render webview when Node in Detail view
    ↓
Destroy WebViewId when Node becomes Cold
    ↓
Remove HashMap mapping
```

### Node Lifecycle (Active/Warm/Cold)

| State | Webview | Rendering | Purpose |
|-------|---------|-----------|---------|
| **Active** | Created, visible | Full size in Detail view | Currently browsing |
| **Warm** | Created, hidden | Off-screen cached render | Quick navigation (pool of 2-4) |
| **Cold** | Destroyed | Not rendered (favicon only) | Memory savings, can recreate if visited again |

**Key Invariant**: Only 1 Active, 2-4 Warm, rest Cold. Total webviews in memory ≈ 5-7 max.

---

## Architecture: Servo Integration Layer

### Modifications to graphshell/app.rs

```rust
pub struct GraphBrowserApp {
    // Existing
    pub graph: Graph,
    pub view: View,
    pub physics_worker: PhysicsWorker,
    
    // New: Servo integration
    pub webview_manager: WebViewManager,  // Servo's API
    pub webview_map: HashMap<WebViewId, NodeKey>,  // Bidirectional lookup
    pub active_node: Option<NodeKey>,  // Current browsing context
    pub warm_pool: Vec<(NodeKey, WebViewId)>,  // Warm standby webviews
}

impl GraphBrowserApp {
    // New methods
    pub fn create_webview_for_node(&mut self, node_key: NodeKey) -> Result<WebViewId> {
        // 1. Get Node URL
        // 2. Create WebViewId via webview_manager.create()
        // 3. Add to HashMap
        // 4. Load URL
        // 5. Return WebViewId
    }
    
    pub fn destroy_webview(&mut self, node_key: NodeKey) -> Result<()> {
        // 1. Get WebViewId from HashMap
        // 2. Destroy via webview_manager.close()
        // 3. Remove from HashMap, warm pool
    }
    
    pub fn migrate_to_cold(&mut self, node_key: NodeKey) -> Result<()> {
        // Transition Warm → Cold
        // Called by physics engine when node moves off-screen for 30+ seconds
    }
    
    pub fn on_node_selected(&mut self, node_key: NodeKey) -> Result<()> {
        // 1. If already Active webview: no-op
        // 2. If in Warm pool: promote to Active
        // 3. If Cold: create new WebViewId
        // 4. Show webview in Detail view
    }
    
    pub fn on_link_clicked(&mut self, source_node: NodeKey, url: &Url) -> Result<()> {
        // 1. Check if URL already in graph
        // 2. If yes: navigate to existing node
        // 3. If no: create new node, create webview, load URL
        // 4. Create edge: source → new node (type: Hyperlink)
    }
}
```

### Servoshell Integration: Event Loop Hooks

**File**: ports/servoshell/desktop/event_loop.rs (modifications)

```rust
// Add channel for graph notifications
pub struct WindowEvent {
    webview_id: WebViewId,
    event: WindowEventType,
}

pub enum WindowEventType {
    LinkClicked(Url),  // Forward to graph
    PageLoaded,         // Notify graph (thumbnail ready)
    TitleUpdated(String),  // Update Node::title
    FaviconAvailable(Vec<u8>),  // Store in Node::favicon
    NavigationStarted,  // Pause physics?
}

// In event loop:
match window_event {
    WindowEventType::LinkClicked(url) => {
        // Send to graph layer
        graph_sender.send(GraphEvent::LinkClicked { 
            webview_id, 
            url 
        })?;
    }
    // ... handle other events
}
```

### Physics Integration: Pause During Interaction

**File**: ports/graphshell/physics/worker.rs (no changes needed)

**File**: ports/graphshell/input/mod.rs (integrate with webview state)

```rust
pub fn handle_mouse_event(...) {
    if webview_is_loaded(node_key) {
        // Forward mouse events to Servo
        servo_window.handle_mouse(normalized_coords);
    } else {
        // Use graph interaction (drag nodes, pan)
        graph_interaction();
    }
}
```

---

## Implementation Steps (Feature Target 1)

### Completed Integration

1. **Webview creation/destruction**
    - Webviews created on demand and torn down when view switches to graph.
    - Bidirectional mapping maintained in application state.

2. **Navigation tracking**
    - URL changes detected and reflected as node + edge updates.
    - History vs hyperlink edges distinguished in navigation logic.

3. **Lifecycle handling**
    - Active webview shown in detail view; graph view destroys webviews.
    - Remaining lifecycle tiers are planned for Phase 1.5+ (thumbnails/favicons).

---

## Key APIs to Integrate With

### libservo WebViewManager

```rust
pub struct WebViewManager { ... }

impl WebViewManager {
    pub fn create(&mut self, parent: NativeWindow) -> WebViewId;
    pub fn close(&mut self, webview_id: WebViewId) -> Result<()>;
    pub fn get_mut(&mut self, webview_id: WebViewId) -> Option<&mut WebView>;
    pub fn iter(&self) -> Iter<WebViewId>;
}

pub trait WindowMethods {
    fn load_url(&self, url: Url);
    fn get_title(&self) -> String;
    fn handle_mouse(&mut self, event: MouseEvent);
    fn handle_key(&mut self, event: KeyboardEvent);
}
```

### Servo Script Messages

For receiving events from script/layout processes:

```rust
pub enum ConstellationMsg {
    InitLoadUrl(Url),
    SetDocumentActivity { .. },
    FocusIFrame { .. },
    // ... other messages
}

// Listen to script process output:
let (tx, rx) = channel();
script_process.send(ScriptMessage::RegisterListener(tx))?;

// In event loop:
match rx.recv() {
    // Forward to graph layer
}
```

---

## Success Criteria (Feature Target 1)

**Functional**:
- [ ] Load https://example.com → creates first node
- [ ] Click link → creates second node + Hyperlink edge
- [ ] Switch to Graph view → webview hidden
- [ ] Double-click node → Detail view shows webview
- [ ] Delete node → webview destroyed
- [ ] 10 pages open → max 7 webviews in RAM

**Quality**:
- [ ] No webview leaks (peak memory stays ~50MB per webview)
- [ ] Navigation latency < 500ms (user perceives responsiveness)
- [ ] No panics on rapid create/destroy cycles
- [ ] Graph remains interactive during webview loads

**Testing**:
- [ ] Integration tests: webview lifecycle (create → load → hide → show → destroy)
- [ ] Memory tests: browse 100 pages → memory stays bounded
- [ ] Navigation tests: click 50 links → all nodes created correctly
- [ ] Edge case: rapid node deletion while webview loading

---

## Known Challenges & Mitigations

### Challenge 1: Navigation Event Interception

**Problem**: How do we intercept link clicks before Servo navigates the webview?

**Options**:
- A. Hook in script process (modify script.rs to send click events)
- B. Create intermediate proxy webview that intercepts clicks
- C. Let Servo navigate, then extract new URL from ConstellationMsg

**Recommendation**: Option A (cleanest, but requires Servo fork modification)

**Fallback**: Option C (simpler, but adds latency)

### Challenge 2: Multi-Webview Rendering

**Problem**: How do we composite multiple webviews into one egui canvas?

**Options**:
- A. Render webview to offscreen texture, copy to egui
- B. Allocate egui Area per webview, show/hide based on View
- C. Use WebRender's capabilities directly (requires lower-level integration)

**Recommendation**: Option B (leverages existing egui + WebRender integration)

### Challenge 3: Lifecycle Transitions

**Problem**: When should we evict Warm nodes to Cold? Physics-based (velocity) or time-based (30 sec off-screen)?

**Recommendation**: Hybrid:
- Physics-based: if node velocity → 0 and not in viewport for 30s, move to Cold
- User-driven: if warm pool full, evict least-recently-accessed

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_webview_create_destroy() {
    let mut app = GraphBrowserApp::new();
    let node = create_test_node("https://example.com");
    
    let wv_id = app.create_webview_for_node(node.id)?;
    assert!(app.webview_map.contains_key(&wv_id));
    
    app.destroy_webview(node.id)?;
    assert!(!app.webview_map.contains_key(&wv_id));
}

#[test]
fn test_lifecycle_warm_to_cold() {
    // Browse 5 pages, verify max 5-7 webviews
    // Move oldest to Cold, verify eviction
}
```

### Integration Tests

```rust
#[test]
fn test_full_browse_session() {
    let app = launch_graphshell();
    
    // Load first page
    app.navigate("https://example.com");
    assert_eq!(app.graph.node_count(), 1);
    
    // Click 10 links
    for i in 0..10 {
        app.click_link(format!("https://example.com/page{}", i));
        assert!(app.graph.finds_node_by_url(...).is_some());
    }
    
    // Verify memory bounded
    assert!(app.webview_manager.active_count() <= 7);
}
```

---

## References

- **Servo Book**: https://book.servo.org/
- **Servo Architecture**: https://github.com/servo/servo/blob/main/docs/architecture.md
- **WebViewManager**: `components/servo/src/lib.rs`
- **WindowMethods**: `components/script/dom/window.rs`
- **GRAPHSHELL_AS_BROWSER.md**: Behavioral spec (how links are clicked, navigation model)
- **IMPLEMENTATION_ROADMAP.md**: Feature Target 1 detailed tasks & validation tests

---

**Next Action**: Start Phase 1.1 (study Servo API), document findings in implementation log.
