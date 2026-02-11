# GraphShell Keyboard Shortcuts

## View Navigation
- **Home** / **Escape**: Toggle between Graph and Detail views
- **C**: Center and fit all nodes to screen (Graph view only)

## Node Management  
- **N**: Create new node at random position near graph center
  - Newly created node is automatically selected (turns gold)
  - Type URL in omnibar and press Enter to navigate the node
  - A tab/webview will be created for the node
- **Delete** / **Backspace**: Remove selected nodes
  - Also closes associated tabs/webviews
  - Works with multiple selection

## Node Selection
- **Click**: Select single node (turns gold)
- **Shift+Click**: Add/remove from selection  
- Selected nodes can be navigated via omnibar

## Physics Controls
- **T**: Toggle physics simulation on/off
- **P**: Show/hide physics configuration panel

## Mouse Controls (Graph View)
- **Click & Drag node**: Move node position
- **Scroll wheel**: Zoom in/out
- **Middle-click drag**: Pan view
- **Double-click node**: Switch to Detail view focused on that node

## Navigation Workflow

### Creating and Navigating New Nodes
1. Press **N** to create new node (appears as gray "about:blank")
2. Node is auto-selected (gold highlight)
3. Type URL in omnibar (top toolbar)
4. Press **Enter**
5. Node URL updates, tab/webview created, node turns blue (active)
6. Press **Home** to toggle to Detail view and see rendered page

### Initial Tab = Initial Node
- On startup, one node is created for the starting URL
- The initial webview/tab is automatically mapped to this node
- Tabs and nodes maintain 1:1 correspondence

### Navigation in Graph View
- Select a node (click it)
- Type new URL in omnibar and press Enter
- The selected node's webview navigates to new URL
- If node has no webview yet, one is created

### Removing Nodes
- Select node(s) in Graph view
- Press **Delete** or **Backspace**
- Node removed from graph, tab closed

## Tips
- Nodes start as "Cold" (gray, no webview)
- Typing URL for selected node promotes it to "Active" (blue, has webview)
- Physics simulation helps organize nodes spatially
- Graph persists across restarts (saved to disk)
