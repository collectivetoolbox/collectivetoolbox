
## Plan: Installer & Auto-Update System

A multi-component system for ctoolbox: a ctb_installer crate providing a cross-platform GUI/TUI installer, code signing tooling, server-side update distribution, and embedded automatic update capabilities. Implementation follows existing crate patterns with anyhow error handling, clap CLI parsing, and Axum routes.

### Outline Review — Identified Deficiencies & Recommendations

1. **Key rotation/revocation** — Your outline mentions the public key but not how to handle compromised keys. Add a `revoked_keys` list in the manifest and a key version/ID.

2. **Chunk integrity during download** — Individual chunk hashes are mentioned but consider streaming verification (verify each chunk as it arrives, not just at assembly time) to fail fast on corruption.

3. **Rollback on failed upgrade** — The atomic upgrade dance is outlined but there's no explicit rollback if the new binary crashes on first start. Add a "canary" first-run check that reverts if the app crashes within N seconds of upgrading.

4. **Partial downloads / resumption** — Not explicitly addressed. Store which chunks are already downloaded in a temp manifest so interrupted downloads resume.

5. **Platform-specific code signing** — On macOS/Windows, users expect Gatekeeper/SmartScreen signing. Note whether your custom signing replaces or supplements OS code signing.

6. **Feature dependency graph** — Your tree-view installer needs a way to express "Feature B requires Feature A". Add `requires: ["feature_id"]` to the manifest file entries.

7. **Manifest size limits** — With many chunks, the JSON manifest could grow large. Consider CBOR or a streaming JSON parser, or split per-file manifests.

8. **Server-side compression** — You mention wrapping chunks in JSON. This adds ~33% overhead from base64. Consider raw bytes for `/chunks/{hash}` with `Content-Type: application/octet-stream` and optional JSON wrapper via `Accept` header negotiation.

9. **Version comparison** — No semantic versioning logic specified. Use `semver` crate and allow prerelease/build metadata.

10. **Offline installer mode** — Your `--install` flag expects files present locally, but the flow for creating an offline bundle isn't specified.

---

### Steps

1. **Create `ctb-installer` crate structure** — Add ctb-installer/ with Cargo.toml depending on `ctb-utilities`. Define core modules: `manifest.rs` (types), `signing.rs`, `chunking.rs`, `download.rs`, `install.rs`, `gui.rs`, `tui.rs`.

2. **Implement manifest types & signing** — Define `ReleaseManifest`, `FileEntry`, `ChunkInfo` structs in manifest.rs. Add ed25519 signing/verification using ring in signing.rs. Include key ID for rotation support.

3. **Implement buzhash chunking** — Add `fastcdc` or custom buzhash in chunking.rs that splits files into ~64KB content-defined chunks, returning chunk hashes and boundaries.

4. **Add `--ctb-dev-sign` CLI command** — Extend `Command` enum in cli/routing.rs. Implement handler that scans release artifacts, chunks them, generates manifest, signs with dev key from `pc_settings`, outputs to `~/ctb_release/`.

5. **Add server-side routes** — In routes.rs, add `/releases/latest.json`, `/releases/{version}.json`, `/releases/chunks/{hash}`, `/releases/public-key`. Implement controllers reading from disk with signature verification via `--ctb-dev-release-check`.

6. **Implement download engine** — In download.rs, create `ChunkDownloader` that fetches missing chunks with retry, streaming verification, resume support, and progress callbacks.

7. **Build egui-based GUI installer** — In gui.rs, implement installer screens: intro/theme toggle, option selection, component tree, progress view, completion. Use `eframe` (egui+winit). Support AccessKit for accessibility.

8. **Build TUI fallback** — In tui.rs, implement simple stdin/stdout Q&A installer flow mirroring GUI states.

9. **Add `--install`, `--update`, `--uninstall` CLI commands** — Extend `Command` enum. `--install` runs full GUI/TUI flow; `--update` checks and applies updates silently; `--uninstall` removes installed files using recorded manifest.

10. **Implement atomic binary upgrade dance** — Extend existing `WaitUpgrade` command in utilities.rs with rollback logic: new binary runs with `--ctb-upgrade-canary`, if it crashes within 30s the old binary is restored from backup.

11. **Add installation record** — Store JSON at `{storage_dir}/installation.json` containing installed files, feature selections, settings, version, install date. Used by repair/uninstall.

12. **Implement background update checker** — In workspace/workspace.rs or a new module, spawn a task on startup that checks for updates (with jittered daily schedule stored in `pc_settings`). Push notification to web UI via new polling endpoint.

13. **Add `--ctb-dev-release-expire` command** — CLI command to GC old chunks referenced only by manifests older than N days.

---

### Further Considerations

1. **Separate installer binary?** — The standalone installer should be a separate smaller binary (no IceCat, minimal deps), separate from the full ctoolbox. Recommend: separate `ctoolbox-installer` binary built from same crate but feature-gated.

2. **OS code signing integration** — Do you want to layer this on top of Windows Authenticode / macOS codesign, or replace them? Recommend: supplement for trust-on-first-use within the app, but still sign with OS tools for Gatekeeper/SmartScreen.

3. **Web UI upgrade prompt mechanism** — Current web UI is request-driven. Options: (A) SSE/WebSocket for push, (B) polling endpoint every 30s, (C) check on page navigation. Recommend B for simplicity initially.

---

### LLM Implementation Prompts

Below are prompts to use for each implementation phase. Each assumes the agent has access to the codebase.

---

**Prompt 1: Crate Scaffolding**
```
Create a new ctb-installer crate at ctoolbox/src/installer/ following the project's crate conventions. It should:
- Have a Cargo.toml depending on ctb-utilities, serde, serde_json, ring, ed25519-dalek, fastcdc, sha2
- Main file installer.rs with submodules: manifest, signing, chunking, download, install, gui, tui
- Add the crate to the root Cargo.toml as a path dependency
- Follow error handling with anyhow, no panics
- Add module stubs with placeholder functions returning Result<()>
```

---

**Prompt 2: Manifest Types**
```
In ctb-installer/manifest.rs, implement the release manifest types:
- ReleaseManifest struct with: format_version (u8), ctoolbox_version (semver::Version), platform (enum Linux/Windows/Mac), date (chrono::DateTime), signature (Option<String>), revoked_key_ids (Vec<String>), files (Vec<FileEntry>)
- FileEntry struct with: path, checksum (sha256), gzip_after_install (bool), feature_id, feature_name (HashMap<String, String> for i18n), requires (Vec<String>), chunks (Vec<ChunkInfo>)
- ChunkInfo struct with: hash, offset, length
- Implement Serialize/Deserialize with serde
- Add method to serialize manifest excluding signature field for signing
- Add method to verify manifest signature given a public key
```

---

**Prompt 3: Ed25519 Signing**
```
In ctb-installer/signing.rs, implement code signing:
- generate_keypair() -> (PrivateKey, PublicKey) using ed25519-dalek
- sign_manifest(manifest: &ReleaseManifest, private_key: &PrivateKey) -> String (base64 signature)
- verify_manifest(manifest: &ReleaseManifest, public_key: &PublicKey) -> Result<bool>
- Add KeyId (first 8 bytes of public key hash) for key rotation
- Store PrivateKey wrapper with Zeroize derive
- Add functions to serialize keys to/from base64 for storage in pc_settings
```

---

**Prompt 4: Content-Defined Chunking**
```
In ctb-installer/chunking.rs, implement file chunking:
- Use fastcdc crate with average chunk size 64KB (min 32KB, max 128KB)
- chunk_file(path: &Path) -> Result<Vec<Chunk>> where Chunk has hash (sha256), offset, length, data
- apply_chunk_to_file(chunk: Chunk, output: &Path) -> Result<()>
- verify_chunk(chunk: &Chunk) -> bool (hash matches data)
- verify_file(path: &Path) -> bool (hash matches data)
- Add streaming variant that writes chunks to a directory by hash
```

---

**Prompt 5: CLI Dev-Sign Command**
```
Add --ctb-dev-sign CLI command in cli/routing.rs and implement in installer crate:
- Add DevSign variant to Command enum with optional --output-dir flag
- Load dev private key from pc_settings (add new field dev_signing_key: MaybeValue<String>)
- Scan configurable input directory for release artifacts
- For each file: chunk it, write chunks to output_dir/bh/{hash}
- Build ReleaseManifest with all file entries
- Sign manifest and write to output_dir/ctb-{platform}-{datetime}.json
- Print summary of files processed and total chunks
```

---

**Prompt 6: Server Release Routes**
```
Add update server routes in io/webui/:
- GET /releases/latest.json -> serve symlinked latest manifest
- GET /releases/{version}.json -> serve specific manifest by version
- GET /releases/chunks/{hash} -> serve raw chunk bytes from bh/ directory
- GET /releases/chunks/{hash}.json -> serve JSON file containing chunk bytes from bh/ directory
- GET /releases/public-key -> return JSON with public_key and key_id
- Add --ctb-dev-release-check CLI command that verifies uploaded release: load manifest, verify signature against configured public key, verify all chunk hashes
- Store public key in server's pc_settings as release_public_key field
```

---

**Prompt 7: Download Engine**
```
In ctb-installer/download.rs, implement chunk downloading:
- ChunkDownloader struct with: server_url, http_client (reqwest), progress_callback
- download_manifest(version: Option<&str>) -> Result<ReleaseManifest>
- download_chunk(hash: &str) -> Result<Chunk> with retry logic (3 attempts, exponential backoff)
- download_file(entry: &FileEntry, cache_dir: &Path) -> Result<PathBuf> that starts with a sparse file, downloads missing chunks, verifies each on arrival, assembling the file, verifies file
- Support resumption: check cache_dir for existing chunks before downloading
- Emit progress events: ChunkDownloaded, FileAssembled, Error
```

---

**Prompt 8: Installation Logic**
```
In ctb-installer/install.rs, implement file installation:
- InstallConfig struct with: install_dir, storage_dir, selected_features, create_desktop_icon, add_to_path, theme, language
- install_file(entry: &FileEntry, source: &Path, config: &InstallConfig) -> Result<()>
- If gzip_after_install, compress before writing, and add .gz to the file name
- Create parent directories as needed
- Set executable permissions on Linux for binary files
- create_desktop_entry(config: &InstallConfig) for Linux .desktop file
- add_to_path(config: &InstallConfig) for shell profile modification (append to .profile/.bashrc)
- Record installed files to installation.json in default storage dir
```

---

**Prompt 9: egui GUI Installer**
```
In ctb-installer/gui.rs, implement the graphical installer using eframe/egui/winit/accesskit:
- InstallerApp struct implementing eframe::App with current_screen enum (Intro, Options, Components, Progress, Complete, Repair, Uninstall)
- Intro screen: logo, welcome text, theme toggle (light/dark), Quick Install vs Customize buttons
- Options screen: install path picker, storage path picker, checkboxes for desktop icon/PATH/etc, language dropdown
- Components screen: tree view of features with checkboxes (use egui_extras::TableBuilder or custom tree)
- Progress screen: overall progress bar, current file progress bar, scrolling log of chunk/file events
- Complete screen: success message, checkbox to launch app, Finish button
- Use egui's built-in dark/light theme switching
- Pass InstallerConfig between screens via shared state
- Verify ctb-installer Cargo.toml - use of lib and bin targets with same file seems strange
```

---

**Prompt 10: File Picker**
```
In ctb-installer/file_picker.rs, implement a file picker widget:
- "Miller column" style browsing (each subfolder appears in a new column to the right)
- Toolbar: Back/forward/up/refresh buttons; New folder button; Hidden file toggle
- Sidebar on left with Home, an option for "This PC" (on Unix-y it would show the root folder; on Windows it would list drives; on Mac it would list the Volumes folder)
- OK and Cancel buttons
- This is meant to swap out in place of RFD in installer/gui.rs.
```

---

**Prompt 11: TUI Installer**
```
In ctb-installer/tui.rs, implement text-mode installer:
- Use standard stdin/stdout, no curses
- Simple numbered menu for screen navigation
- prompt_yes_no(question: &str) -> bool
- prompt_choice(question: &str, options: &[&str]) -> usize
- prompt_path(question: &str, default: &Path) -> PathBuf
- Run through same flow as GUI: intro, options, component selection, progress (print each file), complete
- For progress: print "Downloading chunk 1/100..." style updates
- Support unattended mode via --unattended flag that uses defaults
- Sanity-check ctb-installer/install.rs
```

---

**Prompt 12: Install/Update/Uninstall CLI Commands**
```
Add CLI commands in cli/routing.rs:
- --install: Launch GUI installer (or TUI if --no-gui), using files from current directory if present, otherwise download
- --update: Check for updates silently, download if available, prompt user or apply if --unattended
- --uninstall: Read installation.json, remove all recorded files, remove desktop entry, remove from PATH
- --ctb-upgrade-canary: Internal flag for post-upgrade validation; if process survives 30s, delete backup and exit; if crash, restore backup

Implement the atomic upgrade dance:
1. Download new binary to temp location
2. Copy current binary to backup location  
3. Spawn new binary with --ctb-upgrade-canary --backup-path {backup}
4. Exit current process
5. New binary waits 30s, if still alive: delete backup, respawn normally
6. If new binary crashes: backup is restored by watchdog logic in parent
```

---

**Prompt 13: Background Update Checker**
```
Add background update checking to workspace startup (except when running as a toolbox service subprocess or a lightweight CLI command):
- On first run, generate random time-of-day (0-24h) and store in pc_settings as update_check_time
- On startup, spawn detached task that: waits until scheduled time (or immediately if past), calls /releases/latest.json, compares version to installed version
- If update available, store update_available: true and new version in a shared state or temp file
- Add polling endpoint GET /api/update-status that returns {available: bool, version: string, release_notes_url: string}
- Web UI polls this every 60s when tab is visible
- Add UI notification banner "Update available - Restart to upgrade" with Restart Now and Later buttons
```

---

**Prompt 14: Release Expiration**
```
Add --ctb-dev-release-expire CLI command:
- Accept --older-than {days} argument
- Scan releases/ directory for all manifest JSON files
- Build set of all chunk hashes referenced by manifests newer than threshold
- Scan bh/ directory, delete any chunk not in the keep set
- Delete old manifest files
- Print summary: "Deleted X chunks, Y manifests, freed Z bytes"
```

---

**Prompt 15: Follow-ups**
```
Additional changes regarding installer crate:
- Implement size changes for features (when checking or unchecking features in the installer UI, it should indicate the required disk space - this may need a change to the JSON manifest to include the total filesize of each file, which would be summed for files
- Fix the "default feature tree" thing that's in the TUI and GUI modules and replace it with loading the tree from the manifest, unless that's being handled elsewhere (in which case let's make it more obvious that by "default" it's just a dummy value, but really shouldn't that be encoded in the Rust types if that's the case, like by having it be None until it's set? But in that case why not just pass it in in the constructor?)
- Offer "minimal" and "complete" buttons that uncheck or check all optional features
- Add a note about the need for user data storage space: "These file sizes reflect the storage needed by the application itself. Your own documents will occupy additional space; if you're unsure what you'll need, we recommend having at least 20 GB free."
- Adjust code that uses the bh/ directory if necessary to ensure that all chunks are stored in compressed form on disk.
- Rename "Kiosk mode" and related config options to "Serve Public Web Site Only", to reflect its usage
- URLs for builds should include the platform, since different binaries will need to be served for Windows/Mac/Linux
- Add a way to flag certain features as unavailable in the JSON installation manifest
- If `kiosk_mode` setting is turned on and the `domain_name` setting is *not* ^((www\.)?collectivetoolbox\.com)$, the interactive prompting logic for updates should be skipped and the workspace should restart immediately.
- If `kiosk_mode` setting is turned on and the `domain_name` setting is exactly ^((www\.)?collectivetoolbox\.com)$, automatic updates should not run (as deploys are handled by a separate deploy script).
- When the workspace first starts up, it should check for updates and automatically install one if it's already been downloaded. If it doesn't finish the check (or finish downloading) within a couple seconds, start up anyway.
```

---

**Prompt 16: Deduplicating GUI and TUI code**
```
- First: Currently, the bh/ folder is compressed with gzip on disk. I think it would be preferable to use Brotli for that; let's update it.
- Second, make sure the installer chunk downloader requests good compression (I guess brotli; that's already turned on for the Axum server at least for HTML responses).
- Third, make sure the Axum configuration is set up such that the binary chunks will be served with that compression if the client supports it - Brotli or whatever compression the client accepts should be used for those chunk responses, as well as for JSON/JS/HTML/CSS/SVG - please ensure that configuration is set up appropriately.
- Finally, I'd like you to review the GUI and TUI modules of the installer and refactor out the redundant elements and strings. 
```

---

**Prompt 17: Streaming tarball**
```
- Implement generation of a streaming tarball for offline installers (which should be linked as an option on the homepage for the latest version; the version JSON manifest's name should be provided as a route parameter - "-[platform]-latest" for the most recent; older versions shouldn't get a link). Make sure to support HTTP range requests - this will need some clever logic to resume generating a tarball midstream.
```

---

**Prompt 18: Localization**
```
- Implement localized strings using Fluent for the ctb-installer crate.
```

---

**Prompt 19: Testing**
```
- Make light-dark appearance toggle on installer three-state: "Autodetect theme" (the default), "Light theme", and "Dark theme". 
- Add ru, id, vi, ar locales to installer.
- The en-GB locale bundle says "only differences are listed here" at the top, but then proceeds to have all the strings defined - this seems like an accident - can we address it?
- Finish updating GUI and TUI for localization.
- Double-check Hindi ftl file - it has some Unicode replacement characters, and might have some errors.
- Add comprehensive tests for installer crate:
    - Unit tests for manifest serialization round-trip
    - Unit tests for signing: generate key, sign, verify succeeds; tampered manifest verify fails
    - Unit tests for chunking: chunk file, reassemble, verify identical
    - Integration test: create mock release, sign it, verify it
    - Integration test: mock HTTP server serving chunks, download and assemble file
    - Test atomic upgrade: spawn child process, simulate crash, verify rollback
```
