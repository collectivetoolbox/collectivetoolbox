#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use crate::entity::{FileEntity, FileEntityKind, FileEntityWithPayload};
use crate::payload::PayloadSource;
use crate::streams::{AttachedStream, AttachedStreamSource, FilesystemStreamSource};
use ctb_formats_dcstring::{BinaryPayloadRef, DcBinaryReader, DcBinarySource, DcMixedEncode, DcWriter, LazyBinaryPayload, SharedValueContext};
use std::cell::{RefCell, RefMut};
use std::io::{Read, SeekFrom, Write};

/// Whether records retain their bodies or deliberately contain metadata only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyPolicy {
    Include,
    Omit,
}

enum Source<'source> {
    Primary(RefCell<&'source mut dyn PayloadSource>),
    Stream(&'source AttachedStream),
    Reader(RefCell<&'source mut dyn Read>),
}

struct Entry<'source> {
    source: Source<'source>,
    size: u64,
    sha256: [u8; 32],
}

struct Sources<'source> {
    entries: Vec<Entry<'source>>,
    streams: &'source dyn AttachedStreamSource,
}

struct BorrowedReader<'borrow, 'source, T: ?Sized>(RefMut<'borrow, &'source mut T>);

impl<T: Read + ?Sized> Read for BorrowedReader<'_, '_, T> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(bytes)
    }
}

impl DcBinarySource for Sources<'_> {
    fn open<'source>(&'source self, payload: &BinaryPayloadRef<'_>) -> Result<DcBinaryReader<'source>> {
        let BinaryPayloadRef::Source { source_id, len } = payload else {
            bail!("File encoding requires a captured binary source");
        };
        let entry = self.entries.get(usize::try_from(*source_id)?).context("Unknown binary source")?;
        ensure!(*len == entry.size, "Binary source size differs from captured metadata");
        let reader: Box<dyn Read + 'source> = match &entry.source {
            Source::Primary(source) => {
                let mut source = source.try_borrow_mut()?;
                ensure!(source.total_size() == entry.size, "Primary source size differs from captured metadata");
                source.seek(SeekFrom::Start(0))?;
                Box::new(BorrowedReader(source))
            }
            Source::Stream(stream) => self.streams.open(stream)?,
            Source::Reader(reader) => Box::new(BorrowedReader(reader.try_borrow_mut()?)),
        };
        Ok(DcBinaryReader { reader, sha256: Some(entry.sha256) })
    }
}

impl<'source> Sources<'source> {
    fn add(&mut self, source: Source<'source>, kind: &FileEntityKind) -> Result<LazyBinaryPayload> {
        let FileEntityKind::Regular { size, sha256, .. } = kind else {
            bail!("Only regular entities can contain binary bodies");
        };
        let source_id = u64::try_from(self.entries.len())?;
        self.entries.push(Entry { source, size: *size, sha256: *sha256 });
        Ok(LazyBinaryPayload(BinaryPayloadRef::Source { source_id, len: *size }))
    }

    fn entity(&mut self, entity: &'source FileEntity, bodies: BodyPolicy) -> Result<FileEntity> {
        let mut streams = Vec::new();
        streams.try_reserve_exact(entity.streams.len())?;
        for stream in &entity.streams {
            streams.push(self.stream(stream, bodies)?);
        }
        Ok(FileEntity {
            identity: entity.identity.clone(),
            metadata: entity.metadata.clone(),
            kind: entity.kind.clone(),
            streams,
        })
    }

    fn stream(&mut self, stream: &'source AttachedStream, bodies: BodyPolicy) -> Result<AttachedStream> {
        let payload = if bodies == BodyPolicy::Include {
            Some(self.add(Source::Stream(stream), &stream.entity.kind)?)
        } else { None };
        Ok(AttachedStream {
            name: stream.name.clone(),
            kind: stream.kind,
            entity: Box::new(self.entity(&stream.entity, bodies)?),
            data: None,
            payload,
        })
    }
}

pub(crate) fn encode_entity(
    entity: &FileEntity,
    output: &mut dyn Write,
    primary: Option<&mut dyn PayloadSource>,
    streams: &dyn AttachedStreamSource,
    bodies: BodyPolicy,
    context: &SharedValueContext,
) -> Result<()> {
    ensure!(primary.is_none() || bodies == BodyPolicy::Include, "Metadata-only encoding cannot accept a primary body");
    let mut sources = Sources { entries: Vec::new(), streams };
    let payload = if let Some(primary) = primary {
        Some(sources.add(Source::Primary(RefCell::new(primary)), &entity.kind)?)
    } else { None };
    let record = FileEntityWithPayload { entity: sources.entity(entity, bodies)?, payload };
    record.encode_dc_mixed_to_with_context(&mut DcWriter::with_binary_source(output, &sources), context)
}

pub(crate) fn encode_stream(
    stream: &AttachedStream,
    output: &mut dyn Write,
    reader: &mut dyn Read,
    size: u64,
    context: &SharedValueContext,
) -> Result<()> {
    let mut sources = Sources { entries: Vec::new(), streams: &FilesystemStreamSource };
    let FileEntityKind::Regular { size: expected, .. } = stream.entity.kind else {
        bail!("Attached stream is not a regular payload");
    };
    ensure!(size == expected, "Stream source size differs from metadata");
    let payload = Some(sources.add(Source::Reader(RefCell::new(reader)), &stream.entity.kind)?);
    let record = crate::streams::AttachedStreamWithPayload {
        name: stream.name.clone(),
        kind: stream.kind,
        entity: Box::new(sources.entity(&stream.entity, BodyPolicy::Include)?),
        payload,
    };
    record.encode_dc_mixed_to_with_context(&mut DcWriter::with_binary_source(output, &sources), context)
}