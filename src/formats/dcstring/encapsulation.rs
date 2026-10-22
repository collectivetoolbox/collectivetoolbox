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
    default_syntax_resolver, DcRole, Quantifier, SyntaxElement, SyntaxPattern,
    SyntaxRuleResolver, SyntaxTerm,
};

pub(crate) struct Metadata<'a> {
    pub data_type: Option<&'a DcStr>,
    pub checksum: Option<&'a DcStr>,
    pub checksum_spec: Option<ChecksumSpec>,
}

#[cfg(test)]
std::thread_local! {
    static PAYLOAD_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

struct Cursor<'a> {
    source: &'a DcStr,
    chars: DcChars<'a>,
    checksum_action: crate::DiagnosticAction,
    max_depth: usize,
    resolver: &'a dyn SyntaxRuleResolver,
}

impl<'a> Cursor<'a> {
    fn with_policy(
        source: &'a DcStr,
        policy: &crate::DcProjectionPolicy,
        resolver: &'a dyn SyntaxRuleResolver,
    ) -> Self {
        Self {
            source,
            chars: source.chars(),
            checksum_action: policy.syntax_error,
            max_depth: policy.max_encapsulation_metadata_depth,
            resolver,
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
        let Some(slice) = self.source.get(start..) else {
            return Ok(None);
        };
        let mut nav = crate::DcDocumentNavigator::from_dc_str(slice);
        let pattern = SyntaxPattern::Sequence(vec![SyntaxElement {
            term: SyntaxTerm::NamedConstruct {
                name: "type".to_owned(),
                subtype: None,
                capture_var: None,
            },
            quantifier: Quantifier::ExactOne,
        }]);
        let mut ctx = crate::SyntaxEvalContext::new(self.resolver, None);
        if !nav.eval_pattern(&pattern, &mut ctx)?
            || ctx.has_errors
            || nav.position() == 0
        {
            return Ok(None);
        }
        let consumed = usize::try_from(nav.position())?;
        let end = start.saturating_add(consumed);
        while self.position() < end {
            self.next()?;
        }
        Ok(Some(self.range(start)?))
    }

    fn read_length(&mut self) -> Result<Option<usize>> {
        if !self.peek().is_some_and(|ch| self.resolver.token_fills_role(ch.to_u128(), ctb_formats_dcdata::syntax::DcRole::BeginNumber)) {
            return Ok(None);
        }
        let resolver = self.resolver;
        let length =
            crate::dc_number::read_dc_number_from_reader_with_resolver(|| self.next(), resolver)?;
        Ok(Some(usize::try_from(&length).map_err(|_| {
            anyhow::anyhow!("Invalid encapsulation byte length")
        })?))
    }

    fn read_checksum(&mut self) -> Result<Option<(&'a DcStr, Option<ChecksumSpec>)>> {
        if !self.peek().is_some_and(|ch| self.resolver.token_fills_role(ch.to_u128(), ctb_formats_dcdata::syntax::DcRole::StartEncapsulationBinary)) {
            return Ok(None);
        }
        let start = self.position();
        let (metadata, digest) = self.read_payload()?;
        let spec = self.checksum_spec_for(metadata.data_type, digest)?;
        Ok(Some((self.range(start)?, spec)))
    }

    fn checksum_spec_for(
        &self,
        data_type: Option<&'a DcStr>,
        digest: Vec<u8>,
    ) -> Result<Option<ChecksumSpec>> {
        let data_type = data_type.context("Checksum requires a hash type")?;
        let policy = crate::DcProjectionPolicy {
            syntax_error: self.checksum_action,
            ..crate::DcProjectionPolicy::strict()
        };
        let algorithm = ChecksumSpec::algorithm_for_digest_with_resolver(
            data_type,
            u64::try_from(digest.len())?,
            &policy,
            self.resolver,
        )?;
        Ok(algorithm.map(|algorithm| ChecksumSpec { algorithm, expected_digest: digest }))
    }

    fn read_body(&mut self, length: Option<usize>) -> Result<Vec<u8>> {
        let mut digits = Vec::new();
        loop {
            let ch = self.next()?;
            if self.resolver.token_fills_role(ch.to_u128(), ctb_formats_dcdata::syntax::DcRole::EndEncapsulationBinary) {
                break;
            }
            ensure!(
                self.resolver.token_fills_role(ch.to_u128(), ctb_formats_dcdata::syntax::DcRole::Base64Digit)
                    || self.resolver.token_fills_role(ch.to_u128(), ctb_formats_dcdata::syntax::DcRole::Base64Padding),
                "Expected base64 Dc in binary encapsulation"
            );
            let canon_u128 = self.resolver.canonical_token(ch.to_u128());
            let canon_ch = DcChar::from_u128(canon_u128);
            let short = canon_ch.to_short()?;
            digits.push(short);
        }
        let data = ctb_formats_eite::dc::dc_encapsulated_raw_to_bytes(&digits)?;
        if let Some(length) = length {
            ensure!(length == data.len(), "Encapsulation byte length mismatch");
        }
        Ok(data)
    }

    /// Parses one payload level and every nested checksum-metadata level.
    ///
    /// Nesting is walked with an explicit frame stack instead of recursion so
    /// a raised caller budget cannot exhaust the call stack on adversarial
    /// input. The nesting budget is checked once per payload level entered.
    fn read_payload(&mut self) -> Result<(Metadata<'a>, Vec<u8>)> {
        let resolver = self.resolver;
        let mut frames: Vec<PayloadFrame<'a>> = Vec::new();
        let mut level = loop {
            #[cfg(test)]
            PAYLOAD_READS.with(|count| count.set(count.get().saturating_add(1)));
            ensure!(
                frames.len() < self.max_depth,
                "Encapsulation metadata nesting exceeds resource limit ({} levels)",
                self.max_depth
            );
            let start = self.position();
            let next_ch = self.next()?;
            ensure!(
                resolver.token_fills_role(next_ch.to_u128(), ctb_formats_dcdata::syntax::DcRole::StartEncapsulationBinary),
                "Expected Dc 203"
            );
            let data_type = self.read_type()?;
            let length = self.read_length()?;
            if self.peek().is_some_and(|ch| resolver.token_fills_role(ch.to_u128(), ctb_formats_dcdata::syntax::DcRole::StartEncapsulationBinary)) {
                frames.push(PayloadFrame { start, data_type, length });
                continue;
            }
            let data = self.read_body(length)?;
            break PayloadLevel {
                data_type,
                raw: self.range(start)?,
                data,
                checksum: None,
            };
        };
        loop {
            let PayloadLevel { data_type, raw, data, checksum } = level;
            let metadata = Metadata {
                data_type,
                checksum: checksum.as_ref().map(|(span, _)| *span),
                checksum_spec: checksum.and_then(|(_, spec)| spec),
            };
            verify(&metadata, &data, self.checksum_action)?;
            let Some(frame) = frames.pop() else {
                return Ok((metadata, data));
            };
            let spec = self.checksum_spec_for(data_type, data)?;
            let parent_data = self.read_body(frame.length)?;
            level = PayloadLevel {
                data_type: frame.data_type,
                raw: self.range(frame.start)?,
                data: parent_data,
                checksum: Some((raw, spec)),
            };
        }
    }
}

/// An enclosing payload level awaiting its own checksum-metadata body while a
/// nested checksum payload is being parsed.
struct PayloadFrame<'a> {
    start: usize,
    data_type: Option<&'a DcStr>,
    length: Option<usize>,
}

/// A fully parsed payload level that still has to be verified and, when it is
/// a nested checksum payload, folded into its enclosing level.
struct PayloadLevel<'a> {
    data_type: Option<&'a DcStr>,
    raw: &'a DcStr,
    data: Vec<u8>,
    checksum: Option<(&'a DcStr, Option<ChecksumSpec>)>,
}

fn single_format(data_type: &DcStr, resolver: &dyn SyntaxRuleResolver) -> Result<FormatId> {
    let mut chars: Vec<DcChar> = data_type.chars().collect();
    if chars.len() >= 2
        && resolver.token_fills_role(chars.first().map_or(0, |c| c.to_u128()), DcRole::BeginTypeDef)
        && resolver.token_fills_role(chars.last().map_or(0, |c| c.to_u128()), DcRole::EndTypeDef)
    {
        chars.pop();
        chars.remove(0);
    }
    ensure!(
        chars.len() == 1,
        "Checksum requires a single hash format"
    );
    let ch = chars[0];
    let canon = resolver.canonical_token(ch.to_u128());
    let format = FormatId::from_dc_id(canon)
        .context("Unknown checksum format")?;
    Ok(format)
}

fn verify(metadata: &Metadata<'_>, data: &[u8], checksum_action: crate::DiagnosticAction) -> Result<()> {
    if let Some(spec) = &metadata.checksum_spec {
        let mut reader = data;
        if !spec.matches_reader(&mut reader, u64::try_from(data.len())?)? {
            crate::DcProjectionPolicy::strict().handle(
                checksum_action,
                format!("Encapsulation checksum mismatch ({:?})", spec.algorithm),
            )?;
        }
    }
    Ok(())
}

pub(crate) fn parse_metadata(source: &DcStr) -> Result<Metadata<'_>> {
    parse_metadata_with_policy(source, &crate::DcProjectionPolicy::strict())
}

pub(crate) fn parse_metadata_with_policy<'a>(
    source: &'a DcStr,
    policy: &crate::DcProjectionPolicy,
) -> Result<Metadata<'a>> {
    parse_metadata_with_policy_and_resolver(source, policy, default_syntax_resolver().as_ref())
}

pub(crate) fn parse_metadata_with_policy_and_resolver<'a>(
    source: &'a DcStr,
    policy: &crate::DcProjectionPolicy,
    resolver: &'a dyn SyntaxRuleResolver,
) -> Result<Metadata<'a>> {
    let mut cursor = Cursor::with_policy(source, policy, resolver);
    let data_type = cursor.read_type()?;
    let checksum = cursor.read_checksum()?;
    ensure!(
        cursor.peek().is_none(),
        "Trailing or invalid encapsulation metadata"
    );
    Ok(Metadata {
        data_type,
        checksum: checksum.as_ref().map(|(raw, _)| *raw),
        checksum_spec: checksum.and_then(|(_, spec)| spec),
    })
}

pub(crate) fn validate_metadata(source: &DcStr, data: &[u8]) -> Result<()> {
    validate_metadata_with_policy(source, data, &crate::DcProjectionPolicy::strict())
}

pub(crate) fn validate_metadata_with_policy(
    source: &DcStr,
    data: &[u8],
    policy: &crate::DcProjectionPolicy,
) -> Result<()> {
    validate_metadata_with_policy_and_resolver(source, data, policy, default_syntax_resolver().as_ref())
}

pub(crate) fn validate_metadata_with_policy_and_resolver(
    source: &DcStr,
    data: &[u8],
    policy: &crate::DcProjectionPolicy,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<()> {
    verify(&parse_metadata_with_policy_and_resolver(source, policy, resolver)?, data, policy.syntax_error)
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Expected digest and algorithm for a binary encapsulation.
pub struct ChecksumSpec {
    /// Hash algorithm declared by the encapsulation.
    pub algorithm: SupportedHashFormat,
    /// Digest bytes to compare with the payload's computed hash.
    pub expected_digest: Vec<u8>,
}

impl ChecksumSpec {
    pub(crate) fn algorithm_for_digest(
        data_type: &DcStr,
        length: u64,
        policy: &crate::DcProjectionPolicy,
    ) -> Result<Option<SupportedHashFormat>> {
        Self::algorithm_for_digest_with_resolver(
            data_type,
            length,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    pub(crate) fn algorithm_for_digest_with_resolver(
        data_type: &DcStr,
        length: u64,
        policy: &crate::DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<SupportedHashFormat>> {
        let format = single_format(data_type, resolver)?;
        ensure!(
            ctb_formats_utilities::get_format_info_by_id(format)
                .is_some_and(|info| info.category == "hash"),
            "Checksum type is not a hash format"
        );
        let Some(algorithm) = SupportedHashFormat::from_format_id(format) else {
            policy.handle(
                policy.syntax_error,
                format!("Unsupported encapsulation checksum algorithm: {format:?}; payload integrity was not verified"),
            )?;
            return Ok(None);
        };
        let expected_length = ctb_formats_checksum::StreamingHasher::new(algorithm).finalize().len();
        ensure!(length == u64::try_from(expected_length)?, "Invalid encapsulation checksum digest length");
        Ok(Some(algorithm))
    }

    /// Checks exactly `len` payload bytes without buffering them in memory.
    pub fn matches_reader(
        &self,
        reader: &mut impl std::io::Read,
        len: u64,
    ) -> Result<bool> {
        let mut hasher = ctb_formats_checksum::StreamingHasher::new(self.algorithm);
        let mut limited = std::io::Read::take(reader, len);
        let copied = std::io::copy(&mut limited, &mut hasher)?;
        ensure!(copied == len, "Truncated encapsulation payload while checksumming");
        Ok(hasher.finalize() == self.expected_digest)
    }
}

pub(crate) fn parse_checksum_spec(source: &DcStr) -> Result<Option<ChecksumSpec>> {
    Ok(parse_metadata(source)?.checksum_spec)
}

pub(crate) fn parse_text(source: &DcStr) -> Result<(DcString, Vec<u8>, usize)> {
    parse_text_with_policy(source, &crate::DcProjectionPolicy::strict())
}

pub(crate) fn parse_text_with_policy(
    source: &DcStr,
    policy: &crate::DcProjectionPolicy,
) -> Result<(DcString, Vec<u8>, usize)> {
    parse_text_with_policy_and_resolver(source, policy, default_syntax_resolver().as_ref())
}

pub(crate) fn parse_text_with_policy_and_resolver(
    source: &DcStr,
    policy: &crate::DcProjectionPolicy,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<(DcString, Vec<u8>, usize)> {
    let mut cursor = Cursor::with_policy(source, policy, resolver);
    let (metadata, data) = cursor.read_payload()?;
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
    use crate::{DcMixed, DcMstring};

    #[crate::ctb_test]
    fn nested_checksum_parsed_once_per_level() -> Result<()> {
        let mut payloads = vec![b"payload".to_vec()];
        for _ in 0..6 {
            payloads.push(ctb_formats_checksum::sha256(payloads.last().unwrap()).to_vec());
        }
        let mut checksum = DcString::new();
        for data in payloads.iter().skip(1).rev() {
            let mut metadata = DcString::new();
            metadata.push(ctb_utilities::format_id::DC_SHA256);
            metadata.push_dc_str(&checksum);
            let mut next = DcString::new();
            write_text(&mut next, data, &metadata)?;
            checksum = next;
        }
        let mut text = DcString::new();
        write_text(&mut text, &payloads[0], &checksum)?;
        PAYLOAD_READS.with(|count| count.set(0));
        assert_eq!(parse_text(&text)?.1, payloads[0]);
        assert_eq!(PAYLOAD_READS.with(std::cell::Cell::get), payloads.len());
        PAYLOAD_READS.with(|count| count.set(0));
        let parsed = parse_checksum_spec(&checksum)?.context("Missing checksum")?;
        assert_eq!(parsed.expected_digest, payloads[1]);
        assert_eq!(PAYLOAD_READS.with(std::cell::Cell::get), payloads.len() - 1);
        Ok(())
    }

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
            crate::validate_dc_str_strict(&text)?;
            let restored = DcMstring::from_dc_str(&text)?;
            assert_eq!(restored.as_bytes(), mixed.as_bytes());
            let mut arena = crate::DcSyntaxArena::new();
            let root = crate::DcDocumentNavigator::parse_arena(restored.as_bytes(), &mut arena, &crate::DcProjectionPolicy::strict())?;
            let node = arena.node(root).context("Missing node")?;
            let crate::navigator::DcSyntaxKind::Binary { payload: binary_payload, .. } = &node.kind else {
                bail!("Expected Binary syntax node");
            };
            assert_eq!(binary_payload.materialize_bytes()?, payload.as_bytes());
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
        let mut arena = crate::DcSyntaxArena::new();
        let root = crate::DcDocumentNavigator::parse_arena(text.as_bytes(), &mut arena, &crate::DcProjectionPolicy::strict())?;
        let node = arena.node(root).context("Missing node")?;
        let crate::navigator::DcSyntaxKind::Binary { payload: binary_payload, .. } = &node.kind else {
            bail!("Expected Binary syntax node");
        };
        assert_eq!(binary_payload.materialize_bytes()?.as_ref(), data);
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
    fn every_supported_checksum_validates_encapsulated_payloads() -> Result<()> {
        let data = b"checksum capability coverage";
        for algorithm in SupportedHashFormat::ALL {
            let metadata = checksum_text(
                algorithm.to_format_id().dc_char().context("Missing hash type")?,
                &ctb_formats_checksum::hash(data, *algorithm),
            )?;
            let mut mixed = DcMstring::new();
            mixed.push_binary_with_metadata(data, &metadata)?;
            crate::validate_dc_mixed_strict(&mixed)?;
            let text = mixed.to_dc_string()?;
            crate::validate_dc_str_strict(&text)?;
            let mut corrupted = data.to_vec();
            *corrupted.first_mut().context("Empty payload")? ^= 1;
            ensure!(validate_metadata(&metadata, &corrupted).is_err());
        }
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
        let mut arena = crate::DcSyntaxArena::new();
        crate::DcDocumentNavigator::parse_document_arena(mixed.as_bytes(), &mut arena, &crate::DcProjectionPolicy::strict())?;
        let roots = arena.roots();
        assert_eq!(roots.len(), 2);
        let node0 = arena.node(roots[0]).context("Missing node0")?;
        let crate::navigator::DcSyntaxKind::Binary { payload: p0, .. } = &node0.kind else { bail!("Expected Binary") };
        assert!(p0.materialize_bytes()?.is_empty());
        let node1 = arena.node(roots[1]).context("Missing node1")?;
        let crate::navigator::DcSyntaxKind::Binary { checksum_format, payload: p1, .. } = &node1.kind else { bail!("Expected Binary") };
        assert_eq!(*checksum_format, Some(ctb_utilities::format_id::DC_SHA256));
        assert_eq!(p1.materialize_bytes()?.as_ref(), &[0u8; 32]);
        Ok(())
    }

    #[crate::ctb_test]
    fn unsupported_checksum_obeys_projection_policy() -> Result<()> {
        let checksum = checksum_text(
            FormatId::Md5.dc_char().context("Missing MD5 type")?,
            &[0_u8; 16],
        )?;
        let data = b"retained payload";
        let mut text = DcString::new();
        text.push(DC_START_ENCAPSULATION_BINARY);
        text.push_dc_str(&checksum);
        for short in ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(data)? {
            text.push(DcChar::from_short(short));
        }
        text.push(DC_END_ENCAPSULATION_BINARY);
        assert!(parse_metadata(&checksum).is_err());
        assert!(parse_text(&text).is_err());
        for action in [
            crate::DiagnosticAction::Warn,
            crate::DiagnosticAction::Log,
            crate::DiagnosticAction::Ignore,
        ] {
            let policy = crate::DcProjectionPolicy {
                syntax_error: action,
                ..crate::DcProjectionPolicy::strict()
            };
            let metadata = parse_metadata_with_policy(&checksum, &policy)?;
            assert_eq!(metadata.checksum, Some(&*checksum));
            assert!(metadata.checksum_spec.is_none());
            let (header, restored, consumed) = parse_text_with_policy(&text, &policy)?;
            assert_eq!(header, checksum);
            assert_eq!(restored, data);
            assert_eq!(consumed, text.len());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn unsupported_nested_checksum_preserves_supported_outer_checksum() -> Result<()> {
        let data = b"payload";
        let mut metadata = DcString::new();
        metadata.push(DC_START_ENCAPSULATION_BINARY);
        metadata.push(ctb_utilities::format_id::DC_SHA256);
        metadata.push_dc_str(&checksum_text(
            FormatId::Md5.dc_char().context("Missing MD5 type")?,
            &[0_u8; 16],
        )?);
        for short in ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(&ctb_formats_checksum::sha256(data))? {
            metadata.push(DcChar::from_short(short));
        }
        metadata.push(DC_END_ENCAPSULATION_BINARY);
        assert!(parse_metadata(&metadata).is_err());
        for policy in [crate::DcProjectionPolicy::forgiving(), crate::DcProjectionPolicy::forgiving_quiet()] {
            let parsed = parse_metadata_with_policy(&metadata, &policy)?;
            assert_eq!(parsed.checksum, Some(&*metadata));
            let spec = parsed.checksum_spec.context("Lost supported outer checksum")?;
            assert_eq!(spec.algorithm, SupportedHashFormat::Sha256);
            assert!(spec.matches_reader(&mut data.as_slice(), u64::try_from(data.len())?)?);
            assert!(!spec.matches_reader(&mut b"changed".as_slice(), u64::try_from(data.len())?)?);
        }
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
        let mut arena = crate::DcSyntaxArena::new();
        assert!(crate::DcDocumentNavigator::parse_arena(bad_length.as_bytes(), &mut arena, &crate::DcProjectionPolicy::strict()).is_err());
        let unsupported = checksum_text(
            FormatId::Md5.dc_char().context("Missing MD5 type")?,
            &[0u8; 16],
        )?;
        assert!(validate_metadata(&unsupported, b"payload").is_err());
        for (format, length) in [
            (ctb_utilities::format_id::DC_SHA256, 0),
            (ctb_utilities::format_id::DC_SHA256, 31),
            (ctb_utilities::format_id::DC_SHA256, 33),
        ] {
            let checksum = checksum_text(format, &vec![0; length])?;
            let mut closer = DcString::new();
            closer.push(DC_END_ENCAPSULATION_BINARY);
            let truncated = checksum.get(..checksum.len() - closer.len()).context("Invalid checksum span")?;
            for action in [crate::DiagnosticAction::Error, crate::DiagnosticAction::Warn] {
                let policy = crate::DcProjectionPolicy {
                    syntax_error: action,
                    ..crate::DcProjectionPolicy::strict()
                };
                assert!(parse_metadata_with_policy(&checksum, &policy).is_err());
                assert!(parse_metadata_with_policy(truncated, &policy).is_err());
            }
        }
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

    /// Builds a textual encapsulation chain `levels` payload levels deep,
    /// where each level's checksum payload holds the SHA-256 digest of that
    /// level's data. Returns the chain and the outermost payload's data.
    fn nesting_budget_chain(levels: usize) -> Result<(DcString, Vec<u8>)> {
        let mut digests = Vec::new();
        let mut current = b"leaf payload".to_vec();
        for _ in 0..levels {
            current = ctb_formats_checksum::sha256(&current).to_vec();
            digests.push(current.clone());
        }
        let mut text = DcString::new();
        for _ in 0..levels {
            text.push(DC_START_ENCAPSULATION_BINARY);
            text.push(ctb_utilities::format_id::DC_SHA256);
            for ch in crate::dc_number::integer_to_dc_number_chars(
                &crate::Integer::from(32),
            )? {
                text.push(ch);
            }
        }
        for digest in digests.iter().rev() {
            for dc in
                ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(digest)?
            {
                text.push(DcChar::from_short(dc));
            }
            text.push(DC_END_ENCAPSULATION_BINARY);
        }
        let outermost = digests.first().cloned().context("No payload levels")?;
        Ok((text, outermost))
    }

    #[crate::ctb_test]
    fn encapsulation_nesting_budget_is_caller_configurable() -> Result<()> {
        let default_budget = crate::DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH;
        let (at_default, _) = nesting_budget_chain(default_budget)?;
        assert!(parse_text(&at_default).is_ok());
        let (over_default, _) =
            nesting_budget_chain(default_budget.saturating_add(1))?;
        assert!(parse_text(&over_default).is_err());
        let raised = crate::DcProjectionPolicy::strict()
            .with_max_encapsulation_metadata_depth(
                default_budget.saturating_add(1),
            );
        assert!(parse_text_with_policy(&over_default, &raised).is_ok());
        let (over_raised, _) =
            nesting_budget_chain(default_budget.saturating_add(2))?;
        assert!(parse_text_with_policy(&over_raised, &raised).is_err());
        let mut metadata = DcString::new();
        metadata.push(ctb_utilities::format_id::DC_SHA256);
        metadata.push_dc_str(&over_default);
        assert!(parse_metadata(&metadata).is_err());
        assert!(parse_metadata_with_policy(&metadata, &raised).is_ok());
        Ok(())
    }

    /// Verifies that a raised caller budget parses deep checksum-metadata
    /// nesting iteratively instead of recursing once per level.
    #[crate::ctb_test]
    fn deep_nesting_under_raised_budget_is_stack_safe() -> Result<()> {
        let levels = 20_000;
        let (deep, outermost) = nesting_budget_chain(levels)?;
        let policy = crate::DcProjectionPolicy::strict()
            .with_max_encapsulation_metadata_depth(levels);
        let (header, data, consumed) = parse_text_with_policy(&deep, &policy)?;
        assert_eq!(consumed, deep.len());
        assert!(!header.is_empty());
        assert_eq!(data, outermost);
        Ok(())
    }

    /// Verifies the caller budget is honored end-to-end through document
    /// arena parsing, not only the encapsulation entry points.
    #[crate::ctb_test]
    fn navigator_honors_encapsulation_nesting_budget() -> Result<()> {
        let levels = 100;
        let (deep, outermost) = nesting_budget_chain(levels)?;
        let mut nav = crate::DcDocumentNavigator::from_dc_str(&deep);
        let mut arena = crate::DcSyntaxArena::new();
        assert!(
            nav.parse_document_into_arena(
                &mut arena,
                &crate::DcProjectionPolicy::strict()
            )
            .is_err()
        );
        let policy = crate::DcProjectionPolicy::strict()
            .with_max_encapsulation_metadata_depth(levels);
        let mut nav = crate::DcDocumentNavigator::from_dc_str(&deep);
        let mut arena = crate::DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        assert!(arena.diagnostics().is_empty());
        let Some(root) = arena.roots().first().copied() else {
            bail!("Missing arena root");
        };
        let node = arena.node(root).context("Missing node")?;
        let crate::navigator::DcSyntaxKind::Binary { payload, .. } = &node.kind
        else {
            bail!("Expected Binary node");
        };
        assert_eq!(payload.materialize_bytes()?, outermost);
        Ok(())
    }
}
