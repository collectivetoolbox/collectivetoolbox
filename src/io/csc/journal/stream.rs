#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{
    JournalErrorRecord, JournalErrorRecordPayload, JournalSessionRecord,
    JournalWarningRecord, JournalWarningRecordPayload, session_context,
};
use ctb_formats_checksum::Sha256Stream;
use ctb_formats_dcdata::dc;
use ctb_formats_dcstring::{DcMixedEncode, DcMstring, SharedValueContext};
use ctb_io::file::{Extent, FileEntity, FileEntityKind, PayloadSource};
use std::io::{Read, Seek, SeekFrom, Write};

pub const ARCHIVE_MAGIC: &[u8; 8] = b"CTBCSCA\x01";

pub(crate) fn ensure_clean_metadata(entity: &FileEntity) -> Result<()> {
    entity.check_ok()?;
    ensure!(entity.metadata.warnings.is_empty(), "Archive member has semantic metadata warnings: {}", entity.metadata.warnings.join("; "));
    for stream in &entity.streams {
        ensure_clean_metadata(&stream.entity)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalProfile {
    /// Metadata batches with top-level diagnostic records.
    JournalV4,
    /// One body-bearing or diagnostic record per checked section.
    ArchiveV1,
}

pub struct RecordWriter<W: Write> {
    writer: W,
    sequence: u64,
    context: SharedValueContext,
    failed: bool,
    source_failed: bool,
    sink_failed: bool,
    profile: JournalProfile,
    completed: bool,
}

impl<W: Write> RecordWriter<W> {
    pub fn new(
        mut writer: W,
        session: &JournalSessionRecord,
        profile: JournalProfile,
    ) -> Result<Self> {
        let mut header = DcMstring::new();
        session.encode_dc_mixed(&mut header)?;
        ctb_formats_dcstring::validate_dc_mixed_strict(&header)?;
        writer.write_all(match profile {
            JournalProfile::JournalV4 => super::JOURNAL_MAGIC_V4,
            JournalProfile::ArchiveV1 => ARCHIVE_MAGIC,
        })?;
        writer.write_all(header.as_bytes())?;
        Self::resume_output(writer, session, profile, 0, false)
    }

    pub(crate) fn resume_output(
        writer: W,
        session: &JournalSessionRecord,
        profile: JournalProfile,
        sequence: u64,
        completed: bool,
    ) -> Result<Self> {
        Ok(Self {
            writer,
            sequence,
            context: session_context(session)?,
            failed: false,
            source_failed: false,
            sink_failed: false,
            profile,
            completed,
        })
    }

    pub fn output_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    pub fn into_output(self) -> W {
        self.writer
    }

    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }

    #[cfg(test)]
    pub(crate) fn context(&self) -> &SharedValueContext {
        &self.context
    }

    #[cfg(test)]
    pub(crate) fn sink_failed(&self) -> bool {
        self.sink_failed
    }

    #[cfg(test)]
    pub(crate) fn source_failed(&self) -> bool {
        self.source_failed
    }

    pub(crate) fn ensure_ready(&self) -> Result<()> {
        ensure!(
            !self.failed,
            "Record writer has an uncommitted failed record"
        );
        ensure!(!self.completed, "Record writer is already completed");
        Ok(())
    }

    pub(crate) fn transaction<T>(
        &mut self,
        encode: impl FnOnce(&mut W, &SharedValueContext) -> Result<T>,
        durable: impl FnOnce(&mut W) -> Result<()>,
    ) -> Result<T> {
        self.ensure_ready()?;
        self.failed = true;
        self.source_failed = false;
        let value = encode(&mut self.writer, &self.context)?;
        durable(&mut self.writer)?;
        self.failed = false;
        Ok(value)
    }

    pub(crate) fn write_batch(
        &mut self,
        entities: &[FileEntity],
        durable: impl FnOnce(&mut W) -> Result<()>,
    ) -> Result<()> {
        self.ensure_ready()?;
        ensure!(
            self.profile == JournalProfile::JournalV4,
            "Metadata batches require journal profile"
        );
        self.write_record_durable(
            |writer, context| {
                let mut record = DcMstring::new();
                for entity in entities {
                    if let Some(read_time) = entity.metadata.read_time {
                        read_time
                            .duration_since(std::time::UNIX_EPOCH)
                            .context(
                                "Read time timestamp is before Unix epoch",
                            )?;
                    }
                    entity
                        .encode_dc_mixed_with_context(&mut record, context)?;
                }
                ctb_formats_dcstring::validate_dc_mixed_strict(&record)?;
                writer.write_all(record.payload_bytes())?;
                Ok(())
            },
            durable,
        )
    }

    pub fn write_entity(
        &mut self,
        entity: &FileEntity,
        payload: Option<&mut dyn PayloadSource>,
    ) -> Result<()> {
        self.write_entity_checked(entity, payload, || Ok(()))
    }

    pub fn write_entity_checked(
        &mut self,
        entity: &FileEntity,
        payload: Option<&mut dyn PayloadSource>,
        precommit: impl FnOnce() -> Result<()>,
    ) -> Result<()> {
        self.ensure_ready()?;
        ensure!(
            self.profile == JournalProfile::ArchiveV1,
            "Body records require archive profile"
        );
        ensure_clean_metadata(entity)?;
        if let FileEntityKind::Regular { size, .. } = &entity.kind {
            let source = payload
                .as_ref()
                .context("Regular record requires file data")?;
            ensure!(
                source.total_size() == *size,
                "Source size differs from record metadata"
            );
        } else {
            ensure!(
                payload.is_none(),
                "Non-regular record cannot contain primary data"
            );
        }
        let mut source_failed = false;
        let result = self.write_record(|mut writer, context| {
            if let FileEntityKind::Regular { size, sha256, .. } = &entity.kind {
                let source =
                    payload.context("Regular record requires file data")?;
                let mut source = VerifiedSource {
                    source,
                    digest: Sha256Stream::new(),
                    bytes: 0,
                    size: *size,
                    failed: &mut source_failed,
                };
                entity.encode_record_to_writer_with_payload_and_context(
                    &mut writer,
                    &mut source,
                    context,
                )?;
                if source.bytes != *size
                    || source.digest.clone().finalize() != *sha256
                {
                    *source.failed = true;
                    bail!("Source contents changed while encoding");
                }
                let warnings = source.finish()?;
                if !warnings.is_empty() {
                    *source.failed = true;
                    bail!("Source fidelity warnings: {}", warnings.join("; "));
                }
            } else {
                ensure!(
                    payload.is_none(),
                    "Non-regular record cannot contain primary data"
                );
                let mut record = DcMstring::new();
                entity.encode_dc_mixed_with_context(&mut record, context)?;
                ctb_formats_dcstring::validate_dc_mixed_strict(&record)?;
                writer.write_all(record.payload_bytes())?;
            }
            if let Err(error) = precommit() {
                source_failed = true;
                return Err(error);
            }
            Ok(())
        });
        self.source_failed = result.is_err() && source_failed;
        result
    }

    pub fn record_error(&mut self, error: &JournalErrorRecord) -> Result<()> {
        self.record_error_durable(error, |_| Ok(()))
    }

    pub(crate) fn record_error_durable(
        &mut self,
        error: &JournalErrorRecord,
        durable: impl FnOnce(&mut W) -> Result<()>,
    ) -> Result<()> {
        self.write_diagnostic(
            |writer, _| {
                let mut record = DcMstring::new();
                JournalErrorRecordPayload::from_record(error)?
                    .encode_dc_mixed(&mut record)?;
                ctb_formats_dcstring::validate_dc_mixed_strict(&record)?;
                writer.write_all(record.payload_bytes())?;
                Ok(())
            },
            durable,
        )
    }

    pub fn record_warning(
        &mut self,
        warning: &JournalWarningRecord,
    ) -> Result<()> {
        self.record_warning_durable(warning, |_| Ok(()))
    }

    pub(crate) fn record_warning_durable(
        &mut self,
        warning: &JournalWarningRecord,
        durable: impl FnOnce(&mut W) -> Result<()>,
    ) -> Result<()> {
        self.write_diagnostic(
            |writer, _| {
                let mut record = DcMstring::new();
                JournalWarningRecordPayload::from_record(warning)
                    .encode_dc_mixed(&mut record)?;
                ctb_formats_dcstring::validate_dc_mixed_strict(&record)?;
                writer.write_all(record.payload_bytes())?;
                Ok(())
            },
            durable,
        )
    }

    fn write_diagnostic(
        &mut self,
        encode: impl FnOnce(&mut dyn Write, &SharedValueContext) -> Result<()>,
        durable: impl FnOnce(&mut W) -> Result<()>,
    ) -> Result<()> {
        match self.profile {
            JournalProfile::ArchiveV1 => {
                self.write_record_durable(encode, durable)
            }
            JournalProfile::JournalV4 => self.transaction(
                |writer, context| encode(writer, context),
                durable,
            ),
        }
    }

    pub(crate) fn write_record(
        &mut self,
        encode: impl FnOnce(&mut dyn Write, &SharedValueContext) -> Result<()>,
    ) -> Result<()> {
        self.write_record_durable(encode, |_| Ok(()))
    }

    fn write_record_durable(
        &mut self,
        encode: impl FnOnce(&mut dyn Write, &SharedValueContext) -> Result<()>,
        durable: impl FnOnce(&mut W) -> Result<()>,
    ) -> Result<()> {
        self.ensure_ready()?;
        self.failed = true;
        self.source_failed = false;
        let next = self
            .sequence
            .checked_add(1)
            .context("Record sequence overflow")?;
        let mut writer = ObservedSink {
            writer: &mut self.writer,
            failed: &mut self.sink_failed,
        };
        crate::section::write_section(&mut writer, next, |mut writer| {
            encode(&mut writer, &self.context)
        })?;
        durable(&mut self.writer)?;
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
    pub(crate) fn rollback_source_failure(
        &mut self,
        rollback: impl FnOnce(&mut W) -> Result<()>,
    ) -> Result<()> {
        ensure!(
            self.can_rollback_source_failure(),
            "Record failure is not source-only"
        );
        self.sink_failed = true;
        rollback(&mut self.writer)?;
        self.failed = false;
        self.source_failed = false;
        self.sink_failed = false;
        Ok(())
    }

    pub(crate) fn complete(
        &mut self,
        durable: impl FnOnce(&mut W) -> Result<()>,
    ) -> Result<()> {
        self.ensure_ready()?;
        self.failed = true;
        let mut completion = DcMstring::new();
        completion.push_char(dc::DC_OPERATION_COMPLETED);
        let mut writer = ObservedSink {
            writer: &mut self.writer,
            failed: &mut self.sink_failed,
        };
        writer.write_all(completion.payload_bytes())?;
        writer.flush()?;
        durable(&mut self.writer)?;
        self.completed = true;
        self.failed = false;
        Ok(())
    }

    pub fn finish(mut self) -> Result<W> {
        self.complete(|_| Ok(()))?;
        Ok(self.into_output())
    }

    pub(crate) fn complete_legacy(
        &mut self,
        durable: impl FnOnce(&mut W) -> Result<()>,
    ) -> Result<()> {
        self.transaction(
            |writer, _| {
                writer.write_all(&[super::legacy::TAG_JOB_COMPLETED])?;
                Ok(())
            },
            durable,
        )?;
        self.completed = true;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn map_output<T: Write>(
        self,
        map: impl FnOnce(W) -> T,
    ) -> RecordWriter<T> {
        RecordWriter {
            writer: map(self.writer),
            sequence: self.sequence,
            context: self.context,
            failed: self.failed,
            source_failed: self.source_failed,
            sink_failed: self.sink_failed,
            profile: self.profile,
            completed: self.completed,
        }
    }
}

pub(crate) struct ObservedSink<'writer, W> {
    pub(crate) writer: &'writer mut W,
    pub(crate) failed: &'writer mut bool,
}

impl<W: Write> Write for ObservedSink<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        match self.writer.write(bytes) {
            Ok(0) if !bytes.is_empty() => {
                *self.failed = true;
                Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "Record sink accepted no bytes",
                ))
            }
            Ok(count) if count > bytes.len() => {
                *self.failed = true;
                Err(std::io::Error::other("Invalid record sink write count"))
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

pub(crate) struct VerifiedSource<'source> {
    pub(crate) source: &'source mut dyn PayloadSource,
    pub(crate) digest: Sha256Stream,
    pub(crate) bytes: u64,
    pub(crate) size: u64,
    pub(crate) failed: &'source mut bool,
}

impl Read for VerifiedSource<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = match self.source.read(buffer) {
            Ok(0) if !buffer.is_empty() && self.bytes < self.size => {
                *self.failed = true;
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Record source ended before its declared size",
                ));
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
        self.digest.update(buffer.get(..count).ok_or_else(|| {
            std::io::Error::other("Invalid source read count")
        })?);
        self.bytes = self
            .bytes
            .checked_add(u64::try_from(count).map_err(std::io::Error::other)?)
            .ok_or_else(|| {
                std::io::Error::other("Source byte count overflow")
            })?;
        Ok(count)
    }
}

impl Seek for VerifiedSource<'_> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        if self.bytes != 0 {
            return Err(std::io::Error::other(
                "Cannot seek after hashing source data",
            ));
        }
        let result = self.source.seek(position);
        if result.is_err() {
            *self.failed = true;
        }
        result
    }
}

impl PayloadSource for VerifiedSource<'_> {
    fn total_size(&self) -> u64 {
        self.size
    }
    fn extents(&self) -> &[Extent] {
        self.source.extents()
    }
    fn finish(&mut self) -> Result<Vec<String>> {
        let result = self.source.finish();
        if result.is_err() {
            *self.failed = true;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctb_io::file::MemoryPayloadSource;

    struct ShortOutput(Vec<u8>);

    impl Write for ShortOutput {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let accepted = bytes
                .get(..bytes.len().min(3))
                .context("Invalid test slice")
                .map_err(std::io::Error::other)?;
            self.0.extend_from_slice(accepted);
            Ok(accepted.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn session() -> JournalSessionRecord {
        JournalSessionRecord {
            sources: vec!["source".into()],
            destination: "destination".into(),
            origin_platform: 1,
            environment: Some(std::sync::Arc::new(
                ctb_io_environment::EnvDescription::default(),
            )),
        }
    }

    fn reference_section(
        output: &mut Vec<u8>,
        sequence: u64,
        bytes: &[u8],
    ) -> Result<()> {
        let mut section = DcMstring::new();
        section.push_char(dc::DC_BEGIN_CHECKSUMMED_SECTION);
        let mut body = DcMstring::new();
        body.push_char(dc::DC_SEQUENCE_IDENTIFIER);
        sequence.encode_dc_mixed(&mut body)?;
        let mut body = body.payload_bytes().to_vec();
        body.extend_from_slice(bytes);
        output.extend_from_slice(section.payload_bytes());
        output.extend_from_slice(&body);
        let mut footer = DcMstring::new();
        footer.push_char(dc::DC_END_CHECKSUMMED_SECTION);
        footer.push_char(dc::DC_SECTION_DIGEST);
        footer.push_checksum(
            ctb_utilities::format_id::DC_SHA256,
            &ctb_formats_checksum::sha256(&body),
        )?;
        output.extend_from_slice(footer.payload_bytes());
        Ok(())
    }

    #[crate::ctb_test]
    fn profiles_match_historical_bytes_with_same_session_and_short_output()
    -> Result<()> {
        let session = session();
        let context = session_context(&session)?;
        let entities = [
            FileEntity::from_vec(b"first".to_vec()),
            FileEntity::from_vec(Vec::new()),
        ];
        let error = JournalErrorRecord {
            path: "source".into(),
            raw_path: b"source".to_vec(),
            stage: super::super::JournalErrorStage::PayloadRead,
            error_message: "read failed".into(),
            os_error: Some(5),
            timestamp_sec: 12,
        };
        let warning = JournalWarningRecord {
            code: "capture".into(),
            message: "warning".into(),
            timestamp_sec: 13,
        };
        for profile in [JournalProfile::JournalV4, JournalProfile::ArchiveV1] {
            let mut writer = RecordWriter::new(
                ShortOutput(Vec::new()),
                &session,
                profile,
            )?;
            let mut header = DcMstring::new();
            session.encode_dc_mixed(&mut header)?;
            let mut expected = match profile {
                JournalProfile::JournalV4 => {
                    super::super::JOURNAL_MAGIC_V4.to_vec()
                }
                JournalProfile::ArchiveV1 => ARCHIVE_MAGIC.to_vec(),
            };
            expected.extend_from_slice(header.as_bytes());
            match profile {
                JournalProfile::JournalV4 => {
                    let mut body = DcMstring::new();
                    for entity in &entities {
                        entity.encode_dc_mixed_with_context(
                            &mut body, &context,
                        )?;
                    }
                    reference_section(&mut expected, 1, body.payload_bytes())?;
                    writer.write_batch(&entities, |output| {
                        output.flush()?;
                        Ok(())
                    })?;
                }
                JournalProfile::ArchiveV1 => {
                    for (index, entity) in entities.iter().enumerate() {
                        let data = if index == 0 {
                            b"first".to_vec()
                        } else {
                            Vec::new()
                        };
                        let mut source =
                            MemoryPayloadSource::new(data.clone())?;
                        let mut body = Vec::new();
                        entity
                            .encode_record_to_writer_with_payload_and_context(
                                &mut body,
                                &mut source,
                                &context,
                            )?;
                        reference_section(
                            &mut expected,
                            u64::try_from(index)?
                                .checked_add(1)
                                .context("Test sequence overflow")?,
                            &body,
                        )?;
                        let mut source = MemoryPayloadSource::new(data)?;
                        writer.write_entity(entity, Some(&mut source))?;
                    }
                }
            }
            let mut diagnostic = DcMstring::new();
            JournalErrorRecordPayload::from_record(&error)?
                .encode_dc_mixed(&mut diagnostic)?;
            if profile == JournalProfile::JournalV4 {
                expected.extend_from_slice(diagnostic.payload_bytes());
            } else {
                reference_section(
                    &mut expected,
                    3,
                    diagnostic.payload_bytes(),
                )?;
            }
            writer.record_error(&error)?;
            let mut diagnostic = DcMstring::new();
            JournalWarningRecordPayload::from_record(&warning)
                .encode_dc_mixed(&mut diagnostic)?;
            if profile == JournalProfile::JournalV4 {
                expected.extend_from_slice(diagnostic.payload_bytes());
            } else {
                reference_section(
                    &mut expected,
                    4,
                    diagnostic.payload_bytes(),
                )?;
            }
            writer.record_warning(&warning)?;
            let mut completion = DcMstring::new();
            completion.push_char(dc::DC_OPERATION_COMPLETED);
            expected.extend_from_slice(completion.payload_bytes());
            ensure!(
                writer.finish()?.0 == expected,
                "Profile {profile:?} changed historical wire bytes"
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn durable_failure_keeps_sequence_and_refuses_retry() -> Result<()> {
        let mut writer = RecordWriter::new(
            Vec::new(),
            &session(),
            JournalProfile::JournalV4,
        )?;
        let entity = FileEntity::from_vec(b"payload".to_vec());
        ensure!(
            writer
                .write_batch(std::slice::from_ref(&entity), |_| bail!(
                    "injected sync failure"
                ))
                .is_err()
        );
        ensure!(
            writer.sequence() == 0
                && writer.is_failed()
                && !writer.can_rollback_source_failure()
        );
        let before = writer.output_mut().clone();
        let mut called = false;
        ensure!(
            writer
            .write_batch(&[entity], |_| { called = true; Ok(()) })
                .is_err()
        );
        ensure!(
            writer
            .complete(|_| { called = true; Ok(()) })
                .is_err()
        );
        ensure!(!called, "Refused operation invoked durability");
        ensure!(*writer.output_mut() == before);
        Ok(())
    }

    #[crate::ctb_test]
    fn capture_metadata_precommit_failure_has_no_footer_and_can_rollback() -> Result<()> {
        let mut writer = RecordWriter::new(Vec::new(), &session(), JournalProfile::ArchiveV1)?;
        let checkpoint = writer.output_mut().len();
        let mut entity = FileEntity::from_vec(Vec::new());
        entity.kind = FileEntityKind::Directory;
        ensure!(writer.write_entity_checked(&entity, None, || bail!("injected source change after encode")).is_err());
        ensure!(writer.sequence() == 0 && writer.source_failed() && !writer.sink_failed());
        ensure!(writer.can_rollback_source_failure());
        let bytes = writer.output_mut();
        let mut footer = DcMstring::new();
        footer.push_char(dc::DC_END_CHECKSUMMED_SECTION);
        ensure!(!bytes.ends_with(footer.payload_bytes()));
        writer.rollback_source_failure(|output| { output.truncate(checkpoint); Ok(()) })?;
        writer.write_entity_checked(&entity, None, || Ok(()))?;
        ensure!(writer.sequence() == 1);
        Ok(())
    }

    #[crate::ctb_test]
    fn journal_completion_durability_failure_poisoned_modern_and_legacy() -> Result<()> {
        for legacy in [false, true] {
            let mut writer = RecordWriter::new(Vec::new(), &session(), JournalProfile::JournalV4)?;
            let result = if legacy {
                writer.complete_legacy(|_| bail!("injected completion sync failure"))
            } else {
                writer.complete(|_| bail!("injected completion sync failure"))
            };
            ensure!(result.is_err() && writer.is_failed());
            ensure!(!writer.completed && !writer.can_rollback_source_failure());
            let bytes = writer.output_mut().clone();
            let mut called = false;
            ensure!(writer.complete(|_| { called = true; Ok(()) }).is_err());
            ensure!(writer.complete_legacy(|_| { called = true; Ok(()) }).is_err());
            ensure!(writer.write_batch(&[], |_| { called = true; Ok(()) }).is_err());
            ensure!(writer.transaction(|_, _| { called = true; Ok(()) }, |_| Ok(())).is_err());
            ensure!(!called && *writer.output_mut() == bytes && writer.sequence() == 0);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn completed_profiles_refuse_all_writes_and_duplicate_completion()
    -> Result<()> {
        for profile in [JournalProfile::JournalV4, JournalProfile::ArchiveV1] {
            let mut writer =
                RecordWriter::new(Vec::new(), &session(), profile)?;
            writer.complete(|_| Ok(()))?;
            let before = writer.output_mut().clone();
            let mut called = false;
            ensure!(
                writer.complete(|_| { called = true; Ok(()) }).is_err()
            );
            ensure!(
                writer
                    .write_batch(&[], |_| { called = true; Ok(()) })
                    .is_err()
            );
            ensure!(
                writer
                    .write_entity(&FileEntity::from_vec(Vec::new()), None)
                    .is_err()
            );
            ensure!(
                writer
                    .record_warning(&JournalWarningRecord {
                        code: "late".into(),
                        message: "late".into(),
                        timestamp_sec: 0
                    })
                    .is_err()
            );
            ensure!(!called, "Completed writer invoked durability");
            ensure!(*writer.output_mut() == before);
        }
        Ok(())
    }
}
