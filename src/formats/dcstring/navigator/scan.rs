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

//! Low-level token, binary encapsulation, and primitive frame scanners.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{
    BinaryChecksum, BinaryPayloadRef, BufReader, CachedBalancedContainer,
    CharTarget, DcByteSpan, DcCharExt, DcDocumentNavigator,
    DcExpressionEvaluator, DcFrameHeaderInfo, DcMstring, DcProjectionPolicy,
    DcRole, DcStr, DcSyntaxKind, DefaultDcExpressionEvaluator, Digest, File,
    INLINE_BINARY_THRESHOLD, MAX_NAV_CACHE_ENTRIES, NavBackend,
    NavigatorCheckpoint, Read, Seek, SeekFrom, SyntaxEvalContext,
    SyntaxRuleResolver, SyntaxTerm, decode_dctf_buf, decode_utf_8e_128_buf,
    encode_dctf_buf,
};

pub(super) fn seek_buffered_file(
    reader: &mut BufReader<File>,
    reader_pos: &mut Option<u64>,
    target: u64,
) -> std::io::Result<()> {
    if let Some(current) = *reader_pos {
        if current == target {
            return Ok(());
        }
        if let (Ok(target_i64), Ok(current_i64)) =
            (i64::try_from(target), i64::try_from(current))
            && let Some(delta) = target_i64.checked_sub(current_i64)
        {
            if let Err(err) = reader.seek_relative(delta) {
                *reader_pos = None;
                return Err(err);
            }
            *reader_pos = Some(target);
            return Ok(());
        }
    }
    reader.seek(SeekFrom::Start(target))?;
    *reader_pos = Some(target);
    Ok(())
}

impl<'a> DcDocumentNavigator<'a> {
    pub(super) fn read_token_or_binary_at(
        &mut self,
        offset: u64,
    ) -> Result<Option<(DcChar, u64)>> {
        if offset >= self.active_span.end {
            return Ok(None);
        }
        if let Some(bytes) = self.backend.memory_bytes() {
            let pos = usize::try_from(offset)?;
            let Some(rem) = bytes.get(pos..) else {
                return Ok(None);
            };
            if rem.is_empty() {
                return Ok(None);
            };
            let Some((val, len)) = decode_dctf_buf(rem) else {
                bail!("Invalid DcTF character at byte offset {offset}");
            };
            let next_offset = offset.saturating_add(u64::try_from(len)?);
            if next_offset > self.active_span.end {
                return Ok(None);
            }
            return Ok(Some((DcChar::from_u128(val), next_offset)));
        }
        match &mut self.backend {
            NavBackend::MemoryMstr(_) | NavBackend::MemoryText(_) => {
                unreachable!()
            }
            NavBackend::SeekableFile {
                reader,
                reader_pos,
                is_v1,
                ..
            } => {
                seek_buffered_file(reader, reader_pos, offset)?;
                let mut first = [0_u8; 1];
                match reader.read(&mut first) {
                    Ok(0) => return Ok(None),
                    Ok(_) => {
                        *reader_pos = offset.checked_add(1);
                    }
                    Err(err) => {
                        *reader_pos = None;
                        return Err(err.into());
                    }
                }
                let mut buf = [0_u8; 32];
                buf[0] = first[0];
                if *is_v1 {
                    let seq_len = if first[0] < 0x80 {
                        1usize
                    } else if (first[0] & 0xE0) == 0xC0 {
                        2usize
                    } else if (first[0] & 0xF0) == 0xE0 {
                        3usize
                    } else if (first[0] & 0xF8) == 0xF0 {
                        4usize
                    } else if first[0] == 0xFF {
                        let mut second = [0_u8; 1];
                        if let Err(err) = reader.read_exact(&mut second) {
                            *reader_pos = None;
                            return Err(err)
                                .with_context(|| format!("Truncated UTF-8e128 header at offset {offset}"));
                        }
                        buf[1] = second[0];
                        ensure!(
                            (second[0] & 0xC0) == 0x80,
                            "Invalid UTF-8e128 length byte 0x{:02x} at offset {offset}",
                            second[0]
                        );
                        let l = usize::from(second[0] & 0x3F);
                        ensure!(
                            (4..=22).contains(&l),
                            "Invalid UTF-8e128 continuation count {l} at offset {offset}"
                        );
                        let total = l.saturating_add(2);
                        let tail = buf.get_mut(2..total).context(
                            "UTF-8e128 sequence length exceeds buffer",
                        )?;
                        if let Err(err) = reader.read_exact(tail) {
                            *reader_pos = None;
                            return Err(err)
                                .with_context(|| format!("Truncated UTF-8e128 sequence at offset {offset}"));
                        }
                        total
                    } else {
                        bail!(
                            "Invalid UTF-8e128 lead byte 0x{:02x} at offset {offset}",
                            first[0]
                        );
                    };
                    if seq_len > 1 && first[0] != 0xFF {
                        let tail = buf.get_mut(1..seq_len).context(
                            "UTF-8e128 sequence length exceeds buffer",
                        )?;
                        if let Err(err) = reader.read_exact(tail) {
                            *reader_pos = None;
                            return Err(err)
                                .with_context(|| format!("Truncated UTF-8e128 sequence at offset {offset}"));
                        }
                    }
                    let next_offset =
                        offset.saturating_add(u64::try_from(seq_len)?);
                    *reader_pos = Some(next_offset);
                    let slice =
                        buf.get(..seq_len).context("UTF-8e128 slice bounds")?;
                    let Some((val, _)) = decode_utf_8e_128_buf(slice) else {
                        bail!("Invalid UTF-8e128 sequence at offset {offset}");
                    };
                    if next_offset > self.active_span.end {
                        return Ok(None);
                    }
                    return Ok(Some((DcChar::from_u128(val), next_offset)));
                }
                let seq_len = if first[0] < 0x80 {
                    1usize
                } else if (first[0] & 0xE0) == 0xC0 {
                    2usize
                } else if (first[0] & 0xF0) == 0xE0 {
                    3usize
                } else if (first[0] & 0xF8) == 0xF0 {
                    4usize
                } else if (first[0] & 0xFC) == 0xF8 {
                    5usize
                } else if (first[0] & 0xFE) == 0xFC {
                    6usize
                } else if first[0] == 0xFE {
                    7usize
                } else if first[0] == 0xFF {
                    let mut second = [0_u8; 1];
                    if let Err(err) = reader.read_exact(&mut second) {
                        *reader_pos = None;
                        return Err(err).with_context(|| {
                            format!("Truncated DcTF header at offset {offset}")
                        });
                    }
                    buf[1] = second[0];
                    ensure!(
                        (second[0] & 0xC0) == 0x80,
                        "Invalid DcTF tier byte 0x{:02x} at offset {offset}",
                        second[0]
                    );
                    let tier = usize::from((second[0] >> 2) & 0x0F);
                    let total = tier.saturating_add(8);
                    let tail = buf
                        .get_mut(2..total)
                        .context("DcTF sequence length exceeds buffer")?;
                    if let Err(err) = reader.read_exact(tail) {
                        *reader_pos = None;
                        return Err(err).with_context(|| {
                            format!(
                                "Truncated DcTF sequence at offset {offset}"
                            )
                        });
                    }
                    total
                } else {
                    bail!(
                        "Invalid DcTF lead byte 0x{:02x} at offset {offset}",
                        first[0]
                    );
                };
                if seq_len > 1 && first[0] != 0xFF {
                    let tail = buf
                        .get_mut(1..seq_len)
                        .context("DcTF sequence length exceeds buffer")?;
                    if let Err(err) = reader.read_exact(tail) {
                        *reader_pos = None;
                        return Err(err).with_context(|| {
                            format!(
                                "Truncated DcTF sequence at offset {offset}"
                            )
                        });
                    }
                }
                let next_offset =
                    offset.saturating_add(u64::try_from(seq_len)?);
                *reader_pos = Some(next_offset);
                let slice = buf.get(..seq_len).context("DcTF slice bounds")?;
                let Some((val, _)) = decode_dctf_buf(slice) else {
                    bail!("Invalid DcTF sequence at offset {offset}");
                };
                if next_offset > self.active_span.end {
                    return Ok(None);
                }
                Ok(Some((DcChar::from_u128(val), next_offset)))
            }
        }
    }

    pub(super) fn try_scan_raw_binary_at(
        &mut self,
        start_offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<(u64, DcFrameHeaderInfo)>> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let start_usize = usize::try_from(start_offset)?;
                let rem = mstr
                    .as_bytes()
                    .get(start_usize..)
                    .context("Binary encapsulation start out of bounds")?;
                if !rem.starts_with(&crate::dc_mixed::DC_203_BYTES)
                    || !matches!(
                        rem.get(crate::dc_mixed::DC_203_BYTES.len()).copied(),
                        Some(
                            crate::dc_mixed::TYPE_BINARY
                                | crate::dc_mixed::TYPE_BINARY_METADATA
                                | crate::dc_mixed::TYPE_BINARY_SHA256
                        )
                    )
                {
                    return Ok(None);
                }
                let mut chunks = crate::dc_mixed::DcMixedChunks { slice: rem };
                let Some(chunk) = chunks.next() else {
                    return Ok(None);
                };
                let consumed = rem.len().saturating_sub(chunks.slice.len());
                let end_offset =
                    start_offset.saturating_add(u64::try_from(consumed)?);
                ensure!(
                    end_offset <= self.active_span.end,
                    "Binary encapsulation extends past active container span"
                );
                match chunk {
                    crate::DcMixedChunk::Binary { data, .. } => {
                        let data_len = u64::try_from(data.len())?;
                        let abs_data_start =
                            end_offset.saturating_sub(data_len);
                        Ok(Some((
                            end_offset,
                            DcFrameHeaderInfo::Binary {
                                data_type: None,
                                payload_span: Some(DcByteSpan::new(
                                    abs_data_start,
                                    end_offset,
                                )),
                                payload_len: data_len,
                                checksum: None,
                            },
                        )))
                    }
                    crate::DcMixedChunk::BinaryWithMetadata {
                        data,
                        metadata,
                    } => {
                        let data_len = u64::try_from(data.len())?;
                        let abs_data_start =
                            end_offset.saturating_sub(data_len);
                        let parsed = crate::encapsulation::parse_metadata_with_policy_and_resolver(metadata, policy, resolver)?;
                        let data_type = parsed.data_type.map(DcStr::to_owned);
                        Ok(Some((
                            end_offset,
                            DcFrameHeaderInfo::Binary {
                                data_type,
                                payload_span: Some(DcByteSpan::new(
                                    abs_data_start,
                                    end_offset,
                                )),
                                payload_len: data_len,
                                checksum: parsed.checksum_spec,
                            },
                        )))
                    }
                    crate::DcMixedChunk::Text(_) => Ok(None),
                }
            }
            NavBackend::MemoryText(_) => Ok(None),
            NavBackend::SeekableFile {
                reader,
                reader_pos,
                base_offset,
                end_offset,
                is_v1,
            } => {
                let dc203_len = if *is_v1 {
                    crate::dc_mixed::DC_203_BYTES_V1.len()
                } else {
                    crate::dc_mixed::DC_203_BYTES.len()
                };
                let after_dc203 = start_offset
                    .checked_add(u64::try_from(dc203_len)?)
                    .context("Binary encapsulation offset overflow")?;
                let source_end = self.active_span.end.min(*end_offset);
                if after_dc203 >= source_end {
                    return Ok(None);
                }
                seek_buffered_file(reader, reader_pos, after_dc203)?;
                let mut type_byte = [0_u8; 1];
                if let Err(err) = reader.read_exact(&mut type_byte) {
                    *reader_pos = None;
                    return Err(err)
                        .context("Truncated .dcmm binary encapsulation type");
                }
                *reader_pos = after_dc203.checked_add(1);
                let encap_type = type_byte[0];
                if encap_type >= 0xF4 {
                    return Ok(None);
                }
                let rel_offset =
                    usize::try_from(after_dc203.saturating_sub(*base_offset))?;
                if matches!(encap_type, 1 | 4) {
                    return Err(crate::DcMixedError::UnsupportedLegacyType {
                        offset: rel_offset,
                        type_byte: encap_type,
                    }
                    .into());
                }
                if !matches!(
                    encap_type,
                    crate::dc_mixed::TYPE_BINARY
                        | crate::dc_mixed::TYPE_BINARY_METADATA
                        | crate::dc_mixed::TYPE_BINARY_SHA256
                ) {
                    return Err(
                        crate::DcMixedError::InvalidEncapsulationType {
                            offset: rel_offset,
                            type_byte: encap_type,
                        }
                        .into(),
                    );
                }
                let after_header = after_dc203
                    .checked_add(17)
                    .context("Binary header offset overflow")?;
                ensure!(
                    after_header <= source_end,
                    std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Truncated .dcmm binary header"
                    )
                );
                let mut len_buf = [0_u8; 16];
                if let Err(err) = reader.read_exact(&mut len_buf) {
                    *reader_pos = None;
                    return Err(err)
                        .context("Truncated .dcmm binary data length");
                }
                let mut cur_pos = after_header;
                *reader_pos = Some(cur_pos);
                let data_len = u64::try_from(u128::from_be_bytes(len_buf))
                    .context("Binary data length exceeds u64")?;
                let mut data_type = None;
                let mut checksum = None;
                match encap_type {
                    crate::dc_mixed::TYPE_BINARY => {}
                    crate::dc_mixed::TYPE_BINARY_METADATA => {
                        let after_meta_len = cur_pos
                            .checked_add(16)
                            .context("Metadata header offset overflow")?;
                        ensure!(
                            after_meta_len <= source_end,
                            std::io::Error::new(
                                std::io::ErrorKind::UnexpectedEof,
                                "Truncated .dcmm metadata length"
                            )
                        );
                        let mut meta_len_buf = [0_u8; 16];
                        if let Err(err) = reader.read_exact(&mut meta_len_buf) {
                            *reader_pos = None;
                            return Err(err)
                                .context("Truncated .dcmm metadata length");
                        }
                        cur_pos = after_meta_len;
                        *reader_pos = Some(cur_pos);
                        let meta_len =
                            u64::try_from(u128::from_be_bytes(meta_len_buf))
                                .context("Metadata length exceeds u64")?;
                        let metadata_end = cur_pos
                            .checked_add(meta_len)
                            .context("Metadata end offset overflow")?;
                        let payload_end = metadata_end
                            .checked_add(data_len)
                            .context("Binary payload offset overflow")?;
                        ensure!(
                            payload_end <= source_end,
                            std::io::Error::new(
                                std::io::ErrorKind::UnexpectedEof,
                                "Truncated .dcmm metadata or binary payload",
                            )
                        );
                        ensure!(
                            meta_len <= policy.max_binary_metadata_bytes,
                            "Binary metadata exceeds resource limit ({} bytes)",
                            policy.max_binary_metadata_bytes
                        );
                        let meta_len_usize = usize::try_from(meta_len)?;
                        let mut meta_bytes = Vec::new();
                        meta_bytes.try_reserve_exact(meta_len_usize)?;
                        meta_bytes.resize(meta_len_usize, 0);
                        if let Err(err) = reader.read_exact(&mut meta_bytes) {
                            *reader_pos = None;
                            return Err(err)
                                .context("Truncated .dcmm metadata payload");
                        }
                        cur_pos = metadata_end;
                        *reader_pos = Some(cur_pos);
                        if *is_v1 {
                            let meta_owned =
                                crate::DcString::from_dcutf(meta_bytes)?;
                            let parsed = crate::encapsulation::parse_metadata_with_policy_and_resolver(&meta_owned, policy, resolver)?;
                            data_type = parsed.data_type.map(DcStr::to_owned);
                            checksum = parsed.checksum_spec;
                        } else {
                            let meta_str = DcStr::from_bytes(&meta_bytes)?;
                            let parsed = crate::encapsulation::parse_metadata_with_policy_and_resolver(meta_str, policy, resolver)?;
                            data_type = parsed.data_type.map(DcStr::to_owned);
                            checksum = parsed.checksum_spec;
                        }
                    }
                    crate::dc_mixed::TYPE_BINARY_SHA256 => {
                        let after_digest = cur_pos
                            .checked_add(32)
                            .context("Binary payload offset overflow")?;
                        let payload_end = after_digest
                            .checked_add(data_len)
                            .context("Binary payload offset overflow")?;
                        ensure!(
                            payload_end <= source_end,
                            std::io::Error::new(
                                std::io::ErrorKind::UnexpectedEof,
                                "Truncated .dcmm SHA256 encapsulation",
                            )
                        );
                        let mut digest = vec![0_u8; 32];
                        if let Err(err) = reader.read_exact(&mut digest) {
                            *reader_pos = None;
                            return Err(err)
                                .context("Truncated .dcmm SHA256 digest");
                        }
                        cur_pos = after_digest;
                        *reader_pos = Some(cur_pos);
                        checksum = Some(BinaryChecksum {
                            algorithm: SupportedHashFormat::Sha256,
                            expected_digest: digest,
                        });
                    }
                    _ => {
                        return Err(
                            crate::DcMixedError::InvalidEncapsulationType {
                                offset: rel_offset,
                                type_byte: encap_type,
                            }
                            .into(),
                        );
                    }
                }
                let payload_start = cur_pos;
                let payload_end = payload_start
                    .checked_add(data_len)
                    .context("Binary payload offset overflow")?;
                ensure!(
                    payload_end <= source_end,
                    std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        format!(
                            "Truncated .dcmm binary encapsulation at offset {start_offset} (needs {payload_end}, end is {})",
                            self.active_span.end,
                        )
                    )
                );
                seek_buffered_file(reader, reader_pos, payload_end)?;
                Ok(Some((
                    payload_end,
                    DcFrameHeaderInfo::Binary {
                        data_type,
                        payload_span: Some(DcByteSpan::new(
                            payload_start,
                            payload_end,
                        )),
                        payload_len: data_len,
                        checksum,
                    },
                )))
            }
        }
    }

    pub(super) fn scan_binary_at(
        &mut self,
        start_offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(u64, DcFrameHeaderInfo)> {
        if let Some(res) =
            self.try_scan_raw_binary_at(start_offset, policy, resolver)?
        {
            return Ok(res);
        }
        self.scan_textual_binary_at(start_offset, resolver)
    }

    pub(super) fn scan_textual_binary_at(
        &mut self,
        start_offset: u64,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(u64, DcFrameHeaderInfo)> {
        let Some((first_ch, after_opener)) =
            self.read_token_or_binary_at(start_offset)?
        else {
            bail!(
                "Unclosed textual binary encapsulation (Dc 203) starting at offset {start_offset}"
            );
        };
        if !first_ch.fills_role_with(DcRole::StartEncapsulationBinary, resolver)
        {
            return Ok((after_opener, DcFrameHeaderInfo::None));
        }
        let Some((second_ch, _)) =
            self.read_token_or_binary_at(after_opener)?
        else {
            return Ok((after_opener, DcFrameHeaderInfo::None));
        };
        if !crate::dc_mixed::is_textual_encapsulation_follower(
            second_ch, resolver,
        ) {
            return Ok((after_opener, DcFrameHeaderInfo::None));
        }
        let mut offset = start_offset;
        let mut depth = 0usize;
        let mut escaped = false;
        while let Some((ch, next_offset)) =
            self.read_token_or_binary_at(offset)?
        {
            offset = next_offset;
            if escaped {
                escaped = false;
                continue;
            }
            if ch.fills_role_with(DcRole::Escape, resolver) {
                escaped = true;
                continue;
            }
            if ch.fills_role_with(DcRole::StartEncapsulationBinary, resolver) {
                depth = depth.saturating_add(1);
            } else if ch
                .fills_role_with(DcRole::EndEncapsulationBinary, resolver)
            {
                depth = depth.checked_sub(1).context(
                    "Unexpected binary encapsulation closer (Dc 204)",
                )?;
                if depth == 0 {
                    return Ok((
                        offset,
                        DcFrameHeaderInfo::Binary {
                            data_type: None,
                            payload_span: None,
                            payload_len: 0,
                            checksum: None,
                        },
                    ));
                }
            }
        }
        bail!(
            "Unclosed textual binary encapsulation (Dc 203) starting at offset {start_offset}"
        );
    }

    fn advance_past_literal_type_header(
        &mut self,
        after_opener: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        if let Some((ch, next_off)) =
            self.read_token_or_binary_at(after_opener)?
        {
            if ch.fills_role_with(DcRole::BeginTypeDef, resolver) {
                return self
                    .scan_balanced_type_def(next_off, policy, resolver);
            }
            self.record_diagnostic(
                policy,
                policy.syntax_error,
                after_opener,
                format!("Literal at offset {after_opener} is missing Dc 262..263 type header"),
            )?;
            Ok(after_opener)
        } else {
            self.record_diagnostic(
                policy,
                policy.syntax_error,
                after_opener,
                format!("Unclosed literal at offset {after_opener}: missing Dc 262..263 type header"),
            )?;
            Ok(after_opener)
        }
    }

    fn advance_past_optional_identifier_type_header(
        &mut self,
        after_opener: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        if let Some((ch, next_off)) =
            self.read_token_or_binary_at(after_opener)?
            && ch.fills_role_with(DcRole::BeginTypeDef, resolver)
        {
            return self.scan_balanced_type_def(next_off, policy, resolver);
        }
        Ok(after_opener)
    }

    fn scan_balanced_type_def(
        &mut self,
        start: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        let mut offset = start;
        let mut escaped = false;
        let mut depth = 1usize;
        while let Some((ch, next_offset)) =
            self.read_token_or_binary_at(offset)?
        {
            offset = next_offset;
            if escaped {
                escaped = false;
                continue;
            }
            if ch.fills_role_with(DcRole::Escape, resolver) {
                escaped = true;
                continue;
            }
            if ch.fills_role_with(DcRole::BeginTypeDef, resolver) {
                depth = depth.saturating_add(1);
            } else if ch.fills_role_with(DcRole::EndTypeDef, resolver) {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(offset);
                }
            }
        }
        let closer = ctb_formats_dcdata::dc::DC_END_TYPE_DEF;
        self.record_diagnostic(
            policy,
            policy.syntax_error,
            start,
            format!(
                "Unclosed frame starting at {start}: missing closer {closer:?}"
            ),
        )?;
        Ok(offset)
    }

    pub(super) fn scan_until_token(
        &mut self,
        start: u64,
        closer: DcChar,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        let mut offset = start;
        let mut escaped = false;
        while let Some((ch, next_offset)) =
            self.read_token_or_binary_at(offset)?
        {
            offset = next_offset;
            if escaped {
                escaped = false;
                continue;
            }
            if ch.fills_role_with(DcRole::Escape, resolver) {
                escaped = true;
                continue;
            }
            if ch == closer
                || resolver.is_equivalent_to(ch.to_u128(), closer.to_u128())
            {
                return Ok(offset);
            }
        }
        self.record_diagnostic(
            policy,
            policy.syntax_error,
            start,
            format!(
                "Unclosed frame starting at {start}: missing closer {closer:?}"
            ),
        )?;
        Ok(offset)
    }

    pub(super) fn scan_say_frame_at(
        &mut self,
        start_offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        let Some((say_dc, after_say)) =
            self.read_token_or_binary_at(start_offset)?
        else {
            bail!("Unexpected end of stream at say invocation");
        };
        ensure!(
            say_dc.fills_role_with(DcRole::LangSay, resolver),
            "Expected LangSay (Dc 256), found {say_dc}"
        );
        let Some((param_open, after_open)) =
            self.read_token_or_binary_at(after_say)?
        else {
            bail!(
                "Unexpected end of stream after Dc 256: expected parameter opener"
            );
        };
        ensure!(
            param_open.fills_role_with(DcRole::BeginParameter, resolver),
            "Expected parameter opener (Dc 258) after Dc 256, found {param_open}"
        );
        let Some((lit_open, after_lit_open)) =
            self.read_token_or_binary_at(after_open)?
        else {
            bail!(
                "Unexpected end of stream: expected literal opener after parameter opener"
            );
        };
        ensure!(
            lit_open.fills_role_with(DcRole::BeginLiteral, resolver),
            "Expected literal opener (Dc 260), found {lit_open}"
        );
        let Some((hdr_open, after_262)) =
            self.read_token_or_binary_at(after_lit_open)?
        else {
            bail!("Unexpected end of stream: expected Dc 262 after Dc 260");
        };
        ensure!(
            hdr_open.fills_role_with(DcRole::BeginTypeDef, resolver),
            "Expected Dc 262 after Dc 260 in lang.say"
        );
        let Some((type_dc, after_type)) =
            self.read_token_or_binary_at(after_262)?
        else {
            bail!("Unexpected end of stream: expected type after Dc 262");
        };
        ensure!(
            type_dc.fills_role_with(DcRole::TypeString, resolver)
                || type_dc.fills_role_with(DcRole::TypeAny, resolver),
            "Expected String type (Dc 264) in lang.say literal, found {type_dc}"
        );
        let Some((hdr_close, _)) = self.read_token_or_binary_at(after_type)?
        else {
            bail!("Unexpected end of stream: expected Dc 263 after type");
        };
        ensure!(
            hdr_close.fills_role_with(DcRole::EndTypeDef, resolver),
            "Expected Dc 263 after type in lang.say"
        );
        let (_, lit_end, _) = self.scan_balanced_container(
            after_lit_open,
            lit_open,
            ctb_formats_dcdata::dc::DC_END_LITERAL,
            policy,
            resolver,
        )?;
        let Some((param_close, after_param_close)) =
            self.read_token_or_binary_at(lit_end)?
        else {
            bail!(
                "Unexpected end of stream: expected parameter closer after literal"
            );
        };
        ensure!(
            param_close.fills_role_with(DcRole::EndParameter, resolver),
            "Expected parameter closer (Dc 259), found {param_close}"
        );

        Ok(after_param_close)
    }

    pub(super) fn scan_splice_invocation_at(
        &mut self,
        start_offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<u64>> {
        let Some((first_dc, after_first)) =
            self.read_token_or_binary_at(start_offset)?
        else {
            return Ok(None);
        };
        if first_dc.fills_role_with(DcRole::LangSay, resolver)
            && let Ok(end) =
                self.scan_say_frame_at(start_offset, policy, resolver)
        {
            return Ok(Some(end));
        }
        if resolver
            .resolve_wasm_definition_tokens(first_dc.to_u128())
            .is_none()
        {
            return Ok(None);
        }
        if let Some(target) = CharTarget::from_u128(first_dc.to_u128()) {
            if let Some(closer_u128) = resolver.closer_for_opener(&target) {
                if let Ok((closer_start, after_closer, is_spliced)) = self
                    .scan_balanced_container(
                        after_first,
                        first_dc,
                        DcChar::from_u128(closer_u128),
                        policy,
                        resolver,
                    )
                    && (closer_start < after_closer || is_spliced)
                {
                    return Ok(Some(after_closer));
                }
                return Ok(None);
            }
            if resolver.resolve_rule_tokens(&target).is_some()
                || resolver.resolve_rule(&target).is_some()
            {
                let saved = self.save_checkpoint();
                let saved_diag_len = self.diagnostics.len();
                self.cursor = start_offset;
                self.clear_spliced_tokens();
                let mut ctx = super::eval::SyntaxEvalContext::new(
                    resolver,
                    Some(first_dc.to_u128()),
                );
                ctx.policy.clone_from(policy);
                let matched = if let Some(raw_tokens) =
                    resolver.resolve_rule_tokens(&target)
                {
                    self.eval_pattern_tokens(raw_tokens, &mut ctx)?
                } else if let Some(rule) = resolver.resolve_rule(&target) {
                    self.eval_pattern(&rule.pattern, &mut ctx)?
                } else {
                    false
                };
                let end = if matched
                    && !ctx.has_errors
                    && resolver
                        .matches_script(".EL Routines", first_dc.to_u128())
                {
                    // Reason for fallback: if optional trailing parameter scan fails, fall back to the matched rule end.
                    self.scan_trailing_parameter_frames_with_resolver(
                        self.cursor,
                        policy,
                        resolver,
                    )
                    .unwrap_or(self.cursor)
                } else {
                    self.cursor
                };
                self.restore_checkpoint(saved);
                self.diagnostics.truncate(saved_diag_len);
                if matched && !ctx.has_errors && end >= after_first {
                    return Ok(Some(end));
                }
            }
        }
        if let Ok(end) = self.scan_trailing_parameter_frames_with_resolver(
            after_first,
            policy,
            resolver,
        ) && end > after_first
        {
            return Ok(Some(end));
        }
        Ok(Some(after_first))
    }

    pub(super) fn scan_balanced_container(
        &mut self,
        after_opener: u64,
        opener: DcChar,
        closer: DcChar,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(u64, u64, bool)> {
        struct CloserFrame {
            after_opener: u64,
            closer: DcChar,
            opaque: bool,
            max_rel_depth: usize,
            had_issue: bool,
        }

        self.scan_cache.ensure_resolver(resolver);
        let root_depth_exceeded = self.depth() >= policy.max_depth;
        if root_depth_exceeded {
            self.record_diagnostic(
                policy,
                policy.depth_limit_exceeded,
                after_opener,
                format!(
                    "Container depth limit reached ({}) at offset {after_opener}",
                    policy.max_depth,
                ),
            )?;
        } else if !self.has_spliced_tokens()
            && let Some(&cached) =
                self.scan_cache.balanced_containers.get(&after_opener)
            && cached.after_closer <= self.active_span.end
            && self.depth().saturating_add(cached.max_rel_depth)
                < policy.max_depth
        {
            return Ok((
                cached.closer_start,
                cached.after_closer,
                cached.is_spliced,
            ));
        }

        let diag_base = self.diagnostics.len();
        let mut closer_stack = vec![CloserFrame {
            after_opener,
            closer,
            opaque: resolver.is_opaque_container(opener.to_u128()),
            max_rel_depth: 0,
            had_issue: root_depth_exceeded,
        }];
        let mut offset = after_opener;
        if opener.fills_role_with(DcRole::BeginLiteral, resolver) {
            offset = self
                .advance_past_literal_type_header(offset, policy, resolver)?;
            if self.diagnostics.len() > diag_base
                && let Some(top) = closer_stack.last_mut()
            {
                top.had_issue = true;
            }
        } else if opener.fills_role_with(DcRole::BeginIdentifier, resolver) {
            offset = self.advance_past_optional_identifier_type_header(
                offset, policy, resolver,
            )?;
            if self.diagnostics.len() > diag_base
                && let Some(top) = closer_stack.last_mut()
            {
                top.had_issue = true;
            }
        }

        while let Some((ch, next_offset)) =
            self.read_token_or_binary_at(offset)?
        {
            let token_start = offset;
            if ch.fills_role_with(DcRole::Escape, resolver) {
                if let Some((_, after_escaped)) =
                    self.read_token_or_binary_at(next_offset)?
                {
                    offset = after_escaped;
                    continue;
                }
                self.record_diagnostic(
                    policy,
                    policy.syntax_error,
                    token_start,
                    format!("Dangling escape Dc 255 at offset {token_start}"),
                )?;
                return Ok((next_offset, next_offset, false));
            }
            if closer_stack.last().is_some_and(|f| f.opaque) {
                if closer_stack.last().is_some_and(|f| {
                    f.closer == ch
                        || resolver
                            .is_equivalent_to(ch.to_u128(), f.closer.to_u128())
                }) {
                    if let Some(popped) = closer_stack.pop() {
                        if !popped.had_issue && !self.has_spliced_tokens() {
                            if self.scan_cache.balanced_containers.len()
                                >= MAX_NAV_CACHE_ENTRIES
                            {
                                self.scan_cache.balanced_containers.clear();
                            }
                            self.scan_cache.balanced_containers.insert(
                                popped.after_opener,
                                CachedBalancedContainer {
                                    closer_start: token_start,
                                    after_closer: next_offset,
                                    is_spliced: false,
                                    max_rel_depth: popped.max_rel_depth,
                                },
                            );
                        }
                        if let Some(parent) = closer_stack.last_mut() {
                            parent.max_rel_depth = parent
                                .max_rel_depth
                                .max(popped.max_rel_depth.saturating_add(1));
                            parent.had_issue |= popped.had_issue;
                        } else {
                            return Ok((token_start, next_offset, false));
                        }
                    }
                }
                offset = next_offset;
                continue;
            }
            if ch.fills_role_with(DcRole::LangSay, resolver)
                || resolver.is_splice_trigger_token(ch.to_u128())
            {
                let diag_before_say = self.diagnostics.len();
                if let Ok(Some(after_say)) =
                    self.scan_splice_invocation_at(token_start, policy, resolver)
                {
                    if self.diagnostics.len() > diag_before_say
                        && let Some(top) = closer_stack.last_mut()
                    {
                        top.had_issue = true;
                    }
                    let call_tokens = self.read_tokens_in_span(
                        DcByteSpan::new(token_start, after_say),
                    )?;
                    let call_dcs: Vec<DcChar> = call_tokens
                        .into_iter()
                        .map(DcChar::from_u128)
                        .collect();
                    let evaluator = DefaultDcExpressionEvaluator;
                    if let Ok(Some(spliced)) =
                        evaluator.eval_stream_splice(&call_dcs, resolver)
                    {
                        for frame in &mut closer_stack {
                            frame.had_issue = true;
                        }
                        let mut spliced_iter = spliced.into_iter();
                        while let Some(tok) = spliced_iter.next() {
                            if tok.fills_role_with(DcRole::Escape, resolver) {
                                let _ = spliced_iter.next();
                                continue;
                            }
                            if closer_stack.last().is_some_and(|f| f.opaque) {
                                if closer_stack.last().is_some_and(|f| {
                                    f.closer == tok
                                        || resolver.is_equivalent_to(
                                            tok.to_u128(),
                                            f.closer.to_u128(),
                                        )
                                }) {
                                    closer_stack.pop();
                                    if closer_stack.is_empty() {
                                        return Ok((
                                            after_say, after_say, true,
                                        ));
                                    }
                                }
                                continue;
                            }
                            if closer_stack.last().is_some_and(|f| {
                                f.closer == tok
                                    || resolver.is_equivalent_to(
                                        tok.to_u128(),
                                        f.closer.to_u128(),
                                    )
                            }) {
                                closer_stack.pop();
                                if closer_stack.is_empty() {
                                    return Ok((after_say, after_say, true));
                                }
                            } else if let Some(target) =
                                CharTarget::from_u128(tok.to_u128())
                            {
                                if let Some(inner_closer) =
                                    resolver.closer_for_opener(&target)
                                {
                                    closer_stack.push(CloserFrame {
                                        after_opener: after_say,
                                        closer: DcChar::from_u128(inner_closer),
                                        opaque: resolver
                                            .is_opaque_container(tok.to_u128()),
                                        max_rel_depth: 0,
                                        had_issue: true,
                                    });
                                }
                            }
                        }
                    }
                    offset = after_say;
                    continue;
                }
            }
            if ch.fills_role_with(DcRole::StartEncapsulationBinary, resolver) {
                let diag_before_bin = self.diagnostics.len();
                let (after_bin, _) =
                    self.scan_binary_at(token_start, policy, resolver)?;
                if self.diagnostics.len() > diag_before_bin
                    && let Some(top) = closer_stack.last_mut()
                {
                    top.had_issue = true;
                }
                offset = after_bin;
                continue;
            }
            if closer_stack.last().is_some_and(|f| {
                f.closer == ch
                    || resolver
                        .is_equivalent_to(ch.to_u128(), f.closer.to_u128())
            }) {
                if let Some(popped) = closer_stack.pop() {
                    if !popped.had_issue && !self.has_spliced_tokens() {
                        if self.scan_cache.balanced_containers.len()
                            >= MAX_NAV_CACHE_ENTRIES
                        {
                            self.scan_cache.balanced_containers.clear();
                        }
                        self.scan_cache.balanced_containers.insert(
                            popped.after_opener,
                            CachedBalancedContainer {
                                closer_start: token_start,
                                after_closer: next_offset,
                                is_spliced: false,
                                max_rel_depth: popped.max_rel_depth,
                            },
                        );
                    }
                    if let Some(parent) = closer_stack.last_mut() {
                        parent.max_rel_depth = parent
                            .max_rel_depth
                            .max(popped.max_rel_depth.saturating_add(1));
                        parent.had_issue |= popped.had_issue;
                    } else {
                        return Ok((token_start, next_offset, false));
                    }
                }
                offset = next_offset;
                continue;
            }
            // Sacred opener/closer framing: if an inner container was left unclosed
            // before an outer closer on the stack, synchronize at the outer closer.
            if let Some(stack_pos) = closer_stack.iter().rposition(|f| {
                f.closer == ch
                    || resolver
                        .is_equivalent_to(ch.to_u128(), f.closer.to_u128())
            }) {
                self.record_diagnostic(
                    policy,
                    policy.syntax_error,
                    token_start,
                    format!(
                        "Unclosed inner container before closer {ch:?} at offset {token_start}"
                    ),
                )?;
                closer_stack.truncate(stack_pos);
                for frame in &mut closer_stack {
                    frame.had_issue = true;
                }
                if closer_stack.is_empty() {
                    return Ok((token_start, next_offset, false));
                }
                offset = next_offset;
                continue;
            }
            if let Some(target) = CharTarget::from_u128(ch.to_u128()) {
                if let Some(inner_closer) = resolver.closer_for_opener(&target)
                {
                    let next_depth =
                        self.depth().saturating_add(closer_stack.len());
                    if !self.has_spliced_tokens()
                        && let Some(&cached) = self
                            .scan_cache
                            .balanced_containers
                            .get(&next_offset)
                        && cached.after_closer <= self.active_span.end
                        && next_depth.saturating_add(cached.max_rel_depth)
                            < policy.max_depth
                    {
                        if let Some(parent) = closer_stack.last_mut() {
                            parent.max_rel_depth = parent
                                .max_rel_depth
                                .max(cached.max_rel_depth.saturating_add(1));
                        }
                        offset = cached.after_closer;
                        continue;
                    }
                    let inner_depth_exceeded = next_depth >= policy.max_depth;
                    if inner_depth_exceeded {
                        self.record_diagnostic(
                            policy,
                            policy.depth_limit_exceeded,
                            token_start,
                            format!(
                                "Container depth limit reached ({}) at offset {token_start}",
                                policy.max_depth,
                            ),
                        )?;
                    }
                    closer_stack.try_reserve(1)?;
                    closer_stack.push(CloserFrame {
                        after_opener: next_offset,
                        closer: DcChar::from_u128(inner_closer),
                        opaque: resolver.is_opaque_container(ch.to_u128()),
                        max_rel_depth: 0,
                        had_issue: inner_depth_exceeded,
                    });
                    if ch.fills_role_with(DcRole::BeginLiteral, resolver) {
                        let diag_before_hdr = self.diagnostics.len();
                        offset = self.advance_past_literal_type_header(
                            next_offset,
                            policy,
                            resolver,
                        )?;
                        if self.diagnostics.len() > diag_before_hdr
                            && let Some(top) = closer_stack.last_mut()
                        {
                            top.had_issue = true;
                        }
                        continue;
                    } else if ch
                        .fills_role_with(DcRole::BeginIdentifier, resolver)
                    {
                        let diag_before_hdr = self.diagnostics.len();
                        offset = self
                            .advance_past_optional_identifier_type_header(
                                next_offset,
                                policy,
                                resolver,
                            )?;
                        if self.diagnostics.len() > diag_before_hdr
                            && let Some(top) = closer_stack.last_mut()
                        {
                            top.had_issue = true;
                        }
                        continue;
                    }
                }
            }
            offset = next_offset;
        }

        self.record_diagnostic(
            policy,
            policy.syntax_error,
            after_opener,
            format!("Unclosed container {opener:?}..{closer:?} starting at offset {after_opener}"),
        )?;
        Ok((offset, offset, false))
    }

    pub(super) fn verify_checksummed_section(
        &mut self,
        body_span: DcByteSpan,
        after_closer: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(u64, Option<u64>, bool)> {
        // Parse optional leading 546 SequenceIdentifier inside body_span
        let mut sequence_id = None;
        if let Ok(Some((first_in_body, after_tag))) =
            self.read_token_or_binary_at(body_span.start)
        {
            if first_in_body
                .fills_role_with(DcRole::SequenceIdentifier, resolver)
            {
                if let Ok(Some((num_open, after_num_open))) =
                    self.read_token_or_binary_at(after_tag)
                {
                    if num_open.fills_role_with(DcRole::BeginNumber, resolver) {
                        if let Ok(num_end) = self.scan_until_token(
                            after_num_open,
                            ctb_formats_dcdata::dc::DC_END_NUMBER,
                            policy,
                            resolver,
                        ) {
                            if let Ok((_, digits)) = self
                                .decode_number_span_with_resolver(
                                    DcByteSpan::new(after_tag, num_end),
                                    resolver,
                                )
                            {
                                if let Ok(int_val) = crate::dc_number::parse_dc_number_digits_with_resolver(&digits, resolver) {
                                    sequence_id = u64::try_from(&int_val).ok();
                                }
                            }
                        }
                    }
                }
            }
        }

        // Expect trailing 545 SectionDigest + [checksum] (203..204)
        let Some((digest_tag, after_digest_tag)) =
            self.read_token_or_binary_at(after_closer)?
        else {
            self.record_diagnostic(
                policy,
                policy.syntax_error,
                after_closer,
                "Missing Dc 545 (SectionDigest) after Dc 544 (EndChecksummedSection)".to_owned(),
            )?;
            return Ok((after_closer, sequence_id, false));
        };
        if !digest_tag.fills_role_with(DcRole::SectionDigest, resolver) {
            self.record_diagnostic(
                policy,
                policy.syntax_error,
                after_closer,
                format!("Expected Dc 545 (SectionDigest) after Dc 544, found {digest_tag:?}"),
            )?;
            return Ok((after_closer, sequence_id, false));
        }
        self.cursor = after_digest_tag;
        let mut ctx = SyntaxEvalContext::new(resolver, None);
        ctx.policy.clone_from(policy);
        let checksum_term = SyntaxTerm::NamedConstruct {
            name: "checksum".to_owned(),
            subtype: None,
            capture_var: None,
        };
        let checksum_ok = self.eval_term_single(&checksum_term, &mut ctx)?;
        if !checksum_ok {
            self.record_diagnostic(
                policy,
                policy.syntax_error,
                after_digest_tag,
                format!("Invalid [checksum] after Dc 545 at offset {after_digest_tag}"),
            )?;
        }
        let (final_end, bin_header) =
            self.scan_binary_at(after_digest_tag, policy, resolver)?;
        if !checksum_ok {
            return Ok((final_end, sequence_id, false));
        }
        let checksum = if let DcFrameHeaderInfo::Binary {
            data_type: Some(data_type),
            payload_span: Some(pspan),
            ..
        } = bin_header
        {
            let Some(algorithm) =
                BinaryChecksum::algorithm_for_digest_with_resolver(
                    &data_type,
                    pspan.len(),
                    policy,
                    resolver,
                )?
            else {
                self.syntax_issue(policy, body_span.start, "Section checksum algorithm is unsupported; integrity was not verified".to_owned())?;
                return Ok((final_end, sequence_id, false));
            };
            BinaryChecksum {
                algorithm,
                expected_digest: self.read_span_bytes(pspan)?.into_owned(),
            }
        } else {
            let DcSyntaxKind::Binary {
                data_type: Some(data_type),
                payload,
                ..
            } = self.parse_binary_arena_node(
                DcByteSpan::new(after_digest_tag, final_end),
                policy,
                resolver,
            )?
            else {
                bail!("Section digest requires a typed checksum payload");
            };
            let Some(algorithm) =
                BinaryChecksum::algorithm_for_digest_with_resolver(
                    &data_type,
                    payload.len(),
                    policy,
                    resolver,
                )?
            else {
                self.syntax_issue(policy, body_span.start, "Section checksum algorithm is unsupported; integrity was not verified".to_owned())?;
                return Ok((final_end, sequence_id, false));
            };
            BinaryChecksum {
                algorithm,
                expected_digest: payload.materialize_bytes()?.into_owned(),
            }
        };
        let matches = self.verify_checksum_over_span(body_span, &checksum)?;
        let valid = checksum_ok && matches;
        if !valid {
            self.syntax_issue(
                policy,
                body_span.start,
                "Section checksum mismatch".to_owned(),
            )?;
        }
        Ok((final_end, sequence_id, valid))
    }

    fn verify_checksum_over_span(
        &mut self,
        span: DcByteSpan,
        checksum: &BinaryChecksum,
    ) -> Result<bool> {
        if let Some(bytes) = self.backend.memory_bytes() {
            let mut body = bytes
                .get(span.to_usize_range()?)
                .context("Checksummed span out of bounds")?;
            return checksum.matches_reader(&mut body, span.len());
        }
        match &mut self.backend {
            NavBackend::SeekableFile {
                reader, reader_pos, ..
            } => {
                let saved_pos = match *reader_pos {
                    Some(pos) => pos,
                    None => reader.stream_position()?,
                };
                let result = (|| -> Result<bool> {
                    seek_buffered_file(reader, reader_pos, span.start)?;
                    let ok = checksum.matches_reader(reader, span.len())?;
                    *reader_pos = Some(span.end);
                    Ok(ok)
                })();
                if result.is_err() {
                    *reader_pos = None;
                }
                seek_buffered_file(reader, reader_pos, saved_pos)
                    .context("Failed to restore checksum reader position")?;
                result
            }
            NavBackend::MemoryMstr(_) | NavBackend::MemoryText(_) => {
                unreachable!()
            }
        }
    }

    fn parse_textual_binary_arena_node(
        sub: &DcStr,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<DcSyntaxKind<'a>> {
        let (header, data, _) =
            crate::encapsulation::parse_text_with_policy_and_resolver(
                sub, policy, resolver,
            )?;
        let cf = header.chars().next();
        Ok(DcSyntaxKind::Binary {
            data_type: (!header.is_empty()).then_some(header),
            checksum_format: cf,
            payload: BinaryPayloadRef::Inline(data),
        })
    }

    pub(super) fn parse_binary_arena_node(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<DcSyntaxKind<'a>> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let range = span.to_usize_range()?;
                let slice = mstr
                    .as_bytes()
                    .get(range)
                    .context("Binary node span out of bounds")?;
                let mut chunks = crate::dc_mixed::DcMixedChunks { slice };
                let Some(chunk) = chunks.next() else {
                    return Self::parse_textual_binary_arena_node(
                        DcStr::from_bytes(slice)?,
                        policy,
                        resolver,
                    );
                };
                match chunk {
                    crate::DcMixedChunk::Binary { data, .. } => {
                        Ok(DcSyntaxKind::Binary {
                            data_type: None,
                            checksum_format: None,
                            payload: BinaryPayloadRef::Borrowed(data),
                        })
                    }
                    crate::DcMixedChunk::BinaryWithMetadata {
                        data,
                        metadata,
                    } => {
                        let parsed = crate::encapsulation::parse_metadata_with_policy_and_resolver(metadata, policy, resolver)?;
                        let dt = parsed.data_type.map(DcStr::to_owned);
                        let cf =
                            parsed.data_type.and_then(|d| d.chars().next());
                        Ok(DcSyntaxKind::Binary {
                            data_type: dt,
                            checksum_format: cf,
                            payload: BinaryPayloadRef::Borrowed(data),
                        })
                    }
                    crate::DcMixedChunk::Text(txt) => {
                        Self::parse_textual_binary_arena_node(
                            txt, policy, resolver,
                        )
                    }
                }
            }
            NavBackend::MemoryText(text) => {
                let range = span.to_usize_range()?;
                let sub = text
                    .get(range)
                    .context("Binary node span out of bounds in DcStr")?;
                Self::parse_textual_binary_arena_node(sub, policy, resolver)
            }
            NavBackend::SeekableFile { .. } => {
                let (_, info) =
                    self.scan_binary_at(span.start, policy, resolver)?;
                let DcFrameHeaderInfo::Binary {
                    data_type,
                    payload_span: Some(pspan),
                    payload_len,
                    checksum,
                } = info
                else {
                    bail!("Expected binary header info at span {span:?}");
                };
                let checksum_format =
                    data_type.as_ref().and_then(|d| d.chars().next());
                if let Some(checksum) = &checksum
                    && !self.verify_checksum_over_span(pspan, checksum)?
                {
                    self.syntax_issue(
                        policy,
                        span.start,
                        "Encapsulation checksum mismatch".to_owned(),
                    )?;
                }
                let payload = if payload_len <= INLINE_BINARY_THRESHOLD {
                    let bytes = self.read_span_bytes(pspan)?.into_owned();
                    BinaryPayloadRef::Inline(bytes)
                } else {
                    BinaryPayloadRef::FileSpan {
                        file_offset: pspan.start,
                        len: payload_len,
                        checksum,
                        checksum_action: policy.syntax_error,
                    }
                };
                Ok(DcSyntaxKind::Binary {
                    data_type,
                    checksum_format,
                    payload,
                })
            }
        }
    }

    pub(super) fn decode_literal_from_cursor(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(Vec<DcChar>, DcMstring)> {
        let Some(first) = self.pop_next_token()? else {
            bail!("Empty literal span");
        };
        ensure!(
            first.fills_role_with(DcRole::BeginLiteral, resolver),
            "Expected literal opener (Dc 260), found {first}"
        );
        let mut type_header = Vec::new();
        if self
            .peek_next_token()?
            .is_some_and(|ch| ch.fills_role_with(DcRole::BeginTypeDef, resolver))
        {
            let _ = self.pop_next_token()?;
            let mut depth = 1usize;
            while let Some(tch) = self.pop_next_token()? {
                if tch.fills_role_with(DcRole::BeginTypeDef, resolver) {
                    depth = depth.saturating_add(1);
                } else if tch.fills_role_with(DcRole::EndTypeDef, resolver) {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                type_header.push(tch);
            }
        }
        let mut decoded = DcMstring::new();
        while let Some(mut ch) = self.pop_next_token()? {
            if ch.fills_role_with(DcRole::EndLiteral, resolver) {
                break;
            }
            if ch.fills_role_with(DcRole::Escape, resolver) {
                if let Some(escaped) = self.pop_next_token()? {
                    ch = escaped;
                } else {
                    self.syntax_issue(
                        policy,
                        self.cursor,
                        format!(
                            "Dangling escape Dc 255 in literal at offset {}",
                            self.cursor
                        ),
                    )?;
                    break;
                }
            }
            decoded.push_char(ch);
        }
        Ok((type_header, decoded))
    }

    fn with_span_cursor<R>(
        &mut self,
        span: DcByteSpan,
        f: impl FnOnce(&mut Self) -> Result<R>,
    ) -> Result<R> {
        let saved = self.save_checkpoint();
        let saved_span = self.active_span;
        self.cursor = span.start;
        self.active_span = span;
        self.spliced_head = self.spliced_end;
        let res = f(self);
        self.active_span = saved_span;
        self.restore_checkpoint(saved);
        res
    }

    fn with_stub_cursor<R>(
        &mut self,
        stub: &crate::navigator::DcFrameStub,
        f: impl FnOnce(&mut Self) -> Result<R>,
    ) -> Result<R> {
        if let DcFrameHeaderInfo::SplicedTokens(tokens) = &stub.header_info {
            let saved = self.save_checkpoint();
            let saved_span = self.active_span;
            let start_idx = self.spliced_buf.len();
            self.spliced_buf.extend_from_slice(tokens);
            self.spliced_head = start_idx;
            self.spliced_end = self.spliced_buf.len();
            let res = f(self);
            self.spliced_buf.truncate(start_idx);
            self.active_span = saved_span;
            self.restore_checkpoint(saved);
            res
        } else {
            self.with_span_cursor(stub.outer_span, f)
        }
    }

    pub(super) fn decode_literal_stub_with_resolver(
        &mut self,
        stub: &crate::navigator::DcFrameStub,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(Vec<DcChar>, DcMstring)> {
        self.with_stub_cursor(stub, |nav| {
            nav.decode_literal_from_cursor(policy, resolver)
        })
    }

    pub(super) fn decode_identifier_stub_with_resolver(
        &mut self,
        stub: &crate::navigator::DcFrameStub,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<DcMstring> {
        self.with_stub_cursor(stub, |nav| {
            nav.decode_identifier_from_cursor(policy, resolver)
        })
    }

    pub(super) fn decode_identifier_from_cursor(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<DcMstring> {
        let Some(first) = self.pop_next_token()? else {
            bail!("Empty identifier span");
        };
        ensure!(
            first.fills_role_with(DcRole::BeginIdentifier, resolver),
            "Expected identifier opener (Dc 270), found {first}"
        );
        // Reason for fallback: opener defaults to CharTarget::Dc(270) and closer defaults to DC_END_IDENTIFIER.
        let closer = CharTarget::from_u128(first.to_u128())
            .and_then(|t| resolver.closer_for_opener(&t))
            .map_or(
                ctb_formats_dcdata::dc::DC_END_IDENTIFIER,
                DcChar::from_u128,
            );
        let mut type_ctx = SyntaxEvalContext::new(resolver, None);
        type_ctx.policy.clone_from(policy);
        let before_type = self.save_checkpoint();
        // Reason for fallback: speculative leading-type evaluation failure restores checkpoint.
        if self
            .try_consume_optional_leading_type(closer, &mut type_ctx)
            .unwrap_or(None)
            .is_none()
            || type_ctx.has_errors
            || self
                .peek_next_token()?
                .is_none_or(|ch| ch.fills_role_with(DcRole::EndIdentifier, resolver))
        {
            self.restore_checkpoint(before_type);
        }
        let mut name = DcMstring::new();
        while let Some(mut ch) = self.pop_next_token()? {
            if ch.fills_role_with(DcRole::EndIdentifier, resolver) {
                break;
            }
            if ch.fills_role_with(DcRole::Escape, resolver) {
                if let Some(escaped) = self.pop_next_token()? {
                    ch = escaped;
                } else {
                    self.syntax_issue(
                        policy,
                        self.cursor,
                        format!(
                            "Dangling escape Dc 255 in identifier at offset {}",
                            self.cursor
                        ),
                    )?;
                    break;
                }
            }
            name.push_char(ch);
        }
        Ok(name)
    }

    pub(super) fn decode_identifier_span_with_resolver(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<DcMstring> {
        self.with_span_cursor(span, |nav| {
            nav.decode_identifier_from_cursor(policy, resolver)
        })
    }

    pub(super) fn decode_number_from_cursor(
        &mut self,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(DcChar, Vec<DcChar>)> {
        let Some(first) = self.pop_next_token()? else {
            bail!("Empty number span");
        };
        ensure!(
            first.fills_role_with(DcRole::BeginNumber, resolver),
            "Expected number opener (Dc 6), found {first}"
        );
        let Some(format) = self.pop_next_token()? else {
            bail!("Truncated number format");
        };
        if format.fills_role_with(DcRole::EndNumber, resolver) {
            return Ok((DC_INTEGER, Vec::new()));
        }
        let mut digits = Vec::new();
        while let Some(ch) = self.pop_next_token()? {
            if ch.fills_role_with(DcRole::EndNumber, resolver) {
                break;
            }
            digits.push(ch);
        }
        Ok((format, digits))
    }

    pub(super) fn decode_number_stub_with_resolver(
        &mut self,
        stub: &crate::navigator::DcFrameStub,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(DcChar, Vec<DcChar>)> {
        self.with_stub_cursor(stub, |nav| {
            nav.decode_number_from_cursor(resolver)
        })
    }

    pub(super) fn decode_number_span_with_resolver(
        &mut self,
        span: DcByteSpan,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(DcChar, Vec<DcChar>)> {
        self.with_span_cursor(span, |nav| {
            nav.decode_number_from_cursor(resolver)
        })
    }

    pub(super) fn read_tokens_in_span(
        &mut self,
        span: DcByteSpan,
    ) -> Result<Vec<u128>> {
        let saved_span = self.active_span;
        self.active_span = span;
        let mut offset = span.start;
        let mut tokens = Vec::new();
        while offset < span.end {
            let Some((ch, next_off)) = self.read_token_or_binary_at(offset)?
            else {
                break;
            };
            tokens.push(ch.to_u128());
            offset = next_off;
        }
        self.active_span = saved_span;
        Ok(tokens)
    }

    /// Peeks the next token from the spliced slice cursor (if non-empty) or from the
    /// byte-backed stream at `self.cursor`.
    pub(super) fn peek_next_token(&mut self) -> Result<Option<DcChar>> {
        if let Some(ch) = self.peek_spliced_token() {
            return Ok(Some(ch));
        }
        Ok(self.read_token_or_binary_at(self.cursor)?.map(|(ch, _)| ch))
    }

    /// Consumes and returns the next token from the spliced slice cursor (if non-empty)
    /// or from the byte-backed stream at `self.cursor`, advancing `self.cursor`.
    pub(super) fn pop_next_token(&mut self) -> Result<Option<DcChar>> {
        if let Some(ch) = self.pop_spliced_token() {
            return Ok(Some(ch));
        }
        let Some((ch, next_off)) = self.read_token_or_binary_at(self.cursor)?
        else {
            return Ok(None);
        };
        self.cursor = next_off;
        Ok(Some(ch))
    }

    /// Returns all tokens consumed between `checkpoint` and the navigator's current state.
    pub(super) fn consumed_tokens_since(
        &mut self,
        checkpoint: &NavigatorCheckpoint,
    ) -> Result<Vec<DcChar>> {
        let mut out = Vec::new();
        let spliced_consumed = checkpoint
            .spliced_len()
            .saturating_sub(self.spliced_len());
        if let Some(slice) = self
            .spliced_buf
            .get(checkpoint.spliced_head..checkpoint.spliced_end)
        {
            out.extend(slice.iter().take(spliced_consumed).copied());
        }
        if self.cursor > checkpoint.cursor {
            let raw = self.read_tokens_in_span(DcByteSpan::new(
                checkpoint.cursor,
                self.cursor,
            ))?;
            out.extend(raw.into_iter().map(DcChar::from_u128));
        }
        Ok(out)
    }
}

#[expect(
    clippy::expect_used,
    reason = "encode_dctf_buf returns 1..=23, which is provably within u64::MAX"
)]
pub(super) fn dc_char_dctf_len(ch: DcChar) -> u64 {
    let mut buf = [0_u8; 32];
    let len = encode_dctf_buf(&mut buf, ch.to_u128());
    u64::try_from(len).expect("DcTF encoded length fits in u64")
}

impl DcDocumentNavigator<'_> {
    /// Returns true if the navigator is backed by a `DcMixed1` (`DcUtf`) file.
    #[must_use]
    pub const fn is_v1_encoding(&self) -> bool {
        matches!(self.backend, NavBackend::SeekableFile { is_v1: true, .. })
    }

    /// Returns the encoded byte length of `ch` in the underlying stream's Dc encoding
    /// (`DcUtf` for `DcMixed1`, `DcTF` for `DcMixed2` and in-memory buffers).
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "encoded DcChar length is 1..=24, which is provably within u64::MAX"
    )]
    pub fn char_encoded_len(&self, ch: DcChar) -> u64 {
        let mut buf = [0_u8; 32];
        let len = if self.is_v1_encoding() {
            ctb_formats_dctf::encode_utf_8e_128_buf(&mut buf, ch.to_u128())
        } else {
            encode_dctf_buf(&mut buf, ch.to_u128())
        };
        u64::try_from(len).expect("encoded DcChar length fits in u64")
    }
}
