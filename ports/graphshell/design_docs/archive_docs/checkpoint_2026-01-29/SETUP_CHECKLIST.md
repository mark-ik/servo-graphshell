# Getting Graphshell Running on Windows 11 - Checklist

## Phase 1: Install Prerequisites (Do Once)

### Visual Studio 2022 Build Tools
- [ ] Download from https://visualstudio.microsoft.com/downloads/
- [ ] Run installer
- [ ] Select "Desktop development with C++"
- [ ] Select C++ v143 or later
- [ ] Install Windows 11 SDK
- [ ] Install CMake tools
- [ ] Complete installation (~2 hours)
- [ ] Verify: `cl.exe` works in terminal (C++ compiler)

### Python 3.8+
- [ ] Download from https://www.python.org/downloads/
- [ ] Run installer
- [ ] **CHECK: "Add Python to PATH"**
- [ ] Click "Install Now"
- [ ] Complete installation (~5 minutes)
- [ ] Verify: `python --version` in terminal

### Perl
- [ ] Download ActivePerl from https://www.activestate.com/products/perl/
- [ ] Run installer
- [ ] Accept defaults
- [ ] Ensure "Add Perl to PATH" checked
- [ ] Complete installation (~5 minutes)
- [ ] Verify: `perl -v` in terminal

### Git
- [ ] Download from https://git-scm.com/download/win
- [ ] Run installer
- [ ] Use default settings
- [ ] Ensure "Git from command line" selected
- [ ] Complete installation (~5 minutes)
- [ ] Verify: `git --version` in terminal

### Rust
- [ ] Download from https://rustup.rs/
- [ ] Run installer
- [ ] Accept default settings
- [ ] Complete installation (~5 minutes)
- [ ] Verify: `rustc --version` in terminal
- [ ] Rust should show **1.86.0** (auto-detected from graphshell)

### MozillaBuild (CRITICAL FOR GRAPHSHELL)
- [ ] Download from https://wiki.mozilla.org/MozillaBuild
- [ ] Run installer
- [ ] **IMPORTANT: Install to `C:\mozilla-build`** (default location, must not have spaces)
- [ ] DO NOT choose custom path
- [ ] Complete installation (~10 minutes)
- [ ] Verify: File `C:\mozilla-build\start-shell.bat` exists

**Total setup time: ~3–4 hours** (mostly waiting for Visual Studio to download)

---

## Phase 2: Build Graphshell

### Clone Repository
- [ ] Open Command Prompt or PowerShell
- [ ] Choose build location (e.g., `C:\Users\YourName\Projects`)
- [ ] Run: `git clone https://github.com/markik/graphshell`
- [ ] Navigate: `cd graphshell`

### Build in MozillaBuild Terminal
- [ ] **CRITICAL**: Open **MozillaBuild Terminal**: Run `C:\mozilla-build\start-shell.bat`
- [ ] NOT regular Command Prompt or PowerShell
- [ ] Navigate: `cd /c/Users/YourName/Projects/graphshell`
- [ ] Run: `cargo build --release`
- [ ] **Wait 15–20 minutes** (first build compiles entire Servo)
- [ ] Watch for: "Finished release" message at end
- [ ] Verify: `target/release/graphshell.exe` exists

**Build time: 15–20 minutes** (first time), 2–5 minutes (subsequent)

---

## Phase 3: Test Graphshell

### Launch Graphshell
- [ ] Still in MozillaBuild Terminal
- [ ] Run: `./target/release/graphshell.exe`
- [ ] Window should appear with address bar

### Basic Tests
- [ ] Window opens without crash ✅
- [ ] Can see address bar ✅
- [ ] Can type URL (e.g., `https://example.com`) ✅
- [ ] Press Enter → page loads ✅
- [ ] Can see rendered webpage ✅
- [ ] Can open new tab ✅
- [ ] Can close tabs ✅
- [ ] Can click links on webpage ✅
- [ ] Can close application with X button ✅

### If Tests Pass
- [ ] ✅ Graphshell is working!
- [ ] Proceed to [design_docs/GRAPH_BROWSER_MIGRATION.md](design_docs/GRAPH_BROWSER_MIGRATION.md) for planned UI changes
- [ ] Or explore codebase and make improvements

### If Tests Fail
- [ ] Note the error message
- [ ] Check [WINDOWS_BUILD.md#troubleshooting](WINDOWS_BUILD.md#troubleshooting)
- [ ] Try the suggested fixes
- [ ] If still stuck, search [Servo GitHub Issues](https://github.com/servo/servo/issues)

---

## Phase 4: Future Development

Once Graphshell runs successfully:

### Option A: Contribute to Graph Canvas
- [ ] Read [design_docs/GRAPH_BROWSER_MIGRATION.md](design_docs/GRAPH_BROWSER_MIGRATION.md)
- [ ] Start with Phase 1 (graph engine)
- [ ] Implement force-directed layout

### Option B: Fix Bugs / Add Features
- [ ] Run: `cargo build` (debug, faster compile)
- [ ] Make changes to `src/`
- [ ] Re-run: `cargo build`
- [ ] Test changes
- [ ] Commit to git

### Option C: Explore Codebase
- [ ] Read [README.md](README.md) architecture section
- [ ] Explore `src/graphshell.rs` (main logic)
- [ ] Explore `src/compositor.rs` (rendering)
- [ ] Explore `src/webview/` (webpage embedding)

---

## Helpful Documents

- **[WINDOWS_BUILD.md](WINDOWS_BUILD.md)** — Detailed setup (read if stuck)
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** — One-page reference
- **[BUILD_SETUP_SUMMARY.md](BUILD_SETUP_SUMMARY.md)** — Technical summary
- **[README.md](README.md)** — Current project status
- **[design_docs/](design_docs/)** — Future roadmap

---

## Troubleshooting Quick Links

| Error | Solution |
|-------|----------|
| "MozTools not found" | Use MozillaBuild Terminal (`start-shell.bat`), not CMD |
| "Python not found" | Install Python, add to PATH, restart terminal |
| "Build hangs >30 min" | Normal for first build; check disk space (20GB free) |
| "Link error" | Install Visual Studio 2022 Build Tools (C++) |
| "Permission denied" | Run Command Prompt as Administrator for Git/Python install |

**Full guide**: [WINDOWS_BUILD.md#troubleshooting](WINDOWS_BUILD.md#troubleshooting)

---

## Success Criteria

✅ You can run: `./target/release/graphshell.exe`  
✅ Window appears with address bar  
✅ Can navigate to websites  
✅ Can open/close tabs  
✅ Application is stable (no crashes)  

If all above are true → **You have a working Graphshell browser!**

---

**Estimated total time**: 4–5 hours (mostly waiting for prerequisites to download/install)
