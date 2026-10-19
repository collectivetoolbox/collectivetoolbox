# CSC Archive Implementation Notes

- `File::try_clone` shares the underlying file position. During archive resume,
  finish archive/source verification before truncating and seeking output to the
  append position. A duplicate verification in the writer constructor moved the
  shared handle into a payload and corrupted the subsequent append; the
  interrupted-tail resume regression covers this.
- `DcMstring::as_bytes` includes the DcMixed document header; `payload_bytes`
  does not. Keep document encoders distinct from record-only encoders when
  embedding file records inside checksummed sections.
- Existing flatten projection requires the field routing list, e.g.
  `#[dc(flatten, nested = [319, 321, 325, 323])]` for `FileEntity` fields.
  Reuse generic derive projection rather than special-casing archive records.
- Primary file contents already use Dc 392. Attached-stream contents use Dc 385;
  do not allocate a new semantic identifier or substitute one for the other.
- Shared publication now requires `ExistingEntryPolicy`; materialization defaults
  to `Refuse`, independently of its rewrite optimization. CSC authorization is
  `--overwrite`; moves use explicit `--force`. Resume intent records do not grant
  overwrite permission. Non-Linux Unix no-replace rename currently fails closed.
- Directory metadata authority is scoped to a live sandbox's exclusive creation
  and retained opened identity. Reopening loses that authority. Ownership lookup
  is keyed by exact native path; retained handles still cost one handle per
  created directory and do not lock later path-based metadata operations.
- File publication and moves share `rename_from_parent`, preserving native I/O
  errors so EXDEV can select verified copy fallback. AppleDouble companions use
  staged publication, but paired-file crash recovery remains pending.
- Both magics use journal top-level error/warning records, not separate checked
  diagnostic sections. Completion and byte integrity do not imply clean capture;
  `verify_integrity` is separate from `verify`, which rejects retained diagnostics
  and semantic metadata warnings. Resume refuses committed diagnostics before
  truncation until resolution semantics and durable inventory exist.
- Creation can continue only after source failures. The core records source and
  sink failures separately; sink/encoding ambiguity is fatal. File-output
  rollback discards BufWriter bytes via `into_parts` before truncating/seeking;
  never flush the failed section to roll it back. Stdout cannot roll back.
- Recovery returns the committed prefix with incomplete status after partial
  trailing records. Complete reads reject them. Strict checksum failure is not
  an incomplete write: closed corruption requires explicit warning-bearing
  forgiving salvage. `mark_failed` propagates pending-record commit failures.
- Journal owns `JournalWriter<W>`, `JournalReader`, `JournalPayload`, the scanner,
  and shared capture primitives. `StateJournal` is only the persistent copy
  operation/descriptor workflow around the common writer. Archive-specific type
  aliases and format profiles were removed; do not reintroduce them.
- CSC copy and archive planning share `file::traverse_dir` scheduling. Its
  `directories()` batches capture source guards before reading and validate/order
  siblings before scheduling descent. Physical inventories and exact filename
  checks use `file::read_dir_raw`, never AppleDouble-filtered discovery.
- Archive and journal are literally one format except header magic and optional
  primary bodies. `JournalMagic` selects only header bytes. Both permit metadata
  records, body-bearing records, multi-file batches, top-level diagnostics, and
  the same sequence/completion rules. `ReadMode` and projection policy select
  reader behavior independently of magic. Do not preserve the mistaken earlier
  experimental archive grammar as another compatibility target.
- `JournalReader::verify` checks metadata and bodies that are present, including
  attached streams when primary bytes are omitted. `verify_self_contained`
  additionally requires regular-file bodies; archive extraction requests that
  capability explicitly. `JournalSnapshot` projects this same reader to metadata.
- Historical journal completion is nonterminal on read. Explicit resume preserves
  all committed bytes, including diagnostics after completion; never subtract a
  marker length from valid_length to reopen. Same-instance writing is terminal.
- Descriptor updates after durable journal writes can fail independently.
  Pending-descriptor state blocks new operations; retry_descriptor repairs only
  metadata, without duplicate journal bytes/counters/completion markers.
- Descriptor publication uses `file::write_file_durable`: sync staged contents,
  then atomically replace and sync the parent on Unix; Windows uses write-through
  replacement. Publication failures retain descriptor-retry state.
- `AttachedStream` projects through `AttachedStreamWithPayload` using generic
  conversion derives. File-backed archive bodies remain lazy; copy-journal body
  stripping clears both `data` and `payload`. Body diagnostics recurse through
  stream entities in the shared v4 snapshot path, including direct reader calls.
- Native ADS capture hashes incrementally and records a filesystem origin.
  `AttachedStreamSource` supplies captured or archive-backed readers; regular
  materialization writes and verifies attached bodies before atomic publication.
  Nonresident AppleDouble/AppleSingle conversion is explicitly rejected before
  mutation until companion writers accept the same reader interface. Native
  Windows runtime qualification remains separate from cross-target checking.
- CapturedTree final validation and pending-warning persistence must precede
  completion. Best-effort validation returning Ok can still retain warnings;
  drain them through the shared visitor before discarding snapshots. Companion
  dependencies and cleanup warnings must use the same retained source guards.
- Raw archive capture/discovery disables Apple interpretation explicitly. Tests
  deliberately inspecting malformed or symlink companions as ordinary files must
  request raw capture; do not weaken interpreted-companion validation to fix them.
- Dc derives encode through the shared fallible `DcWriter`; buffered encoding
  delegates to the same implementation. File/stream bodies use the existing
  derived payload-record schemas, not hand-written Dc framing. Encoding views
  retain metadata and source handles without copying binary bodies.
- `BodyPolicy::Omit` is an explicit recursive output choice. Never infer it from
  missing sources, pass empty readers for nonempty bodies, or change filesystem
  origins to synthetic origins to suppress reads. Copy batches and archive
  members share the source-aware file encoder and grammar-validation helper.
- Streaming grammar validation spools exact bytes to an anonymous file and
  reuses the file-backed navigator, preserving payload types, lengths and section
  checksums without payload-to-Dc expansion. Temporary disk space scales with
  record size. Section footers remain withheld on validation failure.
  Binary-source failure tagging
  must preserve source-only rollback without overriding observed sink failures.

## Pending Native Windows Qualification

- `windows_native_qualification_large_ads_source_free_archive` covers three
  multi-megabyte native ADS with decomposed, precomposed and lone-surrogate
  names, lazy archive bodies, source deletion and restored body hashes.
- `windows_native_qualification_reparse_source_free_archive` covers native file
  and directory symlinks and a junction, with exact raw reparse buffer checks.
- Both were run using `scripts/vm` on Windows 7 and Windows 11. Neither passed:
  archive capture rejects the existing change-time restoration warning.
  Windows change time is the metadata-change timestamp, distinct from last
  modification time. Capture retains it, but restoration is unimplemented;
  `JournalWriter` rejects semantic metadata warnings before committing members.
- The Windows suite runner now requests elevation with PowerShell `RunAs`,
  checks the high-integrity token and logs privileges. The elevated Windows 7
  rerun created file and directory symlinks successfully, removing error 1314;
  it then failed creating the junction with invalid reparse data (error 4392).
  The runner lives in a separate git repository `ci-logs/vm`, invoked by
  `scripts/vm`; this environment's elevation change is not distributed by
  repository commits. Source-free extraction assertions remain unqualified. Do
  not suppress the warning or skip the fixtures to claim fidelity; native
  metadata handling and junction buffer handling need follow-up before these
  tests qualify the path.
- Extraction and verification support `--recover` (aliases: `--salvage`,
  `--allow-archive-errors`, `--allow-journal-errors`). In recover mode,
  `JournalReader` uses forgiving projection and recover scan mode, allows
  incomplete archives and archives containing capture diagnostics, restores all
  intact committed members, and logs warnings for damaged or unrecoverable ones
  instead of failing the entire extraction upfront.

Archive capability status and outstanding work are tracked in
[the archive issue](../issues/csc-usability/archive.md).