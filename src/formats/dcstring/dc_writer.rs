#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::*;

use std::io::Write;

/// Supplies nonresident binary data without materializing it in memory.
pub trait DcBinarySource {
    fn open<'source>(&'source self, payload: &BinaryPayloadRef<'_>) -> Result<DcBinaryReader<'source>>;
}

pub struct DcBinaryReader<'source> {
    pub reader: Box<dyn std::io::Read + 'source>,
    pub sha256: Option<[u8; 32]>,
}

/// A headerless DcMM encoder writing directly to a fallible output stream.
pub struct DcWriter<'output> {
    output: &'output mut dyn Write,
    bytes: u64,
    source: Option<&'output dyn DcBinarySource>,
}

impl<'output> DcWriter<'output> {
    pub fn new(output: &'output mut dyn Write) -> Self {
        Self { output, bytes: 0, source: None }
    }

    pub fn with_binary_source(output: &'output mut dyn Write, source: &'output dyn DcBinarySource) -> Self {
        Self { output, bytes: 0, source: Some(source) }
    }

    pub fn push_binary_ref(&mut self, payload: &BinaryPayloadRef<'_>) -> Result<()> {
        if let Some(bytes) = payload.as_resident_bytes() {
            return self.push_binary_with_sha256(bytes);
        }
        let source = self.source.context("Lazy binary encoding requires a backing source")?;
        let mut opened = source.open(payload)?;
        self.push_binary_from_reader(&mut opened.reader, payload.len(), opened.sha256)?;
        let mut extra = [0_u8; 1];
        ensure!(opened.reader.read(&mut extra)? == 0, "Binary source exceeds its declared length");
        Ok(())
    }

    pub const fn len(&self) -> u64 {
        self.bytes
    }

    pub const fn is_empty(&self) -> bool {
        self.bytes == 0
    }

    pub fn push_char(&mut self, character: impl Into<DcChar>) -> Result<()> {
        let mut buffer = [0_u8; 24];
        let length = ctb_formats_utf_8e_128::encode_utf_8e_128_buf(&mut buffer, character.into().0);
        self.write_all(buffer.get(..length).context("Invalid encoded Dc character length")?)?;
        Ok(())
    }

    pub fn push_str(&mut self, text: &str) -> Result<()> {
        self.write_all(text.as_bytes())?;
        Ok(())
    }

    pub fn push_dc_str(&mut self, text: &DcStr) -> Result<()> {
        self.write_all(text.as_bytes())?;
        Ok(())
    }

    pub fn push_binary(&mut self, bytes: &[u8]) -> Result<()> {
        self.push_binary_with_metadata(bytes, &DcString::new())
    }

    pub fn push_binary_with_metadata(&mut self, bytes: &[u8], metadata: &DcStr) -> Result<()> {
        crate::encapsulation::validate_metadata(metadata, bytes)?;
        self.write_all(&crate::dc_mixed::DC_203_BYTES)?;
        self.write_all(&[crate::dc_mixed::TYPE_BINARY_METADATA])?;
        self.write_all(&u128::try_from(bytes.len())?.to_be_bytes())?;
        self.write_all(&u128::try_from(metadata.len())?.to_be_bytes())?;
        self.write_all(metadata.as_bytes())?;
        self.write_all(bytes)?;
        Ok(())
    }

    pub fn push_binary_with_sha256(&mut self, bytes: &[u8]) -> Result<()> {
        let digest = ctb_formats_checksum::sha256(bytes);
        let metadata = crate::encapsulation::checksum_text(ctb_utilities::format_id::DC_SHA256, &digest)?;
        self.push_binary_with_metadata(bytes, &metadata)
    }

    pub fn push_checksum(&mut self, format: impl Into<DcChar>, digest: &[u8]) -> Result<()> {
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
        DcMstring::write_binary_encapsulation(self, reader, size, sha256)
    }
}

impl Write for DcWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let count = self.output.write(bytes)?;
        if count > bytes.len() {
            return Err(std::io::Error::other("Writer returned an invalid byte count"));
        }
        self.bytes = self.bytes.checked_add(u64::try_from(count).map_err(std::io::Error::other)?)
            .ok_or_else(|| std::io::Error::other("Dc output size overflow"))?;
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, crate::DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct DerivedPayload {
        #[dc(392, default, omit_default)]
        body: Option<LazyBinaryPayload>,
    }

    struct PayloadSource(Vec<u8>);

    impl DcBinarySource for PayloadSource {
        fn open<'source>(&'source self, payload: &BinaryPayloadRef<'_>) -> Result<DcBinaryReader<'source>> {
            ensure!(matches!(payload, BinaryPayloadRef::Source { source_id: 0, .. }));
            Ok(DcBinaryReader {
                reader: Box::new(self.0.as_slice()),
                sha256: Some(ctb_formats_checksum::sha256(&self.0)),
            })
        }
    }

    #[crate::ctb_test]
    fn derived_lazy_payload_matches_resident_encoding() -> Result<()> {
        let source = PayloadSource(vec![42; 128 * 1024]);
        let resident = DerivedPayload { body: Some(LazyBinaryPayload(BinaryPayloadRef::Inline(source.0.clone()))) };
        let lazy = DerivedPayload { body: Some(LazyBinaryPayload(BinaryPayloadRef::Source { source_id: 0, len: u64::try_from(source.0.len())? })) };
        let expected = resident.to_dc_mixed()?;
        let mut output = ShortWriter(Vec::new());
        lazy.encode_dc_mixed_to(&mut DcWriter::with_binary_source(&mut output, &source))?;
        ensure!(output.0 == expected.payload_bytes());
        ensure!(lazy.to_dc_mixed().is_err());
        Ok(())
    }

    struct ShortWriter(Vec<u8>);

    impl Write for ShortWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let count = bytes.len().min(3);
            self.0.extend_from_slice(bytes.get(..count).ok_or_else(|| std::io::Error::other("Invalid test slice"))?);
            Ok(count)
        }

        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
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