### Input on Stage 1 Language ("StageL v3") Design

You put your finger on the three exact places where StageL v1/v2 (`docs/chats/2026-10-31-language.md` lines 253–520) falls short for `stage_1_impls/`:
1. **Library-only structure**: StageL v1/v2 only defines named functions (`fname: ...`), with no concept of a document transformation/evaluation entry point or how a Dc implementation consumes an input Dc/byte stream and emits an output stream.
2. **Wishy-washy dynamic typing & implicit string/array copies**: Prefix-typed variables (`n/x`, `an/arr`, `s/str`) had loose runtime coercions, 1-based indexing with out-of-bounds returning `0`/`""`, and assumed strings/arrays live entirely in memory.
3. **No streaming or random-access extent model for large files or graph-sliced documents**: Neither `csc archive` on a 10 GB disk file nor the Dc document UI navigating a huge document made of deduplicated graph slices can copy the payload into WASM's linear memory (capped at 256 pages = 16 MiB by default in `DcWasmResourceLimits`).

Here is a concrete design proposal for **StageL v3** that resolves all three while keeping StageL's compact, low-ceremony feel so its compiler in `stage_0_impls/` stays small.

#### 1. Boundary & Interface: Unified "Routine + Stream Filter" Model
Every `.stagel` file in `stage_1_impls/<dc_id>_<name>.stagel` compiles to a `.WASM` program (`Dc 1927` `"Begin WASM program"`..`Dc 1928` `"End WASM program"`):
- **Single-routineDc implementation (the common case)**: The file body (or its `main` / `_start` routine) is the implementation of that Dc. When the evaluator encounters that Dc in a document, it invokes the routine with a **Cursor/View over the matched syntax span** (or current stream window) and an **Output Writer** that appends tokens or extents to the spliced result.
- **Multi-routine library / helper Dcs**: A `.stagel` file can also define named helper routines (`fn parse_dctf_header(...) -> ...`), which compile to entries in a `Dc 559` `"Begin key-value array (ordered)"`..`Dc 560` `"End key-value array"` module map or separate graph helper nodes that other Stage 1 files can call by Dc/node reference.

#### 2. Tightened Type System (Static Value Types + Zero-Copy Views)
Instead of StageL v1/v2's dynamic `n/` (arbitrary number) and `s/` (in-memory string), StageL v3 should have **fixed, deterministic static types** that map 1-to-1 to WASM registers or 64-bit stream descriptors:
- **Machine Scalars** (live in WASM locals/registers, zero heap allocation):
  - `u8`, `i32`, `u32`, `i64`, `u64`
  - `dc` (a 128-bit Document Character `(high64, low64)` pair in two `i64` locals, or a compact `DcTF` slice)
- **Zero-Copy Stream / Slice Handles** (never copy the underlying payload into WASM linear memory):
  - `view`: A lightweight `(source_id: u32, offset: u64, len: u64)` descriptor representing a contiguous logical byte or Dc range in an input stream, file, or graph node.
  - `cursor`: A mutable reader positioned at `pos: u64` within a `view`.

#### 3. How Data Gets In and Out & How Large Files / Graph Slices Work
To make `csc archive` and the Dc document editor work identically on **10 GB disk files** and **large graph documents stitched from size-limited value nodes**, the kernel's 32-byte linear-memory mailbox (`mem[0..32]`) only needs one natural extension: **Window Paging + Extent Emission**.

##### A. Reading Huge Inputs Without Copying (`view` / `cursor` Window Paging)
- Logical streams (whether backed by a disk file via `mmap`/`pread`, an in-memory buffer, or a tree of deduplicated graph slice nodes `DcShorthand::Long(node_id)` containing `Dc 203` `"Begin encapsulated binary data"`..`Dc 204` `"End encapsulated binary data"`) are presented to the Stage 1 program as a logical 64-bit address space `0..total_byte_len`.
- At any step, the host maps a **sliding window** (e.g., 64 KiB at `[win_offset .. win_offset + win_len]`) into WASM linear memory at `input_ptr..input_ptr + input_len`.
- In StageL v3, reading bytes or Dcs through a `cursor` (`peek_u8(c)`, `read_u32_le(c)`, `read_dc(c)`, `seek(c, new_offset)`):
  - **Fast path (in-window, >99.9% of reads)**: Compiles to a bounds check `offset >= win_offset && offset + n <= win_offset + win_len` followed by a direct `i32.load8_u` / `i64.load` from WASM linear memory. Zero host calls, runs at native WASM speed.
  - **Window miss / seek across a graph slice boundary**: When `c` advances or seeks outside `[win_offset .. win_offset + win_len]`, the StageL runtime helper writes the requested `(source_id, target_offset, min_bytes)` into the mailbox and returns a `StepStatus::MapWindow` yield code from `step()`.
  - The host/kernel transparently resolves `target_offset`—whether by seeking in the disk file or looking up the next graph slice node in the B-tree/slice list—copies just that 64 KiB window into WASM linear memory, and resumes `step()`.
  - **Result**: A Stage 1 parser (`csc archive` reading a central directory at the end of a 10 GB archive, or the UI jumping to line 500,000 of a graph document) seeks to `offset = 10_000_000_000 - 65536` and inspects the directory in **one 64 KiB window page** without ever touching the other 9.99 GB.

##### B. Writing / Stitching Huge Outputs Without Copying (Output Extents)
- What happens when `csc archive` packages a 10 GB file, extracts a 2 GB member, or inserts 10 bytes into the middle of a 500 MB graph document?
- If the Stage 1 program had to copy every byte from `input_ptr` to `output_ptr`, it would copy gigabytes through linear memory.
- Instead, StageL v3's output stream supports two primitives:
  1. `emit_bytes(...)` / `emit_dc(...)`: Appends newly computed small data (headers, archive tables, evaluated expressions) into the linear-memory output buffer (`output_ptr..output_ptr + output_len`).
  2. `emit_extent(view_slice)` (using the existing `Dc 376` `"Extent byte offset"` and `Dc 377` `"Extent byte length"` semantics): Emits a **zero-copy reference** `(source_id, offset: u64, len: u64)` to a sub-range of an input stream or existing graph node!
- When the output stream is materialized:
  - On disk (`csc archive`), an emitted extent becomes a zero-copy `copy_file_range` / `sendfile` or extent-reference node.
  - In the local graph, small emitted buffers are chunked and content-addressed/deduplicated into value nodes (`get_or_insert_value_node`), while unchanged sub-ranges reuse the existing graph slice node IDs (`DcShorthand::Long(node_id)`) directly!
