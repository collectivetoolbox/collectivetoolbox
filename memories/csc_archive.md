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

Archive capability status and outstanding work are tracked in
[the archive issue](../issues/csc-usability/archive.md).