# Issue Inbox

Execution priorities and MVP scope:
[Dc-Native Personal MVP Roadmap](../docs/mvp-roadmap.md). This is a categorized
inbox, not an implementation order. Historical reports need reproduction before
assuming the current code still has the reported fault.

## Topical Backlogs

- Save fixtures for all pre-DcTF format versions

- A way to prevent inserting duplicate nodes (e.g. by checksum; I don't mean conceptually) into the graph would be important. Or more precisely I guess it should potentially be possible to insert them, but it shouldn't *store* something more than once (outside of deliberate/systematic redundancy to be able to recover from corruption etc.)

- Audit the codebase for bogus "Unicode-only string" assumptions on Dc documents (e.g. legacy helpers treating `DcShorthand::Unicode(...)` runs as special strings outside actual Unicode format converters instead of operating uniformly over 128-bit Dc `[string]` literals `Dc 260..Dc 261`).
- [CSC and archives](csc.md): preservation, streams, recovery, file API policy,
  native-platform qualification, and metadata privacy.
- [Dc parser](dc-parser.md): syntax validation, semantic roles, projection,
  derives, diagnostics, and streaming parser work.
- [Dc language and execution compatibility](language.md): graph snapshot pinning,
  minimal trusted kernel, bounded WASM execution, vertical migration gates, and
  future VSO language design.
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

# Theme, probably won't fix:

The glass theme has broken for some reason. I haven't changed anything related to that recently that I recall. I guess it doesn't matter that much since it only ever worked in Firefox anyway.

- Change src/formats/dcdata/data-src/README.dc-blocks.csv to be auto updated and add decimal columns (to the left of the existing hexadecimal columns, which should be secondary).

- I'd also like to make it do content-defined chunking of nodes that exceed a certain size - adding a node that contains say, >16KB? of data would slice it into sections by content (using some algorithm that is hopefully tuned to have a fairly low proportion that hit the 16KB or whatever cap - there are various algorithms for this; I think one is already used in the installer, but it's possible it's not optimal) and thus uploading a large node would actually save several smaller nodes and then one that ties them together.

# Add elif?

What do you think of adding `elif` to StageR 3? It was included in the original StageL:

```stagel
                if ne 0 n/dcBasenbUuidMonitorReprocessNeededCount
                    # Reprocessing non-UUID chars that could have been a UUID. Again, FIXME same as for the start UUID reprocessing bug mentioned in the earlier FIXME.
                    set n/dcBasenbUuidMonitorReprocessNeededCount sub n/dcBasenbUuidMonitorReprocessNeededCount 1
                else
                    # Look for a dcbasenb region end UUID.
                    if eq n/dcBasenbUuidMonitorState 0
                        if arrEq an/latestChar ( 243 188 133 185 )
                            set n/dcBasenbUuidMonitorState 1
                    elif eq n/dcBasenbUuidMonitorState 1
                        if arrEq an/latestChar ( 243 180 182 175 )
                            set n/dcBasenbUuidMonitorState 2
                        else
                            set n/dcBasenbUuidMonitorReprocessNeededCount n/dcBasenbUuidMonitorState
                            set n/dcBasenbUuidMonitorState 0
                    elif eq n/dcBasenbUuidMonitorState 2
                        if arrEq an/latestChar ( 244 136 161 186 )
```

I feel like it might make certain patterns more elegant:

```sl3
    if isWideningConv /srcTy /dstTy
        do
            if and le /sk u8:7 le /dk u8:7
                do
                    emitShort /out u32:2541.
                    lowerExpr /out /env /mScope /fctx /selfNsKey /uses /stdImports /crossDcs /synths /gensym /srcExpr.
                    emitFormat /out scalarFormatId /dk.
                    return.
                ::
                nop.
            if eq /dk u8:8
                do
                    const u64/lid if eq /sk u8:3 u64:8589980697 if eq /sk u8:4 u64:8589980694 if eq /sk u8:5 u64:8589980696 if eq /sk u8:6 u64:8589980693 u64:8589980695.
                    emitStdCall1 /out /stdImports /selfNsKey /uses u8:1 /lid /env /mScope /fctx /crossDcs /synths /gensym /srcExpr.
                    return.
                ::
                nop.
            if eq /dk u8:9
                do
                    const u64/lid if eq /sk u8:3 u64:8589980751 if eq /sk u8:5 u64:8589980750 u64:8589980749.
                    emitStdCall1 /out /stdImports /selfNsKey /uses u8:2 /lid /env /mScope /fctx /crossDcs /synths /gensym /srcExpr.
                    return.
                ::
                nop.
```
I think this is basically an elif chain, but it had to be written as a series of ifs with nop else parameters. (That particular sample is StageL 3 rather than StageR 3 so it wouldn't help that particular case, but it seemed like a good example of what I mean.)

Basically:

```sr3
if gt /i u64:0
    nop.
elif gt /i u64:0
    set /i 2.
    nop.
```

would desugar to

```sr3
if gt /i u64:0
    nop.
    if gt /i u64:0
        set /i 2.
        nop.
```

or something like that. Does that seem like a good idea?

# Possible further performance tuning for WASM substitutes:

*(Note: Inline `mem0` fast-path access on `SubstituteVm`, `opt-level = 2` in dev/test, auto-discovery of runnable System-block snapshots, and Merkle-deduplicated shared-function codegen across snapshots are now implemented in `src/formats/wasm_substitute_rt/vm.rs` and `src/formats/dcstring/converters/wasm_to_rust.rs`.)*

1. **`#[inline(never)]` threshold on medium helpers ([wasm_to_rust.rs](file:///workspaces/ctoolbox/src/formats/dcstring/converters/wasm_to_rust.rs#L48)):**
   `INLINE_NEVER_BYTECODE_THRESHOLD` is currently `128` WASM bytes to keep compile times fast. In release builds (`opt-level = 3`), `#[inline(never)]` still prevents LLVM from inlining helpers larger than `128` WASM bytes inside tight loops; if profiling shows hot medium helpers, consider raising the threshold or restricting `#[inline(never)]` to `tail_dispatch_*` and genuinely large functions.
2. **Per-checkpoint `deduct` struct traffic ([vm.rs](file:///workspaces/ctoolbox/src/formats/wasm_substitute_rt/vm.rs#L559-L574)):**
   `self.vm.deduct(host, 1)` reads/writes three `u64` fields (`op_ordinal`, `quota_remaining`, `quota_consumed`) through `&mut self.vm` on every loop/call/stride checkpoint, and checks `op_ordinal < resume_target_op`. Still vastly faster than `tinywasm`, and likely secondary to memory-access overhead, so only worth tuning (while preserving deterministic V2 checkpoint-ordinal metering) if profiling shows it's hot.

# Output caching

Proposal: temporarily instrument all Dc routine invocations (maybe some special runner so it can be updated periodically?). Run some sample documents, etc, recording function call counts, parameter values, execution times, and return values. Create a bundle of the most frequently occurring and most expensive calls and their return values, and ship that with the application as part of the runtime (maybe in a separate .ctbnbdl file so it can be omitted as an optional installation component to save disk space).

# SL3/SR3/EM3 linters

- Line length lint to limit to 79 chars (SL3/SR3 only, not for EM3)
- Function length lint to limit to 300? 500? lines
- Version of the unwrap-or linter (I wish this weren't needed, but I'm not sure how else to prevent agents avoiding implementing correct edge case handling than making them annotate every case and then reviewing them in separate passes - if there's some better way to actually structurally prevent illogical unwrap-or I'd prefer that, but since it's ultimately a type of logic error I'm not sure that's possible since there's not really a clear heuristic for when it's used appropriately or not).

# Runtime and renderer architecture

- This needs to be moved into EM3/StageR code. Rendering logic also needs some work - most notably it relies on semantic-to-text transformation *in* the renderer, instead of constructing the UI in the runtime and then rendering that. It should reject Dcs in frames coming out of the runtime that are not actually drawable objects - at the moment, that means graphical characters and relevant control sequences, and later, drawing primitives, etc for GUI.
- It should be refactored to organize the code. Currently, impls/std has "teletype.sr3" (which also happens to include an implementation of semanticToText transformation according to the docblock) and "convert_dc_string_to_wasm.sr3". It should use a neat structure, probably something like: impls/std/formats/transformations/dc_to_wasm.em3, impls/std/formats/transformations/semantic_to_text.em3, impls/std/formats/encoding/{unicode,ascii,etc}/from_dcs.em3, impls/std/formats/terminal/teletype.em3, impls/std/runtime/runtime.em3 (runs the document and emits frames in Dc-document format - applying any needed transformations for display like semanticToText - though running semanticToText unconditionally over the entire document does not seem right - in a viewer, for instance, unless a document is *only* consisting of semantic statements, I'd expect the semantic statements to be in an "inspector" or "document properties" type of window, like in LibreOffice or iWork, rather than output as if they're part of the document), impls/std/renderer/renderer.em3 (renders frames to output format - teletype, videoterminal, raster, Web, VR, etc - this is where validation that they're actually rendered documents would live. This would I imagine call out to document transformations to reduce dimensionality, like flattening 3D or higher-dimensional coordinates to 2D for display on screen).
- convert_dc_string_to_wasm.sr3 should be smaller. It should be implemented as document transformations: Dc string -> StageR 3 (which I assume is most of what's in there now), StageR 3 -> StageL 3, StageL 3 -> WASM. In other words, I'd expect its implementation to be basically just a few lines, and the bulk of it to be in impls/std/formats/transformations/.
- The function load_executable_dc_document in runtime.rs is a reinvented wheel. In the short term, it should get rid of the word "executable" as all Dc documents are executable and so it makes no sense. Also, since it converts documents to the native Dc format but doesn't require them to start that way, the word "dc" in its name should also go away. It should just be called "load_document", and be, I think, a trivial wrapper around the existing conversion API in src/formats/conversion. It also looks like it doesn't use the standard CLI operations for document format handling, though I may be overlooking it. In the longer term, (a) it should be in StageR 3; (b) it should be using impls/std/formats/conversion/conversion.em3, which should be the same intent as src/formats/conversion/conversion.rs.

# VM test script excludes various crates from tests by platform, but that seems like the wrong place for it. Also, to the greatest extent possible, all crates should be tested on all platforms (e.g. the exclusion of X11 tests on FreeBSD doesn't seem right, as X11 can run on FreeBSD, though I don't think it's set up - yet - on that particular test VM.) I'd also prefer to use per-platform #[ignore] on the *specific tests* that are impossible to run on a platform (even if in effect it means it runs no tests, I can at least be confident that if one's added later that's not platform-specific, it'll still run), rather than leaving out entire crates in the test script.

# Only split to live and inactive nodes outside of what's tracked in repo

Make the .ctbnbdl files that are in-repo append-only. Have a separate split/ folder that has the live and historical .ctbnbdl files that are gitignored.