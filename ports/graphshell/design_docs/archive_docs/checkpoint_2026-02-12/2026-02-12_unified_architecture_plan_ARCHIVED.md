# Unified Architecture Plan (2026-02-12) — ARCHIVED

> **⚠️ ARCHIVED: Alternative Approach Not Pursued**
>
> **Status**: This document represents an alternative architectural approach 
> that was **NOT PURSUED**. It is archived for historical reference.
>
> **Why archived**: This proposal was created before verifying the user's 
> actual requirements. The user explicitly requested an **egui_tiles-based 
> tiling approach** with separate viewports for graph and webviews, not a 
> continuous zoom-based model.
>
> **Superseded by**:
> - [2026-02-12_architecture_reconciliation.md](../../graphshell_docs/implementation_strategy/2026-02-12_architecture_reconciliation.md) — Documents why this approach was not pursued
> - [2026-02-12_servoshell_inheritance_analysis.md](../../graphshell_docs/implementation_strategy/2026-02-12_servoshell_inheritance_analysis.md) — Describes the egui_tiles approach that IS being pursued
>
> **Key differences from pursued approach**:
> - This doc: Continuous zoom canvas where webviews render as textures ON nodes
> - Actual approach: egui_tiles tiling where webviews are separate panes alongside graph
>
> **Historical value**: This document contains useful analysis of servoshell's 
> constraints and Servo rendering capabilities, even though the proposed solution 
> differs from what was requested.
>
> ---

## Problem Statement

Graphshell's UI was built by forking servoshell's `Gui` and grafting a
`View::Graph` / `View::Detail` enum on top. This approach inherited
servoshell's assumptions — a single window with a tab bar, where exactly
one webview renders full-screen below the toolbar — and then fought them
at every turn.

### Concrete symptoms

| Symptom | Root Cause |
|---------|-----------|
| Webviews destroyed/recreated on every view toggle | Servoshell assumes the active tab's webview owns the entire viewport below the toolbar. We can't composite webview content into graph nodes within that model. |
| Two parallel identity systems (WebViewCollection tabs vs Graph nodes) bridged by a fragile bidirectional HashMap | Servoshell's tab model and our graph model are separate data structures that must be kept in sync. |
| `active_webview_nodes` save/restore hack | Direct consequence of destroying webviews on graph-view entry. |
| Phantom node creation on navigation | `sync_to_graph` infers graph mutations from webview URL changes reactively instead of the graph being the authoritative source. |
| Tab bar hidden in graph view, shown in detail view | We toggle servoshell's UI elements rather than replacing them. |
| Force-consuming all input events in graph view | Prevents invisible webviews from receiving events, but is a band-aid. |
| Address bar has split codepath (graph vs detail) | Because the toolbar was designed for one purpose and we repurposed it for two. |

### What "restricted by instead of enabled by" means

Servoshell is a **reference browser shell** — it demonstrates Servo's
embedding API with a minimal traditional browser UI. Graphshell is a
**spatial browser** where the primary interface IS the graph. By
inheriting servoshell's frame loop and extending it with conditional
branches, we've built a spatial browser that's embarrassed to show its
graph — it has to tear down all the browser parts first.

## Design Principles

1. **The graph IS the UI.** There is no separate "graph view" vs
   "detail view." There is one continuous spatial canvas, and you can
   zoom into a node to see its web content.
2. **A node IS a tab.** No bidirectional mapping. The graph node is the
   single source of truth. A `WebViewId` is a property of a node, not a
   parallel identity.
3. **Webview lifecycle follows the viewport.** Nodes near the camera get
   webviews created (warm up); nodes far away get webviews reclaimed
   (cool down). No bulk destroy/recreate.
4. **The graph owns navigation.** When a link is clicked, the graph
   decides what happens — create a new node, follow an edge to an
   existing node, etc. Navigation is a graph operation, not a webview
   operation that we try to reverse-engineer.
5. **Servo's embedding API is our foundation, not servoshell's Gui.**
   We use `WebView`, `WebViewBuilder`, `RenderingContext` directly, not
   through layers of servoshell UI code.

## Architectural Overview

```
┌─────────────────────────────────────────────────────┐
│                    egui frame                       │
│  ┌───────────────────────────────────────────────┐  │
│  │              Spatial Canvas                   │  │
│  │                                               │  │
│  │   [node]──────[node]       [node]             │  │
│  │      │          │                             │  │
│  │   [node]     [FOCUSED NODE]                   │  │
│  │              ┌──────────────┐                 │  │
│  │              │  webview     │                 │  │
│  │              │  rendered    │                 │  │
│  │              │  as texture  │                 │  │
│  │              └──────────────┘                 │  │
│  │                                               │  │
│  └───────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────┐  │
│  │  Command Bar (universal, context-sensitive)   │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Single continuous canvas

Instead of two exclusive rendering paths, there is one spatial canvas.
The camera has a position and zoom level. At far zoom, you see the full
graph with nodes as icons/thumbnails. As you zoom into a node, the
node's web content becomes visible (rendered as a texture by Servo's
offscreen rendering). At full zoom, the web content fills the viewport
and you're effectively "inside" the node.

### Zoom-based webview lifecycle

The viewport determines which nodes are "warm" (close enough to need a
webview) vs "cold" (too far away, metadata only):

```
Zoom level / distance from camera center
─────────────────────────────────────────
  Active   │  node fills viewport, receives input
  Warm     │  webview running, rendered as texture on node
  Cold     │  thumbnail/favicon only, no webview process
```

This replaces the binary toggle between Graph and Detail views.
Webviews are never bulk-destroyed — they're created and reclaimed based
on proximity to the camera, with hysteresis to prevent thrashing.

### Unified node model

```rust
pub struct Node {
    // Identity
    pub url: String,
    pub title: String,

    // Spatial
    pub position: Point2D<f32>,
    pub velocity: Vector2D<f32>,

    // Webview (owned, not mapped)
    pub webview: Option<WebView>,      // None when cold
    pub thumbnail: Option<Texture>,     // Cached render for cold display

    // Visual + interaction
    pub is_selected: bool,
    pub is_pinned: bool,

    // Lifecycle managed by viewport proximity
    pub thermal_state: ThermalState,  // Active / Warm / Cold
}

pub enum ThermalState {
    /// Web content fills viewport, node receives keyboard/mouse input
    Active,
    /// Webview running, content rendered as texture on graph node
    Warm,
    /// No webview process, display thumbnail/favicon/placeholder
    Cold,
}
```

Key change: `Node` **owns** its `Option<WebView>` directly. No
bidirectional HashMap. No `WebViewCollection` tab ordering. The graph is
the tab manager.

### Navigation as graph operation

When a user clicks a link in a warm/active node's web content:

1. Servo fires a navigation event on that node's `WebView`.
2. Graphshell intercepts it via the embedder delegate.
3. The graph decides: create a new child node with the target URL and
   an edge from the current node, OR navigate the existing node
   (user preference / link type).
4. A new node is created in the graph at a position offset from the
   parent.
5. Camera optionally animates to the new node.

This is the inverse of the current approach where we detect URL changes
after the fact in `sync_to_graph`.

### Command bar replaces toolbar

The address bar, back/forward buttons, and view-toggle button are all
artifacts of servoshell's traditional browser UI. Replace them with a
single command bar (Ctrl+L or always-visible strip):

- **Type a URL** → create a new node or navigate the focused node
- **Type a search query** → search within the graph or web search
- **Graph commands** → pin, delete, connect, etc.

Back/forward become graph-level operations (move focus to parent/child
node along history edges).

## Migration Strategy

This is a significant refactor. Suggested phasing:

### Phase A: Render webview as texture on node (unblock everything else)

**Goal**: Prove the core concept works — a Servo WebView rendered as an
egui texture on a graph node.

1. Use Servo's offscreen rendering to capture webview content to a
   texture.
2. Display that texture as the node's face in egui_graphs custom node
   rendering (we already have `GraphNodeShape` infrastructure from
   favicon work).
3. Keep the rest of the architecture as-is temporarily — just prove the
   rendering pipeline.

**This is the critical path.** If we can render a webview as a texture
on a node, everything else follows.

### Phase B: Viewport-based lifecycle

1. Replace `View` enum with continuous camera model.
2. Implement thermal state transitions based on viewport proximity.
3. Node creates/destroys its own webview based on thermal state.
4. Remove `active_webview_nodes`, `webview_to_node`/`node_to_webview`
   maps.

### Phase C: Navigation interception

1. Implement embedder delegate that intercepts navigation requests.
2. Route link clicks through graph operations (new node + edge).
3. Remove `sync_to_graph` polling mechanism.
4. Remove `webview_previous_url` tracking.

### Phase D: Command bar and input

1. Replace toolbar with command bar.
2. Unify input handling (no more graph-view event suppression).
3. Implement graph-native back/forward (edge traversal).

## Key Technical Questions

### Can Servo render to an offscreen texture that egui can display?

Servo supports `OffscreenRenderingContext`. The question is whether we
can get the rendered output as an OpenGL texture that egui can composite.
Servoshell already uses `render_to_parent_callback` with a glow
`PaintCallback`. We need the equivalent but targeting a texture instead
of the parent window's framebuffer.

Options:
- **Framebuffer Object (FBO)**: Render each webview to its own FBO,
  then use the color attachment texture in egui. This is the standard
  approach.
- **ReadPixels path**: Render → readback → upload as egui texture. Works
  but slow (GPU→CPU→GPU round-trip). Acceptable as first pass.
- **Shared texture**: If Servo and egui share the same GL context, we
  may be able to use the texture directly. Needs investigation.

### How many simultaneous webviews can Servo handle?

Each webview has a script thread and compositor resources. We need to
measure the practical limit. The thermal state system should keep the
warm count small (3-5 nodes), with only 1 active at a time.

### Does egui_graphs support variable-size nodes?

For the zoomed-in webview rendering, nodes need to be larger. We already
have custom `DisplayNode` via `GraphNodeShape`. The size can be
viewport-dependent — small circles when zoomed out, expanding to show
web content as you zoom in.

## What We Keep

- **petgraph StableGraph** — data structure for the graph
- **egui_graphs** — rendering and interaction for the spatial canvas
- **Physics engine** — force-directed layout (already on worker thread)
- **Persistence** — fjall log + redb snapshots
- **Graph, Node, Edge types** — core data model (extended, not replaced)
- **Custom node rendering** (`GraphNodeShape`) — already supports
  favicon; extends naturally to thumbnails and webview textures

## What We Remove / Replace

- **`View` enum** (Graph vs Detail) → continuous zoom model
- **`webview_to_node` / `node_to_webview` HashMaps** → `Node` owns its
  `Option<WebView>`
- **`active_webview_nodes`** → thermal state manages lifecycle
- **`webview_controller.rs`** (`manage_lifecycle`, `sync_to_graph`) →
  viewport-based lifecycle manager
- **`webview_previous_url` tracking** → navigation interception
- **Servoshell tab bar** → command bar or omit
- **`on_window_event` input suppression** → unified spatial input
- **`browser_tab()` widget** → nodes are tabs
- **Toolbar back/forward/reload** → graph-native navigation

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| Servo offscreen rendering may not produce textures egui can use | Phase A is a focused spike to prove this works before committing to full refactor |
| Performance with multiple warm webviews | Thermal state limits concurrent webviews; measure early |
| Loss of traditional browser UX for users who expect tabs | The command bar and zoom interaction need to feel natural; consider an optional sidebar node list |
| Large refactor surface | Phased approach — each phase is functional on its own |
| Upstream servoshell changes become harder to merge | We're already divergent; this makes the divergence intentional and clean rather than accidental |

## Outputs

- This plan document
- Phase A spike proving webview→texture→node rendering pipeline
- Incremental PRs for Phases A through D
- Updated CODEBASE_MAP and DEVELOPER_GUIDE after major milestones
