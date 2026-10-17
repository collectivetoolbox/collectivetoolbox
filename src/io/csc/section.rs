#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_formats_checksum::Sha256Stream;
use ctb_formats_dcdata::dc::{
    DC_BEGIN_CHECKSUMMED_SECTION, DC_END_CHECKSUMMED_SECTION,
    DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER,
};
use ctb_formats_dcstring::{DcMixedEncode, DcMstring};
use std::io::Write;

struct DigestWriter<'writer, W> {
    writer: &'writer mut W,
    digest: Sha256Stream,
}

impl<W: Write> Write for DigestWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let written = self.writer.write(bytes)?;
        let accepted = bytes.get(..written).ok_or_else(|| {
            std::io::Error::other("Writer returned an invalid byte count")
        })?;
        self.digest.update(accepted);
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

pub(crate) fn write_section<W: Write>(
    writer: &mut W,
    sequence: u64,
    encode: impl FnOnce(&mut dyn Write) -> Result<()>,
) -> Result<()> {
    let mut marker = DcMstring::new();
    marker.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
    writer.write_all(marker.payload_bytes())?;

    let mut digest_writer = DigestWriter { writer, digest: Sha256Stream::new() };
    let mut header = DcMstring::new();
    header.push_char(DC_SEQUENCE_IDENTIFIER);
    sequence.encode_dc_mixed(&mut header)?;
    digest_writer.write_all(header.payload_bytes())?;
    encode(&mut digest_writer)?;
    let digest = digest_writer.digest.finalize();

    let mut footer = DcMstring::new();
    footer.push_char(DC_END_CHECKSUMMED_SECTION);
    footer.push_char(DC_SECTION_DIGEST);
    footer.push_checksum(ctb_utilities::format_id::DC_SHA256, &digest)?;
    writer.write_all(footer.payload_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ShortWriter(Vec<u8>);

    impl Write for ShortWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let accepted = bytes.get(..bytes.len().min(3))
                .ok_or_else(|| std::io::Error::other("Invalid test slice"))?;
            self.0.extend_from_slice(accepted);
            Ok(accepted.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[crate::ctb_test]
    fn section_matches_buffered_encoding_with_short_writes() -> Result<()> {
        let mut body = DcMstring::new();
        body.push_char(DC_SEQUENCE_IDENTIFIER);
        7_u64.encode_dc_mixed(&mut body)?;
        let mut expected = DcMstring::new();
        expected.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        let mut actual = ShortWriter(Vec::new());
        write_section(&mut actual, 7, |_| Ok(()))?;
        let mut bytes = expected.payload_bytes().to_vec();
        bytes.extend_from_slice(body.payload_bytes());
        let mut footer = DcMstring::new();
        footer.push_char(DC_END_CHECKSUMMED_SECTION);
        footer.push_char(DC_SECTION_DIGEST);
        footer.push_checksum(
            ctb_utilities::format_id::DC_SHA256,
            &ctb_formats_checksum::sha256(body.payload_bytes()),
        )?;
        bytes.extend_from_slice(footer.payload_bytes());
        ensure!(actual.0 == bytes, "Streaming section changed journal wire bytes");
        Ok(())
    }

    #[crate::ctb_test]
    fn section_failure_does_not_commit() -> Result<()> {
        let mut bytes = Vec::new();
        let outcome = write_section(&mut bytes, 1, |writer| {
            writer.write_all(b"partial")?;
            bail!("injected payload failure")
        });
        ensure!(outcome.is_err());
        ensure!(bytes.ends_with(b"partial"), "Failed section acquired a commit footer");
        Ok(())
    }
}