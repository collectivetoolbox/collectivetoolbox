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

## This should work:

$ ctb character_description -f ctoolbox/built/linux-x64/system_graph_history.rsrc
Error: Unable to autodetect document format for input. Specify --from with one of: [DcIntegerList, Sems, DcText, DcUtf, Dcal, Dcts, DcMixed, DcTf, DcMixed1, DcMixed2, Utf8, AsciiSafeSubset, Ascii, CscJournal]
$ ctb character_description --from DcMixed -f ctoolbox/built/linux-x64/system_graph_history.rsrc
error: invalid value 'DcMixed' for '--from <FROM>'
  [possible values: dcil, sems, dctext, dcutf, dcal, dcts, dcmm, dctf, dcmm1, dcmm2, utf8, asciisafesubset, ascii, cscjournal]

For more information, try '--help'.
$ ctb character_description --from dcmm -f ctoolbox/built/linux-x64/system_graph_history.rsrc
Error: Input bytes do not conform to valid DcMixed2 (.dcmm) format

Caused by:
    Invalid DcMixed magic header: [67, 84, 66, 82, 83]

- Bugs/to-dos observed in the above:

- It doesn't offer a permissive/forgiving parser for the document format (that should be available for any command that parses documents; they should share CLI logic). I guess it should still be strict by default probably when running these lightweight CLI commands, unlike in the yet-to-implement full workspace editor where warnings can be shown more easily/prominently and so it should default to permissive.
- Supported formats are missing a bunch of formats, including some that should be easy to support like .ctbnodes (which should parse into its respective Dc statements, essentially the same as loading the .ctbnodes file's slice of the corresponding .ctbnbdl), csc archives, and .ctbnbdl.
- Autodetection instructions give wrong instructions of what format names are accepted (it could accept either the nickname or the Rust name probably).
- Lists of possible values are not sorted. They should probably be sorted and categorized in a nice list with the description from the format database beside them.


# Installer:

- Option to associate ctoolbox with filetypes (ctoolbox-specific ones defaulted to on; others defaulted to off)
- Option to install shell completion helpers


# Broken detection:

$ ctb character-description -f assets/data/graph/85899_34592.ctbnbdl
Error: Input bytes do not conform to valid DcMixed2 (.dcmm) format

Caused by:
    Encapsulation size 192097702246800632426538688442588496000 exceeds addressable memory at offset 124783

ctbnbdl is a supported format, so it should detect the correct format using its magic. Might be worth checking for other formats that could have magic but don't.

# To fix in utf8.rs:

utf8.rs:

pub fn dcmstring_to_utf8(doc: &crate::DcMstr) -> Result<(Vec<u8>, FormatLog)> {
    dcmstr_to_utf8_with_options(
        doc,
        &DcListUtf8Settings {
            canonicalize_equivalent_dcs: true,
            ..Default::default()
        },
    )
}

This defaults to canonicalizing, rather than varying it depending on context (e.g. yes typically it should canonicalize and give warnings, but in a --strict mode (default for convert at the moment, though I might want to make --permissive the default later) it shouldn't).

# Questionable magic:


0	string		CTBASSET	Collective Toolbox Asset Bundle
!:mime	application/octet-stream
!:ext	rsrc

Any evidence this exists?



0	string		\037\237	Freeze 2.X compressed data
!:mime	application/x-freeze
!:ext	F

0	string		\037\236	Freeze 1.0 compressed data
!:mime	application/x-freeze
!:ext	F

What's the point of this? Upstream magic offers 0	string		\037\237	frozen file 2.1
0	string		\037\236	frozen file 1.0 (or gzip 0.5)
 as I've definitely pointed out before. Why does it keep showing up here?


