# Issue Inbox

Execution priorities and MVP scope:
[Dc-Native Personal MVP Roadmap](../docs/mvp-roadmap.md). This is a categorized
inbox, not an implementation order. Historical reports need reproduction before
assuming the current code still has the reported fault.

## Topical Backlogs

- [CSC and archives](csc.md): preservation, streams, recovery, file API policy,
  native-platform qualification, and metadata privacy.
- [Dc parser](dc-parser.md): syntax validation, semantic roles, projection,
  derives, diagnostics, and streaming parser work.
- [EITE](eite.md): encodings, runtime integration, and editor RPC reproduction.
- [Format detection](format-detection.md): detection parity, source/OS priors,
  and MIME metadata.
- [Installer](installation.md): installer bugs, updates, repair, and release
  manifests. The installer exists; qualify it rather than start it again.
- [Fuzzing](fuzzing.md): harness work for detection, Kaitai, compression, and
  other untrusted-input surfaces.
- [Longer-term projects](projects.md): workspace/graph, collaboration, security
  proposals, and additional tools.

## Interface and Diagnostics

- [ ] Manually verify the previously marked fixes: consistent Restart/Reload
  icon sizes, a visible debug-build badge, and the warning against using real
  accounts in debug builds. The old checkmarks did not establish verification.
- [ ] Make debug logging safer and support configurable request logging.
- [ ] Distinguish restart-required failures from normal errors such as an
  incorrect password in the Web UI.
- [ ] Include service/method and argument sizes in IPC deserialization errors
  without exposing argument contents or secrets.
- [ ] Fix iOS scrolling by tapping the top bar.
- [ ] Give missing routes a useful 404 rather than an unhelpful SPA redirect.
- [ ] Recheck directory-relative links on SPA subpages (for example
  `newsletters.rss` versus `/newsletters/newsletters.rss`); possibly already fixed.
- [ ] Correct gzip output-size estimates on the home page and show progress
  while waiting for a dynamic result.
- [ ] On failed/cancelled downloads, remove the loading overlay and show a
  `ctb.warn` that the download did not start successfully and can be retried.
- [ ] Replace button-styled non-buttons in Handlebars templates with semantic
  `<button>` elements without changing their appearance.
- [ ] Expand existing JS/frontend tests around these workflows.

## Build, Deployment, and Packaging

- [ ] Offer a Linux build without a bundled browser if bundle size still
  warrants it; the old report estimated roughly 700 MB.
- [ ] Confirm a stable build identifier is exposed in version output and use a
  sortable numeric build date in artifact names where useful. Version/JSON
  output already exists; do not implement a second version-reporting path.
- [ ] Generate remaining IPC service boilerplate in the existing build pipeline;
  decide whether generated provenance needs a date/timestamp.
- [ ] Add native Windows CI coverage rather than relying only on cross-builds
  or local VM runs.
- [ ] Update Guix packaging and regenerate it in CI. Retain/export the oldest
  supported container image for compatibility reproduction.
- [ ] Confirm Homebrew packaging works.
- [ ] Decide how to lint JavaScript licensing for LibreJS, including vendor
  files, and report newer versions of vendored dependencies.
- [ ] Remove duplicate exclude-path configuration in Deno.
- [ ] Add cache-busting to asset URLs.
- [ ] [Priority: low] Evaluate output minification without introducing unapproved dependencies:
  [HTML](https://crates.io/crates/swc_html_minifier),
  [JavaScript](https://crates.io/crates/oxc_minifier),
  [CSS](https://crates.io/crates/lightningcss), and
  [SVG](https://github.com/noahbald/oxvg) or
  [svag](https://github.com/bearcove/svag).

## Formats and Utilities

- [ ] Restore the removed semantic coordinate types in `formats/math`, using
  current Dc/derive conventions rather than blindly restoring old code.
- [ ] HQX archive support
- [ ] Implement magic-square for parity
- [ ] [Priority: low] Evaluate multi-file szip support; the format allows more than the current
  single-file workflow. Verify current support before designing an extension.
- [ ] [Priority: low] Represent arbitrary-bit FNV folding (`fnv_xor_fold`)
  through a suitable typed format/parameter API; the old report found no corresponding variant.

## Storage and Accounts

- [ ] Separate unrelated responsibilities accumulated in `db_impl` without
  replacing the existing storage backend merely to follow an old proposal.
- [ ] Define node types canonically in database metadata and a matching backed
  enum rather than maintaining inconsistent numeric meanings.
- [ ] Implement authenticated HTTPS fetch for Cloudflare through shared HTTPS
  utilities.
- [ ] Allow the local client to authenticate to a registered remote account;
  remote account registration is not currently enabled. This remains outside
  the personal MVP.