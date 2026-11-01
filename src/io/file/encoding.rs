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

//! Serialization and binary stream encoding for file entities and metadata.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::entity::{FileEntity, FileEntityKind, FileEntityWithPayload};
use crate::payload::PayloadSource;
use crate::streams::{
    AttachedStream, AttachedStreamSource, FilesystemStreamSource,
};
use ctb_formats_dcstring::{
    BinaryPayloadRef, DcBinaryReader, DcBinarySource, DcMixedEncode,
    LazyBinaryPayload, SharedValueContext, validate_and_write_dc_record,
};
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

struct BorrowedReader<'borrow, 'source, T: ?Sized>(
    RefMut<'borrow, &'source mut T>,
);

impl<T: Read + ?Sized> Read for BorrowedReader<'_, '_, T> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(bytes)
    }
}

impl DcBinarySource for Sources<'_> {
    fn open<'source>(
        &'source self,
        payload: &BinaryPayloadRef<'_>,
    ) -> Result<DcBinaryReader<'source>> {
        let BinaryPayloadRef::Source { source_id, len } = payload else {
            bail!("File encoding requires a captured binary source");
        };
        let entry = self
            .entries
            .get(usize::try_from(*source_id)?)
            .context("Unknown binary source")?;
        ensure!(
            *len == entry.size,
            "Binary source size differs from captured metadata"
        );
        let reader: Box<dyn Read + 'source> = match &entry.source {
            Source::Primary(source) => {
                let mut source = source.try_borrow_mut()?;
                ensure!(
                    source.total_size() == entry.size,
                    "Primary source size differs from captured metadata"
                );
                source.seek(SeekFrom::Start(0))?;
                Box::new(BorrowedReader(source))
            }
            Source::Stream(stream) => self.streams.open(stream)?,
            Source::Reader(reader) => {
                Box::new(BorrowedReader(reader.try_borrow_mut()?))
            }
        };
        Ok(DcBinaryReader {
            reader,
            sha256: Some(entry.sha256),
        })
    }
}

impl<'source> Sources<'source> {
    fn add(
        &mut self,
        source: Source<'source>,
        kind: &FileEntityKind,
    ) -> Result<LazyBinaryPayload> {
        let FileEntityKind::Regular { size, sha256, .. } = kind else {
            bail!("Only regular entities can contain binary bodies");
        };
        let source_id = u64::try_from(self.entries.len())?;
        self.entries.push(Entry {
            source,
            size: *size,
            sha256: *sha256,
        });
        Ok(LazyBinaryPayload(BinaryPayloadRef::Source {
            source_id,
            len: *size,
        }))
    }

    fn entity(
        &mut self,
        entity: &'source FileEntity,
        bodies: BodyPolicy,
    ) -> Result<FileEntity> {
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

    fn stream(
        &mut self,
        stream: &'source AttachedStream,
        bodies: BodyPolicy,
    ) -> Result<AttachedStream> {
        let payload = if bodies == BodyPolicy::Include {
            Some(self.add(Source::Stream(stream), &stream.entity.kind)?)
        } else {
            None
        };
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
    ensure!(
        primary.is_none() || bodies == BodyPolicy::Include,
        "Metadata-only encoding cannot accept a primary body"
    );
    let mut sources = Sources {
        entries: Vec::new(),
        streams,
    };
    let payload = if let Some(primary) = primary {
        Some(
            sources
                .add(Source::Primary(RefCell::new(primary)), &entity.kind)?,
        )
    } else {
        None
    };
    let record = FileEntityWithPayload {
        entity: sources.entity(entity, bodies)?,
        payload,
    };
    validate_and_write_dc_record(output, Some(&sources), |writer| {
        record.encode_dc_mixed_to_with_context(writer, context)
    })
}

pub(crate) fn encode_stream(
    stream: &AttachedStream,
    output: &mut dyn Write,
    reader: &mut dyn Read,
    size: u64,
    context: &SharedValueContext,
) -> Result<()> {
    let mut sources = Sources {
        entries: Vec::new(),
        streams: &FilesystemStreamSource,
    };
    let FileEntityKind::Regular { size: expected, .. } = stream.entity.kind
    else {
        bail!("Attached stream is not a regular payload");
    };
    ensure!(size == expected, "Stream source size differs from metadata");
    let payload = Some(
        sources
            .add(Source::Reader(RefCell::new(reader)), &stream.entity.kind)?,
    );
    let record = crate::streams::AttachedStreamWithPayload {
        name: stream.name.clone(),
        kind: stream.kind,
        entity: Box::new(sources.entity(&stream.entity, BodyPolicy::Include)?),
        payload,
    };
    validate_and_write_dc_record(output, Some(&sources), |writer| {
        record.encode_dc_mixed_to_with_context(writer, context)
    })
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
    use crate::streams::{StreamKind, StreamName};
    use ctb_formats_dcstring::{DcMstring, DcProjectionPolicy};

    fn attached(bytes: &[u8]) -> Result<AttachedStream> {
        AttachedStream::from_data(
            Some(StreamName::from_str("user.stream")),
            StreamKind::ExtendedAttribute,
            bytes.to_vec(),
        )
    }

    #[crate::ctb_test]
    fn body_policy_preserves_nested_metadata_and_filesystem_origins()
    -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("stream");
        std::fs::write(&path, b"contents")?;
        let mut stream = AttachedStream::from_file(
            None,
            StreamKind::ExtendedAttribute,
            &path,
        )?;
        stream.entity.streams.push(attached(b"nested")?);
        let mut entity = FileEntity::from_vec(Vec::new());
        entity.streams.push(stream);
        let origin = entity
            .streams
            .first()
            .context("Missing stream")?
            .entity
            .identity
            .origin
            .clone();
        let mut document = DcMstring::new();
        entity.encode_record_to_writer_with_policy(
            &mut document.writer(),
            None,
            BodyPolicy::Omit,
            &SharedValueContext::default(),
        )?;
        let decoded: FileEntityWithPayload =
            document.decode(&DcProjectionPolicy::strict())?;
        ensure!(decoded.payload.is_none());
        let stream = decoded
            .entity
            .streams
            .first()
            .context("Missing decoded stream")?;
        ensure!(
            stream.entity.identity.origin == origin
                && stream.data.is_none()
                && stream.payload.is_none()
        );
        let nested = stream
            .entity
            .streams
            .first()
            .context("Missing nested stream")?;
        ensure!(nested.data.is_none() && nested.payload.is_none());
        let mut stripped = entity.clone();
        stripped.strip_stream_bodies();
        ensure!(
            stripped
                .streams
                .first()
                .context("Missing stripped stream")?
                .entity
                .identity
                .origin
                == origin
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn body_inclusion_rejects_missing_sources_and_corruption_in_both_sinks()
    -> Result<()> {
        let mut entity = FileEntity::from_vec(b"primary".to_vec());
        let mut stream = attached(b"stream")?;
        stream.strip_body();
        entity.streams.push(stream);
        ensure!(
            entity
                .encode_record_to_writer_with_context(
                    &mut Vec::new(),
                    &SharedValueContext::default()
                )
                .is_err()
        );
        for bytes in [
            b"change!".as_slice(),
            b"short".as_slice(),
            b"primary-extra".as_slice(),
        ] {
            let mut source =
                crate::payload::MemoryPayloadSource::new(bytes.to_vec())?;
            let entity = FileEntity::from_vec(b"primary".to_vec());
            ensure!(
                entity
                    .encode_with_payload(&mut DcMstring::new(), &mut source)
                    .is_err()
            );
            let mut source =
                crate::payload::MemoryPayloadSource::new(bytes.to_vec())?;
            ensure!(
                entity
                    .encode_to_writer_with_payload(&mut Vec::new(), &mut source)
                    .is_err()
            );
        }
        Ok(())
    }
}
