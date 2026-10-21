Execution priorities and MVP scope: [Dc-Native Personal MVP Roadmap](../docs/mvp-roadmap.md).
This file remains the issue inbox; its ordering is not the implementation order.

[Probably done but need manual testing]:
- [x] Icons do not have consistent size (compare "Restart" and "Reload")
- [x] Add a clear badge for when it's running in debug mode, and a warning about not logging in with your real account
------


- [ ] Make the debug logging less of a footgun somehow?
- [ ] Web UI: Errors should have a clear way of knowing whether they are serious errors that require a restart, or "normal" errors like password didn't match.
- [ ] Support for request logging as a configurable option
- [ ] Make sure to have a build with no bundled browser for linux if it stays 700mb
- [ ] Scrolling by tapping top bar doesn’t work on iOS
- [ ] Minifying middleware?
	- [ ] HTML: https://crates.io/crates/swc_html_minifier
	- [ ] JS: https://crates.io/crates/oxc_minifier
	- [ ] CSS: https://crates.io/crates/lightningcss
	- [ ] SVG:
		- [ ] https://github.com/noahbald/oxvg
		- [ ] https://github.com/bearcove/svag
- [ ] Format build date nicely as a sortable numeric string and include in file names maybe instead of the commit
- [ ] Implement HTTPS authenticated fetch for Cloudflare
- [ ] Maybe: "tighten the logging so the next IPC failure reports which method failed to deserialize (service/method + arg sizes)"
- [ ] Generate remaining IPC service boilerplate in build.rs and include date/timestamp in the generated files
- [ ] 404 is not nice looking now it redirects to the SPA
- [ ] SPA pages in subdirectories can't use directory-relative links (e.g. newsletters.rss (doesn't work) versus /newsletters/newsletters.rss (works))
  - [ ] This might actually work now. Need to check
- [ ] MacOS build possibilities:
  - https://actually.fyi/posts/zig-makes-rust-cross-compilation-just-work/
  - "Zig actually provides libSystem.tbd stubs out of the box, so you can cross-compile from any OS to macOS too as long as you only depend on libSystem (the macOS libc.)" - emidoots at https://news.ycombinator.com/item?id=30488979
- [ ] Add OpenGL/Vulkan: Approach it by statically linking LLVM and Mesa software rendering (LLVMpipe/Lavapipe), and delivering separate, optional minimal shim binaries that attempt to dlopen the system X11/libwayland/libGL, proxying over IPC. On other platforms, I'll likely use some dynamic linking, with a preference for statically linking where possible.

### csc

- Add a warning that it records additional metadata and to be careful when considering whether to share the journal/archives unless --no-metadata is passed

- Dc-mixed-serde.rs:     // FIXME doesn't look right. 422 is File flag: read only; 423 is File flag: reparse. Name and format list should proabbly be alternate representations, and 342 the overarching type for this, maybe?

Windows-style streams (:Zone.Identifier)

- ddrescue type recovery

- Csc archiver with verification pass, quick indexing, and being able to materialize/verify a file into an archive; rsync equivalent by sending an archive and ctb binary and then unpacking it on the other end?

- file/identity.rs has an implementation of a list of disallowed Windows names, duplicating clean-name.

- formats/detection/source.rs (and formats/detection/detection.rs): // FIXME: Make sure this detection uses the OS (where file was observed) data from the file struct for OS hints. May also be good to support retrieving the enclosing archive type from the file (in the case of a file that is being detected while it's within an archive) to also use as an OS hint, since archive formats are associated with OSes. (The OS from an archive type is probably a stronger signal than the OS where the archive is observed/unpacked.)

- // Infer platform prior from Apple metadata, enclosing archive format, or recorded OS environment
        let platform = if self.metadata.apple.is_some() { in file/entity.rs should use OS association data from formats CSV

- Windows runner in GitHub actions to run test suite?

- Restore semantic definitions of coordinate types in formats/math (removed recently; could probably find it in a commit, though a new implementation would need to be different)

- Use NSURLIsPackageKey if available, or NSWorkspace::isFilePackageAtPath for older Mac or older GNUstep API, rathre than hard-coding a list of bundle names.

- [ ] Implement fuzzing harness for format detection, Kaitai, and compression (see [fuzzing-assessment-and-plan.md](fuzzing-assessment-and-plan.md))

FNV: //! FIXME: They can be folded to arbitrary bit lengths (fnv_xor_fold) but
//! there's no FormatId or parameterized format variant for that.


- szip docs say the file format supports multiple files, but the program doesn't. It might be something that could be implemented.


- Make ambiguous syntax definitions a validator error.
- Make ambiguous derive macros for Dc serde a build-time error.

- Evaluate adding @ident for Dcs used in structs too, to make errors more obvious?

- all encoding formats really @imply f14 (Text), but aren't noted as such. I guess that's because knowing that doesn't really inform processing of the files.

- f80 String and f14 Text are basically the same thing, though the context/connotation are different. Confusing, but not sure exactly how best to address.

- format syntax rule changed from :[f0..f1114111] to :[f0..f632]. The token range for format references is now capped at 632, the highest defined FormatId, instead of spanning the whole Dc space. From e378fd1bc "wip: Continue Dc parser refactoring". Is this right/necessary? I mean, those are all the *recognized* formats. I guess it might be helpful if the formats block were later shortened, but that seems somewhat unlikely now that it's allocated. If it's useful, the limit at f632 should be derived automatically at build time/when running the validator, so it doesn't get stale.

- Add optional, recommended TOTP step on login

- csc archive: Durable destination identity and interrupted moves
  Status: Partially Implemented
  - Remaining Gaps & Pending Work:
    - Stop silently discarding `restore_applied_directory_ownership` errors in the copy pipeline; propagate or explicitly report receipt restoration failures.
    - Qualify resume across deep subtrees with preexisting uncommitted regular files under `ExistingEntryPolicy::Refuse`, without overwrite. The receipt regression only resumes a previously completed empty subtree after adding a source file, not this crash window.
    - Add durable source inventory and per-source deletion/recovery state for interrupted multi-file moves. `run_mv` currently cleans up using in-memory `copied_entities`, without a durable deletion transaction log.
  - Deferred Items: Live ReFS volume testing is outside this review's required scope. The Windows directory identity helper already queries `FileIdInfo` with a 64-bit fallback, and `InodeKey`/receipt serialization supports 128-bit IDs; preserve that extension boundary.

- "When an alias has no syntax of its own, it will inherit the equivalent token’s rule." I'm not convinced about that. It should be something like: if you need to check "Is this a list opener?" in the serde implementation, it should add a new codegen rule that fills in a table/function holding list openers. Would just be one Dc long in many cases, but then if I'm adding a Dc with an <equiv> relationship, it will be treated as *logically equivalent* to the one that was initially targeted. If an <equiv> does *not* have the exact same syntax, I think that should be an *error when running the validation script*, and so this extra rule is not correct.

- You could use alternative representations basically anywhere. I do see they both show up in one syntax rule, but that looks like a mistake since I'd expect them to be accepted in other cases too. I mean if we have file flags that's one case where they're useful, but once there are multiple date formats supported (in future) I would expect 378 "Timestamp clock resolution" to allow me to fill in :~ [datemagnitude] with, for instance, the value represented in two different time systems, using 397, if I wanted. In other words, it's kind of implied syntax that you can put anywhere it's relevant, just like in 374 "File data extent" `:~ 376 [number:Natural0] 377 [number:Natural0]` it would be valid to write  `374 376 255 377 [some number...] 377 [some number...]`, even though 255 isn't stated to be allowed there, it automatically is because the *point* of it is to allow using Dcs in places where they'd collide with other syntax, like that. Please make sure these cases are handled correctly.

-"parsing currently accepts negative numbers for Natural0"

- "The native archive tests use best-effort metadata restoration, so their pass does not establish strict fidelity for every Windows metadata field." There should be at least as thorough tests that run in default mode (similar to strict-lossless mode, but allowing for differences that *literally cannot* be materialized under the target, like ctime on Linux) too. Please check that there aren't other tests being cheated by using best-effort mode when there's not a corresponding normal-mode test. Please also make sure that the three-way distinction between default mode, best-effort mode, and full strict mode is correctly maintained. The use of best-effort mode in archive tests without at least as many corresponding default-mode tests is only present due to LLM cheating the requirements; it does not represent operator intent.

```

                                             #crate_root::navigator::DcSyntaxKind::Container { opener, .. }
-                                                if matches!(opener.to_short_dc(), ::std::option::Option::Some(397 | 561 | 310)) =>
+                                                if matches!(opener.to_short_dc(), ::std::option::Option::Some(397 | 561)) =>
                                             {
                                                 found = ::std::option::Option::Some(cid);
                                                 break;
@@ -3104,7 +3104,7 @@ fn expand_derive_dc_mixed(input: &DeriveInput) -> syn::Result<proc_macro2::Token
                                                 }
                                             }
                                             #crate_root::navigator::DcSyntaxKind::Container { opener, .. }
-                                                if matches!(opener.to_short_dc(), ::std::option::Option::Some(397 | 561 | 310)) =>
+                                                if matches!(opener.to_short_dc(), ::std::option::Option::Some(397 | 561)) =>
                                             {
```

Obviously wrong; non-data-driven; using bare Dcs in code.

- should MaterializeOptions have a best-effort option?

- ```
/// Reads file metadata JSON and rematerializes the file to the given destination path.
pub fn rematerialize_from_metadata_json(
    json_str: &str,
    dest_path: &Path,
) -> Result<MaterializeReceipt> {
    let record = FileMetadataJson::from_json_str(json_str)?;
    let mut options = MaterializeOptions::default();
    options.force_overwrite = true;
    options.copy_specials = true;
    record.rematerialize(dest_path, &options)
}
```

Hm, does this correspond to the CLI? Why force_overwrite true? Generally things shouldn't default to forcing overwrite. Are CLI options applied? Copy-specials isn't normally default either I don't think. TBD where this is used. Possibly its callers should just set the options they need.

- csc archive: Bounded-memory attached-stream pipeline
  Status: Partially Implemented
  - Remaining Gaps & Pending Work:
    - Extend AppleDouble/AppleSingle conversion writers to consume nonresident attached bodies through `AttachedStreamSource`. Materialization currently rejects these conversion modes before mutation when bodies are nonresident, rather than silently omitting them. Native stream capture, archive writing, lazy projection, and pre-publication extraction use the streaming path.

- impl ctb_formats_dcstring::DcMixedNumber for FinderLabel { This should be using the derive attribute Dc serialization I think. Ad-hoc serializers aren't permitted, unless there's an *extremely* compelling reason.

- Lazy AppleDouble/AppleSingle conversion remains open

- Are Alternate Data Streams, HFS forks, Windows Extended Attributes, and xattrs on Linux all supported correctly? Will it risk mixing up EA and ADS on Windows? Do large ADS transfer correctly between ZFS (testable on FreeBSD), AppleDouble/AppleSingle, and NTFS (on Windows 7 and 11)? A good practical test (add a test script to run this whole process, probably, so it can be re-tried and variations on it added) would be to create a few files with odd, mangled, and differing-only-in-normalization Unicode filenames, each file having a few large (>512kB) streams with odd, mangled, and differing-only-in-normalization Unicode stream names on FreeBSD, and pack them into a `csc archive` there. Copy that out of FreeBSD and save it as a test fixture. On Linux, extract it using AppleDouble files. Then, `csc archive` that folder, with reading AppleDouble files turned on, re-joining the streams to their files. Copy that archive into Windows 11, and extract it. Confirm that the streams were extracted as ADS. `csc archive` it there. Copy that back out. Unpack it on FreeBSD, then do a `ctoolbox csc-diff` against the original files. The test script should ignore expected differences (inode, ctime) and report anything that got mangled or broken in the process.

- Phase 4 Reader Migration (memories/dc-document-parsing.md Section 4):
Replacing hardcoded == token matching with dynamic resolver / DcRole queries in src/formats/dcdata/dc_char.rs, src/formats/dcdata/dc_number_minimal.rs, and src/formats/dcdata/format_spec/dc_stream.rs.
Replacing legacy constant allowlists in src/formats/dcstring/navigator.rs with peek_fills_role / next_if_fills_role over dynamic resolvers.
Complex Expression Splicing inside Projections:
DcProjectionContext::resolve_expression_node resolves bound identifier references (Dc 540), named objects (Dc 276), and routine invocations (Dc 279, Dc 256) containing parameter frames (Dc 258..Dc 259). Fully dynamic runtime evaluations that generate synthetic AST nodes not already present in the CST arena are handled through eval_stream_splice in DcDocumentNavigator rather than synthetic node synthesis during projection.
3. Deferred Items
Synthetic AST Node Materialization from Runtime Evaluator:
If a routine invocation produces an arbitrary dynamic document stream rather than wrapping an argument/parameter, that stream is spliced during parsing via DcDocumentNavigator::eval_stream_splice. Directly materializing synthetic AST nodes inside DcProjectionContext was deferred as CST arena nodes are borrowed from the immutable parsed document.

- **Pre-existing Failure in `ctb-formats-dcdata`**:
  - [`syntax::resolver::tests::test_rule_and_delimiter_pairing_when_non_canonical_or_target_owns_rule`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/resolver.rs#L1474): panics at line 1474 with `left == right` (`Some(1114423)` vs `Some(1115072)`). This test was introduced in prior commit `d015a7d` where `target_open` and `alias_open` cloned `list_open.syntax` containing closer `311` (`DC_END_LIST` / `1114423`) in their pattern, which conflicted with the test's declared `@closes` target `960` (`1115072`).

### Known Gaps & What's Left
- Multi-token `<equiv>` decompositions targeting combinations of formats/characters (e.g. `<equiv>f350 f351`) are forbidden from defining single-token syntax rules or `@closes`, which is actively validated, but multi-token decomposition equivalence resolution itself is handled downstream in parser stages rather than unified in the single-token disjoint set. [not sure what this means]


- Regenerate Guix packaging in CI

Pull and export oldest container

File flags did not have complete implementation

magic-square

- The parsing and serialization/deserialization of Dc documents are currently implemented in a few different ways in different places, I think. (1) What can be DRYed out from them? (2) To what degree can the parser and ser/de implementation be automatically generated from the 'Syntax' specifications in the database, rather than implemented in ad-hoc ways? Can we refactor it to be more data-driven - so when deserialzing for instance, it parses the document from the journal using rules generated from the syntax in the database, and then projects that onto the structs that it's deserializing into, logging/warning/erroring (depending on what the caller, e.g. the csc journal module, specifies)? That feels like it would be a lot cleaner than having two or three separate parsers that work in different ways, and a deserializer that can't handle garbled or outdated formats robustly and elegantly.

  - Agent update 18 Oct 2026: " Grammar tables are generated from the CSV Syntax definitions; parsing builds an arena, then derives project it onto Rust types under caller-selected diagnostic policies. See syntax.rs:61 and navigator.rs:1865. However, handwritten encapsulation parsing and specialized serializers remain. Keep a narrower issue to audit those remaining paths for duplication; fully generating serialization from Syntax is not complete."

Cherry-pick 30f41d4a5

Add a data matching tool. See 2026-08-20-Matching and Linking.txt

License lint JS files LibreJS - how to deal with vendor files?

HQX

db_impl has accumulated a whole bunch of unrelated concerns.

Include optional verification step in compression (default when writing to a file?)

node type should be an enum in database and backed enum with same values in code.

Incorrect file size estimates on home page for gzip. Add spinny loading indication that takes up the time until a dynamic responder is necessary .

Regarding the overlay shown in the web UI: If a download fails (e.g. by pressing escape key after clicking the link, which interrupts it), it should hide the loading overlay and show a `ctb.warn` saying "Sorry, it looks like your download did not start successfully. You may try again if you like."

Consider linting for whether there are newer versions of vendored packages.

Some clipboard bugs on Wayland (haven't tested on X11 yet): Copying to the clipboard with Control-C works, but pasting doesn't seem to do anything. Dragging with the mouse around text to get it into the middle mouse button buffer also doesn't work (though middle-click to insert from the buffer works if another application has set something).

Add JS/frontend tests.

- [ ] Please check the Handlebars templates for all elements with button styling classes, and without changing their appearance, change them to <button> elements so that they respond as expected for buttons.

Confirm Homebrew packaging works.

- [ ] Avoid need for duplicate exclude paths in deno.json.

Cache busting parameters on URLs.

Update Guix packaging.

EITE edit RPC bug:

```
POST

scheme
	http
host
	127.0.0.1:9000
filename
	/api/rpc/eite


{"function":"importDocument","args":["utf8",[244,143,191,187,239,160,129,244,143,191,182,239,160,129,244,131,157,169,239,159,185]]}


from_code_point: unknown code point [63489]
```

- Potentially merge MIME types databases in mime crate, and also load the MIME data from Magdir.

Csc copy fifo

Don’t save environment in archive by default? Or maybe just show warning. Otherwise galse sense of security. Though any low metadata mode would still overlook in file metadata

Enviornment detection All network interfaces; MAC address; Add api in io/system for that?

### Installer

[Installer issues](installation.md).

### Remote Accounts

- [ ] Allow the local client to authenticate to a registered remote account;
  remote account registration is not currently enabled.

- Tests for io/file and csc:
    - Exercise native Windows link/stream materialization and Windows-only test
    branches. Windows-source roundtrips on Linux do not establish native support.
    - Exercise encoding-restrictive filesystems directly, including strict refusal
    and explicit best-effort replacement. Existing checks covered conversion
    logic and Unix raw-byte replication on the host filesystem.
