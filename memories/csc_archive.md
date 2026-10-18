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
- Archive diagnostics reuse journal error/warning payload conversions in checked
  sections. Completion and byte integrity do not imply clean capture;
  `verify_integrity` is separate from `verify`, which rejects retained diagnostics
  and semantic metadata warnings. Resume refuses committed diagnostics before
  truncation until resolution semantics and durable inventory exist.
- Creation can continue only after source failures. The core records source and
  sink failures separately; sink/encoding ambiguity is fatal. File-output
  rollback discards BufWriter bytes via `into_parts` before truncating/seeking;
  never flush the failed section to roll it back. Stdout cannot roll back.
- Archive recovery must reject every byte after completion, including partial
  tokens; permissive archive EOF recovery applies only before completion. Journal `mark_failed`
  must propagate pending-record commit errors before updating its descriptor.
- Journal owns the modern `stream::RecordWriter`, `scanner`, and shared `capture`
  primitives. ArchiveWriter is a type alias, not another state machine. Profiles
  select headers, bodies, diagnostic placement, and legacy-compatible scan rules.
  Do not reintroduce archive-to-journal duplication or journal imports of archive.
- Historical journal completion is nonterminal on read. Explicit resume preserves
  all committed bytes, including diagnostics after completion; never subtract a
  marker length from valid_length to reopen. Same-instance writing is terminal.
- Descriptor updates after durable journal writes can fail independently.
  Pending-descriptor state blocks new operations; retry_descriptor repairs only
  metadata, without duplicate journal bytes/counters/completion markers.
- CapturedTree final validation and pending-warning persistence must precede
  completion. Best-effort validation returning Ok can still retain warnings;
  drain them through the shared visitor before discarding snapshots. Companion
  dependencies and cleanup warnings must use the same retained source guards.
- Raw archive capture/discovery disables Apple interpretation explicitly. Tests
  deliberately inspecting malformed or symlink companions as ordinary files must
  request raw capture; do not weaken interpreted-companion validation to fix them.

Archive capability status and outstanding work are tracked in
[the archive issue](../issues/csc-usability/archive.md).