// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Command-line interface definitions and dispatch for archive operations.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use crate::journal::{JournalReader, JournalWriter};
use crate::journal::{JournalErrorRecord, JournalErrorStage, JournalSessionRecord, JournalWarningRecord, current_platform};
use crate::journal::capture::{CapturedSource, CapturedTree, CommittedHardlinks, attach_environment};
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
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Parser)]
pub struct ArchiveArgs {
    #[command(subcommand)]
    pub command: ArchiveCommand,
}

#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
pub struct ArchiveCreateOptions {
    /// Record full provenance environment information using slow environment detection
    /// (e.g. network discovery for public IP addresses and server time synchronization).
    #[arg(long)]
    pub full_provenance: bool,
    /// Behavior when a source file is modified during capture.
    #[arg(long, value_enum, default_value_t = crate::args::SourceChangePolicy::Error)]
    pub on_source_change: crate::args::SourceChangePolicy,
    /// Stay on the current filesystem and do not cross mount boundaries.
    #[arg(short = 'x', long = "one-file-system")]
    pub one_file_system: bool,
    /// Optional AppleDouble, AppleSingle, and Windows ADS companion reading flags.
    #[command(flatten)]
    pub apple_read: crate::args::AppleReadCliArgs,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ArchiveCommand {
    /// Create a self-contained uncompressed CSC archive.
    ///
    /// When writing to `-` (stdout) with shell redirection (for example,
    /// `> archive.cscarchive`), the shell truncates an existing target file
    /// before `ctb` starts; that truncation cannot be undone by the program.
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
        /// Write as journal format (different magic bytes, omits stream or file bodies).
        #[arg(long)]
        journal: bool,
        #[command(flatten)]
        options: ArchiveCreateOptions,
    },
    /// List committed members and capture diagnostics.
    List { archive: PathBuf },
    /// Verify archive structure, checksums, and file contents without sources.
    Verify {
        archive: PathBuf,
        /// Allow verification of incomplete archives or archives containing capture errors.
        #[arg(long, alias = "salvage", alias = "allow-archive-errors", alias = "allow-journal-errors")]
        recover: bool,
    },
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
        /// Require every timestamp check, including hard platform limits.
        #[arg(long)]
        strict: bool,
        /// Verify restored access times.
        #[arg(long)]
        check_atime: bool,
        /// Verify change times even where the platform cannot restore them.
        #[arg(long)]
        check_ctime: bool,
        /// Verify birth times even where the platform cannot restore them.
        #[arg(long)]
        check_birthtime: bool,
        /// Permit restoration of FIFOs and device nodes as special files.
        #[arg(long)]
        copy_specials_as_specials: bool,
        /// Extract committed members from incomplete archives or archives containing capture errors.
        #[arg(long, alias = "salvage", alias = "allow-archive-errors", alias = "allow-journal-errors")]
        recover: bool,
        /// Optional AppleDouble, AppleSingle, and Windows ADS stream restoration flags.
        #[command(flatten)]
        apple_write: crate::args::AppleWriteCliArgs,
    },
}

pub fn run_archive(args: &ArchiveArgs) -> Result<ToolResult> {
    run_archive_with_stdout(args, None)
}

pub(crate) fn run_archive_with_stdout(
    args: &ArchiveArgs,
    stdout_override: Option<File>,
) -> Result<ToolResult> {
    match &args.command {
        ArchiveCommand::Create { archive, sources, resume, continue_on_error, journal, options: create_options } => {
            let _scope = if create_options.full_provenance {
                ctb_io_environment::GlobalEnvironmentScope::enter_full()
            } else {
                ctb_io_environment::GlobalEnvironmentScope::enter_fresh()
            };
            if *resume {
                ensure!(!continue_on_error, "Cannot combine --resume with --continue-on-error; capture diagnostics cannot be resolved by resume");
                ensure!(archive != Path::new("-"), "Cannot resume archive output on stdout");
                resume_archive_with_options(archive, *journal, create_options)?;
                return Ok(ToolResult::Immediate {
                    stdout: Vec::new(), stderr: b"Archive resumed and completed.\n".to_vec(), exit_code: 0,
                });
            }
            let stdout_handle = if archive == Path::new("-") {
                if let Some(ref file) = stdout_override {
                    Some(file.try_clone()?)
                } else {
                    clone_stdout_file()?
                }
            } else {
                None
            };
            if let Some(ref output) = stdout_handle {
                validate_output_identity(output, sources, &[])?;
            }
            let plan = plan_sources_with_options(sources, archive, create_options)?;
            let (error_count, warning_count, completed) = if archive == Path::new("-") {
                if let Some(ref output) = stdout_handle {
                    validate_output_identity(output, sources, &plan.members)?;
                }
                if let Some(file) = stdout_override {
                    let outcome = create_archive_with_options(
                        ForwardFileOutput(file),
                        sources,
                        archive,
                        &plan,
                        *continue_on_error,
                        None,
                        *journal,
                        create_options,
                    )?;
                    (outcome.error_count, outcome.warning_count, outcome.completed)
                } else {
                    let outcome = create_archive_with_options(
                        std::io::stdout().lock(),
                        sources,
                        archive,
                        &plan,
                        *continue_on_error,
                        None,
                        *journal,
                        create_options,
                    )?;
                    (outcome.error_count, outcome.warning_count, outcome.completed)
                }
            } else {
                let mut options = OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                let file = options.open(archive).with_context(|| format!("Cannot create {}", archive.display()))?;
                if let Err(error) = validate_output_identity(&file, sources, &plan.members) {
                    drop(file);
                    let _ = std::fs::remove_file(archive);
                    return Err(error);
                }
                let outcome = create_archive_with_options(
                    FileArchiveOutput::new(file),
                    sources,
                    archive,
                    &plan,
                    *continue_on_error,
                    None,
                    *journal,
                    create_options,
                )?;
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
            for member in &reader.members {
                if let Err(error) = crate::journal::stream::ensure_clean_metadata(&member.entity) {
                    writeln!(diagnostics, "Member diagnostics {}: {error:#}", member.entity.identity.display_path())?;
                }
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
        ArchiveCommand::Verify { archive, recover } => {
            let reader = if *recover {
                JournalReader::read(File::open(archive)?, crate::journal::ReadMode::Recover, &ctb_formats_dcstring::DcProjectionPolicy::forgiving())?
            } else {
                JournalReader::new(File::open(archive)?)?
            };
            if *recover {
                reader.verify_recover()?;
            } else {
                reader.verify_self_contained()?;
            }
            validate_members(&reader)?;
            let mut diagnostics = String::new();
            use std::fmt::Write;
            let member_diagnostics = append_reader_diagnostics(&reader, &mut diagnostics)?;
            if *recover && (!reader.is_completed() || reader.has_diagnostics() || member_diagnostics > 0) {
                writeln!(
                    diagnostics,
                    "Archive verified in recover mode (incomplete: {}; errors: {}; warnings: {}; member_diagnostics: {}). Archive is not a clean capture.",
                    !reader.is_completed(),
                    reader.errors.len(),
                    reader.warnings.len(),
                    member_diagnostics
                )?;
                Ok(ToolResult::Immediate {
                    stdout: b"Archive verified (with diagnostics).\n".to_vec(),
                    stderr: diagnostics.into_bytes(),
                    exit_code: 1,
                })
            } else {
                Ok(ToolResult::immediate_ok(b"Archive verified.\n".to_vec()))
            }
        }
        ArchiveCommand::Extract { archive, destination, overwrite, trust_symlink_targets,
            best_effort_metadata, strict, check_atime, check_ctime, check_birthtime, copy_specials_as_specials, recover, apple_write } => {
            let reader = if *recover {
                JournalReader::read(File::open(archive)?, crate::journal::ReadMode::Recover, &ctb_formats_dcstring::DcProjectionPolicy::forgiving())?
            } else if *best_effort_metadata && !*strict {
                JournalReader::read(File::open(archive)?, crate::journal::ReadMode::Complete, &ctb_formats_dcstring::DcProjectionPolicy::forgiving())?
            } else {
                JournalReader::new(File::open(archive)?)?
            };
            let (apple_write_mode, apple_single_write_extension) =
                apple_write.resolve(*best_effort_metadata && !*strict)?;
            let options = MaterializeOptions {
                strict_lossless: *strict || !best_effort_metadata,
                check_atime: *strict || *check_atime,
                check_ctime: *strict || *check_ctime || cfg!(windows),
                check_birthtime: *strict || *check_birthtime || cfg!(windows),
                existing_entry_policy: if *overwrite { ctb_io::file::ExistingEntryPolicy::Replace }
                    else { ctb_io::file::ExistingEntryPolicy::Refuse },
                force_overwrite: true,
                symlink_policy: if *trust_symlink_targets { SymlinkValidationPolicy::PreserveVerbatim }
                    else { SymlinkValidationPolicy::RejectEscapingSymlinks },
                path_policy: PathTraversalPolicy::StrictSandboxed,
                copy_specials: *copy_specials_as_specials,
                apple_write_mode,
                apple_single_write_extension,
                ..Default::default()
            };
            let extraction_losses = extract_archive(&reader, destination, &options, *recover)?;
            let mut diagnostics = String::new();
            use std::fmt::Write;
            let member_diagnostics = append_reader_diagnostics(&reader, &mut diagnostics)?;
            for loss in &extraction_losses {
                writeln!(diagnostics, "Extraction loss: {loss}")?;
            }
            if *recover && (!reader.is_completed() || reader.has_diagnostics() || member_diagnostics > 0 || !extraction_losses.is_empty()) {
                writeln!(
                    diagnostics,
                    "Archive extracted in recover mode (incomplete: {}; errors: {}; warnings: {}; member_diagnostics: {}; extraction_losses: {}). Archive is not a clean capture.",
                    !reader.is_completed(),
                    reader.errors.len(),
                    reader.warnings.len(),
                    member_diagnostics,
                    extraction_losses.len()
                )?;
                Ok(ToolResult::Immediate {
                    stdout: b"Archive extracted (with diagnostics).\n".to_vec(),
                    stderr: diagnostics.into_bytes(),
                    exit_code: 1,
                })
            } else {
                Ok(ToolResult::immediate_ok(b"Archive extracted.\n".to_vec()))
            }
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
        read_windows_ads_alongside: false,
    }
}

fn validate_output_path_overlap(output_path: &Path, sources: &[PathBuf]) -> Result<()> {
    let output = resolve_existing_ancestors(output_path)?;
    let working_directory = std::env::current_dir()?;
    for source in sources {
        let absolute = working_directory.join(source);
        let resolved = resolve_existing_ancestors(&absolute)?;
        ensure!(
            output != resolved && !output.starts_with(&resolved),
            "Archive output overlaps a source: {}",
            source.display()
        );
    }
    Ok(())
}

fn plan_sources(sources: &[PathBuf], archive: &Path) -> Result<SourcePlan> {
    plan_sources_with_options(sources, archive, &ArchiveCreateOptions::default())
}

fn plan_sources_with_options(
    sources: &[PathBuf],
    archive: &Path,
    options: &ArchiveCreateOptions,
) -> Result<SourcePlan> {
    ensure!(!sources.is_empty(), "At least one source is required");
    if archive != Path::new("-") {
        validate_output_path_overlap(archive, sources)?;
    }
    let working_directory = std::env::current_dir()?;
    let apple_read_options = options.apple_read.resolve();
    let best_effort_source_change =
        options.on_source_change == crate::args::SourceChangePolicy::BestEffort;
    let mut plan = SourcePlan::default();
    let mut names = HashSet::new();
    for source in sources {
        let absolute = working_directory.join(source);
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
            let traversal_options = TraversalOptions::new()
                .error_policy(ctb_io::file::OnTraversalError::Capture)
                .one_file_system(options.one_file_system)
                .apple_read_options(apple_read_options.clone());
            let mut directories = ctb_io::file::traverse_dir(&absolute, traversal_options)?.directories()?;
            while let Some(directory) = directories.next() {
                let directory = directory?;
                if !directory.errors.is_empty() {
                    let error = anyhow::anyhow!("Source traversal errors: {}", directory.errors.join("; "));
                    plan.diagnostics.push(JournalErrorRecord::from_error(&directory.path, JournalErrorStage::Traversal, &error));
                    plan.members.retain(|member| member.source != directory.path);
                    continue;
                }
                let snapshot = ctb_io::file::payload::SourceSnapshot::capture(&directory.path, best_effort_source_change)?;
                plan.tree.retain(&snapshot);
                let entries = match directories.read_directory(&directory, Ok) {
                    Ok(entries) => entries,
                    Err(error) => {
                        plan.diagnostics.push(JournalErrorRecord::from_error(&directory.path, JournalErrorStage::Traversal, &error));
                        plan.members.retain(|member| member.source != directory.path);
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
                    plan.members.push(SourceMember { source: item.path, relative });
                }
                snapshot.retain_changes(snapshot.validate()?)?;
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

fn clone_stdout_file() -> Result<Option<File>> {
    #[cfg(unix)]
    {
        use std::os::fd::AsFd;
        Ok(Some(File::from(
            std::io::stdout().as_fd().try_clone_to_owned()?,
        )))
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsHandle;
        Ok(Some(File::from(
            std::io::stdout().as_handle().try_clone_to_owned()?,
        )))
    }
    #[cfg(not(any(unix, windows)))]
    {
        Ok(None)
    }
}

fn ensure_path_not_same_output(path: &Path, output_key: &ctb_io::file::InodeKey) -> Result<()> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let source_key = ctb_io::file::extract_file_identity(path, &metadata)?;
    ensure!(
        &source_key != output_key,
        "Archive output is included in the source tree: {}",
        path.display()
    );
    if metadata.file_type().is_symlink() {
        if let Ok(resolved_target) = std::fs::canonicalize(path) {
            if let Ok(target_key) = ctb_io::file::query_file_identity(&resolved_target) {
                ensure!(
                    &target_key != output_key,
                    "Archive output is included in the source tree: {}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn validate_output_identity(
    output: &File,
    sources: &[PathBuf],
    plan: &[SourceMember],
) -> Result<()> {
    if let Some(output_path) = ctb_io::file::try_query_open_file_path(output) {
        validate_output_path_overlap(&output_path, sources)?;
    }
    if let Some(output_key) = ctb_io::file::try_query_open_file_identity(output)? {
        let working_directory = std::env::current_dir()?;
        for source in sources {
            let absolute = working_directory.join(source);
            ensure_path_not_same_output(&absolute, &output_key)?;
        }
        for member in plan {
            ensure_path_not_same_output(&member.source, &output_key)?;
        }
    }
    Ok(())
}

struct ForwardFileOutput(File);

impl Write for ForwardFileOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.write(bytes)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl ArchiveOutput for ForwardFileOutput {}

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

#[derive(Default)]
struct FilesystemCapture {
    apple_read_options: AppleReadOptions,
    best_effort_source_change: bool,
}

impl FilesystemCapture {
    fn from_options(options: &ArchiveCreateOptions) -> Self {
        Self {
            apple_read_options: options.apple_read.resolve(),
            best_effort_source_change: options.on_source_change
                == crate::args::SourceChangePolicy::BestEffort,
        }
    }
}

impl CaptureSource for FilesystemCapture {
    fn capture(&mut self, path: &Path) -> Result<CapturedSource> {
        let mut source = CapturedSource::capture(
            path,
            None,
            &self.apple_read_options,
            self.best_effort_source_change,
        )?;
        source.hash(path)?;
        Ok(source)
    }
}

struct CaptureRequest<'a> {
    sources: &'a [PathBuf],
    archive: &'a Path,
    plan: &'a SourcePlan,
    continue_on_error: bool,
    recovered: Option<&'a JournalReader>,
    journal: bool,
    options: &'a ArchiveCreateOptions,
}

#[cfg(test)]
fn create_archive<W: ArchiveOutput>(output: W, sources: &[PathBuf], archive: &Path, plan: &SourcePlan,
    continue_on_error: bool, recovered: Option<&JournalReader>, journal: bool) -> Result<CaptureOutcome<W>> {
    create_archive_with_options(
        output,
        sources,
        archive,
        plan,
        continue_on_error,
        recovered,
        journal,
        &ArchiveCreateOptions::default(),
    )
}

fn create_archive_with_options<W: ArchiveOutput>(
    output: W,
    sources: &[PathBuf],
    archive: &Path,
    plan: &SourcePlan,
    continue_on_error: bool,
    recovered: Option<&JournalReader>,
    journal: bool,
    options: &ArchiveCreateOptions,
) -> Result<CaptureOutcome<W>> {
    let mut capture = FilesystemCapture::from_options(options);
    capture_archive(
        output,
        CaptureRequest {
            sources,
            archive,
            plan,
            continue_on_error,
            recovered,
            journal,
            options,
        },
        &mut capture,
    )
}

fn persist_source_error<W: ArchiveOutput>(writer: &mut JournalWriter<W>, record: &JournalErrorRecord) -> Result<()> {
    writer.record_error(record).with_context(|| format!("Cannot persist source failure: {}", record.error_message))?;
    writer.output_mut().flush().with_context(|| format!("Cannot flush source failure: {}", record.error_message))?;
    writer.output_mut().sync().with_context(|| format!("Cannot sync source failure: {}", record.error_message))
}

fn persist_warning_record<W: ArchiveOutput>(
    writer: &mut JournalWriter<W>,
    code: &str,
    message: String,
    count: &mut usize,
) -> Result<()> {
    let warning = JournalWarningRecord {
        code: code.to_owned(),
        message,
        timestamp_sec: i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        )?,
    };
    writer
        .record_warning(&warning)
        .with_context(|| format!("Cannot persist metadata warning: {}", warning.message))?;
    writer
        .output_mut()
        .flush()
        .with_context(|| format!("Cannot flush metadata warning: {}", warning.message))?;
    writer
        .output_mut()
        .sync()
        .with_context(|| format!("Cannot sync metadata warning: {}", warning.message))?;
    *count = count.saturating_add(1);
    Ok(())
}

fn persist_metadata_warnings<W: ArchiveOutput>(writer: &mut JournalWriter<W>, entity: &FileEntity,
    path: &Path, count: &mut usize) -> Result<()> {
    crate::journal::capture::visit_warnings(entity, &mut |_entity, message| {
        persist_warning_record(
            writer,
            "FileReadWarning",
            format!("{}: {message}", path.display()),
            count,
        )
    })
}

fn capture_archive<W: ArchiveOutput>(output: W, request: CaptureRequest<'_>, capture: &mut impl CaptureSource) -> Result<CaptureOutcome<W>> {
    let CaptureRequest { sources, archive, plan, continue_on_error, recovered, journal, options } = request;
    let best_effort_source_change =
        options.on_source_change == crate::args::SourceChangePolicy::BestEffort;
    let apple_read_options = options.apple_read.resolve();
    let session = if let Some(recovered) = recovered {
        recovered.session.clone()
    } else {
        let working_directory = std::env::current_dir()?;
        let environment = if options.full_provenance {
            std::sync::Arc::new(ctb_io_environment::capture())
        } else {
            ctb_io_environment::capture_quick_arc()
        };
        JournalSessionRecord {
            sources: sources.iter().map(|source| working_directory.join(source)).collect(),
            destination: archive.to_path_buf(), origin_platform: u32::from(current_platform()),
            environment: Some(environment),
        }
    };
    let magic = if journal {
        crate::journal::stream::JournalMagic::JournalV5
    } else {
        crate::journal::stream::JournalMagic::ArchiveV2
    };
    let mut writer = if let Some(recovered) = recovered { recovered.resume_writer(output)? }
        else { JournalWriter::new(output, &session, magic)? };
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
        let mut effective_relative = member.relative.clone();
        if let Some(name) = member.relative.file_name() {
            for (enabled, extension) in [
                (apple_read_options.read_apple_single_as, ctb_io::file::apple_double::AppleSingleExtension::As),
                (apple_read_options.read_apple_single_asf, ctb_io::file::apple_double::AppleSingleExtension::Asf),
            ] {
                if enabled {
                    if let Some(stem) = extension.strip_from_name(name) {
                        if entity.identity.filename_bytes() == Some(stem.as_encoded_bytes()) {
                            effective_relative = member.relative.with_file_name(stem);
                            break;
                        }
                    }
                }
            }
        }
        crate::journal::capture::finalize_warnings(&snapshot, &mut entity, &mut |_, _| Ok(()))?;
        let change_warnings = if best_effort_source_change {
            snapshot.change_warnings()?
        } else {
            Vec::new()
        };
        if !change_warnings.is_empty() {
            entity
                .metadata
                .warnings
                .retain(|w| !change_warnings.contains(w));
        }
        tree.visit_pending_warnings(&mut |code, message| {
            persist_warning_record(&mut writer, code, message.to_owned(), &mut warning_count)
        })?;
        entity.identity.relative_path = Some(effective_relative.clone());
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
            match hardlinks.lookup(&entity, &effective_relative) {
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
        let mut payload = if matches!(entity.kind, FileEntityKind::Regular { .. }) && !journal {
            let opened: Result<Box<dyn PayloadSource>> = match snapshot.interpreted_payload() {
                Ok(Some(interpreted)) => {
                    let mut mem = ctb_io::file::MemoryPayloadSource::new(interpreted)?;
                    mem.bind_snapshot(&snapshot)?;
                    Ok(Box::new(mem))
                }
                Ok(None) => capture.open(&member.source).and_then(|mut payload| {
                    payload.bind_snapshot(&snapshot)?;
                    Ok(payload)
                }),
                Err(err) => Err(err),
            };
            match opened {
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
            snapshot.retain_changes(snapshot.validate()?)?;
            Ok(())
        };
        let bodies = if journal { ctb_io::file::encoding::BodyPolicy::Omit } else { ctb_io::file::encoding::BodyPolicy::Include };
        let result = if let Some(payload) = &mut payload {
            writer.write_entity_checked(&entity, Some(payload.as_mut()), bodies, precommit)
        } else {
            writer.write_entity_checked(&entity, None, bodies, precommit)
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
            hardlinks.commit(&entity, &effective_relative, Some(&snapshot));
        }
    }
    if let Err(error) = tree.validate() {
        persist_source_error(&mut writer, &JournalErrorRecord::from_error(archive, JournalErrorStage::PostCheck, &error))?;
        error_count = error_count.saturating_add(1);
        if !continue_on_error {
            return Ok(CaptureOutcome { output: writer.into_output(), error_count, warning_count, completed: false });
        }
    }
    tree.visit_pending_warnings(&mut |code, message| {
        persist_warning_record(&mut writer, code, message.to_owned(), &mut warning_count)
    })?;
    let mut output = writer.finish()?;
    output.sync()?;
    Ok(CaptureOutcome { output, error_count, warning_count, completed: true })
}

#[cfg(test)]
fn resume_archive(path: &Path, journal: bool) -> Result<()> {
    resume_archive_with_options(path, journal, &ArchiveCreateOptions::default())
}

fn resume_archive_with_options(
    path: &Path,
    journal: bool,
    options: &ArchiveCreateOptions,
) -> Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let mut magic = [0_u8; 8];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut magic)?;
    let is_journal = &magic == crate::journal::JOURNAL_MAGIC_V5
        || &magic == crate::journal::JOURNAL_MAGIC_V4
        || &magic == crate::journal::JOURNAL_MAGIC_V1
        || journal;
    let recovered = JournalReader::recover(file.try_clone()?)?;
    ensure!(
        !recovered.is_legacy,
        "Cannot resume a legacy CSC archive or journal"
    );
    ensure!(!recovered.completed, "Cannot resume a completed archive");
    if is_journal {
        recovered.verify_contents()?;
    } else {
        recovered.verify_self_contained_prefix()?;
    }
    validate_members(&recovered)?;
    let plan = plan_sources_with_options(&recovered.session.sources, path, options)?;
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
        ctb_io::file::verify_materialized_entity_ext(source, &expected, true, false, false, cfg!(windows))
            .with_context(|| format!("Committed source changed: {}", source.display()))?;
    }
    file.set_len(recovered.valid_length)?;
    file.sync_all()?;
    file.seek(SeekFrom::End(0))?;
    let outcome = create_archive_with_options(
        FileArchiveOutput::new(file),
        &recovered.session.sources,
        path,
        &plan,
        false,
        Some(&recovered),
        is_journal,
        options,
    )?;
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

fn append_reader_diagnostics(reader: &JournalReader, diagnostics: &mut String) -> Result<usize> {
    use std::fmt::Write;
    let mut member_diagnostics = 0_usize;
    for member in &reader.members {
        if let Err(error) = crate::journal::stream::ensure_clean_metadata(&member.entity) {
            writeln!(
                diagnostics,
                "Member diagnostics {}: {error:#}",
                member.entity.identity.display_path()
            )?;
            member_diagnostics = member_diagnostics.saturating_add(1);
        }
    }
    for error in &reader.errors {
        writeln!(
            diagnostics,
            "Capture error ({:?}) {}: {}",
            error.stage,
            error.path.display(),
            error.error_message
        )?;
    }
    for warning in &reader.warnings {
        writeln!(
            diagnostics,
            "Capture warning ({}): {}",
            warning.code, warning.message
        )?;
    }
    Ok(member_diagnostics)
}

fn extract_archive(reader: &JournalReader, destination: &Path, options: &MaterializeOptions, recover: bool) -> Result<Vec<String>> {
    ensure!(cfg!(any(unix, windows)), "Sandboxed archive extraction is unsupported on this platform");
    if !recover {
        reader.verify_self_contained()?;
    }
    validate_members(reader)?;
    let mut losses = Vec::new();
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
            FileEntityKind::Socket | FileEntityKind::Door | FileEntityKind::Inaccessible => {
                if recover {
                    let msg = format!("Archive member cannot be restored: {}; skipping in recover mode", relative.display());
                    warn_fmt!("{msg}");
                    losses.push(msg);
                    continue;
                }
                bail!("Archive member cannot be restored: {}", relative.display());
            }
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
        let orig_relative = member.entity.identity.relative_path.as_deref().context("Missing directory path")?;
        let adapted_buf;
        let relative = if options.strict_lossless || orig_relative.as_os_str().is_empty() || orig_relative == Path::new(".") {
            orig_relative
        } else {
            let (adapted, _) = ctb_io::file::adapt_relative_path_for_materialization(orig_relative, cfg!(windows));
            adapted_buf = adapted;
            &adapted_buf
        };
        sandbox.ensure_dir_all(relative, PathTraversalPolicy::StrictSandboxed)?;
        if let Err(error) = sandbox.authorize_metadata(relative, options.existing_entry_policy) {
            if recover {
                let msg = format!("Failed to authorize directory metadata for {}: {error:#}; skipping in recover mode", relative.display());
                warn_fmt!("{msg}");
                losses.push(msg);
            } else {
                return Err(error);
            }
        }
    }
    for hardlinks in [false, true] {
        for member in reader.members.iter().filter(|member| !member.entity.is_dir()
            && matches!(member.entity.kind, FileEntityKind::Hardlink { .. }) == hardlinks) {
            // Reason for fallback: member without explicit relative path defaults to empty path.
            let relative = member.entity.identity.relative_path.as_deref().unwrap_or(Path::new(""));
            if matches!(member.entity.kind, FileEntityKind::Socket | FileEntityKind::Door | FileEntityKind::Inaccessible) {
                continue;
            }
            let materialize_res = if matches!(member.entity.kind, FileEntityKind::Regular { .. }) {
                match reader.payload(member) {
                    Ok(mut payload) => ctb_io::file::materialize_entity_with_stream_source(&member.entity, Some(&mut payload), &sandbox, options, reader),
                    Err(err) => Err(err),
                }
            } else {
                ctb_io::file::materialize_entity_with_stream_source(&member.entity, None, &sandbox, options, reader)
            };
            let receipt = match materialize_res {
                Ok(receipt) => receipt,
                Err(error) if recover => {
                    let msg = format!("Failed to extract member {}: {error:#}; skipping in recover mode", relative.display());
                    warn_fmt!("{msg}");
                    losses.push(msg);
                    continue;
                }
                Err(error) => return Err(error),
            };
            for warning in &receipt.warnings { warn_fmt!("{warning}"); }
            if !hardlinks {
                let mut expected = member.entity.clone();
                receipt.apply_stream_adaptations(&mut expected);
                if let Err(error) = ctb_io::file::verify_materialized_entity_ext(&receipt.destination_path, &expected, options.strict_lossless, options.check_atime, options.check_ctime, options.check_birthtime) {
                    if recover {
                        let msg = format!("Materialized member verification failed for {}: {error:#}; continuing in recover mode", relative.display());
                        warn_fmt!("{msg}");
                        losses.push(msg);
                    } else {
                        return Err(error);
                    }
                }
            }
        }
    }
    let mut directories: Vec<_> = reader.members.iter().filter(|member| member.entity.is_dir()).collect();
    directories.sort_by_key(|member| std::cmp::Reverse(member.entity.identity.relative_path.as_deref().map(Path::components).map(Iterator::count)));
    for member in directories {
        let relative = member.entity.identity.relative_path.as_deref().context("Missing directory path")?;
        if relative == Path::new(".") {
            if let Err(error) = sandbox.authorize_metadata(relative, options.existing_entry_policy) {
                if recover {
                    let msg = format!("Failed to authorize root directory metadata: {error:#}; skipping in recover mode");
                    warn_fmt!("{msg}");
                    losses.push(msg);
                    continue;
                } else {
                    return Err(error);
                }
            }
            let mut expected = member.entity.clone();
            match ctb_io::file::materialize_directory_streams_and_companions(
                &member.entity,
                destination,
                &sandbox,
                options,
                reader,
            ) {
                Ok((stream_adaptations, warnings)) => {
                    ctb_io::file::MaterializeReceipt {
                        destination_path: destination.to_path_buf(),
                        bytes_written: 0,
                        sha256: None,
                        skipped_identical: false,
                        warnings: Vec::new(),
                        stream_adaptations,
                    }
                    .apply_stream_adaptations(&mut expected);
                    for warning in warnings {
                        warn_fmt!("{warning}");
                    }
                }
                Err(error) if recover => {
                    let msg = format!("Failed to materialize root directory streams: {error:#}; skipping in recover mode");
                    warn_fmt!("{msg}");
                    losses.push(msg);
                }
                Err(error) => return Err(error),
            }
            match ctb_io::file::apply_entity_metadata(destination, None, &member.entity.metadata, false, true, options.strict_lossless, options.check_birthtime) {
                Ok(warnings) => {
                    for warning in warnings { warn_fmt!("{warning}"); }
                }
                Err(error) if recover => {
                    let msg = format!("Failed to apply root directory metadata: {error:#}; skipping in recover mode");
                    warn_fmt!("{msg}");
                    losses.push(msg);
                }
                Err(error) => return Err(error),
            }
            if let Err(error) = ctb_io::file::verify_materialized_entity_ext(
                destination, &expected, options.strict_lossless,
                options.check_atime, options.check_ctime, options.check_birthtime,
            ) {
                if recover {
                    let msg = format!("Directory verification failed for {}: {error:#}; continuing in recover mode", relative.display());
                    warn_fmt!("{msg}");
                    losses.push(msg);
                } else {
                    return Err(error);
                }
            }
        } else {
            let mut verify_path = destination.join(relative);
            let mut expected = member.entity.clone();
            match ctb_io::file::materialize_entity_with_stream_source(&member.entity, None, &sandbox, options, reader) {
                Ok(receipt) => {
                    receipt.apply_stream_adaptations(&mut expected);
                    verify_path = receipt.destination_path;
                    for warning in receipt.warnings { warn_fmt!("{warning}"); }
                }
                Err(error) if recover => {
                    let msg = format!("Failed to materialize directory metadata for {}: {error:#}; skipping in recover mode", relative.display());
                    warn_fmt!("{msg}");
                    losses.push(msg);
                }
                Err(error) => return Err(error),
            }
            if let Err(error) = ctb_io::file::verify_materialized_entity_ext(
                &verify_path, &expected, options.strict_lossless,
                options.check_atime, options.check_ctime, options.check_birthtime,
            ) {
                if recover {
                    let msg = format!("Directory verification failed for {}: {error:#}; continuing in recover mode", relative.display());
                    warn_fmt!("{msg}");
                    losses.push(msg);
                } else {
                    return Err(error);
                }
            }
        }
    }
    Ok(losses)
}

fn existing_metadata(path: &Path) -> Result<Option<std::fs::Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("Cannot inspect {}", path.display())),
    }
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;
    use ctb_formats_dcstring::DcMstring;
    use std::io::Read;
    use crate::verifier::run_csc_verify;
    use crate::args::default_verify_args;

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
            let mut source = FilesystemCapture::default().capture(path)?;
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

    static DEFAULT_CREATE_OPTIONS: ArchiveCreateOptions = ArchiveCreateOptions {
        full_provenance: false,
        on_source_change: crate::args::SourceChangePolicy::Error,
        one_file_system: false,
        apple_read: crate::args::AppleReadCliArgs {
            read_apple_double: None,
            read_apple_double_alongside: false,
            no_read_apple_double_alongside: false,
            read_apple_double_zip: false,
            read_apple_double_netatalk: false,
            read_apple_single: false,
            read_apple_single_without_extension: false,
            no_read_apple_single: false,
            read_apple_single_with_extension: Vec::new(),
            read_windows_ads: false,
            no_read_windows_ads: false,
        },
    };

    fn capture_request<'a>(sources: &'a [PathBuf], archive: &'a Path, plan: &'a SourcePlan,
        continue_on_error: bool) -> CaptureRequest<'a> {
        CaptureRequest { sources, archive, plan, continue_on_error, recovered: None, journal: false, options: &DEFAULT_CREATE_OPTIONS }
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
        let outcome = create_archive(FileArchiveOutput::new(File::create(&archive)?), &sources, &archive, &plan, false, None, false)?;
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
        extract_archive(&reader, &destination, &MaterializeOptions::default(), false)?;
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
        let raw = FilesystemCapture::default().capture(&path)?;
        let read_alongside = AppleReadOptions {
            read_apple_double_alongside: true,
            ..Default::default()
        };
        ensure!(CapturedSource::capture(&path, None, &read_alongside, false).is_err());
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
            FilesystemCapture::default().capture(&path)?;
            ensure!(CapturedSource::capture(&path, None, &read_alongside, false).is_err());
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
            let outcome = create_archive(output, &sources, &archive, &plan, continue_on_error, None, false)?;
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
                let source = FilesystemCapture::default().capture(path)?;
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
                FilesystemCapture::default().capture(path)
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
                archive: path.clone(), sources: vec![missing.clone(), good.clone()], resume: false, continue_on_error, journal: false,
                options: ArchiveCreateOptions::default(),
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
            ensure!(resume_archive(&path, false).is_err());
            ensure!(std::fs::read(&path)? == before);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_safety_failures_never_continue() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        let source_file = source.join("data.txt");
        std::fs::write(&source_file, b"important source payload")?;
        for sources in [vec![source.clone()], vec![source.clone(), source.clone()]] {
            let archive = if sources.len() == 1 { source.join("overlap.cscarchive") }
                else { temporary.path().join("duplicate.cscarchive") };
            ensure!(run_archive(&ArchiveArgs { command: ArchiveCommand::Create {
                archive: archive.clone(), sources, resume: false, continue_on_error: true, journal: false,
                options: ArchiveCreateOptions::default(),
            } }).is_err());
            ensure!(!archive.exists());
        }

        // Redirected stdout (`-`) overlapping source file, file inside source directory,
        // hardlink to source file, or symlink source pointing to redirected output must fail
        // before emitting a single archive byte.
        let nested_dir = source.join("nested");
        std::fs::create_dir(&nested_dir)?;
        let nested_out = nested_dir.join("redirected.cscarchive");
        std::fs::write(&nested_out, b"")?;

        let hardlink_out = temporary.path().join("hardlink_out.cscarchive");
        std::fs::hard_link(&source_file, &hardlink_out)?;

        let symlink_target_out = temporary.path().join("symlink_target_out.cscarchive");
        std::fs::write(&symlink_target_out, b"untouched output")?;
        let symlink_source = temporary.path().join("symlink_source");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&symlink_target_out, &symlink_source)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&symlink_target_out, &symlink_source)?;

        for continue_on_error in [false, true] {
            // 1. Redirected stdout is the source file itself (opened without truncation here to
            //    verify zero bytes are written by ctb; note shell `>` truncates before exec).
            let out_handle = OpenOptions::new().write(true).open(&source_file)?;
            ensure!(run_archive_with_stdout(&ArchiveArgs {
                command: ArchiveCommand::Create {
                    archive: PathBuf::from("-"),
                    sources: vec![source_file.clone()],
                    resume: false,
                    continue_on_error,
                    journal: false,
                    options: ArchiveCreateOptions::default(),
                },
            }, Some(out_handle)).is_err());
            ensure!(std::fs::read(&source_file)? == b"important source payload");

            // 2. Redirected stdout is inside a source directory.
            let out_handle = OpenOptions::new().write(true).open(&nested_out)?;
            ensure!(run_archive_with_stdout(&ArchiveArgs {
                command: ArchiveCommand::Create {
                    archive: PathBuf::from("-"),
                    sources: vec![source.clone()],
                    resume: false,
                    continue_on_error,
                    journal: false,
                    options: ArchiveCreateOptions::default(),
                },
            }, Some(out_handle)).is_err());
            ensure!(std::fs::read(&nested_out)?.is_empty());

            // 3. Redirected stdout is a hardlink to a source file.
            let out_handle = OpenOptions::new().write(true).open(&hardlink_out)?;
            ensure!(run_archive_with_stdout(&ArchiveArgs {
                command: ArchiveCommand::Create {
                    archive: PathBuf::from("-"),
                    sources: vec![source_file.clone()],
                    resume: false,
                    continue_on_error,
                    journal: false,
                    options: ArchiveCreateOptions::default(),
                },
            }, Some(out_handle)).is_err());
            ensure!(std::fs::read(&source_file)? == b"important source payload");

            // 4. Source is a symlink pointing to the redirected stdout file.
            #[cfg(any(unix, windows))]
            {
                let out_handle = OpenOptions::new().write(true).open(&symlink_target_out)?;
                ensure!(run_archive_with_stdout(&ArchiveArgs {
                    command: ArchiveCommand::Create {
                        archive: PathBuf::from("-"),
                        sources: vec![symlink_source.clone()],
                        resume: false,
                        continue_on_error,
                        journal: false,
                        options: ArchiveCreateOptions::default(),
                    },
                }, Some(out_handle)).is_err());
                ensure!(std::fs::read(&symlink_target_out)? == b"untouched output");
            }
        }

        // Valid redirected stdout to an external file succeeds and produces a verifiable archive.
        let valid_stdout_archive = temporary.path().join("valid_stdout.cscarchive");
        let valid_handle = File::create(&valid_stdout_archive)?;
        let res = run_archive_with_stdout(&ArchiveArgs {
            command: ArchiveCommand::Create {
                archive: PathBuf::from("-"),
                sources: vec![source_file.clone()],
                resume: false,
                continue_on_error: false,
                journal: false,
                options: ArchiveCreateOptions::default(),
            },
        }, Some(valid_handle))?;
        ensure!(matches!(res, ToolResult::Immediate { exit_code: 0, .. }));
        let reader = JournalReader::new(File::open(&valid_stdout_archive)?)?;
        reader.verify_self_contained()?;
        ensure!(reader.members.len() == 1);

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
                ensure!(reader.sequence == if continue_on_error { 2 } else { 1 });
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
        let mut writer = JournalWriter::new(File::create(&archive)?, &session, crate::journal::stream::JournalMagic::ArchiveV2)?;
        let mut committed = FileEntity::from_vec(b"salvaged member payload".to_vec());
        committed.identity.relative_path = Some(PathBuf::from("salvaged.txt"));
        let mut payload = ctb_io::file::MemoryPayloadSource::new(b"salvaged member payload".to_vec())?;
        writer.write_entity(&committed, Some(&mut payload))?;
        let mut unrestorable = FileEntity::from_vec(Vec::new());
        unrestorable.identity.relative_path = Some(PathBuf::from("unrestorable.sock"));
        unrestorable.kind = FileEntityKind::Socket;
        writer.write_entity(&unrestorable, None)?;
        writer.record_error(&crate::journal::JournalErrorRecord {
            path: source.clone(), raw_path: source.as_os_str().as_encoded_bytes().to_vec(),
            stage: crate::journal::JournalErrorStage::MetadataRead,
            error_message: "Injected capture failure".to_owned(), os_error: Some(13), timestamp_sec: 0,
        })?;
        drop(writer.finish()?);
        let listed = run_archive(&ArchiveArgs { command: ArchiveCommand::List { archive: archive.clone() } })?;
        ensure!(matches!(listed, ToolResult::Immediate { exit_code: 1, stderr, .. }
            if String::from_utf8_lossy(&stderr).contains("Injected capture failure")));
        ensure!(run_archive(&ArchiveArgs { command: ArchiveCommand::Verify { archive: archive.clone(), recover: false } }).is_err());
        ensure!(run_archive(&ArchiveArgs { command: ArchiveCommand::Extract {
            archive: archive.clone(), destination: destination.clone(), overwrite: false,
            trust_symlink_targets: false, best_effort_metadata: false, copy_specials_as_specials: false,
            strict: false, check_atime: false, check_ctime: false, check_birthtime: false,
            recover: false, apple_write: Default::default(),
        } }).is_err());
        ensure!(!destination.exists(), "Failed capture modified extraction destination");
        let verify_recover = run_archive(&ArchiveArgs { command: ArchiveCommand::Verify { archive: archive.clone(), recover: true } })?;
        let ToolResult::Immediate { exit_code: verify_code, stderr: verify_stderr, .. } = verify_recover else {
            bail!("Expected Immediate ToolResult from archive verify --recover");
        };
        let verify_diag = String::from_utf8_lossy(&verify_stderr);
        ensure!(verify_code == 1, "Recover verify on diagnostic archive must report non-success partial status");
        ensure!(verify_diag.contains("Injected capture failure") && verify_diag.contains("Archive is not a clean capture."));

        let extract_recover = run_archive(&ArchiveArgs { command: ArchiveCommand::Extract {
            archive, destination: destination.clone(), overwrite: false,
            trust_symlink_targets: false, best_effort_metadata: false, copy_specials_as_specials: false,
            strict: false, check_atime: false, check_ctime: false, check_birthtime: false,
            recover: true, apple_write: Default::default(),
        } })?;
        let ToolResult::Immediate { exit_code: extract_code, stderr: extract_stderr, .. } = extract_recover else {
            bail!("Expected Immediate ToolResult from archive extract --recover");
        };
        let extract_diag = String::from_utf8_lossy(&extract_stderr);
        ensure!(extract_code == 1, "Recover extract on diagnostic archive must report non-success partial status");
        ensure!(extract_diag.contains("Injected capture failure"));
        ensure!(extract_diag.contains("Extraction loss: Archive member cannot be restored: unrestorable.sock"));
        ensure!(extract_diag.contains("extraction_losses: 1"));
        ensure!(std::fs::read(destination.join("salvaged.txt"))? == b"salvaged member payload");
        ensure!(!destination.join("unrestorable.sock").exists());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_stdout_argument_parsing() -> Result<()> {
        let args = ArchiveArgs::try_parse_from(["archive", "create", "-", "source/"])?;
        ensure!(matches!(args.command, ArchiveCommand::Create { archive, sources, resume: false, continue_on_error: false, journal: false, .. }
            if archive == Path::new("-") && sources == vec![PathBuf::from("source/")]));
        ensure!(ArchiveArgs::try_parse_from(["archive", "create", "output", "--resume", "--continue-on-error"]).is_err());
        let stdout_resume = ArchiveArgs::try_parse_from(["archive", "create", "-", "--resume"])?;
        ensure!(run_archive(&stdout_resume).is_err());
        let extract_recover = ArchiveArgs::try_parse_from(["archive", "extract", "test.cscarchive", "dest/", "--recover"])?;
        ensure!(matches!(extract_recover.command, ArchiveCommand::Extract {
            recover: true, best_effort_metadata: false, strict: false, ..
        }));
        let targeted = ArchiveArgs::try_parse_from(["archive", "extract", "test.cscarchive", "dest/", "--check-birthtime"])?;
        ensure!(matches!(targeted.command, ArchiveCommand::Extract {
            check_birthtime: true, check_atime: false, check_ctime: false, best_effort_metadata: false, strict: false, ..
        }));
        let full = ArchiveArgs::try_parse_from(["archive", "extract", "test.cscarchive", "dest/", "--strict", "--best-effort-metadata"])?;
        ensure!(matches!(full.command, ArchiveCommand::Extract { strict: true, best_effort_metadata: true, .. }));
        let extract_salvage = ArchiveArgs::try_parse_from(["archive", "extract", "test.cscarchive", "dest/", "--salvage"])?;
        ensure!(matches!(extract_salvage.command, ArchiveCommand::Extract { recover: true, .. }));
        let extract_errors = ArchiveArgs::try_parse_from(["archive", "extract", "test.cscarchive", "dest/", "--allow-archive-errors"])?;
        ensure!(matches!(extract_errors.command, ArchiveCommand::Extract { recover: true, .. }));
        let extract_jerrors = ArchiveArgs::try_parse_from(["archive", "extract", "test.cscarchive", "dest/", "--allow-journal-errors"])?;
        ensure!(matches!(extract_jerrors.command, ArchiveCommand::Extract { recover: true, .. }));
        let verify_recover = ArchiveArgs::try_parse_from(["archive", "verify", "test.cscarchive", "--recover"])?;
        ensure!(matches!(verify_recover.command, ArchiveCommand::Verify { recover: true, .. }));
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[crate::ctb_test]
    fn archive_directory_timestamp_checks_are_explicit() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        let archive = temporary.path().join("directory.cscarchive");
        let sources = vec![source];
        let plan = plan_sources(&sources, &archive)?;
        let outcome = create_archive(File::create(&archive)?, &sources, &archive, &plan, false, None, false)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        let mut reader = JournalReader::new(File::open(&archive)?)?;
        for member in &mut reader.members {
            let timestamps = member.entity.metadata.timestamps.as_mut().context("Missing directory times")?;
            timestamps.ctime_sec = 1;
            timestamps.ctime_nsec = 0;
        }
        extract_archive(&reader, &temporary.path().join("normal"), &MaterializeOptions::default(), false)?;
        for (name, options, expected) in [
            ("ctime", MaterializeOptions { check_ctime: true, ..Default::default() }, "Ctime mismatch"),
            ("full", MaterializeOptions { check_atime: true, check_ctime: true, check_birthtime: true, ..Default::default() }, "Cannot reproduce birth time"),
        ] {
            let error = extract_archive(&reader, &temporary.path().join(name), &options, false)
                .err().context("Explicit timestamp checks unexpectedly passed")?;
            let formatted = format!("{error:#}");
            ensure!(
                formatted.contains(expected) || formatted.contains("Ctime mismatch"),
                "{error:#}"
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_recover_incomplete_archive() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        std::fs::write(source.join("first"), b"hello first")?;
        let archive_path = temporary.path().join("incomplete.cscarchive");
        let sources = vec![source.clone()];
        let plan = plan_sources(&sources, &archive_path)?;
        let file = File::create(&archive_path)?;
        let outcome = create_archive(FileArchiveOutput::new(file), &sources, &archive_path, &plan, false, None, false)?;
        drop(outcome.output);

        let reader = JournalReader::recover(File::open(&archive_path)?)?;
        ensure!(reader.is_completed());

        // Truncate completion marker
        let file = OpenOptions::new().read(true).write(true).open(&archive_path)?;
        let marker_size = 1;
        let len = file.metadata()?.len();
        file.set_len(len.saturating_sub(marker_size))?;

        let destination = temporary.path().join("restored");
        ensure!(run_archive(&ArchiveArgs { command: ArchiveCommand::Extract {
            archive: archive_path.clone(), destination: destination.clone(), overwrite: false,
            trust_symlink_targets: false, best_effort_metadata: false, copy_specials_as_specials: false,
            strict: false, check_atime: false, check_ctime: false, check_birthtime: false,
            recover: false, apple_write: Default::default(),
        } }).is_err());
        ensure!(!destination.exists());

        let verify_recover = run_archive(&ArchiveArgs { command: ArchiveCommand::Verify {
            archive: archive_path.clone(),
            recover: true,
        } })?;
        ensure!(matches!(
            verify_recover,
            ToolResult::Immediate { exit_code: 1, ref stderr, .. }
                if String::from_utf8_lossy(stderr).contains("incomplete: true")
        ));

        let extract_recover = run_archive(&ArchiveArgs { command: ArchiveCommand::Extract {
            archive: archive_path.clone(), destination: destination.clone(), overwrite: false,
            trust_symlink_targets: false, best_effort_metadata: false, copy_specials_as_specials: false,
            strict: false, check_atime: false, check_ctime: false, check_birthtime: false,
            recover: true, apple_write: Default::default(),
        } })?;
        ensure!(matches!(
            extract_recover,
            ToolResult::Immediate { exit_code: 1, ref stderr, .. }
                if String::from_utf8_lossy(stderr).contains("incomplete: true")
        ));
        ensure!(std::fs::read(destination.join("source/first"))? == b"hello first");
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
            archive: archive.clone(), sources: vec![source.join("")], resume: false, continue_on_error: false, journal: false,
            options: ArchiveCreateOptions::default(),
        } })?;
        for name in &names {
            std::fs::remove_file(source.join(name).join(name))?;
            std::fs::remove_dir(source.join(name))?;
        }
        std::fs::remove_dir(source)?;
        let reader = JournalReader::new(File::open(&archive)?)?;
        reader.verify()?;
        extract_archive(&reader, &destination, &MaterializeOptions::default(), false)?;
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
        let outcome = create_archive(FileArchiveOutput::new(file), &sources, &archive_path, &plan, false, None, false)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        std::fs::remove_dir_all(&source)?;
        let reader = JournalReader::new(File::open(&archive_path)?)?;
        reader.verify()?;
        let destination = temporary.path().join("restored");
        let options = MaterializeOptions { force_overwrite: true, ..Default::default() };
        extract_archive(&reader, &destination, &options, false)?;
        ensure!(std::fs::read(destination.join("source/data"))? == data);
        ensure!(std::fs::read(destination.join("source/empty"))?.is_empty());
        ensure!(extract_archive(&reader, &destination, &options, false).is_err());
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
            archive: archive.clone(), sources: vec![source.join("")], resume: false, continue_on_error: false, journal: false,
            options: ArchiveCreateOptions::default(),
        } })?;
        let reader = JournalReader::new(File::open(archive)?)?;
        let options = MaterializeOptions::default();
        ensure!(extract_archive(&reader, &destination, &options, false).is_err());
        ensure!(std::fs::read(destination.join("data"))? == b"original payload");
        ensure!(std::fs::metadata(&destination)?.permissions().mode() & 0o777 == 0o755);
        std::fs::remove_file(destination.join("data"))?;
        ensure!(extract_archive(&reader, &destination, &options, false).is_err());
        ensure!(!destination.join("data").exists());
        let options = MaterializeOptions {
            existing_entry_policy: ctb_io::file::ExistingEntryPolicy::Replace,
            ..options
        };
        extract_archive(&reader, &destination, &options, false)?;
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
        let outcome = create_archive(File::create(&path)?, &sources, &path, &plan, false, None, false)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        let completed = std::fs::read(&path)?;
        let mut marker = DcMstring::new();
        marker.push_char(ctb_formats_dcdata::dc::DC_OPERATION_COMPLETED);
        let recovered_length = u64::try_from(completed.len())?.checked_sub(u64::try_from(marker.payload_bytes().len())?).context("Archive too short")?;
        OpenOptions::new().write(true).open(&path)?.set_len(recovered_length)?;
        let before = std::fs::read(&path)?;
        std::fs::write(&source, b"modified!")?;
        ensure!(resume_archive(&path, false).is_err());
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
        let outcome = create_archive(File::create(&path)?, &sources, &path, &plan, false, None, false)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        let length = std::fs::metadata(&path)?.len();
        let mut marker = DcMstring::new();
        marker.push_char(ctb_formats_dcdata::dc::DC_OPERATION_COMPLETED);
        OpenOptions::new().write(true).open(&path)?.set_len(length.checked_sub(u64::try_from(marker.payload_bytes().len())?).context("Missing completion marker")?)?;
        let mut tail = DcMstring::new();
        tail.push_char(ctb_formats_dcdata::dc::DC_BEGIN_CHECKSUMMED_SECTION);
        OpenOptions::new().append(true).open(&path)?.write_all(tail.payload_bytes())?;
        resume_archive(&path, false)?;
        let reader = JournalReader::new(File::open(&path)?)?;
        reader.verify()?;
        ensure!(reader.members.len() == 1);
        ensure!(resume_archive(&path, false).is_err(), "Completed archive accepted append/resume");
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
        let outcome = create_archive(File::create(&path)?, &sources, &path, &plan, false, None, false)?;
        ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 0);
        drop(outcome.output);
        let reader = JournalReader::new(File::open(&path)?)?;
        let destination = temporary.path().join("destination");
        std::fs::create_dir(&destination)?;
        #[cfg(any(unix, windows))]
        {
            let outside = temporary.path().join("outside");
            std::fs::create_dir(&outside)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&outside, destination.join("source"))?;
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(&outside, destination.join("source"))?;
            ensure!(extract_archive(&reader, &destination, &MaterializeOptions {
                existing_entry_policy: ctb_io::file::ExistingEntryPolicy::Replace,
                ..Default::default()
            }, false).is_err());
            ensure!(!outside.join("data").exists());
        }
        Ok(())
    }

    #[cfg(windows)]
    fn windows_qualification_archive(source: &Path, archive: &Path) -> Result<JournalReader> {
        let sources = vec![source.to_path_buf()];
        let plan = plan_sources(&sources, archive)?;
        let outcome = create_archive(FileArchiveOutput::new(File::create(archive)?), &sources, archive, &plan, false, None, false)?;
        let completed = outcome.completed;
        drop(outcome.output);
        let reader = JournalReader::recover(File::open(archive)?)?;
        ensure!(completed && reader.errors.is_empty() && reader.warnings.is_empty(),
            "Windows capture was not clean: errors {:?}, warnings {:?}", reader.errors, reader.warnings);
        reader.verify_self_contained()?;
        Ok(reader)
    }

    #[cfg(windows)]
    #[crate::ctb_test]
    fn windows_native_qualification_large_ads_source_free_archive() -> Result<()> {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        use ctb_io::file::streams::{AttachedStreamSource, FilesystemStreamSource};
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, b"primary payload")?;
        let mut surrogate = vec![58_u16, 0xd800];
        surrogate.extend(":$DATA".encode_utf16());
        let names = [
            ":e\u{301}:$DATA".encode_utf16().collect::<Vec<_>>(),
            ":\u{e9}:$DATA".encode_utf16().collect::<Vec<_>>(),
            surrogate,
        ];
        let mut expected = Vec::new();
        for (index, name) in names.into_iter().enumerate() {
            let mut path = source.as_os_str().encode_wide().collect::<Vec<_>>();
            path.extend(&name);
            let path = PathBuf::from(std::ffi::OsString::from_wide(&path));
            let size = 2_097_169_u64;
            let byte = u8::try_from(index)?.checked_add(41).context("Fixture byte overflow")?;
            let mut output = File::create(path)?;
            ensure!(std::io::copy(&mut std::io::Read::take(std::io::repeat(byte), size), &mut output)? == size);
            output.sync_all()?;
            let mut digest = ctb_formats_checksum::Sha256Stream::new();
            std::io::copy(&mut std::io::Read::take(std::io::repeat(byte), size), &mut digest)?;
            expected.push((ctb_io::file::StreamName::from_windows_utf16(&name), size, digest.finalize()));
        }
        let archive = temporary.path().join("ads.cscarchive");
        let reader = windows_qualification_archive(&source, &archive)?;
        let member = reader.members.iter().find(|member| member.entity.identity.relative_path.as_deref() == Some(Path::new("source.bin")))
            .context("Missing ADS archive member")?;
        ensure!(member.entity.streams.len() == expected.len());
        for stream in &member.entity.streams {
            ensure!(stream.data.is_none() && matches!(stream.payload.as_ref().map(|payload| &payload.0),
                Some(ctb_formats_dcstring::BinaryPayloadRef::FileSpan { .. })), "Large ADS was materialized in memory");
        }
        std::fs::remove_file(&source)?;
        for strict_lossless in [true, false] {
            let destination = temporary.path().join(if strict_lossless { "default" } else { "best-effort" });
            let options = MaterializeOptions { strict_lossless, ..Default::default() };
            extract_archive(&reader, &destination, &options, false)
                .with_context(|| format!("ADS qualification with strict_lossless={strict_lossless}"))?;
            let restored = destination.join("source.bin");
            ensure!(std::fs::read(&restored)? == b"primary payload");
            let streams = ctb_io::file::streams::read_and_hash_streams(&restored)?;
            ensure!(streams.len() == expected.len());
            for (name, size, sha256) in &expected {
                let stream = streams.iter().find(|stream| stream.name.as_ref() == Some(name)).context("ADS name changed during archive round-trip")?;
                ensure!(stream.data.is_none());
                ensure!(matches!(stream.entity.kind, FileEntityKind::Regular { size: actual, sha256: digest, .. } if actual == *size && digest == *sha256));
                let mut payload = FilesystemStreamSource.open(stream)?;
                let mut digest = ctb_formats_checksum::Sha256Stream::new();
                ensure!(std::io::copy(&mut payload, &mut digest)? == *size);
                ensure!(digest.finalize() == *sha256);
            }
        }
        Ok(())
    }

    #[cfg(windows)]
    #[crate::ctb_test]
    fn windows_native_qualification_reparse_source_free_archive() -> Result<()> {
        use ctb_io::file::metadata::{NativeMetadataValue, reparse, windows};
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir_all(source.join("target-dir"))?;
        std::fs::write(source.join("target-file"), b"target payload")?;
        std::os::windows::fs::symlink_file("target-file", source.join("file-link"))?;
        std::os::windows::fs::symlink_dir("target-dir", source.join("dir-link"))?;
        let junction = source.join("junction");
        std::fs::create_dir(&junction)?;
        let target = std::fs::canonicalize(source.join("target-dir"))?;
        let target = target.to_str().context("Fixture target is not Unicode")?;
        let target = target.strip_prefix(r"\\?\").context("Missing native fixture path prefix")?;
        let buffer = reparse::build_mount_point_reparse_buffer(&format!(r"\??\{target}"), target)?;
        let mut native = FileEntity::from_filesystem(&junction, None)?.metadata.native.context("Missing native Windows metadata")?;
        native.values.insert("reparse.data".into(), NativeMetadataValue::Bytes(buffer));
        windows::apply_windows_reparse_metadata(&junction, &native, true)?;
        let mut expected = Vec::new();
        for name in ["file-link", "dir-link", "junction"] {
            let entity = FileEntity::from_filesystem(&source.join(name), None)?;
            ensure!(matches!(entity.kind, FileEntityKind::Symlink { .. }), "Reparse point was not captured as a link: {name}");
            let native = entity.metadata.native.context("Missing captured reparse metadata")?;
            let bytes = native.values.get("reparse.data").context("Missing captured reparse buffer")?.clone();
            expected.push((name, entity.kind, bytes));
        }
        let archive = temporary.path().join("reparse.cscarchive");
        let reader = windows_qualification_archive(&source, &archive)?;
        for name in ["file-link", "dir-link", "junction"] { std::fs::remove_file(source.join(name)).or_else(|_| std::fs::remove_dir(source.join(name)))?; }
        std::fs::remove_dir_all(&source)?;
        for strict_lossless in [true, false] {
            let destination = temporary.path().join(if strict_lossless { "default" } else { "best-effort" });
            let options = MaterializeOptions { strict_lossless, ..Default::default() };
            extract_archive(&reader, &destination, &options, false)
                .with_context(|| format!("Reparse qualification with strict_lossless={strict_lossless}"))?;
            for (name, kind, bytes) in &expected {
                let restored = FileEntity::from_filesystem(&destination.join("source").join(name), None)?;
                ensure!(&restored.kind == kind, "Reparse target changed: {name}");
                ensure!(restored.metadata.native.as_ref().and_then(|native| native.values.get("reparse.data")) == Some(bytes), "Reparse buffer changed: {name}");
            }
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
        let outcome = create_archive(File::create(&archive_path)?, &sources, &archive_path, &plan, false, None, false)?;
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
            ensure!(run_archive(&ArchiveArgs { command: ArchiveCommand::Verify { archive: archive_path.clone(), recover: false } }).is_err());
            let destination = temporary.path().join("refused");
            ensure!(extract_archive(&reader, &destination, &MaterializeOptions {
                ..Default::default()
            }, false).is_err());
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
        let mut writer = JournalWriter::new(File::create(&archive_path)?, &session,
            crate::journal::stream::JournalMagic::ArchiveV2)?;
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
        let options = MaterializeOptions { force_overwrite: true, ..Default::default() };
        extract_archive(reader, destination, &options, false)?;
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

    #[crate::ctb_test]
    fn archive_create_journal_flag_writes_journal_v5_and_omits_bodies() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        let file1 = source.join("hello.txt");
        std::fs::write(&file1, b"hello world")?;
        let sub = source.join("sub");
        std::fs::create_dir(&sub)?;
        let file2 = sub.join("nested.bin");
        std::fs::write(&file2, vec![0x42; 256])?;

        #[cfg(unix)]
        {
            xattr::set(&file1, "user.ctb.test", b"stream body")?;
        }

        let journal_path = temporary.path().join("manifest.cscjournal");
        let result = run_archive(&ArchiveArgs {
            command: ArchiveCommand::Create {
                archive: journal_path.clone(),
                sources: vec![source.join("")],
                resume: false,
                continue_on_error: false,
                journal: true,
                options: ArchiveCreateOptions::default(),
            },
        })?;
        ensure!(matches!(result, ToolResult::Immediate { exit_code: 0, .. }));

        // 1. Verify leading magic is JOURNAL_MAGIC (b"CTBCSCJ\x05")
        let mut f = File::open(&journal_path)?;
        let mut magic = [0_u8; 8];
        f.read_exact(&mut magic)?;
        ensure!(&magic == crate::journal::JOURNAL_MAGIC, "Expected JOURNAL_MAGIC magic header");

        // 2. Read with JournalReader: regular file bodies and stream bodies must be omitted
        let reader = JournalReader::new(File::open(&journal_path)?)?;
        ensure!(reader.is_completed(), "Journal must be completed");
        ensure!(!reader.members.is_empty(), "Journal must have committed members");
        for member in &reader.members {
            ensure!(member.payload.is_none(), "Journal format must omit file payloads");
            for stream in &member.entity.streams {
                ensure!(stream.data.is_none() && stream.payload.is_none(), "Journal format must omit stream bodies");
            }
        }

        // 3. Verify it can be used with csc-verify against the source directory
        let verify_args = default_verify_args(journal_path.clone(), Some(source.clone()));
        let verify_res = run_csc_verify(&verify_args)?;
        ensure!(matches!(verify_res, ToolResult::Immediate { exit_code: 0, .. }));

        Ok(())
    }

    #[crate::ctb_test]
    fn csc_verify_supports_default_archive_format_source_free() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        std::fs::write(source.join("alpha.txt"), b"alpha content")?;
        std::fs::write(source.join("beta.txt"), b"beta content")?;

        let archive_path = temporary.path().join("test.cscarchive");
        let result = run_archive(&ArchiveArgs {
            command: ArchiveCommand::Create {
                archive: archive_path.clone(),
                sources: vec![source.join("")],
                resume: false,
                continue_on_error: false,
                journal: false,
                options: ArchiveCreateOptions::default(),
            },
        })?;
        ensure!(matches!(result, ToolResult::Immediate { exit_code: 0, .. }));

        // Verify magic is ARCHIVE_MAGIC
        let mut f = File::open(&archive_path)?;
        let mut magic = [0_u8; 8];
        f.read_exact(&mut magic)?;
        ensure!(&magic == crate::journal::ARCHIVE_MAGIC, "Expected ARCHIVE_MAGIC header");

        // Extract to destination
        let dest = temporary.path().join("extracted");
        let extract_res = run_archive(&ArchiveArgs {
            command: ArchiveCommand::Extract {
                archive: archive_path.clone(),
                destination: dest.clone(),
                overwrite: false,
                trust_symlink_targets: false,
                best_effort_metadata: false,
                strict: false,
                check_atime: false,
                check_ctime: false,
                check_birthtime: false,
                copy_specials_as_specials: false,
                recover: false,
                apple_write: Default::default(),
            },
        })?;
        ensure!(matches!(extract_res, ToolResult::Immediate { exit_code: 0, .. }));

        // Remove source completely: verify must succeed without touching source filesystem!
        std::fs::remove_dir_all(&source)?;
        ensure!(!source.exists());

        // Verify using positional target directory
        let verify_args = default_verify_args(archive_path.clone(), Some(dest.clone()));
        let verify_res = run_csc_verify(&verify_args)?;
        ensure!(matches!(verify_res, ToolResult::Immediate { exit_code: 0, .. }));

        // Verify using --dir flag
        let mut verify_flag_args = default_verify_args(archive_path.clone(), None);
        verify_flag_args.dir_flag = Some(dest.clone());
        let flag_res = run_csc_verify(&verify_flag_args)?;
        ensure!(matches!(flag_res, ToolResult::Immediate { exit_code: 0, .. }));

        // Error guidance when verifying archive without specifying directory
        let no_dest_args = default_verify_args(archive_path.clone(), None);
        let err = match run_csc_verify(&no_dest_args) {
            Err(e) => e,
            Ok(_) => bail!("Expected csc-verify without target directory to fail"),
        };
        ensure!(err.to_string().contains("pass target directory as argument or --dir <path>"));

        // Introduce a content corruption in extracted directory
        std::fs::write(dest.join("alpha.txt"), b"corrupted content")?;
        let corrupt_res = run_csc_verify(&verify_args)?;
        ensure!(matches!(corrupt_res, ToolResult::Immediate { exit_code: 1, .. }));

        Ok(())
    }

    #[crate::ctb_test]
    fn archive_legacy_v1_journal_cli_and_resume_refusal_without_truncation() -> Result<()> {
        use crate::journal::legacy::{
            JOURNAL_MAGIC_V1, TAG_BATCH_COMMIT, TAG_ENTITY, TAG_JOB_COMPLETED,
            TAG_SESSION_HEADER, write_bytes, write_entity_payload, write_u32,
            write_u64,
        };
        use ctb_formats_checksum::{Sha256Stream, xxhash};
        use ctb_formats_dcstring::SharedValueContext;

        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        let file_path = source.join("alpha.txt");
        std::fs::write(&file_path, b"alpha")?;

        let context = SharedValueContext::default();
        let mut entity = FileEntity::from_filesystem(&file_path, None)?;
        let expected_rel = Path::new("source").join("alpha.txt");
        entity.identity.relative_path = Some(expected_rel.clone());
        let mut encoded = Vec::new();
        write_entity_payload(&mut encoded, &entity, &context)?;

        let mut batch_hasher = Sha256Stream::new();
        batch_hasher.update(&encoded);
        let batch_hash = batch_hasher.finalize();

        let mut v1_bytes = JOURNAL_MAGIC_V1.to_vec();
        v1_bytes.extend_from_slice(&[
            TAG_SESSION_HEADER,
            crate::journal::current_platform(),
        ]);
        write_u32(&mut v1_bytes, 1)?;
        write_bytes(&mut v1_bytes, source.as_os_str().as_encoded_bytes())?;
        write_bytes(&mut v1_bytes, b"dest")?;
        v1_bytes.push(TAG_ENTITY);
        write_u32(&mut v1_bytes, u32::try_from(encoded.len())?)?;
        v1_bytes.extend_from_slice(&xxhash::xxhash3_64(&encoded));
        v1_bytes.extend_from_slice(&encoded);
        v1_bytes.push(TAG_BATCH_COMMIT);
        write_u64(&mut v1_bytes, 1)?;
        v1_bytes.extend_from_slice(&batch_hash);

        // Incomplete v1 journal with trailing uncommitted bytes to verify resume does not truncate
        let mut incomplete_with_tail = v1_bytes.clone();
        incomplete_with_tail.push(TAG_ENTITY);
        incomplete_with_tail.extend_from_slice(&[0x01, 0x02]);
        let incomplete_path = temporary.path().join("incomplete_v1.cscjournal");
        std::fs::write(&incomplete_path, &incomplete_with_tail)?;

        for journal_flag in [false, true] {
            let err = run_archive(&ArchiveArgs {
                command: ArchiveCommand::Create {
                    archive: incomplete_path.clone(),
                    sources: Vec::new(),
                    resume: true,
                    continue_on_error: false,
                    journal: journal_flag,
                    options: ArchiveCreateOptions::default(),
                },
            })
            .err()
            .context("Resuming legacy v1 journal must fail")?;
            ensure!(
                format!("{err:#}").contains("Cannot resume a legacy CSC archive or journal"),
                "Unexpected error: {err:#}"
            );
            ensure!(
                std::fs::read(&incomplete_path)? == incomplete_with_tail,
                "Refused legacy v1 journal resume must not truncate or modify file"
            );
        }

        // Completed v1 journal supports `archive list` and `archive verify --allow-journal-errors`
        v1_bytes.push(TAG_JOB_COMPLETED);
        let completed_path = temporary.path().join("completed_v1.cscjournal");
        std::fs::write(&completed_path, &v1_bytes)?;

        let list_res = run_archive(&ArchiveArgs {
            command: ArchiveCommand::List {
                archive: completed_path.clone(),
            },
        })?;
        let ToolResult::Immediate { stdout, exit_code: 0, .. } = list_res else {
            bail!("Expected archive list on completed v1 journal to succeed");
        };
        ensure!(
            String::from_utf8_lossy(&stdout).contains(&expected_rel.display().to_string())
        );

        // Strict verify requires self-contained file payloads, so it errors on nonempty journal members
        ensure!(
            run_archive(&ArchiveArgs {
                command: ArchiveCommand::Verify {
                    archive: completed_path.clone(),
                    recover: false,
                },
            })
            .is_err()
        );

        // Recover / --allow-journal-errors verify succeeds on v1 journal
        let verify_recover = run_archive(&ArchiveArgs {
            command: ArchiveCommand::Verify {
                archive: completed_path,
                recover: true,
            },
        })?;
        ensure!(matches!(
            verify_recover,
            ToolResult::Immediate { exit_code: 0, .. }
        ));

        Ok(())
    }

    #[crate::ctb_test]
    fn archive_extract_best_effort_adapts_overlong_filename_and_verifies() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let archive_path = temporary.path().join("overlong.cscarchive");
        let overlong_name = format!("{}.txt", "b".repeat(260));
        let payload_bytes = b"archived overlong file payload".to_vec();
        let sha256 = ctb_formats_checksum::sha256(&payload_bytes);

        {
            let file = File::create(&archive_path)?;
            let session = crate::journal::JournalSessionRecord {
                sources: vec![PathBuf::from("src")],
                destination: PathBuf::new(),
                origin_platform: u32::from(crate::journal::current_platform()),
                environment: None,
            };
            let mut writer = crate::journal::JournalWriter::new(
                file,
                &session,
                crate::journal::JournalMagic::ArchiveV2,
            )?;
            let entity = FileEntity {
                identity: ctb_io::file::FileIdentity {
                    origin: ctb_io::file::FileOrigin::Synthetic,
                    relative_path: Some(PathBuf::from(&overlong_name)),
                    enclosing_path: None,
                    nlink: 1,
                    hardlink_group: None,
                },
                kind: FileEntityKind::Regular {
                    size: u64::try_from(payload_bytes.len())?,
                    sha256,
                    is_sparse: false,
                    extents: vec![ctb_io::file::payload::Extent::Data {
                        offset: 0,
                        length: u64::try_from(payload_bytes.len())?,
                    }],
                },
                metadata: ctb_io::file::FileMetadata::default(),
                streams: Vec::new(),
            };
            let mut payload = ctb_io::file::MemoryPayloadSource::new(payload_bytes.clone())?;
            writer.write_entity_checked(
                &entity,
                Some(&mut payload),
                ctb_io::file::encoding::BodyPolicy::Include,
                || Ok(()),
            )?;
            writer.finish()?;
        }

        // Strict extraction must fail on >255-byte component.
        let strict_dest = temporary.path().join("strict_dest");
        ensure!(
            run_archive(&ArchiveArgs {
                command: ArchiveCommand::Extract {
                    archive: archive_path.clone(),
                    destination: strict_dest,
                    overwrite: false,
                    trust_symlink_targets: false,
                    best_effort_metadata: false,
                    strict: true,
                    check_atime: false,
                    check_ctime: false,
                    check_birthtime: false,
                    copy_specials_as_specials: false,
                    recover: false,
                    apple_write: Default::default(),
                },
            })
            .is_err(),
            "Strict extraction must reject overlong filename"
        );

        // Best-effort extraction must adapt the filename, materialize the file, and pass verification.
        let be_dest = temporary.path().join("be_dest");
        run_archive(&ArchiveArgs {
            command: ArchiveCommand::Extract {
                archive: archive_path.clone(),
                destination: be_dest.clone(),
                overwrite: false,
                trust_symlink_targets: false,
                best_effort_metadata: true,
                strict: false,
                check_atime: false,
                check_ctime: false,
                check_birthtime: false,
                copy_specials_as_specials: false,
                recover: false,
                apple_write: Default::default(),
            },
        })?;

        let (expected_adapted, _) = ctb_io::file::adapt_relative_path_for_materialization(
            Path::new(&overlong_name),
            cfg!(windows),
        );
        ensure!(
            std::fs::read(be_dest.join(&expected_adapted))? == payload_bytes,
            "Adapted file must exist with intact payload at {}",
            expected_adapted.display()
        );

        // `verify_directory_against_manifest` in best-effort mode must locate the adapted file.
        let mut verify_args = default_verify_args(archive_path, Some(be_dest));
        verify_args.criteria.best_effort = true;
        verify_args.criteria.ignore_owner = true;
        verify_args.criteria.ignore_perms = true;
        verify_args.criteria.ignore_mtime = true;
        let report = crate::verifier::verify_directory_against_manifest(&verify_args)?;
        ensure!(
            report.is_clean(),
            "Best-effort verification against archive with adapted filename must pass: {report:?}"
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn archive_csc_option_exposure_apple_streams_and_source_change() -> Result<()> {
        let create_parsed = ArchiveArgs::try_parse_from([
            "archive",
            "create",
            "out.cscarchive",
            "src/",
            "--full-provenance",
            "--on-source-change",
            "best-effort",
            "-x",
            "--read-apple-double-alongside",
            "--read-apple-single-with-extension",
            "as",
            "--read-windows-ads",
        ])?;
        let ArchiveCommand::Create { options, .. } = create_parsed.command else {
            bail!("Expected ArchiveCommand::Create");
        };
        ensure!(options.full_provenance);
        ensure!(options.on_source_change == crate::args::SourceChangePolicy::BestEffort);
        ensure!(options.one_file_system);
        let resolved_read = options.apple_read.resolve();
        ensure!(resolved_read.read_apple_double_alongside);
        ensure!(resolved_read.read_apple_single_as);
        ensure!(resolved_read.read_windows_ads_alongside);

        let extract_parsed = ArchiveArgs::try_parse_from([
            "archive",
            "extract",
            "out.cscarchive",
            "dest/",
            "--force-write-apple-double",
            "alongside",
        ])?;
        let ArchiveCommand::Extract { apple_write, .. } = extract_parsed.command else {
            bail!("Expected ArchiveCommand::Extract");
        };
        ensure!(
            apple_write.force_write_apple_double
                == Some(ctb_io::file::apple_double::AppleDoubleStyle::Alongside)
        );

        // End-to-end AppleDouble + AppleSingle (.as) capture and AppleDouble alongside extraction
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        std::fs::create_dir(&source)?;
        std::fs::write(source.join("doc.txt"), b"primary doc")?;
        let double_archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleDouble,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: None,
            comment: Some("doc comment".to_owned()),
            timestamps: None,
            backup_timestamp_sec: None,
            finder_info: None,
            extended_attributes: Vec::new(),
            data_fork: None,
            resource_fork: Some(b"doc rsrc".to_vec()),
            data_fork_size: None,
            resource_fork_size: Some(8),
            unrecognized_entries: Vec::new(),
            entries: Vec::new(),
        };
        std::fs::write(
            source.join("._doc.txt"),
            ctb_io::file::write_apple_single_double(&double_archive)?,
        )?;

        let mut single_archive = double_archive.clone();
        single_archive.format = ctb_io::file::AppleFormat::AppleSingle;
        single_archive.comment = Some("single comment".to_owned());
        single_archive.data_fork = Some(b"single data fork".to_vec());
        std::fs::write(
            source.join("note.txt.as"),
            ctb_io::file::write_apple_single_double(&single_archive)?,
        )?;

        let archive_path = temporary.path().join("apple.cscarchive");
        let create_res = run_archive(&ArchiveArgs::try_parse_from([
            "archive",
            "create",
            archive_path.to_str().context("Non-UTF8 path")?,
            source.to_str().context("Non-UTF8 path")?,
            "--read-apple-double-alongside",
            "--read-apple-single-with-extension",
            "as",
        ])?)?;
        ensure!(matches!(create_res, ToolResult::Immediate { exit_code: 0, .. }));

        let reader = JournalReader::new(File::open(&archive_path)?)?;
        reader.verify_self_contained()?;
        // `._doc.txt` companion is absorbed into `source/doc.txt` and `note.txt.as` is stripped to `source/note.txt`
        let rel_paths: HashSet<_> = reader
            .members
            .iter()
            .filter_map(|m| m.entity.identity.relative_path.clone())
            .collect();
        ensure!(rel_paths.contains(Path::new("source/doc.txt")));
        ensure!(rel_paths.contains(Path::new("source/note.txt")));
        ensure!(!rel_paths.contains(Path::new("source/._doc.txt")));
        ensure!(!rel_paths.contains(Path::new("source/note.txt.as")));

        let extracted = temporary.path().join("extracted");
        let extract_res = run_archive(&ArchiveArgs::try_parse_from([
            "archive",
            "extract",
            archive_path.to_str().context("Non-UTF8 path")?,
            extracted.to_str().context("Non-UTF8 path")?,
            "--force-write-apple-double",
            "alongside",
        ])?)?;
        ensure!(matches!(extract_res, ToolResult::Immediate { exit_code: 0, .. }));
        ensure!(std::fs::read(extracted.join("source/doc.txt"))? == b"primary doc");
        ensure!(extracted.join("source/._doc.txt").is_file());
        ensure!(std::fs::read(extracted.join("source/note.txt"))? == b"single data fork");
        ensure!(extracted.join("source/._note.txt").is_file());

        // Verify `--on-source-change best-effort` records `SourceModifiedDuringCopy` warning while committing member
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            struct ChmodOnCapture {
                target: PathBuf,
                options: ArchiveCreateOptions,
            }
            impl CaptureSource for ChmodOnCapture {
                fn capture(&mut self, path: &Path) -> Result<CapturedSource> {
                    let best_effort =
                        self.options.on_source_change == crate::args::SourceChangePolicy::BestEffort;
                    let mut source = CapturedSource::capture(
                        path,
                        None,
                        &self.options.apple_read.resolve(),
                        best_effort,
                    )?;
                    if path == self.target {
                        let mode = std::fs::metadata(path)?.permissions().mode() ^ 0o100;
                        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
                    }
                    source.hash(path)?;
                    Ok(source)
                }
            }

            let sc_dir = temporary.path().join("sc_source");
            std::fs::create_dir(&sc_dir)?;
            let sc_sources = vec![sc_dir.clone()];
            let sc_archive = temporary.path().join("sc_best_effort.cscarchive");
            let sc_options = ArchiveCreateOptions {
                on_source_change: crate::args::SourceChangePolicy::BestEffort,
                ..Default::default()
            };
            let sc_plan = plan_sources_with_options(&sc_sources, &sc_archive, &sc_options)?;
            let outcome = capture_archive(
                FileArchiveOutput::new(File::create(&sc_archive)?),
                CaptureRequest {
                    sources: &sc_sources,
                    archive: &sc_archive,
                    plan: &sc_plan,
                    continue_on_error: false,
                    recovered: None,
                    journal: false,
                    options: &sc_options,
                },
                &mut ChmodOnCapture {
                    target: sc_dir,
                    options: sc_options.clone(),
                },
            )?;
            ensure!(outcome.completed && outcome.error_count == 0 && outcome.warning_count == 1);
            drop(outcome.output);
            let sc_reader = JournalReader::recover(File::open(&sc_archive)?)?;
            ensure!(sc_reader.is_completed() && sc_reader.members.len() == 1);
            ensure!(sc_reader
                .warnings
                .iter()
                .any(|w| w.code == "SourceModifiedDuringCopy"));
        }

        Ok(())
    }
}