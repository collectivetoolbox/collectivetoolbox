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
    - Make applied-directory receipts crash-durable: `StateJournal::update_desc_file` writes and renames `.cscdesc` without syncing the file or its containing directory. The journal batch is synced separately, so loss of the descriptor can lose resume authority.
    - Stop silently discarding `restore_applied_directory_ownership` errors in the copy pipeline; propagate or explicitly report receipt restoration failures.
    - Qualify resume across deep subtrees with preexisting uncommitted regular files under `ExistingEntryPolicy::Refuse`, without overwrite. The receipt regression only resumes a previously completed empty subtree after adding a source file, not this crash window.
    - Add durable source inventory and per-source deletion/recovery state for interrupted multi-file moves. `run_mv` currently cleans up using in-memory `copied_entities`, without a durable deletion transaction log.
  - Deferred Items: Live ReFS volume testing is outside this review's required scope. The Windows directory identity helper already queries `FileIdInfo` with a 64-bit fallback, and `InodeKey`/receipt serialization supports 128-bit IDs; preserve that extension boundary.

- csc archive: Bounded-memory attached-stream pipeline
  Status: Partially Implemented
  - Remaining Gaps & Pending Work:
    - Connect lazy stream capture, encoding, and projection end to end. Windows `read_and_hash_streams` uses `std::fs::read`; `AttachedStream::data` decodes Dc 385 into `Vec<u8>`, while its lazy `payload` field is skipped by the derive. The separate `AttachedStreamWithPayload` does not replace that production path. Large stream bodies are saved, but not with bounded memory.
    - Repair `AttachedStream::encode_to_writer_with_stream_reader`: prefix, tag, and suffix use `DcMstring::as_bytes()`, emitting extra document headers inside the record. A temporary comparison against `encode_with_stream_reader` failed on 2026-10-18. The existing incremental-stream test does not exercise either encoder; `encode_with_stream_reader` itself still accumulates the document in memory.
    - Native Windows ADS/reparse runtime qualification for the reviewed revision remains unverified here. Earlier claims of successful Windows 7/11 qualification conflict with the old pending item and were not established by the Linux tests. Live ReFS testing is not required.

- csc copy-journal unexpected-body warnings
  Status: Partially Implemented
  - Remaining Gaps & Pending Work: Stream-body warnings inspect only immediate streams in `read_journal_snapshot_with_policy`, not nested stream entities. Calling public `read_v4_journal_snapshot` directly also bypasses that stream-body warning pass. Keep primary-body and stream-body warning coverage consistent across reader entry points.

Medium: Stream writer emits extra document headers. A temporary byte-comparison probe failed. streams.rs:536
Medium: Unsupported checksums error instead of warning in forgiving mode. Reproduced with MD5. encapsulation.rs:304
Medium: Directory receipts are renamed without fsync, so crash-durable resume authority is not assured. journal.rs:881
Medium: Windows ADS capture still buffers entire bodies; end-to-end bounded-memory streaming is incomplete. windows.rs:120


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

The UI hangs while loading the manifest data. Could you: (1) add a background HTTP request to start loading the manifest when the installer first opens so the user hopefully doesn't need to wait as long for it, and (2) have some sort of "Please wait" screen if it's not ready by the time the user reaches a step that depends on it, so the UI doesn't seem to hang?

Consider linting for whether there are newer versions of vendored packages.

Some clipboard bugs on Wayland (haven't tested on X11 yet): Copying to the clipboard with Control-C works, but pasting doesn't seem to do anything. Dragging with the mouse around text to get it into the middle mouse button buffer also doesn't work (though middle-click to insert from the buffer works if another application has set something).

Cancelling installation just makes it start the download over, rather than stopping it and rolling back any installed files.

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

Hi! I'd like you to work on some issues in the installer:

- Keyboard input is still somewhat laggy, and frequently keys "stick" when pressed. (Mouse input, by contrast, is relatively smooth, and un-"stick"s the stuck keyboard event - keyboard events work smoothly if I'm continuously moving the mouse.)
- The license text view tabbing is messy - it's technically possible to tab to close button, but it also tabs to a lot of other invsible stuff. It also opens off-screen, so parts of the license modal are not visible.

- Navigating the file tree by arrow keys isn't working: it appears to do *something* sometimes when I press the arrow keys, but the effects seem unrelated to the keys I'm pressing. Pressing Up should move the selected item to the preceding entry in the current column. Pressing Down should move to the following entry in the current column. Pressing Down when at the last selectable item in a column should do nothing; similarly for pressing Up when at the first. Pressing Right should move focus to the top item in the next column to the right. Pressing Left should move focus to the enclosing directory in the column before the currently selected column. (In other words the Left key should do the same thing as the Up toolbar button.)

- Home, End, Page Up, and Page Down aren't doing anything.

At narrow screen sizes (e.g. mobile portrait mode), the installer's file picker exhibits bugs. I suspect these are all *symptoms* of a single bug:

  - A gray right margin appears, getting larger as the window gets narrower, taking up precious space.
  - An alternative way to look at that problem is that some widgets move too far to the left. I suspect that may actually be what's happening.
  - Pointer events (hover, click) are misaligned with the UI elements underneath them.
  - Buttons in the Places panel become unaligned with the Places header text, and jiggle when I mouse over them.
  - The text in the Path text input box moves to the left, winding up starting about a quarter inch to the left of the white input control background, and drawn on top of the right side of the "Path:" label.

Could you investigate what the root cause of this broken behavior is, and correct it? I'm not sure if it's in my code or if it may be an underlying bug in egui itself when space for widgets is constrained.

- Scrollbars are very narrow.
- Scrollbars have no buttons.
- Scrollbar gutters are not clearly visible.
- Clicking in the scroll gutter jumps to that point even with a left click. A left click should scroll by a screenfull; a middle click should jump to that point.
- One cannot shift-tab back out of the "Installation directory" textbox. It does nothing.
- Shift-tab has a different problem in the file picker: instead of moving to the previous control, it simply acts as if I'd pressed tab without shift and moves to the next control.
- Arrow keys do not navigate up and down in the file picker columns reliably. Often, they move to somewhat arbitrary other controls outside of the columns: for instance, the toolbar, or folders in the Places panel that are *not* the parent of the current folder.
  - When they do successfully navigate up and down, they move the focus, but don't actually select the focused item, which is confusing.
  - Pressing Up should select the preceding entry in the current column from the current selection.
  - Pressing Down should select the following entry in the current column from the current selection.
  - Pressing Down when at the last selectable item in a column should do nothing; similarly for pressing Up when at the first.
  - Pressing Right should move focus to the first item in the next column to the right.
  - Pressing Left should move focus to the enclosing directory in the column before the currently selected column. (In other words the Left key should do the same thing as the Up toolbar button.)
- The file picker toolbar is currently treated as a single tab-stop; instead, each button should have its own tab stop.
- The Path field in the file picker is not reachable by tab key.

- Scripts with cursive joining are badly mangled. See https://github.com/emilk/egui/issues/2517 (I guess the claim in the readme that it works with non-Latin characters is *technically* true, but I wish I'd realized this up front)

- The installer window sometimes says "Not responding" when it has lost focus/is behind another window. I wonder if it's sleeping the event loop or something like that, and so the desktop environment thinks it's hung. Re-focusing it causes it to resume promptly, so it's not truly hung.
- Closing the window with the close button sometimes causes it to hang. Clicking the close button again usually gets it to close cleanly.
- Component selection list says "(placeholder - manifest not loaded)".
  - I don't see any network traffic from it, so it doesn't seem to be trying to load the installation manifest. It should download the appropriate manifest for the platform when it first opens, so it's all loaded and ready to go by the time the user gets to the component selection screen. If there's a permanent error after a few retries of downloading it, it should show a message saying there was an error and asking whether to retry or cancel and quit.

- Cancel & OK in file picker and locations in side panel are not localized
- Vietnamese has mixed fonts and looks janky


Also also, I'm realizing that I don't think there's any way to have the local client authenticate to a corresponding registered remote account... remote account registration isn't turned on right now, anyway.
