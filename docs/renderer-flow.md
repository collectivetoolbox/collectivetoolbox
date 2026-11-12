Output will be in `src/renderer/` crate for rendering - the split will be, `src/runtime/` snapshots the DOM (or equivalent) using high-semantic-level Dcs in the current state, then passes that snapshot over IPC to `src/renderer/`, then `src/renderer/` lowers the Dcs using the Dc equivalency rules and decorates it with glyph Dcs (not yet defined), then projects that (from the user's viewport perspective - this is where it would project 3D objects onto a 2D surface, for instance - no-op for a pure-Dc screenshot preserving 3D data) to the output format (maybe text, maybe raster, maybe HTML, maybe just a no-op and leave the Dcs as-is for a pure-Dc screenshot, etc). It then hands that completed frame/document/page (depending on the output format) over IPC to `src/io/`, which handles actually interacting with the OS to present it to the user.

Displaying a Dc document is executing it, similarly to PostScript. Execution
belongs to the runtime; the renderer processes the resulting semantic snapshot.
Source-backed typed inspection and localized string editing are described in
[Dc String API](dc-string-api.md). Those operations do not introduce a separate
document type or make typed decoding a prerequisite for execution.
Without user authorization, the runtime permits only a bounded amount of pure
computation. Reading local nodes, persistent storage, file/network I/O, large
memory use, high CPU use, and high-resolution timers require explicit
capabilities and are denied unless the user authorizes them. Raw source
inspection is a separate operation, not the normal document-display model;
quoted payloads must not implicitly execute as programs merely by being parsed
or inspected.

The runtime will not run documents in strict validation mode. They will run in permissive mode, and if there are parse or runtime errors *and* they happen to request a privilege (file I/O, network, high-resolution timers, etc) the user will be very strongly steered toward mocking or denying the permissions due to the errors. The permission system planned for this will have multiple states compared to web browser permissions: deny the permission (in a way that's visible to the document), auto-mock the permission (return automatically spoofed data), manually mock the permission (provide a custom replacement interface with your own spoofed data or mock methods), or allow the permission. The "allow" button is the one that would be hidden away (like bypassing TLS warnings in web browsers) if a document has errors.

Assuming it's a valid DcString (which it should be as it's a validated type - note that any sequence of Dcs is valid), there will be no parser error that prevents at least trying to run it. If things are mangled, it'll want to parse as much as it can, giving up on broken structures and passing them to the runtime as-is. The runtime will also do its very best to make its way to the end of the document - if the document uses too much compute before it's fully parsed, the runtime will raise a permission request, providing an option to break out of infinite loops and such and resume executing/printing the document on the next Dc.

The renderer is intended to use an "n-dimensional" model, where documents (=characters, =windows) can be expressed into any number of dimensions, and are projected into the correct number of dimensions during the final rendering passes after the Dc model is prepared (so for instance a 4D object such as a tesseract could be natively represented, manipulated, and displayed, and would only be folded into two or three dimensions at display time - similar to how computer game engines work).

## Composite Dc Documents: UI Components, Words, Characters, and Glyphs

A typical Dc document is a hierarchical composition of subdocuments and semantic Dcs spanning UI structure, text, and visual representation:

- **UI & Structural Dcs**: Interactive/layout elements are themselves Dcs composed from simpler Dcs—for example, a `"window"` Dc, a `"menu bar"` Dc implemented in terms of `"button"` Dcs, and individual `"button"` Dcs.
- **Characters, Words, and Inline Text**:
  - Individual characters are Dcs.
  - Later, **words** can also be represented as Dcs: either as a semantic concept Dc linked via `Dc 397 "Equivalent representations"` / `Dc 561 "Partially equivalent representations"` to an alternate-representation spelling, or as a dedicated lexical word Dc. Conversely, a paragraph or string can simply be included directly in a document as an ordinary character sequence like a normal text file without requiring word-level tokenization.
- **Shaping to Glyph Dcs & Target Output Projection**:
  - Characters and word spellings resolve to **glyph Dcs** according to explicit inputs: font Dcs, script/language shaping rules, ligatures/contextual forms, and layout parameters. (Because shaping in many scripts is contextual and many-to-many across character clusters, shaping operates over character runs rather than assuming an isolated 1-to-1 character-to-glyph map.)
  - Final rendering of glyph and layout Dcs depends on the **output format**: a one-off conversion to a plain-text file, a terminal stream, an HTML document, a raster framebuffer, or a no-op **Dc-native screenshot** that leaves the resolved Dc snapshot in Dcs.

## Deterministic Precompilation, Caching, and Multi-Document Concurrency

Running every Dc decomposition, parser, widget, and shaping pass through interpreted WASM on every evaluation would be far too slow. Because the graph execution and rendering model is strictly deterministic given explicit inputs (document bytes, `Dc 641 "Global graph system block most recent node ID at time of document creation"` snapshot pins, font/rendering configuration, and ordered external responses), performance relies on two complementary strategies (see [`issues/language.md`](../issues/language.md) for compiler details):

1. **Ahead-of-time Rust transpilation built into `ctoolbox`**: Reusable functions are strongly encouraged to be allocated their own Dcs so the append-only graph stays DRY. Common/hot Dcs transpile via StageR 3 into Rust and ship **precompiled to optimized native machine code** inside the `ctoolbox` binary (keyed to exact resolved implementation node IDs so pinned historical documents preserve bit-for-bit behavior).
2. **Multi-layer caching & precomputed snapshots**: Dcs at intermediate stages of execution and rendering—compiled WASM modules, resolved syntax/shaping tables, shaped glyph runs for `(font_dc, shaping_config, text_run)`, lowered UI subtrees, and static subdocuments—can be cached in memory/storage or shipped precomputed.
3. **Process-isolated concurrency with optional shared memory**: StageL 3 and StageR 3 are single-threaded within a document (intra-document multithreading in EM3 is deferred). Parallelism across subdocuments or workloads can run multiple Dc documents as separate isolated OS processes sharing immutable/sealed memory extents or frame buffers for zero-copy performance without introducing intra-document data races.
