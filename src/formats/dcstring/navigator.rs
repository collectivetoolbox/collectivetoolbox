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
    DC_ASSIGN, DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_FORMAT_REPRESENTATION,
    DC_BEGIN_IDENTIFIER, DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR,
    DC_BEGIN_LEGACY_KV_MAP, DC_BEGIN_LIST, DC_BEGIN_LITERAL, DC_BEGIN_NUMBER,
    DC_BEGIN_PARAMETER, DC_BEGIN_TYPE_DEF, DC_END_ENCAPSULATION_BINARY,
    DC_END_FORMAT_REPRESENTATION, DC_END_IDENTIFIER, DC_END_KV_MAP,
    DC_END_KV_PAIR, DC_END_LEGACY_KV_MAP, DC_END_LIST, DC_END_LITERAL,
    DC_END_NUMBER, DC_END_PARAMETER, DC_END_TYPE_DEF,
    DC_EQUIVALENT_REPRESENTATIONS, DC_ESCAPE, DC_LIST_ELEMENT_SEPARATOR,
    DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT,
    DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS, DC_ROUTINE_INVOCATION,
    DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER, DC_START_ENCAPSULATION_BINARY,
    DC_TYPE_ANY, DC_TYPE_INTERSECTION, DC_TYPE_UNION,
};
use ctb_formats_dcdata::syntax::{
    CharTarget, Quantifier, SyntaxDiagnostic, SyntaxElement, SyntaxPattern,
    SyntaxRuleResolver, SyntaxTerm, default_syntax_resolver,
    parse_data_constraint, parse_dc_syntax,
};
use ctb_formats_utf_8e_128::{decode_utf_8e_128_buf, encode_utf_8e_128_buf};
use sha2::{Digest, Sha256};

use crate::{
    BinaryPayload, DCMM_HEADER_LEN, DCMM_MAGIC, DCMM_UUID, DcChar,
    DcMixedDecode, DcProjectionContext, DcMstr, DcMstring, DcStr, DcString, Integer,
    Natural, SharedValueContext,
};

/// Maximum recursion depth allowed during grammar rule expansion.
pub const MAX_SYNTAX_EXPANSION_DEPTH: usize = 32;

/// Default maximum container and wrapper nesting depth during arena building and projection.
pub const DEFAULT_MAX_PROJECTION_DEPTH: usize = 64;

/// Execution context for evaluating [`SyntaxPattern`] rules directly over a [`DcDocumentNavigator`].
#[derive(Clone)]
pub struct SyntaxEvalContext<'r> {
    /// Syntax rule and named-type resolver.
    pub resolver: &'r dyn SyntaxRuleResolver,
    /// Defining character token (`~`) when evaluating a character's own syntax rule.
    pub self_dc: Option<u128>,
    /// Current recursion depth.
    pub depth: usize,
    /// Maximum allowed recursion depth.
    pub max_depth: usize,
    /// Call stack for cycle detection.
    pub call_stack: Vec<String>,
    /// When true, matching a container via `RuleRef` recursively validates its interior children.
    pub validate_interiors: bool,
    /// Syntax diagnostics recorded during evaluation.
    pub diagnostics: Vec<SyntaxDiagnostic>,
    /// True if any framing or constraint error occurred during evaluation.
    pub has_errors: bool,
}

impl<'r> SyntaxEvalContext<'r> {
    /// Creates a new syntax evaluation context with default recursion bounds.
    #[must_use]
    pub fn new(resolver: &'r dyn SyntaxRuleResolver, self_dc: Option<u128>) -> Self {
        Self {
            resolver,
            self_dc,
            depth: 0,
            max_depth: MAX_SYNTAX_EXPANSION_DEPTH,
            call_stack: Vec::new(),
            validate_interiors: false,
            diagnostics: Vec::new(),
            has_errors: false,
        }
    }
}

/// Format syntax rule wrapping the format's AST syntax pattern expression.
#[derive(Debug, Clone)]
pub struct FormatSyntaxRule {
    /// Evaluated syntax pattern expression for the format.
    pub expr: ctb_formats_dcdata::DcSyntaxExpr,
}

/// Runtime resolver providing syntax rule lookups for Document Character formats.
#[derive(Debug, Clone)]
pub struct SyntaxRuntimeResolver {
    resolver: std::sync::Arc<ctb_formats_dcdata::syntax::DatasetRuleResolver>,
}

impl SyntaxRuntimeResolver {
    /// Looks up the syntax rule for a format by its [`FormatId`].
    #[must_use]
    pub fn get_format_rule(&self, fid: FormatId) -> Option<FormatSyntaxRule> {
        let rule = self.resolver.resolve_rule(&CharTarget::Format(fid))?;
        let expr = match &rule.pattern {
            ctb_formats_dcdata::DcSyntaxExpr::Sequence(items) => {
                let payload_items: Vec<_> = items
                    .iter()
                    .filter(|el| !matches!(el.term, ctb_formats_dcdata::SyntaxTerm::SelfChar))
                    .cloned()
                    .collect();
                ctb_formats_dcdata::DcSyntaxExpr::Sequence(payload_items)
            }
            other => other.clone(),
        };
        Some(FormatSyntaxRule { expr })
    }
}

/// Global runtime registry for Document Character syntax rules and resolvers.
#[derive(Debug, Clone)]
pub struct SyntaxRuntimeRegistry {
    resolver: SyntaxRuntimeResolver,
}

impl SyntaxRuntimeRegistry {
    /// Returns the shared global [`SyntaxRuntimeRegistry`] instance.
    #[must_use]
    pub fn global() -> &'static Self {
        static INSTANCE: std::sync::LazyLock<SyntaxRuntimeRegistry> =
            std::sync::LazyLock::new(|| SyntaxRuntimeRegistry {
                resolver: SyntaxRuntimeResolver {
                    resolver: default_syntax_resolver().clone(),
                },
            });
        &INSTANCE
    }

    /// Returns the syntax rule resolver instance.
    #[must_use]
    pub fn resolver(&self) -> &SyntaxRuntimeResolver {
        &self.resolver
    }
}

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
    /// Maximum container or wrapper nesting depth allowed during arena construction and projection.
    pub max_depth: usize,
    /// Action taken when container or wrapper nesting exceeds [`Self::max_depth`].
    pub depth_limit_exceeded: DiagnosticAction,
}

impl DcProjectionPolicy {
    /// Strict policy: errors on any syntax violation, unknown field tag,
    /// malformed field, missing required field, or exceeded nesting depth;
    /// warns on deprecated syntax.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            syntax_error: DiagnosticAction::Error,
            unknown_field: DiagnosticAction::Error,
            deprecated_syntax: DiagnosticAction::Warn,
            malformed_field: DiagnosticAction::Error,
            missing_required_field: DiagnosticAction::Error,
            max_depth: DEFAULT_MAX_PROJECTION_DEPTH,
            depth_limit_exceeded: DiagnosticAction::Error,
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
            max_depth: DEFAULT_MAX_PROJECTION_DEPTH,
            depth_limit_exceeded: DiagnosticAction::Warn,
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
            max_depth: DEFAULT_MAX_PROJECTION_DEPTH,
            depth_limit_exceeded: DiagnosticAction::Ignore,
        }
    }

    /// Returns a copy of this policy with a custom maximum nesting depth and action.
    #[must_use]
    pub const fn with_max_depth(
        mut self,
        max_depth: usize,
        action: DiagnosticAction,
    ) -> Self {
        self.max_depth = max_depth;
        self.depth_limit_exceeded = action;
        self
    }

    /// Returns true if all structural and projection anomalies are set to `Error`.
    #[must_use]
    pub const fn is_strict(&self) -> bool {
        matches!(self.syntax_error, DiagnosticAction::Error)
            && matches!(self.unknown_field, DiagnosticAction::Error)
            && matches!(self.malformed_field, DiagnosticAction::Error)
            && matches!(self.missing_required_field, DiagnosticAction::Error)
            && matches!(self.depth_limit_exceeded, DiagnosticAction::Error)
    }

    /// Returns true if any structural or projection anomaly is configured to recover/warn/ignore.
    #[must_use]
    pub const fn is_forgiving(&self) -> bool {
        !self.is_strict()
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

impl<A: DcForgivingDefault, B: DcForgivingDefault> DcForgivingDefault for (A, B) {
    fn dc_forgiving_default() -> Self {
        (A::dc_forgiving_default(), B::dc_forgiving_default())
    }
}

impl<A: DcForgivingDefault, B: DcForgivingDefault, C: DcForgivingDefault> DcForgivingDefault
    for (A, B, C)
{
    fn dc_forgiving_default() -> Self {
        (
            A::dc_forgiving_default(),
            B::dc_forgiving_default(),
            C::dc_forgiving_default(),
        )
    }
}

impl<
    A: DcForgivingDefault,
    B: DcForgivingDefault,
    C: DcForgivingDefault,
    D: DcForgivingDefault,
> DcForgivingDefault for (A, B, C, D) {
    fn dc_forgiving_default() -> Self {
        (
            A::dc_forgiving_default(),
            B::dc_forgiving_default(),
            C::dc_forgiving_default(),
            D::dc_forgiving_default(),
        )
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
    #[expect(
        clippy::expect_used,
        reason = "In-memory byte slice lengths fit within u64 on all supported platforms"
    )]
    pub fn len(&self) -> u64 {
        match self {
            Self::Borrowed(slice) => u64::try_from(slice.len())
                .expect("Borrowed slice length fits in u64"),
            Self::Inline(bytes) => u64::try_from(bytes.len())
                .expect("Inline byte buffer length fits in u64"),
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

    /// Materializes in-memory bytes without requiring an on-disk reader.
    ///
    /// # Errors
    /// Returns an error if the payload is an unmaterialized on-disk `FileSpan`.
    pub fn materialize_bytes(&self) -> Result<Cow<'a, [u8]>> {
        self.materialize(None::<&mut std::io::Cursor<&[u8]>>)
    }
}

/// Resolver capable of materializing lazy [`BinaryPayloadRef::FileSpan`] payloads
/// from an underlying seekable file during CST projection.
pub trait DcBinaryResolver: std::fmt::Debug {
    /// Materializes the raw bytes of `payload`, reading from the backing stream if lazy.
    fn materialize_binary<'a>(
        &mut self,
        payload: &BinaryPayloadRef<'a>,
    ) -> Result<Cow<'a, [u8]>>;
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

impl<'a> DcSyntaxNode<'a> {
    /// Returns the child node IDs if this node is a container or tagged construct.
    #[must_use]
    pub fn children(&self) -> &[NodeId] {
        match &self.kind {
            DcSyntaxKind::Container { children, .. } => children,
            DcSyntaxKind::Tagged { args, .. } => args,
            _ => &[],
        }
    }
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

    /// Borrows a node by its [`NodeId`] (alias for [`Self::get`]).
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&DcSyntaxNode<'a>> {
        self.get(id)
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

    /// Splits a sequence of child [`NodeId`]s on [`DC_LIST_ELEMENT_SEPARATOR`] (`Dc 312`).
    #[must_use]
    pub fn split_operands(&self, nodes: &[NodeId]) -> Vec<Vec<NodeId>> {
        let mut result = Vec::new();
        let mut current = Vec::new();
        for &id in nodes {
            if self
                .get(id)
                .is_some_and(|n| matches!(n.kind, DcSyntaxKind::Atom(ch) if ch == DC_LIST_ELEMENT_SEPARATOR))
            {
                result.push(std::mem::take(&mut current));
            } else {
                current.push(id);
            }
        }
        result.push(current);
        result
    }

    /// Extracts the header and key-value operand slices from a map container
    /// (`Dc 559..560` with `Dc 514..515` pairs or legacy `Dc 313..314`).
    ///
    /// When `require_object_headers` is true, requires both leading type tokens
    /// to be [`DC_TYPE_OBJECT`] (`Dc 275`).
    pub fn extract_kv_map_operand_entries(
        &self,
        id: NodeId,
        require_object_headers: bool,
    ) -> Result<(DcChar, Vec<NodeId>, Vec<(Vec<NodeId>, Vec<NodeId>)>)> {
        let node = self.get(id).context("NodeId must exist in arena")?;
        let DcSyntaxKind::Container {
            opener,
            children,
            unclosed,
            ..
        } = &node.kind
        else {
            bail!("Expected key-value map container, found {:?}", node.kind);
        };
        ensure!(!*unclosed, "Unclosed key-value map container ({opener})");
        let is_pairs = *opener == DC_BEGIN_KV_MAP;
        let is_legacy = *opener == DC_BEGIN_LEGACY_KV_MAP;
        ensure!(
            is_pairs || is_legacy,
            "Expected map container (Dc 559 or Dc 313), found opener {opener}"
        );
        let header = children.get(..2).context("Missing map type declarations")?;
        if require_object_headers {
            ensure!(
                header.iter().all(|&child| {
                    self.get(child).is_some_and(
                        |cn| matches!(cn.kind, DcSyntaxKind::Atom(ch) if ch == DC_TYPE_ANY),
                    )
                }),
                "Source-backed typed map editing requires uncounted Object/Object headers"
            );
        } else {
            ensure!(
                header.iter().all(|&child| {
                    self.get(child).is_some_and(|cn| {
                        matches!(
                            cn.kind,
                            DcSyntaxKind::Atom(ch)
                                if ch == DC_TYPE_ANY || ctb_formats_dcdata::is_type_token(ch.0)
                        )
                    })
                }),
                "Missing or invalid map type header tokens"
            );
        }
        let body = children.get(2..).context("Missing map body")?;
        let mut entries = Vec::new();
        if is_legacy {
            if !body.is_empty() {
                let operands = self.split_operands(body);
                for pair in operands.chunks(2) {
                    let [key, value] = pair else {
                        bail!("Unpaired legacy map operand");
                    };
                    ensure!(
                        !key.is_empty() && !value.is_empty(),
                        "Empty legacy map operand"
                    );
                    entries.push((key.to_vec(), value.to_vec()));
                }
            }
        } else {
            for &pair_id in body {
                let pair_node = self.get(pair_id).context("Pair NodeId must exist in arena")?;
                let (pair_opener, pair_children, pair_unclosed) = match &pair_node.kind {
                    DcSyntaxKind::Container {
                        opener,
                        children,
                        unclosed,
                        ..
                    } => (*opener, children, *unclosed),
                    other => bail!("Expected explicitly framed map entry (Dc 514), found {other:?}"),
                };
                ensure!(
                    pair_opener == DC_BEGIN_KV_PAIR && !pair_unclosed,
                    "Expected explicitly framed map entry (Dc 514..515)"
                );
                let operands = self.split_operands(pair_children);
                let [key, value] = operands.as_slice() else {
                    bail!("Expected two pair operands");
                };
                ensure!(!key.is_empty() && !value.is_empty(), "Empty pair operand");
                entries.push((key.to_vec(), value.to_vec()));
            }
        }
        Ok((*opener, header.to_vec(), entries))
    }

    /// Extracts single-node `(key_id, value_id)` pairs from a map container
    /// (`Dc 559..560` or `Dc 313..314`).
    pub fn extract_kv_map_node_pairs(
        &self,
        id: NodeId,
    ) -> Result<(DcChar, Vec<(NodeId, NodeId)>)> {
        let (opener, _, operand_entries) = self.extract_kv_map_operand_entries(id, false)?;
        let mut pairs = Vec::with_capacity(operand_entries.len());
        for (key_nodes, val_nodes) in operand_entries {
            let [k_id] = key_nodes.as_slice() else {
                bail!(
                    "Expected single key node in map entry, found {}",
                    key_nodes.len()
                );
            };
            let [v_id] = val_nodes.as_slice() else {
                bail!(
                    "Expected single value node in map entry, found {}",
                    val_nodes.len()
                );
            };
            pairs.push((*k_id, *v_id));
        }
        Ok((opener, pairs))
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

impl DcFrameStub {
    /// Returns true if this container frame reached the end of the active span
    /// before its closing delimiter was encountered.
    #[must_use]
    pub const fn is_unclosed(&self) -> bool {
        match (self.closer_dc, self.inner_span) {
            (Some(_), Some(inner)) => inner.end >= self.outer_span.end,
            _ => false,
        }
    }
}

#[derive(Debug)]
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
#[derive(Debug)]
pub struct DcDocumentNavigator<'a> {
    backend: NavBackend<'a>,
    cursor: u64,
    active_span: DcByteSpan,
    level_stack: Vec<(DcByteSpan, u64)>,
}

impl<'b> DcBinaryResolver for DcDocumentNavigator<'b> {
    fn materialize_binary<'a>(
        &mut self,
        payload: &BinaryPayloadRef<'a>,
    ) -> Result<Cow<'a, [u8]>> {
        match &mut self.backend {
            NavBackend::SeekableFile { reader, .. } => payload.materialize(Some(reader)),
            NavBackend::MemoryMstr(_) | NavBackend::MemoryText(_) => payload.materialize_bytes(),
        }
    }
}

impl<'a> DcDocumentNavigator<'a> {
    /// Creates a navigator over an in-memory [`DcMstr`].
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "DCMM_HEADER_LEN and in-memory DcMstr byte lengths fit within u64 on all supported platforms"
    )]
    pub fn from_mstr(mstr: &'a DcMstr) -> Self {
        let start = u64::try_from(DCMM_HEADER_LEN)
            .expect("DCMM_HEADER_LEN fits in u64");
        let end = u64::try_from(mstr.len())
            .expect("DcMstr byte length fits in u64");
        Self {
            backend: NavBackend::MemoryMstr(mstr),
            cursor: start,
            active_span: DcByteSpan::new(start, end),
            level_stack: Vec::new(),
        }
    }

    /// Creates a navigator over an in-memory [`DcStr`].
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "In-memory DcStr byte lengths fit within u64 on all supported platforms"
    )]
    pub fn from_dc_str(text: &'a DcStr) -> Self {
        let end = u64::try_from(text.len())
            .expect("DcStr byte length fits in u64");
        Self {
            backend: NavBackend::MemoryText(text),
            cursor: 0,
            active_span: DcByteSpan::new(0, end),
            level_stack: Vec::new(),
        }
    }

    /// Returns the current stream byte offset.
    #[must_use]
    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    /// Creates a streaming and seeking navigator over an on-disk `.dcmm` file
    /// starting at `base_offset` (e.g. `0` for a standalone `.dcmm` file, or `8`
    /// for a `.cscjournal` after its 8-byte journal magic).
    ///
    /// Validates the 21-byte `.dcmm` magic/UUID header once at `base_offset`
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
            "File is too short ({file_len} bytes) to contain a valid 21-byte DcMixed header at offset {base_offset}"
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
        self.level_stack.push((self.active_span, stub.outer_span.end));
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
        let (parent_span, saved_cursor) = self
            .level_stack
            .pop()
            .context("Already at root level of DcDocumentNavigator")?;
        self.active_span = parent_span;
        let next_pos = saved_cursor.min(parent_span.end);
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

    /// Reads a raw byte range from the underlying source and wraps it in a
    /// validated [`DcMstring`].
    pub fn read_span_as_mstring(&mut self, span: DcByteSpan) -> Result<DcMstring> {
        let raw_bytes = self.read_span_bytes(span)?;
        let mut wrapped = Vec::with_capacity(DCMM_HEADER_LEN.saturating_add(raw_bytes.len()));
        wrapped.extend_from_slice(&DCMM_MAGIC);
        wrapped.extend_from_slice(&DCMM_UUID);
        wrapped.extend_from_slice(&raw_bytes);
        Ok(DcMstring::from_bytes(wrapped)?)
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

        // 0. Escape sequence (Dc 255 + escaped character)
        if first_dc == DC_ESCAPE {
            let Some((escaped_dc, after_escaped)) = self.read_token_or_binary_at(after_first)? else {
                policy.handle(
                    policy.syntax_error,
                    format!("Dangling escape Dc 255 at offset {start_offset}"),
                )?;
                self.cursor = after_first;
                return Ok(Some(DcFrameStub {
                    head_dc: first_dc,
                    closer_dc: None,
                    outer_span: DcByteSpan::new(start_offset, after_first),
                    inner_span: None,
                    header_info: DcFrameHeaderInfo::None,
                }));
            };
            self.cursor = after_escaped;
            return Ok(Some(DcFrameStub {
                head_dc: escaped_dc,
                closer_dc: None,
                outer_span: DcByteSpan::new(start_offset, after_escaped),
                inner_span: None,
                header_info: DcFrameHeaderInfo::None,
            }));
        }

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

        let resolver = default_syntax_resolver();

        // 2. Check if this is a literal (260..261) or identifier (270..271)
        if first_dc == DC_BEGIN_LITERAL || first_dc == DC_BEGIN_IDENTIFIER {
            let closer = if first_dc == DC_BEGIN_LITERAL {
                DC_END_LITERAL
            } else {
                DC_END_IDENTIFIER
            };
            let end_offset = self.scan_escaped_text_frame(after_first, closer, policy)?;
            if first_dc == DC_BEGIN_IDENTIFIER {
                if end_offset
                    == after_first.saturating_add(dc_char_utf8e128_len(DC_END_IDENTIFIER))
                {
                    policy.handle(
                        policy.syntax_error,
                        format!("Empty identifier frame (Dc 270..271) at offset {start_offset}"),
                    )?;
                }
                // Check if this identifier is the LHS of an infix rule (e.g. Dc 269 assignment)
                if end_offset < self.active_span.end {
                    if let Ok(Some((next_dc, after_op))) = self.read_token_or_binary_at(end_offset) {
                        if resolver.resolve_infix_rule(next_dc.to_u128()).is_some() {
                            self.cursor = after_op;
                            let Some(rhs_stub) = self.next_sibling_stub(policy)? else {
                                policy.handle(
                                    policy.syntax_error,
                                    format!("Missing value on RHS of infix operator {next_dc:?} at offset {end_offset}"),
                                )?;
                                self.cursor = after_op;
                                return Ok(Some(DcFrameStub {
                                    head_dc: next_dc,
                                    closer_dc: None,
                                    outer_span: DcByteSpan::new(start_offset, after_op),
                                    inner_span: Some(DcByteSpan::new(after_op, after_op)),
                                    header_info: DcFrameHeaderInfo::None,
                                }));
                            };
                            let assign_end = rhs_stub.outer_span.end;
                            self.cursor = assign_end;
                            return Ok(Some(DcFrameStub {
                                head_dc: next_dc,
                                closer_dc: None,
                                outer_span: DcByteSpan::new(start_offset, assign_end),
                                inner_span: Some(DcByteSpan::new(after_op, assign_end)),
                                header_info: DcFrameHeaderInfo::None,
                            }));
                        }
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

        // Reject dangling closing delimiters
        if let Some(opener) = resolver.closing_opener_for(first_dc.to_u128()) {
            policy.handle(
                policy.syntax_error,
                format!(
                    "Unexpected closing delimiter {first_dc:?} (closes {opener}) at offset {start_offset}"
                ),
            )?;
            self.cursor = after_first;
            return Ok(Some(DcFrameStub {
                head_dc: first_dc,
                closer_dc: None,
                outer_span: DcByteSpan::new(start_offset, after_first),
                inner_span: None,
                header_info: DcFrameHeaderInfo::None,
            }));
        }

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

            // 5. Check if `first_dc` has a prefix syntax rule (`:~ ...`)
            if let Some(rule) = resolver.resolve_rule(&target) {
                if rule.pattern.starts_with_self() {
                    if after_first >= self.active_span.end {
                        policy.handle(
                            policy.syntax_error,
                            format!("Prefix operator {first_dc:?} with missing arguments at offset {start_offset}"),
                        )?;
                    }
                    self.cursor = start_offset;
                    let mut ctx = SyntaxEvalContext::new(&**resolver, Some(first_dc.to_u128()));
                    let matched = self.eval_pattern(&rule.pattern, &mut ctx)?;
                    if matched && !ctx.has_errors {
                        let mut last_end = self.cursor;
                        if first_dc == DC_ROUTINE_INVOCATION {
                            last_end = self.scan_trailing_parameter_frames(last_end, policy)?;
                        }
                        self.cursor = last_end;
                        return Ok(Some(DcFrameStub {
                            head_dc: first_dc,
                            closer_dc: None,
                            outer_span: DcByteSpan::new(start_offset, last_end),
                            inner_span: (last_end > after_first)
                                .then(|| DcByteSpan::new(after_first, last_end)),
                            header_info: DcFrameHeaderInfo::None,
                        }));
                    }
                    if matches!(
                        first_dc,
                        DC_EQUIVALENT_REPRESENTATIONS | DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS
                    ) && after_first < self.active_span.end {
                        if let Some((DC_BEGIN_LIST, _)) = self.read_token_or_binary_at(after_first)? {
                            self.cursor = after_first;
                            if let Some(arg_stub) = self.next_sibling_stub(policy)? {
                                return Ok(Some(DcFrameStub {
                                    head_dc: first_dc,
                                    closer_dc: None,
                                    outer_span: DcByteSpan::new(start_offset, arg_stub.outer_span.end),
                                    inner_span: Some(DcByteSpan::new(after_first, arg_stub.outer_span.end)),
                                    header_info: DcFrameHeaderInfo::None,
                                }));
                            }
                        }
                    }
                    if first_dc == DC_OPTIONAL_PRESENT && after_first < self.active_span.end {
                        let mut probe = after_first;
                        let mut depth = 0usize;
                        while probe < self.active_span.end && depth < policy.max_depth {
                            let Some((tok, next_off)) = self.read_token_or_binary_at(probe)? else {
                                break;
                            };
                            if tok == DC_OPTIONAL_PRESENT {
                                probe = next_off;
                                depth = depth.saturating_add(1);
                            } else {
                                break;
                            }
                        }
                        if let Some((tok, _)) = self.read_token_or_binary_at(probe)? {
                            if resolver.closing_opener_for(tok.to_u128()).is_none() {
                                self.cursor = probe;
                                if let Some(arg_stub) = self.next_sibling_stub(policy)? {
                                    let last_end = arg_stub.outer_span.end;
                                    self.cursor = last_end;
                                    return Ok(Some(DcFrameStub {
                                        head_dc: first_dc,
                                        closer_dc: None,
                                        outer_span: DcByteSpan::new(start_offset, last_end),
                                        inner_span: (last_end > after_first)
                                            .then(|| DcByteSpan::new(after_first, last_end)),
                                        header_info: DcFrameHeaderInfo::None,
                                    }));
                                }
                            }
                        }
                    }
                    self.cursor = start_offset;
                }
            }

            // Built-in routine invocation (e.g. Dc 256 `lang.say` in `.EL Routines`)
            if resolver.matches_script(".EL Routines", first_dc.to_u128()) {
                let last_end = self.scan_trailing_parameter_frames(after_first, policy)?;
                self.cursor = last_end;
                return Ok(Some(DcFrameStub {
                    head_dc: first_dc,
                    closer_dc: None,
                    outer_span: DcByteSpan::new(start_offset, last_end),
                    inner_span: (last_end > after_first)
                        .then(|| DcByteSpan::new(after_first, last_end)),
                    header_info: DcFrameHeaderInfo::None,
                }));
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

    fn scan_trailing_parameter_frames(
        &mut self,
        mut offset: u64,
        policy: &DcProjectionPolicy,
    ) -> Result<u64> {
        while let Some((next_dc, after_open)) = self.read_token_or_binary_at(offset)? {
            if next_dc != DC_BEGIN_PARAMETER {
                break;
            }
            let (_, after_close) = self.scan_balanced_container(
                after_open,
                DC_BEGIN_PARAMETER,
                DC_END_PARAMETER,
                policy,
            )?;
            offset = after_close;
        }
        Ok(offset)
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`] and [`DcProjectionPolicy`], reusing the provided
    /// [`DcSyntaxArena`].
    pub fn decode_frame_with_arena<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
    ) -> Result<T> {
        let root_id = self.expand_frame(stub, arena, policy)?;
        let mut ctx = DcProjectionContext::with_shared_context_and_resolver(
            arena,
            policy,
            context,
            Some(self),
        );
        T::decode_dc_node(&mut ctx, root_id)
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`] and [`DcProjectionPolicy`].
    pub fn decode_frame<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
    ) -> Result<T> {
        let mut arena = DcSyntaxArena::new();
        self.decode_frame_with_arena(stub, &mut arena, context, policy)
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

    /// Parses all sibling constructs in the active span into `arena`.
    pub fn parse_document_into_arena(
        &mut self,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        while let Some(stub) = self.next_sibling_stub(policy)? {
            let id = self.build_arena_node_from_stub(&stub, arena, policy)?;
            arena.push_root(id);
        }
        Ok(())
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

    /// Parses a single root node from `bytes` into `arena` using `policy`.
    pub fn parse_arena(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        let mut nav = if bytes.starts_with(&DCMM_MAGIC) {
            let mstr = DcMstr::from_bytes(bytes)?;
            Self::from_mstr(mstr)
        } else {
            let text = DcStr::from_bytes(bytes)?;
            Self::from_dc_str(text)
        };
        nav.parse_node_into_arena(arena, policy)
    }

    /// Parses all document root nodes from `bytes` into `arena` using `policy`.
    pub fn parse_document_arena(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        let mut nav = if bytes.starts_with(&DCMM_MAGIC) {
            let mstr = DcMstr::from_bytes(bytes)?;
            Self::from_mstr(mstr)
        } else {
            let text = DcStr::from_bytes(bytes)?;
            Self::from_dc_str(text)
        };
        nav.parse_document_into_arena(arena, policy)
    }

    fn build_arena_node_from_stub(
        &mut self,
        root_stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        enum ArenaBuildFrame {
            Assignment {
                outer_span: DcByteSpan,
                ident: String,
                value: Option<NodeId>,
            },
            Container {
                outer_span: DcByteSpan,
                opener: DcChar,
                closer: DcChar,
                unclosed: bool,
                children: Vec<NodeId>,
            },
            Tagged {
                outer_span: DcByteSpan,
                tag: DcChar,
                args: Vec<NodeId>,
            },
        }

        enum StubStep {
            Leaf(NodeId),
            Push(ArenaBuildFrame),
        }

        fn validate_equivalent_list_children(
            arena: &DcSyntaxArena<'_>,
            children: &[NodeId],
            policy: &DcProjectionPolicy,
        ) -> Result<()> {
            if policy.is_strict() {
                let has_type_header = children.first().and_then(|&id| arena.get(id)).is_some_and(|node| {
                    matches!(node.kind, DcSyntaxKind::Atom(ch) if ch == DC_TYPE_ANY || ctb_formats_dcdata::is_type_token(ch.0))
                });
                if !has_type_header {
                    policy.handle(
                        policy.syntax_error,
                        "Untyped equivalent representations list: missing type header",
                    )?;
                }
            }
            let mut has_primary = false;
            let mut prev_was_sep = true;
            for &child_id in children {
                let Some(child) = arena.get(child_id) else {
                    continue;
                };
                if let DcSyntaxKind::Atom(ch) = child.kind {
                    if ch == DC_TYPE_ANY || ctb_formats_dcdata::is_type_token(ch.0) {
                        continue;
                    }
                    if ch == DC_LIST_ELEMENT_SEPARATOR {
                        prev_was_sep = true;
                        continue;
                    }
                }
                if !prev_was_sep && has_primary {
                    policy.handle(
                        policy.syntax_error,
                        "Malformed equivalent representations list: missing element separator (Dc 312)",
                    )?;
                }
                prev_was_sep = false;
                has_primary = true;
            }
            if policy.is_strict() && prev_was_sep && has_primary {
                policy.handle(
                    policy.syntax_error,
                    "Malformed equivalent representations list: trailing element separator (Dc 312)",
                )?;
            }
            Ok(())
        }

        fn validate_completed_container(
            arena: &DcSyntaxArena<'_>,
            opener: DcChar,
            children: &[NodeId],
            policy: &DcProjectionPolicy,
        ) -> Result<()> {
            if opener == DC_BEGIN_LIST {
                if let (Some(&first_id), Some(&second_id)) = (children.first(), children.get(1)) {
                    if let (
                        Some(DcSyntaxNode {
                            kind: DcSyntaxKind::Number { digits, .. },
                            ..
                        }),
                        Some(DcSyntaxNode {
                            kind: DcSyntaxKind::Atom(ch),
                            ..
                        }),
                    ) = (arena.get(first_id), arena.get(second_id))
                    {
                        if ctb_formats_dcdata::is_type_token(ch.0) {
                            if let Ok(int_val) = crate::dc_number::parse_dc_number_digits(digits) {
                                if let Ok(expected) = usize::try_from(&int_val) {
                                    let actual = children
                                        .get(2..)
                                        .into_iter()
                                        .flatten()
                                        .filter(|&&id| {
                                            !matches!(
                                                arena.get(id),
                                                Some(DcSyntaxNode {
                                                    kind: DcSyntaxKind::Atom(c),
                                                    ..
                                                }) if *c == DC_LIST_ELEMENT_SEPARATOR
                                            )
                                        })
                                        .count();
                                    if actual != expected {
                                        policy.handle(
                                            policy.syntax_error,
                                            format!(
                                                "List length mismatch: header declared {expected} elements, found {actual}"
                                            ),
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }
            } else if matches!(
                opener,
                DC_EQUIVALENT_REPRESENTATIONS | DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS
            ) {
                validate_equivalent_list_children(arena, children, policy)?;
            } else if opener == DC_BEGIN_FORMAT_REPRESENTATION {
                let mut max_expected_children = 2;
                if let Some(&first_id) = children.first() {
                    if let Some(first_node) = arena.get(first_id) {
                        let maybe_fid = match &first_node.kind {
                            DcSyntaxKind::Atom(ch) => FormatId::from_dc_id(ch.to_u128()),
                            DcSyntaxKind::Identifier { name } => {
                                FormatId::from_shorthand(name)
                                    .or_else(|| FormatId::from_ident(name))
                            }
                            _ => None,
                        };
                        if let Some(fid) = maybe_fid {
                            if let Some(rule) = SyntaxRuntimeRegistry::global()
                                .resolver()
                                .get_format_rule(fid)
                            {
                                if let ctb_formats_dcdata::DcSyntaxExpr::Sequence(items) =
                                    &rule.expr
                                {
                                    max_expected_children =
                                        items.len().saturating_add(1);
                                }
                            }
                        }
                    }
                }
                if children.len() > max_expected_children {
                    policy.handle(
                        policy.syntax_error,
                        format!(
                            "Format representation container (Dc 511) has extraneous trailing children: expected {max_expected_children}, found {}",
                            children.len()
                        ),
                    )?;
                }
            }
            Ok(())
        }

        fn validate_completed_tagged(
            arena: &DcSyntaxArena<'_>,
            tag: DcChar,
            args: &[NodeId],
            policy: &DcProjectionPolicy,
        ) -> Result<()> {
            if matches!(
                tag,
                DC_EQUIVALENT_REPRESENTATIONS | DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS
            ) {
                if let Some(&arg_id) = args.first() {
                    if let Some(DcSyntaxNode {
                        kind: DcSyntaxKind::Container { children, opener, .. },
                        ..
                    }) = arena.get(arg_id)
                    {
                        if *opener != DC_BEGIN_LIST && policy.is_strict() {
                            policy.handle(
                                policy.syntax_error,
                                "Equivalent representations (Dc 397/561) must be enclosed in a list container (Dc 310)",
                            )?;
                        }
                        validate_equivalent_list_children(arena, children, policy)?;
                    } else if policy.is_strict() {
                        policy.handle(
                            policy.syntax_error,
                            "Equivalent representations (Dc 397/561) must be enclosed in a list container (Dc 310)",
                        )?;
                    }
                }
            }
            Ok(())
        }

        let resolver = default_syntax_resolver();
        let base_level_depth = self.level_stack.len();

        let classify_stub = |nav: &mut Self,
                             stub: &DcFrameStub,
                             current_depth: usize,
                             arena: &mut DcSyntaxArena<'a>|
         -> Result<StubStep> {
            let head = stub.head_dc;
            if head == DC_START_ENCAPSULATION_BINARY {
                let node = nav.parse_binary_arena_node(stub.outer_span)?;
                return Ok(StubStep::Leaf(arena.alloc(stub.outer_span, node)));
            }
            if head == DC_BEGIN_LITERAL {
                let (type_header, decoded) = nav.decode_literal_span(stub.outer_span, policy)?;
                return Ok(StubStep::Leaf(arena.alloc(
                    stub.outer_span,
                    DcSyntaxKind::Literal {
                        type_header,
                        decoded,
                    },
                )));
            }
            if head == DC_BEGIN_IDENTIFIER {
                let name = nav.decode_identifier_span(stub.outer_span, policy)?;
                return Ok(StubStep::Leaf(
                    arena.alloc(stub.outer_span, DcSyntaxKind::Identifier { name }),
                ));
            }
            if head == DC_BEGIN_NUMBER {
                let (format, digits) = nav.decode_number_span(stub.outer_span)?;
                return Ok(StubStep::Leaf(
                    arena.alloc(stub.outer_span, DcSyntaxKind::Number { format, digits }),
                ));
            }
            if head == DC_ASSIGN {
                if let Some(inner) = stub.inner_span {
                    if current_depth >= policy.max_depth {
                        let msg = format!(
                            "Document nesting depth exceeds maximum limit ({}) at offset {}",
                            policy.max_depth, stub.outer_span.start
                        );
                        policy.handle(policy.depth_limit_exceeded, &msg)?;
                        arena.push_diagnostic(SyntaxDiagnostic {
                            message: msg.clone(),
                            token_offset: stub.outer_span.start,
                            is_error: true,
                        });
                        return Ok(StubStep::Leaf(arena.alloc(
                            stub.outer_span,
                            DcSyntaxKind::Malformed {
                                raw_tokens: Vec::new(),
                                message: msg,
                            },
                        )));
                    }
                    let ident_span = DcByteSpan::new(
                        stub.outer_span.start,
                        inner.start.saturating_sub(dc_char_utf8e128_len(DC_ASSIGN)),
                    );
                    let ident = nav.decode_identifier_span(ident_span, policy)?;
                    nav.enter_frame(stub)?;
                    return Ok(StubStep::Push(ArenaBuildFrame::Assignment {
                        outer_span: stub.outer_span,
                        ident,
                        value: None,
                    }));
                }
            }
            if let Some(closer) = stub.closer_dc {
                if head == DC_BEGIN_LEGACY_KV_MAP {
                    policy.handle(
                        policy.deprecated_syntax,
                        "Deprecated Dc 313..314 key-value map syntax; prefer Dc 559..560",
                    )?;
                }
                let unclosed = stub.is_unclosed();
                if stub.inner_span.is_some() && !resolver.is_opaque_container(head.to_u128()) {
                    if current_depth >= policy.max_depth {
                        let msg = format!(
                            "Document nesting depth exceeds maximum limit ({}) at offset {}",
                            policy.max_depth, stub.outer_span.start
                        );
                        policy.handle(policy.depth_limit_exceeded, &msg)?;
                        arena.push_diagnostic(SyntaxDiagnostic {
                            message: msg.clone(),
                            token_offset: stub.outer_span.start,
                            is_error: true,
                        });
                        return Ok(StubStep::Leaf(arena.alloc(
                            stub.outer_span,
                            DcSyntaxKind::Malformed {
                                raw_tokens: Vec::new(),
                                message: msg,
                            },
                        )));
                    }
                    nav.enter_frame(stub)?;
                    return Ok(StubStep::Push(ArenaBuildFrame::Container {
                        outer_span: stub.outer_span,
                        opener: head,
                        closer,
                        unclosed,
                        children: Vec::new(),
                    }));
                }
                return Ok(StubStep::Leaf(arena.alloc(
                    stub.outer_span,
                    DcSyntaxKind::Container {
                        opener: head,
                        closer,
                        header: Vec::new(),
                        children: Vec::new(),
                        trailing: Vec::new(),
                        unclosed,
                    },
                )));
            }
            if stub.inner_span.is_some() {
                if current_depth >= policy.max_depth {
                    let msg = format!(
                        "Document nesting depth exceeds maximum limit ({}) at offset {}",
                        policy.max_depth, stub.outer_span.start
                    );
                    policy.handle(policy.depth_limit_exceeded, &msg)?;
                    arena.push_diagnostic(SyntaxDiagnostic {
                        message: msg.clone(),
                        token_offset: stub.outer_span.start,
                        is_error: true,
                    });
                    return Ok(StubStep::Leaf(arena.alloc(
                        stub.outer_span,
                        DcSyntaxKind::Malformed {
                            raw_tokens: Vec::new(),
                            message: msg,
                        },
                    )));
                }
                nav.enter_frame(stub)?;
                return Ok(StubStep::Push(ArenaBuildFrame::Tagged {
                    outer_span: stub.outer_span,
                    tag: head,
                    args: Vec::new(),
                }));
            }
            Ok(StubStep::Leaf(
                arena.alloc(stub.outer_span, DcSyntaxKind::Atom(head)),
            ))
        };

        let mut work_stack: Vec<ArenaBuildFrame> = Vec::new();
        match classify_stub(self, root_stub, 0, arena)? {
            StubStep::Leaf(id) => return Ok(id),
            StubStep::Push(frame) => work_stack.push(frame),
        }

        let res = (|| -> Result<NodeId> {
            loop {
                let current_depth = work_stack.len();
                let need_assignment_rhs = matches!(
                    work_stack.last(),
                    Some(ArenaBuildFrame::Assignment { value: None, .. })
                );
                let finished_assignment = matches!(
                    work_stack.last(),
                    Some(ArenaBuildFrame::Assignment { value: Some(_), .. })
                );

                if finished_assignment {
                    let Some(ArenaBuildFrame::Assignment {
                        outer_span,
                        ident,
                        value: Some(val_id),
                    }) = work_stack.pop()
                    else {
                        unreachable!()
                    };
                    self.exit_frame()?;
                    let node_id = arena.alloc(
                        outer_span,
                        DcSyntaxKind::Assignment {
                            ident,
                            value: val_id,
                        },
                    );
                    if let Some(parent) = work_stack.last_mut() {
                        match parent {
                            ArenaBuildFrame::Assignment { value, .. } => *value = Some(node_id),
                            ArenaBuildFrame::Container { children, .. } => children.push(node_id),
                            ArenaBuildFrame::Tagged { args, .. } => args.push(node_id),
                        }
                        continue;
                    }
                    return Ok(node_id);
                }

                let start = self.cursor;
                if let Some(child_stub) = self.next_sibling_stub(policy)? {
                    match classify_stub(self, &child_stub, current_depth, arena)? {
                        StubStep::Leaf(child_id) => {
                            if let Some(top) = work_stack.last_mut() {
                                match top {
                                    ArenaBuildFrame::Assignment { value, .. } => {
                                        *value = Some(child_id);
                                    }
                                    ArenaBuildFrame::Container { children, .. } => {
                                        children.push(child_id);
                                    }
                                    ArenaBuildFrame::Tagged { args, .. } => {
                                        args.push(child_id);
                                    }
                                }
                            }
                        }
                        StubStep::Push(child_frame) => {
                            work_stack.push(child_frame);
                        }
                    }
                } else {
                    if need_assignment_rhs {
                        bail!("Unexpected end of stream at offset {start}");
                    }
                    let Some(finished_frame) = work_stack.pop() else {
                        bail!("Internal error: empty arena work stack");
                    };
                    self.exit_frame()?;
                    let node_id = match finished_frame {
                        ArenaBuildFrame::Container {
                            outer_span,
                            opener,
                            closer,
                            unclosed,
                            children,
                        } => {
                            validate_completed_container(arena, opener, &children, policy)?;
                            arena.alloc(
                                outer_span,
                                DcSyntaxKind::Container {
                                    opener,
                                    closer,
                                    header: Vec::new(),
                                    children,
                                    trailing: Vec::new(),
                                    unclosed,
                                },
                            )
                        }
                        ArenaBuildFrame::Tagged {
                            outer_span,
                            tag,
                            args,
                        } => {
                            validate_completed_tagged(arena, tag, &args, policy)?;
                            arena.alloc(outer_span, DcSyntaxKind::Tagged { tag, args })
                        }
                        ArenaBuildFrame::Assignment { .. } => unreachable!(),
                    };
                    if let Some(parent) = work_stack.last_mut() {
                        match parent {
                            ArenaBuildFrame::Assignment { value, .. } => *value = Some(node_id),
                            ArenaBuildFrame::Container { children, .. } => children.push(node_id),
                            ArenaBuildFrame::Tagged { args, .. } => args.push(node_id),
                        }
                    } else {
                        return Ok(node_id);
                    }
                }
            }
        })();

        while self.level_stack.len() > base_level_depth {
            let _ = self.exit_frame();
        }
        res
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

    fn try_scan_raw_binary_at(
        &mut self,
        start_offset: u64,
    ) -> Result<Option<(u64, DcFrameHeaderInfo)>> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let start_usize = usize::try_from(start_offset)?;
                let rem = mstr
                    .as_bytes()
                    .get(start_usize..)
                    .context("Binary encapsulation start out of bounds")?;
                let mut chunks = crate::dc_mixed::DcMixedChunks { slice: rem };
                let Some(chunk) = chunks.next() else {
                    return Ok(None);
                };
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
                        Ok(Some((
                            end_offset,
                            DcFrameHeaderInfo::Binary {
                                data_type: None,
                                payload_span: Some(DcByteSpan::new(abs_data_start, end_offset)),
                                payload_len: data_len,
                            },
                        )))
                    }
                    crate::DcMixedChunk::BinaryWithMetadata { data, metadata } => {
                        let data_len = u64::try_from(data.len())?;
                        let abs_data_start = end_offset.saturating_sub(data_len);
                        let data_type = crate::encapsulation::parse_metadata(metadata)?
                            .data_type
                            .map(DcStr::to_owned);
                        Ok(Some((
                            end_offset,
                            DcFrameHeaderInfo::Binary {
                                data_type,
                                payload_span: Some(DcByteSpan::new(abs_data_start, end_offset)),
                                payload_len: data_len,
                            },
                        )))
                    }
                    crate::DcMixedChunk::Text(_) => Ok(None),
                }
            }
            NavBackend::MemoryText(_) => Ok(None),
            NavBackend::SeekableFile {
                reader,
                base_offset,
                ..
            } => {
                let after_dc203 = start_offset
                    .checked_add(u64::try_from(crate::dc_mixed::DC_203_BYTES.len())?)
                    .context("Binary encapsulation offset overflow")?;
                reader.seek(SeekFrom::Start(after_dc203))?;
                let mut type_byte = [0_u8; 1];
                reader
                    .read_exact(&mut type_byte)
                    .context("Truncated .dcmm binary encapsulation type")?;
                let encap_type = type_byte[0];
                let rel_offset = usize::try_from(after_dc203.saturating_sub(*base_offset))?;
                if matches!(encap_type, 1 | 4) {
                    return Err(crate::DcMixedError::UnsupportedLegacyType {
                        offset: rel_offset,
                        type_byte: encap_type,
                    }
                    .into());
                }
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
                        return Err(crate::DcMixedError::InvalidEncapsulationType {
                            offset: rel_offset,
                            type_byte: encap_type,
                        }
                        .into());
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
                reader.seek(SeekFrom::Start(payload_end))?;
                Ok(Some((
                    payload_end,
                    DcFrameHeaderInfo::Binary {
                        data_type,
                        payload_span: Some(DcByteSpan::new(payload_start, payload_end)),
                        payload_len: data_len,
                    },
                )))
            }
        }
    }

    fn scan_binary_at(&mut self, start_offset: u64) -> Result<(u64, DcFrameHeaderInfo)> {
        if let Some(res) = self.try_scan_raw_binary_at(start_offset)? {
            return Ok(res);
        }
        self.scan_textual_binary_at(start_offset)
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
        if closer == DC_END_LITERAL {
            if let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
                if ch == DC_BEGIN_TYPE_DEF {
                    offset = self.scan_until_token(next_off, DC_END_TYPE_DEF, policy)?;
                } else {
                    policy.handle(
                        policy.syntax_error,
                        format!("Literal at offset {after_opener} is missing Dc 262..263 type header"),
                    )?;
                }
            } else {
                policy.handle(
                    policy.syntax_error,
                    format!("Unclosed literal at offset {after_opener}: missing Dc 262..263 type header"),
                )?;
                return Ok(offset);
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

        while let Some((ch, next_offset)) = self.read_token_or_binary_at(offset)? {
            let token_start = offset;
            if ch == DC_ESCAPE {
                if let Some((_, after_escaped)) = self.read_token_or_binary_at(next_offset)? {
                    offset = after_escaped;
                    continue;
                }
                policy.handle(
                    policy.syntax_error,
                    format!("Dangling escape Dc 255 at offset {token_start}"),
                )?;
                return Ok((next_offset, next_offset));
            }
            if ch == DC_START_ENCAPSULATION_BINARY {
                let (after_bin, _) = self.scan_binary_at(token_start)?;
                offset = after_bin;
                continue;
            }
            if ch == DC_BEGIN_LITERAL {
                offset = self.scan_escaped_text_frame(next_offset, DC_END_LITERAL, policy)?;
                continue;
            }
            if ch == DC_BEGIN_IDENTIFIER {
                offset = self.scan_escaped_text_frame(next_offset, DC_END_IDENTIFIER, policy)?;
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
                            if let Ok((_, digits)) = self.decode_number_span(DcByteSpan::new(after_tag, num_end)) {
                                if let Ok(int_val) = crate::dc_number::parse_dc_number_digits(&digits) {
                                    sequence_id = u64::try_from(&int_val).ok();
                                }
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
        let resolver = default_syntax_resolver();
        self.cursor = after_digest_tag;
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        let checksum_term = SyntaxTerm::NamedConstruct {
            name: "checksum".to_owned(),
            subtype: None,
            capture_var: None,
        };
        let checksum_ok = self.eval_term_single(&checksum_term, &mut ctx)?;
        if !checksum_ok {
            policy.handle(
                policy.syntax_error,
                format!("Invalid [checksum] after Dc 545 at offset {after_digest_tag}"),
            )?;
        }
        let (final_end, bin_header) = match self.scan_binary_at(after_digest_tag) {
            Ok(res) => res,
            Err(_) => return Ok((after_digest_tag, sequence_id, false)),
        };
        let expected_digest = match bin_header {
            DcFrameHeaderInfo::Binary {
                payload_span: Some(pspan),
                ..
            } => self.read_span_bytes(pspan)?.into_owned(),
            _ => match self.parse_binary_arena_node(DcByteSpan::new(after_digest_tag, final_end)) {
                Ok(DcSyntaxKind::Binary {
                    payload: BinaryPayloadRef::Borrowed(b),
                    ..
                }) => b.to_vec(),
                Ok(DcSyntaxKind::Binary {
                    payload: BinaryPayloadRef::Inline(b),
                    ..
                }) => b,
                _ => Vec::new(),
            },
        };
        let actual_digest = self.sha256_of_span(body_span)?;
        let valid = checksum_ok && actual_digest.as_slice() == expected_digest.as_slice();
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
                let Some(chunk) = chunks.next() else {
                    let sub = DcStr::from_bytes(slice)?;
                    let (header, data, _) = crate::encapsulation::parse_text(sub)?;
                    let cf = header.chars().next();
                    return Ok(DcSyntaxKind::Binary {
                        data_type: if header.is_empty() { None } else { Some(header) },
                        checksum_format: cf,
                        payload: BinaryPayloadRef::Inline(data),
                    });
                };
                match chunk {
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
        ensure!(first == DC_BEGIN_LITERAL, "Expected Dc 260");
        offset = after_first;
        let mut type_header = Vec::new();
        if let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
            if ch == DC_BEGIN_TYPE_DEF {
                offset = next_off;
                while offset < span.end {
                    let Some((tch, after_tch)) = self.read_token_or_binary_at(offset)? else {
                        break;
                    };
                    offset = after_tch;
                    if tch == DC_END_TYPE_DEF {
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
            if ch == DC_END_LITERAL {
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
        ensure!(first == DC_BEGIN_IDENTIFIER, "Expected Dc 270");
        offset = after_first;
        let mut name = String::new();
        while offset < span.end {
            let Some((mut ch, next_off)) = self.read_token_or_binary_at(offset)? else {
                break;
            };
            offset = next_off;
            if ch == DC_END_IDENTIFIER {
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

    fn read_tokens_in_span(&mut self, span: DcByteSpan) -> Result<Vec<u128>> {
        let saved_span = self.active_span;
        self.active_span = span;
        let mut offset = span.start;
        let mut tokens = Vec::new();
        while offset < span.end {
            let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? else {
                break;
            };
            tokens.push(ch.to_u128());
            offset = next_off;
        }
        self.active_span = saved_span;
        Ok(tokens)
    }

    /// Evaluates a [`SyntaxPattern`] starting at the navigator's current cursor.
    ///
    /// If the pattern matches, advances `self.cursor` past the matched span and
    /// returns `Ok(true)`. If the pattern does not match, rewinds `self.cursor`
    /// to its starting offset and returns `Ok(false)`.
    pub fn eval_pattern(
        &mut self,
        pattern: &SyntaxPattern,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_pos = self.cursor;
        match pattern {
            SyntaxPattern::Alternation(branches) => {
                let mut zero_match: Option<SyntaxEvalContext<'_>> = None;
                for branch in branches {
                    self.cursor = start_pos;
                    let mut trial_ctx = ctx.clone();
                    if self.eval_pattern(branch, &mut trial_ctx)? && !trial_ctx.has_errors {
                        if self.cursor > start_pos {
                            *ctx = trial_ctx;
                            return Ok(true);
                        } else if zero_match.is_none() {
                            zero_match = Some(trial_ctx);
                        }
                    }
                }
                if let Some(trial_ctx) = zero_match {
                    self.cursor = start_pos;
                    *ctx = trial_ctx;
                    return Ok(true);
                }
                self.cursor = start_pos;
                Ok(false)
            }
            SyntaxPattern::Sequence(elements) => {
                for elem in elements {
                    if self.cursor >= self.active_span.end {
                        if elem.quantifier.allows_zero() {
                            continue;
                        }
                        self.cursor = start_pos;
                        return Ok(false);
                    }
                    let matched = self.eval_element(elem, ctx)?;
                    if !matched {
                        self.cursor = start_pos;
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }

    /// Evaluates a quantified [`SyntaxElement`] starting at the current cursor.
    pub fn eval_element(
        &mut self,
        element: &SyntaxElement,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let elem_start = self.cursor;
        let mut count = 0usize;
        while self.cursor < self.active_span.end {
            let iter_start = self.cursor;
            let matched = self.eval_term_single(&element.term, ctx)?;
            if matched && !ctx.has_errors {
                let consumed = self.cursor.saturating_sub(iter_start);
                count = count.saturating_add(1);
                if consumed == 0 || element.quantifier.reached_max(count) {
                    break;
                }
            } else {
                self.cursor = iter_start;
                if ctx.has_errors {
                    return Ok(false);
                }
                break;
            }
        }
        let ok = match element.quantifier {
            Quantifier::ExactOne => count == 1,
            Quantifier::Optional | Quantifier::ZeroOrMore => true,
            Quantifier::OneOrMore => count >= 1,
            Quantifier::Range { min, max } => {
                count >= min && max.is_none_or(|m| count <= m)
            }
        };
        if !ok {
            self.cursor = elem_start;
        }
        Ok(ok)
    }

    /// Evaluates a single [`SyntaxTerm`] occurrence starting at the current cursor.
    pub fn eval_term_single(
        &mut self,
        term: &SyntaxTerm,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, after_first)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        let first = first_dc.to_u128();

        // 1. Equivalent (Dc 397) and partially equivalent (Dc 561) representations wrapper
        let is_equiv_wrapper = first_dc == DC_EQUIVALENT_REPRESENTATIONS
            || first_dc == DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS;
        let term_targets_equiv = match term {
            SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
            SyntaxTerm::CharRef(t) | SyntaxTerm::RuleRef { target: t } => {
                t.to_u128() == DC_EQUIVALENT_REPRESENTATIONS.to_u128()
                    || t.to_u128() == DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS.to_u128()
            }
            _ => false,
        };
        if is_equiv_wrapper
            && !matches!(term, SyntaxTerm::Group(pattern) if pattern_has_payload_constraint(pattern))
            && !term_targets_equiv
        {
            if let Some(wrapper_target) = CharTarget::from_u128(first) {
                if let Some(rule) = ctx.resolver.resolve_rule(&wrapper_target) {
                    let old_self = ctx.self_dc;
                    ctx.self_dc = Some(first);
                    let matched = self.eval_pattern(&rule.pattern, ctx)?;
                    ctx.self_dc = old_self;
                    if matched {
                        let wrapper_end = self.cursor;
                        if matches!(
                            term,
                            SyntaxTerm::NamedConstruct { name, subtype: Some(_), .. }
                                if name == "data" || name == "string"
                        ) {
                            if !self.eval_constrained_equivalent_list(
                                after_first,
                                wrapper_end,
                                term,
                                ctx,
                            )? {
                                self.cursor = start_offset;
                                return Ok(false);
                            }
                        }
                        self.cursor = wrapper_end;
                        return Ok(true);
                    }
                    self.cursor = start_offset;
                }
            }
        }

        // 2. Optional value wrappers: Dc 315 (present) and Dc 316 (absent)
        let term_targets_opt_present = match term {
            SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
            SyntaxTerm::CharRef(t) | SyntaxTerm::RuleRef { target: t } => {
                t.to_u128() == DC_OPTIONAL_PRESENT.to_u128()
            }
            _ => false,
        };
        if first_dc == DC_OPTIONAL_PRESENT && !term_targets_opt_present {
            self.cursor = after_first;
            if self.eval_term_single(term, ctx)? {
                return Ok(true);
            }
            self.cursor = start_offset;
        }
        let term_targets_opt_absent = match term {
            SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
            SyntaxTerm::CharRef(t) | SyntaxTerm::RuleRef { target: t } => {
                t.to_u128() == DC_OPTIONAL_ABSENT.to_u128()
            }
            _ => false,
        };
        if first_dc == DC_OPTIONAL_ABSENT && !term_targets_opt_absent {
            self.cursor = after_first;
            return Ok(true);
        }

        match term {
            SyntaxTerm::SelfChar => {
                if let Some(self_id) = ctx.self_dc {
                    if first == self_id {
                        self.cursor = after_first;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    self.cursor = after_first;
                    Ok(true)
                }
            }
            SyntaxTerm::CharRef(target) => {
                if first == target.to_u128() {
                    self.cursor = after_first;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            SyntaxTerm::CharRange { start, end } => {
                if (start.to_u128()..=end.to_u128()).contains(&first) {
                    self.cursor = after_first;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            SyntaxTerm::CharSet { negated, members } => {
                let in_set = members.iter().any(|m| m.to_u128() == first);
                if !*negated {
                    if in_set {
                        self.cursor = after_first;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    if in_set {
                        return Ok(false);
                    }
                    if first_dc == DC_ESCAPE {
                        let Some((_, after_esc)) = self.read_token_or_binary_at(after_first)? else {
                            ctx.has_errors = true;
                            ctx.diagnostics.push(SyntaxDiagnostic {
                                message: format!("Dangling escape Dc 255 at offset {start_offset}"),
                                token_offset: start_offset,
                                is_error: true,
                            });
                            return Ok(false);
                        };
                        self.cursor = after_esc;
                        return Ok(true);
                    }
                    let in_opaque = ctx
                        .self_dc
                        .is_some_and(|s| ctx.resolver.is_opaque_container(s));
                    if !in_opaque {
                        if first_dc == DC_START_ENCAPSULATION_BINARY {
                            if let Ok((after_bin, _)) = self.scan_binary_at(start_offset) {
                                self.cursor = after_bin;
                                return Ok(true);
                            }
                            return Ok(false);
                        }
                        if first_dc == DC_BEGIN_LITERAL {
                            self.cursor = start_offset;
                            return self.eval_literal_frame(None, ctx);
                        }
                        if first_dc == DC_BEGIN_IDENTIFIER {
                            self.cursor = start_offset;
                            return self.eval_identifier_frame(ctx);
                        }
                        if first_dc == DC_BEGIN_NUMBER {
                            self.cursor = start_offset;
                            return self.eval_number_frame(ctx);
                        }
                        if let Some(target) = CharTarget::from_u128(first) {
                            if let Some(closer_u128) = ctx.resolver.closer_for_opener(&target) {
                                let closer_dc = DcChar::from_u128(closer_u128);
                                if let Ok((_, after_closer)) = self.scan_balanced_container(
                                    after_first,
                                    first_dc,
                                    closer_dc,
                                    &DcProjectionPolicy::strict(),
                                ) {
                                    self.cursor = after_closer;
                                    return Ok(true);
                                }
                                return Ok(false);
                            }
                        }
                    }
                    self.cursor = after_first;
                    Ok(true)
                }
            }
            SyntaxTerm::Group(group_pat) => self.eval_pattern(group_pat, ctx),
            SyntaxTerm::NamedConstruct {
                name,
                subtype,
                ..
            } => {
                if name == "data" {
                    return self.eval_data_construct(subtype.as_deref(), ctx);
                }
                if name == "typed_data" {
                    return self.eval_typed_data_construct(subtype.as_deref(), ctx);
                }
                if name == "script" {
                    if let Some(sub) = subtype {
                        if ctx.resolver.matches_script(sub, first) {
                            self.cursor = after_first;
                            return Ok(true);
                        }
                    }
                    return Ok(false);
                }
                if (name == "format" || name == "formats" || name == "type") && subtype.is_some() {
                    if let Some(sub) = subtype {
                        let script_name = format!("Formats:{sub}");
                        if ctx.resolver.matches_script(&script_name, first)
                            || ctx.resolver.matches_script(sub, first)
                        {
                            self.cursor = after_first;
                            return Ok(true);
                        }
                        if let Some(fmt_id) = FormatId::from_shorthand(sub)
                            .or_else(|| FormatId::from_ident(sub))
                        {
                            if fmt_id.dc_id() == Some(first) {
                                self.cursor = after_first;
                                return Ok(true);
                            }
                        }
                        if name == "type" && first_dc == DC_BEGIN_TYPE_DEF {
                            if let Some((inner_dc, after_inner)) =
                                self.read_token_or_binary_at(after_first)?
                            {
                                if let Some((close_dc, after_close)) =
                                    self.read_token_or_binary_at(after_inner)?
                                {
                                    if close_dc == DC_END_TYPE_DEF {
                                        let inner_u128 = inner_dc.to_u128();
                                        let fmt_match = FormatId::from_shorthand(sub)
                                            .or_else(|| FormatId::from_ident(sub))
                                            .and_then(|f| f.dc_id())
                                            == Some(inner_u128);
                                        if fmt_match
                                            || ctx.resolver.matches_script(&script_name, inner_u128)
                                            || ctx.resolver.matches_script(sub, inner_u128)
                                        {
                                            self.cursor = after_close;
                                            return Ok(true);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return Ok(false);
                }
                if name == "type" && subtype.is_none() {
                    if first_dc == DC_BEGIN_TYPE_DEF {
                        if let Some(target) = CharTarget::from_u128(first) {
                            if let Some(rule) = ctx.resolver.resolve_rule(&target) {
                                let old_self = ctx.self_dc;
                                ctx.self_dc = Some(first);
                                let matched = self.eval_pattern(&rule.pattern, ctx)?;
                                ctx.self_dc = old_self;
                                if matched {
                                    return Ok(true);
                                }
                                self.cursor = start_offset;
                            }
                        }
                    }
                    if first_dc == DC_TYPE_INTERSECTION || first_dc == DC_TYPE_UNION {
                        if let Some(target) = CharTarget::from_u128(first) {
                            if let Some(rule) = ctx.resolver.resolve_rule(&target) {
                                let old_self = ctx.self_dc;
                                ctx.self_dc = Some(first);
                                let matched = self.eval_pattern(&rule.pattern, ctx)?;
                                ctx.self_dc = old_self;
                                if matched {
                                    return Ok(true);
                                }
                                self.cursor = start_offset;
                            }
                        }
                    }
                    let is_format_token =
                        (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                            ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                            .contains(&first);
                    if ctb_formats_dcdata::is_type_token(first)
                        || is_format_token
                        || ctx.resolver.matches_script(".EL Types", first)
                        || ctx.resolver.matches_script("Formats", first)
                    {
                        self.cursor = after_first;
                        return Ok(true);
                    }
                    return Ok(false);
                }
                if (name == "format" || name == "formats") && subtype.is_none() {
                    let is_format_token =
                        (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                            ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                            .contains(&first);
                    if is_format_token || ctx.resolver.matches_script("Formats", first) {
                        self.cursor = after_first;
                        return Ok(true);
                    }
                    return Ok(false);
                }
                if name == "string" {
                    return self.eval_literal_frame(subtype.as_deref(), ctx);
                }
                if name == "identifier" {
                    return self.eval_identifier_frame(ctx);
                }
                if ctx.depth >= ctx.max_depth {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Syntax expansion depth limit reached ({}) for named construct [{name}]",
                            ctx.max_depth
                        ),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    return Ok(false);
                }
                let frame_key = format!("named:{name}");
                if ctx.call_stack.contains(&frame_key) {
                    return Ok(false);
                }
                if let Some(pattern) = ctx
                    .resolver
                    .resolve_named_type_with_arg(name, subtype.as_deref())
                {
                    ctx.call_stack.push(frame_key);
                    ctx.depth = ctx.depth.saturating_add(1);
                    let matched = self.eval_pattern(&pattern, ctx)?;
                    ctx.depth = ctx.depth.saturating_sub(1);
                    ctx.call_stack.pop();
                    if !matched {
                        self.cursor = start_offset;
                    }
                    return Ok(matched);
                }
                Ok(false)
            }
            SyntaxTerm::RuleRef { target } => {
                if ctx.depth >= ctx.max_depth {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Syntax expansion depth limit reached ({}) for rule {target}",
                            ctx.max_depth
                        ),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    return Ok(false);
                }
                let frame_key = format!("rule:{target}");
                if ctx.call_stack.contains(&frame_key) {
                    return Ok(false);
                }
                if let Some(rule) = ctx.resolver.resolve_rule(target) {
                    let old_self = ctx.self_dc;
                    ctx.self_dc = Some(target.to_u128());
                    ctx.call_stack.push(frame_key);
                    ctx.depth = ctx.depth.saturating_add(1);

                    let has_fmt_token = matches!(target, CharTarget::Format(_))
                        && first == target.to_u128();
                    if has_fmt_token && !rule.pattern.starts_with_self() {
                        self.cursor = after_first;
                    }

                    let matched = self.eval_pattern(&rule.pattern, ctx)?;

                    ctx.depth = ctx.depth.saturating_sub(1);
                    ctx.call_stack.pop();
                    ctx.self_dc = old_self;

                    if matched {
                        let matched_end = self.cursor;
                        if ctx.validate_interiors {
                            if let Some(closer_u128) = ctx.resolver.closer_for_opener(target) {
                                if !ctx.resolver.is_opaque_container(first) {
                                    if self
                                        .validate_container_interior_strict(
                                            target,
                                            first_dc,
                                            after_first,
                                            DcChar::from_u128(closer_u128),
                                            matched_end,
                                            ctx.depth.saturating_add(1),
                                        )
                                        .is_err()
                                    {
                                        ctx.has_errors = true;
                                        self.cursor = start_offset;
                                        return Ok(false);
                                    }
                                    self.cursor = matched_end;
                                }
                            }
                        }
                        return Ok(true);
                    }
                    self.cursor = start_offset;
                    return Ok(false);
                }
                if first == target.to_u128() {
                    self.cursor = after_first;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn eval_constrained_equivalent_list(
        &mut self,
        after_wrapper_tag: u64,
        wrapper_end: u64,
        term: &SyntaxTerm,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        if ctx.depth >= ctx.max_depth {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: "Constrained representation expansion depth limit reached".to_owned(),
                token_offset: after_wrapper_tag,
                is_error: true,
            });
            return Ok(false);
        }
        let Some((list_open, after_list_open)) = self.read_token_or_binary_at(after_wrapper_tag)?
        else {
            return Ok(false);
        };
        if list_open != DC_BEGIN_LIST {
            return Ok(false);
        }
        let close_len = dc_char_utf8e128_len(DC_END_LIST);
        if wrapper_end < after_list_open.saturating_add(close_len) {
            return Ok(false);
        }
        let list_body_end = wrapper_end.saturating_sub(close_len);
        let saved_span = self.active_span;
        self.active_span = DcByteSpan::new(after_list_open, list_body_end);
        self.cursor = after_list_open;

        let mut sub_ctx = ctx.clone();
        sub_ctx.depth = sub_ctx.depth.saturating_add(1);

        // Consume optional [number] and required [type] header of 310..311
        let num_term = SyntaxTerm::NamedConstruct {
            name: "number".to_owned(),
            subtype: None,
            capture_var: None,
        };
        let num_pos = self.cursor;
        if !self.eval_term_single(&num_term, &mut sub_ctx)? {
            self.cursor = num_pos;
        }
        let type_term = SyntaxTerm::NamedConstruct {
            name: "type".to_owned(),
            subtype: None,
            capture_var: None,
        };
        if !self.eval_term_single(&type_term, &mut sub_ctx)? {
            self.active_span = saved_span;
            return Ok(false);
        }

        let value_term = SyntaxTerm::NamedConstruct {
            name: "value".to_owned(),
            subtype: None,
            capture_var: None,
        };
        let mut matched_any = false;
        while self.cursor < list_body_end {
            let elem_start = self.cursor;
            let Some((tok, after_tok)) = self.read_token_or_binary_at(elem_start)? else {
                break;
            };
            if tok == DC_LIST_ELEMENT_SEPARATOR {
                self.cursor = after_tok;
                continue;
            }
            let mut cand_ctx = sub_ctx.clone();
            let cand_ok = self.eval_term_single(term, &mut cand_ctx)?;
            if cand_ok && !cand_ctx.has_errors && self.cursor > elem_start {
                matched_any = true;
                continue;
            }
            self.cursor = elem_start;
            if let SyntaxTerm::NamedConstruct { name, .. } = term {
                let unconstrained = SyntaxTerm::NamedConstruct {
                    name: name.clone(),
                    subtype: None,
                    capture_var: None,
                };
                let mut trial_ctx = sub_ctx.clone();
                if self.eval_term_single(&unconstrained, &mut trial_ctx)? && !trial_ctx.has_errors {
                    self.active_span = saved_span;
                    return Ok(false);
                }
                self.cursor = elem_start;
            }
            if !self.eval_term_single(&value_term, &mut sub_ctx)? || self.cursor <= elem_start {
                self.active_span = saved_span;
                return Ok(false);
            }
        }
        self.active_span = saved_span;
        Ok(matched_any)
    }

    fn eval_data_construct(
        &mut self,
        subtype: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, _)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if first_dc != DC_START_ENCAPSULATION_BINARY {
            return Ok(false);
        }
        let constraint = if let Some(sub) = subtype {
            match parse_data_constraint(sub) {
                Ok(c) => Some(c),
                Err(err) => {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!("Invalid data constraint: {err}"),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    return Ok(false);
                }
            }
        } else {
            None
        };

        if let Some((end_offset, DcFrameHeaderInfo::Binary { data_type, payload_len, .. })) =
            self.try_scan_raw_binary_at(start_offset)?
        {
            if let Some((expected_fmt, expected_bytes)) = constraint {
                if let Some(fmt_str) = expected_fmt {
                    let expected_id = FormatId::from_shorthand(fmt_str)
                        .or_else(|| FormatId::from_ident(fmt_str))
                        .and_then(|f| f.dc_id());
                    let actual_id = data_type.as_ref().and_then(|dt| {
                        let mut chars = dt.chars();
                        let first_ch = chars.next()?;
                        if first_ch == DC_BEGIN_TYPE_DEF {
                            chars.next().map(DcChar::to_u128)
                        } else {
                            Some(first_ch.to_u128())
                        }
                    });
                    if expected_id.is_none() || actual_id != expected_id {
                        return Ok(false);
                    }
                }
                if let Some(req_bytes) = expected_bytes {
                    if payload_len != u64::try_from(req_bytes)? {
                        return Ok(false);
                    }
                }
            }
            self.cursor = end_offset;
            return Ok(true);
        }

        let Some(target_203) = CharTarget::from_u128(DC_START_ENCAPSULATION_BINARY.to_u128())
        else {
            return Ok(false);
        };
        let Some(rule_203) = ctx.resolver.resolve_rule(&target_203) else {
            return Ok(false);
        };
        let old_self = ctx.self_dc;
        ctx.self_dc = Some(DC_START_ENCAPSULATION_BINARY.to_u128());
        let matched = self.eval_pattern(&rule_203.pattern, ctx)?;
        ctx.self_dc = old_self;
        if !matched {
            self.cursor = start_offset;
            return Ok(false);
        }
        let end_offset = self.cursor;

        if let Some((expected_fmt, expected_bytes)) = constraint {
            let tokens = self.read_tokens_in_span(DcByteSpan::new(start_offset, end_offset))?;
            if let Some(fmt_str) = expected_fmt {
                let expected_id = FormatId::from_shorthand(fmt_str)
                    .or_else(|| FormatId::from_ident(fmt_str))
                    .and_then(|f| f.dc_id());
                let actual_id =
                    if tokens.get(1).copied() == Some(DC_BEGIN_TYPE_DEF.to_u128()) {
                        tokens.get(2).copied()
                    } else {
                        tokens.get(1).copied()
                    };
                if expected_id.is_none() || actual_id != expected_id {
                    self.cursor = start_offset;
                    return Ok(false);
                }
            }
            if let Some(req_bytes) = expected_bytes {
                match encapsulated_byte_length(&tokens) {
                    Ok(len) if len == req_bytes => {}
                    Ok(_) => {
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                    Err(err) => {
                        ctx.has_errors = true;
                        ctx.diagnostics.push(SyntaxDiagnostic {
                            message: format!("Invalid constrained binary payload: {err}"),
                            token_offset: start_offset,
                            is_error: true,
                        });
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    fn eval_typed_data_construct(
        &mut self,
        subtype: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, _)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if first_dc != DC_START_ENCAPSULATION_BINARY {
            return Ok(false);
        }
        if let Some((end_offset, DcFrameHeaderInfo::Binary { data_type, .. })) =
            self.try_scan_raw_binary_at(start_offset)?
        {
            let Some(ref dt) = data_type else {
                return Ok(false);
            };
            let mut sub_nav = DcDocumentNavigator::from_dc_str(dt);
            let mut sub_ctx = SyntaxEvalContext::new(ctx.resolver, None);
            let type_term = SyntaxTerm::NamedConstruct {
                name: "type".to_owned(),
                subtype: subtype.map(str::to_owned),
                capture_var: None,
            };
            if sub_nav.eval_term_single(&type_term, &mut sub_ctx)?
                && sub_nav.position() == sub_nav.active_span().end
            {
                self.cursor = end_offset;
                return Ok(true);
            }
            return Ok(false);
        }
        let Some(pattern) = ctx.resolver.resolve_named_type_with_arg("typed_data", subtype) else {
            return Ok(false);
        };
        self.eval_pattern(&pattern, ctx)
    }

    fn eval_literal_frame(
        &mut self,
        subtype: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, after_260)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if first_dc != DC_BEGIN_LITERAL {
            return Ok(false);
        }
        let Some((hdr_open, after_262)) = self.read_token_or_binary_at(after_260)? else {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: format!("Unclosed literal at offset {start_offset}: missing Dc 262..263"),
                token_offset: start_offset,
                is_error: true,
            });
            return Ok(false);
        };
        if hdr_open != DC_BEGIN_TYPE_DEF {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: format!("Literal at offset {start_offset} is missing Dc 262..263 type header"),
                token_offset: start_offset,
                is_error: true,
            });
            return Ok(false);
        }
        let Some((type_dc, after_type)) = self.read_token_or_binary_at(after_262)? else {
            ctx.has_errors = true;
            return Ok(false);
        };
        if !ctx.resolver.matches_script(".EL Types", type_dc.to_u128()) {
            ctx.has_errors = true;
            return Ok(false);
        }
        let Some((hdr_close, mut offset)) = self.read_token_or_binary_at(after_type)? else {
            ctx.has_errors = true;
            return Ok(false);
        };
        if hdr_close != DC_END_TYPE_DEF {
            ctx.has_errors = true;
            return Ok(false);
        }

        let mut decoded_payload = Vec::new();
        while let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
            if ch == DC_ESCAPE {
                let Some((escaped, after_esc)) = self.read_token_or_binary_at(next_off)? else {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!("Dangling escape Dc 255 in literal at offset {offset}"),
                        token_offset: offset,
                        is_error: true,
                    });
                    self.cursor = start_offset;
                    return Ok(false);
                };
                if subtype.is_some() {
                    decoded_payload.push(escaped);
                }
                offset = after_esc;
            } else if ch == DC_END_LITERAL {
                if let Some(sub_str) = subtype {
                    if ctx.depth >= ctx.max_depth {
                        ctx.has_errors = true;
                        ctx.diagnostics.push(SyntaxDiagnostic {
                            message: "String constraint expansion depth limit reached".to_owned(),
                            token_offset: start_offset,
                            is_error: true,
                        });
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                    let sub_rule = match parse_dc_syntax(sub_str) {
                        Ok(r) => r,
                        Err(err) => {
                            ctx.has_errors = true;
                            ctx.diagnostics.push(SyntaxDiagnostic {
                                message: format!("Invalid string constraint: {err}"),
                                token_offset: start_offset,
                                is_error: true,
                            });
                            self.cursor = start_offset;
                            return Ok(false);
                        }
                    };
                    let mut payload_dc_str =
                        DcString::with_capacity(decoded_payload.len().saturating_mul(4));
                    for dch in decoded_payload {
                        payload_dc_str.push(dch);
                    }
                    let mut sub_nav = DcDocumentNavigator::from_dc_str(&payload_dc_str);
                    let mut sub_ctx = SyntaxEvalContext::new(ctx.resolver, None);
                    sub_ctx.depth = ctx.depth.saturating_add(1);
                    sub_ctx.max_depth = ctx.max_depth;
                    sub_ctx.call_stack.clone_from(&ctx.call_stack);
                    let sub_ok = sub_nav.eval_pattern(&sub_rule.pattern, &mut sub_ctx)?;
                    ctx.has_errors |= sub_ctx.has_errors;
                    ctx.diagnostics.extend(sub_ctx.diagnostics);
                    if !sub_ok
                        || sub_ctx.has_errors
                        || sub_nav.position() != sub_nav.active_span().end
                    {
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                }
                self.cursor = next_off;
                return Ok(true);
            } else {
                if subtype.is_some() {
                    decoded_payload.push(ch);
                }
                offset = next_off;
            }
        }
        ctx.has_errors = true;
        ctx.diagnostics.push(SyntaxDiagnostic {
            message: format!("Unclosed literal frame starting at offset {start_offset}"),
            token_offset: start_offset,
            is_error: true,
        });
        self.cursor = start_offset;
        Ok(false)
    }

    fn eval_identifier_frame(&mut self, ctx: &mut SyntaxEvalContext<'_>) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, mut offset)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if first_dc != DC_BEGIN_IDENTIFIER {
            return Ok(false);
        }
        let mut char_count = 0usize;
        while let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
            if ch == DC_ESCAPE {
                let Some((_, after_esc)) = self.read_token_or_binary_at(next_off)? else {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!("Dangling escape Dc 255 in identifier at offset {offset}"),
                        token_offset: offset,
                        is_error: true,
                    });
                    self.cursor = start_offset;
                    return Ok(false);
                };
                char_count = char_count.saturating_add(1);
                offset = after_esc;
            } else if ch == DC_END_IDENTIFIER {
                if char_count == 0 {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!("Empty identifier frame (Dc 270..271) at offset {start_offset}"),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    self.cursor = start_offset;
                    return Ok(false);
                }
                self.cursor = next_off;
                return Ok(true);
            } else {
                char_count = char_count.saturating_add(1);
                offset = next_off;
            }
        }
        ctx.has_errors = true;
        ctx.diagnostics.push(SyntaxDiagnostic {
            message: format!("Unclosed identifier frame starting at offset {start_offset}"),
            token_offset: start_offset,
            is_error: true,
        });
        self.cursor = start_offset;
        Ok(false)
    }

    fn eval_number_frame(&mut self, ctx: &mut SyntaxEvalContext<'_>) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, after_6)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if first_dc != DC_BEGIN_NUMBER {
            return Ok(false);
        }
        let Some((fmt_dc, mut offset)) = self.read_token_or_binary_at(after_6)? else {
            return Ok(false);
        };
        if !ctx.resolver.matches_script("Formats:number", fmt_dc.to_u128())
            && !ctx.resolver.matches_script("number", fmt_dc.to_u128())
        {
            return Ok(false);
        }
        let mut digit_count = 0usize;
        while let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
            if ch == DC_END_NUMBER {
                if digit_count == 0 {
                    self.cursor = start_offset;
                    return Ok(false);
                }
                self.cursor = next_off;
                return Ok(true);
            }
            digit_count = digit_count.saturating_add(1);
            offset = next_off;
        }
        self.cursor = start_offset;
        Ok(false)
    }

    /// Strictly validates all constructs in the navigator's active span against
    /// the Document Character syntax rules and named-type definitions.
    pub fn validate_document_strict(&mut self) -> Result<()> {
        let span = self.active_span;
        self.validate_span_strict(span, 0)
    }

    fn validate_span_strict(&mut self, initial_span: DcByteSpan, initial_depth: usize) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        let mut span_stack: Vec<(DcByteSpan, u64, usize)> =
            vec![(initial_span, initial_span.start, initial_depth)];

        let res = (|| -> Result<()> {
            while let Some((span, cursor, depth)) = span_stack.pop() {
                ensure!(
                    depth <= MAX_SYNTAX_EXPANSION_DEPTH,
                    "Document container nesting exceeds maximum depth ({MAX_SYNTAX_EXPANSION_DEPTH})"
                );
                if cursor >= span.end {
                    continue;
                }
                self.active_span = span;
                self.cursor = cursor;
                let sub_spans = self.step_validate_construct_strict(depth)?;
                let next_cursor = self.cursor;
                if next_cursor < span.end {
                    span_stack.push((span, next_cursor, depth));
                }
                for (sub_span, sub_depth) in sub_spans.into_iter().rev() {
                    if !sub_span.is_empty() || sub_depth > MAX_SYNTAX_EXPANSION_DEPTH {
                        span_stack.push((sub_span, sub_span.start, sub_depth));
                    }
                }
            }
            Ok(())
        })();

        self.active_span = saved_span;
        self.cursor = saved_cursor.min(saved_span.end);
        res
    }

    fn step_validate_construct_strict(
        &mut self,
        depth: usize,
    ) -> Result<Vec<(DcByteSpan, usize)>> {
        let start_offset = self.cursor;
        let Some((first_dc, after_first)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(Vec::new());
        };
        let first_u128 = first_dc.to_u128();
        let resolver = default_syntax_resolver();

        // 1. Reject dangling closing delimiters
        if let Some(opener) = resolver.closing_opener_for(first_u128) {
            bail!(
                "Unexpected closing delimiter {first_dc:?} (closes {opener}) at offset {start_offset}"
            );
        }

        // 2. Escape sequence (Dc 255 + next character)
        if first_dc == DC_ESCAPE {
            let Some((_, after_esc)) = self.read_token_or_binary_at(after_first)? else {
                bail!("Dangling escape Dc 255 at offset {start_offset}");
            };
            self.cursor = after_esc;
            return Ok(Vec::new());
        }

        // 3. Binary encapsulation (Dc 203)
        if first_dc == DC_START_ENCAPSULATION_BINARY {
            let mut ctx = SyntaxEvalContext::new(&**resolver, None);
            ctx.validate_interiors = true;
            ctx.depth = depth;
            ensure!(
                self.eval_data_construct(None, &mut ctx)? && !ctx.has_errors,
                "Invalid binary encapsulation (Dc 203..204) at offset {start_offset}"
            );
            return Ok(Vec::new());
        }

        // 4. Literal frame (Dc 260..261)
        if first_dc == DC_BEGIN_LITERAL {
            let mut ctx = SyntaxEvalContext::new(&**resolver, None);
            ctx.validate_interiors = true;
            ctx.depth = depth;
            ensure!(
                self.eval_literal_frame(None, &mut ctx)? && !ctx.has_errors,
                "Invalid literal frame (Dc 260..261) at offset {start_offset}"
            );
            return Ok(Vec::new());
        }

        // 5. Identifier frame (Dc 270..271) and optional infix operator (e.g. Dc 269 assignment)
        if first_dc == DC_BEGIN_IDENTIFIER {
            let mut ctx = SyntaxEvalContext::new(&**resolver, None);
            ctx.validate_interiors = true;
            ctx.depth = depth;
            ensure!(
                self.eval_identifier_frame(&mut ctx)? && !ctx.has_errors,
                "Invalid identifier frame (Dc 270..271) at offset {start_offset}"
            );
            let ident_end = self.cursor;
            if let Some((next_dc, after_op)) = self.read_token_or_binary_at(ident_end)? {
                if resolver.resolve_infix_rule(next_dc.to_u128()).is_some() {
                    if let Some(op_target) = CharTarget::from_u128(next_dc.to_u128()) {
                        if let Some(op_rule) = resolver.resolve_rule(&op_target) {
                            self.cursor = start_offset;
                            let mut op_ctx =
                                SyntaxEvalContext::new(&**resolver, Some(next_dc.to_u128()));
                            op_ctx.validate_interiors = true;
                            op_ctx.depth = depth;
                            ensure!(
                                self.eval_pattern(&op_rule.pattern, &mut op_ctx)?
                                    && !op_ctx.has_errors,
                                "Syntax rule mismatch for infix construct {op_target} at offset {start_offset}"
                            );
                            let assign_end = self.cursor;
                            return Ok(vec![(
                                DcByteSpan::new(after_op, assign_end),
                                depth.saturating_add(1),
                            )]);
                        }
                    }
                }
            }
            return Ok(Vec::new());
        }

        // 6. Number frame (Dc 6..7)
        if first_dc == DC_BEGIN_NUMBER {
            let mut ctx = SyntaxEvalContext::new(&**resolver, None);
            ctx.validate_interiors = true;
            ctx.depth = depth;
            ensure!(
                self.eval_number_frame(&mut ctx)? && !ctx.has_errors,
                "Invalid number frame (Dc 6..7) at offset {start_offset}"
            );
            return Ok(Vec::new());
        }

        // 7. Construct with a syntax rule in the database
        if let Some(target) = CharTarget::from_u128(first_u128) {
            if let Some(rule) = resolver.resolve_rule(&target) {
                ensure!(
                    rule.pattern.starts_with_self(),
                    "Infix or non-self-leading construct {target} cannot start a statement at offset {start_offset}"
                );
                let mut ctx = SyntaxEvalContext::new(&**resolver, Some(first_u128));
                ctx.validate_interiors = true;
                ctx.depth = depth;
                ensure!(
                    self.eval_pattern(&rule.pattern, &mut ctx)? && !ctx.has_errors,
                    "Syntax rule mismatch for {target} at offset {start_offset}"
                );
                let outer_end = self.cursor;

                if let Some(closer_u128) = resolver.closer_for_opener(&target) {
                    if !resolver.is_opaque_container(first_u128) {
                        let sub_spans = self.prepare_container_interior_spans(
                            &target,
                            first_dc,
                            after_first,
                            DcChar::from_u128(closer_u128),
                            outer_end,
                            depth.saturating_add(1),
                        )?;
                        self.cursor = outer_end;
                        return Ok(sub_spans);
                    }
                }
                return Ok(Vec::new());
            }
        }

        // 8. Standalone atomic DcChar
        self.cursor = after_first;
        Ok(Vec::new())
    }

    fn prepare_container_interior_spans(
        &mut self,
        target: &CharTarget,
        opener_dc: DcChar,
        after_opener: u64,
        closer_dc: DcChar,
        outer_end: u64,
        depth: usize,
    ) -> Result<Vec<(DcByteSpan, usize)>> {
        ensure!(
            depth <= MAX_SYNTAX_EXPANSION_DEPTH,
            "Document container nesting exceeds maximum depth ({MAX_SYNTAX_EXPANSION_DEPTH})"
        );
        let resolver = default_syntax_resolver();
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        self.active_span = DcByteSpan::new(after_opener, outer_end);
        let res = (|| -> Result<Vec<(DcByteSpan, usize)>> {
            let (closer_start, after_closer) = self.scan_balanced_container(
                after_opener,
                opener_dc,
                closer_dc,
                &DcProjectionPolicy::strict(),
            )?;
            self.active_span = DcByteSpan::new(after_opener, closer_start);
            self.cursor = after_opener;

            let header_elems = resolver.container_header_elements(target);
            if !header_elems.is_empty() {
                let mut hdr_ctx = SyntaxEvalContext::new(&**resolver, Some(opener_dc.to_u128()));
                hdr_ctx.validate_interiors = true;
                hdr_ctx.depth = depth;
                ensure!(
                    self.eval_pattern(&SyntaxPattern::Sequence(header_elems.to_vec()), &mut hdr_ctx)?
                        && !hdr_ctx.has_errors,
                    "Container header mismatch inside {target} at offset {after_opener}"
                );
            }
            let children_start = self.cursor;

            // For Dc 511 (`:511 [format] [^512]+ 512`), if the format token itself has a
            // syntax rule in the database, validate the interior against that format rule.
            if opener_dc == DC_BEGIN_FORMAT_REPRESENTATION {
                if let Some((fmt_dc, _)) = self.read_token_or_binary_at(after_opener)? {
                    if let Some(fmt_target @ CharTarget::Format(_)) =
                        CharTarget::from_u128(fmt_dc.to_u128())
                    {
                        if resolver.resolve_rule(&fmt_target).is_some() {
                            self.cursor = after_opener;
                            let mut fmt_ctx = SyntaxEvalContext::new(&**resolver, None);
                            fmt_ctx.validate_interiors = true;
                            fmt_ctx.depth = depth;
                            ensure!(
                                self.eval_term_single(
                                    &SyntaxTerm::RuleRef {
                                        target: fmt_target.clone(),
                                    },
                                    &mut fmt_ctx,
                                )? && !fmt_ctx.has_errors
                                    && self.cursor == closer_start,
                                "Format representation interior does not match syntax rule for {fmt_target} at offset {after_opener}"
                            );
                        }
                    }
                }
            }

            let mut spans = vec![(DcByteSpan::new(children_start, closer_start), depth)];
            if outer_end > after_closer {
                spans.push((DcByteSpan::new(after_closer, outer_end), depth));
            }
            Ok(spans)
        })();
        self.active_span = saved_span;
        self.cursor = saved_cursor.min(saved_span.end);
        res
    }

    fn validate_container_interior_strict(
        &mut self,
        target: &CharTarget,
        opener_dc: DcChar,
        after_opener: u64,
        closer_dc: DcChar,
        outer_end: u64,
        depth: usize,
    ) -> Result<()> {
        let sub_spans = self.prepare_container_interior_spans(
            target,
            opener_dc,
            after_opener,
            closer_dc,
            outer_end,
            depth,
        )?;
        for (span, sub_depth) in sub_spans {
            self.validate_span_strict(span, sub_depth)?;
        }
        Ok(())
    }
}

fn pattern_has_payload_constraint(pattern: &SyntaxPattern) -> bool {
    match pattern {
        SyntaxPattern::Sequence(elements) => elements.iter().any(|element| match &element.term {
            SyntaxTerm::NamedConstruct {
                name,
                subtype: Some(_),
                ..
            } => name == "data" || name == "string",
            SyntaxTerm::Group(pattern) => pattern_has_payload_constraint(pattern),
            _ => false,
        }),
        SyntaxPattern::Alternation(branches) => branches.iter().any(pattern_has_payload_constraint),
    }
}

fn encapsulated_byte_length(stream: &[u128]) -> Result<usize> {
    let payload_end = stream.len().checked_sub(1).context("Missing binary terminator")?;
    let mut payload_start = payload_end;
    while let Some(index) = payload_start.checked_sub(1) {
        let token = *stream.get(index).context("Missing binary token")?;
        if !ctb_formats_dcdata::is_dc_base64_encapsulation_char(DcChar::from_u128(token)) {
            break;
        }
        payload_start = index;
    }
    let payload = stream.get(payload_start..payload_end).context("Invalid binary bounds")?;
    ensure!(payload.len().is_multiple_of(4), "Invalid Base64 payload length");
    let digits = payload
        .iter()
        .map(|token| ctb_formats_dcdata::dc_base64_char_to_digit(DcChar::from_u128(*token)))
        .collect::<Result<Vec<_>>>()?;
    let padding = digits.iter().rev().take_while(|digit| **digit == 64).count();
    ensure!(padding <= 2, "Invalid Base64 padding");
    let unpadded = digits.len().checked_sub(padding).context("Invalid Base64 padding")?;
    ensure!(
        digits
            .get(..unpadded)
            .context("Invalid Base64 bounds")?
            .iter()
            .all(|digit| *digit < 64),
        "Embedded Base64 padding"
    );
    if padding > 0 {
        let last = digits
            .get(unpadded.checked_sub(1).context("Missing Base64 digit")?)
            .context("Missing Base64 digit")?;
        let mask = if padding == 2 { 15 } else { 3 };
        ensure!(last & mask == 0, "Noncanonical Base64 trailing bits");
    }
    digits
        .len()
        .checked_div(4)
        .and_then(|groups| groups.checked_mul(3))
        .and_then(|bytes| bytes.checked_sub(padding))
        .context("Binary byte count overflow")
}

#[expect(
    clippy::expect_used,
    reason = "encode_utf_8e_128_buf returns 1..=26, which is provably within u64::MAX"
)]
fn dc_char_utf8e128_len(ch: DcChar) -> u64 {
    let mut buf = [0_u8; 32];
    let len = encode_utf_8e_128_buf(&mut buf, ch.to_u128());
    u64::try_from(len).expect("UTF-8e128 encoded length fits in u64")
}

/// Strictly validates a [`DcMstr`] buffer against all Document Character syntax
/// rules, paired container delimiters, and constraints.
pub fn validate_dc_mixed_strict(mstr: &DcMstr) -> Result<()> {
    let mut nav = DcDocumentNavigator::from_mstr(mstr);
    nav.validate_document_strict()
}

/// Strictly validates a [`DcStr`] buffer against all Document Character syntax
/// rules, paired container delimiters, and constraints.
pub fn validate_dc_str_strict(text: &DcStr) -> Result<()> {
    let mut nav = DcDocumentNavigator::from_dc_str(text);
    nav.validate_document_strict()
}

/// Strictly validates a raw `u128` Document Character token slice against all
/// Document Character syntax rules, paired container delimiters, and constraints.
pub fn validate_dc_tokens_strict(tokens: &[u128]) -> Result<()> {
    let mut dc_str = DcString::with_capacity(tokens.len().saturating_mul(6));
    for &tok in tokens {
        dc_str.push(DcChar::from_u128(tok));
    }
    validate_dc_str_strict(&dc_str)
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
    use ctb_formats_dcdata::syntax::{DatasetRuleResolver, parse_dc_syntax};

    fn dc_string_from_tokens(tokens: &[u128]) -> DcString {
        let mut s = DcString::with_capacity(tokens.len().saturating_mul(6));
        for &tok in tokens {
            s.push(DcChar::from_u128(tok));
        }
        s
    }

    fn eval_tokens_complete(
        tokens: &[u128],
        pattern: &SyntaxPattern,
        self_dc: Option<u128>,
    ) -> Result<(bool, SyntaxEvalContext<'static>)> {
        let resolver = default_syntax_resolver();
        let s = dc_string_from_tokens(tokens);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, self_dc);
        let matched = nav.eval_pattern(pattern, &mut ctx)?;
        let complete =
            matched && !ctx.has_errors && nav.position() == nav.active_span().end;
        Ok((complete, ctx))
    }

    fn literal(text: &str) -> Vec<u128> {
        let mut tokens = vec![
            DcChar::from_short(260).to_u128(),
            DcChar::from_short(262).to_u128(),
            DcChar::from_short(264).to_u128(),
            DcChar::from_short(263).to_u128(),
        ];
        tokens.extend(text.chars().map(u128::from));
        tokens.push(DcChar::from_short(261).to_u128());
        tokens
    }

    #[crate::ctb_test]
    fn constrained_strings_enforce_ascii_and_exact_length() -> Result<()> {
        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        for (text, valid) in [
            ("TEXT", true),
            ("    ", true),
            ("~ !A", true),
            ("", false),
            ("ABC", false),
            ("ABCDE", false),
            ("abc\n", false),
            ("abc\u{7f}", false),
            ("abc\u{e9}", false),
        ] {
            let tokens = literal(text);
            let (complete, _) = eval_tokens_complete(&tokens, &rule.pattern, None)?;
            ensure!(
                complete == valid,
                "Wrong string constraint result for {text:?}"
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn malformed_or_depth_limited_constraints_fail_explicitly() -> Result<()> {
        let tokens = literal("TEXT");
        let s = dc_string_from_tokens(&tokens);
        let resolver = default_syntax_resolver();
        let invalid = SyntaxTerm::NamedConstruct {
            name: "string".to_owned(),
            subtype: Some("]".to_owned()),
            capture_var: None,
        };
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(
            !nav.eval_term_single(&invalid, &mut ctx)?,
            "Malformed string constraint was ignored"
        );
        ensure!(
            ctx.has_errors && !ctx.diagnostics.is_empty(),
            "Missing constraint error"
        );

        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ctx.max_depth = 0;
        ensure!(
            !nav.eval_pattern(&rule.pattern, &mut ctx)?,
            "String constraint exceeded depth limit"
        );
        ensure!(ctx.has_errors, "Missing depth error");

        let (complete, _) =
            eval_tokens_complete(&[u128::from('A')], &rule.pattern, None)?;
        ensure!(!complete, "Unframed string bypassed constraint");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_expression_data_boundaries() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();

        let cases: &[(&str, Vec<u128>, bool)] = &[
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(261)],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(261),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(255),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255), 65, sdc(261)],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(279),
                    sdc(270),
                    65,
                    sdc(271),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(261),
                    sdc(261),
                ],
                true,
            ),
            ("[string]", vec![], false),
            ("[string]", vec![65], false),
            ("[string]", vec![sdc(260), sdc(261)], false),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255)],
                false,
            ),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255), sdc(261)],
                false,
            ),
            ("[identifier]", vec![sdc(270), 65, sdc(271)], true),
            (
                "[identifier]",
                vec![sdc(270), sdc(255), sdc(271), sdc(271)],
                true,
            ),
            (
                "[identifier]",
                vec![sdc(270), sdc(255), sdc(255), sdc(271)],
                true,
            ),
            ("[identifier]", vec![], false),
            ("[identifier]", vec![sdc(270), sdc(271)], false),
            ("[identifier]", vec![sdc(270), 65, sdc(255)], false),
            ("[value]", vec![], false),
            ("[value]", vec![65], false),
            ("[value]", vec![sdc(270), 65, sdc(271)], false),
            ("[value]", vec![sdc(276), sdc(270), 65, sdc(271)], true),
            (
                "[value]",
                vec![
                    sdc(276),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(270),
                    65,
                    sdc(271),
                ],
                true,
            ),
            (
                "[value]",
                vec![sdc(262), sdc(264), sdc(263), sdc(270), 65, sdc(271)],
                false,
            ),
            ("[value]", vec![sdc(279), sdc(270), 65, sdc(271)], true),
            ("[value]", vec![sdc(279)], false),
            ("[statement]", vec![], false),
            ("[statement]", vec![65], false),
            (
                "[statement]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(261)],
                true,
            ),
            ("[statement]", vec![sdc(279), sdc(270), 65, sdc(271)], true),
            (
                "[statement]",
                vec![
                    sdc(270),
                    65,
                    sdc(271),
                    sdc(269),
                    sdc(276),
                    sdc(270),
                    66,
                    sdc(271),
                ],
                true,
            ),
            ("[statement]", vec![sdc(270), 65, sdc(271), sdc(269)], false),
            (
                "258:",
                vec![sdc(258), sdc(279), sdc(270), 65, sdc(271), sdc(259)],
                true,
            ),
            (
                "258:",
                vec![
                    sdc(258),
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(259),
                    sdc(261),
                    sdc(259),
                ],
                true,
            ),
            ("258:", vec![sdc(258), sdc(259)], false),
            ("315:", vec![sdc(315), sdc(276), sdc(270), 65, sdc(271)], true),
            (
                "[checksum]",
                vec![sdc(203), fmt(245), sdc(127), sdc(128), sdc(204)],
                true,
            ),
            ("[checksum]", vec![sdc(203), fmt(245), sdc(204)], true),
            ("[checksum]", vec![sdc(203), sdc(127), sdc(128), sdc(204)], false),
            ("[checksum]", vec![sdc(203), 9999, sdc(127), sdc(204)], false),
        ];
        for (syntax, stream, expected) in cases {
            let rule = parse_dc_syntax(syntax)?;
            let (complete, _) = eval_tokens_complete(stream, &rule.pattern, None)?;
            assert_eq!(complete, *expected, "{syntax}: {stream:?}");
        }

        let rule = parse_dc_syntax("[value]")?;
        let prefix = vec![
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            sdc(255),
            sdc(261),
            sdc(261),
        ];
        let mut stream = prefix.clone();
        stream.extend([sdc(279), sdc(270), 65, sdc(271)]);
        let s = dc_string_from_tokens(&stream);
        let prefix_s = dc_string_from_tokens(&prefix);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**default_syntax_resolver(), None);
        assert!(nav.eval_pattern(&rule.pattern, &mut ctx)?);
        assert!(!ctx.has_errors);
        assert_eq!(nav.position(), u64::try_from(prefix_s.len())?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_data_byte_constraints() -> Result<()> {
        let sdc = |id| DcChar::from_short(id).to_u128();
        for (digits, expected) in [
            (vec![], false),
            (vec![127, 127, 127, 195], false),
            (vec![127, 127, 127, 127, 127, 127, 195, 195], true),
            (vec![127, 127, 127, 127, 127, 127, 127, 195], false),
            (vec![127; 8], false),
            (vec![127, 127, 127, 127, 127, 128, 195, 195], false),
        ] {
            for typed in [false, true] {
                let mut stream = vec![sdc(203)];
                if typed {
                    stream.push(
                        FormatId::Wtf8
                            .dc_id()
                            .context("Missing WTF-8 format identifier")?,
                    );
                }
                stream.extend(digits.iter().map(|id| sdc(*id)));
                stream.push(sdc(204));
                for syntax in ["[data:bytes=4]", "[data:f84;bytes=4]"] {
                    let rule = parse_dc_syntax(syntax)?;
                    let (complete, _) =
                        eval_tokens_complete(&stream, &rule.pattern, None)?;
                    assert_eq!(
                        complete,
                        expected && (typed || syntax == "[data:bytes=4]"),
                        "{syntax}: {stream:?}"
                    );
                    let mut wrapped = vec![sdc(397), sdc(310), sdc(275)];
                    wrapped.extend_from_slice(&stream);
                    wrapped.push(sdc(311));
                    let (wrapped_ok, _) =
                        eval_tokens_complete(&wrapped, &rule.pattern, None)?;
                    assert_eq!(
                        wrapped_ok,
                        expected && (typed || syntax == "[data:bytes=4]"),
                        "wrapped {syntax}: {wrapped:?}"
                    );
                    if syntax == "[data:bytes=4]" {
                        let finder = parse_dc_syntax(
                            "~ ([string:[asciiprintable]{4}] | [data:bytes=4])",
                        )?;
                        let mut tagged = vec![sdc(403)];
                        tagged.extend_from_slice(&wrapped);
                        let (tagged_ok, _) = eval_tokens_complete(
                            &tagged,
                            &finder.pattern,
                            Some(sdc(403)),
                        )?;
                        assert_eq!(
                            tagged_ok, expected,
                            "constrained alternation: {tagged:?}"
                        );
                    }
                }
            }
        }
        for syntax in [
            "[data:bytes=]",
            "[data:bytes=-1]",
            "[data:f84;bytes=4;bytes=5]",
            "[data:f84;length=4]",
            "[data:bytes=999999999999999999999999999999]",
        ] {
            assert!(parse_dc_syntax(syntax).is_err(), "{syntax}");
        }
        let conflicting_lengths = vec![
            sdc(397),
            sdc(310),
            sdc(275),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(195),
            sdc(204),
            sdc(312),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(195),
            sdc(195),
            sdc(204),
            sdc(311),
        ];
        let rule = parse_dc_syntax("[data:bytes=4]")?;
        let (ok, _) =
            eval_tokens_complete(&conflicting_lengths, &rule.pattern, None)?;
        assert!(!ok, "A valid alternative hid an invalid payload byte length");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_mac_metadata_requires_typed_filename_and_rectangle() -> Result<()> {
        let resolver = default_syntax_resolver();
        let sdc = |id| DcChar::from_short(id).to_u128();
        let filename = FormatId::FileName
            .dc_id()
            .context("Missing filename format identifier")?;
        let rectangle = FormatId::QuickDrawRect
            .dc_id()
            .context("Missing rectangle format identifier")?;
        let text = vec![sdc(260), sdc(262), sdc(264), sdc(263), 65, sdc(261)];
        let number = ctb_formats_dcdata::u128_to_dc_number_chars(1)
            .into_iter()
            .map(DcChar::to_u128)
            .collect::<Vec<_>>();
        for (tag, format, payload) in
            [(496, filename, text), (510, rectangle, number.repeat(4))]
        {
            let rule = resolver
                .resolve_rule(&CharTarget::Dc(tag))
                .context("Missing Mac metadata syntax rule")?;
            for framed in [false, true] {
                let mut stream = vec![sdc(tag)];
                if framed {
                    stream.push(sdc(511));
                }
                stream.push(format);
                stream.extend_from_slice(&payload);
                if framed {
                    stream.push(sdc(512));
                }
                let (ok, _) = eval_tokens_complete(
                    &stream,
                    &rule.pattern,
                    Some(sdc(tag)),
                )?;
                assert_eq!(ok, framed, "{stream:?}");
            }
            let mut bare = vec![sdc(tag)];
            bare.extend_from_slice(&payload);
            let (bare_ok, _) =
                eval_tokens_complete(&bare, &rule.pattern, Some(sdc(tag)))?;
            assert!(!bare_ok);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_typed_literal_headers_el_types() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let rule = parse_dc_syntax(":~ [script:.EL Types] 263")?;
        for type_dc in [264u32, 275, 278, 280, 306] {
            let stream = vec![sdc(262), sdc(type_dc), sdc(263)];
            let (ok, _) =
                eval_tokens_complete(&stream, &rule.pattern, Some(sdc(262)))?;
            assert!(ok, "Failed for type Dc {type_dc}");
        }
        let invalid_stream = vec![sdc(262), sdc(246), sdc(263)];
        let (ok, _) = eval_tokens_complete(
            &invalid_stream,
            &rule.pattern,
            Some(sdc(262)),
        )?;
        assert!(!ok);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_non_evaluating_document_parser() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();

        // 1. Parse idiomatic-hello-world.sems: say 'Hello, World!'
        let stream = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            57,
            86,
            93,
            93,
            96,
            30,
            18,
            72,
            96,
            99,
            93,
            85,
            19,
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&stream)?;
        let s = dc_string_from_tokens(&stream);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &DcProjectionPolicy::strict())?;
        assert!(!arena.roots().is_empty());
        let root_id = *arena.roots().first().context("Missing root")?;
        let root = arena.node(root_id).context("Missing root node")?;
        assert!(!root.children().is_empty());

        // 2. Mangled / broken document: unclosed literal followed by unparsed tokens
        let mangled = vec![sdc(260), sdc(262), sdc(264), sdc(263), 10, 20, 99, 100];
        assert!(validate_dc_tokens_strict(&mangled).is_err());
        let mangled_s = dc_string_from_tokens(&mangled);
        let mut mangled_nav = DcDocumentNavigator::from_dc_str(&mangled_s);
        let mut mangled_arena = DcSyntaxArena::new();
        mangled_nav.parse_document_into_arena(
            &mut mangled_arena,
            &DcProjectionPolicy::forgiving(),
        )?;
        assert!(!mangled_arena.nodes().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_strict_document_syntax_validation() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();

        // 1. Valid operation session document (Dc 547 ... 548)
        let valid_session = vec![sdc(547), sdc(270), 99, 115, 99, sdc(271), sdc(548)];
        validate_dc_tokens_strict(&valid_session)?;

        // 2. Dangling closing delimiter: unexpected Dc 548 without opener
        let dangling_closer = vec![sdc(548)];
        let err = validate_dc_tokens_strict(&dangling_closer);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Unexpected closing delimiter"));

        // 3. Unclosed container: Dc 547 without Dc 548
        let unclosed_session =
            vec![sdc(547), sdc(546), sdc(270), 99, 115, 99, sdc(271)];
        assert!(validate_dc_tokens_strict(&unclosed_session).is_err());

        // 4. Checksum section container: Dc 543 ... 544 [545:]
        let valid_checksummed = vec![
            sdc(543),
            sdc(270),
            100,
            sdc(271),
            sdc(544),
            sdc(545),
            sdc(203),
            fmt(245),
            sdc(127),
            sdc(204),
        ];
        validate_dc_tokens_strict(&valid_checksummed)?;

        // 5. Dangling checksum section closer: Dc 544
        assert!(validate_dc_tokens_strict(&[sdc(544)]).is_err());

        // 6. Number container: Dc 6 ... Dc 7
        assert!(validate_dc_tokens_strict(&[sdc(7)]).is_err());

        // 7. Date timestamp with Dc 390 ... 391
        let valid_date = vec![
            sdc(330),
            sdc(390),
            fmt(487),
            sdc(6),
            fmt(199),
            sdc(127),
            sdc(7),
            sdc(391),
        ];
        validate_dc_tokens_strict(&valid_date)?;

        // 8. Raw number without Dc 390 fails strict validation
        let raw_number_date = vec![sdc(330), sdc(6), fmt(199), sdc(127), sdc(7)];
        assert!(validate_dc_tokens_strict(&raw_number_date).is_err());

        // 9. Host local network IP address (Dc 346: :~ [ipaddr]) with IPv4 (f490) and IPv6 (f491)
        for fmt_id in [490, 491] {
            let valid_ip = vec![
                sdc(346),
                sdc(511),
                fmt(fmt_id),
                sdc(6),
                fmt(199),
                sdc(127),
                sdc(7),
                sdc(512),
            ];
            validate_dc_tokens_strict(&valid_ip)?;
        }

        // 10. Raw number without 511/f490/f491 fails IP address strict validation
        let raw_number_ip = vec![sdc(346), sdc(6), fmt(199), sdc(127), sdc(7)];
        assert!(validate_dc_tokens_strict(&raw_number_ip).is_err());

        // 11. File flags description: Dc 388 ... 389 with [list:Semantic:fileflag]
        let valid_flags_list = vec![
            sdc(388),
            sdc(310),
            sdc(275),
            sdc(409),
            sdc(312),
            sdc(410),
            sdc(311),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_list)?;

        // 12. File flags description: Dc 388 ... 389 with raw flags Dc 539
        let valid_flags_raw = vec![
            sdc(388),
            sdc(539),
            sdc(203),
            sdc(127),
            sdc(204),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_raw)?;

        // 13. File flags description: Dc 388 ... 389 with equivalent representations Dc 397
        let valid_flags_equiv = vec![
            sdc(388),
            sdc(397),
            sdc(310),
            sdc(275),
            sdc(310),
            sdc(275),
            sdc(409),
            sdc(311),
            sdc(312),
            sdc(539),
            sdc(203),
            sdc(127),
            sdc(204),
            sdc(311),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_equiv)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_key_value_pair_arrays() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let pair = vec![sdc(514), 107, sdc(312), 118, sdc(515)];
        validate_dc_tokens_strict(&pair)?;
        let resolver = default_syntax_resolver();
        let pair_rule = resolver
            .resolve_rule(&CharTarget::Dc(514))
            .context("Missing pair rule")?;
        let (ok, _) =
            eval_tokens_complete(&pair, &pair_rule.pattern, Some(sdc(514)))?;
        assert!(ok);

        for (begin, end) in [(559, 560), (313, 314)] {
            let empty = vec![sdc(begin), sdc(275), sdc(275), sdc(end)];
            validate_dc_tokens_strict(&empty)?;
            let mut map = vec![sdc(begin), sdc(275), sdc(275)];
            if begin == 559 {
                map.extend_from_slice(&pair);
                map.extend_from_slice(&pair);
            } else {
                map.extend([107, sdc(312), 118, sdc(312), 107, sdc(312), 118]);
            }
            map.push(sdc(end));
            validate_dc_tokens_strict(&map)?;
        }

        let mut nested = vec![sdc(514)];
        nested.extend_from_slice(&pair);
        nested.push(sdc(312));
        nested.extend_from_slice(&pair);
        nested.push(sdc(515));
        validate_dc_tokens_strict(&nested)?;

        let quoted = vec![
            sdc(514),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            sdc(515),
            sdc(312),
            sdc(514),
            sdc(261),
            sdc(312),
            118,
            sdc(515),
        ];
        validate_dc_tokens_strict(&quoted)?;
        let escaped = vec![
            sdc(514),
            sdc(255),
            sdc(515),
            sdc(312),
            sdc(255),
            sdc(312),
            sdc(515),
        ];
        validate_dc_tokens_strict(&escaped)?;

        for broken in [
            vec![sdc(514), 107, sdc(312), 118],
            vec![sdc(514), sdc(312), 118, sdc(515)],
            vec![sdc(514), 107, sdc(312), sdc(515)],
            vec![sdc(559), sdc(275), sdc(275), 107, sdc(312), 118, sdc(560)],
            vec![
                sdc(559),
                sdc(275),
                sdc(275),
                sdc(514),
                107,
                sdc(312),
                118,
                sdc(560),
            ],
            vec![sdc(313), sdc(275), sdc(275), 107, sdc(314)],
        ] {
            assert!(validate_dc_tokens_strict(&broken).is_err(), "{broken:?}");
        }

        let mut legacy_tuple_key = vec![sdc(313), sdc(275), sdc(275)];
        legacy_tuple_key.extend_from_slice(&pair);
        legacy_tuple_key.extend([sdc(312), 118, sdc(314)]);
        validate_dc_tokens_strict(&legacy_tuple_key)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_resolver_and_expansion_bounds() -> Result<()> {
        let resolver = DatasetRuleResolver::load();

        assert!(resolver.resolve_named_type("list").is_some());
        assert!(resolver.resolve_named_type("key_value").is_some());
        assert!(resolver.resolve_named_type("string").is_some());

        let recursive_rule = parse_dc_syntax(":[value]")?;
        let s = dc_string_from_tokens(&[1u128, 2, 3]);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&resolver, None);
        ctx.max_depth = 5;
        ctx.depth = 5;
        assert!(!nav.eval_pattern(&recursive_rule.pattern, &mut ctx)?);
        Ok(())
    }
}
