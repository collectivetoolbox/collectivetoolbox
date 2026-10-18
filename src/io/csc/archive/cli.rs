#[expect(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use crate::journal::{JournalReader, stream::RecordWriter};
use crate::journal::{JournalErrorRecord, JournalErrorStage, JournalSessionRecord, JournalWarningRecord, current_platform};
use crate::journal::capture::{CapturedSource, CapturedTree, CommittedHardlinks, attach_environment, discover_directory};
use clap::{Parser, Subcommand};
use ctb_io::file::apple_double::AppleReadOptions;
use ctb_io::file::{
    DiskPayloadSource, FileEntity, FileEntityKind, FileOrigin,
    LinkPath, MaterializeOptions, PathTraversalPolicy, PayloadSource, SandboxableDir,
    SymlinkValidationPolicy, TraversalOptions, path_has_trailing_slash,
    resolve_existing_ancestors,
};
use ctb_utilities::cli::ToolResult;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Parser)]
pub struct ArchiveArgs {
    #[command(subcommand)]
    pub command: ArchiveCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ArchiveCommand {
    /// Create a self-contained uncompressed CSC archive.
    Create {
        /// Archive output path, or - for binary stdout.
        archive: PathBuf,
        #[arg(required_unless_present = "resume", conflicts_with = "resume", num_args = 1..)]
        sources: Vec<PathBuf>,
        /// Resume an interrupted file-output archive using its recorded sources.
        #[arg(long)]
        resume: bool,
        /// Retain source failures, continue capture, and report nonzero status.
        #[arg(long, conflicts_with = "resume")]
        continue_on_error: bool,
    },
    /// List committed members and capture diagnostics.
    List { archive: PathBuf },
    /// Verify archive structure, checksums, and file contents without sources.
    Verify { archive: PathBuf },
    /// Extract a verified archive beneath a destination directory.
    Extract {
        archive: PathBuf,
        destination: PathBuf,
        /// Replace existing regular files atomically; merge existing directories.
        #[arg(long)]
        overwrite: bool,
        /// Preserve link targets verbatim, including targets outside destination.
        #[arg(long)]
        trust_symlink_targets: bool,
        /// Report filesystem metadata losses instead of requiring exact restoration.
        #[arg(long)]
        best_effort_metadata: bool,
        /// Permit restoration of FIFOs and device nodes as special files.
        #[arg(long)]
        copy_specials_as_specials: bool,
    },
}

pub fn run_archive(args: &ArchiveArgs) -> Result<ToolResult> {
    match &args.command {
        ArchiveCommand::Create { archive, sources, resume, continue_on_error } => {
            let _scope = ctb_io_environment::GlobalEnvironmentScope::enter_fresh();
            if *resume {
                ensure!(!continue_on_error, "Cannot combine --resume with --continue-on-error; capture diagnostics cannot be resolved by resume");
                ensure!(archive != Path::new("-"), "Cannot resume archive output on stdout");
                resume_archive(archive)?;
                return Ok(ToolResult::Immediate {
                    stdout: Vec::new(), stderr: b"Archive resumed and completed.\n".to_vec(), exit_code: 0,
                });
            }
            let plan = plan_sources(sources, archive)?;
            let (error_count, warning_count, completed) = if archive == Path::new("-") {
                #[cfg(unix)]
                {
                    use std::os::fd::AsFd;
                    let output = File::from(std::io::stdout().as_fd().try_clone_to_owned()?);
                    validate_output_identity(&output, &plan.members)?;
                }
                let outcome = create_archive(std::io::stdout().lock(), sources, archive, &plan, *continue_on_error, None)?;
                (outcome.error_count, outcome.warning_count, outcome.completed)
            } else {
                let mut options = OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                let file = options.open(archive).with_context(|| format!("Cannot create {}", archive.display()))?;
                let outcome = create_archive(FileArchiveOutput::new(file), sources, archive, &plan, *continue_on_error, None)?;
                (outcome.error_count, outcome.warning_count, outcome.completed)
            };
            let clean = completed && error_count == 0 && warning_count == 0;
            Ok(ToolResult::Immediate {
                stdout: Vec::new(),
                stderr: if clean { b"Archive created.\n".to_vec() } else {
                    format!("Archive capture {}: {error_count} errors; {warning_count} warnings.\n",
                        if completed { "finished with diagnostics" } else { "stopped incomplete" }).into_bytes()
                },
                exit_code: if clean { 0 } else { 1 },
            })
        }
        ArchiveCommand::List { archive } => {
            let reader = JournalReader::recover(File::open(archive)?)?;
            validate_members(&reader)?;
            let mut output = String::new();
            use std::fmt::Write;
            for member in &reader.members {
                let size = match &member.entity.kind { FileEntityKind::Regular { size, .. } => *size, _ => 0 };
                let path = member.entity.identity.relative_path.as_deref().context("Archive member has no path")?;
                writeln!(output, "{}\t{size}\t{}", member.entity.kind.entity_type().as_str(), path.display())?;
            }
            let mut diagnostics = String::new();
            if !reader.is_completed() {
                diagnostics.push_str("Archive is incomplete; listing committed members only.\n");
            }
            for error in &reader.errors {
                writeln!(diagnostics, "Capture error ({:?}) {}: {}", error.stage, error.path.display(), error.error_message)?;
            }
            for warning in &reader.warnings {
                writeln!(diagnostics, "Capture warning ({}): {}", warning.code, warning.message)?;
            }
            if reader.has_diagnostics() {
                writeln!(diagnostics, "Capture errors: {}; warnings: {}. Archive is not a clean capture.", reader.errors.len(), reader.warnings.len())?;
            }
            Ok(ToolResult::Immediate {
                stdout: output.into_bytes(),
                stderr: diagnostics.into_bytes(),
                exit_code: if reader.is_completed() && !reader.has_diagnostics() { 0 } else { 1 },
            })
        }
        ArchiveCommand::Verify { archive } => {
            let reader = JournalReader::new(File::open(archive)?)?;
            reader.verify_self_contained()?;
            validate_members(&reader)?;
            Ok(ToolResult::immediate_ok(b"Archive verified.\n".to_vec()))
        }
        ArchiveCommand::Extract { archive, destination, overwrite, trust_symlink_targets,
            best_effort_metadata, copy_specials_as_specials } => {
            let reader = JournalReader::new(File::open(archive)?)?;
            let options = MaterializeOptions {
                strict_lossless: !best_effort_metadata,
                existing_entry_policy: if *overwrite { ctb_io::file::ExistingEntryPolicy::Replace }
                    else { ctb_io::file::ExistingEntryPolicy::Refuse },
                force_overwrite: true,
                symlink_policy: if *trust_symlink_targets { SymlinkValidationPolicy::PreserveVerbatim }
                    else { SymlinkValidationPolicy::RejectEscapingSymlinks },
                path_policy: PathTraversalPolicy::StrictSandboxed,
                copy_specials: *copy_specials_as_specials,
                ..Default::default()
            };
            extract_archive(&reader, destination, &options)?;
            Ok(ToolResult::immediate_ok(b"Archive extracted.\n".to_vec()))
        }
    }
}

#[derive(Debug)]
struct SourceMember {
    source: PathBuf,
    relative: PathBuf,
}

#[derive(Default)]
struct SourcePlan {
    members: Vec<SourceMember>,
    diagnostics: Vec<JournalErrorRecord>,
    tree: CapturedTree,
}

fn archive_apple_read_options() -> AppleReadOptions {
    AppleReadOptions {
        read_apple_double_alongside: false,
        read_apple_double_zip: false,
        read_apple_double_netatalk: false,
        read_apple_single_without_extension: false,
        read_apple_single: false,
        read_apple_single_as: false,
        read_apple_single_asf: false,
    }
}

fn plan_sources(sources: &[PathBuf], archive: &Path) -> Result<SourcePlan> {
    ensure!(!sources.is_empty(), "At least one source is required");
    let output = if archive == Path::new("-") { None } else { Some(resolve_existing_ancestors(archive)?) };
    let working_directory = std::env::current_dir()?;
    let mut plan = SourcePlan::default();
    let mut names = HashSet::new();
    for source in sources {
        let absolute = working_directory.join(source);
        if let Some(output) = &output {
            let resolved = resolve_existing_ancestors(&absolute)?;
            ensure!(*output != resolved && !output.starts_with(&resolved),
                "Archive output overlaps a source: {}", source.display());
        }
        let metadata = match std::fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) => {
                plan.diagnostics.push(JournalErrorRecord::from_error(&absolute, JournalErrorStage::MetadataRead, &error.into()));
                continue;
            }
        };
        let prefix = if metadata.is_dir() && path_has_trailing_slash(source) {
            PathBuf::from(".")
        } else {
            PathBuf::from(source.file_name().context("Source has no basename; use a trailing slash for a directory root")?)
        };
        if metadata.is_dir() {
            let relative = normalized_member(&prefix)?;
            ensure!(names.insert(relative.clone()), "Duplicate archive member: {}", relative.display());
            plan.members.push(SourceMember { source: absolute.clone(), relative });
            let mut pending = std::collections::VecDeque::from([absolute.clone()]);
            while let Some(directory) = pending.pop_front() {
                let snapshot = ctb_io::file::payload::SourceSnapshot::capture(&directory, false)?;
                plan.tree.retain(&snapshot);
                let entries = match discover_directory(&directory, &TraversalOptions {
                    apple_read_options: archive_apple_read_options(), ..Default::default()
                }) {
                    Ok(entries) => entries,
                    Err(error) => {
                        plan.diagnostics.push(JournalErrorRecord::from_error(&directory, JournalErrorStage::Traversal, &error));
                        plan.members.retain(|member| member.source != directory);
                        continue;
                    }
                };
                for item in entries {
                if !item.errors.is_empty() {
                    let error = anyhow::anyhow!("Source traversal errors: {}", item.errors.join("; "));
                    plan.diagnostics.push(JournalErrorRecord::from_error(&item.path, JournalErrorStage::Traversal, &error));
                    plan.members.retain(|member| member.source != item.path);
                    continue;
                }
                let relative = normalized_member(&prefix.join(item.path.strip_prefix(&absolute)?))?;
                ensure!(names.insert(relative.clone()), "Duplicate archive member: {}", relative.display());
                if item.is_dir { pending.push_back(item.path.clone()); }
                plan.members.push(SourceMember { source: item.path, relative });
                }
                snapshot.validate()?;
            }
        } else {
            let relative = normalized_member(&prefix)?;
            ensure!(names.insert(relative.clone()), "Duplicate archive member: {}", relative.display());
            plan.members.push(SourceMember { source: absolute, relative });
        }
    }
    Ok(plan)
}

fn normalized_member(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {},
            _ => bail!("Unsafe archive member path: {}", path.display()),
        }
    }
    let bytes = path.as_os_str().as_encoded_bytes();
    ensure!(!bytes.contains(&0) && !bytes.starts_with(b"\\") && bytes.get(1) != Some(&b':'),
        "Invalid archive member prefix: {}", path.display());
    if normalized.as_os_str().is_empty() { normalized.push("."); }
    Ok(normalized)
}

fn validate_output_identity(output: &File, plan: &[SourceMember]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let output_metadata = output.metadata()?;
        for member in plan {
            let source_metadata = std::fs::symlink_metadata(&member.source)?;
            ensure!(source_metadata.dev() != output_metadata.dev() || source_metadata.ino() != output_metadata.ino(),
                "Archive output is included in the source tree: {}", member.source.display());
        }
    }
    #[cfg(not(unix))]
    { let _ = (output, plan); }
    Ok(())
}

trait ArchiveOutput: Write {
    fn checkpoint(&mut self) -> Result<Option<u64>> { Ok(None) }
    fn rollback(&mut self, _position: u64) -> Result<()> { bail!("Archive output cannot roll back") }
    fn sync(&mut self) -> Result<()> { Ok(()) }
}

impl ArchiveOutput for std::io::StdoutLock<'_> {}
impl ArchiveOutput for Vec<u8> {}
impl ArchiveOutput for std::io::Cursor<Vec<u8>> {}

impl ArchiveOutput for File {
    fn checkpoint(&mut self) -> Result<Option<u64>> { Ok(Some(self.stream_position()?)) }
    fn rollback(&mut self, position: u64) -> Result<()> {
        self.set_len(position)?;
        self.seek(SeekFrom::Start(position))?;
        self.sync_data()?;
        Ok(())
    }
    fn sync(&mut self) -> Result<()> { Ok(self.sync_all()?) }
}

struct FileArchiveOutput {
    buffer: Option<BufWriter<File>>,
}

impl FileArchiveOutput {
    fn new(file: File) -> Self { Self { buffer: Some(BufWriter::new(file)) } }
}

impl Write for FileArchiveOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.buffer.as_mut().ok_or_else(|| std::io::Error::other("Archive file buffer unavailable"))?.write(bytes)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.as_mut().ok_or_else(|| std::io::Error::other("Archive file buffer unavailable"))?.flush()
    }
}

impl ArchiveOutput for FileArchiveOutput {
    fn checkpoint(&mut self) -> Result<Option<u64>> {
        self.flush()?;
        Ok(Some(self.buffer.as_mut().context("Archive file buffer unavailable")?.get_mut().stream_position()?))
    }
    fn rollback(&mut self, position: u64) -> Result<()> {
        let (mut file, _) = self.buffer.take().context("Archive file buffer unavailable")?.into_parts();
        ArchiveOutput::rollback(&mut file, position)?;
        self.buffer = Some(BufWriter::new(file));
        Ok(())
    }
    fn sync(&mut self) -> Result<()> {
        self.flush()?;
        self.buffer.as_ref().context("Archive file buffer unavailable")?.get_ref().sync_all()?;
        Ok(())
    }
}

impl Drop for FileArchiveOutput {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() { drop(buffer.into_parts()); }
    }
}

struct CaptureOutcome<W> {
    output: W,
    error_count: usize,
    warning_count: usize,
    completed: bool,
}

trait CaptureSource {
    fn capture(&mut self, path: &Path) -> Result<CapturedSource> {
        let mut source = CapturedSource::capture(path, None, &archive_apple_read_options(), false)?;
        source.hash(path)?;
        Ok(source)
    }
    fn open(&mut self, path: &Path) -> Result<Box<dyn PayloadSource>> { Ok(Box::new(DiskPayloadSource::open(path)?)) }
}

struct FilesystemCapture;
impl CaptureSource for FilesystemCapture {}

struct CaptureRequest<'a> {
    sources: &'a [PathBuf],
    archive: &'a Path,
    plan: &'a SourcePlan,
    continue_on_error: bool,
    recovered: Option<&'a JournalReader>,
}

fn create_archive<W: ArchiveOutput>(output: W, sources: &[PathBuf], archive: &Path, plan: &SourcePlan,
    continue_on_error: bool, recovered: Option<&JournalReader>) -> Result<CaptureOutcome<W>> {
    capture_archive(output, CaptureRequest { sources, archive, plan, continue_on_error, recovered }, &mut FilesystemCapture)
}

fn persist_source_error<W: ArchiveOutput>(writer: &mut RecordWriter<W>, record: &JournalErrorRecord) -> Result<()> {
    writer.record_error(record).with_context(|| format!("Cannot persist source failure: {}", record.error_message))?;
    writer.output_mut().flush().with_context(|| format!("Cannot flush source failure: {}", record.error_message))?;
    writer.output_mut().sync().with_context(|| format!("Cannot sync source failure: {}", record.error_message))
}

fn persist_metadata_warnings<W: ArchiveOutput>(writer: &mut RecordWriter<W>, entity: &FileEntity,
    path: &Path, count: &mut usize) -> Result<()> {
    crate::journal::capture::visit_warnings(entity, &mut |_entity, message| {
        let warning = JournalWarningRecord {
            code: "FileReadWarning".to_owned(), message: format!("{}: {message}", path.display()),
            timestamp_sec: i64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs())?,
        };
        writer.record_warning(&warning).with_context(|| format!("Cannot persist metadata warning: {message}"))?;
        writer.output_mut().flush().with_context(|| format!("Cannot flush metadata warning: {message}"))?;
        writer.output_mut().sync().with_context(|| format!("Cannot sync metadata warning: {message}"))?;
        *count = count.saturating_add(1);
        Ok(())
    })
}

fn capture_archive<W: ArchiveOutput>(output: W, request: CaptureRequest<'_>, capture: &mut impl CaptureSource) -> Result<CaptureOutcome<W>> {
    let CaptureRequest { sources, archive, plan, continue_on_error, recovered } = request;
    let session = if let Some(recovered) = recovered {
        recovered.session.clone()
    } else {
        let working_directory = std::env::current_dir()?;
        JournalSessionRecord {
            sources: sources.iter().map(|source| working_directory.join(source)).collect(),
            destination: archive.to_path_buf(), origin_platform: u32::from(current_platform()),
            environment: Some(ctb_io_environment::capture_quick_arc()),
        }
    };
    let mut writer = if let Some(recovered) = recovered { recovered.resume_writer(output)? }
        else { RecordWriter::new(output, &session, crate::journal::stream::JournalMagic::ArchiveV1)? };
    let mut error_count = 0_usize;
    let mut warning_count = 0_usize;
    for diagnostic in &plan.diagnostics {
        persist_source_error(&mut writer, diagnostic)?;
        error_count = error_count.saturating_add(1);
    }
    if error_count != 0 && !continue_on_error {
        return Ok(CaptureOutcome { output: writer.into_output(), error_count, warning_count, completed: false });
    }
    let mut hardlinks = CommittedHardlinks::default();
    let mut tree = plan.tree.clone();
    let mut committed_paths = HashSet::new();
    if let Some(recovered) = recovered {
        for member in &recovered.members {
            let relative = member.entity.identity.relative_path.as_ref().context("Committed member missing path")?;
            committed_paths.insert(relative.clone());
            if matches!(member.entity.kind, FileEntityKind::Regular { .. }) {
                hardlinks.commit(&member.entity, relative, None);
            }
        }
    }
    for member in &plan.members {
        if committed_paths.contains(&member.relative) { continue; }
        let CapturedSource { mut entity, snapshot } = match capture.capture(&member.source) {
            Ok(source) => source,
            Err(error) => {
                persist_source_error(&mut writer, &JournalErrorRecord::from_error(&member.source, JournalErrorStage::MetadataRead, &error))?;
                error_count = error_count.saturating_add(1);
                if continue_on_error { continue; }
                return Ok(CaptureOutcome { output: writer.into_output(), error_count, warning_count, completed: false });
            }
        };
        tree.retain(&snapshot);
        crate::journal::capture::finalize_warnings(&snapshot, &mut entity, &mut |_, _| Ok(()))?;
        entity.identity.relative_path = Some(member.relative.clone());
        entity.identity.enclosing_path = None;
        if let Some(environment) = &session.environment { attach_environment(&mut entity, environment); }
        persist_metadata_warnings(&mut writer, &entity, &member.source, &mut warning_count)?;
        if let Err(error) = crate::journal::stream::ensure_clean_metadata(&entity) {
            persist_source_error(&mut writer, &JournalErrorRecord::from_error(&member.source, JournalErrorStage::MetadataRead, &error))?;
            error_count = error_count.saturating_add(1);
            if continue_on_error { continue; }
            return Ok(CaptureOutcome { output: writer.into_output(), error_count, warning_count, completed: false });
        }
        if matches!(entity.kind, FileEntityKind::Regular { .. }) && entity.identity.nlink > 1 {
            match hardlinks.lookup(&entity, &member.relative) {
                Ok(Some(target)) => entity.kind = FileEntityKind::Hardlink { target_relative_path: LinkPath::from_relative_path(target)? },
                Ok(None) => {},
                Err(error) => {
                    persist_source_error(&mut writer, &JournalErrorRecord::from_error(&member.source, JournalErrorStage::PostCheck, &error))?;
                    error_count = error_count.saturating_add(1);
                    if continue_on_error { continue; }
                    return Ok(CaptureOutcome { output: writer.into_output(), error_count, warning_count, completed: false });
                }
            }
        }
        let mut payload = if matches!(entity.kind, FileEntityKind::Regular { .. }) {
            match capture.open(&member.source).and_then(|mut payload| { payload.bind_snapshot(&snapshot)?; Ok(payload) }) {
                Ok(payload) => Some(payload),
                Err(error) => {
                    persist_source_error(&mut writer, &JournalErrorRecord::from_error(&member.source, JournalErrorStage::PayloadOpen, &error))?;
                    error_count = error_count.saturating_add(1);
                    if continue_on_error { continue; }
                    return Ok(CaptureOutcome { output: writer.into_output(), error_count, warning_count, completed: false });
                }
            }
        } else { None };
        let checkpoint = writer.output_mut().checkpoint()?;
        let precommit = || {
            tree.validate()?;
            let warnings = snapshot.validate()?;
            ensure!(warnings.is_empty(), "Source changed before commit: {}", warnings.join("; "));
            Ok(())
        };
        let result = if let Some(payload) = &mut payload {
            writer.write_entity_checked(&entity, Some(payload.as_mut()), precommit)
        } else {
            writer.write_entity_checked(&entity, None, precommit)
        };
        if let Err(error) = result {
            if writer.is_failed() {
                if !writer.can_rollback_source_failure() {
                    return Err(error).context("Archive output or unclassified member failure");
                }
                let position = checkpoint.with_context(|| format!("Cannot roll back forward-only archive after source failure: {error:#}"))?;
                writer.rollback_source_failure(|output| output.rollback(position))
                    .with_context(|| format!("Cannot roll back archive after source failure: {error:#}"))?;
            }
            persist_source_error(&mut writer, &JournalErrorRecord::from_error(&member.source, JournalErrorStage::PayloadRead, &error))?;
            error_count = error_count.saturating_add(1);
            if continue_on_error { continue; }
            return Ok(CaptureOutcome { output: writer.into_output(), error_count, warning_count, completed: false });
        }
        writer.output_mut().flush()?;
        writer.output_mut().sync()?;
        if matches!(entity.kind, FileEntityKind::Regular { .. }) {
            hardlinks.commit(&entity, &member.relative, Some(&snapshot));
        }
    }
    if let Err(error) = tree.validate() {
        persist_source_error(&mut writer, &JournalErrorRecord::from_error(archive, JournalErrorStage::PostCheck, &error))?;
        error_count = error_count.saturating_add(1);
        if !continue_on_error {
            return Ok(CaptureOutcome { output: writer.into_output(), error_count, warning_count, completed: false });
        }
    }
    let mut output = writer.finish()?;
    output.sync()?;
    Ok(CaptureOutcome { output, error_count, warning_count, completed: true })
}

fn resume_archive(path: &Path) -> Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let recovered = JournalReader::recover(file.try_clone()?)?;
    ensure!(!recovered.completed, "Cannot resume a completed archive");
    recovered.verify_self_contained_prefix()?;
    validate_members(&recovered)?;
    let plan = plan_sources(&recovered.session.sources, path)?;
    ensure!(plan.diagnostics.is_empty(), "Cannot resume with source planning diagnostics");
    let planned: HashMap<_, _> = plan.members.iter().map(|member| (member.relative.as_path(), member.source.as_path())).collect();
    for committed in &recovered.members {
        let relative = committed.entity.identity.relative_path.as_deref().context("Committed member missing path")?;
        let source = planned.get(relative).context("Committed source disappeared")?;
        let current = FileEntity::from_filesystem(source, None)?;
        ensure!(current.identity.origin == committed.entity.identity.origin, "Committed source identity changed: {}", source.display());
        let mut expected = committed.entity.clone();
        if matches!(expected.kind, FileEntityKind::Hardlink { .. }) {
            expected.kind = current.kind.clone();
        }
        ctb_io::file::verify_materialized_entity_ext(source, &expected, true, false, false)
            .with_context(|| format!("Committed source changed: {}", source.display()))?;
    }
    file.set_len(recovered.valid_length)?;
    file.sync_all()?;
    file.seek(SeekFrom::End(0))?;
    let outcome = create_archive(FileArchiveOutput::new(file), &recovered.session.sources, path, &plan, false, Some(&recovered))?;
    ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0, "Resumed archive has capture diagnostics");
    Ok(())
}

fn validate_members(reader: &JournalReader) -> Result<()> {
    let mut names = HashMap::new();
    for member in &reader.members {
        let path = member.entity.identity.relative_path.as_deref().context("Archive member missing relative path")?;
        let normalized = normalized_member(path)?;
        ensure!(names.insert(normalized, &member.entity.kind).is_none(), "Duplicate archive member: {}", path.display());
    }
    for (path, kind) in &names {
        if path == &PathBuf::from(".") {
            ensure!(matches!(kind, FileEntityKind::Directory | FileEntityKind::Bundle { .. }), "Archive root is not a directory");
        }
        for parent in path.ancestors().skip(1).filter(|parent| !parent.as_os_str().is_empty()) {
            if let Some(parent_kind) = names.get(parent) {
                ensure!(matches!(parent_kind, FileEntityKind::Directory | FileEntityKind::Bundle { .. }),
                    "Archive member descends through a non-directory: {}", parent.display());
            }
        }
        if let FileEntityKind::Hardlink { target_relative_path } = kind {
            let target = normalized_member(&target_relative_path.to_path_buf()?)?;
            ensure!(matches!(names.get(&target), Some(FileEntityKind::Regular { .. })),
                "Hardlink target must name a regular archive member: {}", target.display());
        }
    }
    Ok(())
}

fn extract_archive(reader: &JournalReader, destination: &Path, options: &MaterializeOptions) -> Result<()> {
    ensure!(cfg!(unix), "Sandboxed archive extraction is currently supported only on Unix");
    reader.verify_self_contained()?;
    validate_members(reader)?;
    if let Some(metadata) = existing_metadata(destination)? {
        ensure!(metadata.is_dir() && !metadata.file_type().is_symlink(), "Extraction destination is not a real directory");
    }
    for member in &reader.members {
        let relative = member.entity.identity.relative_path.as_deref().context("Missing archive member path")?;
        let path = destination.join(relative);
        for ancestor in path.ancestors().take_while(|ancestor| *ancestor != destination) {
            if let Some(metadata) = existing_metadata(ancestor)? {
                ensure!(!metadata.file_type().is_symlink(), "Extraction path contains an existing symlink: {}", ancestor.display());
            }
        }
        match &member.entity.kind {
            FileEntityKind::Fifo | FileEntityKind::CharDevice { .. } | FileEntityKind::BlockDevice { .. } => {
                ensure!(options.copy_specials, "Special member requires --copy-specials-as-specials: {}", relative.display());
            }
            FileEntityKind::Socket | FileEntityKind::Door | FileEntityKind::Inaccessible => bail!("Archive member cannot be restored: {}", relative.display()),
            FileEntityKind::Symlink { target } if options.symlink_policy != SymlinkValidationPolicy::PreserveVerbatim => {
                ctb_io::file::validate_symlink_target(destination, &path,
                    target.to_path_buf()?.as_os_str().as_encoded_bytes(), options.symlink_policy)?;
            }
            _ => {},
        }
        if relative != Path::new(".") {
            match std::fs::symlink_metadata(&path) {
                Ok(metadata) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::MetadataExt;
                        let archive_metadata = reader.file.metadata()?;
                        ensure!(metadata.dev() != archive_metadata.dev() || metadata.ino() != archive_metadata.ino(),
                            "Extraction would overwrite the archive itself: {}", path.display());
                    }
                    ensure!(options.existing_entry_policy == ctb_io::file::ExistingEntryPolicy::Replace,
                        "Extraction destination already exists; use --overwrite: {}", path.display());
                    ensure!((metadata.is_file() && matches!(member.entity.kind, FileEntityKind::Regular { .. }))
                        || (metadata.is_dir() && member.entity.is_dir()), "Cannot overwrite this entry type: {}", path.display());
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
                Err(error) => return Err(error.into()),
            }
        }
    }
    let sandbox = SandboxableDir::create_or_open(destination)?;
    for member in reader.members.iter().filter(|member| member.entity.is_dir()) {
        let relative = member.entity.identity.relative_path.as_deref().context("Missing directory path")?;
        sandbox.ensure_dir_all(relative, PathTraversalPolicy::StrictSandboxed)?;
        sandbox.authorize_metadata(relative, options.existing_entry_policy)?;
    }
    for hardlinks in [false, true] {
        for member in reader.members.iter().filter(|member| !member.entity.is_dir()
            && matches!(member.entity.kind, FileEntityKind::Hardlink { .. }) == hardlinks) {
            let receipt = if matches!(member.entity.kind, FileEntityKind::Regular { .. }) {
                let mut payload = reader.payload(member)?;
                member.entity.materialize(Some(&mut payload), &sandbox, options)?
            } else { member.entity.materialize(None, &sandbox, options)? };
            for warning in receipt.warnings { warn_fmt!("{warning}"); }
            if !hardlinks {
                ctb_io::file::verify_materialized_entity_ext(&receipt.destination_path, &member.entity, options.strict_lossless, false, false)?;
            }
        }
    }
    let mut directories: Vec<_> = reader.members.iter().filter(|member| member.entity.is_dir()).collect();
    directories.sort_by_key(|member| std::cmp::Reverse(member.entity.identity.relative_path.as_deref().map(Path::components).map(Iterator::count)));
    for member in directories {
        let relative = member.entity.identity.relative_path.as_deref().context("Missing directory path")?;
        if relative == Path::new(".") {
            sandbox.authorize_metadata(relative, options.existing_entry_policy)?;
            ctb_io::file::write_streams(destination, None, &member.entity.streams, options.strict_lossless)?;
            for warning in ctb_io::file::apply_entity_metadata(destination, None, &member.entity.metadata, false, true, options.strict_lossless)? {
                warn_fmt!("{warning}");
            }
        } else {
            let receipt = member.entity.materialize(None, &sandbox, options)?;
            for warning in receipt.warnings { warn_fmt!("{warning}"); }
        }
    }
    Ok(())
}

fn existing_metadata(path: &Path) -> Result<Option<std::fs::Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("Cannot inspect {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctb_formats_dcstring::DcMstring;
    use std::io::Read;

    #[derive(Clone, Copy)]
    enum CaptureFault { None, Capture, Open, Read, Finish, Warning }

    struct FaultCapture {
        path: PathBuf,
        fault: CaptureFault,
        opened: Vec<PathBuf>,
    }

    impl CaptureSource for FaultCapture {
        fn capture(&mut self, path: &Path) -> Result<CapturedSource> {
            if path == self.path && matches!(self.fault, CaptureFault::Capture) {
                bail!("injected metadata capture failure");
            }
            let mut source = FilesystemCapture.capture(path)?;
            if path == self.path && matches!(self.fault, CaptureFault::Warning) {
                let mut stream = ctb_io::file::AttachedStream::from_data(None,
                    ctb_io::file::StreamKind::infer_from_name(b"fixture"), Vec::new())?;
                stream.entity.metadata.warnings.push("injected recursive warning".to_owned());
                source.entity.streams.push(stream);
            }
            Ok(source)
        }
        fn open(&mut self, path: &Path) -> Result<Box<dyn PayloadSource>> {
            self.opened.push(path.to_path_buf());
            if path == self.path {
                if matches!(self.fault, CaptureFault::Open) { bail!("injected payload open failure"); }
                if matches!(self.fault, CaptureFault::Read | CaptureFault::Finish) {
                    return Ok(Box::new(FaultPayload {
                        source: DiskPayloadSource::open(path)?, bytes: 0,
                        finish_failure: matches!(self.fault, CaptureFault::Finish),
                    }));
                }
            }
            Ok(Box::new(DiskPayloadSource::open(path)?))
        }
    }

    struct FaultPayload {
        source: DiskPayloadSource,
        bytes: usize,
        finish_failure: bool,
    }

    impl Read for FaultPayload {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            if !self.finish_failure && self.bytes >= 32_768 {
                return Err(std::io::Error::other("injected source read failure after buffer boundary"));
            }
            let limit = if self.finish_failure { buffer.len() } else { buffer.len().min(32_768_usize.saturating_sub(self.bytes)) };
            let count = self.source.read(buffer.get_mut(..limit).ok_or_else(|| std::io::Error::other("Invalid test read limit"))?)?;
            self.bytes = self.bytes.saturating_add(count);
            Ok(count)
        }
    }

    impl Seek for FaultPayload {
        fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> { self.source.seek(position) }
    }

    impl PayloadSource for FaultPayload {
        fn bind_snapshot(&mut self, snapshot: &ctb_io::file::payload::SourceSnapshot) -> Result<()> {
            self.source.bind_snapshot(snapshot)
        }
        fn total_size(&self) -> u64 { self.source.total_size() }
        fn extents(&self) -> &[ctb_io::file::Extent] { self.source.extents() }
        fn finish(&mut self) -> Result<Vec<String>> {
            if self.finish_failure { bail!("injected source finish failure"); }
            self.source.finish()
        }
    }

    #[derive(Clone, Default)]
    struct ForwardOutput(std::rc::Rc<std::cell::RefCell<Vec<u8>>>);

    impl Write for ForwardOutput {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    impl ArchiveOutput for ForwardOutput {}

    #[derive(Clone, Copy, Debug)]
    enum OutputFault { Write, WriteZero, Flush, Sync, Rollback, Diagnostic }

    struct FaultOutput {
        output: FileArchiveOutput,
        fault: OutputFault,
        started: bool,
        rolled_back: bool,
    }

    impl Write for FaultOutput {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.started && matches!(self.fault, OutputFault::Write) ||
                self.rolled_back && matches!(self.fault, OutputFault::Diagnostic) {
                return Err(std::io::Error::other("injected sink write failure"));
            }
            if self.started && matches!(self.fault, OutputFault::WriteZero) { return Ok(0); }
            self.output.write(bytes)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            if self.started && matches!(self.fault, OutputFault::Flush) {
                return Err(std::io::Error::other("injected sink flush failure"));
            }
            self.output.flush()
        }
    }

    impl ArchiveOutput for FaultOutput {
        fn checkpoint(&mut self) -> Result<Option<u64>> {
            let position = self.output.checkpoint()?;
            self.started = true;
            Ok(position)
        }
        fn rollback(&mut self, position: u64) -> Result<()> {
            if matches!(self.fault, OutputFault::Rollback) { bail!("injected rollback failure"); }
            self.output.rollback(position)?;
            self.rolled_back = true;
            Ok(())
        }
        fn sync(&mut self) -> Result<()> {
            if self.started && matches!(self.fault, OutputFault::Sync) { bail!("injected sink sync failure"); }
            self.output.sync()
        }
    }

    fn capture_request<'a>(sources: &'a [PathBuf], archive: &'a Path, plan: &'a SourcePlan,
        continue_on_error: bool) -> CaptureRequest<'a> {
        CaptureRequest { sources, archive, plan, continue_on_error, recovered: None }
    }

    #[crate::ctb_test]
    fn archive_raw_apple_source_free_roundtrip() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir_all(source.join("__MACOSX"))?;
        std::fs::create_dir(source.join(".AppleDouble"))?;
        let mut apple = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleDouble,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: None, comment: None, timestamps: None, backup_timestamp_sec: None,
            finder_info: None, extended_attributes: Vec::new(), data_fork: None,
            resource_fork: Some(b"resource".to_vec()), data_fork_size: None,
            resource_fork_size: Some(8), unrecognized_entries: Vec::new(), entries: Vec::new(),
        };
        let double = ctb_io::file::write_apple_single_double(&apple)?;
        apple.format = ctb_io::file::AppleFormat::AppleSingle;
        apple.data_fork = Some(b"single primary".to_vec());
        let single = ctb_io::file::write_apple_single_double(&apple)?;
        let originals = [
            ("valid", b"primary valid".to_vec()), ("._valid", double.clone()),
            ("malformed", b"primary malformed".to_vec()), ("._malformed", b"malformed".to_vec()),
            ("._orphan", b"ordinary dot underscore".to_vec()),
            ("__MACOSX/._valid", double.clone()), (".AppleDouble/valid", double),
            ("single", single.clone()), ("single.as", single.clone()), ("single.asf", single),
        ];
        for (relative, bytes) in &originals { std::fs::write(source.join(relative), bytes)?; }
        let sources = vec![source.clone()];
        let archive = temporary.path().join("backup.cscarchive");
        let plan = plan_sources(&sources, &archive)?;
        ensure!(plan.diagnostics.is_empty());
        let expected: HashSet<_> = originals.iter().map(|(relative, _)| Path::new("source").join(relative))
            .chain([PathBuf::from("source"), PathBuf::from("source/__MACOSX"), PathBuf::from("source/.AppleDouble")]).collect();
        ensure!(plan.members.iter().map(|member| member.relative.clone()).collect::<HashSet<_>>() == expected);
        let outcome = create_archive(FileArchiveOutput::new(File::create(&archive)?), &sources, &archive, &plan, false, None)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        std::fs::remove_dir_all(&source)?;
        let reader = JournalReader::new(File::open(&archive)?)?;
        reader.verify()?;
        ensure!(reader.members.iter().filter_map(|member| member.entity.identity.relative_path.clone()).collect::<HashSet<_>>() == expected);
        for (relative, bytes) in &originals {
            let member = reader.members.iter().find(|member| member.entity.identity.relative_path.as_deref() == Some(Path::new("source").join(relative).as_path()))
                .context("Missing raw Apple member")?;
            ensure!(member.entity.streams.is_empty() && member.entity.metadata.apple.is_none());
            let mut payload = reader.payload(member)?;
            let mut archived = Vec::new();
            payload.read_to_end(&mut archived)?;
            payload.finish()?;
            ensure!(&archived == bytes, "Raw container bytes changed: {relative}");
        }
        let destination = temporary.path().join("restored");
        extract_archive(&reader, &destination, &MaterializeOptions { strict_lossless: false, ..Default::default() })?;
        for (relative, bytes) in &originals {
            ensure!(std::fs::read(destination.join("source").join(relative))? == *bytes);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_raw_capture_has_no_companion_dependency() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("main");
        let companion = temporary.path().join("._main");
        std::fs::write(&path, b"primary")?;
        std::fs::write(&companion, b"malformed")?;
        let raw = FilesystemCapture.capture(&path)?;
        ensure!(CapturedSource::capture(&path, None, &AppleReadOptions::default(), false).is_err());
        std::fs::write(&companion, b"changed companion")?;
        ensure!(raw.snapshot.validate()?.is_empty());
        let mut payload = raw.open(&path)?;
        let mut bytes = Vec::new();
        payload.read_to_end(&mut bytes)?;
        payload.finish()?;
        ensure!(bytes == b"primary");
        #[cfg(unix)]
        {
            std::fs::remove_file(&companion)?;
            std::os::unix::fs::symlink("missing", &companion)?;
            FilesystemCapture.capture(&path)?;
            ensure!(CapturedSource::capture(&path, None, &AppleReadOptions::default(), false).is_err());
            ensure!(std::fs::read_link(&companion)? == Path::new("missing"));
            ensure!(std::fs::read(&path)? == b"primary");
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn capture_final_tree_failure_obeys_continue_policy() -> Result<()> {
        struct MutatingOutput { output: FileArchiveOutput, root: PathBuf, syncs: usize }
        impl Write for MutatingOutput {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> { self.output.write(bytes) }
            fn flush(&mut self) -> std::io::Result<()> { self.output.flush() }
        }
        impl ArchiveOutput for MutatingOutput {
            fn checkpoint(&mut self) -> Result<Option<u64>> { self.output.checkpoint() }
            fn rollback(&mut self, position: u64) -> Result<()> { self.output.rollback(position) }
            fn sync(&mut self) -> Result<()> {
                self.output.sync()?;
                self.syncs = self.syncs.saturating_add(1);
                if self.syncs == 2 { std::fs::write(self.root.join("late"), b"late")?; }
                Ok(())
            }
        }
        for continue_on_error in [false, true] {
            let temporary = tempfile::tempdir()?;
            let root = temporary.path().join("source");
            std::fs::create_dir(&root)?;
            std::fs::write(root.join("child"), b"primary")?;
            let sources = vec![root.clone()];
            let archive = temporary.path().join("output");
            let plan = plan_sources(&sources, &archive)?;
            let output = MutatingOutput { output: FileArchiveOutput::new(File::create(&archive)?), root, syncs: 0 };
            let outcome = create_archive(output, &sources, &archive, &plan, continue_on_error, None)?;
            ensure!(outcome.error_count == 1 && outcome.completed == continue_on_error);
            drop(outcome.output);
            let reader = JournalReader::recover(File::open(&archive)?)?;
            ensure!(reader.members.len() == 2 && reader.errors.len() == 1);
            ensure!(reader.is_completed() == continue_on_error && reader.verify().is_err());
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_archive_retains_snapshot_cleanup_warning() -> Result<()> {
        struct CleanupCapture;
        impl CaptureSource for CleanupCapture {
            fn capture(&mut self, path: &Path) -> Result<CapturedSource> {
                let source = FilesystemCapture.capture(path)?;
                source.snapshot.retain_cleanup_warnings(vec!["injected symlink cleanup warning".to_owned()])?;
                Ok(source)
            }
        }
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("link");
        std::os::unix::fs::symlink("target", &path)?;
        let sources = vec![path];
        let archive = temporary.path().join("output");
        let plan = plan_sources(&sources, &archive)?;
        let outcome = capture_archive(FileArchiveOutput::new(File::create(&archive)?),
            capture_request(&sources, &archive, &plan, false), &mut CleanupCapture)?;
        ensure!(!outcome.completed && outcome.warning_count > 0 && outcome.error_count == 1);
        drop(outcome.output);
        let reader = JournalReader::recover(File::open(&archive)?)?;
        ensure!(reader.warnings.iter().any(|warning| warning.message.contains("injected symlink cleanup")));
        ensure!(reader.members.is_empty());
        ensure!(reader.errors.iter().any(|error| error.error_message.contains("injected symlink cleanup")));
        ensure!(reader.verify().is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn capture_tree_rejects_planning_and_later_child_changes() -> Result<()> {
        struct AddChild { root: PathBuf }
        impl CaptureSource for AddChild {
            fn capture(&mut self, path: &Path) -> Result<CapturedSource> {
                if path != self.root { std::fs::write(self.root.join("added"), b"new")?; }
                FilesystemCapture.capture(path)
            }
        }
        for during_capture in [false, true] {
            for continue_on_error in [false, true] {
                let temporary = tempfile::tempdir()?;
                let root = temporary.path().join("source");
                std::fs::create_dir(&root)?;
                std::fs::write(root.join("child"), b"original")?;
                let sources = vec![root.clone()];
                let archive = temporary.path().join("output");
                let plan = plan_sources(&sources, &archive)?;
                if !during_capture { std::fs::write(root.join("added"), b"new")?; }
                let outcome = capture_archive(FileArchiveOutput::new(File::create(&archive)?),
                    capture_request(&sources, &archive, &plan, continue_on_error), &mut AddChild { root })?;
                ensure!(outcome.error_count > 0);
                ensure!(outcome.completed == continue_on_error);
                drop(outcome.output);
                let reader = JournalReader::recover(File::open(&archive)?)?;
                ensure!(reader.has_diagnostics() && reader.verify().is_err());
                if during_capture { ensure!(reader.members.len() == 1); }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_planning_diagnostics_continue_and_default_stop() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let missing = temporary.path().join("missing");
        let good = temporary.path().join("good");
        std::fs::write(&good, b"good contents")?;
        for continue_on_error in [false, true] {
            let path = temporary.path().join(format!("{continue_on_error}.cscarchive"));
            let result = run_archive(&ArchiveArgs { command: ArchiveCommand::Create {
                archive: path.clone(), sources: vec![missing.clone(), good.clone()], resume: false, continue_on_error,
            } })?;
            ensure!(matches!(result, ToolResult::Immediate { exit_code: 1, stderr, stdout }
                if stdout.is_empty() && !String::from_utf8_lossy(&stderr).contains("Archive created.")));
            let reader = JournalReader::recover(File::open(&path)?)?;
            ensure!(reader.errors.len() == 1 && reader.errors.first().context("Missing diagnostic")?.path == missing);
            ensure!(reader.errors.first().context("Missing diagnostic")?.stage == JournalErrorStage::MetadataRead);
            ensure!(reader.is_completed() == continue_on_error);
            ensure!(reader.members.len() == usize::from(continue_on_error));
            ensure!(reader.verify().is_err());
            if continue_on_error { reader.verify_integrity()?; }
            let before = std::fs::read(&path)?;
            ensure!(resume_archive(&path).is_err());
            ensure!(std::fs::read(&path)? == before);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_safety_failures_never_continue() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        for sources in [vec![source.clone()], vec![source.clone(), source.clone()]] {
            let archive = if sources.len() == 1 { source.join("overlap.cscarchive") }
                else { temporary.path().join("duplicate.cscarchive") };
            ensure!(run_archive(&ArchiveArgs { command: ArchiveCommand::Create {
                archive: archive.clone(), sources, resume: false, continue_on_error: true,
            } }).is_err());
            ensure!(!archive.exists());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_runtime_removed_capture_open_and_recursive_warning() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let failed = temporary.path().join("failed");
        let good = temporary.path().join("good");
        std::fs::write(&good, b"good")?;
        for fault in [CaptureFault::None, CaptureFault::Capture, CaptureFault::Open, CaptureFault::Warning] {
            for continue_on_error in [false, true] {
                std::fs::write(&failed, b"failed")?;
                let archive = temporary.path().join("runtime.cscarchive");
                let sources = vec![failed.clone(), good.clone()];
                let plan = plan_sources(&sources, &archive)?;
                if matches!(fault, CaptureFault::None) { std::fs::remove_file(&failed)?; }
                let mut capture = FaultCapture { path: failed.clone(), fault, opened: Vec::new() };
                let outcome = capture_archive(FileArchiveOutput::new(File::create(&archive)?),
                    capture_request(&sources, &archive, &plan, continue_on_error), &mut capture)?;
                ensure!(outcome.error_count == 1 && outcome.completed == continue_on_error);
                ensure!(outcome.warning_count == usize::from(matches!(fault, CaptureFault::Warning)));
                drop(outcome.output);
                let reader = JournalReader::recover(File::open(&archive)?)?;
                ensure!(reader.errors.len() == 1 && reader.members.len() == usize::from(continue_on_error));
                let expected_stage = if matches!(fault, CaptureFault::Open) { JournalErrorStage::PayloadOpen }
                    else { JournalErrorStage::MetadataRead };
                ensure!(reader.errors.first().context("Missing runtime diagnostic")?.stage == expected_stage);
                ensure!(reader.warnings.len() == usize::from(matches!(fault, CaptureFault::Warning)));
                if matches!(fault, CaptureFault::Warning) {
                    ensure!(!capture.opened.contains(&failed));
                    ensure!(reader.warnings.first().context("Missing warning")?.message.contains("injected recursive warning"));
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_buffered_source_rollback_exact_members_and_sequence() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let first = temporary.path().join("first");
        let failed = temporary.path().join("failed");
        let last = temporary.path().join("last");
        std::fs::write(&first, b"first committed contents")?;
        std::fs::write(&failed, vec![0xAC; 131_072])?;
        std::fs::write(&last, b"last committed contents")?;
        let archive = temporary.path().join("rollback.cscarchive");
        let sources = vec![first, failed.clone(), last];
        let plan = plan_sources(&sources, &archive)?;
        for fault in [CaptureFault::Read, CaptureFault::Finish] {
            for continue_on_error in [false, true] {
                let mut capture = FaultCapture { path: failed.clone(), fault, opened: Vec::new() };
                let outcome = capture_archive(FileArchiveOutput::new(File::create(&archive)?),
                    capture_request(&sources, &archive, &plan, continue_on_error), &mut capture)?;
                ensure!(outcome.error_count == 1 && outcome.warning_count == 0 && outcome.completed == continue_on_error);
                drop(outcome.output);
                let reader = JournalReader::recover(File::open(&archive)?)?;
                ensure!(reader.errors.len() == 1 && reader.members.len() == if continue_on_error { 2 } else { 1 });
                ensure!(reader.sequence == if continue_on_error { 3 } else { 2 });
                let mut contents = Vec::new();
                reader.payload(reader.members.first().context("Missing first member")?)?.read_to_end(&mut contents)?;
                ensure!(contents == b"first committed contents");
                if continue_on_error {
                    reader.verify_integrity()?;
                    contents.clear();
                    reader.payload(reader.members.last().context("Missing last member")?)?.read_to_end(&mut contents)?;
                    ensure!(contents == b"last committed contents");
                }
                ensure!(reader.verify().is_err());
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_forward_only_source_fault_aborts_without_completion() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let failed = temporary.path().join("failed");
        let good = temporary.path().join("good");
        std::fs::write(&failed, vec![0xAC; 131_072])?;
        std::fs::write(&good, b"not attempted")?;
        let archive = temporary.path().join("stdout.cscarchive");
        let sources = vec![failed.clone(), good.clone()];
        let plan = plan_sources(&sources, &archive)?;
        for continue_on_error in [false, true] {
            let output = ForwardOutput::default();
            let bytes = output.0.clone();
            let mut capture = FaultCapture { path: failed.clone(), fault: CaptureFault::Read, opened: Vec::new() };
            let error = capture_archive(output, capture_request(&sources, &archive, &plan, continue_on_error), &mut capture)
                .err().context("Forward output accepted a source fault")?;
            ensure!(format!("{error:#}").contains("injected source read failure"));
            ensure!(!capture.opened.contains(&good) && bytes.borrow().len() > 32_768);
            std::fs::write(&archive, &*bytes.borrow())?;
            let reader = JournalReader::recover(File::open(&archive)?)?;
            ensure!(!reader.is_completed() && reader.members.is_empty() && reader.errors.is_empty());
            ensure!(JournalReader::new(File::open(&archive)?).is_err());
        }
        let mut cursor = std::io::Cursor::new(Vec::new());
        ensure!(cursor.checkpoint()?.is_none() && cursor.rollback(0).is_err());
        let mut vector = Vec::new();
        ensure!(vector.checkpoint()?.is_none() && vector.rollback(0).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_sink_diagnostic_and_rollback_failures_abort() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let failed = temporary.path().join("failed");
        let good = temporary.path().join("good");
        std::fs::write(&failed, vec![0xAC; 131_072])?;
        std::fs::write(&good, b"not attempted")?;
        let archive = temporary.path().join("sink.cscarchive");
        let sources = vec![failed.clone(), good.clone()];
        let plan = plan_sources(&sources, &archive)?;
        for fault in [OutputFault::Write, OutputFault::WriteZero, OutputFault::Flush,
            OutputFault::Sync, OutputFault::Rollback, OutputFault::Diagnostic] {
            let output = FaultOutput {
                output: FileArchiveOutput::new(File::create(&archive)?), fault, started: false, rolled_back: false,
            };
            let source_fault = if matches!(fault, OutputFault::Rollback | OutputFault::Diagnostic) { CaptureFault::Read }
                else { CaptureFault::None };
            let mut capture = FaultCapture { path: failed.clone(), fault: source_fault, opened: Vec::new() };
            let error = capture_archive(output, capture_request(&sources, &archive, &plan, true), &mut capture)
                .err().with_context(|| format!("Sink fault accepted: {fault:?}"))?;
            if matches!(fault, OutputFault::Rollback | OutputFault::Diagnostic) {
                ensure!(format!("{error:#}").contains("injected source read failure"));
            }
            ensure!(!capture.opened.contains(&good));
            let reader = JournalReader::recover(File::open(&archive)?)?;
            ensure!(!reader.is_completed() && reader.errors.is_empty());
        }
        Ok(())
    }

    #[crate::ctb_test]
    #[cfg(unix)]
    fn archive_failed_hardlink_definition_is_never_referenced() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let failed = temporary.path().join("failed");
        let definition = temporary.path().join("definition");
        let reference = temporary.path().join("reference");
        std::fs::write(&failed, vec![0xCA; 131_072])?;
        std::fs::hard_link(&failed, &definition)?;
        std::fs::hard_link(&failed, &reference)?;
        let archive = temporary.path().join("hardlinks.cscarchive");
        let sources = vec![failed.clone(), definition.clone(), reference];
        let plan = plan_sources(&sources, &archive)?;
        let mut capture = FaultCapture { path: failed, fault: CaptureFault::Read, opened: Vec::new() };
        let outcome = capture_archive(FileArchiveOutput::new(File::create(&archive)?),
            capture_request(&sources, &archive, &plan, true), &mut capture)?;
        ensure!(outcome.error_count == 1 && outcome.completed);
        drop(outcome.output);
        let reader = JournalReader::new(File::open(&archive)?)?;
        reader.verify_integrity()?;
        validate_members(&reader)?;
        ensure!(reader.members.len() == 2);
        ensure!(matches!(reader.members.first().context("Missing definition")?.entity.kind, FileEntityKind::Regular { .. }));
        let FileEntityKind::Hardlink { target_relative_path } = &reader.members.last().context("Missing hardlink")?.entity.kind else {
            bail!("Expected a committed hardlink reference");
        };
        ensure!(target_relative_path.to_path_buf()? == Path::new("definition"));
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_diagnostic_status_rejects_extraction() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let archive = temporary.path().join("failed.cscarchive");
        let destination = temporary.path().join("destination");
        let source = temporary.path().join("unreadable");
        let session = JournalSessionRecord {
            sources: vec![source.clone()], destination: archive.clone(),
            origin_platform: u32::from(current_platform()), environment: None,
        };
        let mut writer = RecordWriter::new(File::create(&archive)?, &session, crate::journal::stream::JournalMagic::ArchiveV1)?;
        writer.record_error(&crate::journal::JournalErrorRecord {
            path: source.clone(), raw_path: source.as_os_str().as_encoded_bytes().to_vec(),
            stage: crate::journal::JournalErrorStage::MetadataRead,
            error_message: "Injected capture failure".to_owned(), os_error: Some(13), timestamp_sec: 0,
        })?;
        drop(writer.finish()?);
        let listed = run_archive(&ArchiveArgs { command: ArchiveCommand::List { archive: archive.clone() } })?;
        ensure!(matches!(listed, ToolResult::Immediate { exit_code: 1, stderr, .. }
            if String::from_utf8_lossy(&stderr).contains("Injected capture failure")));
        ensure!(run_archive(&ArchiveArgs { command: ArchiveCommand::Verify { archive: archive.clone() } }).is_err());
        ensure!(run_archive(&ArchiveArgs { command: ArchiveCommand::Extract {
            archive, destination: destination.clone(), overwrite: false,
            trust_symlink_targets: false, best_effort_metadata: true, copy_specials_as_specials: false,
        } }).is_err());
        ensure!(!destination.exists(), "Failed capture modified extraction destination");
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_stdout_argument_parsing() -> Result<()> {
        let args = ArchiveArgs::try_parse_from(["archive", "create", "-", "source/"])?;
        ensure!(matches!(args.command, ArchiveCommand::Create { archive, sources, resume: false, continue_on_error: false }
            if archive == Path::new("-") && sources == vec![PathBuf::from("source/")]));
        ensure!(ArchiveArgs::try_parse_from(["archive", "create", "output", "--resume", "--continue-on-error"]).is_err());
        let stdout_resume = ArchiveArgs::try_parse_from(["archive", "create", "-", "--resume"])?;
        ensure!(run_archive(&stdout_resume).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    #[cfg(unix)]
    fn archive_raw_names_source_free_roundtrip() -> Result<()> {
        use std::os::unix::ffi::OsStringExt;

        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        let archive = temporary.path().join("names.cscarchive");
        std::fs::create_dir(&source)?;
        let names = [
            std::ffi::OsString::from_vec(b"name\xff".to_vec()),
            std::ffi::OsString::from_vec(b"name\xfe".to_vec()),
            std::ffi::OsString::from("caf\u{e9}"),
            std::ffi::OsString::from("cafe\u{301}"),
        ];
        for name in &names {
            std::fs::create_dir(source.join(name))?;
            std::fs::write(source.join(name).join(name), name.as_encoded_bytes())?;
        }
        run_archive(&ArchiveArgs { command: ArchiveCommand::Create {
            archive: archive.clone(), sources: vec![source.join("")], resume: false, continue_on_error: false,
        } })?;
        for name in &names {
            std::fs::remove_file(source.join(name).join(name))?;
            std::fs::remove_dir(source.join(name))?;
        }
        std::fs::remove_dir(source)?;
        let reader = JournalReader::new(File::open(&archive)?)?;
        reader.verify()?;
        extract_archive(&reader, &destination, &MaterializeOptions {
            strict_lossless: false, ..Default::default()
        })?;
        for name in &names {
            ensure!(std::fs::read(destination.join(name).join(name))? == name.as_encoded_bytes());
        }
        ensure!(std::fs::read_dir(destination)?.count() == names.len());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_source_free_tree_roundtrip() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        let data = vec![0xA5; 131_072];
        std::fs::write(source.join("data"), &data)?;
        std::fs::write(source.join("empty"), [])?;
        let archive_path = temporary.path().join("backup.cscarchive");
        let sources = vec![source.clone()];
        let plan = plan_sources(&sources, &archive_path)?;
        let file = File::create(&archive_path)?;
        let outcome = create_archive(FileArchiveOutput::new(file), &sources, &archive_path, &plan, false, None)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        std::fs::remove_dir_all(&source)?;
        let reader = JournalReader::new(File::open(&archive_path)?)?;
        reader.verify()?;
        let destination = temporary.path().join("restored");
        let options = MaterializeOptions { strict_lossless: false, force_overwrite: true, ..Default::default() };
        extract_archive(&reader, &destination, &options)?;
        ensure!(std::fs::read(destination.join("source/data"))? == data);
        ensure!(std::fs::read(destination.join("source/empty"))?.is_empty());
        ensure!(extract_archive(&reader, &destination, &options).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    #[cfg(unix)]
    fn archive_existing_root_requires_overwrite() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        let archive = temporary.path().join("backup.cscarchive");
        std::fs::create_dir(&source)?;
        std::fs::create_dir(&destination)?;
        std::fs::write(source.join("data"), b"new payload")?;
        std::fs::write(destination.join("data"), b"original payload")?;
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o700))?;
        std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o755))?;
        run_archive(&ArchiveArgs { command: ArchiveCommand::Create {
            archive: archive.clone(), sources: vec![source.join("")], resume: false, continue_on_error: false,
        } })?;
        let reader = JournalReader::new(File::open(archive)?)?;
        let options = MaterializeOptions { strict_lossless: false, ..Default::default() };
        ensure!(extract_archive(&reader, &destination, &options).is_err());
        ensure!(std::fs::read(destination.join("data"))? == b"original payload");
        ensure!(std::fs::metadata(&destination)?.permissions().mode() & 0o777 == 0o755);
        std::fs::remove_file(destination.join("data"))?;
        ensure!(extract_archive(&reader, &destination, &options).is_err());
        ensure!(!destination.join("data").exists());
        let options = MaterializeOptions {
            existing_entry_policy: ctb_io::file::ExistingEntryPolicy::Replace,
            ..options
        };
        extract_archive(&reader, &destination, &options)?;
        ensure!(std::fs::read(destination.join("data"))? == b"new payload");
        ensure!(std::fs::metadata(destination)?.permissions().mode() & 0o777 == 0o700);
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_resume_rejects_changed_source_before_truncation() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("data");
        std::fs::write(&source, b"unchanged")?;
        let path = temporary.path().join("backup.cscarchive");
        let sources = vec![source.clone()];
        let plan = plan_sources(&sources, &path)?;
        let outcome = create_archive(File::create(&path)?, &sources, &path, &plan, false, None)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        let completed = std::fs::read(&path)?;
        let mut marker = DcMstring::new();
        marker.push_char(ctb_formats_dcdata::dc::DC_OPERATION_COMPLETED);
        let recovered_length = u64::try_from(completed.len())?.checked_sub(u64::try_from(marker.payload_bytes().len())?).context("Archive too short")?;
        OpenOptions::new().write(true).open(&path)?.set_len(recovered_length)?;
        let before = std::fs::read(&path)?;
        std::fs::write(&source, b"modified!")?;
        ensure!(resume_archive(&path).is_err());
        ensure!(std::fs::read(&path)? == before, "Failed source validation changed the archive");
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_resume_unclosed_tail_and_complete() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("data");
        std::fs::write(&source, vec![0x5A; 16_384])?;
        let path = temporary.path().join("backup.cscarchive");
        let sources = vec![source];
        let plan = plan_sources(&sources, &path)?;
        let outcome = create_archive(File::create(&path)?, &sources, &path, &plan, false, None)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        let length = std::fs::metadata(&path)?.len();
        let mut marker = DcMstring::new();
        marker.push_char(ctb_formats_dcdata::dc::DC_OPERATION_COMPLETED);
        OpenOptions::new().write(true).open(&path)?.set_len(length.checked_sub(u64::try_from(marker.payload_bytes().len())?).context("Missing completion marker")?)?;
        let mut tail = DcMstring::new();
        tail.push_char(ctb_formats_dcdata::dc::DC_BEGIN_CHECKSUMMED_SECTION);
        OpenOptions::new().append(true).open(&path)?.write_all(tail.payload_bytes())?;
        resume_archive(&path)?;
        let reader = JournalReader::new(File::open(&path)?)?;
        reader.verify()?;
        ensure!(reader.members.len() == 1);
        ensure!(resume_archive(&path).is_err(), "Completed archive accepted append/resume");
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_preflight_rejects_traversal_and_symlink_ancestors() -> Result<()> {
        ensure!(normalized_member(Path::new("../outside")).is_err());
        ensure!(normalized_member(Path::new("/outside")).is_err());
        ensure!(normalized_member(Path::new("C:/outside")).is_err());
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        std::fs::write(source.join("data"), b"archive")?;
        let path = temporary.path().join("backup.cscarchive");
        let sources = vec![source];
        let plan = plan_sources(&sources, &path)?;
        let outcome = create_archive(File::create(&path)?, &sources, &path, &plan, false, None)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        let reader = JournalReader::new(File::open(&path)?)?;
        let destination = temporary.path().join("destination");
        std::fs::create_dir(&destination)?;
        #[cfg(unix)]
        {
            let outside = temporary.path().join("outside");
            std::fs::create_dir(&outside)?;
            std::os::unix::fs::symlink(&outside, destination.join("source"))?;
            ensure!(extract_archive(&reader, &destination, &MaterializeOptions {
                existing_entry_policy: ctb_io::file::ExistingEntryPolicy::Replace,
                ..Default::default()
            }).is_err());
            ensure!(!outside.join("data").exists());
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn archive_links_and_attached_streams_capture_fidelity_contract() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        std::fs::write(source.join("data"), b"linked payload")?;
        std::fs::hard_link(source.join("data"), source.join("hardlink"))?;
        std::os::unix::fs::symlink("data", source.join("symlink"))?;
        xattr::set(source.join("data"), "user.ctb.archive", b"stream payload")?;
        let archive_path = temporary.path().join("backup.cscarchive");
        let sources = vec![source.clone()];
        let plan = plan_sources(&sources, &archive_path)?;
        let outcome = create_archive(File::create(&archive_path)?, &sources, &archive_path, &plan, false, None)?;
        let clean = outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0;
        if !clean {
            ensure!(!outcome.completed && outcome.error_count == 1 && outcome.warning_count > 0);
        }
        let error_count = outcome.error_count;
        let warning_count = outcome.warning_count;
        drop(outcome.output);
        std::fs::remove_dir_all(&source)?;
        if !clean {
            let reader = JournalReader::recover(File::open(&archive_path)?)?;
            ensure!(!reader.is_completed());
            ensure!(reader.errors.len() == error_count && reader.warnings.len() == warning_count);
            let link = source.join("symlink");
            ensure!(reader.warnings.iter().any(|warning| warning.message.contains(&link.display().to_string())
                && warning.message.contains("Restoring source access time changed source change time")
                && warning.message.contains("Unix cannot restore change time")));
            ensure!(reader.errors.iter().any(|error| error.path == link
                && error.stage == JournalErrorStage::MetadataRead
                && error.error_message.contains("Unix cannot restore change time")));
            ensure!(!reader.members.iter().any(|member| member.entity.identity.relative_path.as_deref() == Some(Path::new("source/symlink"))));
            ensure!(reader.verify().is_err());
            ensure!(JournalReader::new(File::open(&archive_path)?).is_err());
            let listed = run_archive(&ArchiveArgs { command: ArchiveCommand::List { archive: archive_path.clone() } })?;
            ensure!(matches!(listed, ToolResult::Immediate { exit_code: 1, stderr, .. }
                if String::from_utf8_lossy(&stderr).contains("Unix cannot restore change time")));
            ensure!(run_archive(&ArchiveArgs { command: ArchiveCommand::Verify { archive: archive_path.clone() } }).is_err());
            let destination = temporary.path().join("refused");
            ensure!(extract_archive(&reader, &destination, &MaterializeOptions {
                strict_lossless: false, ..Default::default()
            }).is_err());
            ensure!(!destination.exists());
        } else {
            let reader = JournalReader::new(File::open(&archive_path)?)?;
            ensure!(reader.is_completed() && reader.errors.is_empty() && reader.warnings.is_empty());
            assert_links_and_attached_streams_restored(&reader, &temporary.path().join("restored"))?;
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn archive_links_and_attached_streams_declared_source_free_roundtrip() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let archive_path = temporary.path().join("declared.cscarchive");
        let session = JournalSessionRecord {
            sources: Vec::new(), destination: archive_path.clone(),
            origin_platform: u32::from(current_platform()), environment: None,
        };
        let mut writer = RecordWriter::new(File::create(&archive_path)?, &session,
            crate::journal::stream::JournalMagic::ArchiveV1)?;
        let mut data = FileEntity::from_vec(b"linked payload".to_vec());
        data.identity.relative_path = Some(PathBuf::from("source/data"));
        data.streams.push(ctb_io::file::AttachedStream::from_data(
            Some(ctb_io::file::StreamName::from_bytes(b"user.ctb.archive")),
            ctb_io::file::StreamKind::infer_from_name(b"user.ctb.archive"), b"stream payload".to_vec())?);
        let mut payload = ctb_io::file::MemoryPayloadSource::new(b"linked payload".to_vec())?;
        writer.write_entity(&data, Some(&mut payload))?;
        let mut hardlink = FileEntity::from_vec(Vec::new());
        hardlink.identity.relative_path = Some(PathBuf::from("source/hardlink"));
        hardlink.kind = FileEntityKind::Hardlink { target_relative_path: LinkPath::from_relative_path(Path::new("source/data"))? };
        writer.write_entity(&hardlink, None)?;
        let mut symlink = FileEntity::from_vec(Vec::new());
        symlink.identity.relative_path = Some(PathBuf::from("source/symlink"));
        symlink.kind = FileEntityKind::Symlink { target: LinkPath::from_relative_path(Path::new("data"))? };
        writer.write_entity(&symlink, None)?;
        drop(writer.finish()?);
        let reader = JournalReader::new(File::open(&archive_path)?)?;
        ensure!(reader.is_completed() && reader.errors.is_empty() && reader.warnings.is_empty());
        ensure!(reader.members.len() == 3);
        assert_links_and_attached_streams_restored(&reader, &temporary.path().join("restored"))
    }

    #[cfg(unix)]
    fn assert_links_and_attached_streams_restored(reader: &JournalReader, destination: &Path) -> Result<()> {
        use std::os::unix::fs::MetadataExt;
        reader.verify()?;
        let options = MaterializeOptions { strict_lossless: false, force_overwrite: true, ..Default::default() };
        extract_archive(reader, destination, &options)?;
        let data = destination.join("source/data");
        let hardlink = destination.join("source/hardlink");
        ensure!(std::fs::read(&data)? == b"linked payload");
        ensure!(std::fs::read(&hardlink)? == b"linked payload");
        ensure!(std::fs::metadata(&data)?.ino() == std::fs::metadata(&hardlink)?.ino());
        ensure!(std::fs::read_link(destination.join("source/symlink"))? == Path::new("data"));
        ensure!(xattr::get(&data, "user.ctb.archive")? == Some(b"stream payload".to_vec()));
        ensure!(xattr::get(hardlink, "user.ctb.archive")? == Some(b"stream payload".to_vec()));
        Ok(())
    }
}