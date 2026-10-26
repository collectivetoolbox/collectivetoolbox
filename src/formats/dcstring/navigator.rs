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
use ctb_formats_dcdata::dc_char::DcCharExt;
use ctb_formats_dcdata::syntax::{
    CharTarget, DcRole, DcSyntaxRule, Quantifier, SyntaxDiagnostic,
    SyntaxElement, SyntaxPattern, SyntaxRuleResolver, SyntaxTerm,
    default_syntax_resolver, parse_data_constraint, parse_dc_syntax,
};
use ctb_formats_dctf::{
    decode_dctf_buf, decode_utf_8e_128_buf, encode_dctf_buf, encode_utf_8e_128_buf,
};
use sha2::{Digest, Sha256};

pub use crate::encapsulation::ChecksumSpec as BinaryChecksum;

use crate::{
    BinaryPayload, DCMM_HEADER_LEN, DCMM_MAGIC, DCMM_UUID, DcChar,
    DcMixedDecode, DcProjectionContext, DcMstr, DcMstring, DcStr, DcString, Integer,
    Natural, SharedValueContext,
};


mod arena;
mod eval;
mod policy;
mod resolver;
mod scan;

pub use arena::*;
pub use eval::*;
pub use policy::*;
pub use resolver::*;
#[allow(unused_imports, reason = "Internal scanning helpers shared across navigator submodules")]
pub(super) use scan::*;

#[derive(Debug)]
pub(super) enum NavBackend<'a> {
    MemoryMstr(&'a DcMstr),
    MemoryText(&'a DcStr),
    SeekableFile {
        reader: BufReader<File>,
        base_offset: u64,
        end_offset: u64,
        is_v1: bool,
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
    pub(super) backend: NavBackend<'a>,
    pub(super) cursor: u64,
    pub(super) active_span: DcByteSpan,
    pub(super) level_stack: Vec<(DcByteSpan, u64, Option<DcChar>)>,
    pub(super) diagnostics: Vec<SyntaxDiagnostic>,
    pub(super) spliced_tokens: std::collections::VecDeque<DcChar>,
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
            diagnostics: Vec::new(),
            spliced_tokens: std::collections::VecDeque::new(),
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
            diagnostics: Vec::new(),
            spliced_tokens: std::collections::VecDeque::new(),
        }
    }

    /// Returns the current stream byte offset.
    #[must_use]
    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    /// Returns true if the next token fills `role` under `resolver`.
    ///
    /// # Errors
    /// Returns an error if reading the next token fails.
    pub fn peek_fills_role(&mut self, role: DcRole, resolver: &dyn SyntaxRuleResolver) -> Result<bool> {
        if let Some(ch) = self.peek_token()? {
            Ok(resolver.token_fills_role(ch.to_u128(), role))
        } else {
            Ok(false)
        }
    }

    /// Advances and returns the next token if it fills `role` under `resolver`.
    ///
    /// # Errors
    /// Returns an error if reading the next token fails.
    pub fn next_if_fills_role(&mut self, role: DcRole, resolver: &dyn SyntaxRuleResolver) -> Result<Option<DcChar>> {
        if let Some(ch) = self.peek_token()? {
            if resolver.token_fills_role(ch.to_u128(), role) {
                return self.read_next_token();
            }
        }
        Ok(None)
    }

    /// Peeks the next token in the stream without advancing the cursor.
    ///
    /// # Errors
    /// Returns an error if reading from the stream fails.
    pub fn peek_token(&mut self) -> Result<Option<DcChar>> {
        if let Some(ch) = self.spliced_tokens.front() {
            return Ok(Some(*ch));
        }
        if self.cursor >= self.active_span.end {
            return Ok(None);
        }
        let resolver = default_syntax_resolver();
        let policy = DcProjectionPolicy::forgiving_quiet();
        if self.splice_stream_invocation(&DefaultDcExpressionEvaluator, &**resolver, &policy)? {
            if let Some(ch) = self.spliced_tokens.front() {
                return Ok(Some(*ch));
            }
        }
        if let Some((ch, _)) = self.read_token_or_binary_at(self.cursor)? {
            Ok(Some(ch))
        } else {
            Ok(None)
        }
    }

    /// Consumes and returns the next token from the spliced buffer or backend stream.
    ///
    /// # Errors
    /// Returns an error if reading from the stream fails.
    pub fn read_next_token(&mut self) -> Result<Option<DcChar>> {
        if let Some(ch) = self.spliced_tokens.pop_front() {
            return Ok(Some(ch));
        }
        if self.cursor >= self.active_span.end {
            return Ok(None);
        }
        let resolver = default_syntax_resolver();
        let policy = DcProjectionPolicy::forgiving_quiet();
        if self.splice_stream_invocation(&DefaultDcExpressionEvaluator, &**resolver, &policy)? {
            if let Some(ch) = self.spliced_tokens.pop_front() {
                return Ok(Some(ch));
            }
        }
        if let Some((ch, next_offset)) = self.read_token_or_binary_at(self.cursor)? {
            self.cursor = next_offset;
            return Ok(Some(ch));
        }
        Ok(None)
    }

    /// Splices tokens into the navigator's stream buffer.
    pub fn push_spliced_tokens<I: IntoIterator<Item = DcChar>>(&mut self, tokens: I) {
        for tok in tokens {
            self.spliced_tokens.push_back(tok);
        }
    }

    /// Returns a reference to the active spliced tokens queue.
    #[must_use]
    pub fn spliced_tokens(&self) -> &std::collections::VecDeque<DcChar> {
        &self.spliced_tokens
    }

    /// If the cursor is currently positioned at a stream-splicing invocation (e.g. `256 lang.say`),
    /// parses the invocation frame, executes it, advances the cursor past the call,
    /// and pushes the emitted tokens into `spliced_tokens`.
    pub fn splice_stream_say(&mut self, policy: &DcProjectionPolicy) -> Result<bool> {
        let resolver = default_syntax_resolver();
        self.splice_stream_invocation(&DefaultDcExpressionEvaluator, &**resolver, policy)
    }

    /// Evaluates a stream-splicing invocation at the current cursor via `evaluator`,
    /// advances the cursor past the call, and pushes emitted tokens into `spliced_tokens`.
    pub fn splice_stream_invocation(
        &mut self,
        evaluator: &dyn DcExpressionEvaluator,
        resolver: &dyn SyntaxRuleResolver,
        policy: &DcProjectionPolicy,
    ) -> Result<bool> {
        let Some((ch, _)) = self.read_token_or_binary_at(self.cursor)? else {
            return Ok(false);
        };
        if ch.fills_role_with(DcRole::LangSay, resolver) {
            let Ok(end_offset) =
                self.scan_say_frame_at(self.cursor, &DcProjectionPolicy::forgiving_quiet(), resolver)
            else {
                return Ok(false);
            };
            let call_tokens = self.read_tokens_in_span(DcByteSpan::new(self.cursor, end_offset))?;
            let call_dcs: Vec<DcChar> = call_tokens.into_iter().map(DcChar::from_u128).collect();
            if let Some(spliced) = evaluator.eval_stream_splice(&call_dcs, resolver)? {
                self.cursor = end_offset;
                for tok in spliced {
                    self.spliced_tokens.push_back(tok);
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Consumes and returns the next token from `spliced_tokens` or backend stream,
    /// automatically evaluating any stream-splicing invocation encountered.
    pub fn read_next_token_spliced(
        &mut self,
        evaluator: &dyn DcExpressionEvaluator,
        resolver: &dyn SyntaxRuleResolver,
        policy: &DcProjectionPolicy,
    ) -> Result<Option<DcChar>> {
        if let Some(ch) = self.spliced_tokens.pop_front() {
            return Ok(Some(ch));
        }
        if self.splice_stream_invocation(evaluator, resolver, policy)? {
            if let Some(ch) = self.spliced_tokens.pop_front() {
                return Ok(Some(ch));
            }
        }
        self.read_next_token()
    }

    /// Saves the current cursor and spliced token state into a checkpoint.
    #[must_use]
    pub fn save_checkpoint(&self) -> NavigatorCheckpoint {
        NavigatorCheckpoint {
            cursor: self.cursor,
            active_span: self.active_span,
            spliced_tokens: self.spliced_tokens.clone(),
        }
    }

    /// Restores a previously saved checkpoint.
    pub fn restore_checkpoint(&mut self, checkpoint: NavigatorCheckpoint) {
        self.cursor = checkpoint.cursor;
        self.active_span = checkpoint.active_span;
        self.spliced_tokens = checkpoint.spliced_tokens;
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
                && (header.get(5..21) == Some(&DCMM_UUID)
                    || header.get(5..21) == Some(&crate::DCMM_UUID_V1)),
            "Invalid DcMixed (.dcmm) header magic or UUID"
        );
        let is_v1 = header.get(5..21) == Some(&crate::DCMM_UUID_V1);
        let reader = BufReader::with_capacity(64 * 1024, file);
        Ok(Self {
            backend: NavBackend::SeekableFile {
                reader,
                base_offset,
                end_offset: file_len,
                is_v1,
            },
            cursor: header_end,
            active_span: DcByteSpan::new(header_end, file_len),
            level_stack: Vec::new(),
            diagnostics: Vec::new(),
            spliced_tokens: std::collections::VecDeque::new(),
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
        self.level_stack.push((self.active_span, stub.outer_span.end, stub.closer_dc));
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
        let (parent_span, saved_cursor, _) = self
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
        let is_v1 = self.is_v1_encoding();
        let raw_bytes = self.read_span_bytes(span)?;
        let mut wrapped = Vec::with_capacity(DCMM_HEADER_LEN.saturating_add(raw_bytes.len()));
        wrapped.extend_from_slice(&DCMM_MAGIC);
        if is_v1 {
            wrapped.extend_from_slice(&crate::dc_mixed::DCMM_UUID_V1);
            wrapped.extend_from_slice(&raw_bytes);
            let (mst, _) = crate::dc_mixed::dcmm1_to_dcmstring(&wrapped)?;
            Ok(mst)
        } else {
            wrapped.extend_from_slice(&DCMM_UUID);
            wrapped.extend_from_slice(&raw_bytes);
            Ok(DcMstring::from_bytes(wrapped)?)
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
        self.next_sibling_stub_with_resolver(policy, default_syntax_resolver().as_ref())
    }

    /// Advances to and returns the next sibling [`DcFrameStub`] using the
    /// specified [`SyntaxRuleResolver`].
    pub fn next_sibling_stub_with_resolver(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcFrameStub>> {
        let mut assignment = None;
        let mut rhs_rule = None;
        loop {
            let Some(stub) = self.scan_sibling_stub_with_resolver(policy, resolver)? else {
                if let Some((start, operator, rhs_start)) = assignment {
                    self.syntax_issue(policy, rhs_start, "Missing value on RHS of infix operator".to_owned())?;
                    return Ok(Some(DcFrameStub {
                        head_dc: operator,
                        closer_dc: None,
                        outer_span: DcByteSpan::new(start, self.cursor),
                        inner_span: Some(DcByteSpan::new(rhs_start, self.cursor)),
                        header_info: DcFrameHeaderInfo::None,
                    }));
                }
                return Ok(None);
            };
            if let Some(elements) = rhs_rule.take() {
                self.check_infix_operand(stub.outer_span, elements, policy, resolver)?;
            }
            if stub.head_dc.fills_role_with(DcRole::BeginIdentifier, resolver) && stub.closer_dc.is_some_and(|c| c.fills_role_with(DcRole::EndIdentifier, resolver)) {
                if let Some((operator, after_op)) = self.read_token_or_binary_at(stub.outer_span.end)? {
                    if let Some((lhs, rhs)) = resolver.resolve_infix_rule(operator.to_u128()) {
                        self.check_infix_operand(stub.outer_span, lhs, policy, resolver)?;
                        assignment.get_or_insert((stub.outer_span.start, operator, after_op));
                        rhs_rule = Some(rhs);
                        self.cursor = after_op;
                        continue;
                    }
                }
            }
            if let Some((start, operator, rhs_start)) = assignment {
                return Ok(Some(DcFrameStub {
                    head_dc: operator,
                    closer_dc: None,
                    outer_span: DcByteSpan::new(start, stub.outer_span.end),
                    inner_span: Some(DcByteSpan::new(rhs_start, stub.outer_span.end)),
                    header_info: DcFrameHeaderInfo::None,
                }));
            }
            return Ok(Some(stub));
        }
    }

    pub(super) fn check_infix_operand(
        &mut self,
        span: DcByteSpan,
        elements: &[SyntaxElement],
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        self.active_span = span;
        self.cursor = span.start;
        let mut ctx = SyntaxEvalContext::new(resolver, None);
        ctx.policy.clone_from(policy);
        let result = self.eval_pattern(&SyntaxPattern::Sequence(elements.to_vec()), &mut ctx);
        let complete = self.cursor == span.end && !ctx.has_errors;
        self.active_span = saved_span;
        self.cursor = saved_cursor;
        if !result? || !complete {
            self.syntax_issue(policy, span.start, "Infix operand does not match database syntax".to_owned())?;
        }
        Ok(())
    }

    pub(super) fn scan_sibling_stub(
        &mut self,
        policy: &DcProjectionPolicy,
    ) -> Result<Option<DcFrameStub>> {
        self.scan_sibling_stub_with_resolver(policy, default_syntax_resolver().as_ref())
    }

    pub(super) fn scan_sibling_stub_with_resolver(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcFrameStub>> {
        if let Some(&head_dc) = self.spliced_tokens.front() {
            if head_dc.fills_role_with(DcRole::Escape, resolver) {
                self.spliced_tokens.pop_front();
                let Some(escaped_dc) = self.spliced_tokens.pop_front() else {
                    self.syntax_issue(
                        policy,
                        self.cursor,
                        format!("Dangling escape Dc 255 in spliced stream at offset {}", self.cursor),
                    )?;
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: None,
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: None,
                        header_info: DcFrameHeaderInfo::None,
                    }));
                };
                return Ok(Some(DcFrameStub {
                    head_dc: escaped_dc,
                    closer_dc: None,
                    outer_span: DcByteSpan::new(self.cursor, self.cursor),
                    inner_span: None,
                    header_info: DcFrameHeaderInfo::Escaped,
                }));
            }

            if let Some((_, _, Some(expected_closer))) = self.level_stack.last() {
                if *expected_closer == head_dc || resolver.is_equivalent_to(head_dc.to_u128(), expected_closer.to_u128()) {
                    self.spliced_tokens.pop_front();
                    return Ok(None);
                }
            }

            if head_dc.fills_role_with(DcRole::BeginLiteral, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(260) and closer defaults to DC_END_LITERAL.
                let closer = resolver
                    .closer_for_opener(&CharTarget::from_u128(head_dc.to_u128()).unwrap_or(CharTarget::Dc(260)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_LITERAL);
                let mut has_unescaped_closer = false;
                let mut check_iter = self.spliced_tokens.iter().skip(1);
                while let Some(&t) = check_iter.next() {
                    if t.fills_role_with(DcRole::Escape, resolver) {
                        let _ = check_iter.next();
                    } else if t == closer || resolver.is_equivalent_to(t.to_u128(), closer.to_u128()) {
                        has_unescaped_closer = true;
                        break;
                    }
                }
                if has_unescaped_closer {
                    self.spliced_tokens.pop_front();
                    let mut type_header = Vec::new();
                    if let Some(&hdr_open) = self.spliced_tokens.front() {
                        if hdr_open.fills_role_with(DcRole::BeginTypeDef, resolver) {
                            self.spliced_tokens.pop_front();
                            while let Some(tok) = self.spliced_tokens.pop_front() {
                                if tok.fills_role_with(DcRole::EndTypeDef, resolver) {
                                    break;
                                }
                                type_header.push(tok);
                            }
                        }
                    }
                    let mut decoded = DcMstring::new();
                    while let Some(tok) = self.spliced_tokens.pop_front() {
                        if tok.fills_role_with(DcRole::Escape, resolver) {
                            if let Some(escaped) = self.spliced_tokens.pop_front() {
                                decoded.push_char(escaped);
                            }
                        } else if tok == closer || resolver.is_equivalent_to(tok.to_u128(), closer.to_u128()) {
                            break;
                        } else {
                            decoded.push_char(tok);
                        }
                    }
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: Some(closer),
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: None,
                        header_info: DcFrameHeaderInfo::SplicedLiteral { type_header, decoded },
                    }));
                }
            }

            if head_dc.fills_role_with(DcRole::BeginIdentifier, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(270) and closer defaults to DC_END_IDENTIFIER.
                let closer = resolver
                    .closer_for_opener(&CharTarget::from_u128(head_dc.to_u128()).unwrap_or(CharTarget::Dc(270)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_IDENTIFIER);
                let mut has_unescaped_closer = false;
                let mut check_iter = self.spliced_tokens.iter().skip(1);
                while let Some(&t) = check_iter.next() {
                    if t.fills_role_with(DcRole::Escape, resolver) {
                        let _ = check_iter.next();
                    } else if t == closer || resolver.is_equivalent_to(t.to_u128(), closer.to_u128()) {
                        has_unescaped_closer = true;
                        break;
                    }
                }
                if has_unescaped_closer {
                    self.spliced_tokens.pop_front();
                    if let Some(&first_inner) = self.spliced_tokens.front()
                        && !first_inner.fills_role_with(DcRole::Escape, resolver)
                        && first_inner != closer
                        && !resolver.is_equivalent_to(first_inner.to_u128(), closer.to_u128())
                    {
                        if first_inner.fills_role_with(DcRole::BeginTypeDef, resolver) {
                            if self
                                .spliced_tokens
                                .get(1)
                                .is_some_and(|t| is_single_type_token(*t, resolver))
                                && self
                                    .spliced_tokens
                                    .get(2)
                                    .is_some_and(|t| t.fills_role_with(DcRole::EndTypeDef, resolver))
                            {
                                let _ = self.spliced_tokens.pop_front();
                                let _ = self.spliced_tokens.pop_front();
                                let _ = self.spliced_tokens.pop_front();
                            }
                        } else if is_single_type_token(first_inner, resolver) {
                            let _ = self.spliced_tokens.pop_front();
                        }
                    }
                    let mut name = DcMstring::new();
                    while let Some(tok) = self.spliced_tokens.pop_front() {
                        if tok.fills_role_with(DcRole::Escape, resolver) {
                            if let Some(escaped) = self.spliced_tokens.pop_front() {
                                name.push_char(escaped);
                            }
                        } else if tok == closer || resolver.is_equivalent_to(tok.to_u128(), closer.to_u128()) {
                            break;
                        } else {
                            name.push_char(tok);
                        }
                    }
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: Some(closer),
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: None,
                        header_info: DcFrameHeaderInfo::SplicedIdentifier(name),
                    }));
                }
            }

            if head_dc.fills_role_with(DcRole::BeginNumber, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(6) and closer defaults to DC_END_NUMBER.
                let closer = resolver
                    .closer_for_opener(&CharTarget::from_u128(head_dc.to_u128()).unwrap_or(CharTarget::Dc(6)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_NUMBER);
                if self.spliced_tokens.iter().skip(1).any(|&t| t == closer || resolver.is_equivalent_to(t.to_u128(), closer.to_u128())) {
                    self.spliced_tokens.pop_front();
                    let format = match self.spliced_tokens.front() {
                        Some(&tok) if tok == closer || resolver.is_equivalent_to(tok.to_u128(), closer.to_u128()) => DC_INTEGER,
                        // Reason for fallback: popped number format token defaults to DC_INTEGER.
                        Some(_) => self.spliced_tokens.pop_front().unwrap_or(DC_INTEGER),
                        None => DC_INTEGER,
                    };
                    let mut digits = Vec::new();
                    while let Some(tok) = self.spliced_tokens.pop_front() {
                        if tok == closer || resolver.is_equivalent_to(tok.to_u128(), closer.to_u128()) {
                            break;
                        }
                        digits.push(tok);
                    }
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: Some(closer),
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: None,
                        header_info: DcFrameHeaderInfo::SplicedNumber { format, digits },
                    }));
                }
            }

            let closer_opt = CharTarget::from_u128(head_dc.to_u128())
                .and_then(|t| resolver.closer_for_opener(&t))
                .map(DcChar::from_u128)
                .or_else(|| {
                    if head_dc.fills_role_with(DcRole::BeginList, resolver) {
                        Some(ctb_formats_dcdata::dc::DC_END_LIST)
                    } else {
                        None
                    }
                });
            if let Some(closer) = closer_opt {
                if self.spliced_tokens.iter().skip(1).any(|&t| t == closer || resolver.is_equivalent_to(t.to_u128(), closer.to_u128())) {
                    self.spliced_tokens.pop_front();
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: Some(closer),
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: Some(DcByteSpan::new(self.cursor, self.cursor)),
                        header_info: DcFrameHeaderInfo::None,
                    }));
                }
            }

            self.spliced_tokens.pop_front();
            return Ok(Some(DcFrameStub {
                head_dc,
                closer_dc: None,
                outer_span: DcByteSpan::new(self.cursor, self.cursor),
                inner_span: None,
                header_info: DcFrameHeaderInfo::None,
            }));
        }

        if self.cursor >= self.active_span.end {
            return Ok(None);
        }
        let start_offset = self.cursor;
        let Some((first_dc, after_first)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(None);
        };

        // 0. Escape sequence (Dc 255 + escaped character)
        if first_dc.fills_role_with(DcRole::Escape, resolver) {
            let Some((escaped_dc, after_escaped)) = self.read_token_or_binary_at(after_first)? else {
                self.syntax_issue(
                    policy,
                    start_offset,
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
                header_info: DcFrameHeaderInfo::Escaped,
            }));
        }

        // 1. Check if this is a raw or textual binary encapsulation (Dc 203)
        if first_dc.fills_role_with(DcRole::StartEncapsulationBinary, resolver) {
            let (end_offset, header_info) = self.scan_binary_at(start_offset, policy, resolver)?;
            self.cursor = end_offset;
            return Ok(Some(DcFrameStub {
                head_dc: first_dc,
                closer_dc: Some(ctb_formats_dcdata::dc::DC_END_ENCAPSULATION_BINARY),
                outer_span: DcByteSpan::new(start_offset, end_offset),
                inner_span: None,
                header_info,
            }));
        }

        // 1.5. Stream say invocation (Dc 256)
        if first_dc.fills_role_with(DcRole::LangSay, resolver)
            && !self.level_stack.is_empty()
            && self.splice_stream_invocation(&DefaultDcExpressionEvaluator, resolver, policy)?
        {
            return self.scan_sibling_stub_with_resolver(policy, resolver);
        }

        // 2. Check if this is a literal (260..261) or identifier (270..271)
        if first_dc.fills_role_with(DcRole::BeginLiteral, resolver) || first_dc.fills_role_with(DcRole::BeginIdentifier, resolver) {
            let closer = if first_dc.fills_role_with(DcRole::BeginLiteral, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(260) and closer defaults to DC_END_LITERAL.
                resolver.closer_for_opener(&CharTarget::from_u128(first_dc.to_u128()).unwrap_or(CharTarget::Dc(260)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_LITERAL)
            } else {
                // Reason for fallback: opener defaults to CharTarget::Dc(270) and closer defaults to DC_END_IDENTIFIER.
                resolver.closer_for_opener(&CharTarget::from_u128(first_dc.to_u128()).unwrap_or(CharTarget::Dc(270)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_IDENTIFIER)
            };
            let end_offset = self.scan_escaped_text_frame(after_first, closer, policy, resolver)?;
            self.cursor = start_offset;
            let mut ctx = SyntaxEvalContext::new(resolver, None);
            ctx.policy.clone_from(policy);
            let matched = if first_dc.fills_role_with(DcRole::BeginLiteral, resolver) {
                self.eval_literal_frame(None, None, &mut ctx)?
            } else {
                self.eval_identifier_frame(None, &mut ctx)?
            };
            if !matched || ctx.has_errors {
                self.syntax_issue(policy, start_offset, format!("Invalid text frame {first_dc:?} at offset {start_offset}"))?;
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
        if first_dc.fills_role_with(DcRole::BeginNumber, resolver) {
            // Reason for fallback: opener defaults to CharTarget::Dc(6) and closer defaults to DC_END_NUMBER.
            let closer = resolver.closer_for_opener(&CharTarget::from_u128(first_dc.to_u128()).unwrap_or(CharTarget::Dc(6)))
                .map(DcChar::from_u128)
                .unwrap_or(ctb_formats_dcdata::dc::DC_END_NUMBER);
            let end_offset = self.scan_until_token(after_first, closer, policy, resolver)?;
            self.cursor = start_offset;
            let mut ctx = SyntaxEvalContext::new(resolver, None);
            ctx.policy.clone_from(policy);
            if !self.eval_number_frame(None, None, &mut ctx)? || ctx.has_errors {
                self.syntax_issue(policy, start_offset, format!("Invalid number frame at offset {start_offset}"))?;
            }
            self.cursor = end_offset;
            return Ok(Some(DcFrameStub {
                head_dc: first_dc,
                closer_dc: Some(closer),
                outer_span: DcByteSpan::new(start_offset, end_offset),
                inner_span: None,
                header_info: DcFrameHeaderInfo::None,
            }));
        }

        // Reject dangling closing delimiters
        if let Some(opener) = resolver.closing_opener_for(first_dc.to_u128()) {
            self.syntax_issue(
                policy,
                start_offset,
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
                self.record_diagnostic(policy, policy.deprecated_syntax, start_offset, msg)?;
            }

            // 4. Check if `first_dc` is a paired container opener in the database syntax table
            if let Some(closer_u128) = resolver.closer_for_opener(&target) {
                let closer_dc = DcChar::from_u128(closer_u128);
                let (closer_start, after_closer, is_spliced) =
                    self.scan_balanced_container(after_first, first_dc, closer_dc, policy, resolver)?;
                let inner_span = DcByteSpan::new(after_first, closer_start);
                let mut outer_end = after_closer;
                let mut header_info = if is_spliced {
                    DcFrameHeaderInfo::SplicedCloser
                } else {
                    DcFrameHeaderInfo::None
                };

                // Handle trailing elements after closer (e.g. [545:] after 544 for Dc 543)
                let trailing_elems = resolver.trailing_after_closer(&target);
                if !trailing_elems.is_empty() && first_dc.fills_role_with(DcRole::BeginChecksummedSection, resolver) {
                    let (final_end, seq_id, digest_ok) =
                        self.verify_checksummed_section(inner_span, after_closer, policy, resolver)?;
                    outer_end = final_end;
                    header_info = DcFrameHeaderInfo::ChecksummedSection {
                        sequence_id: seq_id,
                        body_span: inner_span,
                        digest_valid: digest_ok,
                    };
                } else if first_dc.fills_role_with(DcRole::BeginFormatRepresentation, resolver) {
                    if let Ok(Some((fmt_dc, _))) = self.read_token_or_binary_at(after_first) {
                        header_info = DcFrameHeaderInfo::FormatRepresentation { format: fmt_dc };
                    }
                }

                self.check_container_grammar_with_resolver(
                    &target,
                    DcByteSpan::new(start_offset, outer_end),
                    inner_span,
                    policy,
                    resolver,
                )?;
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
                        self.syntax_issue(
                            policy,
                            start_offset,
                            format!("Prefix operator {first_dc:?} with missing arguments at offset {start_offset}"),
                        )?;
                    }
                    self.cursor = start_offset;
                    let mut ctx = SyntaxEvalContext::new(resolver, Some(first_dc.to_u128()));
                    ctx.policy.clone_from(policy);
                    let mut matched = self.eval_pattern(&rule.pattern, &mut ctx)?;
                    if matched
                        && !ctx.has_errors
                        && resolver.matches_script(".EL Routines", first_dc.to_u128())
                        && self
                            .read_token_or_binary_at(self.cursor)?
                            .is_some_and(|(tok, _)| tok.fills_role_with(DcRole::BeginParameter, resolver))
                    {
                        matched = false;
                    }
                    if matched && !ctx.has_errors {
                        let mut last_end = self.cursor;
                        if first_dc.fills_role_with(DcRole::RoutineInvocation, resolver) {
                            last_end = self.scan_trailing_parameter_frames_with_resolver(last_end, policy, resolver)?;
                        }
                        if resolver.is_representation_set(first_dc.to_u128()) {
                            self.check_equivalent_list_grammar_with_resolver(DcByteSpan::new(after_first, last_end), policy, resolver)?;
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
                    if resolver.is_representation_set(first_dc.to_u128())
                        && after_first < self.active_span.end {
                        if let Some((tok, _)) = self.read_token_or_binary_at(after_first)? {
                            if tok.fills_role_with(DcRole::BeginList, resolver) {
                                self.cursor = after_first;
                                if let Some(arg_stub) = self.next_sibling_stub_with_resolver(policy, resolver)? {
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
                    }
                    if first_dc.fills_role_with(DcRole::OptionalPresent, resolver) && after_first < self.active_span.end {
                        let mut probe = after_first;
                        let mut depth = 0usize;
                        while probe < self.active_span.end && depth < policy.max_depth {
                            let Some((tok, next_off)) = self.read_token_or_binary_at(probe)? else {
                                break;
                            };
                            if tok.fills_role_with(DcRole::OptionalPresent, resolver) {
                                probe = next_off;
                                depth = depth.saturating_add(1);
                            } else {
                                break;
                            }
                        }
                        if let Some((tok, _)) = self.read_token_or_binary_at(probe)? {
                            if resolver.closing_opener_for(tok.to_u128()).is_none() {
                                self.cursor = probe;
                                if let Some(arg_stub) = self.next_sibling_stub_with_resolver(policy, resolver)? {
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
                    self.syntax_issue(
                        policy,
                        start_offset,
                        format!("Syntax rule mismatch for {target} at offset {start_offset}"),
                    )?;
                    self.cursor = start_offset;
                }
            }

            // Built-in routine invocation (e.g. Dc 256 `lang.say` in `.EL Routines`)
            if resolver.matches_script(".EL Routines", first_dc.to_u128()) {
                let last_end = self.scan_trailing_parameter_frames_with_resolver(after_first, policy, resolver)?;
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

    pub(super) fn record_diagnostic(
        &mut self,
        policy: &DcProjectionPolicy,
        action: DiagnosticAction,
        offset: u64,
        message: String,
    ) -> Result<()> {
        self.diagnostics.push(SyntaxDiagnostic {
            message: message.clone(),
            token_offset: offset,
            is_error: action == DiagnosticAction::Error,
        });
        policy.handle(action, message)
    }

    pub(super) fn syntax_issue(&mut self, policy: &DcProjectionPolicy, offset: u64, message: String) -> Result<()> {
        self.record_diagnostic(policy, policy.syntax_error, offset, message)
    }

    pub(super) fn check_container_grammar(
        &mut self,
        target: &CharTarget,
        outer: DcByteSpan,
        inner: DcByteSpan,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        self.check_container_grammar_with_resolver(
            target,
            outer,
            inner,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    pub(super) fn check_container_grammar_with_resolver(
        &mut self,
        target: &CharTarget,
        outer: DcByteSpan,
        inner: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        let saved_spliced = self.spliced_tokens.clone();
        let result = (|| -> Result<()> {
            self.active_span = outer;
            self.cursor = outer.start;
            self.spliced_tokens.clear();
            if let Some(rule) = resolver.resolve_rule(target) {
                let mut ctx = SyntaxEvalContext::new(resolver, Some(target.to_u128()));
                ctx.policy.clone_from(policy);
                let matched = self.eval_pattern(&rule.pattern, &mut ctx)?;
                if !matched || ctx.has_errors || self.cursor != outer.end {
                    self.syntax_issue(
                        policy,
                        outer.start,
                        format!("Syntax rule mismatch for {target} at offset {}", outer.start),
                    )?;
                }
            }
            self.active_span = inner;
            self.cursor = inner.start;
            self.spliced_tokens.clear();
            let header = resolver.container_header_elements(target);
            if !header.is_empty() {
                let mut ctx = SyntaxEvalContext::new(resolver, Some(target.to_u128()));
                ctx.policy.clone_from(policy);
                if !self.eval_pattern(&SyntaxPattern::Sequence(header.to_vec()), &mut ctx)?
                    || ctx.has_errors
                {
                    self.syntax_issue(
                        policy,
                        inner.start,
                        format!("Container header mismatch inside {target} at offset {}", inner.start),
                    )?;
                }
            }
            if resolver.token_fills_role(target.to_u128(), DcRole::BeginList) {
                let body_start = self.cursor;
                self.check_declared_list_length_with_resolver(target, inner, body_start, policy, resolver)?;
            }
            if resolver.token_fills_role(target.to_u128(), DcRole::BeginFormatRepresentation) {
                if let Some((format, _)) = self.read_token_or_binary_at(inner.start)? {
                    let canon_format = resolver.canonical_token(format.to_u128());
                    if let Some(format_target @ CharTarget::Format(_)) = CharTarget::from_u128(canon_format) {
                        if resolver.resolve_rule(&format_target).is_some() {
                            self.cursor = inner.start;
                            let mut ctx = SyntaxEvalContext::new(resolver, None);
                            ctx.policy.clone_from(policy);
                            if !self.eval_term_single(&SyntaxTerm::RuleRef { target: format_target.clone() }, &mut ctx)?
                                || ctx.has_errors || self.cursor != inner.end
                            {
                                self.syntax_issue(
                                    policy,
                                    inner.start,
                                    format!("Format representation interior does not match syntax rule for {format_target} at offset {}", inner.start),
                                )?;
                            }
                        }
                    }
                }
            }
            Ok(())
        })();
        self.active_span = saved_span;
        self.cursor = saved_cursor;
        self.spliced_tokens = saved_spliced;
        result
    }

    pub(super) fn check_declared_list_length(
        &mut self,
        list_target: &CharTarget,
        inner: DcByteSpan,
        body_start: u64,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        self.check_declared_list_length_with_resolver(
            list_target,
            inner,
            body_start,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    pub(super) fn check_declared_list_length_with_resolver(
        &mut self,
        list_target: &CharTarget,
        inner: DcByteSpan,
        body_start: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let Some((first_tok, _)) = self.read_token_or_binary_at(inner.start)? else {
            return Ok(());
        };
        if !resolver.token_fills_role(first_tok.to_u128(), DcRole::BeginNumber) {
            return Ok(());
        }
        let mut ctx = SyntaxEvalContext::new(resolver, Some(list_target.to_u128()));
        ctx.policy.clone_from(policy);
        self.cursor = inner.start;
        if !self.eval_number_frame(None, None, &mut ctx)? {
            return Ok(());
        }
        let (_, digits) = self.decode_number_span_with_resolver(DcByteSpan::new(inner.start, self.cursor), resolver)?;
        let length = crate::dc_number::parse_dc_number_digits_with_resolver(&digits, resolver)?;
        let Ok(expected) = usize::try_from(&length) else {
            return self.syntax_issue(policy, inner.start, "Invalid declared list length".to_owned());
        };
        // Reason for fallback: ListElementSeparator role defaults to Dc 312.
        let sep_u128 = DcRole::ListElementSeparator
            .defining_short()
            .map_or(312u128, |s| DcChar::from_short(s).to_u128());
        let operand = SyntaxTerm::CharSet {
            negated: true,
            members: vec![CharTarget::from_u128(sep_u128).context("Missing separator target")?],
        };
        let mut actual = 0_usize;
        let mut in_element = false;
        self.cursor = body_start;
        while self.cursor < inner.end {
            let start = self.cursor;
            if let Some((tok, after)) = self.read_token_or_binary_at(start)? {
                if resolver.token_fills_role(tok.to_u128(), DcRole::ListElementSeparator) {
                    in_element = false;
                    self.cursor = after;
                    continue;
                }
            }
            if !in_element {
                actual = actual.saturating_add(1);
                in_element = true;
            }
            if !self.eval_term_single(&operand, &mut ctx)? || self.cursor <= start {
                break;
            }
        }
        if actual != expected {
            self.syntax_issue(policy, inner.start, format!(
                "List length mismatch: header declared {expected} elements, found {actual}",
            ))?;
        }
        Ok(())
    }

    pub(super) fn check_equivalent_list_grammar(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        self.check_equivalent_list_grammar_with_resolver(
            span,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    pub(super) fn check_equivalent_list_grammar_with_resolver(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        let result = (|| -> Result<()> {
            let Some((list_open, after_open)) = self.read_token_or_binary_at(span.start)? else {
                return self.syntax_issue(policy, span.start, "Equivalent representations require a list".to_owned());
            };
            if !list_open.fills_role_with(DcRole::BeginList, resolver) {
                return self.syntax_issue(policy, span.start, "Equivalent representations require a list".to_owned());
            }
            // Reason for fallback: default closer for BeginList is DC_END_LIST.
            let list_closer = resolver
                .closer_for_opener(&CharTarget::from_u128(list_open.to_u128()).context("Missing list target")?)
                .map(DcChar::from_u128)
                .unwrap_or(ctb_formats_dcdata::dc::DC_END_LIST);
            let body_end = span.end.checked_sub(self.char_encoded_len(list_closer)).context("Invalid equivalent list span")?;
            self.active_span = DcByteSpan::new(after_open, body_end);
            self.cursor = after_open;
            let target = CharTarget::from_u128(list_open.to_u128()).context("Missing list target")?;
            let mut ctx = SyntaxEvalContext::new(resolver, Some(list_open.to_u128()));
            ctx.policy.clone_from(policy);
            if !self.eval_pattern(&SyntaxPattern::Sequence(resolver.container_header_elements(&target).to_vec()), &mut ctx)? || ctx.has_errors {
                return self.syntax_issue(policy, after_open, "Equivalent list header mismatch".to_owned());
            }
            let value = SyntaxTerm::NamedConstruct { name: "value".to_owned(), subtype: None, capture_var: None };
            let mut previous_was_separator = true;
            while self.cursor < body_end {
                let start = self.cursor;
                let Some((head, after)) = self.read_token_or_binary_at(start)? else { break; };
                if head.fills_role_with(DcRole::ListElementSeparator, resolver) {
                    previous_was_separator = true;
                    self.cursor = after;
                    continue;
                }
                if !previous_was_separator {
                    self.syntax_issue(policy, start, "Malformed equivalent representations list: missing element separator (Dc 312)".to_owned())?;
                }
                previous_was_separator = false;
                let mut trial = ctx.clone();
                if self.eval_term_single(&value, &mut trial)? && !trial.has_errors && self.cursor > start {
                    continue;
                }
                self.cursor = start;
                if let Some(target) = CharTarget::from_u128(head.to_u128()) {
                    if resolver.resolve_rule(&target).is_some_and(|rule| rule.pattern.starts_with_self()) {
                        let mut trial = ctx.clone();
                        if self.eval_term_single(&SyntaxTerm::RuleRef { target }, &mut trial)? && !trial.has_errors && self.cursor > start {
                            continue;
                        }
                        self.cursor = start;
                    }
                }
                if self.next_sibling_stub_with_resolver(policy, resolver)?.is_none() || self.cursor <= start {
                    self.cursor = after;
                }
            }
            Ok(())
        })();
        self.active_span = saved_span;
        self.cursor = saved_cursor;
        result
    }

    pub(super) fn scan_trailing_parameter_frames(
        &mut self,
        offset: u64,
        policy: &DcProjectionPolicy,
    ) -> Result<u64> {
        self.scan_trailing_parameter_frames_with_resolver(
            offset,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    pub(super) fn scan_trailing_parameter_frames_with_resolver(
        &mut self,
        mut offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        while let Some((next_dc, after_open)) = self.read_token_or_binary_at(offset)? {
            if !next_dc.fills_role_with(DcRole::BeginParameter, resolver) {
                break;
            }
            // Reason for fallback: default closer for BeginParameter is DC_END_PARAMETER.
            let closer = CharTarget::from_u128(next_dc.to_u128())
                .and_then(|t| resolver.closer_for_opener(&t))
                .map(DcChar::from_u128)
                .unwrap_or(ctb_formats_dcdata::dc::DC_END_PARAMETER);
            let (_, after_close, _) = self.scan_balanced_container(
                after_open,
                next_dc,
                closer,
                policy,
                resolver,
            )?;
            offset = after_close;
        }
        Ok(offset)
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
mod tests;
