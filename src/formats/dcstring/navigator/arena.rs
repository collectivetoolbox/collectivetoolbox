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

//! Syntax arena, byte spans, binary payload references, and frame stubs.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{
    BinaryChecksum, Cow, DCMM_MAGIC, DcCharExt, DcDocumentNavigator,
    DcExpressionEvaluator, DcForgivingDefault, DcMixedDecode, DcMstr,
    DcMstring, DcProjectionContext, DcProjectionPolicy, DcRole, DcStr,
    DcString, DiagnosticAction, Digest, Range, Read, Seek, SeekFrom,
    SharedValueContext, SyntaxDiagnostic, SyntaxRuleResolver, dc_char_dctf_len,
    default_syntax_resolver,
};

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
    /// Nonresident bytes identified by a caller-supplied encoding source.
    /// This handle is never serialized; the encoder writes its actual bytes.
    Source { source_id: u64, len: u64 },
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
        /// Expected encapsulation checksum, retained for deferred reads.
        checksum: Option<BinaryChecksum>,
        /// Caller-selected action for a checksum mismatch.
        checksum_action: DiagnosticAction,
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
            Self::FileSpan { len, .. } | Self::Source { len, .. } => *len,
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
            Self::FileSpan { .. } | Self::Source { .. } => None,
        }
    }

    /// Materializes the payload bytes, reading from `reader` if this is a `FileSpan`.
    pub fn materialize<R: Read + Seek>(
        &self,
        mut reader: Option<&mut R>,
    ) -> Result<Cow<'a, [u8]>> {
        match self {
            Self::Source { .. } => {
                bail!("External binary data requires its encoding source")
            }
            Self::Borrowed(slice) => Ok(Cow::Borrowed(slice)),
            Self::Inline(bytes) => Ok(Cow::Owned(bytes.clone())),
            Self::FileSpan {
                file_offset,
                len,
                checksum,
                checksum_action,
            } => {
                let rdr = reader.context(
                    "Cannot materialize on-disk FileSpan without a seekable reader",
                )?;
                let len_usize = usize::try_from(*len)
                    .context("Binary FileSpan length exceeds usize")?;
                let saved_pos = rdr.stream_position()?;
                let result = (|| -> Result<Cow<'a, [u8]>> {
                    rdr.seek(SeekFrom::Start(*file_offset))?;
                    let mut buf = Vec::new();
                    buf.try_reserve_exact(len_usize)?;
                    buf.resize(len_usize, 0);
                    rdr.read_exact(&mut buf)
                        .context("Failed to read BinaryPayloadRef::FileSpan")?;
                    if let Some(checksum) = checksum {
                        if !checksum
                            .matches_reader(&mut buf.as_slice(), *len)?
                        {
                            DcProjectionPolicy::strict().handle(
                                *checksum_action,
                                "Encapsulation checksum mismatch",
                            )?;
                        }
                    }
                    Ok(Cow::Owned(buf))
                })();
                rdr.seek(SeekFrom::Start(saved_pos)).context(
                    "Failed to restore binary payload reader position",
                )?;
                result
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
    Tagged { tag: DcChar, args: Vec<NodeId> },
    /// Assignment statement (`[identifier] 269 [value]`).
    Assignment { ident: DcMstring, value: NodeId },
    /// Framed literal (`260 [262:] <payload> 261`) with `255` unescaped.
    Literal {
        type_header: Vec<DcChar>,
        decoded: DcMstring,
    },
    /// Framed identifier (`270 <payload> 271`) with `255` unescaped.
    Identifier { name: DcMstring },
    /// Framed number (`6 [format:number] [^7]+ 7`).
    Number { format: DcChar, digits: Vec<DcChar> },
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
    /// True if this node is an `Atom` that was escaped by `Dc 255` (`Escape`).
    pub escaped: bool,
}

impl DcSyntaxNode<'_> {
    pub(super) fn semantic_token(&self) -> Option<u128> {
        let token = match &self.kind {
            DcSyntaxKind::Container { opener, .. } => opener,
            DcSyntaxKind::Tagged { tag, .. } => tag,
            DcSyntaxKind::Atom(token) => token,
            _ => return None,
        };
        Some(token.to_u128())
    }

    /// Recognizes alternative-representation wrappers and equivalent tokens.
    #[must_use]
    pub fn is_representation_set(&self) -> bool {
        self.semantic_token().is_some_and(|token| {
            default_syntax_resolver().is_representation_set(token)
        })
    }

    /// Recognizes wrapper markers, list headers, and representation separators.
    #[must_use]
    pub fn is_representation_scaffolding(&self) -> bool {
        self.semantic_token().is_some_and(|token| {
            let resolver = default_syntax_resolver();
            resolver.is_representation_set(token)
                || resolver.is_representation_header_or_separator(token)
        })
    }

    /// Returns true if this node is an `Atom` that was escaped by `Dc 255` (`Escape`),
    /// stripping one layer of syntactic role at the enclosing level.
    #[must_use]
    pub fn is_escaped_atom(&self) -> bool {
        match self.kind {
            DcSyntaxKind::Atom(ch) => {
                self.escaped
                    || self.span.len() >= dc_char_dctf_len(ch).saturating_add(4)
            }
            _ => false,
        }
    }

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
    pub fn alloc(
        &mut self,
        span: DcByteSpan,
        kind: DcSyntaxKind<'a>,
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(DcSyntaxNode {
            span,
            kind,
            escaped: false,
        });
        id
    }

    /// Allocates an escaped atom node in the arena and returns its [`NodeId`].
    pub fn alloc_escaped_atom(
        &mut self,
        span: DcByteSpan,
        ch: DcChar,
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(DcSyntaxNode {
            span,
            kind: DcSyntaxKind::Atom(ch),
            escaped: true,
        });
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
        self.split_operands_with_resolver(
            nodes,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Splits a sequence of child [`NodeId`]s on [`DC_LIST_ELEMENT_SEPARATOR`] using the provided resolver.
    #[must_use]
    pub fn split_operands_with_resolver(
        &self,
        nodes: &[NodeId],
        resolver: &dyn SyntaxRuleResolver,
    ) -> Vec<Vec<NodeId>> {
        let mut result = Vec::new();
        let mut current = Vec::new();
        for &id in nodes {
            if self.get(id).is_some_and(|n| {
                !n.is_escaped_atom()
                    && matches!(
                        n.kind,
                        DcSyntaxKind::Atom(ch)
                            if ch.fills_role_with(DcRole::ListElementSeparator, resolver)
                    )
            }) {
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
        self.extract_kv_map_operand_entries_with_resolver(
            id,
            require_object_headers,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Extracts operands for each entry in a key-value map container using the
    /// provided [`SyntaxRuleResolver`].
    pub fn extract_kv_map_operand_entries_with_resolver(
        &self,
        id: NodeId,
        require_object_headers: bool,
        resolver: &dyn SyntaxRuleResolver,
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
        let is_pairs = opener.fills_role_with(DcRole::BeginKvMap, resolver);
        let is_legacy =
            opener.fills_role_with(DcRole::BeginLegacyKvMap, resolver);
        ensure!(
            is_pairs || is_legacy,
            "Expected map container (Dc 559 or Dc 313), found opener {opener}"
        );
        let header =
            children.get(..2).context("Missing map type declarations")?;
        if require_object_headers {
            ensure!(
                header.iter().all(|&child| {
                    self.get(child).is_some_and(
                        |cn| matches!(cn.kind, DcSyntaxKind::Atom(ch) if ch.fills_role_with(DcRole::TypeAny, resolver)),
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
                                if ch.fills_role_with(DcRole::TypeAny, resolver)
                                    || ch.fills_role_with(DcRole::TypeToken, resolver)
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
                let operands =
                    self.split_operands_with_resolver(body, resolver);
                for pair in operands.chunks(2) {
                    let [key, value] = pair else {
                        bail!("Unpaired legacy map operand");
                    };
                    ensure!(
                        !key.is_empty() && !value.is_empty(),
                        "Empty legacy map operand"
                    );
                    entries.push((key.clone(), value.clone()));
                }
            }
        } else {
            for &pair_id in body {
                let pair_node = self
                    .get(pair_id)
                    .context("Pair NodeId must exist in arena")?;
                let (pair_opener, pair_children, pair_unclosed) =
                    match &pair_node.kind {
                        DcSyntaxKind::Container {
                            opener,
                            children,
                            unclosed,
                            ..
                        } => (*opener, children, *unclosed),
                        other => bail!(
                            "Expected explicitly framed map entry (Dc 514), found {other:?}"
                        ),
                    };
                ensure!(
                    pair_opener.fills_role_with(DcRole::BeginKvPair, resolver)
                        && !pair_unclosed,
                    "Expected explicitly framed map entry (Dc 514..515)"
                );
                let operands =
                    self.split_operands_with_resolver(pair_children, resolver);
                let [key, value] = operands.as_slice() else {
                    bail!("Expected two pair operands");
                };
                ensure!(
                    !key.is_empty() && !value.is_empty(),
                    "Empty pair operand"
                );
                entries.push((key.clone(), value.clone()));
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
        self.extract_kv_map_node_pairs_with_resolver(
            id,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Extracts single-node `(key_id, value_id)` pairs from a map container
    /// (`Dc 559..560` or `Dc 313..314`) using the provided [`SyntaxRuleResolver`].
    pub fn extract_kv_map_node_pairs_with_resolver(
        &self,
        id: NodeId,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(DcChar, Vec<(NodeId, NodeId)>)> {
        let (opener, _, operand_entries) = self
            .extract_kv_map_operand_entries_with_resolver(
                id, false, resolver,
            )?;
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
    /// Container whose closer delimiter was produced by stream splicing (e.g. `lang.say`).
    SplicedCloser,
    /// Binary encapsulation (`203..204`) with payload byte span and optional data type.
    Binary {
        data_type: Option<DcString>,
        payload_span: Option<DcByteSpan>,
        payload_len: u64,
        checksum: Option<BinaryChecksum>,
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
    FormatRepresentation { format: DcChar },
    /// Spliced framed literal construct (`260..261`).
    SplicedLiteral {
        type_header: Vec<DcChar>,
        decoded: DcMstring,
    },
    /// Spliced framed identifier construct (`270..271`).
    SplicedIdentifier(DcMstring),
    /// Spliced number construct (`6..7`).
    SplicedNumber { format: DcChar, digits: Vec<DcChar> },
    /// Character whose syntactic role at this level was escaped by `Dc 255`.
    Escaped,
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
        if matches!(self.header_info, DcFrameHeaderInfo::SplicedCloser) {
            return false;
        }
        match (self.closer_dc, self.inner_span) {
            (Some(_), Some(inner)) => inner.end >= self.outer_span.end,
            _ => false,
        }
    }
}

impl<'a> DcDocumentNavigator<'a> {
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
        self.decode_frame_with_arena_and_resolver(
            stub,
            arena,
            context,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`], [`DcProjectionPolicy`], and [`SyntaxRuleResolver`],
    /// reusing the provided [`DcSyntaxArena`].
    pub fn decode_frame_with_arena_and_resolver<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<T> {
        let root_id =
            self.expand_frame_with_resolver(stub, arena, policy, resolver)?;
        let (result, projection_diagnostics) = {
            let mut ctx = DcProjectionContext::with_all_resolvers(
                arena,
                policy,
                context,
                Some(self),
                Some(resolver),
                None,
            );
            let res = T::decode_dc_node(&mut ctx, root_id);
            (res, ctx.diagnostics().to_vec())
        };
        for diagnostic in projection_diagnostics {
            arena.push_diagnostic(diagnostic);
        }
        result
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`] and [`DcProjectionPolicy`].
    pub fn decode_frame<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
    ) -> Result<T> {
        self.decode_frame_with_resolver(
            stub,
            context,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`], [`DcProjectionPolicy`], and [`SyntaxRuleResolver`].
    pub fn decode_frame_with_resolver<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<T> {
        let mut arena = DcSyntaxArena::new();
        self.decode_frame_with_arena_and_resolver(
            stub, &mut arena, context, policy, resolver,
        )
    }

    /// Expands a [`DcFrameStub`] into a full CST inside the reusable [`DcSyntaxArena`].
    pub fn expand_frame(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        self.expand_frame_with_resolver(
            stub,
            arena,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Expands a [`DcFrameStub`] into a full CST inside the reusable [`DcSyntaxArena`]
    /// using the provided [`SyntaxRuleResolver`].
    pub fn expand_frame_with_resolver(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<NodeId> {
        let saved_cursor = self.cursor;
        let saved_span = self.active_span;
        self.active_span = stub.outer_span;
        self.cursor = stub.outer_span.start;
        let res =
            self.parse_node_into_arena_with_resolver(arena, policy, resolver);
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
        self.parse_document_into_arena_with_resolver(
            arena,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Parses all sibling constructs in the active span into `arena` using the
    /// provided [`SyntaxRuleResolver`].
    pub fn parse_document_into_arena_with_resolver(
        &mut self,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let mut raw_roots = Vec::new();
        let res = (|| -> Result<()> {
            while let Some(stub) =
                self.next_sibling_stub_with_resolver(policy, resolver)?
            {
                let id = self.build_arena_node_from_stub_with_resolver(
                    &stub, arena, policy, resolver,
                )?;
                raw_roots.push(id);
                for diagnostic in self.diagnostics.drain(..) {
                    arena.push_diagnostic(diagnostic);
                }
            }
            Ok(())
        })();
        for diagnostic in self.diagnostics.drain(..) {
            arena.push_diagnostic(diagnostic);
        }
        res?;
        let normalized_roots = Self::normalize_field_nodes_with_resolver(
            raw_roots, arena, resolver,
        );
        for id in normalized_roots {
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
        self.parse_node_into_arena_with_resolver(
            arena,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Parses the next statement or container at the current cursor into `arena`
    /// using the provided [`SyntaxRuleResolver`].
    pub fn parse_node_into_arena_with_resolver(
        &mut self,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<NodeId> {
        let start = self.cursor;
        let res = (|| -> Result<NodeId> {
            let Some(stub) =
                self.next_sibling_stub_with_resolver(policy, resolver)?
            else {
                bail!("Unexpected end of stream at offset {start}");
            };
            self.build_arena_node_from_stub_with_resolver(
                &stub, arena, policy, resolver,
            )
        })();
        for diagnostic in self.diagnostics.drain(..) {
            arena.push_diagnostic(diagnostic);
        }
        res
    }

    pub(super) fn is_separator(ch: DcChar) -> bool {
        Self::is_separator_with(ch, default_syntax_resolver().as_ref())
    }

    pub(super) fn is_separator_with(
        ch: DcChar,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        ch.fills_role_with(DcRole::ListElementSeparator, resolver)
            || ch.fills_role_with(DcRole::TypeAny, resolver)
            || ch.fills_role_with(DcRole::Escape, resolver)
            || ch.fills_role_with(DcRole::FilePathDelimiter, resolver)
    }

    pub(super) fn is_primitive_value_atom(ch: DcChar) -> bool {
        Self::is_primitive_value_atom_with(
            ch,
            default_syntax_resolver().as_ref(),
        )
    }

    pub(super) fn is_primitive_value_atom_with(
        ch: DcChar,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        ch.fills_role_with(DcRole::PrimitiveValueAtom, resolver)
            || ch.fills_role_with(DcRole::Positive, resolver)
            || ch.fills_role_with(DcRole::Negative, resolver)
    }

    pub(super) fn is_value_atom(ch: DcChar) -> bool {
        Self::is_value_atom_with(ch, default_syntax_resolver().as_ref())
    }

    pub(super) fn is_value_atom_with(
        ch: DcChar,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        Self::is_primitive_value_atom_with(ch, resolver)
            || ch.fills_role_with(DcRole::DomainValueAtom, resolver)
    }

    pub(super) fn is_transparent_wrapper_tag(ch: DcChar) -> bool {
        Self::is_transparent_wrapper_tag_with(
            ch,
            default_syntax_resolver().as_ref(),
        )
    }

    pub(super) fn is_transparent_wrapper_tag_with(
        ch: DcChar,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        ch.fills_role_with(DcRole::OptionalPresent, resolver)
            || resolver.is_representation_set(ch.to_u128())
    }

    pub(super) fn is_pairable_kind(kind: &DcSyntaxKind<'_>) -> bool {
        Self::is_pairable_kind_with(kind, default_syntax_resolver().as_ref())
    }

    pub(super) fn is_pairable_kind_with(
        kind: &DcSyntaxKind<'_>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        match kind {
            DcSyntaxKind::Number { .. }
            | DcSyntaxKind::Binary { .. }
            | DcSyntaxKind::Literal { .. }
            | DcSyntaxKind::Identifier { .. }
            | DcSyntaxKind::Assignment { .. }
            | DcSyntaxKind::Container { .. } => true,
            DcSyntaxKind::Atom(ch) => {
                Self::is_primitive_value_atom_with(*ch, resolver)
            }
            DcSyntaxKind::Tagged { tag, .. } => {
                Self::is_transparent_wrapper_tag_with(*tag, resolver)
            }
            DcSyntaxKind::Malformed { .. } => false,
        }
    }

    pub(crate) fn normalize_field_nodes_with_resolver(
        nodes: Vec<NodeId>,
        arena: &mut DcSyntaxArena<'a>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Vec<NodeId> {
        if nodes.len() <= 1 {
            return nodes;
        }

        // Right-to-left pairing of atom tags with their immediate values into Tagged nodes.
        let mut stack: Vec<NodeId> = Vec::with_capacity(nodes.len());
        for &node_id in nodes.iter().rev() {
            let Some(node) = arena.get(node_id) else {
                continue;
            };
            if let DcSyntaxKind::Atom(ch) = node.kind
                && !node.is_escaped_atom()
            {
                if !Self::is_separator_with(ch, resolver)
                    && !Self::is_value_atom_with(ch, resolver)
                {
                    if let Some(&top_id) = stack.last() {
                        if let Some(top_node) = arena.get(top_id)
                            && (Self::is_pairable_kind_with(
                                &top_node.kind,
                                resolver,
                            ) || top_node.is_escaped_atom())
                        {
                            let start = node.span.start;
                            let end = top_node.span.end;
                            stack.pop();
                            let span = DcByteSpan::new(start, start.max(end));
                            let tagged_id = arena.alloc(
                                span,
                                DcSyntaxKind::Tagged {
                                    tag: ch,
                                    args: vec![top_id],
                                },
                            );
                            stack.push(tagged_id);
                            continue;
                        }
                    }
                }
            }
            stack.push(node_id);
        }
        stack.reverse();
        stack
    }

    /// Parses a single root node from `bytes` into `arena` using `policy` and `resolver`.
    pub fn parse_arena_with_resolver(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<NodeId> {
        let mut nav = if bytes.starts_with(&DCMM_MAGIC) {
            let mstr = DcMstr::from_bytes_with_policy(bytes, policy)?;
            Self::from_mstr(mstr)
        } else {
            let text = DcStr::from_bytes(bytes)?;
            Self::from_dc_str(text)
        };
        nav.parse_document_into_arena_with_resolver(arena, policy, resolver)?;
        arena
            .roots()
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Empty document or node stream"))
    }

    /// Parses a single root node from `bytes` into `arena` using `policy`.
    pub fn parse_arena(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        Self::parse_arena_with_resolver(
            bytes,
            arena,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Parses all document root nodes from `bytes` into `arena` using `policy` and `resolver`.
    pub fn parse_document_arena_with_resolver(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let mut nav = if bytes.starts_with(&DCMM_MAGIC) {
            let mstr = DcMstr::from_bytes_with_policy(bytes, policy)?;
            Self::from_mstr(mstr)
        } else {
            let text = DcStr::from_bytes(bytes)?;
            Self::from_dc_str(text)
        };
        nav.parse_document_into_arena_with_resolver(arena, policy, resolver)
    }

    /// Parses all document root nodes from `bytes` into `arena` using `policy`.
    pub fn parse_document_arena(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        Self::parse_document_arena_with_resolver(
            bytes,
            arena,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    pub(super) fn build_arena_node_from_stub_with_resolver(
        &mut self,
        root_stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<NodeId> {
        enum ArenaBuildFrame {
            Assignment {
                outer_span: DcByteSpan,
                ident: DcMstring,
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

        let base_level_depth = self.level_stack.len();

        let check_depth_exceeded_leaf = |nav: &mut Self,
                                         stub: &DcFrameStub,
                                         current_depth: usize,
                                         arena: &mut DcSyntaxArena<'a>|
         -> Result<Option<StubStep>> {
            if current_depth < policy.max_depth {
                return Ok(None);
            }
            let msg = format!(
                "Document nesting depth exceeds maximum limit ({}) at offset {}",
                policy.max_depth, stub.outer_span.start
            );
            nav.record_diagnostic(
                policy,
                policy.depth_limit_exceeded,
                stub.outer_span.start,
                msg.clone(),
            )?;
            Ok(Some(StubStep::Leaf(arena.alloc(
                stub.outer_span,
                DcSyntaxKind::Malformed {
                    raw_tokens: Vec::new(),
                    message: msg,
                },
            ))))
        };

        let classify_stub = |nav: &mut Self,
                             stub: &DcFrameStub,
                             current_depth: usize,
                             arena: &mut DcSyntaxArena<'a>|
         -> Result<StubStep> {
            match &stub.header_info {
                DcFrameHeaderInfo::Escaped => {
                    return Ok(StubStep::Leaf(
                        arena.alloc_escaped_atom(stub.outer_span, stub.head_dc),
                    ));
                }
                DcFrameHeaderInfo::SplicedLiteral {
                    type_header,
                    decoded,
                } => {
                    return Ok(StubStep::Leaf(arena.alloc(
                        stub.outer_span,
                        DcSyntaxKind::Literal {
                            type_header: type_header.clone(),
                            decoded: decoded.clone(),
                        },
                    )));
                }
                DcFrameHeaderInfo::SplicedIdentifier(name) => {
                    return Ok(StubStep::Leaf(arena.alloc(
                        stub.outer_span,
                        DcSyntaxKind::Identifier { name: name.clone() },
                    )));
                }
                DcFrameHeaderInfo::SplicedNumber { format, digits } => {
                    return Ok(StubStep::Leaf(arena.alloc(
                        stub.outer_span,
                        DcSyntaxKind::Number {
                            format: *format,
                            digits: digits.clone(),
                        },
                    )));
                }
                _ => {}
            }
            let head = stub.head_dc;
            if head.fills_role_with(DcRole::StartEncapsulationBinary, resolver)
            {
                let node = nav.parse_binary_arena_node(
                    stub.outer_span,
                    policy,
                    resolver,
                )?;
                return Ok(StubStep::Leaf(arena.alloc(stub.outer_span, node)));
            }
            if head.fills_role_with(DcRole::BeginLiteral, resolver) {
                let (type_header, decoded) = nav
                    .decode_literal_span_with_resolver(
                        stub.outer_span,
                        policy,
                        resolver,
                    )?;
                return Ok(StubStep::Leaf(arena.alloc(
                    stub.outer_span,
                    DcSyntaxKind::Literal {
                        type_header,
                        decoded,
                    },
                )));
            }
            if head.fills_role_with(DcRole::BeginIdentifier, resolver) {
                let name = nav.decode_identifier_span_with_resolver(
                    stub.outer_span,
                    policy,
                    resolver,
                )?;
                return Ok(StubStep::Leaf(arena.alloc(
                    stub.outer_span,
                    DcSyntaxKind::Identifier { name },
                )));
            }
            if head.fills_role_with(DcRole::BeginNumber, resolver) {
                let (format, digits) = nav.decode_number_span_with_resolver(
                    stub.outer_span,
                    resolver,
                )?;
                return Ok(StubStep::Leaf(arena.alloc(
                    stub.outer_span,
                    DcSyntaxKind::Number { format, digits },
                )));
            }
            if resolver.resolve_infix_rule(head.to_u128()).is_some() {
                if let Some(inner) = stub.inner_span {
                    if let Some(step) = check_depth_exceeded_leaf(
                        nav,
                        stub,
                        current_depth,
                        arena,
                    )? {
                        return Ok(step);
                    }
                    let ident_span = DcByteSpan::new(
                        stub.outer_span.start,
                        inner.start.saturating_sub(nav.char_encoded_len(head)),
                    );
                    let ident = nav.decode_identifier_span_with_resolver(
                        ident_span, policy, resolver,
                    )?;
                    nav.enter_frame(stub)?;
                    return Ok(StubStep::Push(ArenaBuildFrame::Assignment {
                        outer_span: stub.outer_span,
                        ident,
                        value: None,
                    }));
                }
            }
            if let Some(closer) = stub.closer_dc {
                let unclosed = stub.is_unclosed();
                if stub.inner_span.is_some()
                    && !resolver.is_opaque_container(head.to_u128())
                {
                    if let Some(step) = check_depth_exceeded_leaf(
                        nav,
                        stub,
                        current_depth,
                        arena,
                    )? {
                        return Ok(step);
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
                if let Some(step) =
                    check_depth_exceeded_leaf(nav, stub, current_depth, arena)?
                {
                    return Ok(step);
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
                            ArenaBuildFrame::Assignment { value, .. } => {
                                *value = Some(node_id)
                            }
                            ArenaBuildFrame::Container { children, .. } => {
                                children.push(node_id)
                            }
                            ArenaBuildFrame::Tagged { args, .. } => {
                                args.push(node_id)
                            }
                        }
                        continue;
                    }
                    return Ok(node_id);
                }

                let start = self.cursor;
                if let Some(child_stub) =
                    self.next_sibling_stub_with_resolver(policy, resolver)?
                {
                    match classify_stub(
                        self,
                        &child_stub,
                        current_depth,
                        arena,
                    )? {
                        StubStep::Leaf(child_id) => {
                            if let Some(top) = work_stack.last_mut() {
                                match top {
                                    ArenaBuildFrame::Assignment {
                                        value,
                                        ..
                                    } => {
                                        *value = Some(child_id);
                                    }
                                    ArenaBuildFrame::Container {
                                        children,
                                        ..
                                    } => {
                                        children.push(child_id);
                                    }
                                    ArenaBuildFrame::Tagged {
                                        args, ..
                                    } => {
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
                            let is_format_or_temporal = opener.fills_role_with(
                                DcRole::BeginFormatRepresentation,
                                resolver,
                            );
                            let normalized_children = if is_format_or_temporal {
                                children
                            } else {
                                Self::normalize_field_nodes_with_resolver(
                                    children, arena, resolver,
                                )
                            };
                            arena.alloc(
                                outer_span,
                                DcSyntaxKind::Container {
                                    opener,
                                    closer,
                                    header: Vec::new(),
                                    children: normalized_children,
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
                            let normalized_args =
                                Self::normalize_field_nodes_with_resolver(
                                    args, arena, resolver,
                                );
                            let unwrap_param =
                                |id: NodeId,
                                 arena: &DcSyntaxArena<'a>|
                                 -> NodeId {
                                    if let Some(DcSyntaxKind::Container {
                                        opener,
                                        children,
                                        ..
                                    }) = arena.get(id).map(|n| &n.kind)
                                        && opener.fills_role_with(
                                            DcRole::BeginParameter,
                                            resolver,
                                        )
                                        && let [single] = children.as_slice()
                                    {
                                        *single
                                    } else {
                                        id
                                    }
                                };
                            if resolver.is_assignment_construct(tag.to_u128())
                                && let [raw_ident_id, raw_val_id] =
                                    normalized_args.as_slice()
                            {
                                let ident_id =
                                    unwrap_param(*raw_ident_id, arena);
                                let val_id = unwrap_param(*raw_val_id, arena);
                                if let Some(DcSyntaxKind::Identifier { name }) =
                                    arena.get(ident_id).map(|n| &n.kind)
                                {
                                    let ident = name.clone();
                                    arena.alloc(
                                        outer_span,
                                        DcSyntaxKind::Assignment {
                                            ident,
                                            value: val_id,
                                        },
                                    )
                                } else {
                                    arena.alloc(
                                        outer_span,
                                        DcSyntaxKind::Tagged {
                                            tag,
                                            args: normalized_args,
                                        },
                                    )
                                }
                            } else {
                                arena.alloc(
                                    outer_span,
                                    DcSyntaxKind::Tagged {
                                        tag,
                                        args: normalized_args,
                                    },
                                )
                            }
                        }
                        ArenaBuildFrame::Assignment { .. } => unreachable!(),
                    };
                    if let Some(parent) = work_stack.last_mut() {
                        match parent {
                            ArenaBuildFrame::Assignment { value, .. } => {
                                *value = Some(node_id)
                            }
                            ArenaBuildFrame::Container { children, .. } => {
                                children.push(node_id)
                            }
                            ArenaBuildFrame::Tagged { args, .. } => {
                                args.push(node_id)
                            }
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
}
