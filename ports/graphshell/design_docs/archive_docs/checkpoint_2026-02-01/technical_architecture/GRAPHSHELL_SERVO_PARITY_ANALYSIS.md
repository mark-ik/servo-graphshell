# Graphshell-Servo Parity Analysis
## Executive Summary

Graphshell is currently pinned to Servo revision `5e2d42e9445` from **April 17, 2025**. The current Servo main branch (as of February 2, 2026) is **3,452 commits ahead**, representing approximately **9.5 months** of development.

**Critical Status:** Graphshell is significantly outdated and requires a comprehensive update to achieve parity with current Servo.

---

## Version & Dependency Gaps

### 1. Rust Toolchain
- **Graphshell:** `1.85.0` (Edition 2024)
- **Servo:** `1.91.0` (Edition 2024)
- **Action Required:** Update Graphshell's `rust-toolchain.toml` from 1.85.0 → 1.91.0
- **Impact:** Potential compilation issues, missing language features, security fixes

### 2. Core Servo Components
All Servo component dependencies are pinned to obsolete revision `5e2d42e`:

**Components requiring updates:**
- background_hang_monitor
- base
- bluetooth / bluetooth_traits
- canvas
- compositing_traits → **RENAMED to `paint_api`** ⚠️
- constellation / constellation_traits
- devtools
- embedder_traits
- fonts
- layout_thread_2020
- media
- net / net_traits
- profile / profile_traits
- script / script_traits
- servo_allocator
- servo_config
- servo_geometry
- servo_url
- webdriver_server
- webgpu / webgpu_traits

**Action Required:** Update all component git references from `rev = "5e2d42e"` to latest Servo `main` or a specific stable commit.

### 3. Stylo (CSS Engine)
- **Graphshell:** Branch `2025-03-15` (still exists but outdated)
- **Servo:** Revision `fce45cfa72008327d714575913bc9c968fa1446c` (upgraded 2026-01-01 in commit a5752da7d63)
- **Action Required:** 
  - Switch from branch reference to specific revision
  - Update to `rev = "fce45cfa72008327d714575913bc9c968fa1446c"` or later

### 4. WebRender
- **Graphshell:** Branch `0.66`
- **Servo:** Version `0.68` (from crates.io as of commit dd292e6ba98)
- **Action Required:** Update to WebRender 0.68
- **Impact:** Breaking API changes likely in compositor/rendering layer

### 5. Servo Media
- **Graphshell:** Git reference without specific revision
- **Servo:** Revision `f384dbc4ff8b5c6f8db2c763306cbe2281d66391`
- **Action Required:** Pin to specific revision for build reproducibility

### 6. IPC Channel
- **Graphshell:** `0.19`
- **Servo:** `0.20.2`
- **Action Required:** Update workspace dependency to 0.20.x
- **Impact:** Potential breaking changes in IPC protocol

### 7. Other Workspace Dependencies
| Dependency | Graphshell | Servo Current | Notes |
|------------|-------|---------------|-------|
| keyboard-types | 0.7 | 0.8.3 | Minor API changes likely |
| winit | 0.30 | 0.30.12 | Patch update needed |
| rustls | 0.23 | 0.23 | Same major version |
| dpi | 0.1 | 0.1 | Same |

---

## Major Structural Changes in Servo

### 🔴 BREAKING: Component Renames (Commit 9c9d9c863fd)
**Date:** December 2025

**Changes:**
- `compositing` → `paint`
- `compositing_traits` → `paint_api`
- `IOCompositor` → `Paint`

**Impact on Graphshell:**
- All imports of `compositing_traits` must be updated to `paint_api`
- Graphshell's `compositor.rs` and `rendering.rs` likely require significant refactoring
- Message types and trait implementations need updating

**Files requiring updates:**
```
graphshell/src/compositor.rs
graphshell/src/rendering.rs
graphshell/src/graphshell.rs (if it imports compositing)
graphshell/src/webview.rs (if it imports compositing)
graphshell/Cargo.toml (dependency name change)
```

### 2. GenericChannel Migration (Multiple commits)
**Key commits:**
- 8dfeaaf3394: WebGL moved to GenericChannel
- 71cc3b23824: WebGPU uses GenericChannel
- 4089ece1fec: Constellation channels migrated

**Purpose:** More flexible channel abstraction replacing direct IpcChannel usage

**Impact on Graphshell:** If Graphshell directly uses IPC channels for communication with Servo components, may need refactoring to use GenericChannel pattern.

### 3. Accessibility Tree Updates (Commit c2530a49ee1)
**Date:** December 2025
**Change:** Plumbed accessibility tree updates from layout to embedder
**Impact:** Graphshell may need to handle new accessibility messages if it wants to expose a11y features

### 4. WebDriver Migration (Commit 1133eb229a3)
**Change:** WPT now uses WebDriver for all runs
**Impact:** If Graphshell implements WebDriver support, protocol may have changed

### 5. Published CSP Crate (Commit 85da46e5f88)
**Change:** Switched to published content-security-policy crate
**Impact:** CSP handling may have new APIs

---

## New Features & APIs Added to Servo (Post-5e2d42e)

### Web Platform Features
1. **Fullscreen API** - Updated documentation and conformance (cc56218b0c5)
2. **Origin API** - Implemented (56bc26a7cc0)
3. **Color Input Enhancements:**
   - Support for different colorspaces (83bf475370c)
   - Accept any valid CSS color (2204bb23eb9)
4. **WebCrypto Enhancements:**
   - AES-OCB (encrypt, decrypt, generate, export, import operations)
   - ML-DSA (sign, verify, generate, export, import)
   - ML-KEM (encapsulation, decapsulation, generate, export, import)
5. **CSS Features:**
   - `:open` pseudo-class (bd779d141a3)
   - `::details-content` pseudo element (aad74241507)
   - `-webkit-text-security` (db2a7cbe2aa)
   - `caret-color` support (c5d3c6ef608)
   - Registered custom properties with CSSOM (624e9b49e1d)
6. **IndexedDB** - Connection lifecycle improvements (91ee8341955)
7. **SiteDataManager** - Enhanced clear_site_data for localStorage and sessionStorage
8. **GamepadProvider** - New exposed API with Responder types (b6a17611987)
9. **navigator.pdfViewerEnabled** (f822959d7e4)

### Layout & Rendering
1. **Text Security** - `-webkit-text-security` support
2. **Soft Wrap** - Respect `word-break: keep-all` between NU/AL/AI/ID chars (e950e5c2ca0)
3. **Legacy Alignment** - Presentational hints match specification (da7027a6df8)
4. **Customizable `<select>` Elements** - HTML5Ever hooks (94e71e7ec0e)

### Developer Tools
1. **Debugger** - Redundant addDebuggee call removed
2. **Inspector** - ActorEncode implementation and sub-actor cleanup (7528d05a02e)

### Memory & Performance
1. **Memory Reports** - Refactored common infrastructure (5e2d42e9445 - the commit Graphshell is pinned to)
2. **Painter Caching** - Animation cache with smarter updates (from later commits)
3. **Rope Type for TextInput** - Memory-efficient text handling (2efa7d5d96d)

---

## Dependency Update Checklist

### Phase 1: Toolchain & Build System
- [ ] Update `rust-toolchain.toml`: 1.85.0 → 1.91.0
- [ ] Update `Cargo.toml` edition (already 2024, verify all features compile)
- [ ] Test basic compilation with new toolchain

### Phase 2: Workspace Dependencies
- [ ] Update `ipc-channel`: 0.19 → 0.20.2
- [ ] Update `keyboard-types`: 0.7 → 0.8.3
- [ ] Update `winit`: 0.30 → 0.30.12 (if not already latest patch)
- [ ] Update other minor dependencies as needed

### Phase 3: Critical Servo Dependencies
- [ ] **BREAKING:** Update component renames:
  - [ ] Change `compositing_traits` → `paint_api` in Cargo.toml
  - [ ] Update all imports in source files
  - [ ] Update trait implementations
- [ ] Update all Servo component git revisions to latest `main` or stable commit
  - Recommended: Use Servo's latest stable revision from main branch
  - Test approach: Update incrementally and fix compilation errors

### Phase 4: Rendering Stack
- [ ] Update WebRender: 0.66 → 0.68
  - [ ] Update `webrender` dependency
  - [ ] Update `webrender_api` dependency
  - [ ] Update `wr_malloc_size_of` dependency
  - [ ] Fix API breaking changes in compositor/rendering code

### Phase 5: CSS Engine
- [ ] Update Stylo from branch to specific revision
  - [ ] Change all stylo dependencies to use `rev = "fce45cfa72008327d714575913bc9c968fa1446c"` or later
  - [ ] Test CSS rendering and layout

### Phase 6: Media Stack
- [ ] Update servo-media to revision `f384dbc4ff8b5c6f8db2c763306cbe2281d66391`
- [ ] Update servo-media-dummy with same revision

### Phase 7: Testing & Verification
- [ ] Run Graphshell's custom test harness: `cargo test`
- [ ] Verify window creation and event handling
- [ ] Test WebView lifecycle
- [ ] Verify IPC communication between controller and browser process
- [ ] Test all embedding APIs in `graphshell` library crate
- [ ] Manual testing:
  - [ ] Basic page loading
  - [ ] Navigation (back/forward)
  - [ ] Multiple tabs/windows
  - [ ] Downloads
  - [ ] Bookmarks
  - [ ] Input handling (keyboard, mouse, touch)

---

## Estimated Update Effort

### Complexity Assessment
- **Low Risk:** Rust toolchain, minor dependency updates
- **Medium Risk:** Servo component updates without API changes
- **High Risk:** 
  - compositing → paint rename (pervasive changes)
  - WebRender 0.66 → 0.68 (likely compositor API changes)
  - IPC Channel 0.19 → 0.20 (message protocol changes)

### Time Estimates (Developer Hours)
- Phase 1 (Toolchain): 1-2 hours
- Phase 2 (Workspace deps): 2-4 hours
- Phase 3 (Servo components): 8-16 hours
- Phase 4 (WebRender): 8-16 hours (depends on API changes)
- Phase 5 (Stylo): 2-4 hours
- Phase 6 (Media): 1-2 hours
- Phase 7 (Testing): 8-16 hours

**Total: 30-60 hours** (approximately 1-2 weeks of focused work)

### Risk Factors
1. **Unknown API Changes:** Some Servo components may have introduced breaking changes not documented in commit messages
2. **WebRender Migration:** The 0.66 → 0.68 update may require significant compositor refactoring
3. **IPC Protocol:** The IPC channel update might break the embedding API if Graphshell's controller relies on specific message formats
4. **Accumulation Effect:** 3,452 commits represent many subtle changes that may interact in unexpected ways

---

## Recommended Update Strategy

### Option A: Incremental Update (Recommended)
1. Create a dedicated `servo-update` branch
2. Update dependencies in small batches, fixing compilation errors at each step
3. Run tests after each phase
4. Use `cargo check` frequently to catch issues early
5. Keep a detailed log of API changes encountered

### Option B: Fresh Integration
1. Clone latest Servo repository
2. Compare Graphshell's integration points with Servo's `servoshell` implementation
3. Rebuild Graphshell's Servo integration from scratch based on latest patterns
4. Migrate features incrementally

**Recommendation:** Start with **Option A**. If compilation errors become overwhelming or if fundamental architecture mismatches emerge, pivot to **Option B**.

---

## Critical Code Locations in Graphshell

Files most likely to require updates:

### High Priority (Will definitely need changes)
1. `src/compositor.rs` - Uses compositing_traits (now paint_api)
2. `src/rendering.rs` - WebRender integration
3. `src/lib.rs` - Module declarations and re-exports
4. `Cargo.toml` - All dependency updates

### Medium Priority (May need changes)
1. `src/graphshell.rs` - Main browser orchestration, uses Servo components
2. `src/webview.rs` - WebView lifecycle, may use compositing types
3. `src/window.rs` - Window management, Winit integration
4. `graphshell/Cargo.toml` - Library crate dependencies
5. `graphshell/src/lib.rs` - IPC message types and controller API

### Low Priority (Unlikely to need changes)
1. `src/bookmark.rs` - Data model only
2. `src/download.rs` - Data model only
3. `src/storage.rs` - Configuration persistence
4. `src/config.rs` - Configuration types
5. `src/keyboard.rs` - Input mapping
6. `src/touch.rs` - Touch event handling

---

## Long-term Maintenance Recommendations

1. **Pin to Specific Revisions:** Always use `rev = "..."` instead of branches for reproducible builds
2. **Regular Updates:** Update Servo dependencies at least quarterly (every 3 months)
3. **Automated Testing:** Expand the custom test harness to cover more integration scenarios
4. **CI Integration:** Add a CI job that attempts to build against Servo's latest main weekly
5. **Documentation:** Maintain a `SERVO_VERSION.md` file documenting:
   - Current Servo revision in use
   - Date of last update
   - Known issues or workarounds
   - Breaking changes from previous version

---

## Additional Notes

### Why Graphshell Fell Behind
As noted in Graphshell's README, the project is "currently no longer maintained" due to:
- Limited manpower
- Limited funding
- Servo's rapid release cadence

### Contributing to Servo
Several Graphshell innovations were contributed back to Servo (see "Behind the Scenes of Graphshell Browser Development" article). This bidirectional relationship should continue if Graphshell development resumes.

### Alternative Approach: Track Servoshell
Instead of maintaining a separate compositor/rendering layer, consider aligning more closely with Servo's official `servoshell` implementation. Many of Graphshell's early experiments have been incorporated into Servo itself, so the gap between Graphshell's goals and Servo's capabilities may have narrowed.

---

## Next Steps

1. **Decide on Update Scope:** Full parity vs. minimal functional update
2. **Allocate Resources:** Dedicate developer time for the estimated 30-60 hours
3. **Create Update Branch:** Start with Phase 1 (toolchain) and proceed incrementally
4. **Test Continuously:** Don't wait until all updates are complete to test
5. **Document Changes:** Keep notes on API changes for future reference
6. **Consider Archival:** If update proves infeasible, document the gap for future contributors

---

## References

- Servo Repository: https://github.com/servo/servo
- Graphshell Repository: https://github.com/graphshelltile-org/graphshell
- Behind the Scenes Article: https://wusyong.github.io/posts/graphshell-ui/
- Servo Main Branch: commit `8ea5b4bf951` (as of 2026-02-02)
- Graphshell Pinned Revision: `5e2d42e9445` (2025-04-17)
- Commits Behind: 3,452
- Time Gap: ~9.5 months
