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


#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
    use super::*;
    use crate::DcMixedEncode;
    use ctb_formats_dcdata::syntax::{DatasetRuleResolver, parse_dc_syntax};

    #[crate::ctb_test]
    fn declared_list_length_is_shared_by_validation_and_arena() -> Result<()> {
        let mut document = DcMstring::new();
        document.push_char(DC_BEGIN_LIST);
        2_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DC_TYPE_ANY);
        42_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DC_END_LIST);
        ensure!(validate_dc_mixed_strict(&document).is_err(), "Standalone validation ignored the declared list length");
        let mut nav = DcDocumentNavigator::from_mstr(&document);
        let mut arena = DcSyntaxArena::new();
        ensure!(nav.parse_document_into_arena(&mut arena, &DcProjectionPolicy::strict()).is_err());
        nav.seek_to_offset(nav.active_span().start)?;
        nav.parse_document_into_arena(&mut arena, &DcProjectionPolicy::forgiving())?;
        ensure!(!arena.diagnostics().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn forgiving_text_checksum_retains_payload() -> Result<()> {
        let mut text = DcString::new();
        text.push(DC_START_ENCAPSULATION_BINARY);
        text.push_dc_str(&crate::encapsulation::checksum_text(
            ctb_utilities::format_id::DC_SHA256,
            &ctb_formats_checksum::sha256(b"original"),
        )?);
        for short in ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(b"changed")? {
            text.push(DcChar::from_short(short));
        }
        text.push(DC_END_ENCAPSULATION_BINARY);
        for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
            let mut nav = DcDocumentNavigator::from_dc_str(&text);
            let mut arena = DcSyntaxArena::new();
            let result = nav.parse_node_into_arena(&mut arena, &policy);
            if policy.is_strict() {
                ensure!(result.is_err());
            } else {
                let root = result?;
                let DcSyntaxKind::Binary { payload, .. } = &arena.node(root).context("Missing binary node")?.kind else {
                    bail!("Expected binary payload");
                };
                ensure!(payload.materialize_bytes()?.as_ref() == b"changed");
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn grammar_policy_agrees_across_backends_and_preserves_scope() -> Result<()> {
        use std::io::Write;

        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();
        let mut wrong_fourcc = DcMstring::new();
        wrong_fourcc.push_char(DcChar::from_short(403));
        "too long".encode_dc_mixed(&mut wrong_fourcc)?;
        let mut cases = vec![wrong_fourcc];
        for tokens in [
            vec![DC_BEGIN_LIST, DC_END_LIST],
            vec![DC_BEGIN_KV_MAP, DC_END_KV_MAP],
            vec![DC_BEGIN_TYPE_DEF, DC_END_TYPE_DEF],
            vec![DC_BEGIN_FORMAT_REPRESENTATION, DC_END_FORMAT_REPRESENTATION],
            vec![DC_OPTIONAL_PRESENT],
            vec![DcChar::from_short(403)],
            vec![DC_BEGIN_LIST, DC_TYPE_ANY, DC_LIST_ELEMENT_SEPARATOR, DC_END_LIST],
        ] {
            let mut document = DcMstring::new();
            for token in tokens { document.push_char(token); }
            cases.push(document);
        }
        for document in cases {
            let text = document.to_dc_string()?;
            let mut file = tempfile::tempfile()?;
            file.write_all(document.as_bytes())?;
            for mut nav in [DcDocumentNavigator::from_mstr(&document), DcDocumentNavigator::from_dc_str(&text), DcDocumentNavigator::from_dcmm_file(file, 0)?] {
                ensure!(nav.validate_document(&strict).is_err());
                let mut arena = DcSyntaxArena::new();
                ensure!(nav.parse_document_into_arena(&mut arena, &strict).is_err());
                nav.seek_to_offset(nav.active_span().start)?;
                nav.parse_document_into_arena(&mut arena, &forgiving)?;
                ensure!(!arena.diagnostics().is_empty(), "Missing grammar diagnostic");
            }
        }
        let mut document = DcMstring::new();
        document.push_char(DcChar::from_short(317));
        document.push_char(DcChar::from_short(401));
        document.push_char(DcChar::from_short(403));
        7_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DcChar::from_short(402));
        document.push_char(DcChar::from_short(327));
        42_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DcChar::from_short(318));
        let mut nav = DcDocumentNavigator::from_mstr(&document);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &forgiving)?;
        let root = *arena.roots().first().context("Missing file root")?;
        let children = arena.node(root).context("Missing file node")?.children();
        ensure!(children.len() == 2, "Malformed child leaked into parent");
        ensure!(matches!(arena.node(children[0]).context("Missing Finder node")?.kind, DcSyntaxKind::Container { opener, .. } if opener == DcChar::from_short(401)));
        ensure!(matches!(arena.node(children[1]).context("Missing inode node")?.kind, DcSyntaxKind::Tagged { tag, .. } if tag == DcChar::from_short(327)));
        ensure!(!arena.diagnostics().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_rejects_corrupt_sha256() -> Result<()> {
        use std::io::Write;

        let mut document = DcMstring::new();
        document.push_binary(b"changed");
        let mut bytes = document.as_bytes().to_vec();
        let type_offset = DCMM_HEADER_LEN
            .checked_add(crate::dc_mixed::DC_203_BYTES.len())
            .context("Type offset overflow")?;
        *bytes.get_mut(type_offset).context("Missing binary type")? =
            crate::dc_mixed::TYPE_BINARY_SHA256;
        let digest_offset = type_offset.checked_add(17).context("Digest offset overflow")?;
        bytes.splice(digest_offset..digest_offset, ctb_formats_checksum::sha256(b"original"));
        let mut file = tempfile::tempfile()?;
        file.write_all(&bytes)?;
        let mut nav = DcDocumentNavigator::from_dcmm_file(file, 0)?;
        let mut arena = DcSyntaxArena::default();
        let result = nav.parse_document_into_arena(&mut arena, &DcProjectionPolicy::strict());
        ensure!(result.is_err(), "Strict file parsing accepted a corrupt SHA256 payload");
        Ok(())
    }

    fn file_binary_document(
        payload: &[u8],
        algorithm: SupportedHashFormat,
        metadata_encoding: bool,
    ) -> Result<Vec<u8>> {
        let mut bytes = DcMstring::new().as_bytes().to_vec();
        bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        bytes.push(if metadata_encoding {
            crate::dc_mixed::TYPE_BINARY_METADATA
        } else {
            crate::dc_mixed::TYPE_BINARY_SHA256
        });
        bytes.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
        let digest = ctb_formats_checksum::hash(payload, algorithm);
        if metadata_encoding {
            let metadata = crate::encapsulation::checksum_text(
                algorithm.to_format_id().dc_char().context("Missing checksum format")?,
                &digest,
            )?;
            bytes.extend_from_slice(&u128::try_from(metadata.len())?.to_be_bytes());
            bytes.extend_from_slice(metadata.as_bytes());
        } else {
            bytes.extend_from_slice(&digest);
        }
        bytes.extend_from_slice(payload);
        Ok(bytes)
    }

    fn binary_file_navigator(bytes: &[u8], base_offset: u64) -> Result<DcDocumentNavigator<'static>> {
        use std::io::Write;

        let mut file = tempfile::tempfile()?;
        file.write_all(&vec![0; usize::try_from(base_offset)?])?;
        file.write_all(bytes)?;
        DcDocumentNavigator::from_dcmm_file(file, base_offset)
    }

    #[crate::ctb_test]
    fn unsupported_checksum_preserves_payload_across_backends() -> Result<()> {
        let metadata = crate::encapsulation::checksum_text(
            FormatId::Md5.dc_char().context("Missing MD5 type")?,
            &[0_u8; 16],
        )?;
        for length in [0, 7, usize::try_from(INLINE_BINARY_THRESHOLD)?.saturating_add(1)] {
            let payload = vec![0x5a_u8; length];
            let mut bytes = DcMstring::new().into_bytes();
            bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
            bytes.push(crate::dc_mixed::TYPE_BINARY_METADATA);
            bytes.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
            bytes.extend_from_slice(&u128::try_from(metadata.len())?.to_be_bytes());
            bytes.extend_from_slice(metadata.as_bytes());
            bytes.extend_from_slice(&payload);
            ensure!(DcMstr::from_bytes(&bytes).is_err());
            let mut text = DcString::new();
            text.push(DC_START_ENCAPSULATION_BINARY);
            text.push_dc_str(&metadata);
            for short in ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(&payload)? {
                text.push(DcChar::from_short(short));
            }
            text.push(DC_END_ENCAPSULATION_BINARY);
            for action in [DiagnosticAction::Error, DiagnosticAction::Warn, DiagnosticAction::Log, DiagnosticAction::Ignore] {
                let policy = DcProjectionPolicy { syntax_error: action, ..DcProjectionPolicy::strict() };
                let truncated = bytes.get(..bytes.len().saturating_sub(1)).context("Missing truncated fixture")?;
                ensure!(DcDocumentNavigator::parse_document_arena(truncated, &mut DcSyntaxArena::new(), &policy).is_err(), "Unsupported checksum recovery accepted a truncated payload");
                if action != DiagnosticAction::Error {
                    let source = DcMstr::from_bytes_with_policy(&bytes, &policy)?;
                    let inspection = source.inspect_with_policy::<crate::LazyBinaryPayload>(&policy);
                    let restored = inspection.value().context("Inspection lost unsupported-checksum payload")?;
                    ensure!(restored.0.materialize_bytes()?.as_ref() == payload);
                }
                for source in [text.as_bytes(), bytes.as_slice()] {
                    let mut arena = DcSyntaxArena::new();
                    let result = DcDocumentNavigator::parse_document_arena(source, &mut arena, &policy);
                    if action == DiagnosticAction::Error {
                        ensure!(result.is_err());
                    } else {
                        result?;
                        let root = *arena.roots().first().context("Missing binary root")?;
                        let DcSyntaxKind::Binary { payload: restored, .. } = &arena.node(root).context("Missing binary node")?.kind else {
                            bail!("Expected binary payload");
                        };
                        ensure!(restored.materialize_bytes()?.as_ref() == payload);
                    }
                }
                let mut navigator = binary_file_navigator(&bytes, 8)?;
                let mut arena = DcSyntaxArena::new();
                let result = navigator.parse_document_into_arena(&mut arena, &policy);
                if action == DiagnosticAction::Error {
                    ensure!(result.is_err());
                } else {
                    result?;
                    let root = *arena.roots().first().context("Missing binary root")?;
                    let DcSyntaxKind::Binary { payload: restored, .. } = &arena.node(root).context("Missing binary node")?.kind else {
                        bail!("Expected binary payload");
                    };
                    match restored {
                        BinaryPayloadRef::FileSpan { file_offset, len, checksum, .. } => {
                            ensure!(checksum.is_none());
                            ensure!(navigator.read_span_bytes(DcByteSpan::new(*file_offset, file_offset.checked_add(*len).context("Payload span overflow")?))?.as_ref() == payload);
                        }
                        _ => ensure!(restored.materialize_bytes()?.as_ref() == payload),
                    }
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn unsupported_section_checksum_remains_unverified() -> Result<()> {
        let mut document = DcMstring::new();
        document.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        document.push_char(DC_BEGIN_IDENTIFIER);
        document.push_str("member");
        document.push_char(DC_END_IDENTIFIER);
        document.push_char(ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION);
        document.push_char(DC_SECTION_DIGEST);
        document.push_checksum(FormatId::Md5.dc_char().context("Missing MD5 type")?, &[0_u8; 16])?;
        let text = document.to_dc_string()?;
        for action in [DiagnosticAction::Error, DiagnosticAction::Warn, DiagnosticAction::Log, DiagnosticAction::Ignore] {
            let policy = DcProjectionPolicy { syntax_error: action, ..DcProjectionPolicy::strict() };
            for mut navigator in [
                DcDocumentNavigator::from_mstr(&document),
                DcDocumentNavigator::from_dc_str(&text),
                binary_file_navigator(document.as_bytes(), 8)?,
            ] {
                let result = navigator.next_sibling_stub(&policy);
                if action == DiagnosticAction::Error {
                    ensure!(result.is_err());
                } else {
                    let stub = result?.context("Missing checksummed section")?;
                    ensure!(matches!(stub.header_info, DcFrameHeaderInfo::ChecksummedSection { digest_valid: false, .. }));
                    ensure!(navigator.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("unsupported")));
                    navigator.enter_frame(&stub)?;
                    ensure!(navigator.next_sibling_stub(&policy)?.is_some(), "Lost section contents");
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn nested_metadata_checksum_policy() -> Result<()> {
        let payload = b"payload";
        let mut digest_metadata = DcString::new();
        digest_metadata.push(ctb_utilities::format_id::DC_SHA256);
        digest_metadata.push_dc_str(&crate::encapsulation::checksum_text(
            ctb_utilities::format_id::DC_SHA256, &[0; 32],
        )?);
        let mut metadata = DcString::new();
        crate::encapsulation::write_text(
            &mut metadata, &ctb_formats_checksum::sha256(payload), &digest_metadata,
        )?;
        let mut bytes = DcMstring::new().as_bytes().to_vec();
        bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        bytes.push(crate::dc_mixed::TYPE_BINARY_METADATA);
        bytes.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
        bytes.extend_from_slice(&u128::try_from(metadata.len())?.to_be_bytes());
        bytes.extend_from_slice(metadata.as_bytes());
        bytes.extend_from_slice(payload);
        for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
            let mut nav = binary_file_navigator(&bytes, 8)?;
            let result = nav.next_sibling_stub(&policy);
            ensure!(result.is_err() == policy.is_strict(), "Nested checksum ignored caller policy: {result:?}");
            if !policy.is_strict() {
                let stub = result?.context("Missing binary")?;
                let mut arena = DcSyntaxArena::new();
                nav.expand_frame(&stub, &mut arena, &policy)?;
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn assignment_rhs_uses_database_value_grammar() -> Result<()> {
        let mut source = DcString::new();
        source.push(DC_BEGIN_IDENTIFIER);
        source.push_str("a");
        source.push(DC_END_IDENTIFIER);
        source.push(DC_ASSIGN);
        source.push_str("A");
        let mut nav = DcDocumentNavigator::from_dc_str(&source);
        ensure!(nav.next_sibling_stub(&DcProjectionPolicy::strict()).is_err(), "Assignment accepted bare Unicode RHS");
        let mut nav = DcDocumentNavigator::from_dc_str(&source);
        let stub = nav.next_sibling_stub(&DcProjectionPolicy::forgiving())?.context("Missing assignment")?;
        ensure!(stub.outer_span.end == u64::try_from(source.len())?, "Forgiving assignment lost containment");
        ensure!(!nav.diagnostics.is_empty(), "Missing assignment diagnostic");
        ensure!(nav.next_sibling_stub(&DcProjectionPolicy::forgiving())?.is_none(), "Assignment made no progress");
        let mut deep = DcString::new();
        deep.push(DC_BEGIN_IDENTIFIER);
        deep.push_str("a");
        deep.push(DC_END_IDENTIFIER);
        deep.push(DC_ASSIGN);
        for _ in 0..500 {
            deep.push(DC_OPTIONAL_PRESENT);
        }
        deep.push(DC_OPTIONAL_ABSENT);
        let policy = DcProjectionPolicy::strict().with_max_depth(600, DiagnosticAction::Error);
        let mut nav = DcDocumentNavigator::from_dc_str(&deep);
        nav.validate_document(&policy)?;
        let mut nav = DcDocumentNavigator::from_dc_str(&deep);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn assignment_chain_small_stack_subprocess() -> Result<()> {
        const CHILD: &str = "CTB_ASSIGNMENT_STACK_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe()?)
                .args(["--exact", "navigator::tests::assignment_chain_small_stack_subprocess", "--nocapture"])
                .env(CHILD, "1")
                .output()?;
            ensure!(output.status.success(), "Assignment subprocess failed: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(());
        }
        let result = std::thread::Builder::new().stack_size(512 * 1024).spawn(|| -> Result<()> {
            let mut source = DcString::new();
            for _ in 0..256 {
                source.push(DC_BEGIN_IDENTIFIER);
                source.push_str("a");
                source.push(DC_END_IDENTIFIER);
                source.push(DC_ASSIGN);
            }
            source.push(DC_OPTIONAL_ABSENT);
            let policy = DcProjectionPolicy::forgiving_quiet().with_max_depth(512, DiagnosticAction::Ignore);
            let mut nav = DcDocumentNavigator::from_dc_str(&source);
            let stub = nav.next_sibling_stub(&policy)?.context("Missing chain")?;
            ensure!(stub.outer_span.end == u64::try_from(source.len())?, "Chain truncated");
            ensure!(!nav.diagnostics.is_empty(), "Missing chain diagnostic");
            let mut nav = DcDocumentNavigator::from_dc_str(&source);
            ensure!(nav.next_sibling_stub(&DcProjectionPolicy::strict()).is_err(), "Invalid chain accepted");
            Ok(())
        })?.join().map_err(|_| anyhow::anyhow!("Assignment worker panicked"))?;
        result
    }

    #[crate::ctb_test]
    fn file_binary_integrity_shallow_large_span_stays_lazy() -> Result<()> {
        use std::io::Write;

        let payload_len = 8_589_934_592_u64;
        let mut header = DcMstring::new().as_bytes().to_vec();
        header.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        header.push(crate::dc_mixed::TYPE_BINARY_SHA256);
        header.extend_from_slice(&u128::from(payload_len).to_be_bytes());
        header.extend_from_slice(&[0; 32]);
        let payload_start = u64::try_from(header.len())?;
        let end = payload_start.checked_add(payload_len).context("File length overflow")?;
        let mut file = tempfile::tempfile()?;
        file.write_all(&header)?;
        file.set_len(end)?;
        let mut nav = DcDocumentNavigator::from_dcmm_file(file, 0)?;
        let stub = nav.next_sibling_stub(&DcProjectionPolicy::strict())?.context("Missing binary span")?;
        let DcFrameHeaderInfo::Binary { payload_span, checksum, .. } = stub.header_info else {
            bail!("Expected binary header");
        };
        ensure!(payload_span == Some(DcByteSpan::new(payload_start, end)), "Incorrect large span");
        ensure!(checksum.is_some(), "Shallow scan discarded checksum");
        ensure!(nav.position() == end, "Shallow scan did not skip payload");
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_policy_and_lazy_payloads() -> Result<()> {
        for size in [0, 4095, 4096, 4097, 131073] {
            let payload = vec![42; size];
            for (metadata_encoding, algorithm) in [
                (false, SupportedHashFormat::Sha256),
                (true, SupportedHashFormat::Sha256),
                (true, SupportedHashFormat::XxHash64),
            ] {
                for corrupt in [false, true] {
                    let mut bytes = file_binary_document(&payload, algorithm, metadata_encoding)?;
                    if corrupt {
                        if let Some(last) = bytes.last_mut().filter(|_| !payload.is_empty()) {
                            *last ^= 1;
                        } else {
                            continue;
                        }
                    }
                    for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
                        let mut nav = binary_file_navigator(&bytes, 8)?;
                        let stub = nav.next_sibling_stub(&policy)?.context("Missing binary stub")?;
                        let saved_cursor = nav.position();
                        let mut arena = DcSyntaxArena::new();
                        let result = nav.expand_frame(&stub, &mut arena, &policy);
                        ensure!(nav.position() == saved_cursor, "Expansion changed navigator cursor");
                        if corrupt && policy.is_strict() {
                            ensure!(result.is_err(), "Strict expansion accepted corruption");
                            nav.seek_to_stub(&stub)?;
                            ensure!(nav.validate_document_strict().is_err(), "Strict validation accepted corruption");
                            continue;
                        }
                        let root = result?;
                        let DcSyntaxKind::Binary { payload: binary, .. } =
                            &arena.get(root).context("Missing binary node")?.kind else {
                                bail!("Expected binary node");
                            };
                        ensure!(
                            matches!(binary, BinaryPayloadRef::FileSpan { .. }) == (size > 4096),
                            "Wrong eager/lazy payload boundary"
                        );
                        let before_read = match &mut nav.backend {
                            NavBackend::SeekableFile { reader, .. } => reader.stream_position()?,
                            _ => bail!("Expected file backend"),
                        };
                        let materialized = nav.materialize_binary(binary)?;
                        ensure!(materialized.len() == size, "Payload length changed");
                        if !corrupt {
                            ensure!(materialized.as_ref() == payload, "Payload bytes changed");
                        }
                        if matches!(binary, BinaryPayloadRef::FileSpan { .. }) {
                            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend else {
                                bail!("Expected file backend");
                            };
                            ensure!(reader.stream_position()? == before_read, "Materialization changed source cursor");
                        }
                        if !corrupt {
                            nav.seek_to_stub(&stub)?;
                            nav.validate_document_strict()?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_deferred_reads_and_errors_restore_cursor() -> Result<()> {
        use std::io::Write;

        let payload = vec![42; 4097];
        let bytes = file_binary_document(&payload, SupportedHashFormat::Sha256, false)?;
        for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
            let mut nav = binary_file_navigator(&bytes, 8)?;
            let mut arena = DcSyntaxArena::new();
            let root = nav.parse_node_into_arena(&mut arena, &policy)?;
            let DcSyntaxKind::Binary { payload: binary, .. } =
                &arena.get(root).context("Missing binary node")?.kind else {
                    bail!("Expected binary node");
                };
            let BinaryPayloadRef::FileSpan { file_offset, .. } = binary else {
                bail!("Expected deferred payload");
            };
            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend else {
                bail!("Expected file backend");
            };
            let saved = reader.stream_position()?;
            let mut writer = reader.get_ref().try_clone()?;
            writer.seek(SeekFrom::Start(*file_offset))?;
            writer.write_all(&[43])?;
            reader.seek(SeekFrom::Start(saved))?;
            let result = nav.materialize_binary(binary);
            ensure!(result.is_err() == policy.is_strict(), "Deferred checksum policy ignored");
            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend else {
                bail!("Expected file backend");
            };
            ensure!(reader.stream_position()? == saved, "Checksum error changed source cursor");
            writer.set_len(file_offset.checked_add(1).context("Truncation offset overflow")?)?;
            reader.seek(SeekFrom::Start(saved))?;
            ensure!(nav.materialize_binary(binary).is_err(), "Truncation was demoted to a warning");
            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend else {
                bail!("Expected file backend");
            };
            ensure!(reader.stream_position()? == saved, "Read error changed source cursor");
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_metadata_bounds_before_allocation() -> Result<()> {
        for (metadata_len, data_len) in [
            (u128::MAX, 0),
            (u128::from(u64::MAX), 0),
            (u128::try_from(usize::MAX)?, 0),
            (1, u128::from(u64::MAX)),
            (1, 1),
            (0, 1),
        ] {
            let mut bytes = DcMstring::new().as_bytes().to_vec();
            bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
            bytes.push(crate::dc_mixed::TYPE_BINARY_METADATA);
            bytes.extend_from_slice(&data_len.to_be_bytes());
            bytes.extend_from_slice(&metadata_len.to_be_bytes());
            for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
                let mut nav = binary_file_navigator(&bytes, 0)?;
                ensure!(nav.next_sibling_stub(&policy).is_err(), "Forged lengths accepted");
            }
        }
        let bytes = file_binary_document(&[42; 8], SupportedHashFormat::Sha256, true)?;
        let mut nav = binary_file_navigator(&bytes, 0)?;
        let stub = nav.next_sibling_stub(&DcProjectionPolicy::strict())?.context("Missing binary")?;
        let DcFrameHeaderInfo::Binary { payload_span: Some(payload_span), .. } = stub.header_info else {
            bail!("Missing binary span");
        };
        let metadata_start = u64::try_from(DCMM_HEADER_LEN)?
            .checked_add(u64::try_from(crate::dc_mixed::DC_203_BYTES.len())?)
            .and_then(|offset| offset.checked_add(33)).context("Metadata offset overflow")?;
        let metadata_len = payload_span.start.checked_sub(metadata_start).context("Invalid metadata span")?;
        for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
            let mut nav = binary_file_navigator(&bytes, 0)?;
            let error = nav.next_sibling_stub(&policy.clone().with_max_binary_metadata_bytes(metadata_len.checked_sub(1).context("Empty metadata")?))
                .expect_err("Caller metadata limit ignored");
            ensure!(error.to_string().contains("resource limit"), "Unexpected metadata limit error: {error}");
            let mut nav = binary_file_navigator(&bytes, 0)?;
            ensure!(nav.next_sibling_stub(&policy.with_max_binary_metadata_bytes(metadata_len))?.is_some(), "Exact metadata budget rejected");
        }
        let mut nav = binary_file_navigator(&bytes, 0)?;
        nav.active_span.end = nav.active_span.end.checked_sub(1).context("Invalid test span")?;
        ensure!(nav.next_sibling_stub(&DcProjectionPolicy::forgiving()).is_err(), "Payload escaped active span");

        let mut header = DcMstring::new().as_bytes().to_vec();
        header.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        header.push(crate::dc_mixed::TYPE_BINARY_METADATA);
        header.extend_from_slice(&0_u128.to_be_bytes());
        let metadata_len = MAX_BINARY_METADATA_BYTES.checked_add(1).context("Metadata limit overflow")?;
        header.extend_from_slice(&u128::from(metadata_len).to_be_bytes());
        let mut nav = binary_file_navigator(&header, 0)?;
        let file_len = u64::try_from(header.len())?.checked_add(metadata_len).context("File length overflow")?;
        let NavBackend::SeekableFile { reader, end_offset, .. } = &mut nav.backend else {
            bail!("Expected file backend");
        };
        reader.get_ref().set_len(file_len)?;
        *end_offset = file_len;
        nav.active_span.end = file_len;
        let error = nav.next_sibling_stub(&DcProjectionPolicy::forgiving()).expect_err("Oversized metadata accepted");
        ensure!(error.to_string().contains("resource limit"), "Missing explicit resource limit error: {error}");
        Ok(())
    }

    fn dc_string_from_tokens(tokens: &[u128]) -> DcString {
        let mut s = DcString::with_capacity(tokens.len().saturating_mul(6));
        for &tok in tokens {
            s.push(DcChar::from_u128(tok));
        }
        s
    }

    fn eval_tokens_complete(
        tokens: &[u128],
        pattern: &SyntaxPattern,
        self_dc: Option<u128>,
    ) -> Result<(bool, SyntaxEvalContext<'static>)> {
        let resolver = default_syntax_resolver();
        let s = dc_string_from_tokens(tokens);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, self_dc);
        let matched = nav.eval_pattern(pattern, &mut ctx)?;
        let complete =
            matched && !ctx.has_errors && nav.position() == nav.active_span().end;
        Ok((complete, ctx))
    }

    fn literal(text: &str) -> Vec<u128> {
        let mut tokens = vec![
            DcChar::from_short(260).to_u128(),
            DcChar::from_short(262).to_u128(),
            DcChar::from_short(264).to_u128(),
            DcChar::from_short(263).to_u128(),
        ];
        tokens.extend(text.chars().map(u128::from));
        tokens.push(DcChar::from_short(261).to_u128());
        tokens
    }

    #[crate::ctb_test]
    fn constrained_strings_enforce_ascii_and_exact_length() -> Result<()> {
        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        for (text, valid) in [
            ("TEXT", true),
            ("    ", true),
            ("~ !A", true),
            ("", false),
            ("ABC", false),
            ("ABCDE", false),
            ("abc\n", false),
            ("abc\u{7f}", false),
            ("abc\u{e9}", false),
        ] {
            let tokens = literal(text);
            let (complete, _) = eval_tokens_complete(&tokens, &rule.pattern, None)?;
            ensure!(
                complete == valid,
                "Wrong string constraint result for {text:?}"
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn malformed_or_depth_limited_constraints_fail_explicitly() -> Result<()> {
        let tokens = literal("TEXT");
        let s = dc_string_from_tokens(&tokens);
        let resolver = default_syntax_resolver();
        let invalid = SyntaxTerm::NamedConstruct {
            name: "string".to_owned(),
            subtype: Some("]".to_owned()),
            capture_var: None,
        };
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(
            !nav.eval_term_single(&invalid, &mut ctx)?,
            "Malformed string constraint was ignored"
        );
        ensure!(
            ctx.has_errors && !ctx.diagnostics.is_empty(),
            "Missing constraint error"
        );

        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ctx.max_depth = 0;
        ensure!(
            !nav.eval_pattern(&rule.pattern, &mut ctx)?,
            "String constraint exceeded depth limit"
        );
        ensure!(ctx.has_errors, "Missing depth error");

        let (complete, _) =
            eval_tokens_complete(&[u128::from('A')], &rule.pattern, None)?;
        ensure!(!complete, "Unframed string bypassed constraint");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_expression_data_boundaries() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();

        let cases: &[(&str, Vec<u128>, bool)] = &[
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(261)],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(261),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(255),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255), 65, sdc(261)],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(279),
                    sdc(270),
                    65,
                    sdc(271),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(261),
                    sdc(261),
                ],
                true,
            ),
            ("[string]", vec![], false),
            ("[string]", vec![65], false),
            ("[string]", vec![sdc(260), sdc(261)], false),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255)],
                false,
            ),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255), sdc(261)],
                false,
            ),
            ("[identifier]", vec![sdc(270), 65, sdc(271)], true),
            (
                "[identifier]",
                vec![sdc(270), sdc(255), sdc(271), sdc(271)],
                true,
            ),
            (
                "[identifier]",
                vec![sdc(270), sdc(255), sdc(255), sdc(271)],
                true,
            ),
            ("[identifier]", vec![], false),
            ("[identifier]", vec![sdc(270), sdc(271)], false),
            ("[identifier]", vec![sdc(270), 65, sdc(255)], false),
            ("[value]", vec![], false),
            ("[value]", vec![65], false),
            ("[value]", vec![sdc(270), 65, sdc(271)], false),
            ("[value]", vec![sdc(276), sdc(270), 65, sdc(271)], true),
            (
                "[value]",
                vec![
                    sdc(276),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(270),
                    65,
                    sdc(271),
                ],
                true,
            ),
            (
                "[value]",
                vec![sdc(262), sdc(264), sdc(263), sdc(270), 65, sdc(271)],
                false,
            ),
            ("[value]", vec![sdc(279), sdc(270), 65, sdc(271)], true),
            ("[value]", vec![sdc(279)], false),
            ("[statement]", vec![], false),
            ("[statement]", vec![65], false),
            (
                "[statement]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(261)],
                true,
            ),
            ("[statement]", vec![sdc(279), sdc(270), 65, sdc(271)], true),
            (
                "[statement]",
                vec![
                    sdc(270),
                    65,
                    sdc(271),
                    sdc(269),
                    sdc(276),
                    sdc(270),
                    66,
                    sdc(271),
                ],
                true,
            ),
            ("[statement]", vec![sdc(270), 65, sdc(271), sdc(269)], false),
            (
                "258:",
                vec![sdc(258), sdc(279), sdc(270), 65, sdc(271), sdc(259)],
                true,
            ),
            (
                "258:",
                vec![
                    sdc(258),
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(259),
                    sdc(261),
                    sdc(259),
                ],
                true,
            ),
            ("258:", vec![sdc(258), sdc(259)], false),
            ("315:", vec![sdc(315), sdc(276), sdc(270), 65, sdc(271)], true),
            (
                "[checksum]",
                vec![sdc(203), fmt(245), sdc(127), sdc(128), sdc(204)],
                true,
            ),
            ("[checksum]", vec![sdc(203), fmt(245), sdc(204)], true),
            ("[checksum]", vec![sdc(203), sdc(127), sdc(128), sdc(204)], false),
            ("[checksum]", vec![sdc(203), 9999, sdc(127), sdc(204)], false),
        ];
        for (syntax, stream, expected) in cases {
            let rule = parse_dc_syntax(syntax)?;
            let (complete, _) = eval_tokens_complete(stream, &rule.pattern, None)?;
            assert_eq!(complete, *expected, "{syntax}: {stream:?}");
        }

        let rule = parse_dc_syntax("[value]")?;
        let prefix = vec![
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            sdc(255),
            sdc(261),
            sdc(261),
        ];
        let mut stream = prefix.clone();
        stream.extend([sdc(279), sdc(270), 65, sdc(271)]);
        let s = dc_string_from_tokens(&stream);
        let prefix_s = dc_string_from_tokens(&prefix);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**default_syntax_resolver(), None);
        assert!(nav.eval_pattern(&rule.pattern, &mut ctx)?);
        assert!(!ctx.has_errors);
        assert_eq!(nav.position(), u64::try_from(prefix_s.len())?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_data_byte_constraints() -> Result<()> {
        let sdc = |id| DcChar::from_short(id).to_u128();
        for (digits, expected) in [
            (vec![], false),
            (vec![127, 127, 127, 195], false),
            (vec![127, 127, 127, 127, 127, 127, 195, 195], true),
            (vec![127, 127, 127, 127, 127, 127, 127, 195], false),
            (vec![127; 8], false),
            (vec![127, 127, 127, 127, 127, 128, 195, 195], false),
        ] {
            for typed in [false, true] {
                let mut stream = vec![sdc(203)];
                if typed {
                    stream.push(
                        FormatId::Wtf8
                            .dc_id()
                            .context("Missing WTF-8 format identifier")?,
                    );
                }
                stream.extend(digits.iter().map(|id| sdc(*id)));
                stream.push(sdc(204));
                for syntax in ["[data:bytes=4]", "[data:f84;bytes=4]"] {
                    let rule = parse_dc_syntax(syntax)?;
                    let (complete, _) =
                        eval_tokens_complete(&stream, &rule.pattern, None)?;
                    assert_eq!(
                        complete,
                        expected && (typed || syntax == "[data:bytes=4]"),
                        "{syntax}: {stream:?}"
                    );
                    let mut wrapped = vec![sdc(397), sdc(310), sdc(275)];
                    wrapped.extend_from_slice(&stream);
                    wrapped.push(sdc(311));
                    let (wrapped_ok, _) =
                        eval_tokens_complete(&wrapped, &rule.pattern, None)?;
                    assert_eq!(
                        wrapped_ok,
                        expected && (typed || syntax == "[data:bytes=4]"),
                        "wrapped {syntax}: {wrapped:?}"
                    );
                    if syntax == "[data:bytes=4]" {
                        let finder = parse_dc_syntax(
                            "~ ([string:[asciiprintable]{4}] | [data:bytes=4])",
                        )?;
                        let mut tagged = vec![sdc(403)];
                        tagged.extend_from_slice(&wrapped);
                        let (tagged_ok, _) = eval_tokens_complete(
                            &tagged,
                            &finder.pattern,
                            Some(sdc(403)),
                        )?;
                        assert_eq!(
                            tagged_ok, expected,
                            "constrained alternation: {tagged:?}"
                        );
                    }
                }
            }
        }
        for syntax in [
            "[data:bytes=]",
            "[data:bytes=-1]",
            "[data:f84;bytes=4;bytes=5]",
            "[data:f84;length=4]",
            "[data:bytes=999999999999999999999999999999]",
        ] {
            assert!(parse_dc_syntax(syntax).is_err(), "{syntax}");
        }
        let conflicting_lengths = vec![
            sdc(397),
            sdc(310),
            sdc(275),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(195),
            sdc(204),
            sdc(312),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(195),
            sdc(195),
            sdc(204),
            sdc(311),
        ];
        let rule = parse_dc_syntax("[data:bytes=4]")?;
        let (ok, _) =
            eval_tokens_complete(&conflicting_lengths, &rule.pattern, None)?;
        assert!(!ok, "A valid alternative hid an invalid payload byte length");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_mac_metadata_requires_typed_filename_and_rectangle() -> Result<()> {
        let resolver = default_syntax_resolver();
        let sdc = |id| DcChar::from_short(id).to_u128();
        let filename = FormatId::FileName
            .dc_id()
            .context("Missing filename format identifier")?;
        let rectangle = FormatId::QuickDrawRect
            .dc_id()
            .context("Missing rectangle format identifier")?;
        let text = vec![sdc(260), sdc(262), sdc(264), sdc(263), 65, sdc(261)];
        let number = ctb_formats_dcdata::u128_to_dc_number_chars(1)
            .into_iter()
            .map(DcChar::to_u128)
            .collect::<Vec<_>>();
        for (tag, format, payload) in
            [(496, filename, text), (510, rectangle, number.repeat(4))]
        {
            let rule = resolver
                .resolve_rule(&CharTarget::Dc(tag))
                .context("Missing Mac metadata syntax rule")?;
            for framed in [false, true] {
                let mut stream = vec![sdc(tag)];
                if framed {
                    stream.push(sdc(511));
                }
                stream.push(format);
                stream.extend_from_slice(&payload);
                if framed {
                    stream.push(sdc(512));
                }
                let (ok, _) = eval_tokens_complete(
                    &stream,
                    &rule.pattern,
                    Some(sdc(tag)),
                )?;
                assert_eq!(ok, framed, "{stream:?}");
            }
            let mut bare = vec![sdc(tag)];
            bare.extend_from_slice(&payload);
            let (bare_ok, _) =
                eval_tokens_complete(&bare, &rule.pattern, Some(sdc(tag)))?;
            assert!(!bare_ok);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_typed_literal_headers_el_types() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let rule = parse_dc_syntax(":~ [script:.EL Types] 263")?;
        for type_dc in [264u32, 275, 278, 280, 306] {
            let stream = vec![sdc(262), sdc(type_dc), sdc(263)];
            let (ok, _) =
                eval_tokens_complete(&stream, &rule.pattern, Some(sdc(262)))?;
            assert!(ok, "Failed for type Dc {type_dc}");
        }
        let invalid_stream = vec![sdc(262), sdc(246), sdc(263)];
        let (ok, _) = eval_tokens_complete(
            &invalid_stream,
            &rule.pattern,
            Some(sdc(262)),
        )?;
        assert!(!ok);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_non_evaluating_document_parser() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();

        // 1. Parse idiomatic-hello-world.sems: say 'Hello, World!'
        let stream = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            57,
            86,
            93,
            93,
            96,
            30,
            18,
            72,
            96,
            99,
            93,
            85,
            19,
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&stream)?;
        let s = dc_string_from_tokens(&stream);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &DcProjectionPolicy::strict())?;
        assert!(!arena.roots().is_empty());
        let root_id = *arena.roots().first().context("Missing root")?;
        let root = arena.node(root_id).context("Missing root node")?;
        assert!(!root.children().is_empty());

        // 2. Mangled / broken document: unclosed literal followed by unparsed tokens
        let mangled = vec![sdc(260), sdc(262), sdc(264), sdc(263), 10, 20, 99, 100];
        assert!(validate_dc_tokens_strict(&mangled).is_err());
        let mangled_s = dc_string_from_tokens(&mangled);
        let mut mangled_nav = DcDocumentNavigator::from_dc_str(&mangled_s);
        let mut mangled_arena = DcSyntaxArena::new();
        mangled_nav.parse_document_into_arena(
            &mut mangled_arena,
            &DcProjectionPolicy::forgiving(),
        )?;
        assert!(!mangled_arena.nodes().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_strict_document_syntax_validation() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();

        // 1. Valid operation session document (Dc 547 ... 548)
        let valid_session = vec![sdc(547), sdc(270), 99, 115, 99, sdc(271), sdc(548)];
        validate_dc_tokens_strict(&valid_session)?;

        // 2. Dangling closing delimiter: unexpected Dc 548 without opener
        let dangling_closer = vec![sdc(548)];
        let err = validate_dc_tokens_strict(&dangling_closer);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Unexpected closing delimiter"));

        // 3. Unclosed container: Dc 547 without Dc 548
        let unclosed_session =
            vec![sdc(547), sdc(546), sdc(270), 99, 115, 99, sdc(271)];
        assert!(validate_dc_tokens_strict(&unclosed_session).is_err());

        // 4. Checksum section container: Dc 543 ... 544 [545:]
        let mut body = DcMstring::new();
        body.push_char(DC_BEGIN_IDENTIFIER);
        body.push_str("d");
        body.push_char(DC_END_IDENTIFIER);
        let mut valid_checksummed = DcMstring::new();
        valid_checksummed.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        for ch in body.to_dc_string()?.chars() {
            valid_checksummed.push_char(ch);
        }
        valid_checksummed.push_char(ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION);
        valid_checksummed.push_char(DC_SECTION_DIGEST);
        valid_checksummed.push_checksum(ctb_utilities::format_id::DC_SHA256, &ctb_formats_checksum::sha256(body.payload_bytes()))?;
        validate_dc_mixed_strict(&valid_checksummed)?;

        // 5. Dangling checksum section closer: Dc 544
        assert!(validate_dc_tokens_strict(&[sdc(544)]).is_err());

        // 6. Number container: Dc 6 ... Dc 7
        assert!(validate_dc_tokens_strict(&[sdc(7)]).is_err());

        // 7. Date timestamp with Dc 390 ... 391
        let valid_date = vec![
            sdc(330),
            sdc(390),
            fmt(487),
            sdc(6),
            fmt(199),
            sdc(127),
            sdc(7),
            sdc(391),
        ];
        validate_dc_tokens_strict(&valid_date)?;

        // 8. Raw number without Dc 390 fails strict validation
        let raw_number_date = vec![sdc(330), sdc(6), fmt(199), sdc(127), sdc(7)];
        assert!(validate_dc_tokens_strict(&raw_number_date).is_err());

        // 9. Host local network IP address (Dc 346: :~ [ipaddr]) with IPv4 (f490) and IPv6 (f491)
        for fmt_id in [490, 491] {
            let valid_ip = vec![
                sdc(346),
                sdc(511),
                fmt(fmt_id),
                sdc(6),
                fmt(199),
                sdc(127),
                sdc(7),
                sdc(512),
            ];
            validate_dc_tokens_strict(&valid_ip)?;
        }

        // 10. Raw number without 511/f490/f491 fails IP address strict validation
        let raw_number_ip = vec![sdc(346), sdc(6), fmt(199), sdc(127), sdc(7)];
        assert!(validate_dc_tokens_strict(&raw_number_ip).is_err());

        // 11. File flags description: Dc 388 ... 389 with [list:Semantic:fileflag]
        let valid_flags_list = vec![
            sdc(388),
            sdc(310),
            sdc(275),
            sdc(409),
            sdc(312),
            sdc(410),
            sdc(311),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_list)?;

        // 12. File flags description: Dc 388 ... 389 with raw flags Dc 539
        let valid_flags_raw = vec![
            sdc(388),
            sdc(539),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(204),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_raw)?;

        // 13. File flags description: Dc 388 ... 389 with equivalent representations Dc 397
        let valid_flags_equiv = vec![
            sdc(388),
            sdc(397),
            sdc(310),
            sdc(275),
            sdc(310),
            sdc(275),
            sdc(409),
            sdc(311),
            sdc(312),
            sdc(539),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(204),
            sdc(311),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_equiv)?;

        // 14. File flags description: empty Dc 388 389 is valid in strict mode
        let valid_flags_empty = vec![sdc(388), sdc(389)];
        validate_dc_tokens_strict(&valid_flags_empty)?;

        // 15. Mac OS Finder flags with empty flags: 406 388 389 is valid in strict mode
        let valid_finder_flags_empty = vec![sdc(406), sdc(388), sdc(389)];
        validate_dc_tokens_strict(&valid_finder_flags_empty)?;

        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_key_value_pair_arrays() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let pair = vec![sdc(514), 107, sdc(312), 118, sdc(515)];
        validate_dc_tokens_strict(&pair)?;
        let resolver = default_syntax_resolver();
        let pair_rule = resolver
            .resolve_rule(&CharTarget::Dc(514))
            .context("Missing pair rule")?;
        let (ok, _) =
            eval_tokens_complete(&pair, &pair_rule.pattern, Some(sdc(514)))?;
        assert!(ok);

        for (begin, end) in [(559, 560), (313, 314)] {
            let empty = vec![sdc(begin), sdc(275), sdc(275), sdc(end)];
            validate_dc_tokens_strict(&empty)?;
            let mut map = vec![sdc(begin), sdc(275), sdc(275)];
            if begin == 559 {
                map.extend_from_slice(&pair);
                map.extend_from_slice(&pair);
            } else {
                map.extend([107, sdc(312), 118, sdc(312), 107, sdc(312), 118]);
            }
            map.push(sdc(end));
            validate_dc_tokens_strict(&map)?;
        }

        let mut nested = vec![sdc(514)];
        nested.extend_from_slice(&pair);
        nested.push(sdc(312));
        nested.extend_from_slice(&pair);
        nested.push(sdc(515));
        validate_dc_tokens_strict(&nested)?;

        let quoted = vec![
            sdc(514),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            sdc(515),
            sdc(312),
            sdc(514),
            sdc(261),
            sdc(312),
            118,
            sdc(515),
        ];
        validate_dc_tokens_strict(&quoted)?;
        let escaped = vec![
            sdc(514),
            sdc(255),
            sdc(515),
            sdc(312),
            sdc(255),
            sdc(312),
            sdc(515),
        ];
        validate_dc_tokens_strict(&escaped)?;

        for broken in [
            vec![sdc(514), 107, sdc(312), 118],
            vec![sdc(514), sdc(312), 118, sdc(515)],
            vec![sdc(514), 107, sdc(312), sdc(515)],
            vec![sdc(559), sdc(275), sdc(275), 107, sdc(312), 118, sdc(560)],
            vec![
                sdc(559),
                sdc(275),
                sdc(275),
                sdc(514),
                107,
                sdc(312),
                118,
                sdc(560),
            ],
            vec![sdc(313), sdc(275), sdc(275), 107, sdc(314)],
        ] {
            assert!(validate_dc_tokens_strict(&broken).is_err(), "{broken:?}");
        }

        let mut legacy_tuple_key = vec![sdc(313), sdc(275), sdc(275)];
        legacy_tuple_key.extend_from_slice(&pair);
        legacy_tuple_key.extend([sdc(312), 118, sdc(314)]);
        validate_dc_tokens_strict(&legacy_tuple_key)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_resolver_and_expansion_bounds() -> Result<()> {
        let resolver = DatasetRuleResolver::load();

        assert!(resolver.resolve_named_type("list").is_some());
        assert!(resolver.resolve_named_type("key_value").is_some());
        assert!(resolver.resolve_named_type("string").is_some());

        let recursive_rule = parse_dc_syntax(":[value]")?;
        let s = dc_string_from_tokens(&[1u128, 2, 3]);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&resolver, None);
        ctx.max_depth = 5;
        ctx.depth = 5;
        assert!(!nav.eval_pattern(&recursive_rule.pattern, &mut ctx)?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_numeric_and_list_constraints_across_backends_and_evaluated_inputs() -> Result<()> {
        use std::io::Write as _;
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| {
            FormatId::from_shorthand(&format!("f{id}"))
                .and_then(|f| f.dc_id())
                .unwrap()
        };
        let resolver = default_syntax_resolver();
        let strict = DcProjectionPolicy::strict();

        let check_all_backends = |tokens: &[u128], expected_ok: bool| -> Result<()> {
            let dc_str = dc_string_from_tokens(tokens);
            let mut mstr = DcMstring::new();
            for &t in tokens {
                mstr.push_char(t);
            }
            let mut file = tempfile::tempfile()?;
            file.write_all(mstr.as_bytes())?;

            let mut nav_str = DcDocumentNavigator::from_dc_str(&dc_str);
            let mut nav_mstr = DcDocumentNavigator::from_mstr(&mstr);
            let mut nav_file = DcDocumentNavigator::from_dcmm_file(file, 0)?;

            let res_str = nav_str.validate_document_with_resolver(&strict, resolver.as_ref());
            let res_mstr = nav_mstr.validate_document_with_resolver(&strict, resolver.as_ref());
            let res_file = nav_file.validate_document_with_resolver(&strict, resolver.as_ref());

            assert_eq!(res_str.is_ok(), expected_ok, "text backend mismatch for {tokens:?}");
            assert_eq!(res_mstr.is_ok(), expected_ok, "mixed backend mismatch for {tokens:?}");
            assert_eq!(res_file.is_ok(), expected_ok, "file backend mismatch for {tokens:?}");
            Ok(())
        };

        // 1. Dc 405 (Mac OS Finder label color: :~ ([number:Natural0&<8] | ...))
        // 0 (digit 127) and 7 (digit 134) succeed; 8 (digit 135) and -1 (191 128) fail.
        check_all_backends(&[sdc(405), sdc(6), fmt(199), sdc(127), sdc(7)], true)?;
        check_all_backends(&[sdc(405), sdc(6), fmt(199), sdc(134), sdc(7)], true)?;
        check_all_backends(&[sdc(405), sdc(6), fmt(199), sdc(135), sdc(7)], false)?;
        check_all_backends(&[sdc(405), sdc(6), fmt(199), sdc(191), sdc(128), sdc(7)], false)?;

        // 2. Dc 388 (File flags: :~ ([list:Semantic:fileflag] | [539:])? 389)
        // Valid fileflag 409 succeeds; non-fileflag 403 or number fails.
        check_all_backends(
            &[sdc(388), sdc(310), sdc(275), sdc(409), sdc(312), sdc(410), sdc(311), sdc(389)],
            true,
        )?;
        check_all_backends(
            &[sdc(388), sdc(310), sdc(275), sdc(403), sdc(311), sdc(389)],
            false,
        )?;
        check_all_backends(
            &[sdc(388), sdc(310), sdc(275), sdc(6), fmt(199), sdc(127), sdc(7), sdc(311), sdc(389)],
            false,
        )?;

        // 3. Format f504 (Point: :~ [number $dimensions] [list($dimensions):number])
        // 2D point with 2 numbers succeeds; 2D point with 1 number or non-number element fails.
        let num_2 = [sdc(6), fmt(199), sdc(129), sdc(7)];
        let num_10 = [sdc(6), fmt(199), sdc(137), sdc(7)];
        let num_20 = [sdc(6), fmt(199), sdc(147), sdc(7)];
        let mut valid_point = vec![sdc(511), fmt(504)];
        valid_point.extend_from_slice(&num_2);
        valid_point.extend([sdc(310), fmt(199)]);
        valid_point.extend_from_slice(&num_10);
        valid_point.push(sdc(312));
        valid_point.extend_from_slice(&num_20);
        valid_point.extend([sdc(311), sdc(512)]);
        check_all_backends(&valid_point, true)?;

        let mut wrong_len_point = vec![sdc(511), fmt(504)];
        wrong_len_point.extend_from_slice(&num_2);
        wrong_len_point.extend([sdc(310), fmt(199)]);
        wrong_len_point.extend_from_slice(&num_10);
        wrong_len_point.extend([sdc(311), sdc(512)]);
        check_all_backends(&wrong_len_point, false)?;

        let mut wrong_elem_point = vec![sdc(511), fmt(504)];
        wrong_elem_point.extend_from_slice(&num_2);
        wrong_elem_point.extend([sdc(310), sdc(275)]);
        wrong_elem_point.extend_from_slice(&num_10);
        wrong_elem_point.extend([sdc(312), sdc(517), sdc(311), sdc(512)]);
        check_all_backends(&wrong_elem_point, false)?;

        // 4. Unknown or invalid number bounds and non-number subtypes must reject rather than fall through.
        let val_3 = malachite::Integer::from(3);
        assert!(number_satisfies_subtype(&val_3, "Natural0&<8", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&malachite::Integer::from(8), "Natural0&<8", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "UnknownBound", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "<not_a_number", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "Natural0&", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "boolean", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "f0", resolver.as_ref()));

        // 5. Evaluated input (Dc 256 stream-spliced expression) across all backends:
        // Splicing 7 (digit 134) satisfies Dc 405 ([number:Natural0&<8]); splicing 8 (digit 135) fails.
        for (digit, expected_ok) in [(sdc(134), true), (sdc(135), false)] {
            let spliced_label = vec![
                sdc(405),
                sdc(256),
                sdc(258),
                sdc(260),
                sdc(262),
                sdc(264),
                sdc(263),
                sdc(6),
                fmt(199),
                digit,
                sdc(7),
                sdc(261),
                sdc(259),
            ];
            check_all_backends(&spliced_label, expected_ok)?;
        }

        // Splicing $dimensions = 2 via Dc 256 into Format f504 ([number $dimensions] [list($dimensions):number])
        // succeeds for a 2-element list and fails for a 1-element list.
        let mut spliced_dim_ok = vec![
            sdc(511),
            fmt(504),
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
        ];
        spliced_dim_ok.extend_from_slice(&num_2);
        spliced_dim_ok.extend([sdc(261), sdc(259), sdc(310), fmt(199)]);
        spliced_dim_ok.extend_from_slice(&num_10);
        spliced_dim_ok.push(sdc(312));
        spliced_dim_ok.extend_from_slice(&num_20);
        spliced_dim_ok.extend([sdc(311), sdc(512)]);
        check_all_backends(&spliced_dim_ok, true)?;

        let mut spliced_dim_bad = vec![
            sdc(511),
            fmt(504),
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
        ];
        spliced_dim_bad.extend_from_slice(&num_2);
        spliced_dim_bad.extend([sdc(261), sdc(259), sdc(310), fmt(199)]);
        spliced_dim_bad.extend_from_slice(&num_10);
        spliced_dim_bad.extend([sdc(311), sdc(512)]);
        check_all_backends(&spliced_dim_bad, false)?;

        Ok(())
    }

    #[crate::ctb_test]
    fn test_el_routines_types_identifiers_and_assign_equivalence() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();
        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();

        // 1. .EL Routines derived parameter syntax (256, 265, 268, 307):
        //    - 256: void lang.say([string] text) -> accepts 258 [string] 259, rejects 258 [number] 259 or extra 258..259
        let valid_say = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            65,
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&valid_say)?;

        let wrong_type_say = vec![
            sdc(256),
            sdc(258),
            sdc(6),
            fmt(199),
            sdc(127),
            sdc(7),
            sdc(259),
        ];
        assert!(validate_dc_tokens_strict(&wrong_type_say).is_err());
        let wrong_type_say_doc = dc_string_from_tokens(&wrong_type_say);
        let mut forgiving_nav = DcDocumentNavigator::from_dc_str(&wrong_type_say_doc);
        let mut forgiving_arena = DcSyntaxArena::new();
        forgiving_nav.parse_document_into_arena(&mut forgiving_arena, &forgiving)?;
        assert!(!forgiving_arena.diagnostics().is_empty());

        let extra_param_say = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            65,
            sdc(261),
            sdc(259),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            66,
            sdc(261),
            sdc(259),
        ];
        assert!(validate_dc_tokens_strict(&extra_param_say).is_err());

        // 2. Identifiers (270..271) with optional leading [type]?:
        //    - Typed with 262 280 263 (Routine): satisfies [identifier:Routine] and [Routine name]
        let typed_routine_ident = vec![
            sdc(270),
            sdc(262),
            sdc(280),
            sdc(263),
            102,
            111,
            111,
            sdc(271),
        ];
        validate_dc_tokens_strict(&typed_routine_ident)?;
        let rule_routine_ident = parse_dc_syntax(":[identifier:Routine]")?;
        assert!(eval_tokens_complete(&typed_routine_ident, &rule_routine_ident.pattern, None)?.0);
        let rule_routine_name = parse_dc_syntax(":[Routine name]")?;
        assert!(eval_tokens_complete(&typed_routine_ident, &rule_routine_name.pattern, None)?.0);

        //    - Typed with 264 (String): satisfies [identifier], rejected by [identifier:Routine]
        let typed_str_ident = vec![sdc(270), sdc(264), 102, 111, 111, sdc(271)];
        validate_dc_tokens_strict(&typed_str_ident)?;
        assert!(!eval_tokens_complete(&typed_str_ident, &rule_routine_ident.pattern, None)?.0);

        //    - Escaped leading type token (255 264) is part of the identifier name, not a type header
        let mut raw_ident_with_leading_type = DcMstring::new();
        raw_ident_with_leading_type.push_char(DcChar::from_short(264));
        raw_ident_with_leading_type.push_char(DcChar::from_char('x'));
        let mut encoded_ident_doc = DcMstring::new();
        crate::encode_dc_identifier(&mut encoded_ident_doc, &raw_ident_with_leading_type)?;
        let decoded_ident: DcMstring = encoded_ident_doc.decode(&strict)?;
        assert_eq!(decoded_ident, raw_ident_with_leading_type);

        // 3. Dc 307: void lang.invoke([Routine] routine | [Routine name] routineName)
        //    - Accepts inline [Routine] literal (260 262 280 263 266 261)
        let invoke_inline_routine = vec![
            sdc(307),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(280),
            sdc(263),
            sdc(266),
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&invoke_inline_routine)?;

        //    - Accepts [Routine name] (typed identifier 270 262 280 263 ... 271)
        let mut invoke_routine_name = vec![sdc(307), sdc(258)];
        invoke_routine_name.extend_from_slice(&typed_routine_ident);
        invoke_routine_name.push(sdc(259));
        validate_dc_tokens_strict(&invoke_routine_name)?;

        //    - Rejects untyped string literal (260 262 264 263 ... 261) for Dc 307
        let invoke_bad_str = vec![
            sdc(307),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            65,
            sdc(261),
            sdc(259),
        ];
        assert!(validate_dc_tokens_strict(&invoke_bad_str).is_err());

        // 4. Dc 268 (lang.assign) and Dc 269 (Assign operator) equivalence:
        //    - 268 258 270 "x" 271 259 258 [number: 99] 259 normalizes to DcSyntaxKind::Assignment
        //      and resolves 540 bound value references identically to 269
        let mut doc_268 = DcMstring::new();
        doc_268.push_char(DcChar::from_short(268));
        doc_268.push_char(DcChar::from_short(258));
        doc_268.push_char(DC_BEGIN_IDENTIFIER);
        doc_268.push_str("assigned_via_268");
        doc_268.push_char(DC_END_IDENTIFIER);
        doc_268.push_char(DcChar::from_short(259));
        doc_268.push_char(DcChar::from_short(258));
        99u64.encode_dc_mixed(&mut doc_268)?;
        doc_268.push_char(DcChar::from_short(259));
        doc_268.push_char(ctb_formats_dcdata::dc::DC_REFERENCE_BOUND_VALUE);
        doc_268.push_char(DC_BEGIN_IDENTIFIER);
        doc_268.push_str("assigned_via_268");
        doc_268.push_char(DC_END_IDENTIFIER);

        validate_dc_mixed_strict(&doc_268)?;
        let mut nav_268 = DcDocumentNavigator::from_mstr(&doc_268);
        let mut arena_268 = DcSyntaxArena::new();
        nav_268.parse_document_into_arena(&mut arena_268, &strict)?;
        let assign_root = *arena_268.roots().first().context("Missing 268 assignment root")?;
        assert!(matches!(
            arena_268.node(assign_root).context("Missing node")?.kind,
            DcSyntaxKind::Assignment { .. }
        ));
        let ref_root = *arena_268.roots().get(1).context("Missing 540 reference root")?;
        let mut proj_ctx = crate::DcProjectionContext::new(&arena_268, &strict);
        let resolved: u64 = crate::DcMixedDecode::decode_dc_node(&mut proj_ctx, ref_root)?;
        assert_eq!(resolved, 99);

        //    - Binding a Routine literal via 269 allows untyped identifier to satisfy [Routine name] in 307
        let bound_routine_doc = vec![
            sdc(270),
            102,
            110,
            sdc(271),
            sdc(269),
            sdc(260),
            sdc(262),
            sdc(280),
            sdc(263),
            sdc(266),
            sdc(261),
            sdc(307),
            sdc(258),
            sdc(270),
            102,
            110,
            sdc(271),
            sdc(259),
        ];
        validate_dc_tokens_strict(&bound_routine_doc)?;

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_bound_expressions_and_scientific_notation() {
        use std::str::FromStr;

        assert_eq!(
            parse_bound("(-2^7)"),
            Some(malachite::Integer::from(-128))
        );
        assert_eq!(
            parse_bound("-2^7"),
            Some(malachite::Integer::from(-128))
        );
        assert_eq!(
            parse_bound("-(2^15)"),
            Some(malachite::Integer::from(-32768))
        );
        assert_eq!(
            parse_bound("(-2)^8"),
            Some(malachite::Integer::from(256))
        );
        assert_eq!(
            parse_bound("-2^8"),
            Some(malachite::Integer::from(-256))
        );
        assert_eq!(
            parse_bound("(-2^127)"),
            malachite::Integer::from_str("-170141183460469231731687303715884105728").ok()
        );
        assert_eq!(
            parse_bound("(2^8 - 1)"),
            Some(malachite::Integer::from(255))
        );
        assert_eq!(
            parse_bound("1e5"),
            Some(malachite::Integer::from(100_000))
        );
        assert_eq!(
            parse_bound("-1e5"),
            Some(malachite::Integer::from(-100_000))
        );
        assert_eq!(
            parse_bound("(-1.5e3)"),
            Some(malachite::Integer::from(-1_500))
        );
        assert_eq!(
            parse_bound("1E+6"),
            Some(malachite::Integer::from(1_000_000))
        );
        assert_eq!(
            parse_bound("-1000e-2"),
            Some(malachite::Integer::from(-10))
        );
        assert_eq!(parse_bound("1.5e0"), None);
        assert_eq!(parse_bound("15e-1"), None);
    }
