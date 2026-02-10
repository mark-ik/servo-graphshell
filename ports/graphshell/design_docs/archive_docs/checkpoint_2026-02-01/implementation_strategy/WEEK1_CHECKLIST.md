# Architecture Study: Feature Targets & Validation Tests

**Goal**: Complete architecture study phase. Deliverables:
- SlotMap node structure defined and validated
- Adjacency list data structures designed with O(1) neighbor lookup proven
- Physics engine prototype with convergence validated via tests
- Servo multiprocess integration model documented

---

## Feature Target 1: Understand Architecture Foundation

### Context
Study project architecture decisions and Servo's multiprocess model to establish a clear integration plan.

### Tasks
- [ ] Read ARCHITECTURE_DECISIONS.md sections 1-5 (View, Edge, Webview, Physics, Data Structures)
- [ ] Review IMPLEMENTATION_ROADMAP.md Milestone 1.1 details
- [ ] Skim Servo `constellation/pipeline.rs` (pipeline abstraction)
- [ ] Skim Servo `constellation/event_loop.rs` (origin grouping, multiprocess)
- [ ] Review servoshell `src/` structure
- [ ] Study egui 0.33.3 docs (window management, resizing)
- [ ] Examine how servoshell integrates with Servo
- [ ] **Output**: `WEEK1_SERVO_INTEGRATION.md` containing a 1-page "Servo Integration Model" diagram showing how servoshell talks to Servo

### Validation Tests
- [ ] **Test**: Explain origin-grouped processes in your own words—why can't each graph node have its own Servo process?
  - Expected explanation: Origins (not individual pages) should share processes; spawning a process per node wastes memory and defeats Servo's origin grouping.
- [ ] **Test**: Explain why lifecycle + reuse pool are needed instead of "one process per origin, forever"
  - Expected explanation: Memory pressure, startup time for new origins, need to age out unused processes.

---

## Feature Target 2: Design Core Data Structures

### Context
Define graph node and edge structures with stable handles and O(1) neighbor queries.

### Tasks
- [ ] Design SlotMap node structure:
  ```rust
  struct Node {
    id: NodeKey,
    url: String,
    title: String,
    position: Vec2,     // Physics
    velocity: Vec2,     // Physics
    is_selected: bool,
    created_at: DateTime,
    // ... other fields (10-15 total for Phase 1)
  }
  ```
- [ ] List all node fields needed for Phase 1 (no Phase 3+ extras)
- [ ] Verify SlotMap is the right choice (stable handles across deletions)
- [ ] Design Edge structure:
  ```rust
  struct Edge {
    from: NodeKey,
    to: NodeKey,
    edge_type: EdgeType,  // Hyperlink, Bookmark, History, Manual
    color: Color,
    style: LineStyle,     // Solid, Dotted, Bold, Marker
    metadata: EdgeMetadata,
  }
  ```
- [ ] Design adjacency list pattern:
  - `in_edges: Vec<EdgeKey>` (O(1) predecessors)
  - `out_edges: Vec<EdgeKey>` (O(1) successors)
- [ ] **Output**: `WEEK1_DATA_STRUCTURES.md` (node/edge/adjacency definitions with examples)

### Validation Tests
- [ ] **Test**: Data structures fit on one screen (show they're not over-engineered)
- [ ] **Test**: Prove O(1) neighbor lookup:
  - Write pseudocode: `get_neighbors(node_key) -> Vec<NodeKey>` returns in_edges + out_edges without iteration
- [ ] **Test**: No circular dependencies in data structures (verify design is acyclic)

---

## Feature Target 3: Physics Engine Prototype

### Context
Build a minimal physics engine with convergence validation and auto-pause capability.

### Tasks
- [ ] Implement spatial hash for O(n) average case force calculation:
  ```rust
  cell_size = viewport_diagonal / 4
  struct SpatialHash {
    cells: HashMap<(i32, i32), Vec<NodeKey>>,
  }
  ```
- [ ] Implement force calculation:
  - Repulsion between all nodes (O(n) brute force in Week 1)
  - Attraction along edges
  - Damping: velocity *= 0.92 per frame
- [ ] Implement auto-pause:
  - Pause at 5 seconds of low velocity (< 0.001 px/frame)
  - Verify CPU drops to 0% when paused
- [ ] **Output**: `WEEK1_PHYSICS_TEST.md` (convergence data, timing, CPU measurements)

### Validation Tests
- [ ] **Test: Convergence under 2 seconds**
  - Setup: 100 random nodes, random edges
  - Measure: Time until all velocities < 0.001 px/frame
  - Acceptance: Convergence occurs within 2 seconds
- [ ] **Test: Correct force direction**
  - Setup: Two isolated nodes, no edges
  - Verify: They repel each other (velocity away from each other)
- [ ] **Test: Attraction along edges**
  - Setup: Two nodes with edge between them
  - Verify: They attract each other (velocity toward each other)
- [ ] **Test: Auto-pause functionality**
  - Setup: Graph at rest for 5 seconds
  - Verify: Physics loop pauses, CPU drops to 0%
  - Verify: Physics resumes when new force applied

---

## Feature Target 4: Servo Integration Model

### Context
Design the connection between graph nodes, webviews, Servo processes, and Servo's origin-grouped multiprocess model.

### Tasks
- [ ] Read `servoshell/src/window.rs` or equivalent integration point
- [ ] Study how WebViewCollection currently works
- [ ] Sketch servoshell integration plan:
  - How egui window talks to Servo process
  - How to spawn Servo with `-M -S` flags (multiprocess + sandbox)
  - How to capture navigation events (user clicks a link)
- [ ] Design graph node → webview binding:
  - Node creation: User types URL → spawn Servo process for origin if not exists
  - Navigation: User clicks link in webview → create new graph node
  - Process lifecycle: Close all nodes from origin → kill process
  - Lifecycle states: Active/Warm/Cold (per ARCHITECTURE_DECISIONS section 4a)
- [ ] **Output**: `WEEK1_SERVO_INTEGRATION.md` (detailed spawning plan, event flow, process lifecycle)

### Validation Tests
- [ ] **Test: Process spawning logic**
  - Setup: Create 3 nodes from origins A, B, A (same origin twice)
  - Verify: Only 2 Servo processes spawned (one per unique origin)
- [ ] **Test: Navigation event handling**
  - Setup: User is in node A viewing origin X; clicks link to origin Y
  - Verify: New node B is created, linked to node A via hyperlink edge
- [ ] **Test: Process lifecycle**
  - Setup: 2 active nodes from origin A, 1 from origin B
  - Action: Close both nodes from origin A
  - Verify: Origin A's process terminates; origin B's process remains

---

## Feature Target 5: Architecture Review & Integration Plan

### Context
Validate all designed components against architecture decisions and prepare for implementation phase.

### Tasks
- [ ] Review all outputs:
  - WEEK1_DATA_STRUCTURES.md
  - WEEK1_PHYSICS_TEST.md
  - WEEK1_SERVO_INTEGRATION.md
- [ ] Cross-check against ARCHITECTURE_DECISIONS.md (sections 1-5)
- [ ] Identify any conflicts or gaps between design and implementation plan
- [ ] Plan next phase (Week 2 implementation):
  - Create Rust project structure
  - Stub out data structures (SlotMap, Edge, Adjacency list)
  - Stub out physics loop
  - Connect egui window to Servo
- [ ] **Output**: `WEEK1_SUMMARY.md` (architecture validated, implementation plan ready, any open questions)

### Validation Tests
- [ ] **Test: Design consistency**
  - Checklist: All architecture decisions from sections 1-5 are reflected in data structures and physicsplan?
  - Checklist: Servo integration plan aligns with origin-grouped process model?
  - Checklist: Performance expectations (200@60fps target) are achievable with this design?
- [ ] **Test: No surprises for Week 2**
  - Can you write out the top-10 Rust files that will need to be created in Week 2?
  - Can you sketch a call chain from "user creates new node" → "Servo process spawned"?

---

## Expected Outputs

Create these documents in `graphshell_docs/implementation_strategy/`:
- [ ] **WEEK1_DATA_STRUCTURES.md**: Node, Edge, and adjacency list definitions with Phase 1 field justification
- [ ] **WEEK1_PHYSICS_TEST.md**: Convergence test results, timing measurements, auto-pause validation
- [ ] **WEEK1_SERVO_INTEGRATION.md**: Servo Integration Model diagram, process spawning plan, event flow, lifecycle state machine
- [ ] **WEEK1_SUMMARY.md**: Validation checklist, architecture consistency report, Week 2 implementation plan

---

## Success Gate: Knowledge Validation

When all feature targets are complete, you should be able to answer (without looking at docs):

1. [ ] **Data Structures**: How many fields does a Phase 1 Node need? (Expected: 10-15)
2. [ ] **Data Structures**: Why SlotMap and not Vec<Node>? (Expected: Stable handles across deletions)
3. [ ] **Data Structures**: How do you find all neighbors of a node? (Expected: Adjacency list in-struct, O(1) lookup)
4. [ ] **Physics**: What's the convergence time expected for 100 nodes? (Expected: < 2 seconds)
5. [ ] **Physics**: Why auto-pause? (Expected: Save CPU when graph is stable)
6. [ ] **Servo**: How many Servo processes for 3 graph nodes from 2 unique origins? (Expected: 2 processes)
7. [ ] **Servo**: What happens when user clicks a link in a webview? (Expected: New node created, linked via hyperlink edge)
8. [ ] **Process Lifecycle**: What's the origin-grouped process model and why not one process per node? (Expected: Memory efficiency, Servo's design, origin-based isolation)

If you can answer all 8 confidently, you're ready for Week 2 implementation. 🎯
