# Archived: Servo Migration Summary

This Servo migration summary has been archived. See the archived copy for the original build notes and integration details.

Archived copy: [archive_docs/SERVO_MIGRATION_SUMMARY.md](archive_docs/SERVO_MIGRATION_SUMMARY.md)

For current build and architecture guidance, consult [README.md](README.md).
# Servo Migration Summary (servo/main)

## Overview
Graphshell has been upgraded from Servo pinned commit `5e2d42e` to `servo/main` branch, incorporating the latest multi-window and WebView API changes from Servo's active development.

## Key Changes Applied

### Dependency Updates

#### Cargo.toml (workspace level)
- Changed all Servo git dependencies from `rev = "5e2d42e"` to `branch = "main"`
- Updated related Servo org crates to Servo main's pinned versions:
  - `stylo`: `branch = "2025-03-15"` → `rev = "44bf70c4215dde5997aa835081d7320c1f533c95"`
  - `webrender`: `branch = "0.66"` → `rev = "6cafc606096db4715a6119a6e16391aed9af47a5"`
  - `servo-media`: updated to pinned rev `f384dbc4ff8b5c6f8db2c763306cbe2281d66391`
- Updated `log` from `workspace` pinned 0.4.27 to explicit `0.4.29` (required by Servo main)

#### Rust Toolchain
- Updated [rust-toolchain.toml](rust-toolchain.toml) from 1.85.0 → 1.86.0 (Servo main requirement)

### API Migrations

#### Crate Renames
Servo renamed internal components recently (commit ~4 days ago on main):
| Old Crate | New Crate | Impact |
|-----------|-----------|--------|
| `compositing_traits` | `paint_api` | Updated imports in [src/graphshell.rs](src/graphshell.rs#L12) and [src/compositor.rs](src/compositor.rs#L10) |
| `layout_thread_2020` | `layout_api` | Updated [Cargo.toml](Cargo.toml#L121) dependency |

#### Affected Source Files
1. **[src/graphshell.rs](src/graphshell.rs#L12)**: Updated `compositing_traits` import to `paint_api`
   ```rust
   use paint_api::{
       CompositorMsg, CompositorProxy, CrossProcessCompositorApi, WebrenderExternalImageHandlers,
       WebrenderImageHandlerType,
   };
   ```

2. **[src/compositor.rs](src/compositor.rs#L10)**: Updated both display_list and core imports to `paint_api`
   ```rust
   use paint_api::display_list::{CompositorDisplayListInfo, HitTestInfo, ScrollTree};
   use paint_api::{
       CompositionPipeline, CompositorMsg, CompositorProxy, ImageUpdate, SendableFrameTree,
   };
   ```

### Upstream Servo Changes (Relevant PRs)

#### Multi-Window / WebView Delegate (PR #36223, merged Apr 2, 2025)
- **Change**: Moved `WindowMethods` functionality to per-WebView `WebViewDelegate` trait
- **Why**: Supports multi-window architecture; per-WebView screen geometry and offsets
- **Impact on Graphshell**: 
  - Compositor now calls `webview_screen_geometry(webview_id)` instead of global window methods
  - Message types now include `webview_id` in cross-process compositor messages
  - TODO: Adapt [src/compositor.rs](src/compositor.rs#L804) to use new ScreenGeometry API (TODO already present in code)
  - TODO: Update webview delegate implementations in [src/webview/](src/webview/)

#### Related Commits
- Commit `80a6ba5`: Implemented `WebViewDelegate.screen_geometry` for OHOS platform (reference for porting)

## Build Status

### ✅ Successful
- **Graphshell package** (`graphshell v0.0.3`): Compiles successfully with new Servo deps
- **Dependency resolution**: All workspace dependencies resolve without version conflicts
- **Rust toolchain**: Updated and installed (1.86.0)
- **Python dependency**: Installed (Python 3.12.10)

### ⚠️ Requires MozillaBuild (Windows)
- **Full package build** (graphshellview): Blocked on MozillaBuild/LLVM toolchain for mozjs_sys build
  - Error: `MozTools or MozillaBuild not found!`
  - Resolution: Follow https://github.com/servo/mozjs?tab=readme-ov-file#windows
  - This is a standard Servo SpiderMonkey JavaScript engine build requirement for Windows (not graphshell-specific)
  - Alternatively, use the provided Nix shell or Linux/macOS dev environment

## Next Steps

### Immediate (to unblock full build)

**Option A: Set up Windows MozillaBuild (recommended for Windows development)**
1. Follow official Servo/mozjs Windows setup: https://github.com/servo/mozjs?tab=readme-ov-file#windows
2. Install MozillaBuild (includes LLVM, clang, etc.)
3. Re-run `cargo build` in MozillaBuild shell
4. Verify runtime by launching graphshell browser and testing webview functionality

**Option B: Use Linux/macOS or WSL2 (alternative)**
- Servo builds more easily on Linux/macOS without complex Windows toolchain requirements
- Windows Subsystem for Linux 2 (WSL2) provides a Linux environment on Windows

**Option C: Use Nix development shell (if nix installed)**
- Graphshell already has [shell.nix](shell.nix) preconfigured
- Run: `nix-shell` to enter dev environment with all dependencies

### Medium-term (API Adaptation)
1. Implement `WebViewDelegate` trait adapter in `src/webview/` to handle per-webview screen geometry
2. Update message handling in `src/graphshell.rs` and `src/compositor.rs` to route per-webview messages
3. Add unit tests for multi-webview scenarios
4. Test multi-window embedding patterns (create/manage multiple webviews per window)

### Documentation
- Add migration notes to [README.md](README.md) describing Servo main support and known limitations
- Document any API differences discovered during multi-webview testing

## Technical Debt & Known Issues

1. **Unused crate imports**: `servo_geometry` and `webgpu` are still pinned in Cargo.toml; verify they still exist in Servo main crate tree and check for renames
2. **ScreenGeometry TODO**: [src/compositor.rs](src/compositor.rs#L804) contains TODO to use ScreenGeometry API; this should be addressed as part of multi-window work
3. **Python build dependency**: Standard for Servo; document in developer setup guide

## Files Modified

1. [Cargo.toml](Cargo.toml) - Workspace dependency updates (Servo branch → main + version pins)
2. [rust-toolchain.toml](rust-toolchain.toml) - Rust 1.85.0 → 1.86.0
3. [src/graphshell.rs](src/graphshell.rs#L12) - compositing_traits → paint_api import
4. [src/compositor.rs](src/compositor.rs#L10) - compositing_traits → paint_api imports

## References

- Servo main branch: https://github.com/servo/servo/tree/main
- PR #36223 (Multi-window WebViewDelegate): https://github.com/servo/servo/pull/36223
- Servo Workspace Cargo.toml: https://raw.githubusercontent.com/servo/servo/main/Cargo.toml
