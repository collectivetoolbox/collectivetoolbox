#[expect(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use crate::journal::{JournalSessionRecord, SESSION_ENVIRONMENT_BINDING};
use ctb_formats_checksum::Sha256Stream;
use ctb_formats_dcdata::dc;
use ctb_formats_dcstring::{
    BinaryPayloadRef, DcDocumentNavigator, DcFrameHeaderInfo, DcMixedEncode,
    DcMstring, DcProjectionPolicy, SharedValueContext,
};
use ctb_io::file::{Extent, FileEntity, FileEntityKind, FileEntityWithPayload, PayloadSource};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

pub mod cli;

pub const ARCHIVE_MAGIC: &[u8; 8] = b"CTBCSCA\x01";

pub struct ArchiveWriter<W: Write> {
    writer: W,
    sequence: u64,
    context: SharedValueContext,
    failed: bool,
}

impl<W: Write> ArchiveWriter<W> {
    pub fn new(mut writer: W, session: &JournalSessionRecord) -> Result<Self> {
        let mut header = DcMstring::new();
        session.encode_dc_mixed(&mut header)?;
        ctb_formats_dcstring::validate_dc_mixed_strict(&header)?;
        writer.write_all(ARCHIVE_MAGIC)?;
        writer.write_all(header.as_bytes())?;
        Ok(Self { writer, sequence: 0, context: session_context(session)?, failed: false })
    }

    /// Continues encoding after the caller verifies and positions the output.
    pub(crate) fn resume(writer: W, archive: &ArchiveReader) -> Result<Self> {
        ensure!(!archive.completed, "Cannot resume a completed archive");
        Ok(Self { writer, sequence: archive.sequence, context: session_context(&archive.session)?, failed: false })
    }

    pub fn write_entity(&mut self, entity: &FileEntity, payload: Option<&mut dyn PayloadSource>) -> Result<()> {
        ensure!(!self.failed, "Archive writer has an uncommitted failed section");
        entity.check_ok()?;
        let next = self.sequence.checked_add(1).context("Archive sequence overflow")?;
        self.failed = true;
        crate::section::write_section(&mut self.writer, next, |mut writer| {
            if let FileEntityKind::Regular { size, sha256, .. } = &entity.kind {
                let source = payload.context("Regular archive member requires file data")?;
                ensure!(source.total_size() == *size, "Source size differs from archived metadata");
                let mut source = VerifiedSource { source, digest: Sha256Stream::new(), bytes: 0 };
                entity.encode_record_to_writer_with_payload_and_context(&mut writer, &mut source, &self.context)?;
                ensure!(source.bytes == *size && source.digest.clone().finalize() == *sha256,
                    "Source contents changed while archiving");
                let warnings = source.finish()?;
                ensure!(warnings.is_empty(), "Source fidelity warnings: {}", warnings.join("; "));
            } else {
                ensure!(payload.is_none(), "Non-regular archive member cannot contain primary data");
                let mut record = DcMstring::new();
                entity.encode_dc_mixed_with_context(&mut record, &self.context)?;
                ctb_formats_dcstring::validate_dc_mixed_strict(&record)?;
                writer.write_all(record.payload_bytes())?;
            }
            Ok(())
        })?;
        self.failed = false;
        self.sequence = next;
        Ok(())
    }

    pub fn finish(mut self) -> Result<W> {
        ensure!(!self.failed, "Cannot complete archive after a failed section");
        let mut completion = DcMstring::new();
        completion.push_char(dc::DC_OPERATION_COMPLETED);
        self.writer.write_all(completion.payload_bytes())?;
        self.writer.flush()?;
        Ok(self.writer)
    }
}

fn session_context(session: &JournalSessionRecord) -> Result<SharedValueContext> {
    let mut context = SharedValueContext::default();
    if let Some(environment) = &session.environment {
        context.insert(SESSION_ENVIRONMENT_BINDING, std::sync::Arc::clone(environment))?;
        context.insert("environment", std::sync::Arc::clone(environment))?;
    }
    Ok(context)
}

struct VerifiedSource<'source> {
    source: &'source mut dyn PayloadSource,
    digest: Sha256Stream,
    bytes: u64,
}

impl Read for VerifiedSource<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = self.source.read(buffer)?;
        self.digest.update(buffer.get(..count).ok_or_else(|| std::io::Error::other("Invalid source read count"))?);
        self.bytes = self.bytes.checked_add(u64::try_from(count).map_err(std::io::Error::other)?)
            .ok_or_else(|| std::io::Error::other("Source byte count overflow"))?;
        Ok(count)
    }
}

impl Seek for VerifiedSource<'_> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        if self.bytes != 0 {
            return Err(std::io::Error::other("Cannot seek after hashing source data"));
        }
        self.source.seek(position)
    }
}

impl PayloadSource for VerifiedSource<'_> {
    fn total_size(&self) -> u64 { self.source.total_size() }
    fn extents(&self) -> &[Extent] { self.source.extents() }
    fn finish(&mut self) -> Result<Vec<String>> { self.source.finish() }
}

pub struct ArchiveReader {
    file: File,
    pub session: JournalSessionRecord,
    pub members: Vec<FileEntityWithPayload>,
    completed: bool,
    sequence: u64,
    pub(crate) valid_length: u64,
}

impl ArchiveReader {
    pub fn new(file: File) -> Result<Self> {
        Self::scan(file, false)
    }

    pub(crate) fn recover(file: File) -> Result<Self> {
        Self::scan(file, true)
    }

    fn scan(mut file: File, recover: bool) -> Result<Self> {
        file.seek(SeekFrom::Start(0))?;
        let mut magic = [0_u8; 8];
        file.read_exact(&mut magic)?;
        ensure!(&magic == ARCHIVE_MAGIC, "Unsupported CSC archive magic or version");
        let mut navigator = DcDocumentNavigator::from_dcmm_file(file.try_clone()?, 8)?;
        let policy = DcProjectionPolicy::strict();
        let stub = navigator.next_sibling_stub(&policy)?.context("Missing archive session")?;
        ensure!(stub.head_dc == dc::DC_BEGIN_OPERATION_SESSION && !stub.is_unclosed(), "Invalid archive session");
        let session: JournalSessionRecord = navigator.decode_frame(&stub, None, &policy)?;
        let mut valid_length = stub.outer_span.end;
        let context = session_context(&session)?;
        let mut members = Vec::new();
        let mut sequence = 0_u64;
        let mut completed = false;
        let scan_policy = if recover { DcProjectionPolicy::forgiving_quiet() } else { policy.clone() };
        loop {
            let stub = match navigator.next_sibling_stub(&scan_policy) {
                Ok(Some(stub)) => stub,
                Ok(None) => break,
                Err(error) if recover && error.downcast_ref::<std::io::Error>()
                    .is_some_and(|error| error.kind() == std::io::ErrorKind::UnexpectedEof) => break,
                Err(error) => return Err(error).context("Invalid archive record"),
            };
            ensure!(!completed, "Unexpected data after archive completion");
            if recover && stub.head_dc == dc::DC_BEGIN_CHECKSUMMED_SECTION
                && stub.is_unclosed() && stub.outer_span.end == navigator.active_span().end {
                break;
            }
            ensure!(!stub.is_unclosed(), "Incomplete archive section");
            if stub.head_dc == dc::DC_OPERATION_COMPLETED {
                let mut marker = DcMstring::new();
                marker.push_char(dc::DC_OPERATION_COMPLETED);
                ensure!(stub.outer_span.len() == u64::try_from(marker.payload_bytes().len())?, "Invalid archive completion marker");
                completed = true;
                valid_length = stub.outer_span.end;
                continue;
            }
            ensure!(stub.head_dc == dc::DC_BEGIN_CHECKSUMMED_SECTION, "Unexpected archive record {}", stub.head_dc);
            let next = sequence.checked_add(1).context("Archive sequence overflow")?;
            ensure!(matches!(stub.header_info, DcFrameHeaderInfo::ChecksummedSection {
                sequence_id: Some(actual), digest_valid: true, ..
            } if actual == next), "Invalid archive section sequence or checksum");
            navigator.enter_frame(&stub)?;
            let identifier = navigator.next_sibling_stub(&policy)?.context("Missing archive sequence")?;
            ensure!(identifier.head_dc == dc::DC_SEQUENCE_IDENTIFIER, "Invalid archive sequence record");
            let member = navigator.next_sibling_stub(&policy)?.context("Empty archive section")?;
            ensure!(member.head_dc == ctb_formats_dcstring::DcChar::from_short(317), "Archive section does not contain a file");
            let record: FileEntityWithPayload = navigator.decode_frame(&member, Some(&context), &policy)?;
            ensure!(navigator.next_sibling_stub(&policy)?.is_none(), "Archive section contains extra records");
            navigator.exit_frame()?;
            members.push(record);
            sequence = next;
            valid_length = stub.outer_span.end;
        }
        ensure!(completed || recover, "Archive is incomplete (missing completion marker)");
        Ok(Self { file, session, members, completed, sequence, valid_length })
    }

    pub fn payload(&self, member: &FileEntityWithPayload) -> Result<ArchivePayload> {
        let FileEntityKind::Regular { size, extents, .. } = &member.entity.kind else {
            bail!("Only regular files have primary archive payloads");
        };
        let (file, resident, start, length) = match member.payload.as_ref().map(|value| &value.0) {
            Some(BinaryPayloadRef::FileSpan { file_offset, len, .. }) => {
                (Some(self.file.try_clone()?), Vec::new(), *file_offset, *len)
            }
            Some(payload) => (None, payload.as_resident_bytes().context("Missing resident binary data")?.to_vec(), 0, payload.len()),
            None => {
                ensure!(*size == 0, "Nonempty member is missing file data");
                (None, Vec::new(), 0, 0)
            }
        };
        ensure!(length == *size, "Archive payload size mismatch");
        ensure!(start.checked_add(length).is_some(), "Archive payload offset overflow");
        Ok(ArchivePayload { file, resident, start, length, position: 0, extents: extents.clone() })
    }

    pub fn verify(&self) -> Result<()> {
        ensure!(self.completed, "Archive is incomplete");
        self.verify_contents()
    }

    fn verify_contents(&self) -> Result<()> {
        for member in &self.members {
            member.entity.check_ok()?;
            if let FileEntityKind::Regular { size, sha256, is_sparse, extents } = &member.entity.kind {
                let mut end = 0_u64;
                for extent in extents {
                    ensure!(extent.offset() == end && extent.length() > 0, "Invalid archive extent map");
                    end = end.checked_add(extent.length()).context("Archive extent overflow")?;
                }
                ensure!(end == *size, "Archive extents do not cover the logical file");
                ensure!(*is_sparse == extents.iter().any(Extent::is_hole), "Archive sparse flag mismatch");
                let mut payload = self.payload(member)?;
                if let Some(BinaryPayloadRef::FileSpan { checksum: Some(checksum), .. }) = member.payload.as_ref().map(|value| &value.0) {
                    ensure!(checksum.matches_reader(&mut payload, *size)?, "Archive binary encapsulation checksum mismatch");
                    payload.seek(SeekFrom::Start(0))?;
                }
                let mut digest = Sha256Stream::new();
                ensure!(std::io::copy(&mut payload, &mut digest)? == *size, "Truncated archive payload");
                ensure!(digest.finalize() == *sha256, "Archive file checksum mismatch");
            } else {
                ensure!(member.payload.is_none(), "Non-regular member has primary file data");
            }
            verify_streams(&member.entity)?;
        }
        Ok(())
    }
}

fn verify_streams(entity: &FileEntity) -> Result<()> {
    for stream in &entity.streams {
        if let FileEntityKind::Regular { size, sha256, .. } = &stream.entity.kind {
            let bytes = stream.data.as_deref().context("Archive is missing attached stream data")?;
            ensure!(u64::try_from(bytes.len())? == *size && ctb_formats_checksum::sha256(bytes) == *sha256,
                "Archive attached stream checksum or length mismatch");
        }
        verify_streams(&stream.entity)?;
    }
    Ok(())
}

pub struct ArchivePayload {
    file: Option<File>,
    resident: Vec<u8>,
    start: u64,
    length: u64,
    position: u64,
    extents: Vec<Extent>,
}

impl Read for ArchivePayload {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.length.checked_sub(self.position)
            .ok_or_else(|| std::io::Error::other("Archive reader position out of bounds"))?;
        let count = usize::try_from(remaining.min(u64::try_from(buffer.len()).map_err(std::io::Error::other)?))
            .map_err(std::io::Error::other)?;
        let target = buffer.get_mut(..count).ok_or_else(|| std::io::Error::other("Invalid archive read buffer"))?;
        let read = if let Some(file) = &mut self.file {
            file.seek(SeekFrom::Start(self.start.checked_add(self.position)
                .ok_or_else(|| std::io::Error::other("Archive offset overflow"))?))?;
            file.read(target)?
        } else {
            let position = usize::try_from(self.position).map_err(std::io::Error::other)?;
            let mut source = self.resident.get(position..).ok_or_else(|| std::io::Error::other("Archive inline offset out of bounds"))?;
            source.read(target)?
        };
        self.position = self.position.checked_add(u64::try_from(read).map_err(std::io::Error::other)?)
            .ok_or_else(|| std::io::Error::other("Archive position overflow"))?;
        Ok(read)
    }
}

impl Seek for ArchivePayload {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        let next = match position {
            SeekFrom::Start(offset) => Some(offset),
            SeekFrom::Current(offset) => self.position.checked_add_signed(offset),
            SeekFrom::End(offset) => self.length.checked_add_signed(offset),
        }.filter(|offset| *offset <= self.length)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Seek outside archive member"))?;
        self.position = next;
        Ok(next)
    }
}

impl PayloadSource for ArchivePayload {
    fn total_size(&self) -> u64 { self.length }
    fn extents(&self) -> &[Extent] { &self.extents }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctb_io::file::MemoryPayloadSource;

    #[crate::ctb_test]
    fn archive_streaming_roundtrip_and_incomplete_rejection() -> Result<()> {
        let session = JournalSessionRecord {
            sources: vec!["source".into()], destination: "archive.cscarchive".into(),
            origin_platform: u32::from(crate::journal::current_platform()), environment: None,
        };
        let data = vec![0xA5; 131_072];
        let entity = FileEntity::from_vec(data.clone());
        let mut source = MemoryPayloadSource::new(data.clone())?;
        let mut writer = ArchiveWriter::new(Vec::new(), &session)?;
        writer.write_entity(&entity, Some(&mut source))?;
        let bytes = writer.finish()?;
        let mut file = tempfile::tempfile()?;
        file.write_all(&bytes)?;
        let archive = ArchiveReader::new(file.try_clone()?)?;
        archive.verify()?;
        let member = archive.members.first().context("Missing member")?;
        ensure!(member.entity == entity);
        let mut payload = archive.payload(member)?;
        ensure!(payload.seek(SeekFrom::End(1)).is_err());
        let mut decoded = Vec::new();
        payload.read_to_end(&mut decoded)?;
        ensure!(decoded == data);
        file.set_len(u64::try_from(bytes.len())?.checked_sub(1).context("Missing completion byte")?)?;
        ensure!(ArchiveReader::new(file).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_source_mismatch_cannot_complete() -> Result<()> {
        let session = JournalSessionRecord {
            sources: Vec::new(), destination: "-".into(),
            origin_platform: u32::from(crate::journal::current_platform()), environment: None,
        };
        let mut writer = ArchiveWriter::new(Vec::new(), &session)?;
        let entity = FileEntity::from_vec(b"expected".to_vec());
        let mut source = MemoryPayloadSource::new(b"modified".to_vec())?;
        ensure!(writer.write_entity(&entity, Some(&mut source)).is_err());
        ensure!(writer.finish().is_err());
        Ok(())
    }

    struct FlushFailure(Vec<u8>);

    impl Write for FlushFailure {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "injected output failure"))
        }
    }

    #[crate::ctb_test]
    fn archive_forward_only_output_flush_failure_is_an_error() -> Result<()> {
        let session = JournalSessionRecord {
            sources: Vec::new(), destination: "-".into(),
            origin_platform: u32::from(crate::journal::current_platform()), environment: None,
        };
        let writer = ArchiveWriter::new(FlushFailure(Vec::new()), &session)?;
        ensure!(writer.finish().is_err(), "Broken stdout reported success");
        Ok(())
    }
}