## Graph interface and interaction model

Purpose
- Define the interaction model, physics presets, rendering and camera expectations, and short implementation notes for the graph canvas.

Interaction model
- Single left click: select node (Shift+click for multi-select; drag to marquee-select).
- Double left click: open node in a detail window.
- Right click: context menu (create edge, delete, pin, inspect).
- Pan: WASD, arrow keys or middle-mouse drag. 
- Zoom: mouse wheel or +/- keys.

Physics presets
- Liquid (default): strong repulsion, low spring, low damping — organic clustering.
- Solid: moderate repulsion, high spring, high damping — stable manual layouts.
- Gas: large-radius repulsion, very low spring, very low damping — sparse layouts.

Rendering and camera
- Renderer: wgpu-based GPU renderer for nodes, edges, labels, and picking.
- Camera: smooth pan/zoom transforms, bounds clamping, and LOD thresholds that aggregate nodes at zoomed-out levels.

UI patterns and windows
- Detail windows: show a live webview texture for the selected node; connection tabs list adjacent nodes and support in-window navigation.
- Sidebar (Phase 2): session manager, open nodes list, filters/tags, and graph statistics.

Keybindings (core)
- `N`: new node
- `R`: delete selected
- `T`: toggle physics
- `Ctrl+S`: save graph
- `Ctrl+O`: open graph
- `Ctrl+F`: search

Implementation notes
- Phase 1 (MVP) keeps everything in a single application crate; design `graph` and `camera` modules so they do not depend on Servo or any UI types.
- Reuse the existing Servo/WebRender compositor to render a 2D graph canvas; defer a dedicated `wgpu` renderer to Phase 2+.
- Start with a simple UI layer (e.g., egui overlay) integrated directly into the app; a formal `UIBackend` trait and multiple UI implementations are Phase 2+ concerns.
- Treat `graphshell-graph-core` as a **target** architecture: after the MVP is stable, extract the Servo-agnostic graph, physics, camera, and serialization modules into a standalone crate.
- WebView orchestration: use a **small pool** of Servo webviews bound to the focused node (and maybe a few neighbors); nodes always exist as URL/metadata, and webviews are created lazily and reused.
- Evaluate `petgraph` in an experimental branch (Phase 1.5) for algorithmic features; retain a custom implementation if tighter control is required.

Browser extensions and interoperability

- **Embedding in extensions**: Chrome/Firefox extensions embed the graph canvas via the `UIBackend` trait. A minimal extension crate reuses `graphshell-ui` and `graphshell-graph-core`, rendering the canvas in a pop-up or new tab.
- **History import and export**: Graphshell exports graphs as JSON with node metadata (URL, title, timestamp, tags, edges). A bridge module maps browser history APIs (Chrome History, Firefox Places) to Graphshell node format for import. Export is portable to other tools.
- **Portable node format**: Nodes store source URL, favicon, selectors, and custom metadata. Extensions can parse Graphshell JSON, ensuring data portability across applications.
- **WebView backend for extensions**: Extensions use Tao+Wry (lighter than Servo) via the `BrowserEngine` trait. Desktop Graphshell uses Servo; the trait decouples implementation choice.

Notes
- Store concrete physics parameters in code with UI-exposed tuning controls.
- Use Phase 1.5 as an intentional validation period to refine interaction choices based on actual usage.
- Minimize extension boilerplate; most logic lives in reusable core crates.
