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

//! System-block graph compiler and append-only 100-node chunk ledger.
//!
//! Translates the Document Character and Format CSV database into immutable
//! 3-token statement triples, seed classification entities, and deduplicated
//! value nodes in the System block (`8589934592..=12884901887`), persisted as
//! 100-node chunk files in `docs/85899_34592.ctbnodes`, etc.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use ctb_formats_utf_8e_128::{decode_utf_8e_128_buf, encode_utf_8e_128_buf};
use ctb_storage_minimal::global_graph_layout::{
    SYSTEM_REGION_END, SYSTEM_REGION_START,
};
use ctb_storage_minimal::shorthand::DcShorthand;
use ctb_utilities::format_id::{
    DC_APPLE_OS_TYPE, DC_FILE_EXTENSION, DC_INTEGER, DC_MEDIA_TYPE,
    DC_UNIFORM_TYPE_IDENTIFIER,
};

/// Leading 8-byte magic header for `.ctbnbdl` (`CtbNodeBundle`) files.
pub const CTBNBDL_MAGIC: [u8; 8] = *b"CTBNBDL\x01";

/// Standard 21-byte `DcMixed` (`f484`) header (`DcMM\0` + UUID `d5ab819e-9b8e-4d61-adbc-933e8851e79a`).
pub const DCMM_HEADER: [u8; 21] = [
    b'D', b'c', b'M', b'M', 0, 0xd5, 0xab, 0x81, 0x9e, 0x9b, 0x8e, 0x4d, 0x61,
    0xad, 0xbc, 0x93, 0x3e, 0x88, 0x51, 0xe7, 0x9a,
];

use crate::dc::{
    DC_ANNOTATION, DC_APPROXIMATE_DECOMPOSITION, DC_BEGIN_ASSOCIATIVITY_GROUP,
    DC_BEGIN_GRAPH_NODE_DEFINITION, DC_BEGIN_NUMBER, DC_BIDIRECTIONAL_CLASS,
    DC_BOOLEAN_TRUE, DC_CANONICAL_COMBINING_CLASS, DC_CANONICAL_DECOMPOSITION,
    DC_CANONICAL_NAME, DC_CASE_MAPPING_PARTNER, DC_CHARACTER_GENERAL_CATEGORY,
    DC_CHARACTER_SCRIPT, DC_COMPATIBILITY_DECOMPOSITION, DC_CROSS_REFERENCE,
    DC_DATA_FORMAT_DESCRIPTION, DC_DC_GENERAL_CATEGORY, DC_DESCRIPTION,
    DC_ENCLOSING_DOCUMENT, DC_END_ASSOCIATIVITY_GROUP,
    DC_END_GRAPH_NODE_DEFINITION, DC_END_NUMBER, DC_ENTITY_TYPE, DC_ESCAPE,
    DC_EVALUATE_VALUE_AS_ATOM, DC_FORMAL_ALIAS, DC_HAS_A, DC_HAS_VALUE,
    DC_INFORMAL_ALIAS, DC_IS_ASSOCIATED_WITH, DC_IS_BASED_ON,
    DC_IS_CLOSING_DELIMITER_FOR, DC_IS_DEPRECATED, DC_IS_INSTANCE_OF,
    DC_IS_SUBTYPE_OF, DC_MAGIC_SIGNATURE, DC_NEGATED_PATTERN,
    DC_PATTERN_ALTERNATION, DC_PATTERN_CAPTURE, DC_PATTERN_REPLACEMENT,
    DC_PATTERN_REPETITION, DC_PREFERRED, DC_PROGRAMMATIC_IDENTIFIER, DC_RANGE,
    DC_REFERENCE_NAMED_OBJECT, DC_REPLACEMENT, DC_RETRACTS,
    DC_ROUTINE_INVOCATION, DC_SELF_REFERENCE, DC_SEMANTIC_DECOMPOSITION,
    DC_SUBPATTERN_MATCH, DC_SUPERSEDES, DC_SYNTAX_RULE, DC_UNICODE_SCRIPT,
    DcChar,
};
use crate::dc_def::DcDefn;
use crate::dc_number_minimal::u128_to_dc_number_chars;
use crate::format_spec::{DcToken, FormatExpr, encode_dc_stream, parse_format_expr};
use crate::report::ValidationReport;
use crate::shared::{BidiClass, GeneralCategory};
use crate::syntax::ast::{
    ActionArg, CharTarget, DcSyntaxRule, Quantifier, SyntaxAction,
    SyntaxElement, SyntaxPattern, SyntaxTerm,
};
use crate::syntax::parse_dc_syntax;

/// Maximum number of node declarations stored in a single `.ctbnodes` chunk file.
pub const MAX_NODES_PER_CHUNK: usize = 100;

/// Converts a canonical `DcChar` constant into its `DcShorthand` representation.
#[must_use]
fn sh(dc: DcChar) -> DcShorthand {
    DcShorthand::from_global_id(dc.to_u128())
}

/// Single node entry in the System-block `.ctbnodes` ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerNode {
    /// Global node ID in `SYSTEM_REGION_START..=SYSTEM_REGION_END`.
    pub id: u128,
    /// Ordered token sequence defining this node.
    pub tokens: Vec<DcShorthand>,
    /// Human-readable comment rendered after `# `.
    pub comment: String,
}

impl LedgerNode {
    /// Formats this node as a single `.ctbnodes` line (`l<id>: <tokens> # <comment>`).
    #[must_use]
    pub fn format_line(&self) -> String {
        let token_strs: Vec<String> =
            self.tokens.iter().map(ToString::to_string).collect();
        let body = token_strs.join(" ");
        let clean_comment = sanitize_comment(&self.comment);
        match (body.is_empty(), clean_comment.is_empty()) {
            (true, true) => format!("l{}: #", self.id),
            (true, false) => format!("l{}: # {clean_comment}", self.id),
            (false, true) => format!("l{}: {body}", self.id),
            (false, false) => format!("l{}: {body} # {clean_comment}", self.id),
        }
    }

    /// Parses a single non-empty `.ctbnodes` line.
    pub fn parse_line(line: &str, file_label: &str, line_no: usize) -> Result<Self> {
        let trimmed = line.trim();
        ensure!(
            !trimmed.is_empty(),
            "{file_label}:{line_no}: unexpected blank line in .ctbnodes file"
        );
        let (code_part, comment) = match trimmed.split_once('#') {
            Some((before, after)) => (before.trim(), after.trim().to_string()),
            None => (trimmed, String::new()),
        };
        let Some((lhs, rhs)) = code_part.split_once(':') else {
            bail!(
                "{file_label}:{line_no}: missing ':' separator in node declaration '{trimmed}'"
            );
        };
        let lhs_shorthand = DcShorthand::parse(lhs.trim()).with_context(|| {
            format!("{file_label}:{line_no}: invalid node ID shorthand '{lhs}'")
        })?;
        let DcShorthand::Long(id) = lhs_shorthand else {
            bail!(
                "{file_label}:{line_no}: System-block node ID must use 'l<digits>' syntax, found '{lhs}'"
            );
        };
        ensure!(
            (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&id),
            "{file_label}:{line_no}: node ID l{id} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
        );
        let mut tokens = Vec::new();
        for raw_tok in rhs.split_whitespace() {
            let tok = DcShorthand::parse(raw_tok).with_context(|| {
                format!(
                    "{file_label}:{line_no}: invalid token shorthand '{raw_tok}'"
                )
            })?;
            tokens.push(tok);
        }
        Ok(Self {
            id,
            tokens,
            comment,
        })
    }
}

/// Statistics returned by a ledger update operation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LedgerUpdateStats {
    /// Total nodes in the ledger after the update.
    pub total_nodes: usize,
    /// Number of new nodes appended during this run.
    pub nodes_appended: usize,
    /// Number of `591` (`Supersedes`) revision nodes appended during this run.
    pub nodes_superseded: usize,
    /// Number of `592` (`Retracts`) retraction nodes appended during this run.
    pub nodes_retracted: usize,
    /// Number of `.ctbnodes` chunk files written or updated on disk.
    pub files_written: usize,
}

/// Seed entity key identifying a classification entity in the System block.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SeedEntityKey {
    BidiClass(String),
    GeneralCategory(String),
    Script(String),
    NamedConstruct(String),
}

impl SeedEntityKey {
    fn class_dc(&self) -> DcChar {
        match self {
            Self::BidiClass(_) => DC_BIDIRECTIONAL_CLASS,
            Self::GeneralCategory(code) => {
                if code == "!Cx" {
                    DC_DC_GENERAL_CATEGORY
                } else {
                    DC_CHARACTER_GENERAL_CATEGORY
                }
            }
            Self::Script(name) => {
                if name.starts_with('.') {
                    DC_CHARACTER_SCRIPT
                } else {
                    DC_UNICODE_SCRIPT
                }
            }
            Self::NamedConstruct(_) => DC_ENTITY_TYPE,
        }
    }

    fn label_prop_dc(&self) -> DcChar {
        match self {
            Self::BidiClass(_) | Self::GeneralCategory(_) => {
                DC_PROGRAMMATIC_IDENTIFIER
            }
            Self::Script(_) | Self::NamedConstruct(_) => DC_CANONICAL_NAME,
        }
    }

    fn label(&self) -> &str {
        match self {
            Self::BidiClass(s)
            | Self::GeneralCategory(s)
            | Self::Script(s)
            | Self::NamedConstruct(s) => s.as_str(),
        }
    }

    fn comment(&self) -> String {
        match self {
            Self::BidiClass(code) => format!("Bidi class {code}"),
            Self::GeneralCategory(code) => format!("General category {code}"),
            Self::Script(name) => format!("Script {name}"),
            Self::NamedConstruct(name) => {
                format!("Named syntax construct {name}")
            }
        }
    }
}

/// Desired compilation state derived from the CSV dataset.
#[derive(Debug, Default)]
struct DesiredDatasetState {
    /// Ordered seed entities to ensure exist in the ledger.
    seed_entities: BTreeSet<SeedEntityKey>,
    /// Named syntax construct rules keyed by construct name.
    named_construct_rules: BTreeMap<String, (SyntaxPattern, String)>,
    /// Single-valued properties: `(subject, prop_token) -> (value_spec, comment)`.
    single_properties: BTreeMap<(DcShorthand, DcShorthand), (ValueSpec, String)>,
    /// Multi-valued properties: `(subject, prop_token, value_spec) -> comment`.
    multi_properties: BTreeMap<(DcShorthand, DcShorthand, ValueSpec), String>,
    /// Preferred replacements (`<prefer>`): `(subject, value_spec) -> comment`.
    preferred_replacements: BTreeMap<(DcShorthand, ValueSpec), String>,
    /// Direct 3-token binary statements: `(verb, subject, object_spec) -> comment`.
    direct_statements: BTreeMap<(DcShorthand, DcShorthand, ValueSpec), String>,
}

/// Specification of a value or target referenced in a statement.
#[derive(Debug, Clone)]
enum ValueSpec {
    /// Direct reference to an existing Dc/Format/Unicode shorthand.
    Direct(DcShorthand),
    /// Reference to a seed classification entity node.
    SeedEntity(SeedEntityKey),
    /// Deduplicated value node with the given token sequence and comment.
    ValueNode {
        tokens: Vec<DcShorthand>,
        comment: String,
    },
    /// Deferred `.Semantic:Pattern` rule value node (resolved once seed entities are assigned).
    PatternNode {
        rule: Box<DcSyntaxRule>,
        comment: String,
    },
    /// Deferred `FormatExpr` value node (resolved once seed entities are assigned).
    FormatExprNode {
        expr: Box<FormatExpr>,
        raw_comment: String,
    },
}

impl ValueSpec {
    fn sort_key(&self) -> (u8, String, &[DcShorthand]) {
        match self {
            Self::Direct(shorthand) => (0, shorthand.to_string(), &[]),
            Self::SeedEntity(key) => (1, key.comment(), &[]),
            Self::ValueNode { tokens, comment } => (2, comment.clone(), tokens),
            Self::PatternNode { rule, comment } => {
                (3, format!("{}|{comment}", rule.raw), &[])
            }
            Self::FormatExprNode { raw_comment, .. } => {
                (4, raw_comment.clone(), &[])
            }
        }
    }
}

impl PartialEq for ValueSpec {
    fn eq(&self, other: &Self) -> bool {
        self.sort_key() == other.sort_key()
    }
}

impl Eq for ValueSpec {}

impl PartialOrd for ValueSpec {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueSpec {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl std::hash::Hash for ValueSpec {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.sort_key().hash(state);
    }
}

fn sanitize_comment(raw: &str) -> String {
    raw.replace('\r', "\\r").replace('\n', "\\n")
}

/// Encodes a UTF-8 string into the canonical string value node token sequence
/// (`<u<hex> tokens...>`).
#[must_use]
pub fn encode_string_value_tokens(text: &str) -> Vec<DcShorthand> {
    text.chars()
        .map(|ch| DcShorthand::Unicode(u32::from(ch)))
        .collect()
}

/// Decodes a canonical string value node token sequence back into a `String`, if it matches.
#[must_use]
pub fn decode_string_value_tokens(tokens: &[DcShorthand]) -> Option<String> {
    let mut out = String::with_capacity(tokens.len());
    for tok in tokens {
        let DcShorthand::Unicode(cp) = *tok else {
            return None;
        };
        let ch = char::from_u32(cp)?;
        out.push(ch);
    }
    Some(out)
}

/// Encodes an unsigned integer into the canonical base64 integer value node token sequence:
/// `6 f199 <base64 digit Dcs 127..=190...> 7`.
#[must_use]
pub fn encode_integer_value_tokens(value: u128) -> Vec<DcShorthand> {
    let digits = u128_to_dc_number_chars(value);
    let mut tokens = Vec::with_capacity(digits.len().saturating_add(3));
    tokens.push(sh(DC_BEGIN_NUMBER));
    tokens.push(sh(DC_INTEGER));
    for dc in digits {
        tokens.push(sh(dc));
    }
    tokens.push(sh(DC_END_NUMBER));
    tokens
}

fn format_string_summary(text: &str) -> String {
    format!("{text:?}")
}

fn make_string_value_spec(text: &str) -> ValueSpec {
    ValueSpec::ValueNode {
        tokens: encode_string_value_tokens(text),
        comment: format_string_summary(text),
    }
}

fn make_integer_value_spec(value: u128) -> ValueSpec {
    ValueSpec::ValueNode {
        tokens: encode_integer_value_tokens(value),
        comment: value.to_string(),
    }
}

fn build_initial_token_labels(
    dc_defns: &[DcDefn],
    format_defns: &[DcDefn],
) -> HashMap<DcShorthand, String> {
    let mut map =
        HashMap::with_capacity(dc_defns.len().saturating_add(format_defns.len()));
    for defn in dc_defns.iter().chain(format_defns.iter()) {
        if !defn.name.is_empty() {
            map.insert(
                DcShorthand::from_global_id(defn.dc_id),
                defn.name.clone(),
            );
        }
    }
    // Ensure semantic relationship/pattern Dcs have labels even in small unit test fixtures
    let builtins: &[(DcChar, &str)] = &[
        (DC_RANGE, "Range"),
        (DC_BEGIN_NUMBER, "Begin number"),
        (DC_END_NUMBER, "End number"),
        (DC_INTEGER, "Integer number"),
        (DC_REFERENCE_NAMED_OBJECT, "Reference named object"),
        (DC_ROUTINE_INVOCATION, "Routine invocation"),
        (DC_BEGIN_ASSOCIATIVITY_GROUP, "Begin associativity group"),
        (DC_END_ASSOCIATIVITY_GROUP, "End associativity group"),
        (DC_BOOLEAN_TRUE, "Boolean true"),
        (DC_ENTITY_TYPE, "Type"),
        (DC_REPLACEMENT, "Replacement"),
        (DC_PREFERRED, "Preferred"),
        (DC_CANONICAL_NAME, "Canonical name"),
        (DC_DESCRIPTION, "Description"),
        (DC_PROGRAMMATIC_IDENTIFIER, "Programmatic identifier"),
        (DC_INFORMAL_ALIAS, "Informal alias"),
        (DC_FORMAL_ALIAS, "Formal alias"),
        (DC_ANNOTATION, "Annotation"),
        (DC_MAGIC_SIGNATURE, "Magic signature"),
        (
            DC_CANONICAL_COMBINING_CLASS,
            "Unicode canonical combining class",
        ),
        (DC_BIDIRECTIONAL_CLASS, "Unicode bidirectional class"),
        (DC_CASE_MAPPING_PARTNER, "Case mapping partner"),
        (DC_CHARACTER_GENERAL_CATEGORY, "Unicode general category"),
        (DC_HAS_A, "Has a"),
        (DC_IS_INSTANCE_OF, "Is an instance of"),
        (DC_IS_SUBTYPE_OF, "Is a subtype of"),
        (DC_IS_BASED_ON, "Is based on"),
        (DC_IS_DEPRECATED, "Is deprecated"),
        (
            DC_IS_CLOSING_DELIMITER_FOR,
            "Is a closing delimiter of syntactic block for",
        ),
        (DC_SUPERSEDES, "Supersedes"),
        (DC_RETRACTS, "Retracts"),
        (DC_CROSS_REFERENCE, "Cross-reference"),
        (DC_SYNTAX_RULE, "Syntax rule"),
        (
            DC_DATA_FORMAT_DESCRIPTION,
            "Data format or type description",
        ),
        (DC_CHARACTER_SCRIPT, "Dc Script"),
        (DC_IS_ASSOCIATED_WITH, "Is associated with"),
        (DC_SELF_REFERENCE, "Self reference"),
        (DC_PATTERN_ALTERNATION, "Pattern alternation"),
        (DC_PATTERN_REPETITION, "Pattern repetition"),
        (DC_NEGATED_PATTERN, "Negated pattern"),
        (DC_SUBPATTERN_MATCH, "Subpattern match"),
        (DC_PATTERN_CAPTURE, "Pattern capture"),
        (DC_PATTERN_REPLACEMENT, "Pattern replacement"),
        (DC_DC_GENERAL_CATEGORY, "Dc general category"),
        (DC_UNICODE_SCRIPT, "Unicode script"),
        (DC_HAS_VALUE, "Has value"),
        (DC_CANONICAL_DECOMPOSITION, "Canonical decomposition"),
        (
            DC_COMPATIBILITY_DECOMPOSITION,
            "Compatibility decomposition",
        ),
        (DC_APPROXIMATE_DECOMPOSITION, "Approximate decomposition"),
        (DC_SEMANTIC_DECOMPOSITION, "Semantic decomposition"),
        (DC_ENCLOSING_DOCUMENT, "Enclosing document"),
        (DC_EVALUATE_VALUE_AS_ATOM, "Evaluate value as atom"),
        (
            DC_BEGIN_GRAPH_NODE_DEFINITION,
            "Begin graph node definition",
        ),
        (DC_END_GRAPH_NODE_DEFINITION, "End graph node definition"),
        (DC_ESCAPE, "Ignore following Dc"),
        (DC_FILE_EXTENSION, "File extension"),
        (DC_MEDIA_TYPE, "IANA Media Type"),
        (DC_UNIFORM_TYPE_IDENTIFIER, "Uniform Type Identifier"),
        (DC_APPLE_OS_TYPE, "Apple OSType"),
    ];
    for &(dc, label) in builtins {
        map.entry(sh(dc)).or_insert_with(|| label.to_string());
    }
    map
}

/// Formats a global node ID into a `.ctbnodes` chunk filename with `_` grouping
/// every 5 digits from the right (e.g., `8589934592` -> `"85899_34592.ctbnodes"`).
#[must_use]
pub fn format_chunk_filename(first_id: u128) -> String {
    let digits = first_id.to_string();
    let len = digits.len();
    let mut grouped =
        String::with_capacity(len.saturating_add(len.checked_div(5).unwrap_or(0)));
    for (idx, ch) in digits.chars().enumerate() {
        let remaining = len.saturating_sub(idx);
        if idx > 0 && remaining.is_multiple_of(5) {
            grouped.push('_');
        }
        grouped.push(ch);
    }
    format!("{grouped}.ctbnodes")
}

/// Parses a `.ctbnodes` chunk filename (e.g., `"85899_34592.ctbnodes"`) into its starting node ID.
#[must_use]
pub fn parse_chunk_filename_start_id(filename: &str) -> Option<u128> {
    let stem = filename.strip_suffix(".ctbnodes")?;
    if stem.is_empty() || stem.starts_with('_') || stem.ends_with('_') {
        return None;
    }
    let mut clean = String::with_capacity(stem.len());
    for ch in stem.chars() {
        if ch.is_ascii_digit() {
            clean.push(ch);
        } else if ch == '_' {
            // separator
        } else {
            return None;
        }
    }
    if clean.is_empty() {
        return None;
    }
    clean.parse::<u128>().ok()
}

/// Returns the sorted list of `.ctbnodes` chunk files in `graph_dir`.
pub fn list_chunk_files(graph_dir: &Path) -> Result<Vec<PathBuf>> {
    if !graph_dir.exists() {
        return Ok(Vec::new());
    }
    let mut keyed_files: Vec<(u128, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(graph_dir).with_context(|| {
        format!("Failed to read graph directory {}", graph_dir.display())
    })? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if let Some(start_id) = parse_chunk_filename_start_id(name) {
            if (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&start_id) {
                keyed_files.push((start_id, path));
            }
        }
    }
    keyed_files.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    Ok(keyed_files.into_iter().map(|(_, p)| p).collect())
}

/// Loads all existing `LedgerNode` entries from `.ctbnodes` chunk files in `graph_dir`.
pub fn load_ledger_nodes_from_dir(graph_dir: &Path) -> Result<Vec<LedgerNode>> {
    let files = list_chunk_files(graph_dir)?;
    let mut nodes = Vec::new();
    for (chunk_idx, path) in files.iter().enumerate() {
        let chunk_start_id = SYSTEM_REGION_START.saturating_add(
            u128::try_from(chunk_idx.saturating_mul(MAX_NODES_PER_CHUNK))
                .unwrap_or(0),
        );
        let expected_name = format_chunk_filename(chunk_start_id);
        let actual_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        ensure!(
            actual_name == expected_name,
            "Expected sequential chunk file '{expected_name}', found '{actual_name}'"
        );
        let content = std::fs::read_to_string(path).with_context(|| {
            format!("Failed to read chunk file {}", path.display())
        })?;
        let mut chunk_count = 0usize;
        for (idx, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let line_no = idx.saturating_add(1);
            let node = LedgerNode::parse_line(line, actual_name, line_no)?;
            let expected_id = SYSTEM_REGION_START
                .saturating_add(u128::try_from(nodes.len()).unwrap_or(0));
            ensure!(
                node.id == expected_id,
                "{actual_name}:{line_no}: expected sequential node ID l{expected_id}, found l{}",
                node.id
            );
            nodes.push(node);
            chunk_count = chunk_count.saturating_add(1);
        }
        ensure!(
            chunk_count <= MAX_NODES_PER_CHUNK,
            "{actual_name}: chunk contains {chunk_count} nodes, exceeding maximum {MAX_NODES_PER_CHUNK}"
        );
        if chunk_idx.saturating_add(1) < files.len() {
            ensure!(
                chunk_count == MAX_NODES_PER_CHUNK,
                "{actual_name}: non-final chunk has {chunk_count} nodes, expected {MAX_NODES_PER_CHUNK}"
            );
        }
    }
    Ok(nodes)
}

/// Writes `nodes` into 100-node chunk files (`85899_34592.ctbnodes`, ...) in `graph_dir`,
/// updating only files whose contents changed. Returns the number of files written.
pub fn write_ledger_nodes_to_dir(
    graph_dir: &Path,
    nodes: &[LedgerNode],
) -> Result<usize> {
    std::fs::create_dir_all(graph_dir).with_context(|| {
        format!("Failed to create graph directory {}", graph_dir.display())
    })?;
    let mut files_written = 0usize;
    for chunk in nodes.chunks(MAX_NODES_PER_CHUNK) {
        let Some(first_node) = chunk.first() else {
            continue;
        };
        let file_path = graph_dir.join(format_chunk_filename(first_node.id));
        let mut content = String::new();
        for node in chunk {
            content.push_str(&node.format_line());
            content.push('\n');
        }
        if crate::updater::write_if_changed(&file_path, &content)? {
            files_written = files_written.saturating_add(1);
        }
    }
    Ok(files_written)
}

/// Encodes a single `LedgerNode` (`id` + `tokens`) into its `Dc 617 .. Dc 618` token sequence:
/// `617 [atom_dc: l<id>] <escaped body tokens...> 618`.
#[must_use]
pub fn encode_node_definition_tokens(node: &LedgerNode) -> Vec<DcShorthand> {
    let mut out = Vec::with_capacity(node.tokens.len().saturating_add(4));
    out.push(sh(DC_BEGIN_GRAPH_NODE_DEFINITION));
    push_atom_dc(&mut out, DcShorthand::from_global_id(node.id));
    for &tok in &node.tokens {
        if tok == sh(DC_ESCAPE)
            || tok == sh(DC_END_GRAPH_NODE_DEFINITION)
        {
            out.push(sh(DC_ESCAPE));
        }
        out.push(tok);
    }
    out.push(sh(DC_END_GRAPH_NODE_DEFINITION));
    out
}

/// Compiles a slice of `LedgerNode` entries into a `.ctbnbdl` (`CtbNodeBundle`) binary buffer:
/// 8-byte `CTBNBDL\x01` magic + 21-byte `DcMM` header + `UTF-8e-128` encoded `617 .. 618` frames.
pub fn encode_nodes_to_ctbnbdl_bytes(nodes: &[LedgerNode]) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(
        CTBNBDL_MAGIC
            .len()
            .saturating_add(DCMM_HEADER.len())
            .saturating_add(nodes.len().saturating_mul(32)),
    );
    bytes.extend_from_slice(&CTBNBDL_MAGIC);
    bytes.extend_from_slice(&DCMM_HEADER);
    let mut char_buf = [0u8; 24];
    for node in nodes {
        for tok in encode_node_definition_tokens(node) {
            let gid = tok.to_global_id()?;
            let n = encode_utf_8e_128_buf(&mut char_buf, gid);
            if let Some(slice) = char_buf.get(..n) {
                bytes.extend_from_slice(slice);
            }
        }
    }
    Ok(bytes)
}

/// Decodes a `.ctbnbdl` (`CtbNodeBundle`) binary buffer back into a list of `LedgerNode` entries
/// (with empty comments, as `.ctbnbdl` stores only the semantic Dc stream).
pub fn decode_nodes_from_ctbnbdl_bytes(bytes: &[u8]) -> Result<Vec<LedgerNode>> {
    let header_len = CTBNBDL_MAGIC.len().saturating_add(DCMM_HEADER.len());
    ensure!(
        bytes.len() >= header_len,
        ".ctbnbdl buffer is shorter than the {header_len}-byte header ({} bytes)",
        bytes.len()
    );
    let magic_part = bytes.get(..CTBNBDL_MAGIC.len()).unwrap_or_default();
    ensure!(
        magic_part == CTBNBDL_MAGIC,
        "Invalid .ctbnbdl magic bytes: expected {:?}, found {:?}",
        CTBNBDL_MAGIC,
        magic_part
    );
    let dcmm_part = bytes
        .get(CTBNBDL_MAGIC.len()..header_len)
        .unwrap_or_default();
    ensure!(
        dcmm_part == DCMM_HEADER,
        "Invalid DcMM header in .ctbnbdl buffer"
    );

    let mut payload = bytes.get(header_len..).unwrap_or_default();
    let mut all_tokens = Vec::new();
    while !payload.is_empty() {
        let Some((codepoint, consumed)) = decode_utf_8e_128_buf(payload) else {
            bail!("Invalid UTF-8e-128 sequence in .ctbnbdl stream");
        };
        all_tokens.push(DcShorthand::from_global_id(codepoint));
        payload = payload.get(consumed..).unwrap_or_default();
    }

    let mut nodes = Vec::new();
    let mut idx = 0usize;
    let sh_begin = sh(DC_BEGIN_GRAPH_NODE_DEFINITION);
    let sh_end = sh(DC_END_GRAPH_NODE_DEFINITION);
    let sh_escape = sh(DC_ESCAPE);

    while idx < all_tokens.len() {
        let open_tok = *all_tokens
            .get(idx)
            .context("Unexpected end of .ctbnbdl token stream")?;
        ensure!(
            open_tok == sh_begin,
            "Expected Dc 617 (Begin graph node definition) at token index {idx}, found {open_tok}"
        );
        idx = idx.saturating_add(1);
        let id_sh = consume_atom_dc(&all_tokens, &mut idx)
            .context("Invalid or missing [atom_dc] node ID after Dc 617")?;
        let id = id_sh.to_global_id()?;
        let mut body_tokens = Vec::new();
        let mut closed = false;
        while idx < all_tokens.len() {
            let tok = *all_tokens
                .get(idx)
                .context("Unexpected end of node definition in .ctbnbdl")?;
            idx = idx.saturating_add(1);
            if tok == sh_end {
                closed = true;
                break;
            }
            if tok == sh_escape {
                let escaped = *all_tokens.get(idx).context(
                    "Trailing Dc 255 escape at end of .ctbnbdl token stream",
                )?;
                ensure!(
                    escaped == sh_escape || escaped == sh_end,
                    "Unexpected escaped token {escaped} after Dc 255 in .ctbnbdl node body"
                );
                idx = idx.saturating_add(1);
                body_tokens.push(escaped);
            } else {
                body_tokens.push(tok);
            }
        }
        ensure!(
            closed,
            "Unterminated Dc 617 node definition for l{id} (missing Dc 618)"
        );
        nodes.push(LedgerNode {
            id,
            tokens: body_tokens,
            comment: String::new(),
        });
    }
    Ok(nodes)
}

/// Active index of a loaded ledger, tracking value nodes, seed entities, and active statements.
struct LedgerBuilder {
    nodes: Vec<LedgerNode>,
    appended_count: usize,
    superseded_count: usize,
    retracted_count: usize,
    /// Canonical human-readable names for Dc, Format, and allocated Long node tokens.
    token_labels: HashMap<DcShorthand, String>,
    /// Decoded UTF-8 strings for string value nodes, keyed by `u128` node ID.
    string_by_node_id: HashMap<u128, String>,
    /// Maps a value node's canonical token sequence to its allocated `u128` node ID.
    value_node_by_tokens: HashMap<Vec<DcShorthand>, u128>,
    /// Maps a `SeedEntityKey` to its allocated `u128` node ID.
    seed_entity_ids: HashMap<SeedEntityKey, u128>,
    /// Active single-valued properties: `(subject, prop_token) -> (has_a_id, has_val_id, current_val_token)`.
    active_single_props:
        BTreeMap<(DcShorthand, DcShorthand), (u128, u128, DcShorthand)>,
    /// Active multi-valued properties: `(subject, prop_token, val_token) -> (has_a_id, has_val_id)`.
    active_multi_props:
        BTreeMap<(DcShorthand, DcShorthand, DcShorthand), (u128, u128)>,
    /// Active `<prefer>` replacements: `(subject, val_token) -> (has_a_id, has_val_id, pref_qual_id)`.
    active_preferred: BTreeMap<(DcShorthand, DcShorthand), (u128, u128, u128)>,
    /// Active direct binary statements: `(verb, subject, object) -> stmt_id`.
    active_direct_stmts: BTreeMap<(DcShorthand, DcShorthand, DcShorthand), u128>,
}

impl LedgerBuilder {
    /// Initializes a `LedgerBuilder` from existing ledger nodes and indexes all active statements.
    fn from_existing(
        existing: Vec<LedgerNode>,
        single_valued_props: &HashSet<DcShorthand>,
        initial_token_labels: HashMap<DcShorthand, String>,
    ) -> Self {
        let mut token_labels = initial_token_labels;
        let sh_supersedes = sh(DC_SUPERSEDES);
        let sh_retracts = sh(DC_RETRACTS);
        let sh_has_value = sh(DC_HAS_VALUE);
        let sh_has_a = sh(DC_HAS_A);
        let sh_preferred = sh(DC_PREFERRED);
        let sh_replacement = sh(DC_REPLACEMENT);
        let sh_is_instance_of = sh(DC_IS_INSTANCE_OF);
        let sh_enclosing_doc = sh(DC_ENCLOSING_DOCUMENT);
        let sh_prog_ident = sh(DC_PROGRAMMATIC_IDENTIFIER);
        let sh_canon_name = sh(DC_CANONICAL_NAME);
        let sh_bidi_class = sh(DC_BIDIRECTIONAL_CLASS);
        let sh_char_gen_cat = sh(DC_CHARACTER_GENERAL_CATEGORY);
        let sh_dc_gen_cat = sh(DC_DC_GENERAL_CATEGORY);
        let sh_char_script = sh(DC_CHARACTER_SCRIPT);
        let sh_uni_script = sh(DC_UNICODE_SCRIPT);
        let sh_entity_type = sh(DC_ENTITY_TYPE);

        let mut inactive_ids = HashSet::new();
        let mut node_by_id: HashMap<u128, &LedgerNode> =
            HashMap::with_capacity(existing.len());
        for node in &existing {
            node_by_id.insert(node.id, node);
            if let Some((verb, _subj, obj)) =
                parse_statement_tokens(&node.tokens)
            {
                // 591 <new_node> l<old_node> -> old_node is superseded
                // 592 615 l<old_node> -> old_node is retracted
                if verb == sh_supersedes || verb == sh_retracts {
                    if let DcShorthand::Long(old_id) = obj {
                        inactive_ids.insert(old_id);
                    }
                }
            }
        }

        // First pass: index value nodes and find active `610` (`Has value`) and `584 <s1> 570` (`Preferred`) nodes.
        let mut value_node_by_tokens = HashMap::new();
        let mut string_by_node_id: HashMap<u128, String> = HashMap::new();
        let mut active_has_val_by_stmt: HashMap<u128, (u128, DcShorthand)> =
            HashMap::new();
        let mut active_pref_qual_by_stmt: HashMap<u128, u128> = HashMap::new();

        for node in &existing {
            if let Some(decoded) = decode_string_value_tokens(&node.tokens) {
                token_labels.insert(
                    DcShorthand::Long(node.id),
                    format_string_summary(&decoded),
                );
                string_by_node_id.insert(node.id, decoded);
            }
            let stmt_opt = parse_statement_tokens(&node.tokens);
            if stmt_opt.is_none() {
                value_node_by_tokens
                    .entry(node.tokens.clone())
                    .or_insert(node.id);
            }
            if inactive_ids.contains(&node.id) {
                continue;
            }
            if let Some((verb, subj, obj)) = stmt_opt {
                if verb == sh_has_value {
                    if let DcShorthand::Long(has_a_id) = subj {
                        if !inactive_ids.contains(&has_a_id) {
                            active_has_val_by_stmt
                                .insert(has_a_id, (node.id, obj));
                        }
                    }
                } else if verb == sh_has_a && obj == sh_preferred {
                    if let DcShorthand::Long(has_a_id) = subj {
                        if !inactive_ids.contains(&has_a_id) {
                            active_pref_qual_by_stmt.insert(has_a_id, node.id);
                        }
                    }
                }
            }
        }

        // Second pass: reconstruct seed entities and active statements.
        let mut seed_candidate_class: HashMap<u128, DcShorthand> =
            HashMap::new();
        let mut seed_entity_ids = HashMap::new();
        let mut active_single_props = BTreeMap::new();
        let mut active_multi_props = BTreeMap::new();
        let mut active_preferred = BTreeMap::new();
        let mut active_direct_stmts = BTreeMap::new();

        for node in &existing {
            if inactive_ids.contains(&node.id) {
                continue;
            }
            let Some((verb, subj, obj)) = parse_statement_tokens(&node.tokens)
            else {
                continue;
            };

            // Seed entity declaration: `586 615 <class_dc>` where `<class_dc>` is a Short Dc.
            if verb == sh_is_instance_of
                && subj == sh_enclosing_doc
                && matches!(obj, DcShorthand::Short(_))
            {
                seed_candidate_class.insert(node.id, obj);
                continue;
            }

            if verb == sh_has_value
                || verb == sh_supersedes
                || verb == sh_retracts
            {
                continue;
            }
            if verb == sh_has_a && obj == sh_preferred {
                if let DcShorthand::Long(target_id) = subj {
                    if node_by_id.get(&target_id).is_some_and(|n| {
                        parse_statement_tokens(&n.tokens).is_some_and(
                            |(v, _, o)| v == sh_has_a && o == sh_replacement,
                        )
                    }) {
                        continue;
                    }
                }
            }

            if verb == sh_has_a {
                if let Some(&(has_val_id, val_tok)) =
                    active_has_val_by_stmt.get(&node.id)
                {
                    // Check if this names a seed entity (`584 l<seed_id> 572/574` -> `610 ... l<str_id>`).
                    if let (
                        DcShorthand::Long(seed_id),
                        DcShorthand::Long(str_id),
                    ) = (subj, val_tok)
                    {
                        if let (Some(&class_sh), Some(label)) = (
                            seed_candidate_class.get(&seed_id),
                            string_by_node_id.get(&str_id),
                        ) {
                            let key_opt = if class_sh == sh_bidi_class
                                && obj == sh_prog_ident
                            {
                                Some(SeedEntityKey::BidiClass(label.clone()))
                            } else if (class_sh == sh_char_gen_cat
                                || class_sh == sh_dc_gen_cat)
                                && obj == sh_prog_ident
                            {
                                Some(SeedEntityKey::GeneralCategory(
                                    label.clone(),
                                ))
                            } else if (class_sh == sh_char_script
                                || class_sh == sh_uni_script)
                                && obj == sh_canon_name
                            {
                                Some(SeedEntityKey::Script(label.clone()))
                            } else if class_sh == sh_entity_type
                                && obj == sh_canon_name
                            {
                                Some(SeedEntityKey::NamedConstruct(
                                    label.clone(),
                                ))
                            } else {
                                None
                            };
                            if let Some(seed_key) = key_opt {
                                token_labels.insert(
                                    DcShorthand::Long(seed_id),
                                    seed_key.comment(),
                                );
                                seed_entity_ids.insert(seed_key, seed_id);
                            }
                        }
                    }

                    if obj == sh_replacement {
                        if let Some(&pref_id) =
                            active_pref_qual_by_stmt.get(&node.id)
                        {
                            active_preferred.insert(
                                (subj, val_tok),
                                (node.id, has_val_id, pref_id),
                            );
                            continue;
                        }
                    }

                    if single_valued_props.contains(&obj) {
                        active_single_props
                            .insert((subj, obj), (node.id, has_val_id, val_tok));
                    } else {
                        active_multi_props
                            .insert((subj, obj, val_tok), (node.id, has_val_id));
                    }
                }
            } else {
                active_direct_stmts.insert((verb, subj, obj), node.id);
            }
        }

        Self {
            nodes: existing,
            appended_count: 0,
            superseded_count: 0,
            retracted_count: 0,
            token_labels,
            string_by_node_id,
            value_node_by_tokens,
            seed_entity_ids,
            active_single_props,
            active_multi_props,
            active_preferred,
            active_direct_stmts,
        }
    }

    fn describe_token(&self, tok: DcShorthand) -> String {
        if let Some(label) = self.token_labels.get(&tok) {
            return label.clone();
        }
        match tok {
            DcShorthand::Unicode(cp) => format!("U+{cp:04X}"),
            _ => tok.to_string(),
        }
    }

    fn describe_atom_dc_tokens(&self, tok: DcShorthand) -> String {
        if tok == sh(DC_ESCAPE) || tok == sh(DC_EVALUATE_VALUE_AS_ATOM) {
            format!(
                "{}, {}",
                self.describe_token(sh(DC_ESCAPE)),
                self.describe_token(tok)
            )
        } else {
            self.describe_token(tok)
        }
    }

    fn format_tokens_comment(&self, tokens: &[DcShorthand]) -> String {
        if let Some((verb, subj, obj)) = parse_statement_tokens(tokens) {
            return format!(
                "{} ({}, {})",
                self.describe_token(verb),
                self.describe_atom_dc_tokens(subj),
                self.describe_atom_dc_tokens(obj)
            );
        }
        if let Some(decoded) = decode_string_value_tokens(tokens) {
            return format_string_summary(&decoded);
        }
        let parts: Vec<String> =
            tokens.iter().map(|t| self.describe_token(*t)).collect();
        parts.join(", ")
    }

    fn append_raw_node(&mut self, tokens: Vec<DcShorthand>) -> Result<u128> {
        let next_id = SYSTEM_REGION_START
            .saturating_add(u128::try_from(self.nodes.len()).unwrap_or(0));
        ensure!(
            next_id <= SYSTEM_REGION_END,
            "System graph region exhausted at node ID l{next_id}"
        );
        let comment = self.format_tokens_comment(&tokens);
        self.nodes.push(LedgerNode {
            id: next_id,
            tokens,
            comment,
        });
        self.appended_count = self.appended_count.saturating_add(1);
        Ok(next_id)
    }

    fn append_statement_node(
        &mut self,
        verb: DcShorthand,
        subj: DcShorthand,
        obj: DcShorthand,
    ) -> Result<u128> {
        self.append_raw_node(encode_statement_tokens(verb, subj, obj))
    }

    fn get_or_insert_value_node(
        &mut self,
        tokens: Vec<DcShorthand>,
        summary_label: String,
    ) -> Result<u128> {
        if let Some(&existing_id) = self.value_node_by_tokens.get(&tokens) {
            self.token_labels
                .entry(DcShorthand::Long(existing_id))
                .or_insert(summary_label);
            return Ok(existing_id);
        }
        let id = self.append_raw_node(tokens.clone())?;
        self.value_node_by_tokens.insert(tokens, id);
        self.token_labels
            .insert(DcShorthand::Long(id), summary_label);
        Ok(id)
    }

    fn ensure_seed_entity(&mut self, key: &SeedEntityKey) -> Result<u128> {
        if let Some(&id) = self.seed_entity_ids.get(key) {
            return Ok(id);
        }
        let class_dc = key.class_dc();
        let entity_id = self.append_statement_node(
            sh(DC_IS_INSTANCE_OF),
            sh(DC_ENCLOSING_DOCUMENT),
            sh(class_dc),
        )?;
        self.seed_entity_ids.insert(key.clone(), entity_id);
        self.token_labels
            .insert(DcShorthand::Long(entity_id), key.comment());
        Ok(entity_id)
    }

    fn resolve_value_spec(
        &mut self,
        spec: &ValueSpec,
    ) -> Result<DcShorthand> {
        match spec {
            ValueSpec::Direct(shorthand) => Ok(*shorthand),
            ValueSpec::SeedEntity(key) => {
                let id = self.ensure_seed_entity(key)?;
                Ok(DcShorthand::Long(id))
            }
            ValueSpec::ValueNode { tokens, comment } => {
                let id = self
                    .get_or_insert_value_node(tokens.clone(), comment.clone())?;
                Ok(DcShorthand::Long(id))
            }
            ValueSpec::PatternNode { rule, comment } => {
                let tokens = self.encode_syntax_rule_tokens(rule)?;
                let id =
                    self.get_or_insert_value_node(tokens, comment.clone())?;
                Ok(DcShorthand::Long(id))
            }
            ValueSpec::FormatExprNode { expr, raw_comment } => {
                let tokens = self.encode_format_expr_tokens(expr)?;
                if tokens.len() == 1 {
                    if let Some(&single) = tokens.first() {
                        return Ok(single);
                    }
                }
                let id = self
                    .get_or_insert_value_node(tokens, raw_comment.clone())?;
                Ok(DcShorthand::Long(id))
            }
        }
    }

    fn encode_syntax_rule_tokens(
        &mut self,
        rule: &DcSyntaxRule,
    ) -> Result<Vec<DcShorthand>> {
        let mut tokens = Vec::new();
        self.encode_syntax_pattern_tokens(&rule.pattern, &mut tokens)?;
        if let Some(action) = &rule.action {
            self.encode_syntax_action_tokens(action, &mut tokens)?;
        }
        if tokens.is_empty() {
            tokens.push(sh(DC_SELF_REFERENCE));
        }
        Ok(tokens)
    }

    fn encode_syntax_pattern_tokens(
        &mut self,
        pattern: &SyntaxPattern,
        out: &mut Vec<DcShorthand>,
    ) -> Result<()> {
        match pattern {
            SyntaxPattern::Sequence(elements) => {
                for elem in elements {
                    self.encode_syntax_element_tokens(elem, out)?;
                }
            }
            SyntaxPattern::Alternation(branches) => {
                out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                out.push(sh(DC_PATTERN_ALTERNATION));
                for branch in branches {
                    out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                    self.encode_syntax_pattern_tokens(branch, out)?;
                    out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
                }
                out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
            }
        }
        Ok(())
    }

    fn encode_syntax_element_tokens(
        &mut self,
        elem: &SyntaxElement,
        out: &mut Vec<DcShorthand>,
    ) -> Result<()> {
        match &elem.term {
            SyntaxTerm::SelfChar => {
                out.push(sh(DC_SELF_REFERENCE));
            }
            SyntaxTerm::CharRef(target) => {
                out.push(DcShorthand::from_global_id(target.to_u128()));
            }
            SyntaxTerm::CharRange { start, end } => {
                out.push(sh(DC_RANGE));
                out.push(DcShorthand::from_global_id(start.to_u128()));
                out.push(DcShorthand::from_global_id(end.to_u128()));
            }
            SyntaxTerm::CharSet { negated, members } => {
                out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                out.push(sh(if *negated {
                    DC_NEGATED_PATTERN
                } else {
                    DC_PATTERN_ALTERNATION
                }));
                for member in members {
                    out.push(DcShorthand::from_global_id(member.to_u128()));
                }
                out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
            }
            SyntaxTerm::RuleRef { target } => {
                out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                out.push(sh(DC_SUBPATTERN_MATCH));
                out.push(DcShorthand::from_global_id(target.to_u128()));
                out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
            }
            SyntaxTerm::NamedConstruct {
                name,
                subtype,
                capture_var,
            } => {
                let construct_id = self.ensure_seed_entity(
                    &SeedEntityKey::NamedConstruct(name.clone()),
                )?;
                out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                out.push(sh(DC_SUBPATTERN_MATCH));
                out.push(DcShorthand::Long(construct_id));
                if let Some(sub) = subtype {
                    let sub_id = self.get_or_insert_value_node(
                        encode_string_value_tokens(sub),
                        format!("{sub:?}"),
                    )?;
                    out.push(sh(DC_HAS_A));
                    out.push(DcShorthand::Long(sub_id));
                }
                if let Some(var) = capture_var {
                    let var_id = self.get_or_insert_value_node(
                        encode_string_value_tokens(var),
                        format!("${var}"),
                    )?;
                    out.push(sh(DC_PATTERN_CAPTURE));
                    out.push(DcShorthand::Long(var_id));
                }
                out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
            }
            SyntaxTerm::Group(inner) => {
                out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                self.encode_syntax_pattern_tokens(inner, out)?;
                out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
            }
        }

        match elem.quantifier {
            Quantifier::ExactOne => {}
            Quantifier::ZeroOrMore => {
                let zero_id = self.get_or_insert_value_node(
                    encode_integer_value_tokens(0),
                    "0".to_string(),
                )?;
                out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                out.push(sh(DC_PATTERN_REPETITION));
                out.push(sh(DC_RANGE));
                out.push(DcShorthand::Long(zero_id));
                out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
            }
            Quantifier::OneOrMore => {
                let one_id = self.get_or_insert_value_node(
                    encode_integer_value_tokens(1),
                    "1".to_string(),
                )?;
                out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                out.push(sh(DC_PATTERN_REPETITION));
                out.push(sh(DC_RANGE));
                out.push(DcShorthand::Long(one_id));
                out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
            }
            Quantifier::Optional => {
                let zero_id = self.get_or_insert_value_node(
                    encode_integer_value_tokens(0),
                    "0".to_string(),
                )?;
                let one_id = self.get_or_insert_value_node(
                    encode_integer_value_tokens(1),
                    "1".to_string(),
                )?;
                out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                out.push(sh(DC_PATTERN_REPETITION));
                out.push(sh(DC_RANGE));
                out.push(DcShorthand::Long(zero_id));
                out.push(DcShorthand::Long(one_id));
                out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
            }
            Quantifier::Range { min, max } => {
                // Reason for fallback: usize quantifier bounds always fit in u128 on all targets.
                let min_u128 = u128::try_from(min).unwrap_or(0);
                let min_id = self.get_or_insert_value_node(
                    encode_integer_value_tokens(min_u128),
                    min.to_string(),
                )?;
                out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
                out.push(sh(DC_PATTERN_REPETITION));
                out.push(sh(DC_RANGE));
                out.push(DcShorthand::Long(min_id));
                if let Some(max_val) = max {
                    // Reason for fallback: usize quantifier bounds always fit in u128 on all targets.
                    let max_u128 = u128::try_from(max_val).unwrap_or(0);
                    let max_id = self.get_or_insert_value_node(
                        encode_integer_value_tokens(max_u128),
                        max_val.to_string(),
                    )?;
                    out.push(DcShorthand::Long(max_id));
                }
                out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
            }
        }
        Ok(())
    }

    fn encode_syntax_action_tokens(
        &mut self,
        action: &SyntaxAction,
        out: &mut Vec<DcShorthand>,
    ) -> Result<()> {
        let method_id = self.get_or_insert_value_node(
            encode_string_value_tokens(&action.method),
            format!("{:?}", action.method),
        )?;
        out.push(sh(DC_BEGIN_ASSOCIATIVITY_GROUP));
        out.push(sh(DC_PATTERN_REPLACEMENT));
        out.push(sh(DC_ROUTINE_INVOCATION));
        out.push(DcShorthand::Long(method_id));
        for arg in &action.args {
            match arg {
                ActionArg::Variable(var) => {
                    let var_id = self.get_or_insert_value_node(
                        encode_string_value_tokens(var),
                        format!("${var}"),
                    )?;
                    out.push(sh(DC_REFERENCE_NAMED_OBJECT));
                    out.push(DcShorthand::Long(var_id));
                }
                ActionArg::Literal(lit) => {
                    let lit_id = self.get_or_insert_value_node(
                        encode_string_value_tokens(lit),
                        format!("{lit:?}"),
                    )?;
                    out.push(DcShorthand::Long(lit_id));
                }
            }
        }
        out.push(sh(DC_END_ASSOCIATIVITY_GROUP));
        Ok(())
    }

    fn encode_format_expr_tokens(
        &mut self,
        expr: &FormatExpr,
    ) -> Result<Vec<DcShorthand>> {
        let dc_tokens = encode_dc_stream(expr);
        let mut shorthands = Vec::with_capacity(dc_tokens.len());
        for tok in dc_tokens {
            match tok {
                DcToken::Dc(shorthand) => shorthands.push(shorthand),
                DcToken::NamedType(name) => {
                    let seed_key = SeedEntityKey::NamedConstruct(name.clone());
                    if let Some(&construct_id) =
                        self.seed_entity_ids.get(&seed_key)
                    {
                        shorthands.push(DcShorthand::Long(construct_id));
                    } else if let Ok(int_val) = name.parse::<u128>() {
                        let int_id = self.get_or_insert_value_node(
                            encode_integer_value_tokens(int_val),
                            int_val.to_string(),
                        )?;
                        shorthands.push(DcShorthand::Long(int_id));
                    } else {
                        let clean = name
                            .strip_prefix('"')
                            .and_then(|s| s.strip_suffix('"'))
                            .map_or_else(
                                || name.clone(),
                                |s| s.replace("\\\"", "\""),
                            );
                        let str_id = self.get_or_insert_value_node(
                            encode_string_value_tokens(&clean),
                            format!("{clean:?}"),
                        )?;
                        shorthands.push(DcShorthand::Long(str_id));
                    }
                }
            }
        }
        Ok(shorthands)
    }

    fn apply_desired_state(
        &mut self,
        mut desired: DesiredDatasetState,
    ) -> Result<()> {
        // 1. Ensure all seed classification entities exist and add their label/syntax properties.
        for seed_key in &desired.seed_entities {
            let entity_id = self.ensure_seed_entity(seed_key)?;
            let subj = DcShorthand::Long(entity_id);
            let label_prop = sh(seed_key.label_prop_dc());
            let label = seed_key.label();
            desired.single_properties.insert(
                (subj, label_prop),
                (
                    make_string_value_spec(label),
                    format!("{} = {label:?}", seed_key.comment()),
                ),
            );
            if let SeedEntityKey::Script(script_name) = seed_key {
                if let Some((parent_prefix, _)) = script_name.rsplit_once(':') {
                    if !parent_prefix.is_empty() {
                        let parent_key =
                            SeedEntityKey::Script(parent_prefix.to_string());
                        let parent_id = self.ensure_seed_entity(&parent_key)?;
                        desired.direct_statements.insert(
                            (
                                sh(DC_IS_SUBTYPE_OF),
                                subj,
                                ValueSpec::Direct(DcShorthand::Long(parent_id)),
                            ),
                            format!("{script_name} subtype of {parent_prefix}"),
                        );
                    }
                }
            }
        }

        // Add syntax rules for named syntax constructs (`README-named-types.csv`).
        for (name, (pattern, raw_str)) in &desired.named_construct_rules {
            let seed_key = SeedEntityKey::NamedConstruct(name.clone());
            let entity_id = self.ensure_seed_entity(&seed_key)?;
            let subj = DcShorthand::Long(entity_id);
            let rule = DcSyntaxRule {
                pattern: pattern.clone(),
                action: None,
                raw: raw_str.clone(),
            };
            desired.single_properties.insert(
                (subj, sh(DC_SYNTAX_RULE)),
                (
                    ValueSpec::PatternNode {
                        rule: Box::new(rule),
                        comment: format!("[{name}] {raw_str}"),
                    },
                    format!("Syntax rule ([{name}], {raw_str})"),
                ),
            );
        }

        // 2. Resolve all desired single-valued properties and diff against active single-valued properties.
        let mut resolved_single: BTreeMap<
            (DcShorthand, DcShorthand),
            (DcShorthand, String),
        > = BTreeMap::new();
        for ((subj, prop), (val_spec, comment)) in &desired.single_properties {
            let val_tok = self.resolve_value_spec(val_spec)?;
            resolved_single.insert((*subj, *prop), (val_tok, comment.clone()));
        }

        // Retract removed single-valued properties
        let existing_single_keys: Vec<(DcShorthand, DcShorthand)> =
            self.active_single_props.keys().copied().collect();
        for key in existing_single_keys {
            if !resolved_single.contains_key(&key) {
                if let Some((has_a_id, has_val_id, _)) =
                    self.active_single_props.remove(&key)
                {
                    self.retract_node(has_val_id)?;
                    self.retract_node(has_a_id)?;
                }
            }
        }

        // Insert or update (`591` `Supersedes`) single-valued properties
        for ((subj, prop), (val_tok, _comment)) in resolved_single {
            if let Some(&(has_a_id, old_has_val_id, old_val_tok)) =
                self.active_single_props.get(&(subj, prop))
            {
                if old_val_tok != val_tok {
                    let new_has_val_id = self.append_statement_node(
                        sh(DC_HAS_VALUE),
                        DcShorthand::Long(has_a_id),
                        val_tok,
                    )?;
                    self.append_statement_node(
                        sh(DC_SUPERSEDES),
                        DcShorthand::Long(new_has_val_id),
                        DcShorthand::Long(old_has_val_id),
                    )?;
                    self.superseded_count =
                        self.superseded_count.saturating_add(1);
                    self.active_single_props.insert(
                        (subj, prop),
                        (has_a_id, new_has_val_id, val_tok),
                    );
                }
            } else {
                let has_a_id =
                    self.append_statement_node(sh(DC_HAS_A), subj, prop)?;
                let has_val_id = self.append_statement_node(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(has_a_id),
                    val_tok,
                )?;
                self.active_single_props
                    .insert((subj, prop), (has_a_id, has_val_id, val_tok));
            }
        }

        // 3. Resolve and diff multi-valued properties.
        let mut resolved_multi: BTreeMap<
            (DcShorthand, DcShorthand, DcShorthand),
            (Option<String>, String),
        > = BTreeMap::new();
        for ((subj, prop, val_spec), comment) in &desired.multi_properties {
            let val_tok = self.resolve_value_spec(val_spec)?;
            let raw_source = match val_spec {
                ValueSpec::PatternNode { rule, .. } => {
                    Some(rule.raw.clone())
                }
                _ => None,
            };
            resolved_multi.insert(
                (*subj, *prop, val_tok),
                (raw_source, comment.clone()),
            );
        }

        let mut removed_by_subj_prop: BTreeMap<
            (DcShorthand, DcShorthand),
            Vec<(DcShorthand, u128, u128)>,
        > = BTreeMap::new();
        let existing_multi_keys: Vec<(DcShorthand, DcShorthand, DcShorthand)> =
            self.active_multi_props.keys().copied().collect();
        for key in existing_multi_keys {
            if !resolved_multi.contains_key(&key) {
                if let Some((has_a_id, has_val_id)) =
                    self.active_multi_props.remove(&key)
                {
                    removed_by_subj_prop
                        .entry((key.0, key.1))
                        .or_default()
                        .push((key.2, has_a_id, has_val_id));
                }
            }
        }

        let mut added_by_subj_prop: BTreeMap<
            (DcShorthand, DcShorthand),
            Vec<(DcShorthand, Option<String>)>,
        > = BTreeMap::new();
        for ((subj, prop, val_tok), (raw_source, _comment)) in resolved_multi {
            if !self.active_multi_props.contains_key(&(subj, prop, val_tok)) {
                added_by_subj_prop
                    .entry((subj, prop))
                    .or_default()
                    .push((val_tok, raw_source));
            }
        }

        let mut all_multi_subj_props: BTreeSet<(DcShorthand, DcShorthand)> =
            removed_by_subj_prop.keys().copied().collect();
        all_multi_subj_props.extend(added_by_subj_prop.keys().copied());

        for (subj, prop) in all_multi_subj_props {
            let mut removed = removed_by_subj_prop
                .remove(&(subj, prop))
                .unwrap_or_default();
            // Sort by `has_a_id` so superseding preserves original statement order
            removed.sort_by_key(|&(_, has_a_id, _)| has_a_id);
            let mut added = added_by_subj_prop
                .remove(&(subj, prop))
                .unwrap_or_default();

            let mut superseded_pairs: Vec<((u128, u128), DcShorthand)> =
                Vec::new();

            // First pass: pair removed string value nodes whose text matches a new pattern's raw source
            let mut rem_idx = 0usize;
            while rem_idx < removed.len() {
                let Some(&(old_val_tok, has_a_id, old_has_val_id)) =
                    removed.get(rem_idx)
                else {
                    break;
                };
                let matched_add_idx = if let DcShorthand::Long(old_node_id) =
                    old_val_tok
                {
                    if let Some(old_str) =
                        self.string_by_node_id.get(&old_node_id)
                    {
                        added.iter().position(|(_, raw_opt)| {
                            raw_opt.as_deref() == Some(old_str.as_str())
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(add_idx) = matched_add_idx {
                    let (new_val_tok, _) = added.remove(add_idx);
                    removed.remove(rem_idx);
                    superseded_pairs
                        .push(((has_a_id, old_has_val_id), new_val_tok));
                } else {
                    rem_idx = rem_idx.saturating_add(1);
                }
            }

            // Second pass: pair any remaining removed and added values on the same `(subj, prop)`
            while !removed.is_empty() && !added.is_empty() {
                let (_, has_a_id, old_has_val_id) = removed.remove(0);
                let (new_val_tok, _) = added.remove(0);
                superseded_pairs
                    .push(((has_a_id, old_has_val_id), new_val_tok));
            }

            superseded_pairs.sort_by_key(|&((has_a_id, _), _)| has_a_id);
            for ((has_a_id, old_has_val_id), new_val_tok) in superseded_pairs {
                let new_has_val_id = self.append_statement_node(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(has_a_id),
                    new_val_tok,
                )?;
                self.append_statement_node(
                    sh(DC_SUPERSEDES),
                    DcShorthand::Long(new_has_val_id),
                    DcShorthand::Long(old_has_val_id),
                )?;
                self.superseded_count =
                    self.superseded_count.saturating_add(1);
                self.active_multi_props.insert(
                    (subj, prop, new_val_tok),
                    (has_a_id, new_has_val_id),
                );
            }

            // Retract any leftover unpaired removed values
            for (_, has_a_id, old_has_val_id) in removed {
                self.retract_node(old_has_val_id)?;
                self.retract_node(has_a_id)?;
            }

            // Append any leftover unpaired added values
            for (new_val_tok, _) in added {
                let has_a_id =
                    self.append_statement_node(sh(DC_HAS_A), subj, prop)?;
                let has_val_id = self.append_statement_node(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(has_a_id),
                    new_val_tok,
                )?;
                self.active_multi_props
                    .insert((subj, prop, new_val_tok), (has_a_id, has_val_id));
            }
        }

        // 4. Resolve and diff `<prefer>` replacements (`569` + `570`).
        let mut resolved_preferred: BTreeMap<
            (DcShorthand, DcShorthand),
            String,
        > = BTreeMap::new();
        for ((subj, val_spec), comment) in &desired.preferred_replacements {
            let val_tok = self.resolve_value_spec(val_spec)?;
            resolved_preferred.insert((*subj, val_tok), comment.clone());
        }

        let existing_pref_keys: Vec<(DcShorthand, DcShorthand)> =
            self.active_preferred.keys().copied().collect();
        for key in existing_pref_keys {
            if !resolved_preferred.contains_key(&key) {
                if let Some((has_a_id, has_val_id, pref_id)) =
                    self.active_preferred.remove(&key)
                {
                    self.retract_node(pref_id)?;
                    self.retract_node(has_val_id)?;
                    self.retract_node(has_a_id)?;
                }
            }
        }

        for ((subj, val_tok), _comment) in resolved_preferred {
            if !self.active_preferred.contains_key(&(subj, val_tok)) {
                let has_a_id = self.append_statement_node(
                    sh(DC_HAS_A),
                    subj,
                    sh(DC_REPLACEMENT),
                )?;
                let has_val_id = self.append_statement_node(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(has_a_id),
                    val_tok,
                )?;
                let pref_id = self.append_statement_node(
                    sh(DC_HAS_A),
                    DcShorthand::Long(has_a_id),
                    sh(DC_PREFERRED),
                )?;
                self.active_preferred
                    .insert((subj, val_tok), (has_a_id, has_val_id, pref_id));
            }
        }

        // 5. Resolve and diff direct 3-token statements (`586`, `587`, `588`, `589`, `590`, `598`).
        let mut resolved_direct: BTreeMap<
            (DcShorthand, DcShorthand, DcShorthand),
            String,
        > = BTreeMap::new();
        for ((verb, subj, obj_spec), comment) in &desired.direct_statements {
            let obj_tok = self.resolve_value_spec(obj_spec)?;
            resolved_direct.insert((*verb, *subj, obj_tok), comment.clone());
        }

        let existing_direct_keys: Vec<(DcShorthand, DcShorthand, DcShorthand)> =
            self.active_direct_stmts.keys().copied().collect();
        for key in existing_direct_keys {
            if !resolved_direct.contains_key(&key) {
                if let Some(stmt_id) = self.active_direct_stmts.remove(&key) {
                    self.retract_node(stmt_id)?;
                }
            }
        }

        for ((verb, subj, obj_tok), _comment) in resolved_direct {
            if !self
                .active_direct_stmts
                .contains_key(&(verb, subj, obj_tok))
            {
                let stmt_id =
                    self.append_statement_node(verb, subj, obj_tok)?;
                self.active_direct_stmts
                    .insert((verb, subj, obj_tok), stmt_id);
            }
        }

        Ok(())
    }

    fn retract_node(&mut self, target_id: u128) -> Result<u128> {
        let id = self.append_statement_node(
            sh(DC_RETRACTS),
            sh(DC_ENCLOSING_DOCUMENT),
            DcShorthand::Long(target_id),
        )?;
        self.retracted_count = self.retracted_count.saturating_add(1);
        Ok(id)
    }
}

fn single_valued_property_set() -> HashSet<DcShorthand> {
    HashSet::from([
        sh(DC_CANONICAL_NAME),
        sh(DC_DESCRIPTION),
        sh(DC_PROGRAMMATIC_IDENTIFIER),
        sh(DC_CANONICAL_COMBINING_CLASS),
        sh(DC_BIDIRECTIONAL_CLASS),
        sh(DC_CASE_MAPPING_PARTNER),
        sh(DC_SYNTAX_RULE),
        sh(DC_DATA_FORMAT_DESCRIPTION),
    ])
}

fn is_relationship_verb(tok: DcShorthand) -> bool {
    tok == sh(DC_HAS_A)
        || tok == sh(DC_IS_INSTANCE_OF)
        || tok == sh(DC_IS_SUBTYPE_OF)
        || tok == sh(DC_IS_BASED_ON)
        || tok == sh(DC_IS_DEPRECATED)
        || tok == sh(DC_IS_CLOSING_DELIMITER_FOR)
        || tok == sh(DC_SUPERSEDES)
        || tok == sh(DC_RETRACTS)
        || tok == sh(DC_IS_ASSOCIATED_WITH)
        || tok == sh(DC_HAS_VALUE)
}

fn push_atom_dc(out: &mut Vec<DcShorthand>, tok: DcShorthand) {
    if tok == sh(DC_ESCAPE) || tok == sh(DC_EVALUATE_VALUE_AS_ATOM) {
        out.push(sh(DC_ESCAPE));
    }
    out.push(tok);
}

/// Encodes a relationship statement `<verb> [atom_dc] [atom_dc]`, escaping
/// `Dc 255` (`Ignore following Dc`) and `Dc 616` (`Evaluate value as atom`) with `Dc 255`.
#[must_use]
pub fn encode_statement_tokens(
    verb: DcShorthand,
    subj: DcShorthand,
    obj: DcShorthand,
) -> Vec<DcShorthand> {
    let mut out = Vec::with_capacity(5);
    out.push(verb);
    push_atom_dc(&mut out, subj);
    push_atom_dc(&mut out, obj);
    out
}

fn consume_atom_dc(
    tokens: &[DcShorthand],
    idx: &mut usize,
) -> Option<DcShorthand> {
    let tok = *tokens.get(*idx)?;
    *idx = idx.saturating_add(1);
    if tok == sh(DC_ESCAPE) {
        let escaped = *tokens.get(*idx)?;
        if escaped == sh(DC_ESCAPE)
            || escaped == sh(DC_EVALUATE_VALUE_AS_ATOM)
        {
            *idx = idx.saturating_add(1);
            Some(escaped)
        } else {
            None
        }
    } else if tok == sh(DC_EVALUATE_VALUE_AS_ATOM) {
        None
    } else {
        Some(tok)
    }
}

/// Parses a relationship statement `<verb> [atom_dc] [atom_dc]` back into `(verb, subj, obj)`.
#[must_use]
pub fn parse_statement_tokens(
    tokens: &[DcShorthand],
) -> Option<(DcShorthand, DcShorthand, DcShorthand)> {
    let verb = *tokens.first()?;
    if !is_relationship_verb(verb) {
        return None;
    }
    let mut idx = 1usize;
    let subj = consume_atom_dc(tokens, &mut idx)?;
    let obj = consume_atom_dc(tokens, &mut idx)?;
    if idx == tokens.len() {
        Some((verb, subj, obj))
    } else {
        None
    }
}

/// Compiles `dc_defns`, `format_defns`, `scripts`, and `named_types_bytes` into a
/// deterministic `DesiredDatasetState`.
fn compile_desired_dataset_state(
    dc_defns: &[DcDefn],
    format_defns: &[DcDefn],
    known_scripts: &BTreeSet<String>,
    named_types_bytes: Option<&[u8]>,
) -> Result<DesiredDatasetState> {
    let mut state = DesiredDatasetState::default();

    // 1. Seed scripts from `README.scripts.csv` (and parent scripts).
    for script in known_scripts {
        insert_script_and_parents(&mut state.seed_entities, script);
    }

    // 2. Seed named syntax constructs from `README-named-types.csv`.
    if let Some(bytes) = named_types_bytes {
        let vec_bytes = bytes.to_vec();
        let table = csv_tools::parse_csv_reader(
            &vec_bytes,
            csv_tools::CsvParseOptions {
                has_header: true,
                flexible: true,
                ..Default::default()
            },
        )
        .context("Failed to parse README-named-types.csv")?;
        for row_idx in 0..table.row_count() {
            let line_no = row_idx.saturating_add(2);
            let Some(row) = table.row(row_idx) else {
                continue;
            };
            if row.iter().all(|c| c.trim().is_empty()) {
                continue;
            }
            let name = row.first().map_or("", |s| s.trim());
            let raw_syntax = row.get(1).map_or("", |s| s.trim());
            ensure!(
                !name.is_empty(),
                "README-named-types.csv row {line_no} is missing named type"
            );
            ensure!(
                !raw_syntax.is_empty(),
                "README-named-types.csv row {line_no} ('{name}') has no syntax"
            );
            let parsed = parse_dc_syntax(raw_syntax).with_context(|| {
                format!(
                    "README-named-types.csv row {line_no} ('{name}') has invalid syntax '{raw_syntax}'"
                )
            })?;
            state
                .seed_entities
                .insert(SeedEntityKey::NamedConstruct(name.to_string()));
            state.named_construct_rules.insert(
                name.to_string(),
                (parsed.pattern, raw_syntax.to_string()),
            );
        }
    }

    // Also register `.EL Types` built-in type names as named syntax constructs if not already present.
    for defn in dc_defns {
        if defn.script == ".EL Types" {
            if let Some(short_id) = defn.short_id {
                if let Ok(short_u32) = u32::try_from(short_id) {
                    if let Some(type_name) =
                        crate::validation::extract_el_type_name(&defn.name)
                    {
                        state.seed_entities.insert(
                            SeedEntityKey::NamedConstruct(type_name.clone()),
                        );
                        state
                            .named_construct_rules
                            .entry(type_name)
                            .or_insert_with(|| {
                                (
                                    SyntaxPattern::Sequence(vec![
                                        SyntaxElement::exact(SyntaxTerm::CharRef(
                                            CharTarget::Dc(short_u32),
                                        )),
                                    ]),
                                    format!(":{short_u32}"),
                                )
                            });
                    }
                }
            }
        }
    }

    // 3. Compile all Dc and Format definitions in deterministic `dc_id` order.
    let mut all_defns: Vec<&DcDefn> =
        dc_defns.iter().chain(format_defns.iter()).collect();
    all_defns.sort_by_key(|d| d.dc_id);

    for defn in all_defns {
        compile_defn_statements(&mut state, defn)?;
    }

    Ok(state)
}

fn insert_script_and_parents(
    seed_entities: &mut BTreeSet<SeedEntityKey>,
    script: &str,
) {
    if script.is_empty() {
        return;
    }
    seed_entities.insert(SeedEntityKey::Script(script.to_string()));
    if let Some((parent, _)) = script.rsplit_once(':') {
        if !parent.is_empty() {
            insert_script_and_parents(seed_entities, parent);
        }
    }
}

fn compile_defn_statements(
    state: &mut DesiredDatasetState,
    defn: &DcDefn,
) -> Result<()> {
    let subj = DcShorthand::from_global_id(defn.dc_id);

    // Canonical name (`572`)
    if !defn.name.is_empty() {
        state.single_properties.insert(
            (subj, sh(DC_CANONICAL_NAME)),
            (
                make_string_value_spec(&defn.name),
                format!("{subj} name {:?}", defn.name),
            ),
        );
    }

    // Description (`573`)
    if !defn.description.is_empty() {
        state.single_properties.insert(
            (subj, sh(DC_DESCRIPTION)),
            (
                make_string_value_spec(&defn.description),
                format!("{subj} description"),
            ),
        );
    }

    // Programmatic identifier (`574`)
    if let Some(ident) = &defn.ident {
        if !ident.is_empty() {
            state.single_properties.insert(
                (subj, sh(DC_PROGRAMMATIC_IDENTIFIER)),
                (
                    make_string_value_spec(ident),
                    format!("{subj} ident {ident:?}"),
                ),
            );
        }
    }

    // Character-specific properties (combining class `580`, bidi class `581`, general category `583`/`608`)
    if defn.format.is_none() {
        if defn.combining_class != 0 {
            let cc_u128 = u128::from(defn.combining_class);
            state.single_properties.insert(
                (subj, sh(DC_CANONICAL_COMBINING_CLASS)),
                (
                    make_integer_value_spec(cc_u128),
                    format!("{subj} combining class {cc_u128}"),
                ),
            );
        }

        let bidi_code = defn.bidi_class.as_str().to_string();
        let bidi_key = SeedEntityKey::BidiClass(bidi_code.clone());
        state.seed_entities.insert(bidi_key.clone());
        state.single_properties.insert(
            (subj, sh(DC_BIDIRECTIONAL_CLASS)),
            (
                ValueSpec::SeedEntity(bidi_key),
                format!("{subj} bidi {bidi_code}"),
            ),
        );

        let gc_code = defn.general_category.as_str().to_string();
        let gc_key = SeedEntityKey::GeneralCategory(gc_code.clone());
        state.seed_entities.insert(gc_key.clone());
        state.direct_statements.insert(
            (sh(DC_IS_INSTANCE_OF), subj, ValueSpec::SeedEntity(gc_key)),
            format!("Is an instance of ({subj}, {gc_code})"),
        );
    }

    // Script (`596` / `609`)
    if !defn.script.is_empty() {
        insert_script_and_parents(&mut state.seed_entities, &defn.script);
        let script_key = SeedEntityKey::Script(defn.script.clone());
        state.direct_statements.insert(
            (
                sh(DC_IS_INSTANCE_OF),
                subj,
                ValueSpec::SeedEntity(script_key),
            ),
            format!("Is an instance of ({subj}, {})", defn.script),
        );
    }

    // Case mapping partner (`582`)
    if let Some(partner_short) = defn.casing_partner {
        let partner_sh = DcShorthand::Short(partner_short);
        state.single_properties.insert(
            (subj, sh(DC_CASE_MAPPING_PARTNER)),
            (
                ValueSpec::Direct(partner_sh),
                format!("{subj} case partner {partner_sh}"),
            ),
        );
    }

    // Deprecation (`589 <subj> 517`)
    if defn.is_deprecated {
        state.direct_statements.insert(
            (
                sh(DC_IS_DEPRECATED),
                subj,
                ValueSpec::Direct(sh(DC_BOOLEAN_TRUE)),
            ),
            format!("Is deprecated ({subj})"),
        );
    }

    // Closing delimiter (`590 <subj> <opener>`)
    if let Some(opener) = defn.closes {
        state.direct_statements.insert(
            (
                sh(DC_IS_CLOSING_DELIMITER_FOR),
                subj,
                ValueSpec::Direct(opener),
            ),
            format!("Closes delimiter ({subj}, {opener})"),
        );
    }

    // `@implies(...)` -> `587 <subj> <target>`
    for imp in &defn.implies {
        let trimmed = imp.trim();
        if trimmed.is_empty() {
            continue;
        }
        let obj_spec = parse_relation_target_spec(trimmed)?;
        state.direct_statements.insert(
            (sh(DC_IS_SUBTYPE_OF), subj, obj_spec),
            format!("Is a subtype of ({subj}, {trimmed})"),
        );
    }

    // `@based_on(...)` -> `588 <subj> <target>`
    for base in &defn.based_on {
        let trimmed = base.trim();
        if trimmed.is_empty() {
            continue;
        }
        let obj_spec = parse_relation_target_spec(trimmed)?;
        state.direct_statements.insert(
            (sh(DC_IS_BASED_ON), subj, obj_spec),
            format!("Is based on ({subj}, {trimmed})"),
        );
    }

    // `@os(...)` -> `598 <subj> <os_format>`
    for os_raw in &defn.os_associations {
        let trimmed = os_raw.trim();
        if let Ok(os_sh) = DcShorthand::parse(trimmed) {
            state.direct_statements.insert(
                (
                    sh(DC_IS_ASSOCIATED_WITH),
                    subj,
                    ValueSpec::Direct(os_sh),
                ),
                format!("Is associated with OS ({subj}, {os_sh})"),
            );
        }
    }

    // Informal aliases (`576`)
    for alias in &defn.aliases {
        if !alias.is_empty() {
            state.multi_properties.insert(
                (subj, sh(DC_INFORMAL_ALIAS), make_string_value_spec(alias)),
                format!("{subj} alias {alias:?}"),
            );
        }
    }

    // Formal aliases (`577`) + `@title(...)`
    if let Some(title) = &defn.title {
        if !title.is_empty() {
            state.multi_properties.insert(
                (subj, sh(DC_FORMAL_ALIAS), make_string_value_spec(title)),
                format!("{subj} title {title:?}"),
            );
        }
    }
    for (_kind, alias) in &defn.formal_aliases {
        if !alias.is_empty() {
            state.multi_properties.insert(
                (subj, sh(DC_FORMAL_ALIAS), make_string_value_spec(alias)),
                format!("{subj} formal alias {alias:?}"),
            );
        }
    }

    // Annotations (`578`)
    for ann in &defn.annotations {
        if !ann.is_empty() {
            state.multi_properties.insert(
                (subj, sh(DC_ANNOTATION), make_string_value_spec(ann)),
                format!("{subj} annotation {ann:?}"),
            );
        }
    }

    // Magic signatures (`579`)
    for magic in &defn.magic_names {
        if !magic.is_empty() {
            state.multi_properties.insert(
                (subj, sh(DC_MAGIC_SIGNATURE), make_string_value_spec(magic)),
                format!("{subj} magic {magic:?}"),
            );
        }
    }

    // Cross-references (`593`)
    for xref in &defn.cross_references {
        let clean = xref.strip_prefix('>').unwrap_or(xref).trim();
        if clean.is_empty() {
            continue;
        }
        let val_spec = if let Ok(target_sh) = DcShorthand::parse(clean) {
            ValueSpec::Direct(target_sh)
        } else {
            make_string_value_spec(clean)
        };
        state.multi_properties.insert(
            (subj, sh(DC_CROSS_REFERENCE), val_spec),
            format!("{subj} xref {clean}"),
        );
    }

    // Decompositions (`611`..=`614` and `<prefer>` `569`+`570`)
    for decomp in &defn.decompositions {
        let Some((tag_raw, payload_raw)) = decomp.split_once('>') else {
            continue;
        };
        let tag = tag_raw.trim_start_matches('<').trim();
        let payload = payload_raw.trim();
        if payload.is_empty() {
            continue;
        }
        let mut payload_tokens = Vec::new();
        for tok_str in payload.split_whitespace() {
            if let Ok(shorthand) = DcShorthand::parse(tok_str) {
                payload_tokens.push(shorthand);
            }
        }
        let val_spec = if payload_tokens.len() == 1 {
            #[expect(
                clippy::expect_used,
                reason = "payload_tokens.len() == 1 checked immediately above"
            )]
            ValueSpec::Direct(*payload_tokens.first().expect("single token"))
        } else if !payload_tokens.is_empty() {
            ValueSpec::ValueNode {
                tokens: payload_tokens,
                comment: format!("<{tag}> {payload}"),
            }
        } else {
            make_string_value_spec(payload)
        };

        if tag == "prefer" {
            state.preferred_replacements.insert(
                (subj, val_spec),
                format!("{subj} preferred replacement {payload}"),
            );
        } else {
            let decomp_dc = match tag {
                "equiv" => DC_CANONICAL_DECOMPOSITION,
                "approx" => DC_APPROXIMATE_DECOMPOSITION,
                "semantic" => DC_SEMANTIC_DECOMPOSITION,
                _ => DC_COMPATIBILITY_DECOMPOSITION,
            };
            state.multi_properties.insert(
                (subj, sh(decomp_dc), val_spec),
                format!("{subj} <{tag}> {payload}"),
            );
        }
    }

    // Syntax rule (`594`)
    if let Some(rule) = &defn.syntax {
        collect_named_constructs_from_pattern(
            &rule.pattern,
            &mut state.seed_entities,
        );
        state.single_properties.insert(
            (subj, sh(DC_SYNTAX_RULE)),
            (
                ValueSpec::PatternNode {
                    rule: Box::new(rule.clone()),
                    comment: format!("{subj} {}", rule.raw),
                },
                format!("Syntax rule ({subj}, {})", rule.raw),
            ),
        );
    }

    // Format-specific metadata (`595`, `f264`, `f636`, `f637`, `f638`)
    if let Some(fmt) = &defn.format {
        if let Some(expr) = &fmt.format_spec {
            state.single_properties.insert(
                (subj, sh(DC_DATA_FORMAT_DESCRIPTION)),
                (
                    ValueSpec::FormatExprNode {
                        expr: Box::new(expr.clone()),
                        raw_comment: format!("{subj} format spec"),
                    },
                    format!("Format spec ({subj})"),
                ),
            );
        }
        if let Some(exts) = &fmt.extensions {
            for ext in exts.split(',').map(str::trim).filter(|s| !s.is_empty())
            {
                let ext_spec = if crate::syntax::is_extension_regex(ext) {
                    let rule = crate::syntax::parse_extension_regex(ext)?;
                    ValueSpec::PatternNode {
                        rule: Box::new(rule),
                        comment: ext.to_string(),
                    }
                } else {
                    make_string_value_spec(ext)
                };
                state.multi_properties.insert(
                    (subj, sh(DC_FILE_EXTENSION), ext_spec),
                    format!("{subj} ext {ext:?}"),
                );
            }
        }
        if let Some(mimes) = &fmt.mime {
            for mime in
                mimes.split(',').map(str::trim).filter(|s| !s.is_empty())
            {
                state.multi_properties.insert(
                    (subj, sh(DC_MEDIA_TYPE), make_string_value_spec(mime)),
                    format!("{subj} mime {mime:?}"),
                );
            }
        }
        if let Some(utis) = &fmt.uti {
            for uti in utis.split(',').map(str::trim).filter(|s| !s.is_empty())
            {
                state.multi_properties.insert(
                    (
                        subj,
                        sh(DC_UNIFORM_TYPE_IDENTIFIER),
                        make_string_value_spec(uti),
                    ),
                    format!("{subj} uti {uti:?}"),
                );
            }
        }
        if let Some(apple) = &fmt.apple_type {
            let trimmed = apple.trim();
            if !trimmed.is_empty() {
                state.multi_properties.insert(
                    (
                        subj,
                        sh(DC_APPLE_OS_TYPE),
                        make_string_value_spec(trimmed),
                    ),
                    format!("{subj} ostype {trimmed:?}"),
                );
            }
        }
    }

    Ok(())
}

fn collect_named_constructs_from_pattern(
    pattern: &SyntaxPattern,
    seed_entities: &mut BTreeSet<SeedEntityKey>,
) {
    match pattern {
        SyntaxPattern::Sequence(elements) => {
            for elem in elements {
                match &elem.term {
                    SyntaxTerm::NamedConstruct { name, .. } => {
                        seed_entities
                            .insert(SeedEntityKey::NamedConstruct(name.clone()));
                    }
                    SyntaxTerm::Group(inner) => {
                        collect_named_constructs_from_pattern(
                            inner,
                            seed_entities,
                        );
                    }
                    _ => {}
                }
            }
        }
        SyntaxPattern::Alternation(branches) => {
            for branch in branches {
                collect_named_constructs_from_pattern(branch, seed_entities);
            }
        }
    }
}

fn parse_relation_target_spec(raw: &str) -> Result<ValueSpec> {
    if let Ok(shorthand) = DcShorthand::parse(raw) {
        return Ok(ValueSpec::Direct(shorthand));
    }
    if raw.starts_with('[') && raw.ends_with(']') {
        let rule = parse_dc_syntax(&format!(":{raw}"))?;
        return Ok(ValueSpec::PatternNode {
            rule: Box::new(rule),
            comment: raw.to_string(),
        });
    }
    if let Ok(expr) = parse_format_expr(&format!("@({raw})")) {
        return Ok(ValueSpec::FormatExprNode {
            expr: Box::new(expr),
            raw_comment: raw.to_string(),
        });
    }
    Ok(make_string_value_spec(raw))
}

/// Updates an in-memory list of `LedgerNode`s against `dc_defns`, `format_defns`,
/// `known_scripts`, and `named_types_bytes`.
pub fn update_ledger_in_memory(
    existing_nodes: Vec<LedgerNode>,
    dc_defns: &[DcDefn],
    format_defns: &[DcDefn],
    known_scripts: &BTreeSet<String>,
    named_types_bytes: Option<&[u8]>,
) -> Result<(Vec<LedgerNode>, LedgerUpdateStats)> {
    let single_props = single_valued_property_set();
    let initial_labels = build_initial_token_labels(dc_defns, format_defns);
    let mut builder = LedgerBuilder::from_existing(
        existing_nodes,
        &single_props,
        initial_labels,
    );
    let desired = compile_desired_dataset_state(
        dc_defns,
        format_defns,
        known_scripts,
        named_types_bytes,
    )?;
    builder.apply_desired_state(desired)?;
    let stats = LedgerUpdateStats {
        total_nodes: builder.nodes.len(),
        nodes_appended: builder.appended_count,
        nodes_superseded: builder.superseded_count,
        nodes_retracted: builder.retracted_count,
        files_written: 0,
    };
    Ok((builder.nodes, stats))
}

/// Loads the CSV dataset from `repo_root` and updates `docs/*.ctbnodes`
/// using the append-only revision ledger algorithm.
pub fn update_system_graph_ledger(repo_root: &Path) -> Result<LedgerUpdateStats> {
    let data_dir = repo_root.join("src/formats/dcdata/data");
    let categories_dir = data_dir.join("categories");
    let formats_dir = categories_dir.join("formats");
    let scripts_path = data_dir.join("README.scripts.csv");
    let named_types_path = data_dir.join("README-named-types.csv");
    let graph_dir = repo_root.join("docs");

    let mut report = ValidationReport::new();
    let scripts_bytes = std::fs::read(&scripts_path).with_context(|| {
        format!("Failed to read {}", scripts_path.display())
    })?;
    let known_scripts_set = crate::validation::validate_scripts_table(
        &scripts_bytes,
        "src/formats/dcdata/data/README.scripts.csv",
        &mut report,
    );
    let known_scripts: BTreeSet<String> =
        known_scripts_set.iter().cloned().collect();

    let named_types_bytes =
        std::fs::read(&named_types_path).with_context(|| {
            format!("Failed to read {}", named_types_path.display())
        })?;
    let known_named_types =
        crate::validation::extract_named_type_names(&named_types_bytes);

    let format_files_data =
        crate::updater::load_format_category_csvs_with_assigned_ids(
            &formats_dir,
        )?;
    let format_files_iter: Vec<(&str, &[u8])> = format_files_data
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();
    let format_defns = crate::format::validate_format_files_data(
        format_files_iter,
        &formats_dir.display().to_string(),
        Some(&known_scripts_set),
        &mut report,
    );
    let format_ids: HashSet<usize> =
        format_defns.iter().filter_map(|d| d.short_id).collect();
    let format_syntax_ids: HashSet<usize> = format_defns
        .iter()
        .filter(|d| d.syntax.is_some())
        .filter_map(|d| d.short_id)
        .collect();
    let format_registry =
        crate::syntax::build_format_validation_registry(&format_defns);

    let dc_files_data =
        crate::updater::load_dc_category_csvs_with_assigned_ids(
            &categories_dir,
        )?;
    let dc_files_iter: Vec<(&str, &[u8])> = dc_files_data
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();
    let dc_defns = crate::validation::validate_dc_files_data(
        dc_files_iter,
        &categories_dir.display().to_string(),
        &format_ids,
        None,
        Some(&known_named_types),
        Some(&known_scripts_set),
        Some(&format_syntax_ids),
        Some(&format_registry),
        &mut report,
    );
    report.ensure_valid().with_context(|| {
        format!("Invalid dataset under {}", data_dir.display())
    })?;

    let existing_nodes = load_ledger_nodes_from_dir(&graph_dir)?;
    let (updated_nodes, mut stats) = update_ledger_in_memory(
        existing_nodes,
        &dc_defns,
        &format_defns,
        &known_scripts,
        Some(&named_types_bytes),
    )?;
    stats.files_written = write_ledger_nodes_to_dir(&graph_dir, &updated_nodes)?;
    Ok(stats)
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

    fn sample_dc_defn(short_id: usize, name: &str, desc: &str) -> DcDefn {
        DcDefn {
            dc_id: 1_114_112 + u128::try_from(short_id).unwrap(),
            short_id: Some(short_id),
            ident: Some(format!("DC_TEST_{short_id}")),
            role_def: false,
            title: None,
            name: name.to_string(),
            category: "semantic".to_string(),
            combining_class: 0,
            bidi_class: BidiClass::BN,
            casing_partner: None,
            general_category: GeneralCategory::NonUnicodeControl,
            script: ".Semantic".to_string(),
            is_deprecated: false,
            decompositions: Vec::new(),
            aliases: vec!["test-alias".to_string()],
            formal_aliases: Vec::new(),
            cross_references: Vec::new(),
            annotations: Vec::new(),
            implies: vec!["571".to_string()],
            based_on: Vec::new(),
            os_associations: Vec::new(),
            magic_names: Vec::new(),
            syntax: Some(parse_dc_syntax(":~").unwrap()),
            closes: None,
            description: desc.to_string(),
            format: None,
            source_file: "test.csv".to_string(),
            line_number: 2,
        }
    }

    #[crate::ctb_test]
    fn test_ledger_initial_generation_and_idempotence() -> Result<()> {
        let defn = sample_dc_defn(900, "Test character", "Initial description");
        let scripts = BTreeSet::from([".Semantic".to_string()]);

        let (nodes1, stats1) =
            update_ledger_in_memory(Vec::new(), &[defn.clone()], &[], &scripts, None)?;
        assert!(stats1.total_nodes > 0);
        assert_eq!(stats1.nodes_appended, stats1.total_nodes);
        assert_eq!(stats1.nodes_superseded, 0);
        assert_eq!(stats1.nodes_retracted, 0);

        // Second run with identical inputs must be a no-op
        let (nodes2, stats2) =
            update_ledger_in_memory(nodes1.clone(), &[defn], &[], &scripts, None)?;
        assert_eq!(stats2.nodes_appended, 0);
        assert_eq!(stats2.nodes_superseded, 0);
        assert_eq!(stats2.nodes_retracted, 0);
        assert_eq!(nodes1, nodes2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_ledger_supersedes_on_single_value_property_change() -> Result<()> {
        let mut defn =
            sample_dc_defn(900, "Original name", "Original description");
        let scripts = BTreeSet::from([".Semantic".to_string()]);

        let (nodes1, _) =
            update_ledger_in_memory(Vec::new(), &[defn.clone()], &[], &scripts, None)?;
        let initial_len = nodes1.len();

        // Modify the single-valued Canonical name (`572`)
        defn.name = "Updated name".to_string();
        let (nodes2, stats2) =
            update_ledger_in_memory(nodes1.clone(), &[defn], &[], &scripts, None)?;

        // Existing prefix of nodes must be byte-for-byte identical
        assert_eq!(&nodes2[..initial_len], &nodes1[..]);
        assert_eq!(stats2.nodes_superseded, 1);
        assert_eq!(stats2.nodes_retracted, 0);
        // Appended: 1 new string value node + 1 new `610` (`Has value`) node + 1 `591` (`Supersedes`) node = 3
        assert_eq!(stats2.nodes_appended, 3);
        let last = nodes2.last().unwrap();
        assert_eq!(last.tokens.first(), Some(&sh(DC_SUPERSEDES)));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_ledger_retracts_on_statement_removal() -> Result<()> {
        let mut defn =
            sample_dc_defn(900, "Test character", "Description text");
        let scripts = BTreeSet::from([".Semantic".to_string()]);

        let (nodes1, _) =
            update_ledger_in_memory(Vec::new(), &[defn.clone()], &[], &scripts, None)?;
        let initial_len = nodes1.len();

        // Remove `@implies(571)` (`587`) and informal alias (`576`)
        defn.implies.clear();
        defn.aliases.clear();
        let (nodes2, stats2) =
            update_ledger_in_memory(nodes1.clone(), &[defn], &[], &scripts, None)?;

        assert_eq!(&nodes2[..initial_len], &nodes1[..]);
        // Retracted: 1 direct `587` statement + 2 nodes (`584` and `610`) for the multi-valued alias = 3
        assert_eq!(stats2.nodes_retracted, 3);
        assert_eq!(stats2.nodes_appended, 3);
        for node in &nodes2[initial_len..] {
            assert_eq!(node.tokens.first(), Some(&sh(DC_RETRACTS)));
            assert_eq!(node.tokens.get(1), Some(&sh(DC_ENCLOSING_DOCUMENT)));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_atom_dc_escaping_for_dc255_and_dc616() -> Result<()> {
        let defn_255 = sample_dc_defn(255, "Ignore following Dc", "Escape");
        let defn_616 =
            sample_dc_defn(616, "Evaluate value as atom", "Evaluate");
        let scripts = BTreeSet::from([".Semantic".to_string()]);

        let (nodes1, stats1) = update_ledger_in_memory(
            Vec::new(),
            &[defn_255.clone(), defn_616.clone()],
            &[],
            &scripts,
            None,
        )?;
        assert!(stats1.total_nodes > 0);

        // Verify that statements about Dc 255 and Dc 616 escape their subject with Dc 255
        let escaped_255 = nodes1.iter().any(|n| {
            n.tokens
                == vec![
                    sh(DC_HAS_A),
                    sh(DC_ESCAPE),
                    sh(DC_ESCAPE),
                    sh(DC_CANONICAL_NAME),
                ]
                && n.comment
                    == "Has a (Ignore following Dc, Ignore following Dc, Canonical name)"
        });
        assert!(escaped_255, "Expected `584 255 255 572` statement node");

        let escaped_616 = nodes1.iter().any(|n| {
            n.tokens
                == vec![
                    sh(DC_HAS_A),
                    sh(DC_ESCAPE),
                    sh(DC_EVALUATE_VALUE_AS_ATOM),
                    sh(DC_CANONICAL_NAME),
                ]
                && n.comment
                    == "Has a (Ignore following Dc, Evaluate value as atom, Canonical name)"
        });
        assert!(escaped_616, "Expected `584 255 616 572` statement node");

        // Second run must be idempotent
        let (nodes2, stats2) = update_ledger_in_memory(
            nodes1.clone(),
            &[defn_255, defn_616],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(stats2.nodes_appended, 0);
        assert_eq!(nodes1, nodes2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_chunk_file_rollover_at_100_nodes() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let graph_dir = temp.path().join("docs");
        let mut nodes = Vec::new();
        for i in 0..205usize {
            let i_u128 = u128::try_from(i).unwrap();
            let gid = SYSTEM_REGION_START + i_u128;
            nodes.push(LedgerNode {
                id: gid,
                tokens: encode_integer_value_tokens(i_u128),
                comment: format!("int {i}"),
            });
        }
        let written = write_ledger_nodes_to_dir(&graph_dir, &nodes)?;
        assert_eq!(written, 3);

        let files = list_chunk_files(&graph_dir)?;
        assert_eq!(files.len(), 3);
        assert!(files[0].ends_with("85899_34592.ctbnodes"));
        assert!(files[1].ends_with("85899_34692.ctbnodes"));
        assert!(files[2].ends_with("85899_34792.ctbnodes"));

        let reloaded = load_ledger_nodes_from_dir(&graph_dir)?;
        assert_eq!(reloaded, nodes);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_empty_node_document_and_unwrapped_string_roundtrip() -> Result<()> {
        let empty_node = LedgerNode {
            id: SYSTEM_REGION_START,
            tokens: Vec::new(),
            comment: String::new(),
        };
        assert_eq!(
            empty_node.format_line(),
            format!("l{SYSTEM_REGION_START}: #")
        );
        let parsed_empty = LedgerNode::parse_line(
            &empty_node.format_line(),
            "test.ctbnodes",
            1,
        )?;
        assert_eq!(parsed_empty, empty_node);

        let str_tokens = encode_string_value_tokens("Range");
        assert_eq!(
            str_tokens,
            vec![
                DcShorthand::Unicode(0x52),
                DcShorthand::Unicode(0x61),
                DcShorthand::Unicode(0x6e),
                DcShorthand::Unicode(0x67),
                DcShorthand::Unicode(0x65),
            ]
        );
        assert_eq!(
            decode_string_value_tokens(&str_tokens).as_deref(),
            Some("Range")
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_ctbnbdl_bundle_encode_decode_roundtrip() -> Result<()> {
        let nodes = vec![
            LedgerNode {
                id: SYSTEM_REGION_START,
                tokens: vec![
                    sh(DC_IS_INSTANCE_OF),
                    sh(DC_ENCLOSING_DOCUMENT),
                    sh(DC_BIDIRECTIONAL_CLASS),
                ],
                comment: "Seed entity".to_string(),
            },
            LedgerNode {
                id: SYSTEM_REGION_START + 1,
                tokens: encode_string_value_tokens("Range"),
                comment: "\"Range\"".to_string(),
            },
            LedgerNode {
                id: SYSTEM_REGION_START + 2,
                tokens: Vec::new(),
                comment: String::new(),
            },
            LedgerNode {
                id: SYSTEM_REGION_START + 3,
                tokens: vec![
                    sh(DC_HAS_A),
                    sh(DC_ESCAPE),
                    sh(DC_END_GRAPH_NODE_DEFINITION),
                    sh(DC_CANONICAL_NAME),
                ],
                comment: "Node containing Dc 255 and Dc 618 in body".to_string(),
            },
        ];

        let bundle_bytes = encode_nodes_to_ctbnbdl_bytes(&nodes)?;
        assert!(bundle_bytes.starts_with(&CTBNBDL_MAGIC));
        assert_eq!(
            &bundle_bytes[CTBNBDL_MAGIC.len()..CTBNBDL_MAGIC.len() + DCMM_HEADER.len()],
            &DCMM_HEADER
        );

        let decoded = decode_nodes_from_ctbnbdl_bytes(&bundle_bytes)?;
        assert_eq!(decoded.len(), nodes.len());
        for (orig, dec) in nodes.iter().zip(decoded.iter()) {
            assert_eq!(orig.id, dec.id);
            assert_eq!(orig.tokens, dec.tokens);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_extension_regex_pattern_compilation_and_superseding() -> Result<()> {
        use crate::dc::DC_WILDCARD_PATTERN;
        use crate::dc_def::FormatDetails;

        let mut fmt_defn =
            sample_dc_defn(192, "Internet Archive metadata XML", "IA XML");
        fmt_defn.dc_id = 2_228_416; // f192
        fmt_defn.script = ".Formats:database".to_string();
        fmt_defn.format = Some(FormatDetails {
            extensions: Some(".xml, ~_meta\\.xml$~".to_string()),
            ..FormatDetails::default()
        });
        let scripts = BTreeSet::from([".Formats:database".to_string()]);

        let (nodes1, _) = update_ledger_in_memory(
            Vec::new(),
            &[],
            &[fmt_defn.clone()],
            &scripts,
            None,
        )?;

        // Verify that `~_meta\.xml$~` compiled to a pattern node starting with `600` (`Wildcard pattern`)
        let has_wildcard_pattern = nodes1
            .iter()
            .any(|n| n.tokens.first() == Some(&sh(DC_WILDCARD_PATTERN)));
        assert!(
            has_wildcard_pattern,
            "Expected ~_meta\\.xml$~ to compile into a pattern node starting with Dc 600 (Wildcard pattern)"
        );

        // Now modify the regex extension on `f192` and verify that it supersedes via `591` (`Supersedes`)
        // without retracting the `584 f192 f264` (`Has a`) statement node.
        fmt_defn.format.as_mut().unwrap().extensions =
            Some(".xml, ~^_meta\\.xml$~".to_string());
        let (nodes2, stats2) = update_ledger_in_memory(
            nodes1,
            &[],
            &[fmt_defn],
            &scripts,
            None,
        )?;
        assert_eq!(stats2.nodes_superseded, 1);
        assert_eq!(stats2.nodes_retracted, 0);
        assert_eq!(
            nodes2.last().unwrap().tokens.first(),
            Some(&sh(DC_SUPERSEDES))
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_live_system_graph_ledger_has_zero_drift() -> Result<()> {
        let repo_root = crate::updater::find_repository_root()?;
        let stats = update_system_graph_ledger(&repo_root)?;
        assert_eq!(
            stats.nodes_appended, 0,
            "System graph ledger had pending appends; run ./scripts/validate-data-tables"
        );
        assert_eq!(
            stats.nodes_superseded, 0,
            "System graph ledger had pending supersedes; run ./scripts/validate-data-tables"
        );
        assert_eq!(
            stats.nodes_retracted, 0,
            "System graph ledger had pending retractions; run ./scripts/validate-data-tables"
        );
        assert_eq!(
            stats.files_written, 0,
            "System graph ledger modified chunk files during test; run ./scripts/validate-data-tables"
        );
        Ok(())
    }
}
