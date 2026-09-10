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

//! Document String (`DcString`, `DcStr`), Document Character (`DcChar`), and
//! related Document Text formats (`DcText`, `DcList`, `DcUtf`).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub use ctb_formats_utf_8e_128::{
    decode_utf_8e_128, decode_utf_8e_128_buf, encode_utf_8e_128,
    encode_utf_8e_128_buf,
};
pub use ctb_formats_dcdata as ctb_formats_dc_data;
pub use crate as ctb_formats_dctext;
pub use ctb_formats_dcdata::dc::{
    GID_ESCAPE, GID_LONG_DC, SHORT_DC_ESCAPE, SHORT_DC_LONG_DC,
    SHORT_DC_REGION_END, SHORT_DC_REGION_START,
};
pub use ctb_formats_utilities::ConversionOutput;

pub mod character_description;
pub mod cli;
pub mod cli_identifiers;
pub mod converters;
pub mod dc_char;
pub mod dc_number;
pub mod dc_str;
pub mod dc_string;
pub mod error;

pub use dc_char::DcChar;
pub use dc_str::{DcCharIndices, DcChars, DcStr, validate_dcutf};
pub use dc_string::DcString;
pub use error::DcUtfError;

pub use converters::{
    DcList, dcarray_to_dclist, dcarray_to_dctext, dcal, dcal_to_dclist,
    dclist_to_dcal, dclist_to_dcarray, dclist_to_dctext, dclist_to_dcutf,
    dcstring_to_dctext, dctext, dctext_to_dcarray, dctext_to_dclist,
    dctext_to_dcstring, dctext_to_dcutf, dcutf_to_dclist, dcutf_to_dctext,
    format_blob_preview, utf8,
};

pub use dc_number::{
    GID_BASE64_END, GID_BASE64_PADDING, GID_BASE64_START, GID_BEGIN_NUMBER,
    GID_END_NUMBER, GID_FORMAT_199, GID_NEGATIVE, GID_POSITIVE,
    SHORT_DC_BASE64_END, SHORT_DC_BASE64_PADDING, SHORT_DC_BASE64_START,
    SHORT_DC_BEGIN_NUMBER, SHORT_DC_END_NUMBER, SHORT_DC_NEGATIVE,
    SHORT_DC_POSITIVE, SHORT_ID_FORMAT_199, base64_char_to_global_dc,
    base64_char_to_short_dc, base64_str_to_global_dcs, base64_str_to_short_dcs,
    global_dc_to_base64_char, global_dcs_to_base64_str,
    i128_to_dc_number_global, i128_to_dc_number_short,
    integer_to_dc_number_global, integer_to_dc_number_short,
    natural_to_dc_number_global, natural_to_dc_number_short,
    parse_dc_number_global, parse_dc_number_global_i128, parse_dc_number_short,
    parse_dc_number_short_i128, read_dc_number_global, read_dc_number_short,
    short_dc_to_base64_char, short_dcs_to_base64_str, u128_to_dc_number_global,
    u128_to_dc_number_short,
};

pub use character_description::{
    describe_dcal, describe_dclist, describe_graph_id,
};
pub use cli::{
    CharacterDescriptionArgs, CharacterDescriptionInputFormat,
    execute_cli_character_description,
};
pub use cli_identifiers::{
    GidArgs, ShortDcArgs, ShortFmtArgs, execute_cli_gid, execute_cli_short_dc,
    execute_cli_short_fmt, parse_graph_or_short_id,
};
pub use converters::utf8::{
    DcListUtf8Settings, dclist_from_utf8, dclist_to_utf8, utf8_to_dclist,
};
