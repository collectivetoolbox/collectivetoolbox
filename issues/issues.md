- [ ] Icons do not have consistent size (compare "Restart" and "Reload")
- [ ] Add a clear badge for when it's running in debug mode, and a warning about not logging in with your real account
  - [ ] Make the debug logging less of a footgun somehow?
- [ ] Web UI: Errors should have a clear way of knowing whether they are serious errors that require a restart, or "normal" errors like password didn't match.
- [ ] Implement collapse_filtered and collapse_only in base.rs
- [ ] Support for request logging as a configurable option
- [ ] Record a build ID and expose it via --version
- [ ] Make sure to have a build with no bundled browser for linux if it stays 700mb
- [ ] Scrolling by tapping top bar doesn’t work on iOS
- [ ] Minifying middleware?
	- [ ] HTML: https://crates.io/crates/swc_html_minifier
	- [ ] JS: https://crates.io/crates/oxc_minifier
	- [ ] CSS: https://crates.io/crates/lightningcss
	- [ ] SVG:
		- [ ] https://github.com/noahbald/oxvg
		- [ ] https://github.com/bearcove/svag
- [ ] One crate (don't remember which, workspace maybe?) doesn't get a link in the rustdocs for some reason
- [ ] Format build date nicely as a sortable numeric string and include in file names maybe instead of the commit
- [ ] Implement HTTPS authenticated fetch for Cloudflare
- [ ] Maybe: "tighten the logging so the next IPC failure reports which method failed to deserialize (service/method + arg sizes)"
- [ ] Generate remaining IPC service boilerplate in build.rs and include date/timestamp in the generated files
- [ ] Have some sort of environment helper function to avoid having a "Restart PC" button on the web
- [ ] 404 is not nice looking now it redirects to the SPA
- [ ] SPA pages in subdirectories can't use directory-relative links (e.g. newsletters.rss (doesn't work) versus /newsletters/newsletters.rss (works))
  - [ ] This might actually work now. Need to check
- [ ] MacOS build possibilities:
  - https://actually.fyi/posts/zig-makes-rust-cross-compilation-just-work/
  - "Zig actually provides libSystem.tbd stubs out of the box, so you can cross-compile from any OS to macOS too as long as you only depend on libSystem (the macOS libc.)" - emidoots at https://news.ycombinator.com/item?id=30488979
- [ ] Add OpenGL/Vulkan: Approach it by statically linking LLVM and Mesa software rendering (LLVMpipe/Lavapipe), and delivering separate, optional minimal shim binaries that attempt to dlopen the system X11/libwayland/libGL, proxying over IPC. On other platforms, I'll likely use some dynamic linking, with a preference for statically linking where possible.

### Installer

ALL SOURCE CODE EXCEPT ICECAT MISSING FROM GUIX IMAGE

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

Please check the Handlebars templates for all elements with button styling classes, and without changing their appearance, change them to <button> elements so that they respond as expected for buttons.

Confirm Homebrew packaging works.

Avoid need for duplicate exclude paths in deno.json.

Cache busting parameters on URLs.

Update Guix packaging.

Hi! I'd like you to work on some issues in the installer:

- Keyboard input is still somewhat laggy, and frequently keys "stick" when pressed. (Mouse input, by contrast, is relatively smooth, and un-"stick"s the stuck keyboard event - keyboard events work smoothly if I'm continuously moving the mouse.)
- The license text view doesn't capture keyboard focus and can't be closed nor the controls tabbed between using only the keyboard; focus remains on the outer window. It should capture the tab focus like the file picker modal.

- Navigating the file tree by arrow keys isn't working: it appears to do *something* sometimes when I press the arrow keys, but the effects seem unrelated to the keys I'm pressing. Pressing Up should move the selected item to the preceding entry in the current column. Pressing Down should move to the following entry in the current column. Pressing Down when at the last selectable item in a column should do nothing; similarly for pressing Up when at the first. Pressing Right should move focus to the top item in the next column to the right. Pressing Left should move focus to the enclosing directory in the column before the currently selected column. (In other words the Left key should do the same thing as the Up toolbar button.)

- Home, End, Page Up, and Page Down aren't doing anything.
- It's possible to type into the license text box.
-

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
- The License modal does not trap keyboard focus, so it's possible to tab out of it. It should use `set_focus_lock_filter` like the file picker to ensure you can't tab out.
- The License modal does not respond to pressing the escape key to close it.
- Could you refactor out the logic to make a modal dialog into its own module so the file picker, license dialog, and any other similar features later added can use the same component?

- Scripts with cursive joining are badly mangled. See https://github.com/emilk/egui/issues/2517 (I guess the claim in the readme that it works with non-Latin characters is *technically* true, but I wish I'd realized this up front)

- The installer window sometimes says "Not responding" when it has lost focus/is behind another window. I wonder if it's sleeping the event loop or something like that, and so the desktop environment thinks it's hung. Re-focusing it causes it to resume promptly, so it's not truly hung.
- Closing the window with the close button sometimes causes it to hang. Clicking the close button again usually gets it to close cleanly.
- Component selection list says "(placeholder - manifest not loaded)".
  - I don't see any network traffic from it, so it doesn't seem to be trying to load the installation manifest. It should download the appropriate manifest for the platform when it first opens, so it's all loaded and ready to go by the time the user gets to the component selection screen. If there's a permanent error after a few retries of downloading it, it should show a message saying there was an error and asking whether to retry or cancel and quit.

- Cancel & OK in file picker and locations in side panel are not localized
- Vietnamese has mixed fonts and looks janky


Also also, I'm realizing that I don't think there's any way to have the local client authenticate to a corresponding registered remote account... remote account registration isn't turned on right now, anyway.

No way to log out. "Register or log in" should probably become "Log out" when logged in.


