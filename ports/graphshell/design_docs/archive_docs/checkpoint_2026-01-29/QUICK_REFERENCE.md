# Graphshell Windows 11 - Quick Build Reference

## Prerequisites Checklist

- [ ] Visual Studio 2022 Build Tools (C++ workload)
- [ ] Python 3.8+ (in PATH)
- [ ] Perl (in PATH)  
- [ ] Git (in PATH)
- [ ] Rust 1.86.0 (from rustup.rs)
- [ ] MozillaBuild (installed to `C:\mozilla-build`)

**Full instructions**: See [WINDOWS_BUILD.md](WINDOWS_BUILD.md)

## Build Steps

### Step 1: Open MozillaBuild Terminal
```cmd
C:\mozilla-build\start-shell.bat
```

### Step 2: Navigate to Graphshell
```bash
cd /c/path/to/graphshell
```

### Step 3: Build
```bash
cargo build --release
```

### Step 4: Run
```bash
./target/release/graphshell.exe
```

## Expected Time
- **First build**: 15–20 minutes (entire Servo stack)
- **Subsequent builds**: 2–5 minutes (incremental)

## Troubleshooting

| Issue | Fix |
|-------|-----|
| "MozTools not found" | Use MozillaBuild Terminal (`start-shell.bat`), not CMD |
| "Python not found" | Install Python, add to PATH, restart terminal |
| "Build hangs" | Normal on first build; may take 20+ minutes. Check disk space (need 20GB free). |
| "Link error" | Ensure Visual Studio 2022 Build Tools with C++ installed |

**Full troubleshooting**: See [WINDOWS_BUILD.md](WINDOWS_BUILD.md#troubleshooting)

## Development Commands

```bash
# Debug build (faster compile, slower runtime)
cargo build

# Release build (slower compile, optimized runtime)
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Lint code
cargo clippy

# Clean build artifacts
cargo clean
```

## Architecture

**Current**: Tab-based browser with Servo integration  
**Planned**: Force-directed graph canvas (see [design_docs/](design_docs/))

### Core Files
- `src/main.rs` — Event loop entry point
- `src/graphshell.rs` — Main browser logic (1200 lines)
- `src/compositor.rs` — Rendering coordinator
- `src/window.rs` — Window management
- `src/webview/` — WebView embedding

## After Building

Once `graphshell.exe` runs:

1. ✅ Type URL in address bar
2. ✅ Navigate to websites
3. ✅ Open/close tabs
4. ✅ Use back/forward buttons
5. ✅ Download files

See [design_docs/](design_docs/) for planned UI migration to spatial graph.

---

**More help?** → [WINDOWS_BUILD.md](WINDOWS_BUILD.md)  
**Setup summary?** → [BUILD_SETUP_SUMMARY.md](BUILD_SETUP_SUMMARY.md)  
**Main docs?** → [README.md](README.md)
