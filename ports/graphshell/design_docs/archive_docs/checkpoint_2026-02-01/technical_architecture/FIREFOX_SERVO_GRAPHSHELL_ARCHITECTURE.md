# FIREFOX ARCHITECTURE & GRAPHSHELL: LESSONS & INTEGRATION POINTS

**Purpose**: Analyze Firefox's architecture decisions and identify what Graphshell should adopt, adapt, or avoid.

---

## Part 1: Firefox's Multi-Process Architecture

### 1.1 The Problem Firefox Solved (2015-2018)

**Before multiprocess** (Firefox < 48, 2015):
- Single process running all tabs
- One malicious/crashed tab crashes entire browser
- Long operations (JavaScript) freeze all tabs
- Memory waste: empty tabs kept in RAM

**Solution**: E10S (Electrolysis) — multiprocess redesign

### 1.2 How Firefox Structures Processes

```
Main Process (parent/browser)
│
├─ UI (omnibar, menus, buttons)
├─ IPC Router (message passing)
├─ Security Policy
├─ GPU Process (recent addition)
│
└─ Content Processes (children, e.g., 4-8 total)
   ├─ Content 1: example.com, mail.example.com, ...
   ├─ Content 2: github.com, gist.github.com, ...
   ├─ Content 3: youtube.com, ...
   └─ Content 4: Local files, etc.
```

**Key rule**: One origin per process (or few origins per process).

### 1.3 Process Allocation Strategy

**Origin grouping** (as of Firefox 54, 2017):

```rust
pub fn assign_process(origin: &Origin, process_pool: &[ProcessHandle]) -> ProcessHandle {
    // Deterministic hash-based assignment:
    // origin.hash() % process_pool.len() → process index
    
    let idx = origin.hash() % process_pool.len();
    process_pool[idx].clone()
}
```

**Why**: 
- Multiple tabs for same origin share process (memory efficient)
- Different origins get different processes (security isolation)
- Deterministic (same origin always goes to same process)

**Pool size**:
- Desktop: 4-8 processes (configurable, default 4)
- Mobile: 2-4 processes (lower power budget)
- High-end: 8+ processes possible

### 1.4 IPC (Inter-Process Communication)

**Transport**: Named pipes (Windows), Unix sockets (macOS/Linux)

**Message pattern**:

```
Content Process          Main Process
      │                       │
      ├──[1] SyncMessage────>│ (blocks until response)
      │                       │
      │<─────[2] Response────┤
      │
      ├──[3] AsyncMessage──>│ (fire-and-forget)
      │                       │
      │<─[4] AckMessage─────┤ (later, out-of-order)
```

**Types**:
- **Sync messages**: Critical (e.g., permission checks, window focus) — block until answered
- **Async messages**: Fire-and-forget (e.g., page load, scroll position)

**Volume**: ~10-50 messages per second during normal browsing.

### 1.5 Security Model: Sandboxing

**Content processes are sandboxed**:

**macOS**:
- Uses `sandbox` (built-in)
- Disallows: file I/O (except temp), network direct calls, kernel extensions
- Allowed: IPC to parent, GPU access, JavaScript JIT

**Linux**:
- Uses `seccomp-bpf` (kernel syscall filter)
- Allows only whitelisted syscalls
- Blocks: fork, ptrace, raw sockets, file writes

**Windows**:
- Uses Job Objects + LPAC (Low Privilege App Container)
- Restricted: Registry access, file system beyond temp
- Allowed: Direct GPU calls, audio output

**Servo's implementation** (our target):
- `-S` flag activates `gaol` library
- Similar to Firefox's sandbox
- Per-platform profiles (macOS, Linux, Windows)

### 1.6 Memory Management

**Key insight**: Firefox doesn't serialize page state.

**Instead**:
- Content process keeps DOM tree in memory
- Parent process has lightweight reference
- On crash: re-load page (DOM lost, but OK)

**For Graphshell**: This is what we're adopting (avoid serialization).

### 1.7 Crash Recovery

**When content process crashes**:

```
1. Parent process detects child death (wait() returns)
2. Parent notifies UI (red "X" on tab, sad face)
3. User clicks "Reload": Parent spawns new process
4. New process: fetch page, render from scratch
5. History: intact (stored in parent process)
```

**For Graphshell**: Should do similar (detect process death, mark node as "dead", allow respawn).

---

## Part 2: How Servo Implements Firefox's Model

### 2.1 Servo's EventLoop & Multiprocess

**Servo's `-M` flag** (enable multiprocess):

```rust
// In servo/components/constellation/event_loop.rs

pub struct EventLoop {
    origin: Origin,
    process_id: ProcessId,
    is_alive: bool,
}

pub struct Constellation {
    event_loops: HashMap<Origin, EventLoop>,  // One per origin
}

impl Constellation {
    pub fn create_window(&mut self, origin: Origin) -> EventLoopHandle {
        if let Some(loop_) = self.event_loops.get(&origin) {
            // Reuse existing process
            loop_.handle()
        } else {
            // Spawn new process
            let event_loop = EventLoop::new(origin.clone());
            self.event_loops.insert(origin, event_loop);
            event_loop.handle()
        }
    }
}
```

**Direct analogy**:
- Servo's EventLoop = Firefox's content process
- Servo's Constellation = Firefox's parent process
- Origin-based grouping = Firefox's process assignment

### 2.2 Servo's Pipeline & Display List

**Immutable tree pipeline** (same as Firefox Stylo):

```
Navigation (user clicks link)
    ↓
Script (parse HTML, execute JavaScript)
    ↓
Style (apply CSS via Stylo)
    ↓
Layout (calculate positions via Taffy)
    ↓
Paint (generate display list)
    ↓
Rasterize (WebRender turns display list → GPU commands)
    ↓
Composite (final pixels on screen)
```

**For Graphshell**: We layer on top:
- Servo renders webview (steps 1-7 above)
- Graphshell reads rendered pixels
- Graphshell composites graph + webview in egui

### 2.3 Servo's Sandboxing (gaol)

```rust
// In servo/components/constellation/sandboxing.rs

pub mod platform {
    #[cfg(target_os = "macos")]
    pub use self::macos::*;
    
    #[cfg(target_os = "linux")]
    pub use self::linux::*;
    
    #[cfg(target_os = "windows")]
    pub use self::windows::*;
}

pub fn sandbox_content_process() {
    // Platform-specific sandbox activation
    platform::activate_sandbox();
}
```

**For Graphshell**: Invoke with:
```bash
cargo run --release -- -M -S
# -M: Enable multiprocess (EventLoops)
# -S: Activate sandbox (gaol)
```

---

## Part 3: Graphshell-Specific Adaptations

### 3.1 Process Lifecycle in Graphshell Context

**Firefox model** (tab-centric):
```
User creates tab → Spawn process → User closes tab → Kill process
```

**Graphshell model** (node-centric, multiple nodes per origin):
```
User creates node A from origin O → Spawn process for O
User creates node B from origin O → Reuse process (no new spawn)
User closes nodes A, B → Kill process for O
```

**Tracking**:

```rust
pub struct ProcessManager {
    processes: HashMap<Origin, ProcessHandle>,
    origin_ref_count: HashMap<Origin, usize>,  // How many nodes?
}

impl ProcessManager {
    pub fn add_node(&mut self, origin: &Origin) {
        *self.origin_ref_count.entry(origin.clone()).or_insert(0) += 1;
        
        if !self.processes.contains_key(origin) {
            let process = spawn_servo_process(origin);
            self.processes.insert(origin.clone(), process);
        }
    }
    
    pub fn remove_node(&mut self, origin: &Origin) {
        if let Some(count) = self.origin_ref_count.get_mut(origin) {
            *count -= 1;
            
            if *count == 0 {
                // Last node from this origin closed
                if let Some(process) = self.processes.remove(origin) {
                    process.kill();  // Kill process
                }
            }
        }
    }
}
```

### 3.2 Detail View as Process Window

**Firefox**: Each content process can have multiple windows (tabs).

**Graphshell**: Each process can have multiple windows (nodes), but we only show one in detail view.

```rust
pub struct DetailView {
    current_origin: Origin,
    webview_handle: ProcessHandle,
    // Only one webview visible at a time in detail view
}

impl DetailView {
    pub fn switch_to_node(&mut self, node: &Node) {
        let new_origin = node.url.origin();
        
        if new_origin != self.current_origin {
            // Different origin: different process
            self.current_origin = new_origin;
            self.webview_handle = process_manager.get_process(&new_origin);
        }
        
        // Navigate process to node URL
        self.webview_handle.navigate(node.url.clone());
    }
}
```

### 3.3 Crash Recovery for Nodes

**When Servo process crashes**:

```rust
pub struct NodeMonitor {
    node_to_process: HashMap<NodeKey, Origin>,
    process_monitor: ProcessMonitor,
}

impl NodeMonitor {
    pub fn on_process_crashed(&mut self, origin: &Origin, graph: &mut Graph) {
        // Find all nodes from this origin
        for (node_id, node_origin) in &self.node_to_process {
            if node_origin == origin {
                // Mark node as "dead" (show indicator in graph)
                graph.node_mut(*node_id).state = NodeState::Dead;
            }
        }
        
        // Respawn on next interaction
        process_manager.respawn_process(origin);
    }
}

pub enum NodeState {
    Alive,
    Dead,       // Process crashed
    Loading,    // Waiting for page load
}
```

**UI**: Dead nodes appear faded/grayed out until respawned.

### 3.4 Memory Pressure Handling

**Firefox approach**: Monitor available RAM, suspend background tabs.

**Graphshell adaptation**: 

```rust
pub struct MemoryManager {
    max_process_count: usize,
    min_free_ram: u64,  // 500MB
}

impl MemoryManager {
    pub fn check_memory_pressure(&mut self, process_manager: &mut ProcessManager) {
        let free_ram = available_system_ram();
        
        if free_ram < self.min_free_ram {
            // Kill oldest/least-used origin process
            let victim_origin = self.find_lru_origin();
            process_manager.suspend_or_kill(&victim_origin);
        }
    }
    
    pub fn find_lru_origin(&self) -> Origin {
        // Find origin not interacted with in longest time
    }
}
```

---

## Part 4: Differences & Trade-Offs

### 4.1 Why Graphshell Can't Just Copy Firefox

| Feature | Firefox | Graphshell | Why Different |
|---------|---------|-------|---|
| **Tab bar** | Primary UI | Graph is primary | Different UX paradigm |
| **Window count** | Many (per tab) | One (detail view) | Simpler rendering |
| **Session restore** | Per-tab persistence | Per-graph session | Graph is unit of work |
| **History** | Global (all tabs) | Per-node stack + global | Graph structure matters |
| **Extensions** | Plugins run in-process | Phase 3+ P2P plugins | Security/modularity |

### 4.2 Where Graphshell Should Adopt Firefox Pattern

| Pattern | Firefox | Graphshell |
|---------|---------|-------|
| **Multiprocess** | Content isolated ✅ | Content isolated ✅ |
| **Sandboxing** | gaol-like ✅ | gaol (-S flag) ✅ |
| **IPC** | Named pipes ✅ | ipc-channel ✅ |
| **Crash recovery** | Reload page ✅ | Respawn process ✅ |
| **Memory management** | Monitor + suspend ✅ | Monitor + kill ⚠️ (simpler) |

### 4.3 Where Graphshell Goes Different (Intentionally)

| Pattern | Firefox | Graphshell | Why |
|---------|---------|-------|-----|
| **Process pooling** | Fixed pool (4-8) | Dynamic (1 per origin) | Graph UX doesn't need pooling |
| **Serialization** | No (reload page) | No (respawn process) | Same approach ✅ |
| **DOM state** | Lost on crash | Lost on crash | Acceptable tradeoff |
| **History sync** | Device sync (cloud) | P2P (Phase 3) | Different trust model |

---

## Part 5: Cross-Domain Interactions: Servo ↔ Servoshell ↔ Graphshell

### 5.1 Dependency Chain

```
graphshell (our app)
  ├─ servoshell (fork/integration)
  │   └─ servo (browser engine)
  │       ├─ ipc-channel (cross-process messaging)
  │       ├─ webrender (GPU rendering)
  │       ├─ stylo (CSS engine)
  │       └─ taffy (layout engine)
  ├─ egui 0.33.3 (UI framework)
  ├─ tokio (async runtime)
  └─ serde (serialization)
```

### 5.2 How Data Flows

**User clicks link in detail view**:

```
[Servo Content Process]
  1. JavaScript handler: window.location = newUrl
  2. Network request fetched
  3. HTML parsed, JS executed
  4. Paint command generated

         ↓ [IPC: display list + navigation event]

[Servo Parent Process (Constellation)]
  5. Receive navigation event
  6. Create/reuse content process
  7. Notify parent via IPC

         ↓ [IPC: "page loaded" event]

[Graphshell App (servoshell integration)]
  8. Receive notification
  9. Create graph node (if new URL)
  10. Add edge (from current → new)
  11. Switch detail view to new node
  12. Update graph rendering

         ↓ [egui rendering]

[GPU / Screen]
  13. Render updated graph + detail webview
```

### 5.3 Servo-Graphshell Integration Points

#### Point 1: Window Creation

```rust
// servoshell calls this:
pub fn create_window(url: Url) -> ServoShellWindow {
    let window = ServoShellWindow::new();
    graphshell_app.on_window_created(&window);  // ← Graphshell hook
    window
}

// Graphshell listens:
impl GraphshellApp {
    pub fn on_window_created(&mut self, window: &ServoShellWindow) {
        // Create graph node for URL
        let node = Node {
            url: window.url().clone(),
            position: random_position(),
            ..
        };
        self.graph.add_node(node);
    }
}
```

#### Point 2: Navigation Events

```rust
// Servo's IPC sends:
pub enum NavigationMessage {
    DocumentStarted { pipeline_id: PipelineId, url: Url },
    HistoryPushState { entry: HistoryEntry },
    LinkClicked { target_url: Url, user_activation: bool },
}

// Graphshell subscribes:
impl GraphshellApp {
    pub fn on_navigation_message(&mut self, msg: NavigationMessage) {
        match msg {
            NavigationMessage::LinkClicked { target_url, .. } => {
                // Handle as per Section 1.1 of GRAPHSHELL_AS_BROWSER.md
                self.on_link_clicked(&target_url);
            }
            _ => {}
        }
    }
}
```

#### Point 3: WebRender Display List

```rust
// Servo generates display list:
pub struct DisplayList {
    items: Vec<DisplayItem>,  // Rectangles, text, images, etc.
}

// Graphshell reads it:
impl GraphshellApp {
    pub fn on_display_list_updated(&mut self, list: &DisplayList) {
        // Extract metadata from DOM (if needed):
        // - Page title (already cached in Node)
        // - Favicon (already cached)
        // - Screenshot (for thumbnail) ← Optional, Phase 2
    }
}
```

### 5.4 Critical Servo Crate Dependencies

#### ipc-channel (0.20.2)

**Purpose**: Cross-process message passing (IPC).

**For Graphshell**:
- Servo already uses it for content ↔ parent communication
- Graphshell can hook into existing IPC for navigation events
- Or: Add new message types for Graphshell-specific events

```rust
// Servo already does this:
pub fn send_message_to_parent(msg: ScriptMsg) {
    ROUTER.route(msg);  // Route to parent process
}

// Graphshell extends:
pub enum ScriptMsg {
    // Servo's existing variants...
    GraphshellNodeCreated { url: Url, title: String },  // ← Custom
    GraphshellLinkClicked { target_url: Url },          // ← Custom
}
```

#### WebRender (0.68)

**Purpose**: GPU-accelerated rendering (display lists → pixels).

**For Graphshell**:
- Servo uses WebRender for web content
- Graphshell renders graph via egui (which uses webrender or wgpu)
- Both can coexist (composite in egui)

```
Servo Content          Servo Parent        Graphshell App
  │                      │                    │
  └──[paint list]──>     └──[GPU cmd]────>   │
                                               │
                                        ┌──────┴─────┐
                                        │             │
                                    [egui]      [Servo webview]
                                        │             │
                                        └──────┬──────┘
                                               │
                                           [Composite]
                                               │
                                            [Screen]
```

#### Stylo (CSS Engine) & Taffy (Layout)

**Purpose**: CSS styling and layout calculation.

**For Graphshell**: Mostly transparent (Servo handles it).

**Relevant for P2P**: If Graphshell ever stores page snapshots, would need to know about layout (position of elements for semantic extraction).

### 5.5 Servoshell Integration Checklist

**Phase 1, Week 1-2**:
- [ ] Fork servoshell
- [ ] Integrate graph module
- [ ] Listen to navigation events (Servo IPC)
- [ ] On link click: create/reuse nodes
- [ ] On process crash: mark node dead
- [ ] Test with `-M -S` flags

**Phase 2**:
- [ ] Extract metadata from Servo webview (title, favicon, description)
- [ ] Implement process pooling (if memory testing shows need)
- [ ] Add crash recovery (respawn process)

**Phase 3**:
- [ ] Sync metadata via P2P (cross-peer graph updates)
- [ ] Multi-peer conflict resolution

---

## Part 6: Servo-Specific Concerns & Solutions

### 6.1 Servo is Incomplete as a Browser

**Known gaps**:
- Limited extension system (not like Firefox)
- Some web APIs missing (WebUSB, WebBluetooth, etc.)
- Smaller web compat than Firefox (90% vs 99%)

**For Graphshell**:
- **Acceptable** (Graphshell is research tool, not general-purpose browser)
- Falls back to error page gracefully
- User can open in Firefox if needed

### 6.2 Servo's Performance vs Firefox

**Servo is slower for**:
- Large DOMs (< 5000 elements OK, > 10000 slow)
- Complex CSS (many pseudo-elements, deep nesting)
- Heavy JavaScript

**Graphshell's advantage**:
- Graphs are smaller (100-1000 nodes typical)
- No need to optimize for complex web pages
- Acceptable performance for research use

### 6.3 Servo's Memory Usage

**Servo uses more RAM per process than Chrome content**:
- ~100-200MB per process (vs ~80-120MB Chrome)
- 10 processes = 1-2GB (acceptable on modern hardware)

**Graphshell's mitigation**:
- Process pooling (Phase 2)
- Memory pressure monitoring (Phase 1)
- Process suspension (Phase 2, optional)

### 6.4 Servo's Windows Build Issues

**Known issues**:
- MozillaBuild shell required (not standard cmd.exe)
- Some dependencies require MSVC compiler
- Build time: 15-30 minutes (vs 5-10 on macOS/Linux)

**For Graphshell (Windows)**:
- Documented in WINDOWS_BUILD.md ✅
- One-time setup (~1 hour)
- Builds work reliably after setup

---

## Summary: Firefox ↔ Servo ↔ Graphshell Architecture

```
Firefox (Reference)
├─ Multiprocess: Content isolated ✅
├─ Sandboxing: gaol-like ✅
├─ IPC: Named pipes ✅
├─ Crash recovery: Reload page ✅
└─ Process pooling: Fixed pool ⚠️ (Graphshell: dynamic)

Servo (Our Engine)
├─ Adopts Firefox's multiprocess model ✅
├─ Uses ipc-channel (cross-platform IPC) ✅
├─ Has gaol sandboxing (-S flag) ✅
├─ Adds: WebRender (GPU rendering) ✅
└─ Limitation: Smaller web compat (acceptable for Graphshell)

Graphshell (Our App)
├─ Layers graph on top of Servo ✅
├─ Listens to navigation events (IPC) ✅
├─ Creates nodes from URLs ✅
├─ Reuses processes per origin ✅
└─ Adds: P2P sync (Phase 3, new)
```

**Verdict**: Graphshell's architecture is sound. Servo provides 80% of what we need. Focus on graph logic + P2P, not reimplementing Firefox.
