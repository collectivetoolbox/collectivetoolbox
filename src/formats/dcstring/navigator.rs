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

//! Database-syntax-driven hierarchical navigator, reusable syntax arena, and
//! projection policies for Document Character (`Dc`) streams and files.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ctb_formats_dcdata::dc::{
    DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_FORMAT_REPRESENTATION,
    DC_BEGIN_NUMBER, DC_END_ENCAPSULATION_BINARY, DC_END_NUMBER, DC_ESCAPE,
    DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER, DC_START_ENCAPSULATION_BINARY,
};
use ctb_formats_dcdata::syntax::{
    CharTarget, Quantifier, SyntaxDiagnostic, SyntaxElement, SyntaxPattern,
    SyntaxRuleResolver, SyntaxTerm, default_syntax_resolver,
};
use ctb_formats_utf_8e_128::{decode_utf_8e_128_buf, encode_utf_8e_128_buf};
use sha2::{Digest, Sha256};

use crate::{
    BinaryPayload, DCMM_HEADER_LEN, DCMM_MAGIC, DCMM_UUID, DcChar,
    DcMixedDecode, DcMixedReader, DcMstr, DcMstring, DcStr, DcString, Integer,
    Natural, SharedValueContext,
};

/// Maximum binary payload size (in bytes) eagerly loaded into memory when
/// navigating or expanding frames from a seekable on-disk `.dcmm` file.
/// Larger binary regions are represented as [`BinaryPayloadRef::FileSpan`].
pub const INLINE_BINARY_THRESHOLD: u64 = 4096;

/// Action to take when a syntax or projection anomaly is encountered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticAction {
    /// Immediately abort parsing or projection and return an `Err`.
    Error,
    /// Emit a warning via `warn_fmt!` and continue with recovery/fallback.
    Warn,
    /// Emit an informational log via `log_fmt!` and continue with recovery/fallback.
    Log,
    /// Continue silently with recovery/fallback (diagnostics are still recorded in the arena/inspector).
    Ignore,
}

/// Caller-specified policy governing how syntax errors, unknown tags,
/// deprecated constructs, and malformed or missing fields are handled during
/// Dc document parsing and struct projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcProjectionPolicy {
    /// Framing or syntax rule violation in Stage 1 (e.g. unclosed frame, bad literal header).
    pub syntax_error: DiagnosticAction,
    /// Unknown Dc tag encountered inside a container (e.g. forward-compatible field or statement).
    pub unknown_field: DiagnosticAction,
    /// Deprecated syntax or legacy representation (e.g. `<prefer>` replacement).
    pub deprecated_syntax: DiagnosticAction,
    /// Type mismatch or domain error when projecting a known field onto its Rust type.
    pub malformed_field: DiagnosticAction,
    /// Missing required field when projecting a struct (uses [`DcForgivingDefault`] if non-`Error`).
    pub missing_required_field: DiagnosticAction,
}

impl DcProjectionPolicy {
    /// Strict policy: errors on any syntax violation, unknown field tag,
    /// malformed field, or missing required field; warns on deprecated syntax.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            syntax_error: DiagnosticAction::Error,
            unknown_field: DiagnosticAction::Error,
            deprecated_syntax: DiagnosticAction::Warn,
            malformed_field: DiagnosticAction::Error,
            missing_required_field: DiagnosticAction::Error,
        }
    }

    /// Forgiving / best-effort policy: logs warnings and recovers at frame or
    /// field boundaries, projecting all valid fields and substituting fallback
    /// values for missing or damaged fields.
    #[must_use]
    pub const fn forgiving() -> Self {
        Self {
            syntax_error: DiagnosticAction::Warn,
            unknown_field: DiagnosticAction::Warn,
            deprecated_syntax: DiagnosticAction::Warn,
            malformed_field: DiagnosticAction::Warn,
            missing_required_field: DiagnosticAction::Warn,
        }
    }

    /// Quiet forgiving policy for interactive source inspection (`DcSource::inspect`),
    /// recording diagnostics without emitting warning logs.
    #[must_use]
    pub const fn forgiving_quiet() -> Self {
        Self {
            syntax_error: DiagnosticAction::Ignore,
            unknown_field: DiagnosticAction::Ignore,
            deprecated_syntax: DiagnosticAction::Ignore,
            malformed_field: DiagnosticAction::Ignore,
            missing_required_field: DiagnosticAction::Ignore,
        }
    }

    /// Returns true if all structural and projection anomalies are set to `Error`.
    #[must_use]
    pub const fn is_strict(&self) -> bool {
        matches!(self.syntax_error, DiagnosticAction::Error)
            && matches!(self.unknown_field, DiagnosticAction::Error)
            && matches!(self.malformed_field, DiagnosticAction::Error)
            && matches!(self.missing_required_field, DiagnosticAction::Error)
    }

    /// Applies the configured [`DiagnosticAction`] for an anomaly message.
    ///
    /// # Errors
    /// Returns an error if `action` is [`DiagnosticAction::Error`].
    pub fn handle(
        &self,
        action: DiagnosticAction,
        message: impl Into<String>,
    ) -> Result<()> {
        let msg = message.into();
        match action {
            DiagnosticAction::Error => bail!("{msg}"),
            DiagnosticAction::Warn => {
                warn_fmt!("{msg}");
                Ok(())
            }
            DiagnosticAction::Log => {
                log_fmt!("{msg}");
                Ok(())
            }
            DiagnosticAction::Ignore => Ok(()),
        }
    }
}

/// Fallback value provider used when projecting structs or fields under a
/// non-`Error` [`DcProjectionPolicy`] (`forgiving()`).
pub trait DcForgivingDefault: Sized {
    /// Produces a placeholder value when a required field or element is missing
    /// or malformed in forgiving mode.
    fn dc_forgiving_default() -> Self;
}

macro_rules! impl_forgiving_default_via_default {
    ($($t:ty),*) => {
        $(
            impl DcForgivingDefault for $t {
                fn dc_forgiving_default() -> Self {
                    Self::default()
                }
            }
        )*
    };
}

impl_forgiving_default_via_default!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    bool,
    String,
    PathBuf,
    Natural,
    Integer,
    BinaryPayload,
    DcMstring,
    DcString
);

impl DcForgivingDefault for Ipv4Addr {
    fn dc_forgiving_default() -> Self {
        Self::UNSPECIFIED
    }
}

impl DcForgivingDefault for Ipv6Addr {
    fn dc_forgiving_default() -> Self {
        Self::UNSPECIFIED
    }
}

impl DcForgivingDefault for IpAddr {
    fn dc_forgiving_default() -> Self {
        Self::V4(Ipv4Addr::UNSPECIFIED)
    }
}

impl DcForgivingDefault for SystemTime {
    fn dc_forgiving_default() -> Self {
        UNIX_EPOCH
    }
}

impl DcForgivingDefault for Duration {
    fn dc_forgiving_default() -> Self {
        Self::ZERO
    }
}

impl DcForgivingDefault for FormatId {
    fn dc_forgiving_default() -> Self {
        Self::Utf8
    }
}

impl<T> DcForgivingDefault for Option<T> {
    fn dc_forgiving_default() -> Self {
        None
    }
}

impl<T> DcForgivingDefault for Vec<T> {
    fn dc_forgiving_default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> DcForgivingDefault for BTreeMap<K, V> {
    fn dc_forgiving_default() -> Self {
        Self::new()
    }
}

impl<T: DcForgivingDefault> DcForgivingDefault for Box<T> {
    fn dc_forgiving_default() -> Self {
        Self::new(T::dc_forgiving_default())
    }
}

impl<T: DcForgivingDefault> DcForgivingDefault for Arc<T> {
    fn dc_forgiving_default() -> Self {
        Self::new(T::dc_forgiving_default())
    }
}

impl<T: DcForgivingDefault, const N: usize> DcForgivingDefault for [T; N] {
    fn dc_forgiving_default() -> Self {
        std::array::from_fn(|_| T::dc_forgiving_default())
    }
}

/// Byte span in a source buffer or on-disk file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DcByteSpan {
    /// Inclusive start byte offset.
    pub start: u64,
    /// Exclusive end byte offset.
    pub end: u64,
}

impl DcByteSpan {
    /// Creates a new byte span.
    #[must_use]
    pub const fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    /// Returns the length in bytes.
    #[must_use]
    pub const fn len(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the span is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Converts to a `Range<usize>` for indexing in-memory buffers.
    pub fn to_usize_range(self) -> Result<Range<usize>> {
        Ok(usize::try_from(self.start)?..usize::try_from(self.end)?)
    }
}

/// Reference to a binary payload inside a parsed Dc document.
///
/// For in-memory `.dcmm` buffers, payloads are borrowed zero-copy slices.
/// For seekable on-disk `.dcmm` files, large payloads remain lazy file offsets
/// (`FileSpan`) so multi-gigabyte binary regions are not read into RAM during
/// parsing or tree navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryPayloadRef<'a> {
    /// Zero-copy slice borrowed from an in-memory `.dcmm` buffer.
    Borrowed(&'a [u8]),
    /// Decoded bytes from base64 `DcStr` or small inline payload ($\le 4\text{ KiB}$).
    Inline(Vec<u8>),
    /// Lazy file byte span inside an on-disk `.dcmm` file.
    FileSpan {
        /// Byte offset of the raw payload within the file.
        file_offset: u64,
        /// Length of the raw payload in bytes.
        len: u64,
    },
}

impl<'a> BinaryPayloadRef<'a> {
    /// Returns the payload byte length.
    #[must_use]
    pub fn len(&self) -> u64 {
        match self {
            Self::Borrowed(slice) => u64::try_from(slice.len()).unwrap_or(0),
            Self::Inline(bytes) => u64::try_from(bytes.len()).unwrap_or(0),
            Self::FileSpan { len, .. } => *len,
        }
    }

    /// Returns true if the payload has zero bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the in-memory bytes if already resident (`Borrowed` or `Inline`).
    #[must_use]
    pub fn as_resident_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Borrowed(slice) => Some(slice),
            Self::Inline(bytes) => Some(bytes.as_slice()),
            Self::FileSpan { .. } => None,
        }
    }

    /// Materializes the payload bytes, reading from `reader` if this is a `FileSpan`.
    pub fn materialize<R: Read + Seek>(
        &self,
        mut reader: Option<&mut R>,
    ) -> Result<Cow<'a, [u8]>> {
        match self {
            Self::Borrowed(slice) => Ok(Cow::Borrowed(slice)),
            Self::Inline(bytes) => Ok(Cow::Owned(bytes.clone())),
            Self::FileSpan { file_offset, len } => {
                let rdr = reader.as_deref_mut().context(
                    "Cannot materialize on-disk FileSpan without a seekable reader",
                )?;
                let len_usize = usize::try_from(*len)
                    .context("Binary FileSpan length exceeds usize")?;
                let saved_pos = rdr.stream_position()?;
                rdr.seek(SeekFrom::Start(*file_offset))?;
                let mut buf = vec![0_u8; len_usize];
                let read_res = rdr.read_exact(&mut buf);
                let _ = rdr.seek(SeekFrom::Start(saved_pos));
                read_res.context("Failed to read BinaryPayloadRef::FileSpan")?;
                Ok(Cow::Owned(buf))
            }
        }
    }
}

/// Index of a node inside a [`DcSyntaxArena`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Syntax node kind in a [`DcSyntaxArena`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DcSyntaxKind<'a> {
    /// Delimited container (`opener .. closer`, with optional header and trailing suffix).
    Container {
        opener: DcChar,
        closer: DcChar,
        header: Vec<NodeId>,
        children: Vec<NodeId>,
        trailing: Vec<NodeId>,
        unclosed: bool,
    },
    /// Prefix-tagged construct (`:~ <arg1> <arg2> ...`).
    Tagged {
        tag: DcChar,
        args: Vec<NodeId>,
    },
    /// Assignment statement (`[identifier] 269 [value]`).
    Assignment {
        ident: String,
        value: NodeId,
    },
    /// Framed literal (`260 [262:]{1} <payload> 261`) with `255` unescaped.
    Literal {
        type_header: Vec<DcChar>,
        decoded: String,
    },
    /// Framed identifier (`270 <payload> 271`) with `255` unescaped.
    Identifier {
        name: String,
    },
    /// Framed number (`6 [format:number] [^7]+ 7`).
    Number {
        format: DcChar,
        digits: Vec<DcChar>,
    },
    /// Binary encapsulation (`203 [type]? [number]? [checksum]? ... 204`).
    Binary {
        data_type: Option<DcString>,
        checksum_format: Option<DcChar>,
        payload: BinaryPayloadRef<'a>,
    },
    /// Standalone Document Character token (e.g. boolean `517`/`518`, `316`, `513`, enum tag).
    Atom(DcChar),
    /// Malformed or garbled span isolated during resilient recovery.
    Malformed {
        raw_tokens: Vec<DcChar>,
        message: String,
    },
}

/// A span-annotated CST node inside a [`DcSyntaxArena`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcSyntaxNode<'a> {
    /// Source byte span of this node.
    pub span: DcByteSpan,
    /// Syntactic kind and payload.
    pub kind: DcSyntaxKind<'a>,
}

/// Flat, reusable arena holding parsed [`DcSyntaxNode`]s for a frame or document.
///
/// Calling [`DcSyntaxArena::clear`] resets the arena between records while
/// retaining allocated capacity, achieving $O(1)$ steady-state memory usage
/// when streaming across multi-gigabyte documents or journals.
#[derive(Debug, Clone, Default)]
pub struct DcSyntaxArena<'a> {
    nodes: Vec<DcSyntaxNode<'a>>,
    roots: Vec<NodeId>,
    diagnostics: Vec<SyntaxDiagnostic>,
}

impl<'a> DcSyntaxArena<'a> {
    /// Creates an empty syntax arena.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the arena for reuse without releasing heap capacity.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.roots.clear();
        self.diagnostics.clear();
    }

    /// Allocates a node in the arena and returns its [`NodeId`].
    pub fn alloc(&mut self, span: DcByteSpan, kind: DcSyntaxKind<'a>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(DcSyntaxNode { span, kind });
        id
    }

    /// Appends a root node ID.
    pub fn push_root(&mut self, id: NodeId) {
        self.roots.push(id);
    }

    /// Returns the top-level root nodes in the arena.
    #[must_use]
    pub fn roots(&self) -> &[NodeId] {
        &self.roots
    }

    /// Borrows a node by its [`NodeId`].
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&DcSyntaxNode<'a>> {
        self.nodes.get(id.0)
    }

    /// Returns all nodes in the arena.
    #[must_use]
    pub fn nodes(&self) -> &[DcSyntaxNode<'a>] {
        &self.nodes
    }

    /// Records a syntax diagnostic encountered while parsing.
    pub fn push_diagnostic(&mut self, diagnostic: SyntaxDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Returns all diagnostics recorded in this arena.
    #[must_use]
    pub fn diagnostics(&self) -> &[SyntaxDiagnostic] {
        &self.diagnostics
    }
}

/// Lightweight header summary attached to a [`DcFrameStub`] during shallow
/// level navigation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DcFrameHeaderInfo {
    /// Standard frame or statement without specialized header metadata.
    #[default]
    None,
    /// Binary encapsulation (`203..204`) with payload byte span and optional data type.
    Binary {
        data_type: Option<DcString>,
        payload_span: Option<DcByteSpan>,
        payload_len: u64,
    },
    /// Checksummed document section (`543..544` + `545` digest).
    ChecksummedSection {
        /// Optional leading `546` sequence identifier inside the section.
        sequence_id: Option<u64>,
        /// Byte span of the `543..544` body covered by the SHA-256 digest.
        body_span: DcByteSpan,
        /// True if the trailing `545` SHA-256 digest matched the body bytes.
        digest_valid: bool,
    },
    /// Format representation (`511 [format] ... 512`).
    FormatRepresentation {
        format: DcChar,
    },
}

/// Lightweight descriptor for a single sibling frame or statement at the
/// navigator's current tree depth, produced without allocating child AST nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcFrameStub {
    /// Leading Document Character of this frame or statement.
    pub head_dc: DcChar,
    /// Matching closing delimiter if this frame is a container.
    pub closer_dc: Option<DcChar>,
    /// Full byte span of the frame (including opener, closer, and any trailing suffix such as `545`).
    pub outer_span: DcByteSpan,
    /// Interior byte span (between opener and closer) that `enter_frame` drills into.
    pub inner_span: Option<DcByteSpan>,
    /// Lightweight header metadata (e.g., checksummed section verification or binary span).
    pub header_info: DcFrameHeaderInfo,
}

enum NavBackend<'a> {
    MemoryMstr(&'a DcMstr),
    MemoryText(&'a DcStr),
    SeekableFile {
        reader: BufReader<File>,
        base_offset: u64,
        end_offset: u64,
    },
}

/// Generalized hierarchical navigator and seeker over `.dcmm` files, borrowed
/// [`DcMstr`] slices, or [`DcStr`] buffers.
///
/// Supports:
/// - Shallow sibling stepping at the current tree level ([`Self::next_sibling_stub`])
/// - Drilling into and out of container levels ([`Self::enter_frame`], [`Self::exit_frame`])
/// - Random-access seeking to bookmarked stubs or offsets ([`Self::seek_to_stub`], [`Self::seek_to_offset`])
/// - On-demand CST expansion into a reusable [`DcSyntaxArena`] ([`Self::expand_frame`])
/// - Direct typed projection of a frame stub ([`Self::decode_frame`])
pub struct DcDocumentNavigator<'a> {
    backend: NavBackend<'a>,
    cursor: u64,
    active_span: DcByteSpan,
    level_stack: Vec<(DcByteSpan, u64)>,
}

impl<'a> DcDocumentNavigator<'a> {
    /// Creates a navigator over an in-memory [`DcMstr`].
    #[must_use]
    pub fn from_mstr(mstr: &'a DcMstr) -> Self {
        let start = u64::try_from(DCMM_HEADER_LEN).unwrap_or(10);
        let end = u64::try_from(mstr.len()).unwrap_or(start);
        Self {
            backend: NavBackend::MemoryMstr(mstr),
            cursor: start,
            active_span: DcByteSpan::new(start, end),
            level_stack: Vec::new(),
        }
    }

    /// Creates a navigator over an in-memory [`DcStr`].
    #[must_use]
    pub fn from_dc_str(text: &'a DcStr) -> Self {
        let end = u64::try_from(text.len()).unwrap_or(0);
        Self {
            backend: NavBackend::MemoryText(text),
            cursor: 0,
            active_span: DcByteSpan::new(0, end),
            level_stack: Vec::new(),
        }
    }

    /// Creates a streaming and seeking navigator over an on-disk `.dcmm` file
    /// starting at `base_offset` (e.g. `0` for a standalone `.dcmm` file, or `8`
    /// for a `.cscjournal` after its 8-byte journal magic).
    ///
    /// Validates the 10-byte `.dcmm` magic/UUID header once at `base_offset`
    /// without reading the rest of the file into memory.
    pub fn from_dcmm_file(mut file: File, base_offset: u64) -> Result<Self> {
        let file_len = file
            .metadata()
            .context("Failed to stat .dcmm file")?
            .len();
        let header_end = base_offset
            .checked_add(u64::try_from(DCMM_HEADER_LEN)?)
            .context("dcmm base_offset overflow")?;
        ensure!(
            file_len >= header_end,
            "File is too short ({file_len} bytes) to contain a valid 10-byte DcMixed header at offset {base_offset}"
        );
        file.seek(SeekFrom::Start(base_offset))
            .context("Failed to seek to DcMixed header")?;
        let mut header = [0_u8; DCMM_HEADER_LEN];
        file.read_exact(&mut header)
            .context("Failed to read DcMixed header")?;
        ensure!(
            header.get(..5) == Some(&DCMM_MAGIC)
                && header.get(5..21) == Some(&DCMM_UUID),
            "Invalid DcMixed (.dcmm) header magic or UUID"
        );
        let reader = BufReader::with_capacity(64 * 1024, file);
        Ok(Self {
            backend: NavBackend::SeekableFile {
                reader,
                base_offset,
                end_offset: file_len,
            },
            cursor: header_end,
            active_span: DcByteSpan::new(header_end, file_len),
            level_stack: Vec::new(),
        })
    }

    /// Returns the current byte offset of the navigator cursor.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.cursor
    }

    /// Returns the active level's byte span bounds.
    #[must_use]
    pub const fn active_span(&self) -> DcByteSpan {
        self.active_span
    }

    /// Returns the current nesting depth (0 at document root).
    #[must_use]
    pub fn depth(&self) -> usize {
        self.level_stack.len()
    }

    /// Seeks the cursor to an arbitrary byte offset within the active level span.
    pub fn seek_to_offset(&mut self, offset: u64) -> Result<()> {
        ensure!(
            offset >= self.active_span.start && offset <= self.active_span.end,
            "Seek offset {offset} is outside active level span {:?}",
            self.active_span
        );
        self.cursor = offset;
        if let NavBackend::SeekableFile { reader, .. } = &mut self.backend {
            reader.seek(SeekFrom::Start(offset))?;
        }
        Ok(())
    }

    /// Seeks the cursor to the start of a previously observed [`DcFrameStub`].
    pub fn seek_to_stub(&mut self, stub: &DcFrameStub) -> Result<()> {
        self.seek_to_offset(stub.outer_span.start)
    }

    /// Enters the interior of a container [`DcFrameStub`], restricting subsequent
    /// [`Self::next_sibling_stub`] calls to the container's children until
    /// [`Self::exit_frame`] is called.
    pub fn enter_frame(&mut self, stub: &DcFrameStub) -> Result<()> {
        let inner = stub
            .inner_span
            .context("Cannot enter a non-container DcFrameStub")?;
        self.level_stack.push((self.active_span, self.cursor));
        self.active_span = inner;
        self.cursor = inner.start;
        if let NavBackend::SeekableFile { reader, .. } = &mut self.backend {
            reader.seek(SeekFrom::Start(inner.start))?;
        }
        Ok(())
    }

    /// Exits the current container level and restores the enclosing level's bounds,
    /// positioning the cursor at the end of the exited container.
    pub fn exit_frame(&mut self) -> Result<()> {
        let prev_end = self.active_span.end;
        let (parent_span, _saved_cursor) = self
            .level_stack
            .pop()
            .context("Already at root level of DcDocumentNavigator")?;
        self.active_span = parent_span;
        let next_pos = prev_end.min(parent_span.end);
        self.seek_to_offset(next_pos)
    }

    /// Reads a raw byte range from the underlying source into a `Vec<u8>`.
    pub fn read_span_bytes(&mut self, span: DcByteSpan) -> Result<Cow<'a, [u8]>> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let range = span.to_usize_range()?;
                let slice = mstr
                    .as_bytes()
                    .get(range)
                    .context("Span out of bounds in DcMstr")?;
                Ok(Cow::Borrowed(slice))
            }
            NavBackend::MemoryText(text) => {
                let range = span.to_usize_range()?;
                let slice = text
                    .as_bytes()
                    .get(range)
                    .context("Span out of bounds in DcStr")?;
                Ok(Cow::Borrowed(slice))
            }
            NavBackend::SeekableFile { reader, .. } => {
                let len = usize::try_from(span.len())
                    .context("Span length exceeds usize")?;
                reader.seek(SeekFrom::Start(span.start))?;
                let mut buf = vec![0_u8; len];
                reader
                    .read_exact(&mut buf)
                    .context("Failed to read span bytes from file")?;
                reader.seek(SeekFrom::Start(self.cursor))?;
                Ok(Cow::Owned(buf))
            }
        }
    }

    /// Computes the SHA-256 digest of a byte span, streaming in 64 KiB chunks
    /// when backed by a file so large sections do not allocate in memory.
    pub fn sha256_of_span(&mut self, span: DcByteSpan) -> Result<[u8; 32]> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let range = span.to_usize_range()?;
                let slice = mstr
                    .as_bytes()
                    .get(range)
                    .context("Span out of bounds for SHA-256")?;
                Ok(ctb_formats_checksum::sha256(slice))
            }
            NavBackend::MemoryText(text) => {
                let range = span.to_usize_range()?;
                let slice = text
                    .as_bytes()
                    .get(range)
                    .context("Span out of bounds for SHA-256")?;
                Ok(ctb_formats_checksum::sha256(slice))
            }
            NavBackend::SeekableFile { reader, .. } => {
                reader.seek(SeekFrom::Start(span.start))?;
                let mut hasher = Sha256::new();
                let mut remaining = span.len();
                let mut buf = [0_u8; 65536];
                while remaining > 0 {
                    let to_read = usize::try_from(remaining.min(65536))?;
                    let slice = buf
                        .get_mut(..to_read)
                        .context("Buffer slice bounds invalid")?;
                    reader.read_exact(slice)?;
                    hasher.update(&*slice);
                    remaining = remaining.saturating_sub(u64::try_from(to_read)?);
                }
                reader.seek(SeekFrom::Start(self.cursor))?;
                Ok(hasher.finalize().into())
            }
        }
    }

    /// Steps across the next sibling statement or container at the current level
    /// without allocating an AST for its children.
    ///
    /// When backed by a seekable file, binary regions (`203..204`) are skipped in
    /// $O(1)$ via `SeekFrom::Current`, and checksummed sections (`543..544` + `545`)
    /// are verified incrementally.
    pub fn next_sibling_stub(
        &mut self,
        policy: &DcProjectionPolicy,
    ) -> Result<Option<DcFrameStub>> {
        if self.cursor >= self.active_span.end {
            return Ok(None);
        }
        let start_offset = self.cursor;
        let Some((first_dc, after_first)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(None);
        };

        // 1. Check if this is a raw or textual binary encapsulation (Dc 203)
        if first_dc == DC_START_ENCAPSULATION_BINARY {
            let (end_offset, header_info) = self.scan_binary_at(start_offset)?;
            self.cursor = end_offset;
            return Ok(Some(DcFrameStub {
                head_dc: DC_START_ENCAPSULATION_BINARY,
                closer_dc: Some(DC_END_ENCAPSULATION_BINARY),
                outer_span: DcByteSpan::new(start_offset, end_offset),
                inner_span: None,
                header_info,
            }));
        }

        // 2. Check if this is a literal (260..261) or identifier (270..271)
        let dc_begin_literal = DcChar::from_short(260);
        let dc_end_literal = DcChar::from_short(261);
        let dc_begin_ident = DcChar::from_short(270);
        let dc_end_ident = DcChar::from_short(271);
        let dc_assign = DcChar::from_short(269);

        if first_dc == dc_begin_literal || first_dc == dc_begin_ident {
            let closer = if first_dc == dc_begin_literal {
                dc_end_literal
            } else {
                dc_end_ident
            };
            let end_offset = self.scan_escaped_text_frame(after_first, closer, policy)?;
            // Check if this identifier is the LHS of an assignment (269)
            if first_dc == dc_begin_ident && end_offset < self.active_span.end {
                if let Ok(Some((next_dc, after_assign))) = self.read_token_or_binary_at(end_offset) {
                    if next_dc == dc_assign {
                        self.cursor = after_assign;
                        let rhs_stub = self
                            .next_sibling_stub(policy)?
                            .context("Missing value on RHS of Dc 269 assignment")?;
                        let assign_end = rhs_stub.outer_span.end;
                        self.cursor = assign_end;
                        return Ok(Some(DcFrameStub {
                            head_dc: dc_assign,
                            closer_dc: None,
                            outer_span: DcByteSpan::new(start_offset, assign_end),
                            inner_span: Some(DcByteSpan::new(after_assign, assign_end)),
                            header_info: DcFrameHeaderInfo::None,
                        }));
                    }
                }
            }
            self.cursor = end_offset;
            return Ok(Some(DcFrameStub {
                head_dc: first_dc,
                closer_dc: Some(closer),
                outer_span: DcByteSpan::new(start_offset, end_offset),
                inner_span: Some(DcByteSpan::new(after_first, end_offset)),
                header_info: DcFrameHeaderInfo::None,
            }));
        }

        // 3. Check if this is a number (6..7)
        if first_dc == DC_BEGIN_NUMBER {
            let end_offset = self.scan_until_token(after_first, DC_END_NUMBER, policy)?;
            self.cursor = end_offset;
            return Ok(Some(DcFrameStub {
                head_dc: DC_BEGIN_NUMBER,
                closer_dc: Some(DC_END_NUMBER),
                outer_span: DcByteSpan::new(start_offset, end_offset),
                inner_span: None,
                header_info: DcFrameHeaderInfo::None,
            }));
        }

        let resolver = default_syntax_resolver();
        if let Some(target) = CharTarget::from_u128(first_dc.to_u128()) {
            if resolver.is_deprecated(&target) {
                let msg = if let Some(pref) = resolver.preferred_replacement(&target) {
                    format!("Deprecated construct {target} encountered (prefer {pref})")
                } else {
                    format!("Deprecated construct {target} encountered")
                };
                policy.handle(policy.deprecated_syntax, msg)?;
            }

            // 4. Check if `first_dc` is a paired container opener in the database syntax table
            if let Some(closer_u128) = resolver.closer_for_opener(&target) {
                let closer_dc = DcChar::from_u128(closer_u128);
                let (closer_start, after_closer) =
                    self.scan_balanced_container(after_first, first_dc, closer_dc, policy)?;
                let inner_span = DcByteSpan::new(after_first, closer_start);
                let mut outer_end = after_closer;
                let mut header_info = DcFrameHeaderInfo::None;

                // Handle trailing elements after closer (e.g. [545:] after 544 for Dc 543)
                let trailing_elems = resolver.trailing_after_closer(&target);
                if !trailing_elems.is_empty() && first_dc == DC_BEGIN_CHECKSUMMED_SECTION {
                    let (final_end, seq_id, digest_ok) =
                        self.verify_checksummed_section(inner_span, after_closer, policy)?;
                    outer_end = final_end;
                    header_info = DcFrameHeaderInfo::ChecksummedSection {
                        sequence_id: seq_id,
                        body_span: inner_span,
                        digest_valid: digest_ok,
                    };
                } else if first_dc == DC_BEGIN_FORMAT_REPRESENTATION {
                    if let Ok(Some((fmt_dc, _))) = self.read_token_or_binary_at(after_first) {
                        header_info = DcFrameHeaderInfo::FormatRepresentation { format: fmt_dc };
                    }
                }

                self.cursor = outer_end;
                return Ok(Some(DcFrameStub {
                    head_dc: first_dc,
                    closer_dc: Some(closer_dc),
                    outer_span: DcByteSpan::new(start_offset, outer_end),
                    inner_span: Some(inner_span),
                    header_info,
                }));
            }

            // 5. Check if `first_dc` has a prefix syntax rule (`:~ <arg1> <arg2> ...`)
            if let Some(rule) = resolver.resolve_rule(&target) {
                let arity = prefix_rule_fixed_arity(&rule.pattern);
                if arity > 0 {
                    self.cursor = after_first;
                    let mut last_end = after_first;
                    for _ in 0..arity {
                        if let Some(arg_stub) = self.next_sibling_stub(policy)? {
                            last_end = arg_stub.outer_span.end;
                        } else {
                            policy.handle(
                                policy.syntax_error,
                                format!("Truncated arguments after prefix tag {target}"),
                            )?;
                            break;
                        }
                    }
                    self.cursor = last_end;
                    return Ok(Some(DcFrameStub {
                        head_dc: first_dc,
                        closer_dc: None,
                        outer_span: DcByteSpan::new(start_offset, last_end),
                        inner_span: Some(DcByteSpan::new(after_first, last_end)),
                        header_info: DcFrameHeaderInfo::None,
                    }));
                }
            }
        }

        // 6. Standalone atomic DcChar (e.g. boolean 517/518, 316, 551, enum discriminant)
        self.cursor = after_first;
        Ok(Some(DcFrameStub {
            head_dc: first_dc,
            closer_dc: None,
            outer_span: DcByteSpan::new(start_offset, after_first),
            inner_span: None,
            header_info: DcFrameHeaderInfo::None,
        }))
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`] and [`DcProjectionPolicy`].
    pub fn decode_frame<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
    ) -> Result<T> {
        let span = stub.outer_span;
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let mut reader = if let Some(ctx) = context {
                    DcMixedReader::with_context(mstr, ctx)
                } else {
                    DcMixedReader::new(mstr)
                };
                reader.set_position_range(
                    usize::try_from(span.start)?,
                    usize::try_from(span.end)?,
                )?;
                T::decode_dc_mixed(&mut reader, policy)
            }
            NavBackend::MemoryText(text) => {
                let range = span.to_usize_range()?;
                let sub = text
                    .get(range)
                    .context("Frame stub range out of bounds in DcStr")?;
                let mut reader = if let Some(ctx) = context {
                    DcMixedReader::from_dc_str_with_context(sub, ctx)
                } else {
                    DcMixedReader::from_dc_str(sub)
                };
                T::decode_dc_mixed(&mut reader, policy)
            }
            NavBackend::SeekableFile { .. } => {
                let raw_bytes = self.read_span_bytes(span)?;
                let mut wrapped = Vec::with_capacity(DCMM_HEADER_LEN.saturating_add(raw_bytes.len()));
                wrapped.extend_from_slice(&DCMM_MAGIC);
                wrapped.extend_from_slice(&DCMM_UUID);
                wrapped.extend_from_slice(&raw_bytes);
                let mstr = DcMstr::from_bytes(&wrapped)?;
                let mut reader = if let Some(ctx) = context {
                    DcMixedReader::with_context(mstr, ctx)
                } else {
                    DcMixedReader::new(mstr)
                };
                T::decode_dc_mixed(&mut reader, policy)
            }
        }
    }

    /// Expands a [`DcFrameStub`] into a full CST inside the reusable [`DcSyntaxArena`].
    pub fn expand_frame(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        let saved_cursor = self.cursor;
        let saved_span = self.active_span;
        self.active_span = stub.outer_span;
        self.cursor = stub.outer_span.start;
        let res = self.parse_node_into_arena(arena, policy);
        self.active_span = saved_span;
        let _ = self.seek_to_offset(saved_cursor.min(saved_span.end));
        res
    }

    /// Parses the next statement or container at the current cursor into `arena`.
    pub fn parse_node_into_arena(
        &mut self,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        let start = self.cursor;
        let Some(stub) = self.next_sibling_stub(policy)? else {
            bail!("Unexpected end of stream at offset {start}");
        };
        self.build_arena_node_from_stub(&stub, arena, policy)
    }

    fn build_arena_node_from_stub(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        let head = stub.head_dc;
        if head == DC_START_ENCAPSULATION_BINARY {
            let node = self.parse_binary_arena_node(stub.outer_span)?;
            return Ok(arena.alloc(stub.outer_span, node));
        }
        if head == DcChar::from_short(260) {
            let (type_header, decoded) = self.decode_literal_span(stub.outer_span, policy)?;
            return Ok(arena.alloc(
                stub.outer_span,
                DcSyntaxKind::Literal {
                    type_header,
                    decoded,
                },
            ));
        }
        if head == DcChar::from_short(270) {
            let name = self.decode_identifier_span(stub.outer_span, policy)?;
            return Ok(arena.alloc(stub.outer_span, DcSyntaxKind::Identifier { name }));
        }
        if head == DC_BEGIN_NUMBER {
            let (format, digits) = self.decode_number_span(stub.outer_span)?;
            return Ok(arena.alloc(stub.outer_span, DcSyntaxKind::Number { format, digits }));
        }
        let dc_assign = DcChar::from_short(269);
        if head == dc_assign {
            if let Some(inner) = stub.inner_span {
                let ident_span = DcByteSpan::new(
                    stub.outer_span.start,
                    inner.start.saturating_sub(dc_char_utf8e128_len(dc_assign)),
                );
                let ident = self.decode_identifier_span(ident_span, policy)?;
                self.enter_frame(stub)?;
                let val_id = self.parse_node_into_arena(arena, policy)?;
                self.exit_frame()?;
                return Ok(arena.alloc(
                    stub.outer_span,
                    DcSyntaxKind::Assignment {
                        ident,
                        value: val_id,
                    },
                ));
            }
        }
        if let Some(closer) = stub.closer_dc {
            let mut children = Vec::new();
            if stub.inner_span.is_some() {
                self.enter_frame(stub)?;
                while let Some(child_stub) = self.next_sibling_stub(policy)? {
                    let child_id = self.build_arena_node_from_stub(&child_stub, arena, policy)?;
                    children.push(child_id);
                }
                self.exit_frame()?;
            }
            return Ok(arena.alloc(
                stub.outer_span,
                DcSyntaxKind::Container {
                    opener: head,
                    closer,
                    header: Vec::new(),
                    children,
                    trailing: Vec::new(),
                    unclosed: false,
                },
            ));
        }
        if stub.inner_span.is_some() {
            let mut args = Vec::new();
            self.enter_frame(stub)?;
            while let Some(arg_stub) = self.next_sibling_stub(policy)? {
                let arg_id = self.build_arena_node_from_stub(&arg_stub, arena, policy)?;
                args.push(arg_id);
            }
            self.exit_frame()?;
            return Ok(arena.alloc(stub.outer_span, DcSyntaxKind::Tagged { tag: head, args }));
        }
        Ok(arena.alloc(stub.outer_span, DcSyntaxKind::Atom(head)))
    }

    fn read_token_or_binary_at(&mut self, offset: u64) -> Result<Option<(DcChar, u64)>> {
        if offset >= self.active_span.end {
            return Ok(None);
        }
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let pos = usize::try_from(offset)?;
                let Some(rem) = mstr.as_bytes().get(pos..) else {
                    return Ok(None);
                };
                if rem.is_empty() {
                    return Ok(None);
                }
                let Some((val, len)) = decode_utf_8e_128_buf(rem) else {
                    bail!("Invalid UTF-8e128 character at byte offset {offset}");
                };
                let next_offset = offset.saturating_add(u64::try_from(len)?);
                if next_offset > self.active_span.end {
                    return Ok(None);
                }
                Ok(Some((DcChar::from_u128(val), next_offset)))
            }
            NavBackend::MemoryText(text) => {
                let pos = usize::try_from(offset)?;
                let Some(rem) = text.as_bytes().get(pos..) else {
                    return Ok(None);
                };
                if rem.is_empty() {
                    return Ok(None);
                }
                let Some((val, len)) = decode_utf_8e_128_buf(rem) else {
                    bail!("Invalid UTF-8e128 character at byte offset {offset}");
                };
                let next_offset = offset.saturating_add(u64::try_from(len)?);
                if next_offset > self.active_span.end {
                    return Ok(None);
                }
                Ok(Some((DcChar::from_u128(val), next_offset)))
            }
            NavBackend::SeekableFile { reader, .. } => {
                reader.seek(SeekFrom::Start(offset))?;
                let mut first = [0_u8; 1];
                if reader.read(&mut first)? == 0 {
                    return Ok(None);
                }
                let mut buf = [0_u8; 32];
                buf[0] = first[0];
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
                    reader
                        .read_exact(&mut second)
                        .with_context(|| format!("Truncated UTF-8e128 header at offset {offset}"))?;
                    buf[1] = second[0];
                    ensure!(
                        (second[0] & 0xC0) == 0x80,
                        "Invalid UTF-8e128 length byte 0x{:02x} at offset {offset}",
                        second[0]
                    );
                    let l = usize::from(second[0] & 0x3F);
                    ensure!(
                        (4..=24).contains(&l),
                        "Invalid UTF-8e128 continuation count {l} at offset {offset}"
                    );
                    let total = l.saturating_add(2);
                    let tail = buf
                        .get_mut(2..total)
                        .context("UTF-8e128 sequence length exceeds buffer")?;
                    reader
                        .read_exact(tail)
                        .with_context(|| format!("Truncated UTF-8e128 sequence at offset {offset}"))?;
                    total
                } else {
                    bail!(
                        "Invalid UTF-8e128 lead byte 0x{:02x} at offset {offset}",
                        first[0]
                    );
                };
                if seq_len > 1 && first[0] != 0xFF {
                    let tail = buf
                        .get_mut(1..seq_len)
                        .context("UTF-8e128 sequence length exceeds buffer")?;
                    reader
                        .read_exact(tail)
                        .with_context(|| format!("Truncated UTF-8e128 sequence at offset {offset}"))?;
                }
                let slice = buf.get(..seq_len).context("UTF-8e128 slice bounds")?;
                let Some((val, _)) = decode_utf_8e_128_buf(slice) else {
                    bail!("Invalid UTF-8e128 sequence at offset {offset}");
                };
                let next_offset = offset.saturating_add(u64::try_from(seq_len)?);
                if next_offset > self.active_span.end {
                    return Ok(None);
                }
                Ok(Some((DcChar::from_u128(val), next_offset)))
            }
        }
    }

    fn scan_binary_at(&mut self, start_offset: u64) -> Result<(u64, DcFrameHeaderInfo)> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let start_usize = usize::try_from(start_offset)?;
                let rem = mstr
                    .as_bytes()
                    .get(start_usize..)
                    .context("Binary encapsulation start out of bounds")?;
                let mut chunks = crate::dc_mixed::DcMixedChunks { slice: rem };
                let chunk = chunks
                    .next()
                    .context("Empty binary encapsulation chunk")?;
                let consumed = rem.len().saturating_sub(chunks.slice.len());
                let end_offset = start_offset.saturating_add(u64::try_from(consumed)?);
                ensure!(
                    end_offset <= self.active_span.end,
                    "Binary encapsulation extends past active container span"
                );
                match chunk {
                    crate::DcMixedChunk::Binary { data, .. } => {
                        let data_len = u64::try_from(data.len())?;
                        let abs_data_start = end_offset.saturating_sub(data_len);
                        Ok((
                            end_offset,
                            DcFrameHeaderInfo::Binary {
                                data_type: None,
                                payload_span: Some(DcByteSpan::new(abs_data_start, end_offset)),
                                payload_len: data_len,
                            },
                        ))
                    }
                    crate::DcMixedChunk::BinaryWithMetadata { data, metadata } => {
                        let data_len = u64::try_from(data.len())?;
                        let abs_data_start = end_offset.saturating_sub(data_len);
                        let data_type = crate::encapsulation::parse_metadata(metadata)?
                            .data_type
                            .map(DcStr::to_owned);
                        Ok((
                            end_offset,
                            DcFrameHeaderInfo::Binary {
                                data_type,
                                payload_span: Some(DcByteSpan::new(abs_data_start, end_offset)),
                                payload_len: data_len,
                            },
                        ))
                    }
                    crate::DcMixedChunk::Text(_) => {
                        self.scan_textual_binary_at(start_offset)
                    }
                }
            }
            NavBackend::MemoryText(_) => self.scan_textual_binary_at(start_offset),
            NavBackend::SeekableFile { reader, .. } => {
                // On-disk .dcmm binary encapsulation layout (matches dc_mixed.rs):
                // 6 bytes: DC_203_BYTES (Dc 203)
                // 1 byte: type_byte (0 = TYPE_BINARY, 2 = TYPE_BINARY_METADATA, 3 = TYPE_BINARY_SHA256)
                // 16 bytes: u128 BE data_len
                // If 2 (TYPE_BINARY_METADATA): u128 BE meta_len + meta_len bytes of DcUtf metadata
                // If 3 (TYPE_BINARY_SHA256): 32 bytes SHA-256
                // data_len bytes of raw payload (skipped via SeekFrom::Start!)
                let after_dc203 = start_offset
                    .checked_add(u64::try_from(crate::dc_mixed::DC_203_BYTES.len())?)
                    .context("Binary encapsulation offset overflow")?;
                reader.seek(SeekFrom::Start(after_dc203))?;
                let mut type_byte = [0_u8; 1];
                reader
                    .read_exact(&mut type_byte)
                    .context("Truncated .dcmm binary encapsulation type")?;
                let encap_type = type_byte[0];
                let mut len_buf = [0_u8; 16];
                reader
                    .read_exact(&mut len_buf)
                    .context("Truncated .dcmm binary data length")?;
                let data_len = u64::try_from(u128::from_be_bytes(len_buf))
                    .context("Binary data length exceeds u64")?;
                let mut data_type = None;
                match encap_type {
                    crate::dc_mixed::TYPE_BINARY => {}
                    crate::dc_mixed::TYPE_BINARY_METADATA => {
                        let mut meta_len_buf = [0_u8; 16];
                        reader
                            .read_exact(&mut meta_len_buf)
                            .context("Truncated .dcmm metadata length")?;
                        let meta_len = u64::try_from(u128::from_be_bytes(meta_len_buf))
                            .context("Metadata length exceeds u64")?;
                        let meta_len_usize = usize::try_from(meta_len)?;
                        let mut meta_bytes = vec![0_u8; meta_len_usize];
                        reader
                            .read_exact(&mut meta_bytes)
                            .context("Truncated .dcmm metadata payload")?;
                        let meta_str = DcStr::from_bytes(&meta_bytes)?;
                        data_type = crate::encapsulation::parse_metadata(meta_str)?
                            .data_type
                            .map(DcStr::to_owned);
                    }
                    crate::dc_mixed::TYPE_BINARY_SHA256 => {
                        reader.seek(SeekFrom::Current(32))?;
                    }
                    _ => {
                        bail!(
                            "Invalid .dcmm binary encapsulation type {encap_type} at offset {start_offset}"
                        );
                    }
                }
                let payload_start = reader.stream_position()?;
                let payload_end = payload_start
                    .checked_add(data_len)
                    .context("Binary payload offset overflow")?;
                ensure!(
                    payload_end <= self.active_span.end,
                    "Truncated .dcmm binary encapsulation at offset {start_offset} (needs {payload_end}, end is {})",
                    self.active_span.end
                );
                // Seek directly past the binary payload in O(1) without reading it into RAM!
                reader.seek(SeekFrom::Start(payload_end))?;
                Ok((
                    payload_end,
                    DcFrameHeaderInfo::Binary {
                        data_type,
                        payload_span: Some(DcByteSpan::new(payload_start, payload_end)),
                        payload_len: data_len,
                    },
                ))
            }
        }
    }

    fn scan_textual_binary_at(&mut self, start_offset: u64) -> Result<(u64, DcFrameHeaderInfo)> {
        let mut offset = start_offset;
        let mut depth = 0usize;
        while let Some((ch, next_offset)) = self.read_token_or_binary_at(offset)? {
            offset = next_offset;
            if ch == DC_START_ENCAPSULATION_BINARY {
                depth = depth.saturating_add(1);
            } else if ch == DC_END_ENCAPSULATION_BINARY {
                depth = depth.checked_sub(1).context("Unexpected Dc 204")?;
                if depth == 0 {
                    return Ok((
                        offset,
                        DcFrameHeaderInfo::Binary {
                            data_type: None,
                            payload_span: None,
                            payload_len: 0,
                        },
                    ));
                }
            }
        }
        bail!("Unclosed textual binary encapsulation (Dc 203) starting at offset {start_offset}");
    }

    fn scan_escaped_text_frame(
        &mut self,
        after_opener: u64,
        closer: DcChar,
        policy: &DcProjectionPolicy,
    ) -> Result<u64> {
        let mut offset = after_opener;
        if closer == DcChar::from_short(261) {
            // Scan required [262:]{1} type header first if present
            if let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
                if ch == DcChar::from_short(262) {
                    offset = self.scan_until_token(next_off, DcChar::from_short(263), policy)?;
                } else {
                    policy.handle(
                        policy.syntax_error,
                        format!("Literal at offset {after_opener} is missing Dc 262..263 type header"),
                    )?;
                }
            }
        }
        while let Some((ch, next_offset)) = self.read_token_or_binary_at(offset)? {
            offset = next_offset;
            if ch == DC_ESCAPE {
                if let Some((_, after_escaped)) = self.read_token_or_binary_at(offset)? {
                    offset = after_escaped;
                } else {
                    policy.handle(
                        policy.syntax_error,
                        format!("Dangling escape Dc 255 at offset {offset}"),
                    )?;
                    return Ok(offset);
                }
            } else if ch == closer {
                return Ok(offset);
            }
        }
        policy.handle(
            policy.syntax_error,
            format!("Unclosed text frame: missing closer {closer:?}"),
        )?;
        Ok(offset)
    }

    fn scan_until_token(
        &mut self,
        start: u64,
        closer: DcChar,
        policy: &DcProjectionPolicy,
    ) -> Result<u64> {
        let mut offset = start;
        while let Some((ch, next_offset)) = self.read_token_or_binary_at(offset)? {
            offset = next_offset;
            if ch == closer {
                return Ok(offset);
            }
        }
        policy.handle(
            policy.syntax_error,
            format!("Unclosed frame starting at {start}: missing closer {closer:?}"),
        )?;
        Ok(offset)
    }

    fn scan_balanced_container(
        &mut self,
        after_opener: u64,
        opener: DcChar,
        closer: DcChar,
        policy: &DcProjectionPolicy,
    ) -> Result<(u64, u64)> {
        let resolver = default_syntax_resolver();
        let mut closer_stack = vec![closer];
        let mut offset = after_opener;

        let dc_begin_literal = DcChar::from_short(260);
        let dc_end_literal = DcChar::from_short(261);
        let dc_begin_ident = DcChar::from_short(270);
        let dc_end_ident = DcChar::from_short(271);

        while let Some((ch, next_offset)) = self.read_token_or_binary_at(offset)? {
            let token_start = offset;
            if ch == DC_START_ENCAPSULATION_BINARY {
                let (after_bin, _) = self.scan_binary_at(token_start)?;
                offset = after_bin;
                continue;
            }
            if ch == dc_begin_literal {
                offset = self.scan_escaped_text_frame(next_offset, dc_end_literal, policy)?;
                continue;
            }
            if ch == dc_begin_ident {
                offset = self.scan_escaped_text_frame(next_offset, dc_end_ident, policy)?;
                continue;
            }
            if ch == DC_BEGIN_NUMBER {
                offset = self.scan_until_token(next_offset, DC_END_NUMBER, policy)?;
                continue;
            }
            if closer_stack.last() == Some(&ch) {
                closer_stack.pop();
                if closer_stack.is_empty() {
                    return Ok((token_start, next_offset));
                }
                offset = next_offset;
                continue;
            }
            // Sacred opener/closer framing: if an inner container was left unclosed
            // before an outer closer on the stack, synchronize at the outer closer.
            if let Some(stack_pos) = closer_stack.iter().rposition(|&c| c == ch) {
                policy.handle(
                    policy.syntax_error,
                    format!(
                        "Unclosed inner container before closer {ch:?} at offset {token_start}"
                    ),
                )?;
                closer_stack.truncate(stack_pos);
                if closer_stack.is_empty() {
                    return Ok((token_start, next_offset));
                }
                offset = next_offset;
                continue;
            }
            if let Some(target) = CharTarget::from_u128(ch.to_u128()) {
                if let Some(inner_closer) = resolver.closer_for_opener(&target) {
                    closer_stack.push(DcChar::from_u128(inner_closer));
                }
            }
            offset = next_offset;
        }

        policy.handle(
            policy.syntax_error,
            format!("Unclosed container {opener:?}..{closer:?} starting at offset {after_opener}"),
        )?;
        Ok((offset, offset))
    }

    fn verify_checksummed_section(
        &mut self,
        body_span: DcByteSpan,
        after_closer: u64,
        policy: &DcProjectionPolicy,
    ) -> Result<(u64, Option<u64>, bool)> {
        // Parse optional leading 546 SequenceIdentifier inside body_span
        let mut sequence_id = None;
        if let Ok(Some((first_in_body, after_tag))) = self.read_token_or_binary_at(body_span.start) {
            if first_in_body == DC_SEQUENCE_IDENTIFIER {
                if let Ok(Some((num_open, after_num_open))) = self.read_token_or_binary_at(after_tag) {
                    if num_open == DC_BEGIN_NUMBER {
                        if let Ok(num_end) = self.scan_until_token(after_num_open, DC_END_NUMBER, policy) {
                            let (_, digits) = self.decode_number_span(DcByteSpan::new(after_tag, num_end))?;
                            if let Ok(int_val) = decode_integer_from_digits(&digits) {
                                sequence_id = u64::try_from(&int_val).ok();
                            }
                        }
                    }
                }
            }
        }

        // Expect trailing 545 SectionDigest + [checksum] (203..204)
        let Some((digest_tag, after_digest_tag)) = self.read_token_or_binary_at(after_closer)? else {
            policy.handle(
                policy.syntax_error,
                "Missing Dc 545 (SectionDigest) after Dc 544 (EndChecksummedSection)",
            )?;
            return Ok((after_closer, sequence_id, false));
        };
        if digest_tag != DC_SECTION_DIGEST {
            policy.handle(
                policy.syntax_error,
                format!("Expected Dc 545 (SectionDigest) after Dc 544, found {digest_tag:?}"),
            )?;
            return Ok((after_closer, sequence_id, false));
        }
        let (final_end, bin_header) = self.scan_binary_at(after_digest_tag)?;
        let expected_digest = match bin_header {
            DcFrameHeaderInfo::Binary {
                payload_span: Some(pspan),
                ..
            } => self.read_span_bytes(pspan)?.into_owned(),
            _ => {
                let node = self.parse_binary_arena_node(DcByteSpan::new(after_digest_tag, final_end))?;
                match node {
                    DcSyntaxKind::Binary {
                        payload: BinaryPayloadRef::Borrowed(b),
                        ..
                    } => b.to_vec(),
                    DcSyntaxKind::Binary {
                        payload: BinaryPayloadRef::Inline(b),
                        ..
                    } => b,
                    _ => Vec::new(),
                }
            }
        };
        let actual_digest = self.sha256_of_span(body_span)?;
        let valid = actual_digest.as_slice() == expected_digest.as_slice();
        if !valid {
            policy.handle(
                policy.syntax_error,
                format!(
                    "Checksummed section (Dc 543..544) SHA-256 mismatch at span {:?}",
                    body_span
                ),
            )?;
        }
        Ok((final_end, sequence_id, valid))
    }

    fn parse_binary_arena_node(&mut self, span: DcByteSpan) -> Result<DcSyntaxKind<'a>> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let range = span.to_usize_range()?;
                let slice = mstr
                    .as_bytes()
                    .get(range)
                    .context("Binary node span out of bounds")?;
                let mut chunks = crate::dc_mixed::DcMixedChunks { slice };
                match chunks.next().context("Empty binary span")? {
                    crate::DcMixedChunk::Binary { data, .. } => {
                        Ok(DcSyntaxKind::Binary {
                            data_type: None,
                            checksum_format: None,
                            payload: BinaryPayloadRef::Borrowed(data),
                        })
                    }
                    crate::DcMixedChunk::BinaryWithMetadata { data, metadata } => {
                        let parsed = crate::encapsulation::parse_metadata(metadata)?;
                        let dt = parsed.data_type.map(DcStr::to_owned);
                        let cf = parsed.data_type.and_then(|d| d.chars().next());
                        Ok(DcSyntaxKind::Binary {
                            data_type: dt,
                            checksum_format: cf,
                            payload: BinaryPayloadRef::Borrowed(data),
                        })
                    }
                    crate::DcMixedChunk::Text(txt) => {
                        let (header, data, _) = crate::encapsulation::parse_text(txt)?;
                        let cf = header.chars().next();
                        Ok(DcSyntaxKind::Binary {
                            data_type: if header.is_empty() { None } else { Some(header) },
                            checksum_format: cf,
                            payload: BinaryPayloadRef::Inline(data),
                        })
                    }
                }
            }
            NavBackend::MemoryText(text) => {
                let range = span.to_usize_range()?;
                let sub = text
                    .get(range)
                    .context("Binary node span out of bounds in DcStr")?;
                let (header, data, _) = crate::encapsulation::parse_text(sub)?;
                let cf = header.chars().next();
                Ok(DcSyntaxKind::Binary {
                    data_type: if header.is_empty() { None } else { Some(header) },
                    checksum_format: cf,
                    payload: BinaryPayloadRef::Inline(data),
                })
            }
            NavBackend::SeekableFile { .. } => {
                let (_, info) = self.scan_binary_at(span.start)?;
                let DcFrameHeaderInfo::Binary {
                    data_type,
                    payload_span: Some(pspan),
                    payload_len,
                } = info
                else {
                    bail!("Expected binary header info at span {:?}", span);
                };
                let checksum_format = data_type.as_ref().and_then(|d| d.chars().next());
                let payload = if payload_len <= INLINE_BINARY_THRESHOLD {
                    let bytes = self.read_span_bytes(pspan)?.into_owned();
                    BinaryPayloadRef::Inline(bytes)
                } else {
                    BinaryPayloadRef::FileSpan {
                        file_offset: pspan.start,
                        len: payload_len,
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

    fn decode_literal_span(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
    ) -> Result<(Vec<DcChar>, String)> {
        let mut offset = span.start;
        let Some((first, after_first)) = self.read_token_or_binary_at(offset)? else {
            bail!("Empty literal span");
        };
        ensure!(first == DcChar::from_short(260), "Expected Dc 260");
        offset = after_first;
        let mut type_header = Vec::new();
        if let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
            if ch == DcChar::from_short(262) {
                offset = next_off;
                while offset < span.end {
                    let Some((tch, after_tch)) = self.read_token_or_binary_at(offset)? else {
                        break;
                    };
                    offset = after_tch;
                    if tch == DcChar::from_short(263) {
                        break;
                    }
                    type_header.push(tch);
                }
            }
        }
        let mut decoded = String::new();
        while offset < span.end {
            let Some((mut ch, next_off)) = self.read_token_or_binary_at(offset)? else {
                break;
            };
            offset = next_off;
            if ch == DcChar::from_short(261) {
                break;
            }
            if ch == DC_ESCAPE {
                if let Some((escaped, after_esc)) = self.read_token_or_binary_at(offset)? {
                    ch = escaped;
                    offset = after_esc;
                }
            }
            if let Ok(cp) = ch.to_unicode() {
                if let Some(c) = char::from_u32(cp) {
                    decoded.push(c);
                } else {
                    policy.handle(policy.syntax_error, format!("Invalid Unicode codepoint {cp} in literal"))?;
                }
            } else {
                policy.handle(policy.syntax_error, format!("Non-Unicode character {ch:?} in literal"))?;
            }
        }
        Ok((type_header, decoded))
    }

    fn decode_identifier_span(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
    ) -> Result<String> {
        let mut offset = span.start;
        let Some((first, after_first)) = self.read_token_or_binary_at(offset)? else {
            bail!("Empty identifier span");
        };
        ensure!(first == DcChar::from_short(270), "Expected Dc 270");
        offset = after_first;
        let mut name = String::new();
        while offset < span.end {
            let Some((mut ch, next_off)) = self.read_token_or_binary_at(offset)? else {
                break;
            };
            offset = next_off;
            if ch == DcChar::from_short(271) {
                break;
            }
            if ch == DC_ESCAPE {
                if let Some((escaped, after_esc)) = self.read_token_or_binary_at(offset)? {
                    ch = escaped;
                    offset = after_esc;
                }
            }
            if let Ok(cp) = ch.to_unicode() {
                if let Some(c) = char::from_u32(cp) {
                    name.push(c);
                } else {
                    policy.handle(policy.syntax_error, format!("Invalid Unicode codepoint {cp} in identifier"))?;
                }
            } else {
                policy.handle(policy.syntax_error, format!("Non-Unicode character {ch:?} in identifier"))?;
            }
        }
        Ok(name)
    }

    fn decode_number_span(&mut self, span: DcByteSpan) -> Result<(DcChar, Vec<DcChar>)> {
        let mut offset = span.start;
        let Some((first, after_first)) = self.read_token_or_binary_at(offset)? else {
            bail!("Empty number span");
        };
        ensure!(first == DC_BEGIN_NUMBER, "Expected Dc 6");
        offset = after_first;
        let Some((format, after_fmt)) = self.read_token_or_binary_at(offset)? else {
            bail!("Truncated number format");
        };
        offset = after_fmt;
        let mut digits = Vec::new();
        while offset < span.end {
            let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? else {
                break;
            };
            offset = next_off;
            if ch == DC_END_NUMBER {
                break;
            }
            digits.push(ch);
        }
        Ok((format, digits))
    }
}

fn dc_char_utf8e128_len(ch: DcChar) -> u64 {
    let mut buf = [0_u8; 32];
    let len = encode_utf_8e_128_buf(&mut buf, ch.to_u128());
    u64::try_from(len).unwrap_or(6)
}

fn decode_integer_from_digits(digits: &[DcChar]) -> Result<Integer> {
    let mut chars = Vec::with_capacity(digits.len().saturating_add(3));
    chars.push(DC_BEGIN_NUMBER);
    chars.push(crate::dc_number::DC_INTEGER);
    chars.extend_from_slice(digits);
    chars.push(DC_END_NUMBER);
    let mut iter = chars.into_iter();
    crate::dc_number::read_dc_number_from_reader(|| {
        iter.next().context("Unexpected end of number digits")
    })
}

/// Computes the fixed argument count following `~` for a prefix syntax rule
/// (e.g. `:~ [filepath]` -> 1, `:~ 376 [number] 377 [number]` -> 2 tagged args
/// or 4 tokens if flat).
pub(crate) fn prefix_rule_fixed_arity(pattern: &SyntaxPattern) -> usize {
    match pattern {
        SyntaxPattern::Sequence(elems) => {
            let Some(first) = elems.first() else {
                return 0;
            };
            if !matches!(first.term, SyntaxTerm::SelfChar) {
                return 0;
            }
            // If all remaining elements are ExactOne and none are negated charsets (`[^...]`)
            let tail = elems.get(1..).unwrap_or(&[]);
            if tail.is_empty() {
                return 0;
            }
            for el in tail {
                if el.quantifier != Quantifier::ExactOne {
                    return 0;
                }
                if matches!(el.term, SyntaxTerm::CharSet { negated: true, .. }) {
                    return 0;
                }
            }
            // Check for extent-style rules `:~ 376 [number] 377 [number]` where 376 and 377
            // are themselves prefix tags of arity 1!
            let resolver = default_syntax_resolver();
            let mut step_count = 0usize;
            let mut idx = 0usize;
            while let Some(el) = tail.get(idx) {
                step_count = step_count.saturating_add(1);
                if let SyntaxTerm::CharRef(ref target) = el.term {
                    if let Some(sub_rule) = resolver.resolve_rule(target) {
                        let sub_arity = prefix_rule_fixed_arity(&sub_rule.pattern);
                        if sub_arity > 0 {
                            idx = idx.saturating_add(1).saturating_add(sub_arity);
                            continue;
                        }
                    }
                }
                idx = idx.saturating_add(1);
            }
            step_count
        }
        SyntaxPattern::Alternation(branches) => {
            let first_arity = branches.first().map_or(0, prefix_rule_fixed_arity);
            if first_arity > 0
                && branches
                    .iter()
                    .all(|b| prefix_rule_fixed_arity(b) == first_arity)
            {
                first_arity
            } else {
                0
            }
        }
    }
}
