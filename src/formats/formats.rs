// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
pub use ctb_utilities::ipc::service_prelude::*;

use ctb_formats_utilities::FormatLog;
use std::collections::HashMap;
use uuid::Uuid;

pub use ctb_formats_alias as alias;
pub use ctb_formats_applescript as applescript;
pub use ctb_formats_base16b as base16b;
pub use ctb_formats_base64 as base64;
pub use ctb_formats_checksum as checksum;
pub use ctb_formats_compression as compression;
pub use ctb_formats_ctb_asset_bundle as ctb_asset_bundle;
pub use ctb_formats_dceutils as dceutils;
pub use ctb_formats_eite as eite;
pub use ctb_formats_encoding as encoding;
pub use ctb_formats_hexdump as hexdump;
pub use ctb_formats_html as html;
pub use ctb_formats_internetarchive as internetarchive;
pub use ctb_formats_ipaddr as ipaddr;
pub use ctb_formats_javascript as javascript;
pub use ctb_formats_json as json;
pub use ctb_formats_lnk as lnk;
pub use ctb_formats_markdown as markdown;
pub use ctb_formats_math as math;
pub use ctb_formats_multipart as multipart;
pub use ctb_formats_pan as pan;
pub use ctb_formats_pdf as pdf;
pub use ctb_formats_pem as pem;
pub use ctb_formats_perl as perl;
pub use ctb_formats_stagel as stagel;
pub use ctb_formats_troff as troff;
pub use ctb_formats_unicode as unicode;
pub use ctb_formats_uri as uri;
pub use ctb_formats_useragent as useragent;
pub use ctb_formats_utf_8e_128 as utf_8e_128;
pub use ctb_formats_utf8 as utf8;
pub use ctb_formats_warc as warc;
pub use ctb_formats_wfscan as wfscan;
pub use ctb_formats_wtf8 as wtf8;
pub use ctb_formats_x86 as x86;

pub fn string_result_with_log_to_vec(
    result: Result<(String, FormatLog)>,
) -> Result<(Vec<u8>, FormatLog)> {
    result.map(|res| {
        let result_bytes = res.0.into_bytes();
        (result_bytes, res.1)
    })
}

pub fn get_format_uuids<'a>() -> HashMap<Vec<u8>, Vec<u8>> {
    HashMap::from([(
        strtovec("9ba60c52-9cf8-41a7-b3ea-7a1e14f6c5d7"),
        strtovec("html"),
    )])
}

#[expect(
    clippy::expect_used,
    clippy::unwrap_in_result,
    reason = "Slice bounds are guaranteed by preceding len checks"
)]
pub fn get_format_from_uuid(document: Vec<u8>) -> Option<Vec<u8>> {
    let head = if document.len() < 36 {
        document
    } else {
        document
            .get(..36)
            .expect("document length is >= 36")
            .to_vec()
    };
    let uuid_val = get_uuid_from_document(head)?;
    get_format_uuids().get(&uuid_val).cloned()
}

#[expect(
    clippy::expect_used,
    clippy::unwrap_in_result,
    reason = "Slice bounds for 16-byte binary UUID are guaranteed by preceding len >= 16 check"
)]
pub fn get_uuid_from_document(document: Vec<u8>) -> Option<Vec<u8>> {
    if document.len() < 16 {
        return None;
    }

    let uuid_binary = Uuid::from_slice(
        document.get(..16).expect("document length is >= 16"),
    )
    .ok()?
    .hyphenated()
    .to_string()
    .into_bytes();
    // Reason for fallback: document may be under 36 bytes, in which case empty slice safely prevents panic during textual UUID check.
    let uuid_string =
        String::from_utf8_lossy(document.get(..36).unwrap_or(&[]))
            .to_string()
            .into_bytes();

    let formats = get_format_uuids();

    if formats.contains_key(&uuid_binary) {
        return Some(uuid_binary);
    } else if formats.contains_key(&uuid_string) {
        return Some(uuid_string);
    }

    None
}

pub fn convert_if_needed(document: Vec<u8>) -> Vec<u8> {
    // TODO

    document
}

pub fn convert_from(document: Vec<u8>, _filetype: Vec<u8>) -> Vec<u8> {
    // TODO

    document
}

#[cfg(test)]
#[expect(
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

    #[crate::ctb_test]
    fn can_get_uuid_from_document() {
        assert_eq!(
            strtovec("9ba60c52-9cf8-41a7-b3ea-7a1e14f6c5d7"),
            get_uuid_from_document(strtovec(
                "9ba60c52-9cf8-41a7-b3ea-7a1e14f6c5d7<html>"
            ))
            .unwrap()
        );
    }

    #[crate::ctb_test]
    fn can_get_format_from_uuid() {
        assert_eq!(
            strtovec("html"),
            get_format_from_uuid(strtovec(
                "9ba60c52-9cf8-41a7-b3ea-7a1e14f6c5d7<html>"
            ))
            .unwrap()
        );
    }
}
