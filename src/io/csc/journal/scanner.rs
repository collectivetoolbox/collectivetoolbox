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

//! Sequential scanning and validation of journal records and stream frames.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{
    session_context, JournalErrorRecord, JournalErrorRecordPayload,
    JournalSessionRecord, JournalWarningRecord, JournalWarningRecordPayload,
    JournalWarningRecordStringPayload,
};
use ctb_formats_dcdata::dc;
use ctb_formats_dcstring::{
    DcDocumentNavigator, DcFrameHeaderInfo, DcFrameStub,
    DcProjectionPolicy, DcSyntaxArena, SharedValueContext,
};
use ctb_io::file::FileEntityWithPayload;
use std::fs::File;
use std::time::UNIX_EPOCH;

/// Whether to require a completed operation or return its committed prefix.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReadMode {
    Complete,
    Recover,
}

pub(crate) struct ScanResult {
    pub session: JournalSessionRecord,
    pub members: Vec<FileEntityWithPayload>,
    pub errors: Vec<JournalErrorRecord>,
    pub warnings: Vec<JournalWarningRecord>,
    pub sequence: u64,
    pub completed: bool,
    pub valid_length: u64,
}

fn retain_recovery_diagnostics(
    arena: &DcSyntaxArena<'_>,
    policy: &DcProjectionPolicy,
    warnings: &mut Vec<JournalWarningRecord>,
) {
    if policy.is_strict() {
        return;
    }
    for diagnostic in arena.diagnostics() {
        if !diagnostic.message.starts_with("Deprecated Document Character") {
            warnings.push(JournalWarningRecord {
                code: "journal-decode-diagnostic".to_owned(),
                message: diagnostic.message.clone(),
                timestamp_sec: 0,
            });
        }
    }
}

fn warning<'source>(
    navigator: &mut DcDocumentNavigator<'source>,
    stub: &DcFrameStub,
    arena: &mut DcSyntaxArena<'source>,
    context: &SharedValueContext,
    policy: &DcProjectionPolicy,
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
        Err(timestamped_error) => {
            arena.clear();
            let payload: JournalWarningRecordStringPayload = navigator.decode_frame_with_arena(
                stub, arena, Some(context), policy)
                .with_context(|| format!("Invalid journal warning (timestamped form: {timestamped_error:#})"))?;
            JournalWarningRecordPayload {
                message: payload.message,
                timestamp: UNIX_EPOCH,
            }
        }
    };
    Ok(payload.to_record())
}

pub(crate) fn scan(
    file: File,
    mode: ReadMode,
    policy: &DcProjectionPolicy,
) -> Result<ScanResult> {
    let mut navigator = DcDocumentNavigator::from_dcmm_file(file, 8)?;
    let quiet = DcProjectionPolicy::forgiving_quiet();
    let scan_policy = if mode == ReadMode::Recover {
        &quiet
    } else {
        policy
    };
    let mut arena = DcSyntaxArena::new();
    let session_stub = navigator
        .next_sibling_stub(scan_policy)?
        .context("Session header is incomplete")?;
    ensure!(
        session_stub.head_dc == dc::DC_BEGIN_OPERATION_SESSION
            && !session_stub.is_unclosed(),
        "Session header is incomplete"
    );
    let session: JournalSessionRecord = navigator.decode_frame_with_arena(
        &session_stub,
        &mut arena,
        None,
        policy,
    )?;
    if policy.is_strict() {
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
    retain_recovery_diagnostics(&arena, policy, &mut result.warnings);
    loop {
        let stub = match navigator.next_sibling_stub(scan_policy) {
            Ok(Some(stub)) => stub,
            Ok(None) => break,
            Err(error)
                if mode == ReadMode::Recover
                    && error.downcast_ref::<std::io::Error>().is_some_and(
                        |error| {
                            error.kind() == std::io::ErrorKind::UnexpectedEof
                        },
                    ) =>
            {
                result.completed = false;
                break;
            }
            Err(error) => {
                return Err(error).context("Invalid journal record structure")
            }
        };
        if stub.is_unclosed() {
            ensure!(
                mode == ReadMode::Recover
                    && stub.outer_span.end == navigator.active_span().end,
                "Incomplete journal record"
            );
            result.completed = false;
            break;
        }
        match stub.head_dc {
            head if head == dc::DC_BEGIN_CHECKSUMMED_SECTION => {
                let (sequence, digest_valid) = match stub.header_info {
                    DcFrameHeaderInfo::ChecksummedSection { sequence_id: Some(sequence), digest_valid, .. } => (sequence, digest_valid),
                    _ => bail!("Missing sequence identifier in closed checksummed section"),
                };
                if !digest_valid {
                    if policy.is_strict() {
                        bail!("Digest mismatch in journal batch {sequence}");
                    }
                    let message = format!("Journal batch {sequence} was salvaged despite an invalid section digest; its integrity is not verified");
                    policy.handle(policy.syntax_error, &message)?;
                    result.warnings.push(JournalWarningRecord {
                        code: "journal-digest-mismatch".to_owned(),
                        message,
                        timestamp_sec: 0,
                    });
                }
                navigator.enter_frame(&stub)?;
                let identifier = navigator
                    .next_sibling_stub(scan_policy)?
                    .context("Missing sequence identifier")?;
                ensure!(
                    identifier.head_dc == dc::DC_SEQUENCE_IDENTIFIER,
                    "Invalid sequence identifier"
                );
                let mut members = Vec::new();
                while let Some(child) =
                    navigator.next_sibling_stub(scan_policy)?
                {
                    ensure!(
                        child.head_dc
                            == ctb_formats_dcstring::DcChar::from_short(317),
                        "Unsupported journal batch record {}",
                        child.head_dc
                    );
                    arena.clear();
                    members.push(navigator.decode_frame_with_arena::<FileEntityWithPayload>(
                        &child, &mut arena, Some(&context), policy)
                        .with_context(|| format!("Failed to decode FileEntity in batch {sequence}"))?);
                    retain_recovery_diagnostics(&arena, policy, &mut result.warnings);
                }
                navigator.exit_frame()?;
                result.members.extend(members);
                result.sequence = sequence;
                result.completed = false;
            }
            head if head == dc::DC_OPERATION_COMPLETED => {
                ensure!(
                    stub.outer_span.len()
                        == navigator.char_encoded_len(dc::DC_OPERATION_COMPLETED),
                    "Invalid completion marker"
                );
                result.completed = true;
            }
            head if head == dc::DC_BEGIN_ERROR_RECORD => {
                arena.clear();
                let payload: JournalErrorRecordPayload = navigator
                    .decode_frame_with_arena(
                        &stub,
                        &mut arena,
                        Some(&context),
                        policy,
                    )
                    .context("Invalid journal error record")?;
                retain_recovery_diagnostics(&arena, policy, &mut result.warnings);
                result.errors.push(payload.to_record(policy)?);
            }
            head if head == dc::DC_BEGIN_WARNING_RECORD => {
                let w = warning(
                    &mut navigator,
                    &stub,
                    &mut arena,
                    &context,
                    policy,
                )?;
                retain_recovery_diagnostics(&arena, policy, &mut result.warnings);
                result.warnings.push(w);
            }
            head => bail!(
                "Unsupported journal record {head} at offset {}",
                stub.outer_span.start
            ),
        }
        result.valid_length = stub.outer_span.end;
    }
    if result.valid_length != navigator.active_span().end {
        result.completed = false;
    }
    ensure!(
        mode == ReadMode::Recover || result.completed,
        "Journal is incomplete (missing completion marker)"
    );
    Ok(result)
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
    use crate::journal::stream::{JournalMagic, JournalWriter};
    use ctb_formats_dcstring::DcMstring;
    use ctb_io::file::{FileEntity, MemoryPayloadSource};
    use std::io::Write;

    fn fixture(magic: JournalMagic) -> Result<Vec<u8>> {
        let session = JournalSessionRecord {
            sources: Vec::new(),
            destination: "output".into(),
            origin_platform: 1,
            environment: None,
        };
        let mut writer = JournalWriter::new(Vec::new(), &session, magic)?;
        let entity = FileEntity::from_vec(b"contents".to_vec());
        writer.write_batch(&[], |_| Ok(()))?;
        writer.write_batch(&[entity.clone(), entity.clone()], |_| Ok(()))?;
        let mut payload = MemoryPayloadSource::new(b"contents".to_vec())?;
        writer.write_entity(&entity, Some(&mut payload))?;
        writer.write_record(|mut output, context| {
            for _ in 0..2 {
                let mut payload =
                    MemoryPayloadSource::new(b"contents".to_vec())?;
                entity.encode_record_to_writer_with_payload_and_context(
                    &mut output,
                    &mut payload,
                    context,
                )?;
            }
            Ok(())
        })?;
        writer.record_warning(&JournalWarningRecord {
            code: "capture".into(),
            message: "test diagnostic".into(),
            timestamp_sec: 0,
        })?;
        writer.finish()
    }

    fn read(bytes: &[u8], mode: ReadMode) -> Result<ScanResult> {
        let mut file = tempfile::tempfile()?;
        file.write_all(bytes)?;
        scan(file, mode, &DcProjectionPolicy::strict())
    }

    #[crate::ctb_test]
    fn either_magic_accepts_batches_optional_bodies_and_diagnostics(
    ) -> Result<()> {
        let journal = fixture(JournalMagic::JournalV4)?;
        let archive = fixture(JournalMagic::ArchiveV1)?;
        ensure!(journal.get(8..) == archive.get(8..));
        for bytes in [&journal, &archive] {
            let scanned = read(bytes, ReadMode::Complete)?;
            ensure!(
                scanned.members.len() == 5
                    && scanned.warnings.len() == 1
                    && scanned.completed
            );
            ensure!(
                scanned
                    .members
                    .iter()
                    .filter(|member| member.payload.is_some())
                    .count()
                    == 3
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn closed_corruption_is_not_an_incomplete_tail() -> Result<()> {
        for magic in [JournalMagic::JournalV4, JournalMagic::ArchiveV1] {
            let session = JournalSessionRecord {
                sources: Vec::new(),
                destination: "output".into(),
                origin_platform: 1,
                environment: None,
            };
            let mut entity = FileEntity::from_vec(Vec::new());
            entity.identity.relative_path = Some("original".into());
            let mut writer = JournalWriter::new(Vec::new(), &session, magic)?;
            writer.write_entity(&entity, None)?;
            let mut bytes = writer.into_output();
            let position = bytes
                .windows(b"original".len())
                .position(|part| part == b"original")
                .context("Missing fixture filename")?;
            *bytes.get_mut(position).context("Missing corruption byte")? = b'X';
            ensure!(
                read(&bytes, ReadMode::Recover).is_err(),
                "Recovery discarded a closed corrupt section"
            );
            let mut file = tempfile::tempfile()?;
            file.write_all(&bytes)?;
            let salvaged = scan(
                file,
                ReadMode::Recover,
                &DcProjectionPolicy::forgiving_quiet(),
            )?;
            ensure!(salvaged.members.len() == 1 && !salvaged.completed);
            ensure!(salvaged
                .warnings
                .iter()
                .any(|warning| warning.code == "journal-digest-mismatch"));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn journal_sequence_and_completion_rules_apply_to_either_magic(
    ) -> Result<()> {
        for magic in [JournalMagic::JournalV4, JournalMagic::ArchiveV1] {
            let session = JournalSessionRecord {
                sources: Vec::new(),
                destination: "output".into(),
                origin_platform: 1,
                environment: None,
            };
            let mut writer = JournalWriter::new(Vec::new(), &session, magic)?;
            for sequence in [0, 2, 2, 1] {
                crate::section::write_section(
                    writer.output_mut(),
                    sequence,
                    |output| {
                        let mut record = DcMstring::new();
                        ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(
                            &FileEntity::from_vec(Vec::new()),
                            &mut record,
                        )?;
                        output.write_all(record.payload_bytes())?;
                        Ok(())
                    },
                )?;
            }
            let mut bytes = writer.finish()?;
            let mut warning = DcMstring::new();
            ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(
                &JournalWarningRecordPayload::from_record(
                    &JournalWarningRecord {
                        code: "retained".into(),
                        message: "after completion".into(),
                        timestamp_sec: 0,
                    },
                ),
                &mut warning,
            )?;
            bytes.extend_from_slice(warning.payload_bytes());
            let read = read(&bytes, ReadMode::Complete)?;
            ensure!(
                read.completed
                    && read.sequence == 1
                    && read.members.len() == 4
                    && read.warnings.len() == 1
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn recovery_and_completion_policy_are_independent_of_magic() -> Result<()> {
        for magic in [JournalMagic::JournalV4, JournalMagic::ArchiveV1] {
            let bytes = fixture(magic)?;
            for length in 8..bytes.len() {
                let Some(prefix) = bytes.get(..length) else {
                    bail!("Invalid fixture prefix");
                };
                ensure!(read(prefix, ReadMode::Complete).is_err());
                if let Ok(scanned) = read(prefix, ReadMode::Recover) {
                    ensure!(
                        !scanned.completed
                            && scanned.valid_length <= u64::try_from(length)?
                    );
                }
            }
        }
        Ok(())
    }
}
