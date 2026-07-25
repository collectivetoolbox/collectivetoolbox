# X11-Related Data/Resources Inventory

## Overview
Repository contains embedded X11 input runtime resources and UI assets that are independent of C X11 library bindings. These would remain necessary even in a pure-Rust X11 client crate scenario.

## Resource Directories & Current Usage

### 1. **XKB Keyboard Data** (PRIMARY)
**Location:** `built/assets/x11/xkb/` (generated from source during build)
**Subdirectories:** `rules/`, `symbols/`, `geometry/`, `types/`, `keycodes/`, `compat/`
**Size:** ~100 KB (from xkeyboard-config distribution)
**Source in build:** `vendor/x11/c_src/xkeyboard-config*/share/X11/xkb/`

**Current Usage:**
- **Called from:** `src/bin.rs` (main entry) + `src/installer/bin.rs`
- **Handler:** `src/storage/minimal/xkb.rs::ensure_xkb_config_root()` 
- **Embedded via:** `include_dir!()` macro in `storage/minimal` crate
- **Behavior:** Extracts embedded XKB tree to temp dir + sets `XKB_CONFIG_ROOT` env var
- **Used by:** libxkbcommon (dynamic) to resolve keyboard layouts/compose rules

**Content type:** XKB configuration rules, geometry data, keyboard layout definitions
**IME/Compose support:** Contains compose sequences, IME hints (Japanese IMEOP/IMEOFF keycodes in `keycodes/xfree86`)

### 2. **Fonts**
**Location:** `assets/resources/fonts/` + `src/installer/data/resources/fonts/`

**Embedded in UI:**
- **Web UI fonts** → `built/assets/resources/fonts/` (bundled in web app)
- **Installer GUI fonts** → Included in installer build
  - Noto Sans family (4 variants)
  - Fira Code
  - Noto Emoji
  - Noto Sans CJK (JP, SC, KR)
  - Noto Sans Indic (Bengali, Devanagari)
  - Noto Sans Arabic
  - EB Garamond
  - Goudy Bookletter
  - Noto Nastaliq Urdu

**Current Usage:**
- `src/installer/gui/theme.rs`: egui font registration
- Web UI: CSS font-face imports
- **Licenses:** OFL 1.1 (Noto, Fira Code), various
- **Not X11-specific** but necessary for offline rendering

### 3. **UI Icons**
**Location:** `assets/resources/icons/`
**Content:** SVG icons (Material Design, Phosphor, Fluent) + PNG favicon
**Current Usage:**
- `src/installer/install.rs` line 374: Desktop shortcut icon
- Web UI: Various UI icons
- **Not X11-specific** but required for complete UI

### 4. **Locale/i18n Data**
**Location:** `src/installer/locales/` (22 languages)
**Format:** Fluent FTL files
**Supported:** en-US (default), ar, bn, de, en-GB, es, fa, fil, fr, hi, id, it, ja, ko, nl, pl, pt-BR, ru, tr, ur, vi, zh-CN

**Current Usage:**
- `src/installer/i18n.rs`: Fluent i18n engine
- Installer UI multilingual support
- **Not X11-specific**

### 5. **Accessibility Metadata**
**Current Usage:**
- `src/installer/Cargo.toml`: egui feature `accesskit`
- `src/installer/gui/theme.rs`: High-contrast scrollbar styles, disabled widget text handling
- `vendor/kas-core/src/accesskit.rs`: AccessKit bridge
- **Not data files; implemented via egui/accesskit**

## Cursor/Theme Resources
**Finding:** No embedded cursor themes or X11 cursor data found in repo.
- **Reason:** winit/egui handle cursor rendering; no custom X11 cursor files present
- Cursor icons sourced from winit's `cursor-icon` crate or `wayland-cursor` crate
- `xcursor` library (C) provides system cursor themes but not bundled

## Clipboard/Selection Protocol Resources
**Finding:** No protocol definition files embedded.
- **Reason:** Clipboard protocol implementations are in code (smithay-clipboard for Wayland, X11 built into C libraries)
- `src/installer/Cargo.toml`: egui-winit feature `clipboard` handles abstraction
- **Libraries used:** smithay-clipboard (Wayland), system X11 libs

## Recommended Architecture (Thin Rust X11 Crate)

### KEEP in-tree (relocate if needed):
1. **`built/assets/x11/xkb/`** → `src/storage/minimal/` or standalone `ctb-xkb-data` crate
   - **Reason:** Non-negotiable; required for keyboard input on all keyboard layouts
   - **Reuse:** Same `include_dir!` + temp-extract pattern works for pure-Rust client
   - **Alternative:** Could compile to binary embedded data using `bincode` for smaller footprint

2. **`assets/resources/fonts/`** → Already separated (installer-specific)
   - **Reuse:** Installer fonts can stay; web UI fonts bundle independently

3. **`src/installer/locales/`** → Keep as-is
   - **Reason:** Installer remains its own crate with dedicated i18n

### CAN REMOVE/DECOUPLE:
- **X11 C library bindings** (`vendor/x11/`) → Replace with pure-Rust crate (e.g., x11rb)
- **xkbcommon C library** → Potentially replaceable with native Rust keyboard/compose handling (larger lift)

### NO CHANGES NEEDED:
- Icon/font assets (already decoupled from X11 code)
- Accessibility metadata (egui-driven; library feature-gated)

## Build Integration
- **Main build:** `build.rs` → calls `ctb_build_support::prepare_assets()`
- **Asset packer:** `src/build_support/asset_packer.rs` lines 118-164
- **Rerun directive:** `print_rerun_directives()` watches `vendor/x11/c_src/xkeyboard-config*/`

## Storage/Minimal Crate Specifics
- **Path:** `src/storage/minimal/Cargo.toml`
- **Deps:** `include_dir`, `tempfile`, `once_cell` (for XKB initialization)
- **Compile-time:** XKB tree embedded during build
- **Runtime:** Extracted to temp; persists via `OnceCell` for process lifetime
