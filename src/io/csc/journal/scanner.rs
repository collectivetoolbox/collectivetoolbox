#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{
    JournalErrorRecord, JournalErrorRecordPayload, JournalSessionRecord,
    JournalWarningRecord, JournalWarningRecordPayload,
    JournalWarningRecordStringPayload, session_context,
};
use ctb_formats_dcdata::dc;
use ctb_formats_dcstring::{
    DcDocumentNavigator, DcFrameHeaderInfo, DcFrameStub, DcMixedDecode,
    DcMstring, DcProjectionPolicy, DcSyntaxArena, SharedValueContext,
};
use std::fs::File;
use std::time::UNIX_EPOCH;

#[derive(Clone, Copy)]
pub(crate) enum ArchiveMode {
    Complete,
    Recover,
}

pub(crate) enum ReaderProfile<'a> {
    Journal {
        projection_policy: &'a DcProjectionPolicy,
    },
    Archive {
        mode: ArchiveMode,
    },
}

impl ReaderProfile<'_> {
    fn is_archive(&self) -> bool {
        matches!(self, Self::Archive { .. })
    }

    fn recovers_tail(&self) -> bool {
        !matches!(
            self,
            Self::Archive {
                mode: ArchiveMode::Complete
            }
        )
    }
}

pub(crate) struct ScanResult<Member> {
    pub session: JournalSessionRecord,
    pub members: Vec<Member>,
    pub errors: Vec<JournalErrorRecord>,
    pub warnings: Vec<JournalWarningRecord>,
    pub sequence: u64,
    pub completed: bool,
    pub valid_length: u64,
}

fn unexpected_eof(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<std::io::Error>()
        .is_some_and(|error| error.kind() == std::io::ErrorKind::UnexpectedEof)
}

fn warning<'source>(
    navigator: &mut DcDocumentNavigator<'source>,
    stub: &DcFrameStub,
    arena: &mut DcSyntaxArena<'source>,
    context: &SharedValueContext,
    policy: &DcProjectionPolicy,
    legacy: bool,
) -> Result<JournalWarningRecord> {
    arena.clear();
    let payload = match navigator
        .decode_frame_with_arena::<JournalWarningRecordPayload>(
            stub,
            arena,
            Some(context),
            policy,
        ) {
        Ok(payload) => payload,
        Err(timestamped_error) if legacy => {
            arena.clear();
            let payload: JournalWarningRecordStringPayload = navigator.decode_frame_with_arena(
                stub, arena, Some(context), policy,
            ).with_context(|| format!("Failed to decode closed journal warning record (timestamped form: {timestamped_error:#})"))?;
            JournalWarningRecordPayload {
                message: payload.message,
                timestamp: UNIX_EPOCH,
            }
        }
        Err(error) => return Err(error),
    };
    Ok(payload.to_record())
}

pub(crate) fn scan<Member: DcMixedDecode>(
    file: File,
    profile: ReaderProfile<'_>,
) -> Result<ScanResult<Member>> {
    let mut navigator = DcDocumentNavigator::from_dcmm_file(file, 8)?;
    let strict = DcProjectionPolicy::strict();
    let quiet = DcProjectionPolicy::forgiving_quiet();
    let policy = match &profile {
        ReaderProfile::Journal { projection_policy } => *projection_policy,
        ReaderProfile::Archive { .. } => &strict,
    };
    let scan_policy = if profile.recovers_tail() {
        &quiet
    } else {
        &strict
    };
    let mut arena = DcSyntaxArena::new();
    let child_policy = if profile.is_archive() {
        &strict
    } else {
        &quiet
    };
    let session_stub = navigator
        .next_sibling_stub(child_policy)?
        .context("Session header is incomplete")?;
    ensure!(
        session_stub.head_dc == dc::DC_BEGIN_OPERATION_SESSION
            && !session_stub.is_unclosed(),
        "Session header is incomplete"
    );
    let session: JournalSessionRecord = navigator
        .decode_frame_with_arena(&session_stub, &mut arena, None, policy)
        .context("Failed to decode session header")?;
    if !profile.is_archive() && policy.is_strict() {
        u8::try_from(session.origin_platform)
            .context("Invalid origin platform code in session header")?;
    }
    let context = session_context(&session)?;
    let mut result = ScanResult {
        session,
        members: Vec::new(),
        errors: Vec::new(),
        warnings: Vec::new(),
        sequence: 0,
        completed: false,
        valid_length: session_stub.outer_span.end,
    };
    loop {
        let stub = match navigator.next_sibling_stub(scan_policy) {
            Ok(Some(stub)) => stub,
            Ok(None) => break,
            Err(error)
                if profile.recovers_tail()
                    && (!profile.is_archive() || !result.completed)
                    && unexpected_eof(&error) =>
            {
                break;
            }
            Err(error) => {
                return Err(error).context("Invalid record structure");
            }
        };
        ensure!(
            !profile.is_archive() || !result.completed,
            "Unexpected data after archive completion"
        );
        if stub.head_dc == dc::DC_BEGIN_CHECKSUMMED_SECTION {
            if stub.is_unclosed() {
                ensure!(
                    !profile.is_archive()
                        || (profile.recovers_tail()
                            && stub.outer_span.end
                                == navigator.active_span().end),
                    "Incomplete archive section"
                );
                break;
            }
            let (sequence, digest_valid) = match stub.header_info {
                DcFrameHeaderInfo::ChecksummedSection {
                    sequence_id: Some(sequence),
                    digest_valid,
                    ..
                } => (sequence, digest_valid),
                _ => bail!(
                    "Missing sequence identifier in closed checksummed section"
                ),
            };
            if profile.is_archive() {
                ensure!(
                    sequence
                        == result
                            .sequence
                            .checked_add(1)
                            .context("Archive sequence overflow")?
                        && digest_valid,
                    "Invalid archive section sequence or checksum"
                );
            } else if !digest_valid {
                if policy.is_strict() {
                    ensure!(
                        stub.outer_span.end == navigator.active_span().end,
                        "Digest mismatch in interior journal batch {sequence}"
                    );
                    warn!(
                        "Digest mismatch in final journal checksummed section under strict policy; rolling back tail"
                    );
                    break;
                }
                let message = format!(
                    "Journal batch {sequence} was salvaged despite an invalid section digest; its integrity is not verified"
                );
                policy.handle(policy.syntax_error, &message)?;
                result.warnings.push(JournalWarningRecord {
                    code: "journal-digest-mismatch".to_owned(),
                    message,
                    timestamp_sec: 0,
                });
            }
            navigator.enter_frame(&stub)?;
            let identifier = navigator
                .next_sibling_stub(child_policy)?
                .context("Missing sequence identifier")?;
            ensure!(
                identifier.head_dc == dc::DC_SEQUENCE_IDENTIFIER,
                "Invalid sequence identifier"
            );
            let mut members = Vec::new();
            let mut errors = Vec::new();
            let mut warnings = Vec::new();
            let mut count = 0_usize;
            while let Some(child) = navigator.next_sibling_stub(child_policy)? {
                count = count
                    .checked_add(1)
                    .context("Section record count overflow")?;
                ensure!(
                    !profile.is_archive() || count == 1,
                    "Archive section contains extra records"
                );
                arena.clear();
                match child.head_dc {
                    head if profile.is_archive()
                        && head == dc::DC_BEGIN_ERROR_RECORD =>
                    {
                        let payload: JournalErrorRecordPayload = navigator
                            .decode_frame_with_arena(
                                &child,
                                &mut arena,
                                Some(&context),
                                policy,
                            )?;
                        errors.push(payload.to_record(policy)?);
                    }
                    head if profile.is_archive()
                        && head == dc::DC_BEGIN_WARNING_RECORD =>
                    {
                        warnings.push(warning(
                            &mut navigator,
                            &child,
                            &mut arena,
                            &context,
                            policy,
                            false,
                        )?);
                    }
                    head => {
                        ensure!(
                            !profile.is_archive()
                                || head
                                    == ctb_formats_dcstring::DcChar::from_short(
                                        317
                                    ),
                            "Unknown archive section record {head}"
                        );
                        members.push(navigator.decode_frame_with_arena::<Member>(
                            &child, &mut arena, Some(&context), policy,
                        ).with_context(|| format!("Failed to decode FileEntity in batch {sequence} (digest valid: {digest_valid}) (pass --allow-journal-errors for forgiving recovery)"))?);
                    }
                }
            }
            ensure!(
                !profile.is_archive() || count == 1,
                "Empty archive section"
            );
            navigator.exit_frame()?;
            result.members.extend(members);
            result.errors.extend(errors);
            result.warnings.extend(warnings);
            result.sequence = sequence;
            result.completed = false;
        } else if stub.head_dc == dc::DC_OPERATION_COMPLETED {
            let mut marker = DcMstring::new();
            marker.push_char(dc::DC_OPERATION_COMPLETED);
            ensure!(
                stub.outer_span.len()
                    == u64::try_from(marker.payload_bytes().len())?,
                "Invalid completion marker"
            );
            result.completed = true;
        } else {
            ensure!(
                !profile.is_archive(),
                "Unexpected archive record {}",
                stub.head_dc
            );
            ensure!(
                stub.head_dc == dc::DC_BEGIN_ERROR_RECORD
                    || stub.head_dc == dc::DC_BEGIN_WARNING_RECORD,
                "Unsupported journal record {:?} at offset {}",
                stub.head_dc,
                stub.outer_span.start
            );
            if stub.is_unclosed() {
                break;
            }
            arena.clear();
            if stub.head_dc == dc::DC_BEGIN_ERROR_RECORD {
                let payload: JournalErrorRecordPayload = navigator
                    .decode_frame_with_arena(
                        &stub,
                        &mut arena,
                        Some(&context),
                        policy,
                    )
                    .context("Failed to decode closed journal error record")?;
                result.errors.push(payload.to_record(policy)?);
            } else {
                result.warnings.push(warning(
                    &mut navigator,
                    &stub,
                    &mut arena,
                    &context,
                    policy,
                    true,
                )?);
            }
        }
        result.valid_length = stub.outer_span.end;
    }
    if profile.is_archive() {
        ensure!(
            !result.completed
                || result.valid_length == navigator.active_span().end,
            "Unexpected bytes after archive completion"
        );
        ensure!(
            result.completed || profile.recovers_tail(),
            "Archive is incomplete (missing completion marker)"
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctb_formats_dcstring::DcMixedEncode;
    use ctb_io::file::{FileEntity, FileEntityWithPayload};
    use std::io::{Read, Seek, SeekFrom, Write};

    fn document(body: &[u8]) -> Result<File> {
        let session = JournalSessionRecord {
            sources: Vec::new(),
            destination: "scan".into(),
            origin_platform: 1,
            environment: None,
        };
        let mut header = DcMstring::new();
        session.encode_dc_mixed(&mut header)?;
        let mut file = tempfile::tempfile()?;
        file.write_all(super::super::JOURNAL_MAGIC_V4)?;
        file.write_all(header.as_bytes())?;
        file.write_all(body)?;
        Ok(file)
    }

    fn section(sequence: u64, members: usize) -> Result<Vec<u8>> {
        let mut body = DcMstring::new();
        for _ in 0..members {
            FileEntity::from_vec(Vec::new()).encode_dc_mixed(&mut body)?;
        }
        checked(sequence, body.payload_bytes())
    }

    fn checked(sequence: u64, body: &[u8]) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        crate::section::write_section(&mut bytes, sequence, |output| {
            output.write_all(body)?;
            Ok(())
        })?;
        Ok(bytes)
    }

    fn journal(
        body: &[u8],
        policy: &DcProjectionPolicy,
    ) -> Result<ScanResult<FileEntity>> {
        scan(
            document(body)?,
            ReaderProfile::Journal {
                projection_policy: policy,
            },
        )
    }

    fn archive(
        body: &[u8],
        mode: ArchiveMode,
    ) -> Result<ScanResult<FileEntityWithPayload>> {
        scan(document(body)?, ReaderProfile::Archive { mode })
    }

    fn completion() -> Vec<u8> {
        let mut marker = DcMstring::new();
        marker.push_char(dc::DC_OPERATION_COMPLETED);
        marker.payload_bytes().to_vec()
    }

    #[crate::ctb_test]
    fn scanner_sequence_and_cardinality_profiles() -> Result<()> {
        let policy = DcProjectionPolicy::strict();
        for members in [0, 1, 2] {
            for sequence in [0, 1, 5] {
                let body = section(sequence, members)?;
                let scanned = journal(&body, &policy)?;
                ensure!(
                    scanned.sequence == sequence
                        && scanned.members.len() == members
                );
                ensure!(
                    archive(&body, ArchiveMode::Recover).is_ok()
                        == (sequence == 1 && members == 1)
                );
            }
        }
        let mut body = Vec::new();
        for sequence in [0, 5, 5, 2, 0] {
            body.extend_from_slice(&section(sequence, 1)?);
        }
        let scanned = journal(&body, &policy)?;
        ensure!(scanned.sequence == 0 && scanned.members.len() == 5);
        ensure!(archive(&body, ArchiveMode::Recover).is_err());
        let mut missing_sequence = DcMstring::new();
        missing_sequence.push_char(dc::DC_BEGIN_CHECKSUMMED_SECTION);
        missing_sequence.push_char(dc::DC_END_CHECKSUMMED_SECTION);
        missing_sequence.push_char(dc::DC_SECTION_DIGEST);
        missing_sequence.push_checksum(
            ctb_utilities::format_id::DC_SHA256,
            &ctb_formats_checksum::sha256(&[]),
        )?;
        ensure!(journal(missing_sequence.payload_bytes(), &policy).is_err());
        ensure!(
            archive(missing_sequence.payload_bytes(), ArchiveMode::Recover)
                .is_err()
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn scanner_digest_final_interior_and_salvage_policy() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let mut bad = section(1, 1)?;
        let last = bad.last_mut().context("Missing digest")?;
        *last ^= 1;
        let base = journal(&[], &strict)?.valid_length;
        let rolled_back = journal(&bad, &strict)?;
        ensure!(
            rolled_back.valid_length == base && rolled_back.members.is_empty()
        );
        ensure!(archive(&bad, ArchiveMode::Recover).is_err());
        ensure!(archive(&bad, ArchiveMode::Complete).is_err());
        let salvaged = journal(&bad, &DcProjectionPolicy::forgiving_quiet())?;
        ensure!(salvaged.members.len() == 1 && salvaged.warnings.len() == 1);
        ensure!(
            salvaged
                .warnings
                .first()
                .context("Missing salvage warning")?
                .code
                == "journal-digest-mismatch"
        );
        let mut rejecting = DcProjectionPolicy::forgiving_quiet();
        rejecting.syntax_error = strict.syntax_error;
        ensure!(journal(&bad, &rejecting).is_err());
        for suffix in [section(2, 1)?, completion(), vec![0xff]] {
            let mut interior = bad.clone();
            interior.extend_from_slice(&suffix);
            ensure!(journal(&interior, &strict).is_err());
            ensure!(archive(&interior, ArchiveMode::Recover).is_err());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn scanner_diagnostic_placement_completion_and_legacy_warning() -> Result<()>
    {
        let strict = DcProjectionPolicy::strict();
        let mut warning = DcMstring::new();
        JournalWarningRecordPayload::new("capture", "detail: retained", -17)
            .encode_dc_mixed(&mut warning)?;
        let top = journal(warning.payload_bytes(), &strict)?;
        ensure!(
            top.warnings
                .first()
                .context("Missing warning")?
                .timestamp_sec
                == -17
        );
        ensure!(
            archive(warning.payload_bytes(), ArchiveMode::Recover).is_err()
        );
        let wrapped = checked(1, warning.payload_bytes())?;
        ensure!(archive(&wrapped, ArchiveMode::Recover)?.warnings.len() == 1);
        ensure!(journal(&wrapped, &strict).is_err());
        let mut legacy = DcMstring::new();
        JournalWarningRecordStringPayload {
            message: "old: warning".into(),
        }
        .encode_dc_mixed(&mut legacy)?;
        let old = journal(legacy.payload_bytes(), &strict)?;
        let old = old.warnings.first().context("Missing old warning")?;
        ensure!(
            old.code == "old"
                && old.message == "warning"
                && old.timestamp_sec == 0
        );
        ensure!(
            archive(&checked(1, legacy.payload_bytes())?, ArchiveMode::Recover)
                .is_err()
        );
        let mut body = completion();
        body.extend_from_slice(warning.payload_bytes());
        ensure!(journal(&body, &strict)?.completed);
        ensure!(archive(&body, ArchiveMode::Recover).is_err());
        body.extend_from_slice(&section(7, 0)?);
        ensure!(!journal(&body, &strict)?.completed);
        ensure!(archive(&body, ArchiveMode::Recover).is_err());
        ensure!(archive(&[], ArchiveMode::Complete).is_err());
        ensure!(!archive(&[], ArchiveMode::Recover)?.completed);
        ensure!(archive(&completion(), ArchiveMode::Complete)?.completed);
        let mut invalid = DcMstring::new();
        invalid.push_char(dc::DC_BEGIN_WARNING_RECORD);
        invalid.push_char(dc::DC_END_WARNING_RECORD);
        ensure!(journal(invalid.payload_bytes(), &strict).is_err());
        ensure!(
            archive(
                &checked(1, invalid.payload_bytes())?,
                ArchiveMode::Recover
            )
            .is_err()
        );
        let mut unknown = DcMstring::new();
        unknown.push_char(dc::DC_SOURCE_PATH);
        ensure!(journal(unknown.payload_bytes(), &strict).is_err());
        ensure!(
            archive(
                &checked(1, unknown.payload_bytes())?,
                ArchiveMode::Recover
            )
            .is_err()
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn scanner_truncation_boundaries_and_terminal_completion() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let prefix = section(1, 1)?;
        let base = journal(&prefix, &strict)?.valid_length;
        let second = section(2, 1)?;
        for length in 0..second.len() {
            let mut body = prefix.clone();
            body.extend_from_slice(
                second.get(..length).context("Section prefix")?,
            );
            let scanned = journal(&body, &strict)?;
            ensure!(
                scanned.valid_length == base && scanned.members.len() == 1,
                "Journal accepted partial section at {length}"
            );
            match archive(&body, ArchiveMode::Recover) {
                Ok(scanned) => ensure!(
                    scanned.valid_length == base && scanned.members.len() == 1,
                    "Archive accepted partial section at {length}"
                ),
                Err(error) => {
                    let mut navigator = DcDocumentNavigator::from_dcmm_file(
                        document(&body)?,
                        8,
                    )?;
                    let quiet = DcProjectionPolicy::forgiving_quiet();
                    navigator
                        .next_sibling_stub(&quiet)?
                        .context("Missing session")?;
                    navigator
                        .next_sibling_stub(&quiet)?
                        .context("Missing committed section")?;
                    let stub = navigator
                        .next_sibling_stub(&quiet)?
                        .context("Missing closed tail")?;
                    ensure!(
                        !stub.is_unclosed(),
                        "Archive rejected unclosed tail at {length}: {error:#}"
                    );
                }
            }
        }
        let mut warning = DcMstring::new();
        JournalWarningRecordPayload::new("capture", "tail", 0)
            .encode_dc_mixed(&mut warning)?;
        for length in 0..warning.payload_bytes().len() {
            let mut body = prefix.clone();
            body.extend_from_slice(
                warning
                    .payload_bytes()
                    .get(..length)
                    .context("Warning prefix")?,
            );
            ensure!(
                journal(&body, &strict)?.valid_length == base,
                "Partial warning accepted at {length}"
            );
        }
        let marker = completion();
        for length in 0..marker.len() {
            let mut body = prefix.clone();
            body.extend_from_slice(
                marker.get(..length).context("Completion prefix")?,
            );
            ensure!(journal(&body, &strict)?.valid_length == base);
            ensure!(!archive(&body, ArchiveMode::Recover)?.completed);
            ensure!(archive(&body, ArchiveMode::Complete).is_err());
        }
        for suffix in [vec![0xff], vec![b' '], marker.clone(), second] {
            for length in 1..=suffix.len() {
                let mut body = marker.clone();
                body.extend_from_slice(
                    suffix.get(..length).context("Trailing prefix")?,
                );
                ensure!(
                    archive(&body, ArchiveMode::Recover).is_err(),
                    "Archive accepted trailing bytes at {length}"
                );
                ensure!(archive(&body, ArchiveMode::Complete).is_err());
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn scanner_lazy_spans_and_context_survive_arena_resets() -> Result<()> {
        use ctb_formats_dcstring::BinaryPayloadRef;
        use ctb_io::file::MemoryPayloadSource;
        use std::sync::Arc;
        let environment =
            Arc::new(ctb_io_environment::EnvDescription::default());
        let session = JournalSessionRecord {
            sources: Vec::new(),
            destination: "archive".into(),
            origin_platform: 1,
            environment: Some(Arc::clone(&environment)),
        };
        let data = vec![0xa5; 131_072];
        let mut entity = FileEntity::from_vec(data.clone());
        entity.metadata.environment = Some(environment);
        let mut writer =
            super::super::stream::RecordWriter::new(Vec::new(), &session, super::super::stream::JournalProfile::ArchiveV1)?;
        for _ in 0..2 {
            let mut source = MemoryPayloadSource::new(data.clone())?;
            writer.write_entity(&entity, Some(&mut source))?;
        }
        writer.record_warning(&JournalWarningRecord {
            code: "reset".into(),
            message: "arena".into(),
            timestamp_sec: 0,
        })?;
        let bytes = writer.finish()?;
        let mut file = tempfile::tempfile()?;
        file.write_all(&bytes)?;
        let scanned = scan::<FileEntityWithPayload>(
            file.try_clone()?,
            ReaderProfile::Archive {
                mode: ArchiveMode::Complete,
            },
        )?;
        ensure!(scanned.members.len() == 2 && scanned.warnings.len() == 1);
        let environment = scanned
            .session
            .environment
            .as_ref()
            .context("Missing session environment")?;
        for member in scanned.members {
            ensure!(Arc::ptr_eq(
                environment,
                member
                    .entity
                    .metadata
                    .environment
                    .as_ref()
                    .context("Missing shared environment")?
            ));
            let payload = member.payload.context("Missing payload")?;
            let BinaryPayloadRef::FileSpan {
                file_offset, len, ..
            } = payload.0
            else {
                bail!("Scanner eagerly materialized primary payload");
            };
            file.seek(SeekFrom::Start(file_offset))?;
            let mut actual = Vec::new();
            (&mut file).take(len).read_to_end(&mut actual)?;
            ensure!(actual == data);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn scanner_session_and_caller_projection_policy() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving_quiet();
        let mut record = DcMstring::new();
        JournalErrorRecordPayload::new(
            "source".into(),
            256_u64 << 32,
            "failure".into(),
            -9,
        )
        .encode_dc_mixed(&mut record)?;
        ensure!(journal(record.payload_bytes(), &strict).is_err());
        let scanned = journal(record.payload_bytes(), &forgiving)?;
        let error = scanned.errors.first().context("Missing error")?;
        ensure!(
            error.stage == super::super::JournalErrorStage::Traversal
                && error.timestamp_sec == -9
        );
        ensure!(archive(record.payload_bytes(), ArchiveMode::Recover).is_err());
        ensure!(
            archive(&checked(1, record.payload_bytes())?, ArchiveMode::Recover)
                .is_err()
        );
        let session = JournalSessionRecord {
            sources: Vec::new(),
            destination: "overflow".into(),
            origin_platform: 256,
            environment: None,
        };
        let mut header = DcMstring::new();
        session.encode_dc_mixed(&mut header)?;
        let mut file = tempfile::tempfile()?;
        file.write_all(super::super::JOURNAL_MAGIC_V4)?;
        file.write_all(header.as_bytes())?;
        ensure!(
            scan::<FileEntity>(
                file.try_clone()?,
                ReaderProfile::Journal {
                    projection_policy: &strict
                }
            )
            .is_err()
        );
        let snapshot = super::super::read_v4_journal_snapshot(
            &mut file,
            std::path::Path::new("policy-test"),
            false,
            &forgiving,
        )?;
        ensure!(snapshot.origin_platform == super::super::current_platform());
        let scanned = scan::<FileEntityWithPayload>(
            file,
            ReaderProfile::Archive {
                mode: ArchiveMode::Recover,
            },
        )?;
        ensure!(scanned.session.origin_platform == 256);
        for length in 0..header.payload_bytes().len() {
            let mut file = tempfile::tempfile()?;
            file.write_all(super::super::JOURNAL_MAGIC_V4)?;
            file.write_all(DcMstring::new().as_bytes())?;
            file.write_all(
                header
                    .payload_bytes()
                    .get(..length)
                    .context("Session prefix")?,
            )?;
            ensure!(
                scan::<FileEntity>(
                    file.try_clone()?,
                    ReaderProfile::Journal {
                        projection_policy: &forgiving
                    }
                )
                .is_err(),
                "Journal accepted partial session at {length}"
            );
            ensure!(
                scan::<FileEntityWithPayload>(
                    file,
                    ReaderProfile::Archive {
                        mode: ArchiveMode::Recover
                    }
                )
                .is_err(),
                "Archive recovery accepted partial session at {length}"
            );
        }
        Ok(())
    }
}
