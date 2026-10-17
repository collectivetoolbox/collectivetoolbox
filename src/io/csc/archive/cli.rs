#[expect(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use super::{ArchiveReader, ArchiveWriter};
use crate::journal::{JournalSessionRecord, current_platform};
use clap::{Parser, Subcommand};
use ctb_io::file::{
    DirTraverser, DiskPayloadSource, FileEntity, FileEntityKind, FileOrigin,
    InodeKey, LinkPath, MaterializeOptions, PathTraversalPolicy, SandboxableDir,
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
    },
    /// List the members of a completed archive.
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
        ArchiveCommand::Create { archive, sources, resume } => {
            let _scope = ctb_io_environment::GlobalEnvironmentScope::enter_fresh();
            if *resume {
                ensure!(archive != Path::new("-"), "Cannot resume archive output on stdout");
                resume_archive(archive)?;
                return Ok(ToolResult::Immediate {
                    stdout: Vec::new(), stderr: b"Archive resumed and completed.\n".to_vec(), exit_code: 0,
                });
            }
            let plan = plan_sources(sources, archive)?;
            if archive == Path::new("-") {
                #[cfg(unix)]
                {
                    use std::os::fd::AsFd;
                    let output = File::from(std::io::stdout().as_fd().try_clone_to_owned()?);
                    validate_output_identity(&output, &plan)?;
                }
                drop(create_archive(std::io::stdout().lock(), sources, archive, &plan, None, None)?);
            } else {
                let mut options = OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                let file = options.open(archive).with_context(|| format!("Cannot create {}", archive.display()))?;
                let sync = file.try_clone()?;
                create_archive(BufWriter::new(file), sources, archive, &plan, Some(&sync), None)?;
                sync.sync_all()?;
            }
            Ok(ToolResult::Immediate {
                stdout: Vec::new(), stderr: b"Archive created.\n".to_vec(), exit_code: 0,
            })
        }
        ArchiveCommand::List { archive } => {
            let reader = ArchiveReader::recover(File::open(archive)?)?;
            validate_members(&reader)?;
            let mut output = String::new();
            use std::fmt::Write;
            for member in &reader.members {
                let size = match &member.entity.kind { FileEntityKind::Regular { size, .. } => *size, _ => 0 };
                let path = member.entity.identity.relative_path.as_deref().context("Archive member has no path")?;
                writeln!(output, "{}\t{size}\t{}", member.entity.kind.entity_type().as_str(), path.display())?;
            }
            Ok(ToolResult::Immediate {
                stdout: output.into_bytes(),
                stderr: if reader.completed { Vec::new() } else { b"Archive is incomplete; listing committed members only.\n".to_vec() },
                exit_code: if reader.completed { 0 } else { 1 },
            })
        }
        ArchiveCommand::Verify { archive } => {
            let reader = ArchiveReader::new(File::open(archive)?)?;
            reader.verify()?;
            validate_members(&reader)?;
            Ok(ToolResult::immediate_ok(b"Archive verified.\n".to_vec()))
        }
        ArchiveCommand::Extract { archive, destination, overwrite, trust_symlink_targets,
            best_effort_metadata, copy_specials_as_specials } => {
            let reader = ArchiveReader::new(File::open(archive)?)?;
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

fn plan_sources(sources: &[PathBuf], archive: &Path) -> Result<Vec<SourceMember>> {
    ensure!(!sources.is_empty(), "At least one source is required");
    let output = if archive == Path::new("-") { None } else { Some(resolve_existing_ancestors(archive)?) };
    let mut plan = Vec::new();
    let mut names = HashSet::new();
    for source in sources {
        let metadata = std::fs::symlink_metadata(source)?;
        if let Some(output) = &output {
            let resolved = resolve_existing_ancestors(source)?;
            ensure!(*output != resolved && !(metadata.is_dir() && output.starts_with(&resolved)),
                "Archive output overlaps a source: {}", source.display());
        }
        let prefix = if metadata.is_dir() && path_has_trailing_slash(source) {
            PathBuf::from(".")
        } else {
            PathBuf::from(source.file_name().context("Source has no basename; use a trailing slash for a directory root")?)
        };
        if metadata.is_dir() {
            let traverser = DirTraverser::new(source, TraversalOptions::default().yield_root(true))?;
            for item in traverser {
                let item = item?;
                ensure!(item.errors.is_empty(), "Source traversal errors: {}", item.errors.join("; "));
                let relative = normalized_member(&prefix.join(&item.relative_path))?;
                ensure!(names.insert(relative.clone()), "Duplicate archive member: {}", relative.display());
                plan.push(SourceMember { source: item.path, relative });
            }
        } else {
            let relative = normalized_member(&prefix)?;
            ensure!(names.insert(relative.clone()), "Duplicate archive member: {}", relative.display());
            plan.push(SourceMember { source: source.clone(), relative });
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

fn create_archive<W: Write>(output: W, sources: &[PathBuf], archive: &Path, plan: &[SourceMember], sync: Option<&File>, recovered: Option<&ArchiveReader>) -> Result<W> {
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
    let mut writer = if let Some(recovered) = recovered { ArchiveWriter::resume(output, recovered)? }
        else { ArchiveWriter::new(output, &session)? };
    let mut hardlinks = HashMap::<InodeKey, PathBuf>::new();
    let mut committed_paths = HashSet::new();
    if let Some(recovered) = recovered {
        for member in &recovered.members {
            let relative = member.entity.identity.relative_path.as_ref().context("Committed member missing path")?;
            committed_paths.insert(relative.clone());
            if matches!(member.entity.kind, FileEntityKind::Regular { .. }) {
                if let FileOrigin::Filesystem { key, .. } = &member.entity.identity.origin {
                    hardlinks.insert(key.clone(), relative.clone());
                }
            }
        }
    }
    for member in plan {
        let mut entity = FileEntity::from_filesystem(&member.source, None)?;
        entity.identity.relative_path = Some(member.relative.clone());
        entity.identity.enclosing_path = None;
        if let Some(environment) = &session.environment { entity.set_environment(environment.clone()); }
        if matches!(entity.kind, FileEntityKind::Regular { .. }) && entity.identity.nlink > 1 {
            if let FileOrigin::Filesystem { key, .. } = &entity.identity.origin {
                if let Some(target) = hardlinks.get(key) {
                    if target != &member.relative {
                        entity.kind = FileEntityKind::Hardlink { target_relative_path: LinkPath::from_relative_path(target)? };
                    }
                } else {
                    hardlinks.insert(key.clone(), member.relative.clone());
                }
            }
        }
        if committed_paths.contains(&member.relative) {
            continue;
        }
        if matches!(entity.kind, FileEntityKind::Regular { .. }) {
            let mut payload = DiskPayloadSource::open(&member.source)?;
            writer.write_entity(&entity, Some(&mut payload))?;
        } else {
            writer.write_entity(&entity, None)?;
        }
        writer.writer.flush()?;
        if let Some(file) = sync { file.sync_data()?; }
    }
    writer.finish()
}

fn resume_archive(path: &Path) -> Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let recovered = ArchiveReader::recover(file.try_clone()?)?;
    ensure!(!recovered.completed, "Cannot resume a completed archive");
    recovered.verify_contents()?;
    validate_members(&recovered)?;
    let plan = plan_sources(&recovered.session.sources, path)?;
    let planned: HashMap<_, _> = plan.iter().map(|member| (member.relative.as_path(), member.source.as_path())).collect();
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
    let sync = file.try_clone()?;
    create_archive(BufWriter::new(file), &recovered.session.sources, path, &plan, Some(&sync), Some(&recovered))?;
    sync.sync_all()?;
    Ok(())
}

fn validate_members(reader: &ArchiveReader) -> Result<()> {
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

fn extract_archive(reader: &ArchiveReader, destination: &Path, options: &MaterializeOptions) -> Result<()> {
    ensure!(cfg!(unix), "Sandboxed archive extraction is currently supported only on Unix");
    reader.verify()?;
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

    #[crate::ctb_test]
    fn archive_stdout_argument_parsing() -> Result<()> {
        let args = ArchiveArgs::try_parse_from(["archive", "create", "-", "source/"])?;
        ensure!(matches!(args.command, ArchiveCommand::Create { archive, sources, resume: false }
            if archive == Path::new("-") && sources == vec![PathBuf::from("source/")]));
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
            archive: archive.clone(), sources: vec![source.join("")], resume: false,
        } })?;
        for name in &names {
            std::fs::remove_file(source.join(name).join(name))?;
            std::fs::remove_dir(source.join(name))?;
        }
        std::fs::remove_dir(source)?;
        let reader = ArchiveReader::new(File::open(&archive)?)?;
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
        create_archive(BufWriter::new(file), &sources, &archive_path, &plan, None, None)?;
        std::fs::remove_dir_all(&source)?;
        let reader = ArchiveReader::new(File::open(&archive_path)?)?;
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
            archive: archive.clone(), sources: vec![source.join("")], resume: false,
        } })?;
        let reader = ArchiveReader::new(File::open(archive)?)?;
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
        create_archive(File::create(&path)?, &sources, &path, &plan, None, None)?;
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
        create_archive(File::create(&path)?, &sources, &path, &plan, None, None)?;
        let length = std::fs::metadata(&path)?.len();
        let mut marker = DcMstring::new();
        marker.push_char(ctb_formats_dcdata::dc::DC_OPERATION_COMPLETED);
        OpenOptions::new().write(true).open(&path)?.set_len(length.checked_sub(u64::try_from(marker.payload_bytes().len())?).context("Missing completion marker")?)?;
        let mut tail = DcMstring::new();
        tail.push_char(ctb_formats_dcdata::dc::DC_BEGIN_CHECKSUMMED_SECTION);
        OpenOptions::new().append(true).open(&path)?.write_all(tail.payload_bytes())?;
        resume_archive(&path)?;
        let reader = ArchiveReader::new(File::open(&path)?)?;
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
        create_archive(File::create(&path)?, &sources, &path, &plan, None, None)?;
        let reader = ArchiveReader::new(File::open(&path)?)?;
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
    fn archive_links_and_attached_streams_roundtrip() -> Result<()> {
        use std::os::unix::fs::MetadataExt;
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
        create_archive(File::create(&archive_path)?, &sources, &archive_path, &plan, None, None)?;
        std::fs::remove_dir_all(&source)?;
        let reader = ArchiveReader::new(File::open(&archive_path)?)?;
        reader.verify()?;
        let destination = temporary.path().join("restored");
        let options = MaterializeOptions { strict_lossless: false, force_overwrite: true, ..Default::default() };
        extract_archive(&reader, &destination, &options)?;
        let data = destination.join("source/data");
        let hardlink = destination.join("source/hardlink");
        ensure!(std::fs::metadata(&data)?.ino() == std::fs::metadata(&hardlink)?.ino());
        ensure!(std::fs::read_link(destination.join("source/symlink"))? == Path::new("data"));
        ensure!(xattr::get(data, "user.ctb.archive")? == Some(b"stream payload".to_vec()));
        Ok(())
    }
}