#[expect(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use crate::journal::{
    JournalErrorRecord, JournalErrorRecordPayload, JournalSessionRecord,
    JournalWarningRecord, JournalWarningRecordPayload, SESSION_ENVIRONMENT_BINDING,
};
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
    source_failed: bool,
    sink_failed: bool,
}

impl<W: Write> ArchiveWriter<W> {
    pub fn new(mut writer: W, session: &JournalSessionRecord) -> Result<Self> {
        let mut header = DcMstring::new();
        session.encode_dc_mixed(&mut header)?;
        ctb_formats_dcstring::validate_dc_mixed_strict(&header)?;
        writer.write_all(ARCHIVE_MAGIC)?;
        writer.write_all(header.as_bytes())?;
        Ok(Self { writer, sequence: 0, context: session_context(session)?, failed: false, source_failed: false, sink_failed: false })
    }

    /// Continues encoding after the caller verifies and positions the output.
    pub(crate) fn resume(writer: W, archive: &ArchiveReader) -> Result<Self> {
        ensure!(!archive.completed, "Cannot resume a completed archive");
        ensure!(!archive.has_diagnostics(), "Cannot resume an archive containing capture diagnostics");
        for member in &archive.members {
            ensure_clean_metadata(&member.entity)?;
        }
        Ok(Self { writer, sequence: archive.sequence, context: session_context(&archive.session)?, failed: false, source_failed: false, sink_failed: false })
    }

    pub fn write_entity(&mut self, entity: &FileEntity, payload: Option<&mut dyn PayloadSource>) -> Result<()> {
        ensure!(!self.failed, "Archive writer has an uncommitted failed section");
        ensure_clean_metadata(entity)?;
        if let FileEntityKind::Regular { size, .. } = &entity.kind {
            let source = payload.as_ref().context("Regular archive member requires file data")?;
            ensure!(source.total_size() == *size, "Source size differs from archived metadata");
        } else {
            ensure!(payload.is_none(), "Non-regular archive member cannot contain primary data");
        }
        let mut source_failed = false;
        let result = self.write_record(|mut writer, context| {
            if let FileEntityKind::Regular { size, sha256, .. } = &entity.kind {
                let source = payload.context("Regular archive member requires file data")?;
                let mut source = VerifiedSource {
                    source, digest: Sha256Stream::new(), bytes: 0, size: *size, failed: &mut source_failed,
                };
                entity.encode_record_to_writer_with_payload_and_context(&mut writer, &mut source, context)?;
                if source.bytes != *size || source.digest.clone().finalize() != *sha256 {
                    *source.failed = true;
                    bail!("Source contents changed while archiving");
                }
                let warnings = source.finish()?;
                if !warnings.is_empty() {
                    *source.failed = true;
                    bail!("Source fidelity warnings: {}", warnings.join("; "));
                }
            } else {
                ensure!(payload.is_none(), "Non-regular archive member cannot contain primary data");
                let mut record = DcMstring::new();
                entity.encode_dc_mixed_with_context(&mut record, context)?;
                ctb_formats_dcstring::validate_dc_mixed_strict(&record)?;
                writer.write_all(record.payload_bytes())?;
            }
            Ok(())
        });
        self.source_failed = result.is_err() && source_failed;
        result
    }

    pub fn record_error(&mut self, error: &JournalErrorRecord) -> Result<()> {
        self.write_record(|writer, _| {
            let mut record = DcMstring::new();
            JournalErrorRecordPayload::from_record(error)?.encode_dc_mixed(&mut record)?;
            ctb_formats_dcstring::validate_dc_mixed_strict(&record)?;
            writer.write_all(record.payload_bytes())?;
            Ok(())
        })
    }

    pub fn record_warning(&mut self, warning: &JournalWarningRecord) -> Result<()> {
        self.write_record(|writer, _| {
            let mut record = DcMstring::new();
            JournalWarningRecordPayload::from_record(warning).encode_dc_mixed(&mut record)?;
            ctb_formats_dcstring::validate_dc_mixed_strict(&record)?;
            writer.write_all(record.payload_bytes())?;
            Ok(())
        })
    }

    fn write_record(&mut self, encode: impl FnOnce(&mut dyn Write, &SharedValueContext) -> Result<()>) -> Result<()> {
        ensure!(!self.failed, "Archive writer has an uncommitted failed section");
        self.failed = true;
        self.source_failed = false;
        let next = self.sequence.checked_add(1).context("Archive sequence overflow")?;
        let mut writer = ObservedSink { writer: &mut self.writer, failed: &mut self.sink_failed };
        crate::section::write_section(&mut writer, next, |mut writer| {
            encode(&mut writer, &self.context)
        })?;
        self.failed = false;
        self.sequence = next;
        Ok(())
    }

    pub(crate) fn is_failed(&self) -> bool {
        self.failed
    }

    /// Reports whether only an observed member source failure needs rollback.
    pub(crate) fn can_rollback_source_failure(&self) -> bool {
        self.failed && self.source_failed && !self.sink_failed
    }

    /// The caller must discard the failed section and restore the output position.
    /// A failed callback permanently disables this writer, even if retried.
    pub(crate) fn rollback_source_failure(&mut self, rollback: impl FnOnce(&mut W) -> Result<()>) -> Result<()> {
        ensure!(self.can_rollback_source_failure(), "Archive failure is not source-only");
        self.sink_failed = true;
        rollback(&mut self.writer)?;
        self.failed = false;
        self.source_failed = false;
        self.sink_failed = false;
        Ok(())
    }

    pub fn finish(mut self) -> Result<W> {
        ensure!(!self.failed, "Cannot complete archive after a failed section");
        let mut completion = DcMstring::new();
        completion.push_char(dc::DC_OPERATION_COMPLETED);
        let mut writer = ObservedSink { writer: &mut self.writer, failed: &mut self.sink_failed };
        writer.write_all(completion.payload_bytes())?;
        writer.flush()?;
        Ok(self.writer)
    }
}

struct ObservedSink<'writer, W> {
    writer: &'writer mut W,
    failed: &'writer mut bool,
}

impl<W: Write> Write for ObservedSink<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        match self.writer.write(bytes) {
            Ok(0) if !bytes.is_empty() => {
                *self.failed = true;
                Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "Archive sink accepted no bytes"))
            }
            Ok(count) if count > bytes.len() => {
                *self.failed = true;
                Err(std::io::Error::other("Invalid archive sink write count"))
            }
            Err(error) => {
                if error.kind() != std::io::ErrorKind::Interrupted {
                    *self.failed = true;
                }
                Err(error)
            }
            result => result,
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let result = self.writer.flush();
        if result.is_err() {
            *self.failed = true;
        }
        result
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
    size: u64,
    failed: &'source mut bool,
}

impl Read for VerifiedSource<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = match self.source.read(buffer) {
            Ok(0) if !buffer.is_empty() && self.bytes < self.size => {
                *self.failed = true;
                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Archive source ended before its declared size"));
            }
            Ok(count) if count > buffer.len() => {
                return Err(std::io::Error::other("Invalid source read count"));
            }
            Ok(count) => count,
            Err(error) => {
                *self.failed = true;
                return Err(error);
            }
        };
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
        let result = self.source.seek(position);
        if result.is_err() {
            *self.failed = true;
        }
        result
    }
}

impl PayloadSource for VerifiedSource<'_> {
    fn total_size(&self) -> u64 { self.size }
    fn extents(&self) -> &[Extent] { self.source.extents() }
    fn finish(&mut self) -> Result<Vec<String>> {
        let result = self.source.finish();
        if result.is_err() {
            *self.failed = true;
        }
        result
    }
}

pub struct ArchiveReader {
    file: File,
    pub session: JournalSessionRecord,
    pub members: Vec<FileEntityWithPayload>,
    pub errors: Vec<JournalErrorRecord>,
    pub warnings: Vec<JournalWarningRecord>,
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
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut sequence = 0_u64;
        let mut completed = false;
        let scan_policy = if recover { DcProjectionPolicy::forgiving_quiet() } else { policy.clone() };
        loop {
            let stub = match navigator.next_sibling_stub(&scan_policy) {
                Ok(Some(stub)) => stub,
                Ok(None) => break,
                Err(error) if recover && !completed && error.downcast_ref::<std::io::Error>()
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
            match member.head_dc {
                head if head == ctb_formats_dcstring::DcChar::from_short(317) => {
                    members.push(navigator.decode_frame::<FileEntityWithPayload>(&member, Some(&context), &policy)?);
                }
                head if head == dc::DC_BEGIN_ERROR_RECORD => {
                    let record: JournalErrorRecordPayload = navigator.decode_frame(&member, Some(&context), &policy)?;
                    errors.push(record.to_record(&policy)?);
                }
                head if head == dc::DC_BEGIN_WARNING_RECORD => {
                    let record: JournalWarningRecordPayload = navigator.decode_frame(&member, Some(&context), &policy)?;
                    warnings.push(record.to_record());
                }
                head => bail!("Unknown archive section record {head}"),
            }
            ensure!(navigator.next_sibling_stub(&policy)?.is_none(), "Archive section contains extra records");
            navigator.exit_frame()?;
            sequence = next;
            valid_length = stub.outer_span.end;
        }
        ensure!(!completed || valid_length == navigator.active_span().end,
            "Unexpected bytes after archive completion");
        ensure!(completed || recover, "Archive is incomplete (missing completion marker)");
        Ok(Self { file, session, members, errors, warnings, completed, sequence, valid_length })
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

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn has_diagnostics(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }

    /// Checks completion and structural/payload integrity, not capture success
    /// or authenticity.
    pub fn verify_integrity(&self) -> Result<()> {
        ensure!(self.completed, "Archive is incomplete");
        self.verify_payloads()
    }

    fn verify_contents(&self) -> Result<()> {
        ensure!(!self.has_diagnostics(), "Archive capture has {} errors and {} warnings", self.errors.len(), self.warnings.len());
        for member in &self.members {
            ensure_clean_metadata(&member.entity)?;
        }
        self.verify_payloads()
    }

    fn verify_payloads(&self) -> Result<()> {
        for member in &self.members {
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

fn ensure_clean_metadata(entity: &FileEntity) -> Result<()> {
    entity.check_ok()?;
    ensure!(entity.metadata.warnings.is_empty(), "Archive member has semantic metadata warnings: {}", entity.metadata.warnings.join("; "));
    for stream in &entity.streams {
        ensure_clean_metadata(&stream.entity)?;
    }
    Ok(())
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
        #[crate::ctb_test]
        fn archive_rejects_truncated_records_after_completion() -> Result<()> {
            let session = JournalSessionRecord {
                sources: Vec::new(), destination: "archive.cscarchive".into(),
                origin_platform: u32::from(crate::journal::current_platform()), environment: None,
            };
            let bytes = ArchiveWriter::new(Vec::new(), &session)?.finish()?;
            let mut marker = DcMstring::new();
            marker.push_char(dc::DC_BEGIN_CHECKSUMMED_SECTION);
            for length in 1..=marker.payload_bytes().len() {
                let mut file = tempfile::tempfile()?;
                file.write_all(&bytes)?;
                file.write_all(marker.payload_bytes().get(..length).context("Invalid test marker prefix")?)?;
                ensure!(ArchiveReader::recover(file.try_clone()?).is_err(),
                    "Recovery accepted trailing marker prefix of {length} bytes");
                ensure!(ArchiveReader::new(file).is_err(), "Strict read accepted trailing marker bytes");
            }
            Ok(())
        }

    use super::*;
    use ctb_io::file::MemoryPayloadSource;

    fn diagnostic_writer() -> Result<ArchiveWriter<Vec<u8>>> {
        ArchiveWriter::new(Vec::new(), &JournalSessionRecord {
            sources: Vec::new(), destination: "-".into(),
            origin_platform: u32::from(crate::journal::current_platform()), environment: None,
        })
    }

    fn archive_file(bytes: &[u8]) -> Result<File> {
        let mut file = tempfile::tempfile()?;
        file.write_all(bytes)?;
        Ok(file)
    }

    fn error_record() -> JournalErrorRecord {
        let path = std::path::PathBuf::from("unread/source");
        JournalErrorRecord {
            raw_path: path.as_os_str().as_encoded_bytes().to_vec(), path,
            stage: crate::journal::JournalErrorStage::PayloadRead,
            error_message: "injected diagnostic failure".into(), os_error: Some(-5), timestamp_sec: -17,
        }
    }

    fn warning_record() -> JournalWarningRecord {
        JournalWarningRecord {
            code: "capture-caveat".into(), message: "injected warning: retained detail".into(), timestamp_sec: -23,
        }
    }

    #[crate::ctb_test]
    fn archive_diagnostics_roundtrip_integrity_and_resume_refusal() -> Result<()> {
        let mut error = error_record();
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            error.path = std::ffi::OsString::from_vec(b"unread/\xff".to_vec()).into();
            error.raw_path = error.path.as_os_str().as_encoded_bytes().to_vec();
        }
        let warning = warning_record();
        let mut writer = diagnostic_writer()?;
        writer.record_error(&error)?;
        writer.record_warning(&warning)?;
        let entity = FileEntity::from_vec(b"good".to_vec());
        let mut source = MemoryPayloadSource::new(b"good".to_vec())?;
        writer.write_entity(&entity, Some(&mut source))?;
        ensure!(writer.sequence == 3 && !writer.is_failed());
        let prefix = writer.writer.clone();
        let recovered = ArchiveReader::recover(archive_file(&prefix)?)?;
        ensure!(!recovered.is_completed() && recovered.sequence == 3);
        ensure!(recovered.verify_contents().is_err());
        ensure!(ArchiveWriter::resume(Vec::new(), &recovered).is_err());
        let bytes = writer.finish()?;
        let archive = ArchiveReader::new(archive_file(&bytes)?)?;
        ensure!(archive.is_completed() && archive.has_diagnostics());
        ensure!(archive.errors == vec![error] && archive.warnings == vec![warning]);
        ensure!(archive.members.len() == 1 && archive.sequence == 3);
        archive.verify_integrity()?;
        ensure!(archive.verify().is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_warning_only_is_not_clean() -> Result<()> {
        let mut writer = diagnostic_writer()?;
        writer.record_warning(&warning_record())?;
        let archive = ArchiveReader::new(archive_file(&writer.finish()?)?)?;
        archive.verify_integrity()?;
        ensure!(archive.errors.is_empty() && archive.warnings.len() == 1);
        ensure!(archive.verify().is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_committed_invalid_diagnostic_decode_is_not_recoverable() -> Result<()> {
        let mut writer = diagnostic_writer()?;
        writer.write_record(|output, _| {
            let payload = JournalErrorRecordPayload::new("source".into(), 256_u64 << 32, "bad stage".into(), 0);
            let mut record = DcMstring::new();
            payload.encode_dc_mixed(&mut record)?;
            output.write_all(record.payload_bytes())?;
            Ok(())
        })?;
        let bytes = writer.finish()?;
        ensure!(ArchiveReader::new(archive_file(&bytes)?).is_err());
        ensure!(ArchiveReader::recover(archive_file(&bytes)?).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_legacy_metadata_error_is_capture_failure_not_corruption() -> Result<()> {
        let mut entity = FileEntity::from_vec(Vec::new());
        entity.metadata.errors.push("unread metadata".into());
        let mut writer = diagnostic_writer()?;
        writer.write_record(|output, _| {
            let mut record = DcMstring::new();
            entity.encode_dc_mixed(&mut record)?;
            output.write_all(record.payload_bytes())?;
            Ok(())
        })?;
        let archive = ArchiveReader::new(archive_file(&writer.finish()?)?)?;
        archive.verify_integrity()?;
        ensure!(archive.verify().is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_diagnostic_corruption_is_not_recoverable() -> Result<()> {
        let mut writer = diagnostic_writer()?;
        writer.record_error(&error_record())?;
        let mut bytes = writer.finish()?;
        let position = bytes.windows(b"injected".len()).position(|part| part == b"injected")
            .context("Missing diagnostic message")?;
        *bytes.get_mut(position).context("Missing corruption byte")? = b'X';
        ensure!(ArchiveReader::new(archive_file(&bytes)?).is_err());
        ensure!(ArchiveReader::recover(archive_file(&bytes)?).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_closed_unknown_extra_and_invalid_sequence_are_rejected() -> Result<()> {
        for sequence in [0, 2] {
            let mut writer = diagnostic_writer()?;
            crate::section::write_section(&mut writer.writer, sequence, |output| {
                let mut record = DcMstring::new();
                JournalWarningRecordPayload::from_record(&warning_record()).encode_dc_mixed(&mut record)?;
                output.write_all(record.payload_bytes())?;
                Ok(())
            })?;
            let bytes = writer.finish()?;
            ensure!(ArchiveReader::new(archive_file(&bytes)?).is_err());
            ensure!(ArchiveReader::recover(archive_file(&bytes)?).is_err());
        }
        for extra in [false, true] {
            let mut writer = diagnostic_writer()?;
            let session = JournalSessionRecord {
                sources: Vec::new(), destination: "-".into(), origin_platform: 1, environment: None,
            };
            writer.write_record(|output, _| {
                let mut record = DcMstring::new();
                if extra {
                    JournalWarningRecordPayload::from_record(&warning_record()).encode_dc_mixed(&mut record)?;
                }
                session.encode_dc_mixed(&mut record)?;
                output.write_all(record.payload_bytes())?;
                Ok(())
            })?;
            let bytes = writer.finish()?;
            ensure!(ArchiveReader::new(archive_file(&bytes)?).is_err());
            ensure!(ArchiveReader::recover(archive_file(&bytes)?).is_err());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_metadata_warnings_rejected_before_emission_and_on_legacy_read() -> Result<()> {
        for (nested, is_error) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut warned = FileEntity::from_vec(Vec::new());
            if is_error {
                warned.metadata.errors.push("unread metadata detail".into());
            } else {
                warned.metadata.warnings.push("unread metadata detail".into());
            }
            let entity = if nested {
                let mut entity = FileEntity::from_vec(Vec::new());
                entity.streams.push(ctb_io::file::AttachedStream {
                    name: None, kind: ctb_io::file::StreamKind::MacOsResourceFork,
                    entity: Box::new(warned), data: Some(Vec::new()),
                });
                entity
            } else { warned };
            let mut writer = diagnostic_writer()?;
            let before = writer.writer.clone();
            let error = writer.write_entity(&entity, None).expect_err("Metadata warning accepted");
            ensure!(error.to_string().contains("unread metadata detail"));
            ensure!(writer.writer == before && writer.sequence == 0 && !writer.is_failed());
            ensure!(!writer.can_rollback_source_failure());
            writer.record_warning(&warning_record())?;
            ArchiveReader::new(archive_file(&writer.finish()?)?)?.verify_integrity()?;

            let mut legacy = diagnostic_writer()?;
            legacy.write_record(|output, context| {
                let mut record = DcMstring::new();
                entity.encode_dc_mixed_with_context(&mut record, context)?;
                output.write_all(record.payload_bytes())?;
                Ok(())
            })?;
            let prefix = legacy.writer.clone();
            let recovered = ArchiveReader::recover(archive_file(&prefix)?)?;
            ensure!(recovered.verify_contents().is_err());
            ensure!(ArchiveWriter::resume(Vec::new(), &recovered).is_err());
            let archive = ArchiveReader::new(archive_file(&legacy.finish()?)?)?;
            archive.verify_integrity()?;
            ensure!(archive.verify().is_err());
        }
        Ok(())
    }

    struct FailingSink {
        bytes: Vec<u8>,
        remaining: usize,
        zero: bool,
        interrupted: bool,
    }

    impl Write for FailingSink {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.interrupted {
                self.interrupted = false;
                return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
            }
            if self.remaining == 0 {
                if self.zero {
                    return Ok(0);
                }
                return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "injected sink failure"));
            }
            let count = self.remaining.min(bytes.len());
            self.bytes.extend_from_slice(bytes.get(..count).expect("Bounded test write"));
            self.remaining = self.remaining.saturating_sub(count);
            Ok(count)
        }

        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }

    #[crate::ctb_test]
    fn archive_failed_diagnostics_poison_without_completion() -> Result<()> {
        for warning in [false, true] {
            let initial = diagnostic_writer()?;
            let mut writer = ArchiveWriter {
                writer: FailingSink { bytes: initial.writer, remaining: 9, zero: false, interrupted: false },
                sequence: 0, context: initial.context, failed: false,
                source_failed: false, sink_failed: false,
            };
            let result = if warning { writer.record_warning(&warning_record()) }
                else { writer.record_error(&error_record()) };
            ensure!(result.is_err() && writer.is_failed() && writer.sequence == 0);
            ensure!(!writer.can_rollback_source_failure());
            let before = writer.writer.bytes.clone();
            ensure!(writer.record_error(&error_record()).is_err());
            ensure!(writer.record_warning(&warning_record()).is_err());
            ensure!(writer.write_entity(&FileEntity::from_vec(Vec::new()), None).is_err());
            ensure!(writer.writer.bytes == before);
            let recovered = ArchiveReader::recover(archive_file(&before)?)?;
            ensure!(!recovered.is_completed() && recovered.sequence == 0);
            ensure!(ArchiveReader::new(archive_file(&before)?).is_err());
            ensure!(writer.finish().is_err());
        }
        let mut writer = diagnostic_writer()?;
        let mut error = error_record();
        error.os_error = Some(0);
        ensure!(writer.record_error(&error).is_err());
        ensure!(writer.is_failed() && writer.sequence == 0);
        ensure!(!writer.can_rollback_source_failure());
        ensure!(writer.record_warning(&warning_record()).is_err());
        ensure!(writer.finish().is_err());
        Ok(())
    }

    #[derive(Clone, Copy)]
    enum SourceFault { Read, Seek, Finish, Warning, Short, Hash }

    struct FaultySource {
        source: MemoryPayloadSource,
        fault: SourceFault,
    }

    impl FaultySource {
        fn new(fault: SourceFault) -> Result<Self> {
            let data = if matches!(fault, SourceFault::Hash) { b"modified" } else { b"expected" };
            Ok(Self { source: MemoryPayloadSource::new(data.to_vec())?, fault })
        }
    }

    impl Read for FaultySource {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            match self.fault {
                SourceFault::Read => Err(std::io::Error::other("injected source read failure")),
                SourceFault::Short => Ok(0),
                _ => self.source.read(buffer),
            }
        }
    }

    impl Seek for FaultySource {
        fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
            if matches!(self.fault, SourceFault::Seek) {
                return Err(std::io::Error::other("injected source seek failure"));
            }
            self.source.seek(position)
        }
    }

    impl PayloadSource for FaultySource {
        fn total_size(&self) -> u64 { self.source.total_size() }
        fn extents(&self) -> &[Extent] { self.source.extents() }
        fn finish(&mut self) -> Result<Vec<String>> {
            match self.fault {
                SourceFault::Finish => bail!("injected source finish failure"),
                SourceFault::Warning => Ok(vec!["injected source fidelity warning".into()]),
                _ => Ok(Vec::new()),
            }
        }
    }

    #[crate::ctb_test]
    fn archive_source_failures_allow_cursor_rollback_and_continuation() -> Result<()> {
        for fault in [SourceFault::Read, SourceFault::Seek, SourceFault::Finish,
            SourceFault::Warning, SourceFault::Short, SourceFault::Hash] {
            let initial = diagnostic_writer()?;
            let mut writer = ArchiveWriter {
                writer: std::io::Cursor::new(initial.writer), context: initial.context,
                sequence: 0, failed: false, source_failed: false, sink_failed: false,
            };
            writer.writer.seek(SeekFrom::End(0))?;
            writer.record_warning(&warning_record())?;
            let boundary = writer.writer.position();
            let entity = FileEntity::from_vec(b"expected".to_vec());
            let mut source = FaultySource::new(fault)?;
            let error = writer.write_entity(&entity, Some(&mut source)).expect_err("Source fault accepted");
            if matches!(fault, SourceFault::Short) {
                ensure!(error.downcast_ref::<std::io::Error>()
                    .is_some_and(|error| error.kind() == std::io::ErrorKind::UnexpectedEof));
            }
            ensure!(writer.is_failed() && writer.can_rollback_source_failure() && writer.sequence == 1);
            let partial = writer.writer.get_ref().clone();
            let recovered = ArchiveReader::recover(archive_file(&partial)?)?;
            ensure!(recovered.sequence == 1 && recovered.members.is_empty() && !recovered.is_completed());
            ensure!(ArchiveReader::new(archive_file(&partial)?).is_err());
            ensure!(writer.record_error(&error_record()).is_err());
            ensure!(writer.can_rollback_source_failure());
            writer.rollback_source_failure(|output| {
                output.get_mut().truncate(usize::try_from(boundary)?);
                output.set_position(boundary);
                Ok(())
            })?;
            ensure!(!writer.is_failed() && !writer.can_rollback_source_failure() && writer.sequence == 1);
            writer.record_error(&error_record())?;
            let mut source = MemoryPayloadSource::new(b"expected".to_vec())?;
            writer.write_entity(&entity, Some(&mut source))?;
            let bytes = writer.finish()?.into_inner();
            let archive = ArchiveReader::new(archive_file(&bytes)?)?;
            archive.verify_integrity()?;
            ensure!(archive.sequence == 3 && archive.members.len() == 1 && archive.errors.len() == 1);
            ensure!(archive.verify().is_err());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_source_rollback_file_callback_and_failed_callback() -> Result<()> {
        let session = JournalSessionRecord {
            sources: Vec::new(), destination: "-".into(), origin_platform: 1, environment: None,
        };
        for fail_rollback in [false, true] {
            let mut writer = ArchiveWriter::new(tempfile::tempfile()?, &session)?;
            let boundary = writer.writer.stream_position()?;
            let entity = FileEntity::from_vec(b"expected".to_vec());
            let mut source = FaultySource::new(SourceFault::Finish)?;
            ensure!(writer.write_entity(&entity, Some(&mut source)).is_err());
            ensure!(writer.can_rollback_source_failure());
            let result = writer.rollback_source_failure(|output| {
                output.set_len(boundary)?;
                if fail_rollback { bail!("injected rollback failure"); }
                output.seek(SeekFrom::Start(boundary))?;
                Ok(())
            });
            ensure!(writer.sequence == 0);
            if fail_rollback {
                ensure!(result.is_err() && writer.is_failed() && !writer.can_rollback_source_failure());
                ensure!(writer.rollback_source_failure(|_| panic!("Rollback retried")).is_err());
                ensure!(writer.record_error(&error_record()).is_err());
                ensure!(writer.finish().is_err());
            } else {
                result?;
                writer.record_error(&error_record())?;
                let mut source = MemoryPayloadSource::new(b"expected".to_vec())?;
                writer.write_entity(&entity, Some(&mut source))?;
                let archive = ArchiveReader::new(writer.finish()?)?;
                archive.verify_integrity()?;
                ensure!(archive.sequence == 2 && archive.members.len() == 1 && archive.errors.len() == 1);
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_sink_failures_at_every_section_byte_forbid_rollback() -> Result<()> {
        let entity = FileEntity::from_vec(b"expected".to_vec());
        let mut complete = diagnostic_writer()?;
        let start = complete.writer.len();
        let mut source = MemoryPayloadSource::new(b"expected".to_vec())?;
        complete.write_entity(&entity, Some(&mut source))?;
        let length = complete.writer.len().checked_sub(start).context("Invalid section length")?;
        for zero in [false, true] {
            for remaining in 0..length {
                let initial = diagnostic_writer()?;
                let mut writer = ArchiveWriter {
                    writer: FailingSink { bytes: initial.writer, remaining, zero, interrupted: true },
                    context: initial.context, sequence: 0, failed: false, source_failed: false, sink_failed: false,
                };
                let mut source = MemoryPayloadSource::new(b"expected".to_vec())?;
                let error = writer.write_entity(&entity, Some(&mut source)).expect_err("Sink fault accepted");
                if zero {
                    ensure!(error.downcast_ref::<std::io::Error>()
                        .is_some_and(|error| error.kind() == std::io::ErrorKind::WriteZero));
                }
                ensure!(writer.is_failed() && writer.sink_failed && !writer.can_rollback_source_failure());
                ensure!(writer.sequence == 0);
                ensure!(writer.rollback_source_failure(|_| panic!("Sink failure rolled back")).is_err());
                ensure!(writer.finish().is_err());
            }
        }
        let initial = diagnostic_writer()?;
        let mut writer = ArchiveWriter {
            writer: FailingSink { bytes: initial.writer, remaining: usize::MAX, zero: false, interrupted: true },
            context: initial.context, sequence: 0, failed: false, source_failed: false, sink_failed: false,
        };
        let mut source = FaultySource::new(SourceFault::Hash)?;
        ensure!(writer.write_entity(&entity, Some(&mut source)).is_err());
        ensure!(writer.can_rollback_source_failure() && !writer.sink_failed);
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_observed_source_and_sink_failures_sink_dominates() -> Result<()> {
        let mut source = FaultySource::new(SourceFault::Read)?;
        let mut source_failed = false;
        let mut source = VerifiedSource {
            source: &mut source, bytes: 0, size: 8, digest: Sha256Stream::new(), failed: &mut source_failed,
        };
        let initial = diagnostic_writer()?;
        let mut writer = ArchiveWriter {
            writer: FlushFailure(initial.writer),
            context: initial.context, sequence: 0, failed: false, source_failed: false, sink_failed: false,
        };
        ensure!(writer.write_record(|output, _| {
            ensure!(source.read(&mut [0_u8; 1]).is_err());
            output.flush()?;
            bail!("arbitrary encoder failure")
        }).is_err());
        ensure!(source_failed && writer.sink_failed && !writer.can_rollback_source_failure());
        ensure!(writer.rollback_source_failure(|_| panic!("Arbitrary encoder reset")).is_err());

        let initial = diagnostic_writer()?;
        let mut writer = ArchiveWriter {
            writer: FailingSink { bytes: initial.writer, remaining: usize::MAX, zero: false, interrupted: false },
            context: initial.context, sequence: 0, failed: false, source_failed: false, sink_failed: false,
        };
        let entity = FileEntity::from_vec(b"expected".to_vec());
        let mut source = FaultySource::new(SourceFault::Hash)?;
        ensure!(writer.write_entity(&entity, Some(&mut source)).is_err());
        ensure!(writer.can_rollback_source_failure());
        writer.writer.remaining = 0;
        ensure!(writer.rollback_source_failure(|output| {
            output.write_all(b"rollback sink failure")?;
            Ok(())
        }).is_err());
        ensure!(writer.source_failed && writer.sink_failed && !writer.can_rollback_source_failure());
        ensure!(writer.rollback_source_failure(|_| panic!("Mixed failure rolled back")).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_payload_prevalidation_leaves_writer_ready() -> Result<()> {
        let entity = FileEntity::from_vec(b"expected".to_vec());
        let mut writer = diagnostic_writer()?;
        let before = writer.writer.clone();
        ensure!(writer.write_entity(&entity, None).is_err());
        let mut source = MemoryPayloadSource::new(Vec::new())?;
        ensure!(writer.write_entity(&entity, Some(&mut source)).is_err());
        ensure!(writer.writer == before && !writer.is_failed() && !writer.can_rollback_source_failure());
        ensure!(writer.rollback_source_failure(|_| panic!("Ready writer rolled back")).is_err());
        writer.record_error(&error_record())?;
        ArchiveReader::new(archive_file(&writer.finish()?)?)?.verify_integrity()?;
        Ok(())
    }

    #[crate::ctb_test]
    fn archive_boundary_flush_failure_is_sticky() -> Result<()> {
        let mut output = FlushFailure(Vec::new());
        let mut failed = false;
        let mut observed = ObservedSink { writer: &mut output, failed: &mut failed };
        ensure!(observed.flush().is_err());
        observed.write_all(b"later successful write")?;
        ensure!(failed);
        Ok(())
    }

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