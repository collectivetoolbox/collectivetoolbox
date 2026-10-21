# Format Detection

## Remaining Gaps vs Upstream

Open items retained from the earlier comparison with `file`. Unless noted,
these have not been re-audited during this documentation cleanup; verify the
current behavior before implementing a reported gap.

### Decompression

- [ ] Support bounded recursive inspection of lrzip (`.lrz`) contents. The
  earlier report identified ZPAQ decompression and multi-file archive streaming
  as prerequisites; verify those dependencies are still outstanding.

### ELF Introspection

- [ ] Verify the claimed DWARF parity gap against upstream `readelf.c`, then
  track any missing compilation-unit inspection separately from optional
  compiler-producer/build-flag extraction. Do not assume full DWARF unwinding
  is required for `file` parity.
- [ ] Decode core-dump register state beyond x86-64, i386, and ARM/AArch64
  (including MIPS, SPARC, Alpha, and PowerPC).
- [ ] Handle Solaris core-dump `prstatus_t` layout variations.
- [ ] Cross-reference `SHT_SUNW_cap` symbol capabilities with dynamic symbols.

### OLE2 Compound Documents

- [ ] Follow multi-sector directory and SAT/FAT/MiniFAT chains rather than
  inspecting only the first directory sector.
- [ ] Parse SummaryInformation and DocumentSummaryInformation property streams:
  title, subject, author, keywords, comments, template, last saved by, revision,
  editing time, creation/save timestamps, word/page/slide counts, and application.

### Filesystem Entities

- [ ] Resolve symlink targets and distinguish broken links.
- [ ] Read and report character/block device major and minor numbers.
- [ ] Report setuid, setgid, sticky bits, and BSD file flags.
- [ ] Detect Solaris doors where supported.

### Text and Encodings

- [ ] Detect UTF-7 text.
- [ ] Re-run text magic rules on decoded UTF-8 buffers for UTF-16, UTF-32, and
  EBCDIC inputs, rather than matching only the original bytes. Cover encoded
  XML and scripts, including applicable BOM-less cases.

### Context and Metadata

- [ ] Use the recorded source OS from the file entity for detection hints,
  not the host running detection. Include the enclosing archive's format/OS
  association; it may be stronger evidence than the extraction host.
- [ ] Derive platform priors from the formats database rather than treating
  the mere presence of Apple metadata as a hardcoded platform decision in
  `file/entity.rs`.
- [ ] Evaluate consolidating the MIME crate's mappings with Magdir MIME data,
  while retaining provenance and resolving conflicting evidence explicitly.

### CLI Compatibility and Limits

- [ ] Verify and fill gaps in `-e` / `--exclude` test filtering: `apptype`,
  `ascii`/`text`, `cdf`, `compress`, `csv`, `elf`, `encoding`, `soft`, `tar`,
  `json`, and `simh`, accounting for intentionally unsupported subsystems.
- [ ] Verify and fill gaps in `-P` / `--parameter` limits: `bytes`, `elf_notes`,
  `elf_phnum`, `elf_shnum`, `elf_shsize`, `encoding`, `indir`, `name`, `regex`,
  and `magwarn`.
- [ ] Verify remaining execution/output compatibility: `-k` / `--keep-going`,
  `-b` / `--brief`, `-N` (no padding), and `-s` (inspect device contents).

## Intentional Exclusions

- OS/2 `DosQueryAppType` integration is platform-specific and intentionally
  omitted; it is not an open portable-detection task.
- External decompressor command fallbacks differ from upstream but are not an
  implementation task: detection must remain self-contained rather than depend
  on installed system tools.