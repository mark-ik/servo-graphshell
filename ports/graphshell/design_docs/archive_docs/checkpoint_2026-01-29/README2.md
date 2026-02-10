# Graphshell

Graphshell is an experimental browser built on [Servo](https://servo.org/), using Rust and WebRender for rendering.

## Current State

Graphshell is currently a **tab-based browser** with support for:
- Multiple tabs and windows via Winit event loop
- WebRender-based rendering with Servo integration
- Download management
- Configuration system
- Clipboard support (via arboard)
- Keyboard and touch input handling

**Planned**: Migration to a force-directed graph canvas interface for spatial browsing (see [design_docs/](design_docs/) for research and specifications).

## Quick Start

### Windows 11 Setup

**See [WINDOWS_BUILD.md](WINDOWS_BUILD.md) for detailed step-by-step instructions.**

TL;DR:
1. Install Visual Studio 2022 Build Tools with C++ workload
2. Install Python 3.8+, Perl, Git (add all to PATH)
3. Install Rust from https://rustup.rs/
4. Install MozillaBuild from https://wiki.mozilla.org/MozillaBuild
5. Open MozillaBuild Terminal: `C:\mozilla-build\start-shell.bat`
6. Build:
   ```bash
   cd /c/path/to/graphshell
   cargo build --release
   ./target/release/graphshell.exe
   ```

### Linux / macOS Setup

```bash
git clone https://github.com/markik/graphshell
cd graphshell
cargo build --release
./target/release/graphshell
```

### Alternative: Nix Shell (Linux/macOS)

If you have Nix installed:
```bash
nix-shell
cargo build --release
./target/release/graphshell
```

## Requirements

- **Rust** (see [rust-toolchain.toml](rust-toolchain.toml)) — Latest stable recommended
- **Platform tooling** for Servo builds:
  - **Windows 11**: MozillaBuild (see [Windows 11 Setup](#windows-11-setup) above)
  - **Linux**: `build-essential`, Python 3.8+, Perl
  - **macOS**: Xcode Command Line Tools
- **Python 3.8+** in PATH
- **Perl** in PATH (for Servo build scripts)

## Architecture

### Core Components

- **[src/main.rs](src/main.rs)**: Winit-based event loop with ApplicationHandler
- **[src/graphshell.rs](src/graphshell.rs)**: Main Graphshell struct integrating Servo constellation, compositor, and webview pool
- **[src/compositor.rs](src/compositor.rs)**: Rendering coordination with WebRender and display lists
- **[src/window.rs](src/window.rs)**: Window management and event handling
- **[src/tab.rs](src/tab.rs)**: Tab data structures
- **[src/webview/](src/webview/)**: WebView embedding and context menu handling
  - [context_menu.rs](src/webview/context_menu.rs): Right-click menu
  - [webview.rs](src/webview/webview.rs): WebView lifecycle management
  - [prompt.rs](src/webview/prompt.rs): Alert/prompt dialogs
- **[src/download.rs](src/download.rs)**: Download manager
- **[src/storage.rs](src/storage.rs)**: Persistence layer
- **[src/config.rs](src/config.rs)**: Configuration management
- **[src/keyboard.rs](src/keyboard.rs)**: Keyboard input handling
- **[src/touch.rs](src/touch.rs)**: Touch input handling
- **[src/rendering.rs](src/rendering.rs)**: Rendering utilities
- **[src/errors.rs](src/errors.rs)**: Error types

### Crates

- **graphshell** (library): Builder pattern and public API ([graphshell/src/main.rs](graphshell/src/main.rs) demonstrates usage)
- **graphshellview_messages**: IPC message types for webview communication
- **graphshellview_build**: Build support utilities

### Dependencies

Key external dependencies:
- **Servo**: constellation (tab/pipeline management), compositor, script, layout, canvas, webrender (from `servo/main` branch)
- **Winit**: Window creation and event loop
- **crossbeam**: Channel-based concurrency
- **ipc-channel**: Inter-process communication
- **arboard**: Clipboard access
- **serde**: Serialization/deserialization

## Troubleshooting

### Windows: "MozTools or MozillaBuild not found"
- Ensure you're running in **MozillaBuild Terminal** (`start-shell.bat`), not regular Command Prompt
- Verify MozillaBuild installation in `C:\mozilla-build`
- Check that Perl and Python are in your PATH within the MozillaBuild environment

### Windows: "Python not found"
- Install Python 3.8+ from https://www.python.org
- Add Python to system PATH
- Verify in MozillaBuild Terminal: `python --version`

### Compilation hangs
- Servo builds are large and may take 10–15 minutes on first build
- Subsequent builds are faster (incremental compilation)
- Monitor system resources; Servo uses significant RAM (8GB+ recommended)

## Design Documents

Research, specifications, and future roadmap:
- [design_docs/GRAPH_INTERFACE.md](design_docs/GRAPH_INTERFACE.md) — Interaction model for planned graph canvas
- [design_docs/GRAPH_BROWSER_MIGRATION.md](design_docs/GRAPH_BROWSER_MIGRATION.md) — Migration plan from tabs to graph
- [design_docs/PROJECT_PHILOSOPHY.md](design_docs/PROJECT_PHILOSOPHY.md) — Vision and design principles
- [design_docs/verse_docs/VERSE.md](design_docs/verse_docs/VERSE.md) — Phase 3+ tokenization and P2P research
- [design_docs/](design_docs/) — Full archive of research and specifications

## Contributing

See [.github/CONTRIBUTING.md](.github/CONTRIBUTING.md) and [.github/CODE_OF_CONDUCT.md](.github/CODE_OF_CONDUCT.md).

## License

Dual-licensed: MIT or Apache-2.0

## References

- [Servo Browser Engine](https://servo.org/)
- [WebRender](https://github.com/servo/webrender)
- [Winit](https://github.com/rust-windowing/winit)
- [Servo on Servo Wiki](https://github.com/servo/servo/wiki)

