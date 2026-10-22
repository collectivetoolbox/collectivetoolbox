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

//! Direct fallible writer and binary source traits for Document Character
//! (`Dc`) streams and files.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::{BinaryPayloadRef, DcChar, DcMstring, DcStr, DcString};

use std::io::Write;

/// Supplies nonresident binary data without materializing it in memory.
pub trait DcBinarySource {
    fn open<'source>(
        &'source self,
        payload: &BinaryPayloadRef<'_>,
    ) -> Result<DcBinaryReader<'source>>;
}

pub struct DcBinaryReader<'source> {
    pub reader: Box<dyn std::io::Read + 'source>,
    pub sha256: Option<[u8; 32]>,
}

/// Identifies a binary-source read or verification failure during encoding.
#[derive(Debug)]
pub struct DcBinarySourceError;

impl std::fmt::Display for DcBinarySourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Binary source encoding or verification failed")
    }
}

impl std::error::Error for DcBinarySourceError {}

/// A headerless DcMM encoder writing directly to a fallible output stream.
pub struct DcWriter<'output> {
    output: &'output mut dyn Write,
    bytes: u64,
    source: Option<&'output dyn DcBinarySource>,
    grammar: Option<std::io::BufWriter<std::fs::File>>,
}

impl<'output> DcWriter<'output> {
    pub fn new(output: &'output mut dyn Write) -> Self {
        Self {
            output,
            bytes: 0,
            source: None,
            grammar: None,
        }
    }

    pub fn with_binary_source(
        output: &'output mut dyn Write,
        source: &'output dyn DcBinarySource,
    ) -> Self {
        Self {
            output,
            bytes: 0,
            source: Some(source),
            grammar: None,
        }
    }

    pub fn push_binary_ref(
        &mut self,
        payload: &BinaryPayloadRef<'_>,
    ) -> Result<()> {
        if let Some(bytes) = payload.as_resident_bytes() {
            return self.push_binary_with_sha256(bytes);
        }
        let result = (|| -> Result<()> {
            let source = self
                .source
                .context("Lazy binary encoding requires a backing source")?;
            let mut opened = source.open(payload)?;
            self.push_binary_from_reader(
                &mut opened.reader,
                payload.len(),
                opened.sha256,
            )?;
            let mut extra = [0_u8; 1];
            ensure!(
                opened.reader.read(&mut extra)? == 0,
                "Binary source exceeds its declared length"
            );
            Ok(())
        })();
        result.context(DcBinarySourceError)
    }

    pub const fn len(&self) -> u64 {
        self.bytes
    }

    pub const fn is_empty(&self) -> bool {
        self.bytes == 0
    }

    pub fn push_char(&mut self, character: impl Into<DcChar>) -> Result<()> {
        let character = character.into();
        let mut buffer = [0_u8; 24];
        let length = ctb_formats_utf_8e_128::encode_utf_8e_128_buf(
            &mut buffer,
            character.0,
        );
        self.emit(
            buffer
                .get(..length)
                .context("Invalid encoded Dc character length")?,
        )?;
        Ok(())
    }

    pub fn push_str(&mut self, text: &str) -> Result<()> {
        self.emit(text.as_bytes())?;
        Ok(())
    }

    pub fn push_dc_str(&mut self, text: &DcStr) -> Result<()> {
        self.emit(text.as_bytes())?;
        Ok(())
    }

    pub fn push_binary(&mut self, bytes: &[u8]) -> Result<()> {
        self.push_binary_with_metadata(bytes, &DcString::new())
    }

    pub fn push_binary_with_metadata(
        &mut self,
        bytes: &[u8],
        metadata: &DcStr,
    ) -> Result<()> {
        crate::encapsulation::validate_metadata(metadata, bytes)?;
        self.emit(&crate::dc_mixed::DC_203_BYTES)?;
        self.emit(&[crate::dc_mixed::TYPE_BINARY_METADATA])?;
        self.emit(&u128::try_from(bytes.len())?.to_be_bytes())?;
        self.emit(&u128::try_from(metadata.len())?.to_be_bytes())?;
        self.emit(metadata.as_bytes())?;
        self.emit(bytes)?;
        Ok(())
    }

    pub fn push_binary_with_sha256(&mut self, bytes: &[u8]) -> Result<()> {
        let digest = ctb_formats_checksum::sha256(bytes);
        let metadata = crate::encapsulation::checksum_text(
            ctb_utilities::format_id::DC_SHA256,
            &digest,
        )?;
        self.push_binary_with_metadata(bytes, &metadata)
    }

    pub fn push_checksum(
        &mut self,
        format: impl Into<DcChar>,
        digest: &[u8],
    ) -> Result<()> {
        let mut metadata = DcString::new();
        metadata.push(format.into());
        self.push_binary_with_metadata(digest, &metadata)
    }

    pub fn push_binary_from_reader<R: std::io::Read + ?Sized>(
        &mut self,
        reader: &mut R,
        size: u64,
        sha256: Option<[u8; 32]>,
    ) -> Result<()> {
        DcMstring::write_binary_encapsulation(
            &mut RawOutput(self),
            reader,
            size,
            sha256,
        )?;
        Ok(())
    }

    fn emit(&mut self, bytes: &[u8]) -> Result<()> {
        RawOutput(self).write_all(bytes)?;
        Ok(())
    }
}

/// Emits a headerless record and validates its grammar before returning.
///
/// Binary framing, lengths and supplied checksums are checked during emission.
/// Exact emitted bytes are spooled to an anonymous file and checked with the
/// file-backed navigator, without expanding binary payloads into characters.
/// This bounds memory but requires temporary disk space proportional to the
/// record size. Errors can leave partial output; callers must withhold
/// their transaction footer until this function succeeds.
/// FIXME: Will break on `csc archive` of large files.
pub fn validate_and_write_dc_record(
    output: &mut dyn Write,
    source: Option<&dyn DcBinarySource>,
    encode: impl FnOnce(&mut DcWriter<'_>) -> Result<()>,
) -> Result<()> {
    let mut grammar = std::io::BufWriter::new(
        tempfile::tempfile().context("Cannot create Dc validation spool")?,
    );
    grammar.write_all(DcMstring::new().as_bytes())?;
    let mut writer = DcWriter {
        output,
        bytes: 0,
        source,
        grammar: Some(grammar),
    };
    encode(&mut writer)?;
    let grammar = writer
        .grammar
        .take()
        .context("Missing encoding grammar")?
        .into_inner()?;
    crate::DcDocumentNavigator::from_dcmm_file(grammar, 0)?
        .validate_document_strict()
}

impl Write for DcWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.grammar.is_some() {
            return Err(std::io::Error::other(
                "Raw writes bypass Dc grammar validation",
            ));
        }
        RawOutput(self).write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}

struct RawOutput<'writer, 'output>(&'writer mut DcWriter<'output>);

impl Write for RawOutput<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let writer = &mut self.0;
        let count = writer.output.write(bytes)?;
        if count > bytes.len() {
            return Err(std::io::Error::other(
                "Writer returned an invalid byte count",
            ));
        }
        if let Some(grammar) = &mut writer.grammar {
            grammar.write_all(bytes.get(..count).ok_or_else(|| {
                std::io::Error::other("Invalid output byte count")
            })?)?;
        }
        writer.bytes = writer
            .bytes
            .checked_add(u64::try_from(count).map_err(std::io::Error::other)?)
            .ok_or_else(|| std::io::Error::other("Dc output size overflow"))?;
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.output.flush()
    }
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
    use crate::{DcMixedEncode, LazyBinaryPayload, dc};

    #[derive(Debug, PartialEq, Eq, crate::DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct DerivedPayload {
        #[dc(392, default, omit_default)]
        body: Option<LazyBinaryPayload>,
    }

    struct PayloadSource(Vec<u8>);

    #[crate::ctb_test]
    fn byte_backed_grammar_preserves_payload_type_and_length_constraints()
    -> Result<()> {
        use ctb_formats_dcdata::syntax::{
            default_syntax_resolver, parse_dc_syntax,
        };
        let resolver = default_syntax_resolver();
        for size in [0, 31, 32, 33, 128 * 1024] {
            for typed in [false, true] {
                let mut metadata = DcString::new();
                if typed {
                    metadata.push(ctb_utilities::format_id::DC_SHA256);
                }
                let payload = vec![42; size];
                let mut buffered = DcMstring::new();
                buffered.push_binary_with_metadata(&payload, &metadata)?;
                let mut output = Vec::new();
                let mut grammar =
                    std::io::BufWriter::new(tempfile::tempfile()?);
                grammar.write_all(DcMstring::new().as_bytes())?;
                let mut writer = DcWriter {
                    output: &mut output,
                    bytes: 0,
                    source: None,
                    grammar: Some(grammar),
                };
                writer.push_binary_with_metadata(&payload, &metadata)?;
                let recorded = writer
                    .grammar
                    .take()
                    .context("Missing validation document")?
                    .into_inner()?;
                for constraint in [
                    ":[data:bytes=32]",
                    ":[data:Sha256;bytes=32]",
                    ":[data:bytes=131072]",
                ] {
                    let pattern = parse_dc_syntax(constraint)?;
                    let mut expected =
                        crate::SyntaxEvalContext::new(&**resolver, None);
                    let mut actual =
                        crate::SyntaxEvalContext::new(&**resolver, None);
                    let expected_match =
                        crate::DcDocumentNavigator::from_mstr(&buffered)
                            .eval_pattern(&pattern.pattern, &mut expected)?;
                    let actual_match =
                        crate::DcDocumentNavigator::from_dcmm_file(
                            recorded.try_clone()?,
                            0,
                        )?
                        .eval_pattern(&pattern.pattern, &mut actual)?;
                    ensure!(
                        actual_match == expected_match,
                        "Byte-backed grammar differs for size {size}, typed {typed}, constraint {constraint}"
                    );
                    if constraint == ":[data:bytes=32]" {
                        ensure!(actual_match == (size == 32));
                    }
                    if constraint == ":[data:Sha256;bytes=32]" {
                        ensure!(actual_match == (size == 32 && typed));
                    }
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn byte_backed_grammar_checks_exact_streamed_section_bytes() -> Result<()> {
        let payload = vec![42_u8; 128 * 1024];
        for supplied_digest in [false, true] {
            let payload_digest =
                supplied_digest.then(|| ctb_formats_checksum::sha256(&payload));
            let mut body = Vec::new();
            DcWriter::new(&mut body).push_binary_from_reader(
                &mut payload.as_slice(),
                u64::try_from(payload.len())?,
                payload_digest,
            )?;
            let valid_digest = ctb_formats_checksum::sha256(&body);
            for digest in [valid_digest, [0_u8; 32]] {
                let mut output = ShortWriter(Vec::new());
                let result =
                    validate_and_write_dc_record(&mut output, None, |writer| {
                        writer.push_char(dc::DC_BEGIN_CHECKSUMMED_SECTION)?;
                        writer.push_binary_from_reader(
                            &mut payload.as_slice(),
                            u64::try_from(payload.len())?,
                            payload_digest,
                        )?;
                        writer.push_char(dc::DC_END_CHECKSUMMED_SECTION)?;
                        writer.push_char(dc::DC_SECTION_DIGEST)?;
                        writer.push_checksum(
                            ctb_utilities::format_id::DC_SHA256,
                            &digest,
                        )
                    });
                ensure!(
                    result.is_ok() == (digest == valid_digest),
                    "Unexpected streamed section result: {result:?}"
                );
                let mut document = DcMstring::new().into_bytes();
                document.extend_from_slice(&output.0);
                let document = DcMstring::from_bytes(document)?;
                ensure!(
                    crate::navigator::validate_dc_mixed_strict(&document)
                        .is_ok()
                        == result.is_ok()
                );
            }
        }
        Ok(())
    }

    #[derive(crate::DcMixed)]
    #[dc(encode_only)]
    #[dc(begin = 317, end = 318)]
    struct BorrowedPayload<'payload> {
        #[dc(392)]
        body: &'payload LazyBinaryPayload,
    }

    struct FailingWriter {
        remaining: usize,
    }

    impl Write for FailingWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.remaining == 0 {
                return Err(std::io::Error::other("Injected output failure"));
            }
            let count = bytes.len().min(self.remaining);
            self.remaining = self.remaining.saturating_sub(count);
            Ok(count)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[crate::ctb_test]
    fn borrowed_derives_propagate_every_output_failure() -> Result<()> {
        let body =
            LazyBinaryPayload(BinaryPayloadRef::Inline(b"borrowed".to_vec()));
        let record = BorrowedPayload { body: &body };
        let buffered = record.to_dc_mixed()?;
        let mut output = ShortWriter(Vec::new());
        record.write_dc_mixed(&mut output)?;
        ensure!(output.0 == buffered.as_bytes());
        for remaining in 0..buffered.as_bytes().len() {
            ensure!(
                record
                    .write_dc_mixed(&mut FailingWriter { remaining })
                    .is_err()
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn streamed_grammar_validation_rejects_unclosed_records_and_raw_writes()
    -> Result<()> {
        ensure!(
            validate_and_write_dc_record(&mut Vec::new(), None, |writer| {
                writer.push_char(dc::DC_BEGIN_LIST)
            })
            .is_err()
        );
        ensure!(
            validate_and_write_dc_record(&mut Vec::new(), None, |writer| {
                writer.write_all(b"unvalidated")?;
                Ok(())
            })
            .is_err()
        );
        let payload = DerivedPayload {
            body: Some(LazyBinaryPayload(BinaryPayloadRef::Inline(
                vec![42; 128 * 1024],
            ))),
        };
        let mut bytes = DcMstring::new().into_bytes();
        validate_and_write_dc_record(&mut bytes, None, |writer| {
            payload.encode_dc_mixed_to(writer)
        })?;
        crate::validate_dc_mixed_strict(&DcMstring::from_bytes(bytes)?)?;
        Ok(())
    }

    impl DcBinarySource for PayloadSource {
        fn open<'source>(
            &'source self,
            payload: &BinaryPayloadRef<'_>,
        ) -> Result<DcBinaryReader<'source>> {
            ensure!(matches!(
                payload,
                BinaryPayloadRef::Source { source_id: 0, .. }
            ));
            Ok(DcBinaryReader {
                reader: Box::new(self.0.as_slice()),
                sha256: Some(ctb_formats_checksum::sha256(&self.0)),
            })
        }
    }

    #[crate::ctb_test]
    fn derived_lazy_payload_matches_resident_encoding() -> Result<()> {
        let source = PayloadSource(vec![42; 128 * 1024]);
        let resident = DerivedPayload {
            body: Some(LazyBinaryPayload(BinaryPayloadRef::Inline(
                source.0.clone(),
            ))),
        };
        let lazy = DerivedPayload {
            body: Some(LazyBinaryPayload(BinaryPayloadRef::Source {
                source_id: 0,
                len: u64::try_from(source.0.len())?,
            })),
        };
        let expected = resident.to_dc_mixed()?;
        let mut output = ShortWriter(Vec::new());
        lazy.encode_dc_mixed_to(&mut DcWriter::with_binary_source(
            &mut output,
            &source,
        ))?;
        ensure!(output.0 == expected.payload_bytes());
        ensure!(lazy.to_dc_mixed().is_err());
        Ok(())
    }

    struct ShortWriter(Vec<u8>);

    impl Write for ShortWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let count = bytes.len().min(3);
            self.0.extend_from_slice(
                bytes.get(..count).ok_or_else(|| {
                    std::io::Error::other("Invalid test slice")
                })?,
            );
            Ok(count)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[crate::ctb_test]
    fn streaming_primitives_match_buffered_output() -> Result<()> {
        let mut expected = DcMstring::new();
        expected.push_char(dc::DC_BEGIN_LIST);
        expected.push_binary_with_sha256(b"payload")?;
        expected.push_char(dc::DC_END_LIST);
        let mut output = ShortWriter(Vec::new());
        let mut writer = DcWriter::new(&mut output);
        writer.push_char(dc::DC_BEGIN_LIST)?;
        writer.push_binary_with_sha256(b"payload")?;
        writer.push_char(dc::DC_END_LIST)?;
        ensure!(writer.len() == u64::try_from(expected.payload_bytes().len())?);
        ensure!(output.0 == expected.payload_bytes());
        Ok(())
    }
}
