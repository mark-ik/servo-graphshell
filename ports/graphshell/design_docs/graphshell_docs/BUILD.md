# Building Graphshell Graph Browser

This guide covers building Graphshell Graph Browser on Windows, macOS, and Linux.

Graphshell is built on Servo, a modern parallel browser engine written in Rust. It uses the `mach` build system (a Python wrapper around Cargo) to manage dependencies, compilation, and testing.

**Last Verified**: February 9, 2026 (Rust 1.91.0, Servo main)  
**Status**: Core browsing graph functional (~4,500 LOC), builds successfully  
**Port Location**: `c:\Users\mark_\Code\servo\ports\graphshell\`

---

## Quick Start (All Platforms)

### Prerequisites

1. **Git**: https://git-scm.com
2. **Rust 1.91.0+**: https://rustup.rs/
3. **Python 3.8+**: https://www.python.org
4. Platform-specific tools (see below)

### Build in 3 Steps

```bash
git clone https://github.com/servo/servo.git
cd servo
./mach bootstrap           # Install dependencies (interactive, ~5-10 min first time)
./mach build -r graphshell  # Build graphshell in release mode (~15-30 min first build)
```

See platform-specific instructions below for additional setup requirements.

---

## Platform-Specific Setup

### macOS

**System Requirements:**
- macOS 10.13+ (Intel or Apple Silicon)
- Xcode Command Line Tools
- 16GB RAM recommended (8GB minimum)
- 20GB free disk space

**Installation:**

1. **Install Xcode Command Line Tools:**
   ```bash
   xcode-select --install
   ```

2. **Install Homebrew:**
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

3. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

4. **Clone and bootstrap:**
   ```bash
   git clone https://github.com/servo/servo.git
   cd servo
   ./mach bootstrap
   ```

5. **Build:**
   ```bash
   ./mach build -r graphshell
   ```

**Running:**
```bash
./target/release/graphshell https://example.com
# or
./mach run -r graphshell -- https://example.com
```

---

### Linux (Debian/Ubuntu-based)

**System Requirements:**
- Ubuntu 18.04 LTS+ or Debian 10+
- 8GB RAM minimum (16GB recommended)
- 20GB free disk space

**Installation:**

1. **Install dependencies:**
   ```bash
   sudo apt update
   sudo apt install -y \
       curl git python3 pip build-essential \
       pkg-config libssl-dev libfontconfig1-dev \
       libfreetype6-dev libxrender-dev libxcb1-dev
   ```

2. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

3. **Clone and bootstrap:**
   ```bash
   git clone https://github.com/servo/servo.git
   cd servo
   ./mach bootstrap
   ```

4. **Build:**
   ```bash
   ./mach build -r graphshell
   ```

**Running:**
```bash
./target/release/graphshell https://example.com
```

**Note for Fedora/RHEL:**
```bash
sudo dnf install -y python3 git curl gcc-c++ pkg-config \
    openssl-devel fontconfig-devel freetype-devel \
    libxrender-devel libxcb-devel
```

---

### Windows 11/10

**System Requirements:**
- Windows 10 (21H2+) or Windows 11
- x86_64 CPU
- 16GB RAM recommended (8GB minimum)
- 25GB free disk space (builds are large)

**Installation:**

1. **Visual Studio Build Tools:**
   
   Download from: https://visualstudio.microsoft.com/downloads/
   
   Choose "Visual Studio Build Tools 2022" (or full VS 2022), and during installation select:
   - [x] Desktop development with C++
   - [x] MSVC v143 or later
   - [x] Windows 10/11 SDK (latest)
   - [x] CMake tools (optional but useful)

2. **Install Rust:**
   
   Download https://win.rustup.rs/ and run the installer. Select "Quick install via Visual Studio".
   
   Verify:
   ```cmd
   rustup --version
   rustc --version
   cargo --version
   ```

3. **Install Python and Git:**
   
   - **Python 3.8+**: https://www.python.org/downloads/ (check "Add Python to PATH")
   - **Git**: https://git-scm.com/download/win

4. **Clone and bootstrap:**
   
   Open PowerShell or Command Prompt:
   ```cmd
   git clone https://github.com/servo/servo.git
   cd servo
   .\mach bootstrap
   ```
   
   Note: `mach Bootstrap` on Windows will install additional dependencies (takes ~10 minutes on first run).

5. **Build:**
   ```cmd
   .\mach build -r graphshell
   ```

**Running:**
```cmd
.\target\release\graphshell.exe https://example.com
# or
.\mach run -r graphshell -- https://example.com
```

**Troubleshooting:**

- **"mach not found"**: Use `.\mach` (with dot-backslash) instead of `./mach`
- **"link.exe not found"**: Ensure Visual Studio Build Tools installed with C++ workload
- **"Python not found"**: Reinstall Python with "Add Python to PATH" checked
- **Build hangs**: Check disk space (need 25GB free); Servo builds are large

---

### Linux (NixOS)

**If using NixOS with Flakes:**

```bash
git clone https://github.com/servo/servo.git
cd servo
nix-shell shell.nix
./mach bootstrap
./mach build -r graphshell
```

The `shell.nix` file configures all dependencies automatically.

---

### WSL (Windows Subsystem for Linux)

WSL2 is supported; WSL1 may have performance issues.

1. **Enable WSL2:**
   ```powershell
   wsl --set-default-version 2
   ```

2. **Install Ubuntu 20.04 LTS or later:**
   ```powershell
   wsl --install -d Ubuntu
   ```

3. **Inside WSL, follow Linux instructions (above)**

4. **For GUI support (WSLg):**
   - Windows 11 has WSLg built-in
   - Windows 10: Install [WSLg Preview](https://github.com/microsoft/wslg)

**Note:** WSL adds a virtualization layer; native Windows or Linux builds are faster.

---

## Build Profiles

The `mach build` command supports different optimization levels:

| Profile | Command | Build Time | Runtime Speed | Use Case |
|---------|---------|-----------|----------------|----------|
| **debug** | `./mach build -d graphshell` | 5-10 min | Slow | Local development, debugging |
| **release** | `./mach build -r graphshell` | 15-30 min | Fast | Testing, benchmarking |
| **production** | `./mach build --prod graphshell` | 20-40 min | Fastest | Release builds, distribution |

**Recommendation:** Use `-r` (release) for development and testing.

---

## Common Build Options

```bash
# Clean build (remove artifacts)
./mach clean

# Build specific crate
./mach build -r graphshell

# Build with all features enabled
./mach build -r graphshell -f all

# Build with Address Sanitizer (debug builds only)
./mach build --with-asan

# Build with Thread Sanitizer
./mach build --with-tsan

# Get full list of options
./mach build --help
```

---

## Running Graphshell Graph Browser

After a successful build, run with:

```bash
# Direct execution
./target/release/graphshell https://example.com

# Using mach (recommended)
./mach run -r -- https://example.com

# Debug build
./mach run -d -- https://example.com

# With specific options
./mach run -r graphshell -- --help
```

### First Run

1. **Check that graphshell launches** and opens a window
2. **Type a URL** in the address bar (e.g., `https://example.com`)
3. **Press Enter** to load the page
4. **Test navigation** by clicking links
5. **Close gracefully** (Ctrl+C or close button)

---

## Development Workflow

### Typical Edit-Build-Test Cycle

```bash
# Make changes to code
nano components/servo/lib.rs

# Build (very fast for incremental changes)
./mach build -r graphshell

# Run and test
./mach run -r graphshell -- https://example.com

# Check code formatting
./mach fmt

# Run lints
./mach clippy graphshell

# Run tests
./mach test-unit graphshell
```

### Faster Iteration

Use **debug builds** during development (faster compilation, slower execution):

```bash
./mach build -d graphshell    # ~3-5 min incremental
./mach run -d graphshell -- https://example.com
```

Switch to **release builds** when benchmarking or preparing to ship:

```bash
./mach build -r graphshell
./mach run -r graphshell -- https://example.com
```

---

## Troubleshooting

### All Platforms

**Problem: "mach: command not found"**
- Ensure you're in the Servo repository root (where `mach` script is located)
- On Windows, use `.\mach` instead of `./mach`

**Problem: "Rust version mismatch"**
```bash
rustup update
rustup toolchain install 1.91.0
```

**Problem: "Out of disk space" during build**
- Servo build requires 20-25GB total space
- Check available space: `df -h` (Linux/macOS) or `dir C:\` (Windows)
- Clean old builds: `./mach clean`

**Problem: Build timeout or hangs**
- First build is slow (10-30 minutes depending on hardware)
- Check CPU/RAM usage; Servo uses significant resources
- Ensure stable internet (downloading dependencies)
- Try again; sometimes network hiccups cause hangs

---

### macOS-Specific

**Problem: "clang: error: unknown argument"**
- Update Xcode Command Line Tools: `xcode-select --install`

**Problem: "Port already in use"**
- Graphshell may be trying to bind to a reserved port
- Try: `./mach run -r graphshell -- --help` to see options

---

### Linux-Specific

**Problem: "libfontconfig not found"**
```bash
# Debian/Ubuntu
sudo apt install libfontconfig1-dev

# Fedora
sudo dnf install fontconfig-devel
```

**Problem: "Could not find native static library"**
- Run `./mach bootstrap` again to reinstall dependencies

---

### Windows-Specific

**Problem: "link.exe not found"**
- Visual Studio C++ tools not properly installed
- Check: Start > Visual Studio Installer > Click "Modify" next to Build Tools
- Ensure "MSVC v143 - VS 2022 C++ x64/x86 build tools" is checked

**Problem: "python not found" in mach**
- Python not in system PATH
- Reinstall Python 3.8+ with "Add Python to PATH" checked
- Restart PowerShell/Command Prompt after installing

**Problem: Build extremely slow**
- Antivirus scanning `target/` directory; add exception or disable temporarily
- Disk fragmentation; defragmentation may help
- Insufficient RAM; minimize background programs

---

## Build Statistics

Typical build times on modern hardware:

| Platform | Profile | First Build | Incremental |
|----------|---------|-------------|-------------|
| macOS (M1) | release | ~10 min | ~2 min |
| Linux (Ryzen 5) | release | ~15 min | ~3 min |
| Windows (i7) | release | ~20 min | ~4 min |

**Note:** First builds longer because all dependencies compile. Incremental builds are much faster.

---

## Next Steps

Once Graphshell builds successfully:

1. **Explore the architecture**: Read [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)
2. **Understand the design**: See [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)
3. **Run tests**: `./mach test-unit graphshell`
4. **Check code quality**:
   ```bash
   ./mach fmt --check graphshell
   ./mach clippy graphshell
   ```
5. **Start implementing features**: Refer to [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) for feature targets

---

## Additional Resources

- **Servo Book**: https://book.servo.org/
  - [Building Servo](https://book.servo.org/building/building.html)
  - [Platform-specific guides](https://book.servo.org/building/linux.html)
- **Servo GitHub**: https://github.com/servo/servo
- **Servo Zulip Chat**: https://servo.zulipchat.com/
- **Rust Book**: https://doc.rust-lang.org/book/
- **Cargo Documentation**: https://doc.rust-lang.org/cargo/

---

## Version Information

- **Rustc**: 1.91.0+ (auto-managed by rust-toolchain.toml)
- **Servo**: Latest main branch
- **Python**: 3.8+
- **Git**: Any recent version
