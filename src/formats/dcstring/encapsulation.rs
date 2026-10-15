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

//! Shared textual binary encapsulation and mixed-mode metadata codecs.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::{DcChar, DcChars, DcStr, DcString};
use ctb_formats_dcdata::dc::{
    DC_END_ENCAPSULATION_BINARY, DC_START_ENCAPSULATION_BINARY,
};
use ctb_formats_dcdata::syntax::{
    MatchContext, MatchMode, Quantifier, SyntaxElement, SyntaxPattern,
    SyntaxTerm, default_syntax_resolver, match_pattern,
    Quantifier, SyntaxElement, SyntaxPattern, SyntaxTerm,
};

pub(crate) struct Metadata<'a> {
    pub data_type: Option<&'a DcStr>,
    pub checksum: Option<&'a DcStr>,
}

struct Cursor<'a> {
    source: &'a DcStr,
    chars: DcChars<'a>,
}

impl<'a> Cursor<'a> {
    fn new(source: &'a DcStr) -> Self {
        Self {
            source,
            chars: source.chars(),
        }
    }

    fn position(&self) -> usize {
        self.source.len().saturating_sub(self.chars.slice.len())
    }

    fn peek(&self) -> Option<DcChar> {
        self.chars.clone().next()
    }

    fn next(&mut self) -> Result<DcChar> {
        self.chars.next().context("Truncated binary encapsulation")
    }

    fn range(&self, start: usize) -> Result<&'a DcStr> {
        self.source
            .get(start..self.position())
            .context("Invalid encapsulation range")
    }

    fn read_type(&mut self) -> Result<Option<&'a DcStr>> {
        let start = self.position();
        let tokens: Vec<u128> =
            self.chars.clone().map(DcChar::to_u128).collect();
        let Some(slice) = self.source.get(start..) else {
            return Ok(None);
        };
        let resolver = ctb_formats_dcdata::syntax::default_syntax_resolver();
        let mut nav = crate::DcDocumentNavigator::from_dc_str(slice);
        let pattern = SyntaxPattern::Sequence(vec![SyntaxElement {
            term: SyntaxTerm::NamedConstruct {
                name: "type".to_owned(),
                subtype: None,
                capture_var: None,
            },
            quantifier: Quantifier::ExactOne,
        }]);
        let mut context = MatchContext::new(None)
            .with_mode(MatchMode::Strict)
            .with_resolver(default_syntax_resolver().clone());
        let outcome = match_pattern(&tokens, &pattern, &mut context);
        if !outcome.is_matched() {
        let mut ctx = crate::SyntaxEvalContext::new(&**resolver, None);
        if !nav.eval_pattern(&pattern, &mut ctx)?
            || ctx.has_errors
            || nav.position() == 0
        {
            return Ok(None);
        }
        for _ in 0..outcome.consumed_tokens() {
        let consumed = usize::try_from(nav.position()).unwrap_or(0);
        let end = start.saturating_add(consumed);
        while self.position() < end {
            self.next()?;
        }
        Ok(Some(self.range(start)?))
    }

    fn read_length(&mut self) -> Result<Option<usize>> {
        if self.peek() != Some(crate::dc_number::DC_BEGIN_NUMBER) {
            return Ok(None);
        }
        let length =
            crate::dc_number::read_dc_number_from_reader(|| self.next())?;
        Ok(Some(usize::try_from(&length).map_err(|_| {
            anyhow::anyhow!("Invalid encapsulation byte length")
        })?))
    }

    fn read_checksum(&mut self, depth: usize) -> Result<Option<&'a DcStr>> {
        if self.peek() != Some(DC_START_ENCAPSULATION_BINARY) {
            return Ok(None);
        }
        let start = self.position();
        let (metadata, _) = self.read_payload(depth)?;
        let data_type = metadata
            .data_type
            .context("Checksum requires a hash type")?;
        let format = single_format(data_type)?;
        ensure!(
            ctb_formats_utilities::get_format_info_by_id(format)
                .is_some_and(|info| info.category == "hash"),
            "Checksum type is not a hash format"
        );
        Ok(Some(self.range(start)?))
    }

    fn read_payload(
        &mut self,
        depth: usize,
    ) -> Result<(Metadata<'a>, Vec<u8>)> {
        ensure!(
            depth < 32,
            "Encapsulation metadata nesting exceeds 32 levels"
        );
        ensure!(
            self.next()? == DC_START_ENCAPSULATION_BINARY,
            "Expected Dc 203"
        );
        let data_type = self.read_type()?;
        let length = self.read_length()?;
        let checksum = self.read_checksum(depth.saturating_add(1))?;
        let mut digits = Vec::new();
        loop {
            let ch = self.next()?;
            if ch == DC_END_ENCAPSULATION_BINARY {
                break;
            }
            let short = ch.to_short()?;
            ensure!(
                ctb_formats_eite::dc::is_dc_base64_encapsulation_character(
                    short
                ),
                "Expected base64 Dc in binary encapsulation"
            );
            digits.push(short);
        }
        let data = ctb_formats_eite::dc::dc_encapsulated_raw_to_bytes(&digits)?;
        if let Some(length) = length {
            ensure!(length == data.len(), "Encapsulation byte length mismatch");
        }
        let metadata = Metadata {
            data_type,
            checksum,
        };
        verify(&metadata, &data, depth)?;
        Ok((metadata, data))
    }
}

fn single_format(data_type: &DcStr) -> Result<FormatId> {
    let mut chars = data_type.chars();
    let ch = chars.next().context("Missing checksum type")?;
    let format = FormatId::from_dc_id(ch.to_u128())
        .context("Unknown checksum format")?;
    ensure!(
        chars.next().is_none(),
        "Checksum requires a single hash format"
    );
    Ok(format)
}

fn verify(metadata: &Metadata<'_>, data: &[u8], depth: usize) -> Result<()> {
    if let Some(checksum) = metadata.checksum {
        let mut cursor = Cursor::new(checksum);
        let (digest_metadata, digest) =
            cursor.read_payload(depth.saturating_add(1))?;
        ensure!(cursor.peek().is_none(), "Trailing checksum content");
        let format = single_format(
            digest_metadata.data_type.context("Missing checksum type")?,
        )?;
        let algorithm = SupportedHashFormat::from_format_id(format)
            .with_context(|| {
                format!(
                    "Unsupported encapsulation checksum algorithm: {format:?}"
                )
            })?;
        ensure!(
            ctb_formats_checksum::hash(data, algorithm) == digest,
            "Encapsulation checksum mismatch ({format:?})"
        );
    }
    Ok(())
}

pub(crate) fn parse_metadata(source: &DcStr) -> Result<Metadata<'_>> {
    let mut cursor = Cursor::new(source);
    let data_type = cursor.read_type()?;
    let checksum = cursor.read_checksum(0)?;
    ensure!(
        cursor.peek().is_none(),
        "Trailing or invalid encapsulation metadata"
    );
    Ok(Metadata {
        data_type,
        checksum,
    })
}

pub(crate) fn validate_metadata(source: &DcStr, data: &[u8]) -> Result<()> {
    verify(&parse_metadata(source)?, data, 0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChecksumSpec {
    pub algorithm: SupportedHashFormat,
    pub expected_digest: Vec<u8>,
}

pub(crate) fn parse_checksum_spec(source: &DcStr) -> Result<Option<ChecksumSpec>> {
    let metadata = parse_metadata(source)?;
    if let Some(checksum) = metadata.checksum {
        let mut cursor = Cursor::new(checksum);
        let (digest_metadata, digest) = cursor.read_payload(1)?;
        ensure!(cursor.peek().is_none(), "Trailing checksum content");
        let format = single_format(
            digest_metadata.data_type.context("Missing checksum type")?,
        )?;
        let algorithm = SupportedHashFormat::from_format_id(format)
            .with_context(|| {
                format!(
                    "Unsupported encapsulation checksum algorithm: {format:?}"
                )
            })?;
        Ok(Some(ChecksumSpec {
            algorithm,
            expected_digest: digest,
        }))
    } else {
        Ok(None)
    }
}

pub(crate) fn parse_text(source: &DcStr) -> Result<(DcString, Vec<u8>, usize)> {
    let mut cursor = Cursor::new(source);
    let (metadata, data) = cursor.read_payload(0)?;
    let mut header = DcString::new();
    if let Some(data_type) = metadata.data_type {
        header.push_dc_str(data_type);
    }
    if let Some(checksum) = metadata.checksum {
        header.push_dc_str(checksum);
    }
    Ok((header, data, cursor.position()))
}

pub(crate) fn write_text(
    target: &mut DcString,
    data: &[u8],
    metadata: &DcStr,
) -> Result<()> {
    let metadata = parse_metadata(metadata)?;
    target.push(DC_START_ENCAPSULATION_BINARY);
    if let Some(data_type) = metadata.data_type {
        target.push_dc_str(data_type);
    }
    for ch in crate::dc_number::integer_to_dc_number_chars(
        &crate::Integer::from(data.len()),
    )? {
        target.push(ch);
    }
    if let Some(checksum) = metadata.checksum {
        target.push_dc_str(checksum);
    }
    for dc in ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(data)? {
        target.push(DcChar::from_short(dc));
    }
    target.push(DC_END_ENCAPSULATION_BINARY);
    Ok(())
}

pub(crate) fn checksum_text(format: DcChar, digest: &[u8]) -> Result<DcString> {
    let mut data_type = DcString::new();
    data_type.push(format);
    let mut out = DcString::new();
    write_text(&mut out, digest, &data_type)?;
    Ok(out)
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
    use crate::{DcMixed, DcMixedReader, DcMstring};

    #[crate::ctb_test]
    fn typed_dcutf_payload_is_binary_not_inline_text() -> Result<()> {
        let mut payload = DcString::from("hello");
        payload.push(DC_END_ENCAPSULATION_BINARY);
        payload.push(DC_START_ENCAPSULATION_BINARY);
        for dc in ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(b"data")? {
            payload.push(DcChar::from_short(dc));
        }
        for checksummed in [false, true] {
            let mut metadata = DcString::new();
            metadata.push(FormatId::DcUtf.dc_char().context("Missing DcUtf type")?);
            if checksummed {
                metadata.push_dc_str(&checksum_text(
                    ctb_utilities::format_id::DC_SHA256,
                    &ctb_formats_checksum::sha256(payload.as_bytes()),
                )?);
            }
            let mut mixed = DcMstring::new();
            mixed.push_binary_with_metadata(payload.as_bytes(), &metadata)?;
            let text = mixed.to_dc_string()?;
            ctb_formats_dcdata::syntax::validate_dc_document_strict(
                &text.chars().map(DcChar::to_u128).collect::<Vec<_>>(),
            )?;
            crate::validate_dc_str_strict(&text)?;
            let restored = DcMstring::from_dc_str(&text)?;
            assert_eq!(restored.as_bytes(), mixed.as_bytes());
            let mut reader = DcMixedReader::new(&restored);
            assert_eq!(reader.peek_char()?, Some(DC_START_ENCAPSULATION_BINARY));
            assert_eq!(&*reader.read_binary_payload()?, payload.as_bytes());
            assert!(reader.peek_char()?.is_none());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn typed_binary_metadata_and_non_sha256_checksum() -> Result<()> {
        let data = b"\xff\x00raw payload";
        let mut metadata = DcString::new();
        metadata.push(
            FormatId::FileName
                .dc_char()
                .context("Missing filename type")?,
        );
        let algorithm = SupportedHashFormat::XxHash64;
        let checksum = checksum_text(
            algorithm
                .to_format_id()
                .dc_char()
                .context("Missing hash type")?,
            &ctb_formats_checksum::hash(data, algorithm),
        )?;
        metadata.push_dc_str(&checksum);
        let mut mixed = DcMstring::new();
        mixed.push_binary_with_metadata(data, &metadata)?;
        let text = mixed.to_dc_string()?;
        ctb_formats_dcdata::syntax::validate_dc_document_strict(
            &text.chars().map(DcChar::to_u128).collect::<Vec<_>>(),
        )?;
        crate::validate_dc_str_strict(&text)?;
        let restored = DcMstring::from_dc_str(&text)?;
        let chunk = restored.chunks().next().context("Missing binary chunk")?;
        let DcMixed::BinaryWithMetadata {
            data: restored_data,
            metadata: restored_metadata,
        } = chunk
        else {
            bail!("Expected type-2 binary");
        };
        assert_eq!(restored_data, data);
        assert_eq!(restored_metadata, &*metadata);
        let mut reader = DcMixedReader::from_dc_str(&text);
        assert_eq!(&*reader.read_binary_payload()?, data);
        assert!(reader.peek_char()?.is_none());
        let mut corrupted = data.to_vec();
        *corrupted.first_mut().context("Empty payload")? ^= 1;
        assert!(
            mixed
                .push_binary_with_metadata(&corrupted, &metadata)
                .is_err()
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn optional_metadata_empty_payload_and_legacy_base64() -> Result<()> {
        let mut old = DcString::new();
        old.push(DC_START_ENCAPSULATION_BINARY);
        for short in
            ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(b"legacy")?
        {
            old.push(DcChar::from_short(short));
        }
        old.push(DC_END_ENCAPSULATION_BINARY);
        let (metadata, data, consumed) = parse_text(&old)?;
        assert!(metadata.is_empty());
        assert_eq!(data, b"legacy");
        assert_eq!(consumed, old.len());
        let shorts =
            ctb_formats_eite::dc::bytes_to_dc_encapsulated_binary(b"legacy")?;
        let source =
            crate::converters::dctext::dcarray_to_dcstring(&shorts)?.result;
        assert_eq!(parse_text(&source)?.1, b"legacy");
        let mut mixed = DcMstring::new();
        mixed.push_binary(b"");
        let text = mixed.to_dc_string()?;
        assert_eq!(
            text.chars().nth(1),
            Some(crate::dc_number::DC_BEGIN_NUMBER)
        );
        assert!(parse_text(&text)?.1.is_empty());
        mixed.push_checksum(ctb_utilities::format_id::DC_SHA256, &[0u8; 32])?;
        let mut reader = DcMixedReader::new(&mixed);
        assert!(reader.read_binary_payload()?.is_empty());
        let (format, digest) = reader.read_checksum_payload()?;
        assert_eq!(format, Some(ctb_utilities::format_id::DC_SHA256));
        assert_eq!(&*digest, &[0u8; 32]);
        Ok(())
    }

    #[crate::ctb_test]
    fn rejects_bad_lengths_checksums_and_metadata() -> Result<()> {
        let mut bad_length = DcString::new();
        bad_length.push(DC_START_ENCAPSULATION_BINARY);
        for ch in crate::dc_number::integer_to_dc_number_chars(
            &crate::Integer::from(1),
        )? {
            bad_length.push(ch);
        }
        bad_length.push(DC_END_ENCAPSULATION_BINARY);
        assert!(parse_text(&bad_length).is_err());
        let mut reader = DcMixedReader::from_dc_str(&bad_length);
        assert!(reader.read_binary_payload().is_err());
        let unsupported = checksum_text(
            FormatId::Md5.dc_char().context("Missing MD5 type")?,
            &[0u8; 16],
        )?;
        assert!(validate_metadata(&unsupported, b"payload").is_err());
        assert!(parse_metadata(&DcString::from("not metadata")).is_err());
        let mut mixed = DcMstring::new();
        mixed.push_binary(b"payload");
        let original = mixed.into_bytes();
        for end in crate::DCMM_HEADER_LEN.saturating_add(6)..original.len() {
            assert!(
                crate::validate_dcmst(
                    original.get(..end).context("Invalid test range")?
                )
                .is_err()
            );
        }
        let mut overflow = original.clone();
        let start = crate::DCMM_HEADER_LEN.saturating_add(23);
        overflow
            .get_mut(start..start.saturating_add(16))
            .context("Missing metadata length")?
            .fill(0xff);
        assert!(crate::validate_dcmst(&overflow).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn legacy_sha256_runs_and_streaming_writers() -> Result<()> {
        let data = b"legacy";
        let digest = ctb_formats_checksum::sha256(data);
        let mut legacy = DcMstring::new();
        legacy.push_chunk(&DcMixed::binary_sha256(data, digest))?;
        let restored = DcMstring::from_dc_str(&legacy.to_dc_string()?)?;
        assert!(
            restored
                .chunks()
                .next()
                .context("Missing chunk")?
                .verify_checksum()
        );
        let mut bytes = DcMstring::new().into_bytes();
        DcMstring::write_binary_encapsulation(
            &mut bytes,
            &mut data.as_slice(),
            u64::try_from(data.len())?,
            Some(digest),
        )?;
        let streamed = DcMstring::from_bytes(bytes)?;
        assert!(
            streamed
                .chunks()
                .next()
                .context("Missing streamed chunk")?
                .verify_checksum()
        );
        let mut buffered = DcMstring::new();
        buffered.push_binary_from_reader(
            &mut data.as_slice(),
            u64::try_from(data.len())?,
            Some(digest),
        )?;
        assert_eq!(buffered.as_bytes(), streamed.as_bytes());
        let original = buffered.as_bytes().to_vec();
        assert!(
            buffered
                .push_binary_from_reader(&mut data.as_slice(), 100, None)
                .is_err()
        );
        assert_eq!(buffered.as_bytes(), original);
        assert!(
            buffered
                .push_binary_from_reader(
                    &mut data.as_slice(),
                    u64::try_from(data.len())?,
                    Some([0u8; 32]),
                )
                .is_err()
        );
        assert_eq!(buffered.as_bytes(), original);
        Ok(())
    }
}
