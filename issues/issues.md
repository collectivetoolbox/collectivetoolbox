# Issue Inbox

Execution priorities and MVP scope:
[Dc-Native Personal MVP Roadmap](../docs/mvp-roadmap.md). This is a categorized
inbox, not an implementation order. Historical reports need reproduction before
assuming the current code still has the reported fault.

## Topical Backlogs

- Save fixtures for all pre-DcTF format versions

- A way to prevent inserting duplicate nodes (e.g. by checksum; I don't mean conceptually) into the graph would be important. Or more precisely I guess it should potentially be possible to insert them, but it shouldn't *store* something more than once (outside of deliberate/systematic redundancy to be able to recover from corruption etc.)
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

- Unframed .dctf (f641, 2228865 : DcTF [document] {.dctf}) vs .dcutf (f89, 2228313 : DcUtf [document] {.dcutf}) content sniffing: As noted by the reviewer, raw .dctf has no magic header and overlaps UTF-8 and DcUtf on byte prefixes, so format detection relies on extension/explicit format selection rather than magic bytes.

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
- [ ] Add test for getting help function (from `5e3a561bb:issues/issues.md.backup:6`, distinct from existing `get_test_help_bytes` asset bundle test if referring to CLI/UI help retrieval).
- [ ] `1114370 : <Dc> Begin parameter {syntax: `:~ [value] 259`}` character description output should use <Dc 123> or <Dc f521> etc. to show the shorthand too.

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
- [ ] Address cross-platform full-suite test blockers:
  - Headless Linux doctest failure in the software-renderer GUI example when `WAYLAND_DISPLAY`, `WAYLAND_SOCKET`, and `DISPLAY` are unset.
  - WINE `build-support` `initrd` test failure attempting to execute uninstalled guest `rustc`.
  - Native Windows 7 full-workspace test suite 1,800-second timeout during JavaScript tests.
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

- [ ] Revisit whether `BaseAlphabet` (`f359`, `2228583` in `formats/number.csv`: "A set of characters used in a positional number system") and `BaseAlphabetVariant` (`f613`, `2228837` in `formats/format_variant.csv`: "Base alphabet variation") are duplicates or should remain distinct.
- [ ] Restore the removed semantic coordinate types in `formats/math`, using
  current Dc/derive conventions rather than blindly restoring old code.
- [ ] HQX archive support
- [ ] Implement magic-square for parity
- [ ] [Priority: low] Evaluate multi-file szip support; the format allows more than the current
  single-file workflow. Verify current support before designing an extension.
- [ ] [Priority: low] Represent arbitrary-bit FNV folding (`fnv_xor_fold`)
  through a suitable typed format/parameter API; the old report found no corresponding variant.
- [ ] Cherry-pick `30f41d4a5` or implement from scratch

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

# Installer:

- Option to associate ctoolbox with filetypes (ctoolbox-specific ones defaulted to on; others defaulted to off)
- Option to install shell completion helpers

# Startup is slow again

$ /home/a/.local/bin/ctoolbox
[2026-11-03T05:01:44.340425Z WARN environment.rs:1265:27 log workspace:497753/workspace] Local IPv6 discovery failed: Failed to route to target for local IPv6 discovery: Network unreachable (os error 101)
[2026-11-03T05:01:44.342571Z WARN asset_bundle.rs:614:13 log network:497767/network] ("v86 resource bundle not found at {}; v86 VM assets will not be loaded", "/home/a/.local/bin/v86_images.rsrc")
[2026-11-03T05:01:44.342570Z WARN asset_bundle.rs:614:13 log storage:497768/storage] ("v86 resource bundle not found at {}; v86 VM assets will not be loaded", "/home/a/.local/bin/v86_images.rsrc")
[2026-11-03T05:01:44.342593Z WARN asset_bundle.rs:614:13 log io:497766/io] ("v86 resource bundle not found at {}; v86 VM assets will not be loaded", "/home/a/.local/bin/v86_images.rsrc")

[2026-11-03T05:02:08.075927Z WARN environment.rs:1272:27 log workspace:497753/workspace] Public IPv4 discovery failed: Failed to ensure CRLite cache is ready: GET https://CollectiveToolbox.com/crlite/manifest.json: Failed to GET https://CollectiveToolbox.com/crlite/manifest.json: error sending request for url (https://collectivetoolbox.com/crlite/manifest.json): client error (Connect): dns error: failed to lookup address information: Try again
[2026-11-03T05:02:31.691332Z WARN environment.rs:1276:27 log workspace:497753/workspace] Public IPv6 discovery failed: Failed to ensure CRLite cache is ready: GET https://CollectiveToolbox.com/crlite/manifest.json: Failed to GET https://CollectiveToolbox.com/crlite/manifest.json: error sending request for url (https://collectivetoolbox.com/crlite/manifest.json): client error (Connect): dns error: failed to lookup address information: Try again
[2026-11-03T05:02:55.318050Z WARN environment.rs:1283:21 log workspace:497753/workspace] Server clock offset discovery failed: Failed to ensure CRLite cache is ready: GET https://CollectiveToolbox.com/crlite/manifest.json: Failed to GET https://CollectiveToolbox.com/crlite/manifest.json: error sending request for url (https://collectivetoolbox.com/crlite/manifest.json): client error (Connect): dns error: failed to lookup address information: Try again

This is all supposed to be hard-capped at 5 seconds or something like that. It would also possibly be good to open the UI *before* the update check and environment capture is complete so it doesn't take so long.

Unrelated, the glass theme has broken for some reason. I haven't changed anything related to that recently that I recall. I guess it doesn't matter that much since it only ever worked in Firefox anyway.


ctoolbox convert -f /some/ascii/file --to sems # no errors, no warnings
ctoolbox convert -f /some/ascii/file --to sems --strict # no errors, no warnings
ctoolbox convert -f /some/ascii/file --to sems --strict --normalize none # no errors, no warnings
ctoolbox convert -f /some/ascii/file --to sems --permissive --normalize none # no errors, no warnings

Docs do not explain which is the default:

      --permissive                  Parse and export document in permissive/forgiving mode (recovering from invalid headers, characters, or checksums with warnings)
      --strict                      Parse and export document in strict mode (rejecting invalid headers, characters, or non-canonical sequences)
      --normalize <NORMALIZE>       Character and Unicode normalization policy (none, as-needed, all) [possible values: none, as-needed, all]

ASCII is not representable in SEMS without normalization. --strict should exit 1 as the document is not representable, though I guess it can still run the conversion. --permissive --normalize none should turn everything into a replacement character. Only cases where this should work are `--permissive` (defaulting to as-needed normalization), `--permissive --normalize as-needed`, `--permissive --normalize all`, or `--strict --normalize as-needed` or `--strict --normalize all` (--strict should default to no normalization). I think `--permissive --normalize as-needed` should probably be the default. Adding --strict should turn off normalization, unless a normalization-on option is specifically requested on the CLI. If any characters were normalized, warnings should be shown on stdout (in as-needed mode, a warning for each normalized character; in all mode, a single note).
