# Installer

Consolidated from the former installation drafts and installer reports in
issues.md. Historical reports below remain open pending reproduction; this
cleanup does not establish whether they still occur on current builds.

## Reported Bugs Needing Reproduction

- [ ] Keyboard input is laggy and keys stick until the mouse moves.
- [ ] The window sometimes becomes "Not responding" while unfocused; focusing
  it resumes processing. Closing the window sometimes hangs until closed again.
- [ ] Manifest loading blocks the UI, or component selection remains at
  "(placeholder - manifest not loaded)". Fetch the platform manifest in the
  background on startup; show a waiting state when needed and offer retry or
  cancel after repeated failures.
- [ ] Cancelling installation restarts the download instead of stopping and
  rolling back installed files.
- [ ] The license modal opens partly off-screen and includes invisible tab
  stops instead of keeping focus within visible modal controls.
- [ ] Shift-Tab cannot leave the Installation directory textbox.
- [ ] Scrollbars are too narrow, lack buttons and visible gutters, and jump on
  left-click. Left-click should page; middle-click should jump.
- [ ] Cursive joining is broken; see
  <https://github.com/emilk/egui/issues/2517>.
- [ ] Vietnamese text mixes fonts inconsistently.
- [ ] File picker Cancel/OK buttons and sidebar locations are not localized.

### File Picker

- [ ] At narrow widths, a growing gray right margin appears, controls and Path
  text shift left, pointer targets no longer match controls, and Places buttons
  misalign and move on hover. Investigate these as a potentially shared layout
  bug rather than independent cosmetic problems.
- [ ] Shift-Tab moves forward rather than backward.
- [ ] Arrow keys escape into unrelated controls or move focus without selecting
  the item. Up/Down should select the preceding/following item in the current
  column, stopping at its ends; Right should focus the first item in the next
  column; Left should select the enclosing directory, like the Up toolbar button.
- [ ] Home, End, Page Up, and Page Down do nothing.
- [ ] Each toolbar button needs its own tab stop; the Path field must be
  reachable by Tab.

## Pending Work From Earlier Proposals

These requests are retained because their end-to-end status was not established
in this cleanup. Existing installer modules are not evidence of full parity.

- [ ] Audit upgrade/downgrade/repair detection against both version and build
  date, including equal-version rebuilds.
- [ ] Audit interrupted-download resume, per-file and overall progress,
  integrity retries with backoff, and useful download/install logs.
- [ ] Audit installation records and repair/uninstall behavior: preserve file
  inventory, selected features/settings, version/build date, and record schema
  version in the default storage directory.
- [ ] Audit component dependency selection and partially selected groups.
- [ ] Verify offline bundle export and installation without contacting the
  release server.
- [ ] Audit automatic updates end to end: startup/daily checks, persisted
  schedule and selections, background download, restart/defer notifications,
  safe binary replacement, and automatic rollback after a failed canary run.
- [ ] Confirm installer options cover storage directory, desktop/start-menu
  integration, PATH, language/theme, keyboard layout, UI mode, and per-user
  versus system-wide installation.
- [ ] Retain first-run setup requirements outside basic file installation:
  skip redundant setup when configured; detect available terminal/GUI/browser
  and accessibility needs; allow UI choice and settings now or later, including
  public serving/domain/certificate and initial-page preferences. Decide the
  scope of display-manager sessions and operation without a window manager.
- [ ] Audit release publication/verification and expiry against the old
  requirements: signed/versioned platform manifests, key rotation/revocation,
  localized component metadata and dependencies, file/chunk hashes, installed
  compression policy, and removal of unreferenced expired chunks.

Native platform signing (Authenticode/SmartScreen/Gatekeeper) was explicitly
deferred in the original proposal and remains deferred here.