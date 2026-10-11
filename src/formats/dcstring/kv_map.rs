// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared framing for canonical pair arrays and legacy alternating maps.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::{DcChar, DcMixedReader, DcMstring};
use ctb_formats_dcdata::dc::{
    DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR, DC_BEGIN_LEGACY_KV_MAP, DC_END_KV_MAP,
    DC_END_KV_PAIR, DC_END_LEGACY_KV_MAP,
};

/// Wire framing of a key-value array, determined by its opening Dc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DcKvMapEncoding {
    /// Deprecated Dc 313..314 alternating key/value operands.
    Legacy,
    /// Dc 559..560 containing adjacent Dc 514..515 pairs.
    Pairs,
}

impl DcKvMapEncoding {
    /// Writes the array header with Object key and value types.
    pub fn encode_header(self, target: &mut DcMstring) {
        target.push_char(match self {
            Self::Legacy => DC_BEGIN_LEGACY_KV_MAP,
            Self::Pairs => DC_BEGIN_KV_MAP,
        });
        target.push_char(DcChar::from_short(275));
        target.push_char(DcChar::from_short(275));
    }

    /// Reads either supported Object-typed array header.
    ///
    /// # Errors
    /// Returns an error for an unsupported opener or type declaration.
    pub fn decode_header(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let opener = reader.next_char()?;
        let encoding = if opener == DC_BEGIN_KV_MAP {
            Self::Pairs
        } else if opener == DC_BEGIN_LEGACY_KV_MAP {
            Self::Legacy
        } else {
            bail!("Expected key-value array opener, found {opener:?}");
        };
        reader.expect_short_dc(275)?;
        reader.expect_short_dc(275)?;
        Ok(encoding)
    }

    /// Writes the boundary before an entry.
    pub fn encode_entry_begin(self, target: &mut DcMstring, first: bool) {
        match self {
            Self::Pairs => target.push_char(DC_BEGIN_KV_PAIR),
            Self::Legacy if !first => target.push_char(DcChar::from_short(312)),
            Self::Legacy => {}
        }
    }

    /// Writes the boundary after an entry.
    pub fn encode_entry_end(self, target: &mut DcMstring) {
        if self == Self::Pairs {
            target.push_char(DC_END_KV_PAIR);
        }
    }

    /// Writes the closing array delimiter.
    pub fn encode_end(self, target: &mut DcMstring) {
        target.push_char(self.closer());
    }

    /// Reads an entry boundary or consumes the array end.
    ///
    /// # Errors
    /// Returns an error for missing or mismatched delimiters. The opener fixes
    /// the interpretation; malformed pair arrays never fall back to legacy.
    pub fn decode_entry_begin(
        self,
        reader: &mut DcMixedReader<'_>,
        first: bool,
    ) -> Result<bool> {
        if reader.peek_char()? == Some(self.closer()) {
            reader.next_char()?;
            return Ok(false);
        }
        match self {
            Self::Pairs => reader.expect_char(DC_BEGIN_KV_PAIR)?,
            Self::Legacy if !first => reader.expect_short_dc(312)?,
            Self::Legacy => {}
        }
        Ok(true)
    }

    /// Reads the required end of a framed pair.
    ///
    /// # Errors
    /// Returns an error for a missing Dc 515 in a pair array.
    pub fn decode_entry_end(
        self,
        reader: &mut DcMixedReader<'_>,
    ) -> Result<()> {
        if self == Self::Pairs {
            reader.expect_char(DC_END_KV_PAIR)?;
        }
        Ok(())
    }

    const fn closer(self) -> DcChar {
        match self {
            Self::Legacy => DC_END_LEGACY_KV_MAP,
            Self::Pairs => DC_END_KV_MAP,
        }
    }
}
