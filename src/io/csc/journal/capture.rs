#[expect(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use ctb_io::file::{FileEntity, FileEntityKind, FileOrigin, InodeKey, PayloadSource};
use ctb_io::file::apple_double::AppleReadOptions;
use ctb_io::file::payload::{DiskPayloadSource, SourceSnapshot};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::{Read, Seek, SeekFrom};

pub(crate) struct CapturedPayload<P> {
    source: P,
    snapshot: SourceSnapshot,
}

impl<P: PayloadSource> CapturedPayload<P> {
    pub fn new(mut source: P, snapshot: &SourceSnapshot) -> Result<Self> {
        source.bind_snapshot(snapshot)?;
        Ok(Self { source, snapshot: snapshot.clone() })
    }
}

impl<P: PayloadSource> Read for CapturedPayload<P> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> { self.source.read(buffer) }
}

impl<P: PayloadSource> Seek for CapturedPayload<P> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> { self.source.seek(position) }
}

impl<P: PayloadSource> PayloadSource for CapturedPayload<P> {
    fn total_size(&self) -> u64 { self.source.total_size() }
    fn extents(&self) -> &[ctb_io::file::Extent] { self.source.extents() }
    fn opened_with_noatime(&self) -> bool { self.source.opened_with_noatime() }
    fn finish(&mut self) -> Result<Vec<String>> {
        let warnings = self.source.finish()?;
        self.snapshot.retain_changes(self.snapshot.validate()?)?;
        Ok(warnings)
    }
}

pub(crate) struct CapturedSource {
    pub entity: FileEntity,
    pub snapshot: SourceSnapshot,
}

#[derive(Clone, Default)]
pub(crate) struct CapturedTree {
    snapshots: Vec<SourceSnapshot>,
    reported_warnings: std::cell::RefCell<std::collections::HashSet<(String, String)>>,
}

impl CapturedTree {
    pub fn retain(&mut self, snapshot: &SourceSnapshot) {
        self.snapshots.push(snapshot.clone());
    }

    pub fn validate(&self) -> Result<()> {
        for snapshot in &self.snapshots {
            snapshot.retain_changes(snapshot.validate()?)?;
        }
        Ok(())
    }

    pub fn visit_pending_warnings(&self, visitor: &mut impl FnMut(&str, &str) -> Result<()>) -> Result<()> {
        for snapshot in &self.snapshots {
            visit_snapshot_warnings(snapshot, &mut |code, warning| {
                let key = (code.to_owned(), warning.to_owned());
                if !self.reported_warnings.borrow().contains(&key) {
                    visitor(code, warning)?;
                    self.reported_warnings.borrow_mut().insert(key);
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}

impl CapturedSource {
    pub fn capture(path: &Path, base: Option<&Path>, apple: &AppleReadOptions, best_effort: bool) -> Result<Self> {
        let snapshot = SourceSnapshot::capture(path, best_effort)?;
        let mut entity = FileEntity::from_filesystem_metadata_only_captured(path, base, apple, Some(&snapshot))?;
        attach_environment(&mut entity, &ctb_io_environment::capture_quick_arc());
        snapshot.retain_changes(snapshot.validate()?)?;
        finalize_warnings(&snapshot, &mut entity, &mut |_, _| Ok(()))?;
        Ok(Self { entity, snapshot })
    }

    pub fn open(&self, path: &Path) -> Result<DiskPayloadSource> {
        open_payload(path, &self.snapshot)
    }

    pub fn hash(&mut self, path: &Path) -> Result<()> {
        if let FileEntityKind::Regular { sha256, .. } = &self.entity.kind {
            if *sha256 == [0_u8; 32] {
                let mut payload = self.open(path)?;
                hash_primary(&mut self.entity, &mut payload, path)?;
                for warning in payload.finish()? { self.entity.metadata.push_warning(warning); }
            }
        }
        finalize_warnings(&self.snapshot, &mut self.entity, &mut |_, _| Ok(()))
    }
}

pub(crate) fn open_payload(path: &Path, snapshot: &SourceSnapshot) -> Result<DiskPayloadSource> {
    snapshot.validate()?;
    let mut payload = DiskPayloadSource::open(path)?;
    payload.bind_snapshot(snapshot)?;
    Ok(payload)
}

pub(crate) fn hash_primary<P: PayloadSource>(entity: &mut FileEntity, payload: &mut P, path: &Path) -> Result<()> {
    if let FileEntityKind::Regular { sha256, extents, is_sparse, .. } = &mut entity.kind {
        if *sha256 == [0_u8; 32] {
            *sha256 = ctb_io::file::payload::hash_payload_stream(payload, extents, *is_sparse, path)?;
        }
    }
    Ok(())
}

pub(crate) fn attach_environment(entity: &mut FileEntity, environment: &std::sync::Arc<ctb_io_environment::EnvDescription>) {
    if entity.environment().is_none() {
        entity.metadata.set_environment(std::sync::Arc::clone(environment));
    }
    for stream in &mut entity.streams { attach_environment(&mut stream.entity, environment); }
}

#[derive(Default)]
pub(crate) struct CommittedHardlinks {
    entries: HashMap<InodeKey, CommittedHardlink>,
}

struct CommittedHardlink {
    relative: PathBuf,
    kind: FileEntityKind,
    timestamps: Option<ctb_io::file::metadata::FileTimestamps>,
    snapshot: Option<SourceSnapshot>,
}

impl CommittedHardlinks {
    pub fn lookup(&self, entity: &FileEntity, relative: &Path) -> Result<Option<&Path>> {
        let FileOrigin::Filesystem { key, .. } = &entity.identity.origin else { return Ok(None); };
        let Some(definition) = self.entries.get(key) else { return Ok(None); };
        if definition.relative == relative { return Ok(None); }
        if let Some(snapshot) = &definition.snapshot {
            ensure!(snapshot.validate()?.is_empty(), "Hardlink defining source version changed: {}", definition.relative.display());
        }
        ensure!(entity.kind == definition.kind && (definition.snapshot.is_some() || entity.metadata.timestamps == definition.timestamps),
            "Hardlink source version changed since defining member: {}", relative.display());
        Ok(Some(&definition.relative))
    }

    pub fn commit(&mut self, entity: &FileEntity, relative: &Path, snapshot: Option<&SourceSnapshot>) {
        if entity.is_regular() {
            if let FileOrigin::Filesystem { key, .. } = &entity.identity.origin {
                self.entries.insert(key.clone(), CommittedHardlink {
                    relative: relative.to_path_buf(), kind: entity.kind.clone(),
                    timestamps: entity.metadata.timestamps.clone(), snapshot: snapshot.cloned(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn apple_companion_fixture(path: &Path) -> Result<()> {
        let archive = ctb_io::file::AppleArchive {
            format: ctb_io::file::AppleFormat::AppleDouble,
            version: ctb_io::file::VERSION_2_0_BE,
            real_name: None, comment: None, timestamps: None, backup_timestamp_sec: None,
            finder_info: None, extended_attributes: Vec::new(), data_fork: None,
            resource_fork: Some(b"resource".to_vec()), data_fork_size: None,
            resource_fork_size: Some(8), unrecognized_entries: Vec::new(), entries: Vec::new(),
        };
        std::fs::write(path, ctb_io::file::write_apple_single_double(&archive)?)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn capture_companion_mutation_rejects_finish_without_publication() -> Result<()> {
        for replace in [false, true] {
            let temporary = tempfile::tempdir()?;
            let path = temporary.path().join("source");
            let companion = temporary.path().join("._source");
            std::fs::write(&path, b"primary")?;
            apple_companion_fixture(&companion)?;
            let options = AppleReadOptions { read_apple_double_alongside: true, ..Default::default() };
            let mut source = CapturedSource::capture(&path, None, &options, false)?;
            source.hash(&path)?;
            source.entity.identity.relative_path = Some(PathBuf::from("output"));
            let mut payload = source.open(&path)?;
            if replace { std::fs::rename(&companion, temporary.path().join("old"))?; }
            std::fs::write(&companion, b"changed companion")?;
            ensure!(payload.finish().is_err());
            let destination = temporary.path().join("destination");
            let sandbox = ctb_io::file::SandboxableDir::create_or_open(&destination)?;
            ensure!(ctb_io::file::materialize_entity(&source.entity, Some(&mut payload), &sandbox,
                &ctb_io::file::MaterializeOptions { strict_lossless: false, ..Default::default() }).is_err());
            ensure!(!destination.join("output").exists());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn capture_rejects_malformed_and_ambiguous_companions() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("source");
        let companion = temporary.path().join("._source");
        std::fs::write(&path, b"primary")?;
        std::fs::write(&companion, b"malformed")?;
        let options = AppleReadOptions { read_apple_double_alongside: true, read_apple_double_netatalk: true, ..Default::default() };
        ensure!(CapturedSource::capture(&path, None, &options, false).is_err());
        apple_companion_fixture(&companion)?;
        std::fs::create_dir(temporary.path().join(".AppleDouble"))?;
        apple_companion_fixture(&temporary.path().join(".AppleDouble/source"))?;
        let error = CapturedSource::capture(&path, None, &options, false).err().context("Ambiguous companions accepted")?;
        ensure!(error.to_string().contains("Multiple AppleDouble"));
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_cleanup_warning_finalizer_is_idempotent() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("link");
        std::os::unix::fs::symlink("target", &path)?;
        let mut source = CapturedSource::capture(&path, None, &Default::default(), false)?;
        let warning = "injected symlink timestamp cleanup warning".to_owned();
        source.snapshot.retain_cleanup_warnings(vec![warning.clone()])?;
        let mut diagnostics = Vec::new();
        for _ in 0..2 {
            finalize_warnings(&source.snapshot, &mut source.entity, &mut |code, message| {
                diagnostics.push((code.to_owned(), message.to_owned())); Ok(())
            })?;
        }
        ensure!(source.snapshot.cleanup_warnings()?.contains(&warning));
        ensure!(source.entity.metadata.warnings.iter().filter(|message| **message == warning).count() == 1);
        ensure!(diagnostics == [("FileReadWarning".to_owned(), warning)]);
        ensure!(source.snapshot.validate()?.is_empty());
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_final_tree_warning_visitor_retries_failure_without_duplicates() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("source");
        std::fs::create_dir(&path)?;
        let source = CapturedSource::capture(&path, None, &Default::default(), true)?;
        let mut tree = CapturedTree::default();
        tree.retain(&source.snapshot);
        tree.validate()?;
        let mode = std::fs::metadata(&path)?.permissions().mode() ^ 0o100;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))?;
        let cleanup = format!("late cleanup warning: {}", path.display());
        source.snapshot.retain_cleanup_warnings(vec![cleanup.clone()])?;
        tree.validate()?;
        ensure!(tree.visit_pending_warnings(&mut |_, _| bail!("injected persistence failure")).is_err());
        let mut diagnostics = Vec::new();
        for _ in 0..2 {
            tree.validate()?;
            tree.visit_pending_warnings(&mut |code, message| {
                diagnostics.push((code.to_owned(), message.to_owned()));
                Ok(())
            })?;
        }
        ensure!(diagnostics.len() == 2);
        ensure!(diagnostics.iter().any(|(code, message)| code == "SourceModifiedDuringCopy"
            && message.contains(&path.display().to_string())));
        ensure!(diagnostics.contains(&("FileReadWarning".to_owned(), cleanup)));
        std::fs::write(path.join("new-child"), b"changed inventory")?;
        ensure!(tree.validate().is_err(), "BestEffort must not waive inventory changes");
        std::fs::rename(&path, temporary.path().join("old"))?;
        std::fs::create_dir(&path)?;
        ensure!(tree.validate().is_err(), "BestEffort must not waive identity replacement");
        Ok(())
    }

    #[crate::ctb_test]
    fn capture_finish_rejects_replacement_before_materializer_publish() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("source");
        std::fs::write(&path, b"original")?;
        let mut source = CapturedSource::capture(&path, None, &Default::default(), false)?;
        source.hash(&path)?;
        source.entity.identity.relative_path = Some(PathBuf::from("output"));
        let mut payload = source.open(&path)?;
        std::fs::rename(&path, temporary.path().join("old"))?;
        std::fs::write(&path, b"original")?;
        let destination = temporary.path().join("destination");
        let sandbox = ctb_io::file::SandboxableDir::create_or_open(&destination)?;
        ensure!(ctb_io::file::materialize_entity(&source.entity, Some(&mut payload), &sandbox,
            &ctb_io::file::MaterializeOptions { strict_lossless: false, ..Default::default() }).is_err());
        ensure!(!destination.join("output").exists(), "Changed source was published");
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_directory_entries_do_not_follow_symlinks() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source = temporary.path().join("source");
        let outside = temporary.path().join("outside");
        std::fs::create_dir(&source)?;
        std::fs::create_dir(&outside)?;
        std::fs::write(outside.join("hidden"), b"outside")?;
        std::os::unix::fs::symlink(&outside, source.join("link"))?;
        let entries = ctb_io::file::read_dir_safe(&source, &ctb_io::file::TraversalOptions::default())?;
        ensure!(entries.len() == 1);
        let entry = entries.first().context("Missing symlink entry")?;
        ensure!(entry.is_symlink && !entry.is_dir && entry.errors.is_empty());
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_open_rejects_replacement_and_chmod_even_best_effort() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("source");
        std::fs::write(&path, b"original")?;
        let source = CapturedSource::capture(&path, None, &Default::default(), false)?;
        let mode = std::fs::metadata(&path)?.permissions().mode() ^ 0o100;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))?;
        ensure!(source.open(&path).is_err(), "chmod-only changes must fail before open");
        let source = CapturedSource::capture(&path, None, &Default::default(), true)?;
        std::fs::rename(&path, temporary.path().join("old"))?;
        std::fs::write(&path, b"original")?;
        ensure!(source.open(&path).is_err(), "BestEffort must not waive inode replacement");
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_best_effort_retains_changes_without_fidelity_failure() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("source");
        std::fs::write(&path, b"original")?;
        let source = CapturedSource::capture(&path, None, &Default::default(), true)?;
        let mode = std::fs::metadata(&path)?.permissions().mode() ^ 0o100;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))?;
        let mut payload = source.open(&path)?;
        ensure!(payload.finish()?.is_empty());
        ensure!(!source.snapshot.change_warnings()?.is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn capture_recursive_warning_visitor_preserves_attached_entities() -> Result<()> {
        let mut entity = FileEntity::from_vec(Vec::new());
        entity.metadata.push_warning("root".to_owned());
        let mut child = ctb_io::file::AttachedStream::from_data(None,
            ctb_io::file::StreamKind::infer_from_name(b"child"), Vec::new())?;
        let mut grandchild = child.clone();
        grandchild.entity.metadata.push_warning("grandchild".to_owned());
        child.entity.metadata.push_warning("child".to_owned());
        child.entity.streams.push(grandchild);
        entity.streams.push(child);
        let mut messages = Vec::new();
        visit_warnings(&entity, &mut |_, message| { messages.push(message.to_owned()); Ok(()) })?;
        ensure!(messages == ["root", "child", "grandchild"]);
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_hardlinks_commit_only_self_cross_roots_and_source_version() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("source");
        let alias = temporary.path().join("alias");
        std::fs::write(&path, b"original")?;
        std::fs::hard_link(&path, &alias)?;
        let mut source = CapturedSource::capture(&path, None, &Default::default(), false)?;
        source.hash(&path)?;
        let mut alias_source = CapturedSource::capture(&alias, None, &Default::default(), false)?;
        alias_source.hash(&alias)?;
        let mut map = CommittedHardlinks::default();
        ensure!(map.lookup(&source.entity, Path::new("failed/definition"))?.is_none());
        ensure!(map.lookup(&alias_source.entity, Path::new("other/root"))?.is_none());
        map.commit(&source.entity, Path::new("first/root"), Some(&source.snapshot));
        ensure!(map.lookup(&source.entity, Path::new("first/root"))?.is_none());
        ensure!(map.lookup(&alias_source.entity, Path::new("other/root"))? == Some(Path::new("first/root")));
        std::fs::write(&alias, b"modified payload")?;
        ensure!(map.lookup(&alias_source.entity, Path::new("other/root")).is_err());
        Ok(())
    }
}

pub(crate) fn visit_warnings(
    entity: &FileEntity,
    visitor: &mut impl FnMut(&FileEntity, &str) -> Result<()>,
) -> Result<()> {
    for warning in &entity.metadata.warnings {
        visitor(entity, warning)?;
    }
    for stream in &entity.streams {
        visit_warnings(&stream.entity, visitor)?;
    }
    Ok(())
}

pub(crate) fn finalize_warnings(
    snapshot: &SourceSnapshot,
    entity: &mut FileEntity,
    visitor: &mut impl FnMut(&str, &str) -> Result<()>,
) -> Result<()> {
    visit_snapshot_warnings(snapshot, &mut |code, warning| {
        if !entity.metadata.warnings.iter().any(|existing| existing == warning) {
            visitor(code, warning)?;
            entity.metadata.push_warning(warning.to_owned());
        }
        Ok(())
    })
}

fn visit_snapshot_warnings(
    snapshot: &SourceSnapshot,
    visitor: &mut impl FnMut(&str, &str) -> Result<()>,
) -> Result<()> {
    for (code, warnings) in [("SourceModifiedDuringCopy", snapshot.change_warnings()?),
        ("FileReadWarning", snapshot.cleanup_warnings()?)] {
        for warning in warnings {
            visitor(code, &warning)?;
        }
    }
    Ok(())
}