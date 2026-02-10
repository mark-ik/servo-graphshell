# Quick Start: Building and Testing Graphshell

## Build Instructions

### Debug Build (Faster)
```bash
cd c:\Users\mark_\Code\servo
cargo build -p graphshell
```
**Time:** ~3-4 minutes  
**Output:** `target/debug/graphshell` (~500MB)

### Release Build (Optimized)
```bash
cargo build -p graphshell --release
```
**Time:** ~8-12 minutes  
**Output:** `target/release/graphshell` (~50MB, much faster runtime)

## Running Graphshell

### Start with Graph View
```bash
./target/release/graphshell -M https://example.com
```

Flags:
- `-M`: Enable multiprocess (each origin in separate process)
- URL: Initial page to load

### Graph View Controls

| Action | Result |
|--------|--------|
| Click node | Select (visual feedback) |
| Double-click node | Activate webview |
| Drag canvas | Pan camera |
| Click 🕸 button | Toggle between graph/detail view |
| New Node (⊞) | Add webview node |

## Testing Scenario

1. Start with 3-4 nodes open:
   ```bash
   ./target/release/servo -M https://example.com https://example.org https://google.com
   ```

2. In graph view:
   - Observe force-directed layout converges
   - Click nodes to select them (orange glow)
   - Double-click to switch webview focus
   - Click 🕸 to see cluster strip appear
   - Navigate with back/forward to add more nodes

3. Check visual feedback:
   - Active node (blue)
   - Selected node (orange with glow)
   - Inactive nodes (gray)
   - Edge colors vary by type

## Debugging

### Enable Verbose Output
```bash
RUST_LOG=debug ./target/release/servo -M
```

### Check Compilation Issues
```bash
cd c:\Users\mark_\Code\servo\ports\graphshell
cargo check
```

### View Recent Commits
```bash
git log --oneline graph-browser-canvas -10
```

### Check Current Branch
```bash
git branch -v
```

## Common Issues

### Build Hangs
- Run `cargo clean` to reset build state
- Check disk space (Servo is large ~50GB total)

### Webview Not Appearing
- Verify `-M` flag is passed for multiprocess
- Check terminal output for errors
- Try simpler URL: `https://example.com`

### Graph Not Rendering
- Verify GPU drivers are up to date
- Check egui version in Cargo.toml (should be 0.33.3)
- Look for error output in terminal

## Performance Profiling

### Frame Time
- Graph physics: ~1ms
- egui rendering: ~5-10ms
- WebRender composition: ~5-15ms
- Total: ~30-40ms (25-33 FPS target)

### Memory Usage
- Per node: ~500 bytes
- Servo itself: ~200-300MB
- Graph canvas: ~1-5MB for 100+ nodes

## Next Development Steps

1. **Add Node Labels** (Week 4)
   - Display page title in each node
   - Use egui text rendering

2. **Zoom Support** (Week 5)
   - Scroll wheel handler
   - Keyboard shortcuts (Ctrl +/-)

3. **Detail View** (Week 6-7)
   - Split screen with graph on left, webview on right
   - Switch between full graph and detail mode

4. **History Edges** (Week 8)
   - Track browser history
   - Create edges for navigation chains

## Useful Git Commands

```bash
# View current branch
git branch

# Switch to graph branch
git checkout graph-browser-canvas

# View commit history
git log --oneline

# See what changed
git diff main graph-browser-canvas

# Sync with upstream
git fetch origin
git merge origin/main
```

## Documentation Files

- `PHASE1_PROGRESS.md` - This session's accomplishments
- `IMPLEMENTATION_ROADMAP.md` - Full 24-week plan
- `ARCHITECTURE_DECISIONS.md` - Why we chose servoshell
- `design_docs/` - All design documentation

---

**Ready to build and test!** Run:
```bash
cd c:\Users\mark_\Code\servo
cargo build -p servoshell --release
./target/release/servo -M https://example.com
```
