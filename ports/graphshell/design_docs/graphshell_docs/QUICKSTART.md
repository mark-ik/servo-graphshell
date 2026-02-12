# Quick Start: Building and Testing Graphshell

## Build Instructions

### Debug Build (Faster)
```bash
cd c:\Users\mark_\Code\servo
./mach build -d graphshell
```

### Release Build (Optimized)
```bash
./mach build -r graphshell
```

## Running Graphshell

### Start with Graph View
```bash
./mach run -r graphshell -- https://example.com
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
   ./mach run -r graphshell -- -M https://example.com https://example.org https://google.com
   ```

2. In graph view:
   - Observe force-directed layout converges
   - Click nodes to select them (orange glow)
   - Double-click to switch webview focus
   - Navigate with back/forward to add more nodes

3. Check visual feedback:
   - Active node (blue)
   - Selected node (orange with glow)
   - Inactive nodes (gray)
   - Edge colors vary by type

## Debugging

### Enable Verbose Output
```bash
RUST_LOG=debug ./mach run -r graphshell -- -M https://example.com
```

### Check Compilation Issues
```bash
cd c:\Users\mark_\Code\servo\ports\graphshell
cargo check
```

## Common Issues

### Build Hangs
- Run `./mach clean` to reset build state
- Check disk space (Servo builds are large)

### Webview Not Appearing
- Verify `-M` flag is passed for multiprocess
- Check terminal output for errors
- Try simpler URL: `https://example.com`

### Graph Not Rendering
- Verify GPU drivers are up to date
- Check egui version in Cargo.toml (should be 0.33.3)
- Look for error output in terminal

## Documentation Files

- [README.md](README.md) - Project overview
- [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) - Feature targets and validation tests
- [ARCHITECTURAL_OVERVIEW.md](ARCHITECTURAL_OVERVIEW.md) - Architecture decisions and code map
