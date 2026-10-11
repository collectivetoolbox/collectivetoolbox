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

//! High-level APIs for bidirectional conversions between `DcMstring` and external documents.
//!
//! # Architecture
//!
//! Document conversion in Collective Toolbox is organized around [`DcMstring`] as the
//! universal internal representation. Conversions are bidirectional:
//! - **Import**: Converts external document formats (e.g. UTF-8, DcText, Dcts, Dcal,
//!   DcIntegerList, DcMixed, DcUtf, SEMS, Ascii, CscJournal) into [`DcMstring`].
//! - **Export**: Serializes a [`DcMstring`] into any supported output format
//!   ([`SupportedDcOutputFormat`]).
//!
//! ## In-Memory Conversions
//!
//! For whole documents in memory or files on disk:
//! - [`convert_bytes_to_dcmstring`]: Parses raw bytes with optional format detection.
//! - [`convert_file_to_dcmstring`]: Reads a file from disk (or stdin) into a [`DcMstring`].
//! - [`convert_cscjournal_to_dcmstring`]: Converts legacy and v4+ cscjournal files to a [`DcMstring`].
//! - [`convert_dcmstring_to_bytes`]: Serializes [`DcMstring`] to target format bytes.
//! - [`convert_dcmstring_to_file`]: Writes [`DcMstring`] to a target format file or stdout.
//!
//! ## Streaming Conversions
//!
//! For bounded-memory processing of large documents and standard Unix pipelines:
//! - [`stream_reader_to_dcmwriter`]: Streams an input source into framed `.dcmm` bytes.
//! - [`stream_file_to_dcmwriter`]: Streams a file on disk into framed `.dcmm` bytes.
//! - [`stream_dcmreader_to_writer`]: Streams framed `.dcmm` bytes into a target format.
//! - [`stream_dcmfile_to_writer`]: Streams a `.dcmm` file into a target format writer.
//!
//! ## Format Detection
//!
//! When [`FormatSpec::Auto`] is specified, format detection is delegated directly to
//! the authoritative format detection engine in [`ctb_formats_detection`].
//!
//! ## Diagnostics and Logs
//!
//! All conversions return a [`ctb_formats_utilities::FormatLog`] containing warnings,
//! errors, and diagnostic notices generated during conversion.

pub use crate::convert_bytes::{
    DcConversionOptions, DcConversionResult, DcExportOptions, DcExportResult,
    FormatSpec, convert_bytes_to_dcmstring, convert_cscjournal_to_dcmstring,
    convert_dcmstring_to_bytes, convert_dcmstring_to_file, convert_file_to_dcmstring,
};
pub use crate::convert_stream::{
    StreamConversionReceipt, StreamExportReceipt, stream_dcmfile_to_writer,
    stream_dcmreader_to_writer, stream_file_to_dcmwriter,
    stream_reader_to_dcmwriter,
};

use ctb_utilities::format_support::{
    SupportedDcInputFormat, SupportedDcOutputFormat,
};
use ctb_utilities::FormatId;

/// Returns the ordered list of all supported input formats for conversion to `DcMstring`.
#[must_use]
pub fn list_supported_input_formats() -> &'static [FormatId] {
    SupportedDcInputFormat::ALL_FORMAT_IDS
}

/// Returns the ordered list of all supported output formats for conversion from `DcMstring`.
#[must_use]
pub fn list_supported_output_formats() -> &'static [FormatId] {
    SupportedDcOutputFormat::ALL_FORMAT_IDS
}
