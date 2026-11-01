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
//! 100-node chunk files in `assets/data/graph/85899_34592.ctbnodes`, etc.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use ctb_storage_minimal::global_graph_layout::{
    SYSTEM_REGION_END, SYSTEM_REGION_START,
};
use ctb_storage_minimal::shorthand::DcShorthand;
use ctb_utilities::format_id::{
    DC_APPLE_OS_TYPE, DC_COLLECTIVE_TOOLBOX, DC_FILE_EXTENSION, DC_INTEGER,
    DC_LINE_ENDING_CR, DC_LINE_ENDING_CR_LF, DC_LINE_ENDING_LF,
    DC_LINE_ENDING_LF_CR, DC_LINE_ENDING_NL, DC_LINE_ENDING_RS, DC_MEDIA_TYPE,
    DC_UNIFORM_TYPE_IDENTIFIER,
};

pub use crate::ctbnbdl::{
    CTBNBDL_MAGIC, CtbnbdlFramingDcs, DCMM_HEADER, DurableTransactionOutcome,
    FaultInjectionPoint, LEDGER_JOURNAL_REL_PATH, LedgerNode, LedgerReadGuard,
    MAX_BYTES_PER_BUNDLE, MAX_NODES_PER_BUNDLE, MAX_NODES_PER_CHUNK,
    PlannedLedgerItem, TransactionManifest, chunk_filename_for_node_id,
    compile_ctbnodes_dir_to_ctbnbdl,
    compile_ctbnodes_dir_to_ctbnbdl_with_limits,
    compile_ctbnodes_file_to_ctbnbdl_bytes,
    compile_inactive_ctbnodes_dir_to_ctbnbdl,
    compile_inactive_ctbnodes_dir_to_ctbnbdl_with_limits,
    compute_nodes_slice_ledger_sha256, consume_atom_dc_with,
    decode_nodes_from_ctbnbdl_bytes_with, encode_node_definition_tokens_with,
    encode_nodes_to_ctbnbdl_bundles_with_limits,
    encode_nodes_to_ctbnbdl_bytes_with, ensure_ledger_journal_clean,
    execute_durable_ledger_transaction_with_planner,
    format_chunk_bundle_filename, format_chunk_filename, inspect_ledger_journal,
    is_relationship_verb_with, list_chunk_files, load_ledger_nodes_from_dir,
    load_ledger_nodes_from_file, load_ledger_nodes_from_str,
    parse_chunk_bundle_filename_start_id, parse_chunk_filename_start_id,
    parse_statement_tokens_with, partition_active_and_inactive_nodes_with,
    push_atom_dc_with, sanitize_comment, sha256_bytes_hex,
    sync_highwater_shadow_ledgers, verify_nodes_append_only_against_baseline,
    verify_repo_graph_append_only, write_chunk_file_append_only,
    write_ledger_nodes_to_dir,
};

pub mod inbox;
pub use inbox::*;

use crate::dc::{
    DC_ANNOTATION, DC_APPROXIMATE_DECOMPOSITION, DC_BEGIN_ASSOCIATIVITY_GROUP,
    DC_BEGIN_GRAPH_NODE_DEFINITION, DC_BEGIN_NUMBER, DC_BEGIN_PARAMETER,
    DC_BIDIRECTIONAL_CLASS, DC_BOOLEAN_TRUE, DC_CANONICAL_COMBINING_CLASS,
    DC_CANONICAL_DECOMPOSITION, DC_CANONICAL_NAME, DC_CASE_MAPPING_PARTNER,
    DC_CHARACTER_GENERAL_CATEGORY, DC_CHARACTER_SCRIPT,
    DC_COMPATIBILITY_DECOMPOSITION, DC_CROSS_REFERENCE,
    DC_DATA_FORMAT_DESCRIPTION, DC_DC_FORMAT_EXPORT_SUPPORT,
    DC_DC_FORMAT_IMPORT_SUPPORT, DC_DC_FORMAT_TEST_COVERAGE,
    DC_DC_FORMAT_VARIATION_NICKNAME, DC_DC_GENERAL_CATEGORY,
    DC_DEFAULT_FORMAT_VARIANT, DC_DESCRIPTION, DC_ENCLOSING_DOCUMENT,
    DC_END_ASSOCIATIVITY_GROUP, DC_END_GRAPH_NODE_DEFINITION, DC_END_NUMBER,
    DC_ENTITY_TYPE, DC_ESCAPE, DC_EVALUATE_VALUE_AS_ATOM, DC_FORMAL_ALIAS,
    DC_FORMAT_VARIATION_ASPECT, DC_GENERAL_FORMAT_EXPORT_SUPPORT,
    DC_GENERAL_FORMAT_IMPORT_SUPPORT, DC_GENERAL_FORMAT_TEST_COVERAGE,
    DC_HAS_A, DC_HAS_VALUE, DC_IMPLEMENTATION_SUPPORT_LEVEL, DC_INFORMAL_ALIAS,
    DC_IS_ASSOCIATED_WITH, DC_IS_BASED_ON, DC_IS_CLOSING_DELIMITER_FOR,
    DC_IS_DEPRECATED, DC_IS_INSTANCE_OF, DC_IS_SUBTYPE_OF, DC_MAGIC_SIGNATURE,
    DC_NEGATED_PATTERN, DC_NICKNAME, DC_PATTERN_ALTERNATION,
    DC_PATTERN_CAPTURE, DC_PATTERN_REPLACEMENT, DC_PATTERN_REPETITION,
    DC_PREFERRED, DC_PROGRAMMATIC_IDENTIFIER, DC_RANGE,
    DC_REFERENCE_NAMED_OBJECT, DC_REPLACEMENT, DC_RETRACTS,
    DC_ROUTINE_INVOCATION, DC_SELF_REFERENCE, DC_SEMANTIC_DECOMPOSITION,
    DC_SUBPATTERN_MATCH, DC_SUPERSEDES, DC_SUPPORT_COMPLETE,
    DC_SUPPORT_LOSSLESS_ROUNDTRIP, DC_SUPPORT_LOSSLESS_STRICT_VALIDATION,
    DC_SUPPORT_NOT_APPLICABLE, DC_SUPPORT_PARTIAL, DC_SUPPORT_WORK_IN_PROGRESS,
    DC_SYNTAX_RULE, DC_UNICODE_SCRIPT, DC_WILDCARD_PATTERN, DcChar,
};
use crate::dc_def::DcDefn;
use crate::dc_number_minimal::u128_to_dc_number_chars;
use crate::format_spec::{DcToken, FormatExpr, encode_dc_stream, parse_format_expr};
use crate::report::ValidationReport;
use crate::shared::{BidiClass, GeneralCategory, SupportLevel};
use crate::syntax::ast::{
    ActionArg, CharTarget, DcSyntaxRule, Quantifier, SyntaxAction,
    SyntaxElement, SyntaxPattern, SyntaxTerm,
};
use crate::syntax::parse_dc_syntax;

/// Converts a canonical `DcChar` constant into its `DcShorthand` representation.
#[must_use]
fn sh(dc: DcChar) -> DcShorthand {
    DcShorthand::from_global_id(dc.to_u128())
}

/// Returns the canonical `.ctbnbdl` framing Document Characters from `ctb_formats_dcdata::dc`.
#[must_use]
pub fn default_ctbnbdl_framing_dcs() -> CtbnbdlFramingDcs {
    CtbnbdlFramingDcs {
        begin_node: sh(DC_BEGIN_GRAPH_NODE_DEFINITION),
        end_node: sh(DC_END_GRAPH_NODE_DEFINITION),
        escape: sh(DC_ESCAPE),
        evaluate_as_atom: sh(DC_EVALUATE_VALUE_AS_ATOM),
        has_a: sh(DC_HAS_A),
        has_value: sh(DC_HAS_VALUE),
        preferred: sh(DC_PREFERRED),
        supersedes: sh(DC_SUPERSEDES),
        retracts: sh(DC_RETRACTS),
        enclosing_document: sh(DC_ENCLOSING_DOCUMENT),
        is_instance_of: sh(DC_IS_INSTANCE_OF),
        is_subtype_of: sh(DC_IS_SUBTYPE_OF),
        is_based_on: sh(DC_IS_BASED_ON),
        is_deprecated: sh(DC_IS_DEPRECATED),
        is_closing_delimiter_for: sh(DC_IS_CLOSING_DELIMITER_FOR),
        is_associated_with: sh(DC_IS_ASSOCIATED_WITH),
    }
}

/// Partitions `nodes` into `(active_nodes, inactive_nodes)` using [`default_ctbnbdl_framing_dcs`].
#[must_use]
pub fn partition_active_and_inactive_nodes(
    nodes: &[LedgerNode],
) -> (Vec<LedgerNode>, Vec<LedgerNode>) {
    partition_active_and_inactive_nodes_with(nodes, default_ctbnbdl_framing_dcs())
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
pub enum SeedEntityKey {
    /// Unicode bidirectional class (`BN`, `L`, `R`, etc.).
    BidiClass(String),
    /// Unicode or Dc general category (`!Cx`, `Po`, `Ll`, etc.).
    GeneralCategory(String),
    /// Dc or Unicode script (`.Semantic`, `.Symbols`, `Latin`, etc.).
    Script(String),
    /// Named syntax construct (`number`, `string`, `value`, `atom_dc`, etc.).
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
    /// Multi-valued property entries qualified with `Dc 570` (`Preferred`) as the primary choice:
    /// `(subject, prop_token, value_spec)`.
    preferred_multi_properties: BTreeSet<(DcShorthand, DcShorthand, ValueSpec)>,
    /// Single-valued property qualifiers attached to a multi-valued property's `584` (`Has a`) statement node:
    /// `((subject, prop_token, value_spec), qual_prop_token) -> (qual_value_spec, comment)`.
    multi_property_single_qualifiers: BTreeMap<
        ((DcShorthand, DcShorthand, ValueSpec), DcShorthand),
        (ValueSpec, String),
    >,
    /// Preferred replacements (`<prefer>`): `(subject, value_spec) -> comment`.
    preferred_replacements: BTreeMap<(DcShorthand, ValueSpec), String>,
    /// Direct 3-token binary statements: `(verb, subject, object_spec) -> comment`.
    direct_statements: BTreeMap<(DcShorthand, DcShorthand, ValueSpec), String>,
    /// Maps `@vary_as` nicknames (e.g. `"radix"`, `"encoding"`) to their `.Formats:format_variant` Dc shorthand.
    vary_as_to_format_variant: HashMap<String, DcShorthand>,
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

/// Encodes aUTF-8 string into the canonical string value node token sequence
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
    u128_to_dc_number_chars(value)
        .into_iter()
        .map(sh)
        .collect()
}

/// Decodes a canonical base64 integer value node token sequence (`6 f199 <base64 digits> 7`)
/// back into a `u128` (supporting both normalized `6 f199 .. 7` and legacy `6 f199 6 f199 .. 7 7`).
#[must_use]
pub fn decode_integer_value_tokens(tokens: &[DcShorthand]) -> Option<u128> {
    if tokens.len() < 4 {
        return None;
    }
    if tokens.first() != Some(&sh(DC_BEGIN_NUMBER))
        || tokens.get(1) != Some(&sh(DC_INTEGER))
        || tokens.last() != Some(&sh(DC_END_NUMBER))
    {
        return None;
    }
    let mut digit_slice = tokens.get(2..tokens.len().saturating_sub(1))?;
    if digit_slice.first() == Some(&sh(DC_BEGIN_NUMBER))
        && digit_slice.get(1) == Some(&sh(DC_INTEGER))
        && digit_slice.last() == Some(&sh(DC_END_NUMBER))
    {
        digit_slice =
            digit_slice.get(2..digit_slice.len().saturating_sub(1))?;
    }
    if digit_slice.is_empty() {
        return None;
    }
    let base64_start_short = crate::dc::DC_BASE64_START.to_short().ok()?;
    let mut val: u128 = 0;
    for &tok in digit_slice {
        let DcShorthand::Short(short_id) = tok else {
            return None;
        };
        let short_u32 = u32::try_from(short_id).ok()?;
        let offset = short_u32.checked_sub(base64_start_short)?;
        if offset >= 64 {
            return None;
        }
        val = val
            .checked_mul(64)?
            .checked_add(u128::from(offset))?;
    }
    Some(val)
}

fn as_printable_ascii(tok: DcShorthand) -> Option<char> {
    if let DcShorthand::Unicode(cp) = tok {
        if (0x20..=0x7E).contains(&cp) {
            return char::from_u32(cp);
        }
    }
    None
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
        (DC_NICKNAME, "Nickname"),
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
        (
            DC_GENERAL_FORMAT_IMPORT_SUPPORT,
            "General import support for a format",
        ),
        (
            DC_DC_FORMAT_IMPORT_SUPPORT,
            "Import support to Dc documents for a format",
        ),
        (
            DC_GENERAL_FORMAT_EXPORT_SUPPORT,
            "General export support for a format",
        ),
        (
            DC_DC_FORMAT_EXPORT_SUPPORT,
            "Export support from Dc documents for a format",
        ),
        (
            DC_GENERAL_FORMAT_TEST_COVERAGE,
            "General test coverage for a format",
        ),
        (
            DC_DC_FORMAT_TEST_COVERAGE,
            "Test coverage of Dc document conversion for a format",
        ),
        (
            DC_IMPLEMENTATION_SUPPORT_LEVEL,
            "Implementation support level",
        ),
        (DC_SUPPORT_NOT_APPLICABLE, "Support not applicable"),
        (DC_SUPPORT_WORK_IN_PROGRESS, "Work-in-progress support"),
        (DC_SUPPORT_PARTIAL, "Partial or basic dialect support"),
        (DC_SUPPORT_COMPLETE, "Complete implementation"),
        (DC_SUPPORT_LOSSLESS_ROUNDTRIP, "Can be losslessly roundtripped"),
        (
            DC_SUPPORT_LOSSLESS_STRICT_VALIDATION,
            "Lossless roundtripping with strict validation",
        ),
        (DC_FORMAT_VARIATION_ASPECT, "Format variation aspect"),
        (DC_DEFAULT_FORMAT_VARIANT, "Default format variant"),
        (
            DC_DC_FORMAT_VARIATION_NICKNAME,
            "Dc format variation nickname",
        ),
        (DC_FILE_EXTENSION, "File extension"),
        (DC_MEDIA_TYPE, "IANA Media Type"),
        (DC_UNIFORM_TYPE_IDENTIFIER, "Uniform Type Identifier"),
        (DC_APPLE_OS_TYPE, "Apple OSType"),
        (DC_COLLECTIVE_TOOLBOX, "Collective Toolbox"),
        (DC_LINE_ENDING_LF, "Lines terminated by LF"),
        (DC_LINE_ENDING_CR, "Lines terminated by CR"),
        (DC_LINE_ENDING_CR_LF, "Lines terminated by CRLF"),
        (DC_LINE_ENDING_LF_CR, "Lines terminated by LFCR"),
        (DC_LINE_ENDING_RS, "Lines terminated by RS"),
        (DC_LINE_ENDING_NL, "Lines terminated by NL (NEL)"),
    ];
    for &(dc, label) in builtins {
        map.entry(sh(dc)).or_insert_with(|| label.to_string());
    }
    map
}

/// Encodes a single `LedgerNode` (`id` + `tokens`) into its `Dc 617 .. Dc 618` token sequence:
/// `617 [atom_dc: l<id>] <escaped body tokens...> 618`.
#[must_use]
pub fn encode_node_definition_tokens(node: &LedgerNode) -> Vec<DcShorthand> {
    encode_node_definition_tokens_with(node, default_ctbnbdl_framing_dcs())
}

/// Compiles a slice of `LedgerNode` entries into a `.ctbnbdl` (`CtbNodeBundle`) binary buffer:
/// 8-byte `CTBNBDL\x01` magic + 21-byte `DcMM` header + `UTF-8e-128` encoded `617 .. 618` frames.
pub fn encode_nodes_to_ctbnbdl_bytes(nodes: &[LedgerNode]) -> Result<Vec<u8>> {
    encode_nodes_to_ctbnbdl_bytes_with(nodes, default_ctbnbdl_framing_dcs())
}

/// Decodes a `.ctbnbdl` (`CtbNodeBundle`) binary buffer back into a list of `LedgerNode` entries
/// (with empty comments, as `.ctbnbdl` stores only the semantic Dc stream).
pub fn decode_nodes_from_ctbnbdl_bytes(bytes: &[u8]) -> Result<Vec<LedgerNode>> {
    decode_nodes_from_ctbnbdl_bytes_with(bytes, default_ctbnbdl_framing_dcs())
}

/// Active index of a loaded ledger, tracking value nodes, seed entities, and active statements.
struct LedgerBuilder {
    nodes: Vec<LedgerNode>,
    appended_count: usize,
    superseded_count: usize,
    retracted_count: usize,
    /// Verified provenance ownership (`Csv` vs `Manual`) for existing nodes.
    provenance_map: NodeProvenanceMap,
    /// Maps a CSV node ID (`u128`) to an active `Manual` node ID (`u128`) that references it.
    manual_referenced_ids: HashMap<u128, u128>,
    /// Non-canonical ambiguity branch statement IDs (`Dc 348 "Ambiguous values"`) pending `Dc 592 "Retracts"`.
    pending_ambiguity_branch_retractions: Vec<u128>,
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
    /// Active `584 l<has_a_id> 570` (`Preferred`) qualifiers on multi-valued properties:
    /// `(subject, prop_token, val_token) -> pref_qual_id`.
    active_multi_preferred:
        BTreeMap<(DcShorthand, DcShorthand, DcShorthand), u128>,
    /// Active `<prefer>` replacements: `(subject, val_token) -> (has_a_id, has_val_id, pref_qual_id)`.
    active_preferred: BTreeMap<(DcShorthand, DcShorthand), (u128, u128, u128)>,
    /// Active direct binary statements: `(verb, subject, object) -> stmt_id`.
    active_direct_stmts: BTreeMap<(DcShorthand, DcShorthand, DcShorthand), u128>,
    /// Maps `@vary_as` nicknames to their `.Formats:format_variant` Dc shorthand.
    vary_as_to_format_variant: HashMap<String, DcShorthand>,
}

impl LedgerBuilder {
    /// Initializes a `LedgerBuilder` from existing ledger nodes with default (`Csv`) provenance.
    #[cfg(test)]
    fn from_existing(
        existing: Vec<LedgerNode>,
        single_valued_props: &HashSet<DcShorthand>,
        initial_token_labels: HashMap<DcShorthand, String>,
    ) -> Self {
        Self::from_existing_with_provenance(
            existing,
            single_valued_props,
            initial_token_labels,
            NodeProvenanceMap::default(),
        )
    }

    /// Initializes a `LedgerBuilder` from existing ledger nodes and indexes all active statements
    /// via `SeedGraphIndex::from_nodes_with_single_props`.
    fn from_existing_with_provenance(
        existing: Vec<LedgerNode>,
        single_valued_props: &HashSet<DcShorthand>,
        initial_token_labels: HashMap<DcShorthand, String>,
        provenance_map: NodeProvenanceMap,
    ) -> Self {
        let mut token_labels = initial_token_labels;
        let (effective_inactive_ids, _, _) =
            crate::seed_graph::compute_effective_inactive_ids(&existing);
        let (_, pending_ambiguity_branch_retractions) =
            crate::seed_graph::compute_non_canonical_ambiguity_branch_ids(
                &existing,
                &effective_inactive_ids,
            );
        let index = crate::seed_graph::SeedGraphIndex::from_nodes_with_single_props(
            existing,
            single_valued_props,
        );

        let mut manual_referenced_ids = HashMap::new();
        for node in &index.nodes {
            if !index.inactive_ids.contains(&node.id)
                && provenance_map.owner(node.id) == NodeProvenance::Manual
            {
                for &tok in &node.tokens {
                    if let DcShorthand::Long(ref_id) = tok
                        && !index.inactive_ids.contains(&ref_id)
                        && provenance_map.owner(ref_id) == NodeProvenance::Csv
                    {
                        manual_referenced_ids.entry(ref_id).or_insert(node.id);
                    }
                }
            }
        }

        for (&node_id, decoded) in &index.string_by_node_id {
            token_labels.insert(
                DcShorthand::Long(node_id),
                format_string_summary(decoded),
            );
        }
        for (&node_id, val) in &index.integer_by_node_id {
            token_labels.insert(
                DcShorthand::Long(node_id),
                val.to_string(),
            );
        }
        for (seed_key, &seed_id) in &index.seed_entity_ids {
            token_labels.insert(
                DcShorthand::Long(seed_id),
                seed_key.comment(),
            );
        }
        for node in &index.nodes {
            token_labels
                .entry(DcShorthand::Long(node.id))
                .or_insert_with(|| node.comment.clone());
        }

        let mut vary_as_to_format_variant = default_vary_as_to_format_variant();
        let sh_var_nick = sh(DC_DC_FORMAT_VARIATION_NICKNAME);
        for (&(subj, prop), &(_, _, val_tok)) in &index.active_single_props {
            if prop == sh_var_nick {
                if let DcShorthand::Long(str_id) = val_tok {
                    if let Some(nick) = index.string_by_node_id.get(&str_id) {
                        vary_as_to_format_variant.insert(nick.clone(), subj);
                    }
                }
            }
        }

        Self {
            nodes: index.nodes,
            appended_count: 0,
            superseded_count: 0,
            retracted_count: 0,
            provenance_map,
            manual_referenced_ids,
            pending_ambiguity_branch_retractions,
            token_labels,
            string_by_node_id: index.string_by_node_id,
            value_node_by_tokens: index.value_node_by_tokens,
            seed_entity_ids: index.seed_entity_ids,
            active_single_props: index.active_single_props,
            active_multi_props: index.active_multi_props,
            active_multi_preferred: index.active_multi_preferred,
            active_preferred: index.active_preferred,
            active_direct_stmts: index.active_direct_stmts,
            vary_as_to_format_variant,
        }
    }

    fn is_manual_node(&self, node_id: u128) -> bool {
        self.provenance_map.owner(node_id) == NodeProvenance::Manual
    }

    fn ensure_can_modify_csv_node(
        &self,
        target_id: u128,
        action: &str,
    ) -> Result<()> {
        ensure!(
            !self.is_manual_node(target_id),
            "Cross-ownership conflict: CSV reconciliation cannot {action} manual node l{target_id}"
        );
        if let Some(&manual_id) = self.manual_referenced_ids.get(&target_id) {
            bail!(
                "Cross-ownership conflict: CSV reconciliation cannot {action} CSV node l{target_id} because active manual node l{manual_id} references it"
            );
        }
        Ok(())
    }

    fn describe_token(&self, tok: DcShorthand) -> String {
        if let Some(label) = self.token_labels.get(&tok) {
            return label.clone();
        }
        match tok {
            DcShorthand::Unicode(cp) => {
                if let Some(name) = ctb_formats_unicode::get_unicode_name(cp) {
                    name
                } else {
                    format!("U+{cp:04X}")
                }
            }
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
        let mut parts: Vec<String> = Vec::new();
        let mut i = 0usize;
        while i < tokens.len() {
            let Some(&tok) = tokens.get(i) else {
                break;
            };
            if tok == sh(DC_RANGE) {
                parts.push(self.describe_token(tok));
                i = i.saturating_add(1);
                for _ in 0..2 {
                    if let Some(&bound) = tokens.get(i) {
                        if matches!(bound, DcShorthand::Unicode(_)) {
                            parts.push(self.describe_token(bound));
                            i = i.saturating_add(1);
                        } else {
                            break;
                        }
                    }
                }
                continue;
            }
            if let Some(ch) = as_printable_ascii(tok) {
                let mut text = String::new();
                text.push(ch);
                i = i.saturating_add(1);
                while let Some(&next_tok) = tokens.get(i) {
                    if let Some(next_ch) = as_printable_ascii(next_tok) {
                        text.push(next_ch);
                        i = i.saturating_add(1);
                    } else {
                        break;
                    }
                }
                parts.push(format_string_summary(&text));
                continue;
            }
            parts.push(self.describe_token(tok));
            i = i.saturating_add(1);
        }
        parts.join(", ")
    }

    fn append_raw_node(&mut self, tokens: Vec<DcShorthand>) -> Result<u128> {
        let next_id = SYSTEM_REGION_START
            .saturating_add(u128::try_from(self.nodes.len())?);
        ensure!(
            next_id <= SYSTEM_REGION_END,
            "System graph region exhausted at node ID l{next_id}"
        );
        let comment = self.format_tokens_comment(&tokens);
        self.nodes.push(LedgerNode {
            id: next_id,
            tokens,
            comment: comment.clone(),
        });
        self.token_labels
            .insert(DcShorthand::Long(next_id), comment);
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
        let sh_begin_param = sh(DC_BEGIN_PARAMETER);
        let mut shorthands = Vec::with_capacity(dc_tokens.len());
        let mut prev_sh = None;
        for tok in dc_tokens {
            match tok {
                DcToken::Dc(shorthand) => {
                    shorthands.push(shorthand);
                    prev_sh = Some(shorthand);
                }
                DcToken::NamedType(name) => {
                    if prev_sh == Some(sh_begin_param) {
                        if let Some(&variant_sh) =
                            self.vary_as_to_format_variant.get(&name)
                        {
                            shorthands.push(variant_sh);
                            prev_sh = Some(variant_sh);
                            continue;
                        }
                    }
                    let seed_key = SeedEntityKey::NamedConstruct(name.clone());
                    let resolved = if let Some(&construct_id) =
                        self.seed_entity_ids.get(&seed_key)
                    {
                        DcShorthand::Long(construct_id)
                    } else if let Ok(int_val) = name.parse::<u128>() {
                        let int_id = self.get_or_insert_value_node(
                            encode_integer_value_tokens(int_val),
                            int_val.to_string(),
                        )?;
                        DcShorthand::Long(int_id)
                    } else {
                        // Reason for fallback: unquoted literal names are used as-is; quoted strings unescape interior quotes.
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
                        DcShorthand::Long(str_id)
                    };
                    shorthands.push(resolved);
                    prev_sh = Some(resolved);
                }
            }
        }
        Ok(shorthands)
    }

    fn apply_desired_state(
        &mut self,
        mut desired: DesiredDatasetState,
    ) -> Result<()> {
        for (nick, variant_sh) in &desired.vary_as_to_format_variant {
            self.vary_as_to_format_variant
                .insert(nick.clone(), *variant_sh);
        }

        // 0. Retract any non-canonical ambiguity branch statements (`Dc 348 "Ambiguous values"`).
        let pending_retractions =
            std::mem::take(&mut self.pending_ambiguity_branch_retractions);
        for stmt_id in pending_retractions {
            if !self.is_manual_node(stmt_id) {
                self.retract_node(stmt_id)?;
            }
        }

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

        let seed_entity_id_set: HashSet<u128> =
            self.seed_entity_ids.values().copied().collect();

        // 2. Resolve all desired single-valued properties and diff against active single-valued properties.
        let mut resolved_single: BTreeMap<
            (DcShorthand, DcShorthand),
            (DcShorthand, String),
        > = BTreeMap::new();
        for ((subj, prop), (val_spec, comment)) in &desired.single_properties {
            let val_tok = self.resolve_value_spec(val_spec)?;
            resolved_single.insert((*subj, *prop), (val_tok, comment.clone()));
        }

        // Retract removed single-valued properties on top-level/seed subjects (qualifiers on
        // multi-valued property statement nodes are reconciled in Step 3 / Step 3c).
        let existing_single_keys: Vec<(DcShorthand, DcShorthand)> =
            self.active_single_props.keys().copied().collect();
        for key in existing_single_keys {
            if matches!(key.0, DcShorthand::Long(id) if !seed_entity_id_set.contains(&id))
            {
                continue;
            }
            if !resolved_single.contains_key(&key) {
                if let Some(&(has_a_id, has_val_id, _)) =
                    self.active_single_props.get(&key)
                {
                    if self.is_manual_node(has_a_id)
                        || self.is_manual_node(has_val_id)
                    {
                        continue;
                    }
                }
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
                    ensure!(
                        !self.is_manual_node(has_a_id)
                            && !self.is_manual_node(old_has_val_id),
                        "Cross-ownership conflict: CSV reconciliation for single-valued property ({subj}, {prop}) with value {val_tok} conflicts with active manual node l{has_a_id}/l{old_has_val_id} (value {old_val_tok})"
                    );
                    let new_has_val_id = self.append_statement_node(
                        sh(DC_HAS_VALUE),
                        DcShorthand::Long(has_a_id),
                        val_tok,
                    )?;
                    self.supersede_node(new_has_val_id, old_has_val_id)?;
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
                if let Some(&(has_a_id, has_val_id)) =
                    self.active_multi_props.get(&key)
                {
                    if self.is_manual_node(has_a_id)
                        || self.is_manual_node(has_val_id)
                    {
                        continue;
                    }
                }
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
            // Reason for fallback: (subj, prop) may exist only in added_by_subj_prop, defaulting removed list to empty.
            let mut removed = removed_by_subj_prop
                .remove(&(subj, prop))
                .unwrap_or_default();
            // Sort by `has_a_id` so superseding preserves original statement order
            removed.sort_by_key(|&(_, has_a_id, _)| has_a_id);
            // Reason for fallback: (subj, prop) may exist only in removed_by_subj_prop, defaulting added list to empty.
            let mut added = added_by_subj_prop
                .remove(&(subj, prop))
                .unwrap_or_default();

            let mut superseded_pairs: Vec<
                ((u128, u128), (DcShorthand, DcShorthand)),
            > = Vec::new();

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
                    superseded_pairs.push((
                        (has_a_id, old_has_val_id),
                        (old_val_tok, new_val_tok),
                    ));
                } else {
                    rem_idx = rem_idx.saturating_add(1);
                }
            }

            // Second pass: pair any remaining removed and added values on the same `(subj, prop)`
            while !removed.is_empty() && !added.is_empty() {
                let (old_val_tok, has_a_id, old_has_val_id) = removed.remove(0);
                let (new_val_tok, _) = added.remove(0);
                superseded_pairs.push((
                    (has_a_id, old_has_val_id),
                    (old_val_tok, new_val_tok),
                ));
            }

            superseded_pairs.sort_by_key(|&((has_a_id, _), _)| has_a_id);
            for ((has_a_id, old_has_val_id), (old_val_tok, new_val_tok)) in
                superseded_pairs
            {
                let new_has_val_id = self.append_statement_node(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(has_a_id),
                    new_val_tok,
                )?;
                self.supersede_node(new_has_val_id, old_has_val_id)?;
                if let Some(pref_id) =
                    self.active_multi_preferred.remove(&(subj, prop, old_val_tok))
                {
                    self.active_multi_preferred
                        .insert((subj, prop, new_val_tok), pref_id);
                }
                self.active_multi_props.insert(
                    (subj, prop, new_val_tok),
                    (has_a_id, new_has_val_id),
                );
            }

            // Retract any leftover unpaired removed values (and any qualifiers on their `has_a_id`)
            for (old_val_tok, has_a_id, old_has_val_id) in removed {
                if let Some(pref_id) =
                    self.active_multi_preferred.remove(&(subj, prop, old_val_tok))
                {
                    self.retract_node(pref_id)?;
                }
                let qual_keys: Vec<(DcShorthand, DcShorthand)> = self
                    .active_single_props
                    .keys()
                    .filter(|&&(s, _)| s == DcShorthand::Long(has_a_id))
                    .copied()
                    .collect();
                for qk in qual_keys {
                    if let Some((q_has_a_id, q_has_val_id, _)) =
                        self.active_single_props.remove(&qk)
                    {
                        self.retract_node(q_has_val_id)?;
                        self.retract_node(q_has_a_id)?;
                    }
                }
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

        // 3b. Resolve and diff `Dc 570` (`Preferred`) qualifiers on multi-valued properties.
        let mut resolved_multi_pref: BTreeSet<(
            DcShorthand,
            DcShorthand,
            DcShorthand,
        )> = BTreeSet::new();
        for (subj, prop, val_spec) in &desired.preferred_multi_properties {
            let val_tok = self.resolve_value_spec(val_spec)?;
            resolved_multi_pref.insert((*subj, *prop, val_tok));
        }

        let existing_multi_pref_keys: Vec<(
            DcShorthand,
            DcShorthand,
            DcShorthand,
        )> = self.active_multi_preferred.keys().copied().collect();
        for key in existing_multi_pref_keys {
            if !resolved_multi_pref.contains(&key) {
                if let Some(&pref_id) = self.active_multi_preferred.get(&key) {
                    if self.is_manual_node(pref_id) {
                        continue;
                    }
                }
                if let Some(pref_id) = self.active_multi_preferred.remove(&key)
                {
                    self.retract_node(pref_id)?;
                }
            }
        }

        for (subj, prop, val_tok) in resolved_multi_pref {
            if !self
                .active_multi_preferred
                .contains_key(&(subj, prop, val_tok))
            {
                if let Some(&(has_a_id, _)) =
                    self.active_multi_props.get(&(subj, prop, val_tok))
                {
                    let pref_id = self.append_statement_node(
                        sh(DC_HAS_A),
                        DcShorthand::Long(has_a_id),
                        sh(DC_PREFERRED),
                    )?;
                    self.active_multi_preferred
                        .insert((subj, prop, val_tok), pref_id);
                }
            }
        }

        // 3c. Resolve and diff single-valued qualifiers attached to multi-valued property statements.
        let mut resolved_qual_single: BTreeMap<
            (DcShorthand, DcShorthand),
            (DcShorthand, String),
        > = BTreeMap::new();
        for (((subj, prop, val_spec), qual_prop), (qual_val_spec, comment)) in
            &desired.multi_property_single_qualifiers
        {
            let val_tok = self.resolve_value_spec(val_spec)?;
            let qual_val_tok = self.resolve_value_spec(qual_val_spec)?;
            if let Some(&(has_a_id, _)) =
                self.active_multi_props.get(&(*subj, *prop, val_tok))
            {
                resolved_qual_single.insert(
                    (DcShorthand::Long(has_a_id), *qual_prop),
                    (qual_val_tok, comment.clone()),
                );
            }
        }

        let existing_qual_single_keys: Vec<(DcShorthand, DcShorthand)> = self
            .active_single_props
            .keys()
            .filter(|&&(s, _)| {
                matches!(s, DcShorthand::Long(id) if !seed_entity_id_set.contains(&id))
            })
            .copied()
            .collect();
        for key in existing_qual_single_keys {
            if !resolved_qual_single.contains_key(&key) {
                if let Some(&(has_a_id, has_val_id, _)) =
                    self.active_single_props.get(&key)
                {
                    if self.is_manual_node(has_a_id)
                        || self.is_manual_node(has_val_id)
                    {
                        continue;
                    }
                }
                if let Some((has_a_id, has_val_id, _)) =
                    self.active_single_props.remove(&key)
                {
                    self.retract_node(has_val_id)?;
                    self.retract_node(has_a_id)?;
                }
            }
        }

        for ((qual_subj, qual_prop), (qual_val_tok, _comment)) in
            resolved_qual_single
        {
            if let Some(&(has_a_id, old_has_val_id, old_val_tok)) =
                self.active_single_props.get(&(qual_subj, qual_prop))
            {
                if old_val_tok != qual_val_tok {
                    ensure!(
                        !self.is_manual_node(has_a_id)
                            && !self.is_manual_node(old_has_val_id),
                        "Cross-ownership conflict: CSV reconciliation for qualifier ({qual_subj}, {qual_prop}) with value {qual_val_tok} conflicts with active manual node l{has_a_id}/l{old_has_val_id} (value {old_val_tok})"
                    );
                    let new_has_val_id = self.append_statement_node(
                        sh(DC_HAS_VALUE),
                        DcShorthand::Long(has_a_id),
                        qual_val_tok,
                    )?;
                    self.supersede_node(new_has_val_id, old_has_val_id)?;
                    self.active_single_props.insert(
                        (qual_subj, qual_prop),
                        (has_a_id, new_has_val_id, qual_val_tok),
                    );
                }
            } else {
                let has_a_id = self.append_statement_node(
                    sh(DC_HAS_A),
                    qual_subj,
                    qual_prop,
                )?;
                let has_val_id = self.append_statement_node(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(has_a_id),
                    qual_val_tok,
                )?;
                self.active_single_props.insert(
                    (qual_subj, qual_prop),
                    (has_a_id, has_val_id, qual_val_tok),
                );
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
                if let Some(&(has_a_id, has_val_id, pref_id)) =
                    self.active_preferred.get(&key)
                {
                    if self.is_manual_node(has_a_id)
                        || self.is_manual_node(has_val_id)
                        || self.is_manual_node(pref_id)
                    {
                        continue;
                    }
                }
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
                if let Some(&stmt_id) = self.active_direct_stmts.get(&key) {
                    if self.is_manual_node(stmt_id) {
                        continue;
                    }
                }
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

    fn supersede_node(&mut self, new_id: u128, old_id: u128) -> Result<u128> {
        self.ensure_can_modify_csv_node(old_id, "supersede")?;
        let id = self.append_statement_node(
            sh(DC_SUPERSEDES),
            DcShorthand::Long(new_id),
            DcShorthand::Long(old_id),
        )?;
        self.superseded_count = self.superseded_count.saturating_add(1);
        Ok(id)
    }

    fn retract_node(&mut self, target_id: u128) -> Result<u128> {
        self.ensure_can_modify_csv_node(target_id, "retract")?;
        let id = self.append_statement_node(
            sh(DC_RETRACTS),
            sh(DC_ENCLOSING_DOCUMENT),
            DcShorthand::Long(target_id),
        )?;
        self.retracted_count = self.retracted_count.saturating_add(1);
        Ok(id)
    }
}

/// Returns the canonical set of single-valued property Document Characters.
#[must_use]
pub fn single_valued_property_set() -> HashSet<DcShorthand> {
    HashSet::from([
        sh(DC_CANONICAL_NAME),
        sh(DC_DESCRIPTION),
        sh(DC_PROGRAMMATIC_IDENTIFIER),
        sh(DC_CANONICAL_COMBINING_CLASS),
        sh(DC_BIDIRECTIONAL_CLASS),
        sh(DC_CASE_MAPPING_PARTNER),
        sh(DC_SYNTAX_RULE),
        sh(DC_DATA_FORMAT_DESCRIPTION),
        sh(DC_IMPLEMENTATION_SUPPORT_LEVEL),
        sh(DC_DEFAULT_FORMAT_VARIANT),
        sh(DC_DC_FORMAT_VARIATION_NICKNAME),
    ])
}

/// Returns the default mapping from `@vary_as` nicknames to `.Formats:format_variant` shorthands.
#[must_use]
pub fn default_vary_as_to_format_variant() -> HashMap<String, DcShorthand> {
    HashMap::from([
        ("radix".to_string(), DcShorthand::Format(354)),
        ("lineEndings".to_string(), DcShorthand::Format(609)),
        ("unicodePua".to_string(), DcShorthand::Format(610)),
        ("case".to_string(), DcShorthand::Format(611)),
        ("padding".to_string(), DcShorthand::Format(612)),
        ("alphabet".to_string(), DcShorthand::Format(613)),
        ("encoding".to_string(), DcShorthand::Format(650)),
    ])
}

fn push_atom_dc(out: &mut Vec<DcShorthand>, tok: DcShorthand) {
    push_atom_dc_with(out, tok, default_ctbnbdl_framing_dcs());
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

/// Parses a relationship statement `<verb> [atom_dc] [atom_dc]` back into `(verb, subj, obj)`.
#[must_use]
pub fn parse_statement_tokens(
    tokens: &[DcShorthand],
) -> Option<(DcShorthand, DcShorthand, DcShorthand)> {
    parse_statement_tokens_with(tokens, default_ctbnbdl_framing_dcs())
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
            // Reason for fallback: missing first column defaults to empty string and triggers ensure! error below.
            let name = row.first().map_or("", |s| s.trim());
            // Reason for fallback: missing second column defaults to empty string and triggers ensure! error below.
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
            collect_named_constructs_from_pattern(
                &parsed.pattern,
                &mut state.seed_entities,
            );
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
                        if !state.named_construct_rules.contains_key(&type_name)
                        {
                            let (pattern, raw_syntax) = if let Some(imp) = defn
                                .implies
                                .iter()
                                .map(|s| s.trim())
                                .find(|s| {
                                    s.starts_with('[') && s.ends_with(']')
                                }) {
                                let raw = format!(":{imp}");
                                let parsed =
                                    parse_dc_syntax(&raw).with_context(|| {
                                        format!(
                                            "Invalid @implies syntax '{imp}' on .EL Types '{}'",
                                            defn.name
                                        )
                                    })?;
                                (parsed.pattern, raw)
                            } else {
                                let lower = type_name.to_ascii_lowercase();
                                if let Some((existing_pat, existing_raw)) =
                                    state
                                        .named_construct_rules
                                        .get(&lower)
                                        .cloned()
                                {
                                    (existing_pat, existing_raw)
                                } else {
                                    let raw = format!(
                                        ":260 262 {short_u32} 263 [^261]+ 261"
                                    );
                                    let parsed = parse_dc_syntax(&raw)
                                        .with_context(|| {
                                            format!(
                                                "Failed to synthesize pattern for .EL Types '{type_name}'"
                                            )
                                        })?;
                                    (parsed.pattern, raw)
                                }
                            };
                            collect_named_constructs_from_pattern(
                                &pattern,
                                &mut state.seed_entities,
                            );
                            state
                                .named_construct_rules
                                .insert(type_name, (pattern, raw_syntax));
                        }
                    }
                }
            }
        }
    }

    // 3. Compile all Dc and Format definitions in deterministic `dc_id` order.
    let mut all_defns: Vec<&DcDefn> =
        dc_defns.iter().chain(format_defns.iter()).collect();
    all_defns.sort_by_key(|d| d.dc_id);

    state.vary_as_to_format_variant = default_vary_as_to_format_variant();
    for defn in &all_defns {
        if defn.script == ".Formats:format_variant" {
            if let Some(fmt) = &defn.format {
                if let Some(vary_as) = &fmt.vary_as {
                    if !vary_as.is_empty() {
                        state.vary_as_to_format_variant.insert(
                            vary_as.clone(),
                            DcShorthand::from_global_id(defn.dc_id),
                        );
                    }
                }
            }
        }
    }

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
    if !matches!(subj, DcShorthand::Format(_)) {
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

    // Informal aliases (`576`) for Document Characters, or Nicknames (`575`) for Formats
    let alias_prop = if defn.format.is_some() {
        sh(DC_NICKNAME)
    } else {
        sh(DC_INFORMAL_ALIAS)
    };
    let non_empty_aliases: Vec<&str> = defn
        .aliases
        .iter()
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .collect();
    for (idx, &alias) in non_empty_aliases.iter().enumerate() {
        let spec = make_string_value_spec(alias);
        if defn.format.is_some() && non_empty_aliases.len() > 1 && idx == 0 {
            state
                .preferred_multi_properties
                .insert((subj, alias_prop, spec.clone()));
        }
        state.multi_properties.insert(
            (subj, alias_prop, spec),
            format!("{subj} alias {alias:?}"),
        );
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
        // Reason for fallback: cross-reference without leading '>' prefix is used directly.
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
        let mut all_shorthands = true;
        for tok_str in payload.split_whitespace() {
            if let Ok(shorthand) = DcShorthand::parse(tok_str) {
                payload_tokens.push(shorthand);
            } else {
                all_shorthands = false;
                break;
            }
        }
        let val_spec = if all_shorthands && payload_tokens.len() == 1 {
            #[expect(
                clippy::expect_used,
                reason = "payload_tokens.len() == 1 checked immediately above"
            )]
            ValueSpec::Direct(*payload_tokens.first().expect("single token"))
        } else if all_shorthands && !payload_tokens.is_empty() {
            ValueSpec::ValueNode {
                tokens: payload_tokens,
                comment: format!("<{tag}> {payload}"),
            }
        } else if let Ok(expr) = parse_format_expr(payload)
            && !matches!(expr, FormatExpr::Dc(_) | FormatExpr::NamedType(_))
        {
            ValueSpec::FormatExprNode {
                expr: Box::new(expr),
                raw_comment: format!("<{tag}> {payload}"),
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
            let ext_list: Vec<&str> = exts
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            for (idx, ext) in ext_list.iter().copied().enumerate() {
                let ext_spec = if crate::syntax::is_extension_regex(ext) {
                    let rule = crate::syntax::parse_extension_regex(ext)?;
                    ValueSpec::PatternNode {
                        rule: Box::new(rule),
                        comment: ext.to_string(),
                    }
                } else {
                    make_string_value_spec(ext)
                };
                if ext_list.len() > 1 && idx == 0 {
                    state.preferred_multi_properties.insert((
                        subj,
                        sh(DC_FILE_EXTENSION),
                        ext_spec.clone(),
                    ));
                }
                state.multi_properties.insert(
                    (subj, sh(DC_FILE_EXTENSION), ext_spec),
                    format!("{subj} ext {ext:?}"),
                );
            }
        }
        if let Some(mimes) = &fmt.mime {
            let mime_list: Vec<&str> = mimes
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            for (idx, mime) in mime_list.iter().copied().enumerate() {
                let spec = make_string_value_spec(mime);
                if mime_list.len() > 1 && idx == 0 {
                    state.preferred_multi_properties.insert((
                        subj,
                        sh(DC_MEDIA_TYPE),
                        spec.clone(),
                    ));
                }
                state.multi_properties.insert(
                    (subj, sh(DC_MEDIA_TYPE), spec),
                    format!("{subj} mime {mime:?}"),
                );
            }
        }
        if let Some(utis) = &fmt.uti {
            let uti_list: Vec<&str> = utis
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            for (idx, uti) in uti_list.iter().copied().enumerate() {
                let spec = make_string_value_spec(uti);
                if uti_list.len() > 1 && idx == 0 {
                    state.preferred_multi_properties.insert((
                        subj,
                        sh(DC_UNIFORM_TYPE_IDENTIFIER),
                        spec.clone(),
                    ));
                }
                state.multi_properties.insert(
                    (subj, sh(DC_UNIFORM_TYPE_IDENTIFIER), spec),
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

        // Format support & test coverage on `f649` (`Collective Toolbox`)
        for (raw_opt, gen_cap_dc, dc_cap_dc) in [
            (
                fmt.import_support.as_deref(),
                DC_GENERAL_FORMAT_IMPORT_SUPPORT,
                DC_DC_FORMAT_IMPORT_SUPPORT,
            ),
            (
                fmt.export_support.as_deref(),
                DC_GENERAL_FORMAT_EXPORT_SUPPORT,
                DC_DC_FORMAT_EXPORT_SUPPORT,
            ),
            (
                fmt.tests.as_deref(),
                DC_GENERAL_FORMAT_TEST_COVERAGE,
                DC_DC_FORMAT_TEST_COVERAGE,
            ),
        ] {
            if let Some(raw) = raw_opt {
                let parsed = SupportLevel::parse(raw)?;
                for (cap_dc, lvl) in
                    [(gen_cap_dc, parsed.general), (dc_cap_dc, parsed.dc)]
                {
                        if let Some(lvl_dc) = support_level_to_dc(lvl) {
                            let ctb_subj = sh(DC_COLLECTIVE_TOOLBOX);
                            let cap_sh = sh(cap_dc);
                            let fmt_val = ValueSpec::Direct(subj);
                            state.multi_properties.insert(
                                (ctb_subj, cap_sh, fmt_val.clone()),
                                format!("{ctb_subj} {cap_sh} {subj}"),
                            );
                            state.multi_property_single_qualifiers.insert(
                                (
                                    (ctb_subj, cap_sh, fmt_val),
                                    sh(DC_IMPLEMENTATION_SUPPORT_LEVEL),
                                ),
                                (
                                    ValueSpec::Direct(sh(lvl_dc)),
                                    format!(
                                        "{ctb_subj} {cap_sh} {subj} support level {}",
                                        sh(lvl_dc)
                                    ),
                                ),
                            );
                        }
                    }
                }
        }

        // Format variation nickname (`Dc 634`): `@vary_as`
        if let Some(vary_as) = &fmt.vary_as {
            if !vary_as.is_empty() {
                state.single_properties.insert(
                    (subj, sh(DC_DC_FORMAT_VARIATION_NICKNAME)),
                    (
                        make_string_value_spec(vary_as),
                        format!("{subj} vary_as {vary_as:?}"),
                    ),
                );
            }
        }

        // Format variation aspects (`Dc 632`): `@varies` and `Variant Types`
        for &target_sh in &fmt.varies {
            state.multi_properties.insert(
                (
                    subj,
                    sh(DC_FORMAT_VARIATION_ASPECT),
                    ValueSpec::Direct(target_sh),
                ),
                format!("{subj} varies {target_sh}"),
            );
        }
        if let Some(vtypes) = &fmt.variant_types {
            for vtype in
                vtypes.split(',').map(str::trim).filter(|s| !s.is_empty())
            {
                let &target_sh = state
                    .vary_as_to_format_variant
                    .get(vtype)
                    .with_context(|| {
                        format!(
                            "Unknown format variant type '{vtype}' on {subj}; no .Formats:format_variant defines @vary_as(\"{vtype}\")"
                        )
                    })?;
                state.multi_properties.insert(
                    (
                        subj,
                        sh(DC_FORMAT_VARIATION_ASPECT),
                        ValueSpec::Direct(target_sh),
                    ),
                    format!("{subj} variant_type {vtype} ({target_sh})"),
                );
            }
        }

        // Default format variant (`Dc 633`): `@default_line_ending`
        if let Some(dle) = &fmt.default_line_ending {
            let dle_sh = match dle.as_str() {
                "lf" => sh(DC_LINE_ENDING_LF),
                "cr" => sh(DC_LINE_ENDING_CR),
                "crlf" => sh(DC_LINE_ENDING_CR_LF),
                "lfcr" => sh(DC_LINE_ENDING_LF_CR),
                "rs" => sh(DC_LINE_ENDING_RS),
                "nl" => sh(DC_LINE_ENDING_NL),
                other => {
                    bail!("Unknown @default_line_ending({other:?}) on {subj}")
                }
            };
            state.single_properties.insert(
                (subj, sh(DC_DEFAULT_FORMAT_VARIANT)),
                (
                    ValueSpec::Direct(dle_sh),
                    format!("{subj} default variant {dle_sh}"),
                ),
            );
        }
    }

    Ok(())
}

fn support_level_to_dc(level: i32) -> Option<DcChar> {
    match level {
        -1 => Some(DC_SUPPORT_NOT_APPLICABLE),
        1 => Some(DC_SUPPORT_WORK_IN_PROGRESS),
        2 => Some(DC_SUPPORT_PARTIAL),
        3 => Some(DC_SUPPORT_COMPLETE),
        4 => Some(DC_SUPPORT_LOSSLESS_ROUNDTRIP),
        5 => Some(DC_SUPPORT_LOSSLESS_STRICT_VALIDATION),
        _ => None,
    }
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
    update_ledger_in_memory_with_provenance(
        existing_nodes,
        NodeProvenanceMap::default(),
        dc_defns,
        format_defns,
        known_scripts,
        named_types_bytes,
    )
}

/// Updates an in-memory list of `LedgerNode`s against `dc_defns`, `format_defns`,
/// `known_scripts`, and `named_types_bytes`, enforcing `provenance_map` cross-ownership rules.
pub fn update_ledger_in_memory_with_provenance(
    existing_nodes: Vec<LedgerNode>,
    provenance_map: NodeProvenanceMap,
    dc_defns: &[DcDefn],
    format_defns: &[DcDefn],
    known_scripts: &BTreeSet<String>,
    named_types_bytes: Option<&[u8]>,
) -> Result<(Vec<LedgerNode>, LedgerUpdateStats)> {
    let single_props = single_valued_property_set();
    let initial_labels = build_initial_token_labels(dc_defns, format_defns);
    let mut builder = LedgerBuilder::from_existing_with_provenance(
        existing_nodes,
        &single_props,
        initial_labels,
        provenance_map,
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

fn plan_csv_reconciliation_items_for_repo(
    repo_root: &Path,
    existing_nodes: &[LedgerNode],
    committed_pairs: &[(
        crate::ctbnbdl::JournalRecord,
        crate::ctbnbdl::JournalRecord,
    )],
) -> Result<(
    Vec<PlannedLedgerItem>,
    BTreeMap<String, String>,
    Vec<LedgerNode>,
    LedgerUpdateStats,
)> {
    let prov_map = load_and_validate_node_provenance(
        repo_root,
        existing_nodes,
        committed_pairs,
    )?;

    let data_dir = repo_root.join("src/formats/dcdata/data-src");
    let categories_dir = data_dir.join("categories");
    let formats_dir = categories_dir.join("formats");
    let scripts_path = data_dir.join("README.scripts.csv");
    let named_types_path = data_dir.join("README-named-types.csv");

    let mut pinned_input_sha256s: BTreeMap<String, String> = BTreeMap::new();
    let mut report = ValidationReport::new();
    let scripts_bytes = std::fs::read(&scripts_path).with_context(|| {
        format!("Failed to read {}", scripts_path.display())
    })?;
    pinned_input_sha256s.insert(
        "src/formats/dcdata/data-src/README.scripts.csv".to_string(),
        sha256_bytes_hex(&scripts_bytes),
    );
    let known_scripts_set = crate::validation::validate_scripts_table(
        &scripts_bytes,
        "src/formats/dcdata/data-src/README.scripts.csv",
        &mut report,
    );
    let known_scripts: BTreeSet<String> =
        known_scripts_set.iter().cloned().collect();

    let named_types_bytes =
        std::fs::read(&named_types_path).with_context(|| {
            format!("Failed to read {}", named_types_path.display())
        })?;
    pinned_input_sha256s.insert(
        "src/formats/dcdata/data-src/README-named-types.csv".to_string(),
        sha256_bytes_hex(&named_types_bytes),
    );
    let known_named_types =
        crate::validation::extract_named_type_names(&named_types_bytes);

    let format_files_data =
        crate::updater::load_format_category_csvs_with_assigned_ids(
            &formats_dir,
        )?;
    for (name, bytes) in &format_files_data {
        pinned_input_sha256s.insert(
            format!("src/formats/dcdata/data-src/categories/formats/{name}"),
            sha256_bytes_hex(bytes),
        );
    }
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
    for (name, bytes) in &dc_files_data {
        pinned_input_sha256s.insert(
            format!("src/formats/dcdata/data-src/categories/{name}"),
            sha256_bytes_hex(bytes),
        );
    }
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

    let (updated_nodes, stats) = update_ledger_in_memory_with_provenance(
        existing_nodes.to_vec(),
        prov_map,
        &dc_defns,
        &format_defns,
        &known_scripts,
        Some(&named_types_bytes),
    )?;
    verify_nodes_append_only_against_baseline(
        "existing working-tree ledger",
        existing_nodes,
        &updated_nodes,
    )?;

    // Reason for fallback: verify_nodes_append_only_against_baseline guarantees updated_nodes.len() >= existing_nodes.len().
    let new_slice = updated_nodes
        .get(existing_nodes.len()..)
        .unwrap_or_default();
    let mut planned_items = Vec::with_capacity(new_slice.len());
    for node in new_slice {
        planned_items.push(PlannedLedgerItem {
            node_id: node.id,
            raw_line: node.format_line(),
            target_chunk: chunk_filename_for_node_id(node.id),
            tokens: Vec::new(),
            comment: String::new(),
            provenance: "csv".to_string(),
            rationale: None,
            allow_duplicate_of: Vec::new(),
            inbox_name: None,
            allocated_name: None,
            inbox_sha256: None,
            dcsh_content: None,
        });
    }

    Ok((planned_items, pinned_input_sha256s, updated_nodes, stats))
}

/// Plans a `csv_reconciliation` transaction manifest against `repo_root` under `LedgerReadGuard`
/// and writes the resulting [`TransactionManifest`] JSON to `out_path`.
pub fn plan_csv_reconciliation_manifest(
    repo_root: &Path,
    manifest_id: &str,
    out_path: &Path,
) -> Result<TransactionManifest> {
    let read_guard = LedgerReadGuard::acquire(repo_root)?;
    ensure_ledger_journal_clean(repo_root, &read_guard)?;
    let graph_dir = repo_root.join("assets/data/graph");
    let journal_path = repo_root.join(LEDGER_JOURNAL_REL_PATH);
    let existing_nodes = load_ledger_nodes_from_dir(&graph_dir)?;
    let parsed_journal = inspect_ledger_journal(&journal_path)?;

    let (planned_items, pinned_input_sha256s, updated_nodes, _stats) =
        plan_csv_reconciliation_items_for_repo(
            repo_root,
            &existing_nodes,
            &parsed_journal.committed_pairs,
        )?;

    // Reason for fallback: empty ledger before first node has baseline_last_id == 0.
    let baseline_last_id = existing_nodes.last().map_or(0, |n| n.id);
    let baseline_node_count = existing_nodes.len();
    let baseline_ledger_sha256 =
        compute_nodes_slice_ledger_sha256(&existing_nodes);
    // Reason for fallback: empty ledger after reconciliation has final_last_id == baseline_last_id.
    let final_last_id = updated_nodes.last().map_or(baseline_last_id, |n| n.id);
    let final_node_count = updated_nodes.len();
    let final_ledger_sha256 = compute_nodes_slice_ledger_sha256(&updated_nodes);

    let manifest = TransactionManifest {
        manifest_id: manifest_id.to_string(),
        kind: "csv_reconciliation".to_string(),
        baseline_last_id,
        baseline_node_count,
        baseline_ledger_sha256,
        final_last_id,
        final_node_count,
        final_ledger_sha256,
        pinned_input_sha256s,
        planned_items,
    };

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create directory {}", parent.display())
        })?;
    }
    let json = serde_json::to_string_pretty(&manifest)
        .context("Failed to serialize reconciliation manifest JSON")?;
    std::fs::write(out_path, format!("{json}\n")).with_context(|| {
        format!("Failed to write manifest to {}", out_path.display())
    })?;

    Ok(manifest)
}

/// Loads the CSV dataset from `repo_root` and updates `assets/data/graph/*.ctbnodes`
/// using the append-only revision ledger algorithm inside a durable journaled transaction.
pub fn update_system_graph_ledger(repo_root: &Path) -> Result<LedgerUpdateStats> {
    update_system_graph_ledger_with_options(repo_root, None, false, None)
}

/// Same as [`update_system_graph_ledger`], with optional `required_manifest_path`,
/// `dry_run`, and [`FaultInjectionPoint`].
pub fn update_system_graph_ledger_with_options(
    repo_root: &Path,
    required_manifest_path: Option<&Path>,
    dry_run: bool,
    fault: Option<FaultInjectionPoint>,
) -> Result<LedgerUpdateStats> {
    let outcome = execute_durable_ledger_transaction_with_planner(
        repo_root,
        "csv_reconciliation",
        "csv-reconcile",
        required_manifest_path,
        dry_run,
        fault,
        |_write_guard, baseline_nodes, committed_pairs| {
            let (planned_items, pinned_input_sha256s, _updated_nodes, stats) =
                plan_csv_reconciliation_items_for_repo(
                    repo_root,
                    baseline_nodes,
                    committed_pairs,
                )?;
            Ok((planned_items, pinned_input_sha256s, stats))
        },
    )?;

    if !dry_run {
        let _ = verify_repo_committed_node_provenance(repo_root)?;
    }

    // Reason for fallback: when resuming from a pre-existing BATCH_PENDING record, planner closure is skipped and stats are populated from outcome counts.
    let mut stats = outcome.planner_payload.unwrap_or_else(|| LedgerUpdateStats {
        total_nodes: outcome.total_nodes,
        nodes_appended: outcome.appended_count,
        nodes_superseded: 0,
        nodes_retracted: 0,
        files_written: outcome.files_written,
    });
    stats.total_nodes = outcome.total_nodes;
    stats.nodes_appended = outcome.appended_count;
    stats.files_written = outcome.files_written;
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
    fn test_compile_ctbnodes_dir_to_ctbnbdl_and_framing_resolution() -> Result<()> {
        let repo_root = crate::updater::find_repository_root()?;
        let graph_dir = repo_root.join("assets/data/graph");
        let resolved_framing =
            ctb_build_support::dc_codegen::resolve_ctbnbdl_framing_dcs(
                &graph_dir,
            )?;
        let def_framing = default_ctbnbdl_framing_dcs();
        assert_eq!(
            resolved_framing.begin_node.to_global_id()?,
            def_framing.begin_node.to_global_id()?
        );
        assert_eq!(
            resolved_framing.end_node.to_global_id()?,
            def_framing.end_node.to_global_id()?
        );
        assert_eq!(
            resolved_framing.escape.to_global_id()?,
            def_framing.escape.to_global_id()?
        );
        assert_eq!(
            resolved_framing.evaluate_as_atom.to_global_id()?,
            def_framing.evaluate_as_atom.to_global_id()?
        );
        assert_eq!(
            resolved_framing.has_a.to_global_id()?,
            def_framing.has_a.to_global_id()?
        );
        assert_eq!(
            resolved_framing.has_value.to_global_id()?,
            def_framing.has_value.to_global_id()?
        );
        assert_eq!(
            resolved_framing.preferred.to_global_id()?,
            def_framing.preferred.to_global_id()?
        );
        assert_eq!(
            resolved_framing.supersedes.to_global_id()?,
            def_framing.supersedes.to_global_id()?
        );
        assert_eq!(
            resolved_framing.retracts.to_global_id()?,
            def_framing.retracts.to_global_id()?
        );
        assert_eq!(
            resolved_framing.enclosing_document.to_global_id()?,
            def_framing.enclosing_document.to_global_id()?
        );
        assert_eq!(
            resolved_framing.is_instance_of.to_global_id()?,
            def_framing.is_instance_of.to_global_id()?
        );
        assert_eq!(
            resolved_framing.is_subtype_of.to_global_id()?,
            def_framing.is_subtype_of.to_global_id()?
        );
        assert_eq!(
            resolved_framing.is_based_on.to_global_id()?,
            def_framing.is_based_on.to_global_id()?
        );
        assert_eq!(
            resolved_framing.is_deprecated.to_global_id()?,
            def_framing.is_deprecated.to_global_id()?
        );
        assert_eq!(
            resolved_framing.is_closing_delimiter_for.to_global_id()?,
            def_framing.is_closing_delimiter_for.to_global_id()?
        );
        assert_eq!(
            resolved_framing.is_associated_with.to_global_id()?,
            def_framing.is_associated_with.to_global_id()?
        );

        let tmp = tempfile::tempdir()?;
        let src_dir = tmp.path().join("docs");
        let dst_dir = tmp.path().join("built_docs");
        std::fs::create_dir_all(&src_dir)?;
        std::fs::create_dir_all(&dst_dir)?;

        let id0 = SYSTEM_REGION_START;
        let id1 = SYSTEM_REGION_START + 1;
        let id2 = SYSTEM_REGION_START + 2;
        let id3 = SYSTEM_REGION_START + 3;
        let id4 = SYSTEM_REGION_START + 4;
        let id5 = SYSTEM_REGION_START + 5;
        let id6 = SYSTEM_REGION_START + 6;
        let nodes = vec![
            LedgerNode {
                id: id0,
                tokens: vec![
                    sh(DC_IS_INSTANCE_OF),
                    sh(DC_ENCLOSING_DOCUMENT),
                    sh(DC_BIDIRECTIONAL_CLASS),
                ],
                comment: "Seed entity".to_string(),
            },
            LedgerNode {
                id: id1,
                tokens: encode_string_value_tokens("OldL"),
                comment: "\"OldL\"".to_string(),
            },
            LedgerNode {
                id: id2,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_A),
                    DcShorthand::Long(id0),
                    sh(DC_PROGRAMMATIC_IDENTIFIER),
                ),
                comment: "Has a".to_string(),
            },
            LedgerNode {
                id: id3,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(id2),
                    DcShorthand::Long(id1),
                ),
                comment: "Old Has value".to_string(),
            },
            LedgerNode {
                id: id4,
                tokens: encode_string_value_tokens("L"),
                comment: "\"L\"".to_string(),
            },
            LedgerNode {
                id: id5,
                tokens: encode_statement_tokens(
                    sh(DC_HAS_VALUE),
                    DcShorthand::Long(id2),
                    DcShorthand::Long(id4),
                ),
                comment: "New Has value".to_string(),
            },
            LedgerNode {
                id: id6,
                tokens: encode_statement_tokens(
                    sh(DC_SUPERSEDES),
                    DcShorthand::Long(id5),
                    DcShorthand::Long(id3),
                ),
                comment: "Supersedes".to_string(),
            },
        ];
        write_ledger_nodes_to_dir(&src_dir, &nodes)?;

        let (expected_active, expected_inactive) =
            partition_active_and_inactive_nodes(&nodes);
        assert_eq!(
            expected_active.iter().map(|n| n.id).collect::<Vec<_>>(),
            vec![id0, id2, id4, id5]
        );
        assert_eq!(
            expected_inactive.iter().map(|n| n.id).collect::<Vec<_>>(),
            vec![id1, id3, id6]
        );

        // Seed a stale `.ctbnodes` file in `dst_dir` to ensure it is removed during compilation.
        let stale_ctbnodes =
            dst_dir.join(format_chunk_filename(SYSTEM_REGION_START));
        std::fs::write(&stale_ctbnodes, "stale")?;

        let compiled_count = compile_ctbnodes_dir_to_ctbnbdl(
            &src_dir,
            &dst_dir,
            default_ctbnbdl_framing_dcs(),
        )?;
        assert_eq!(compiled_count, 1);
        assert!(
            !stale_ctbnodes.exists(),
            "Expected stale .ctbnodes in destination to be removed"
        );

        let bundle_path =
            dst_dir.join(format_chunk_bundle_filename(SYSTEM_REGION_START));
        assert!(bundle_path.is_file());
        let bundle_bytes = std::fs::read(&bundle_path)?;
        let decoded = decode_nodes_from_ctbnbdl_bytes(&bundle_bytes)?;
        assert_eq!(decoded.len(), expected_active.len());
        for (orig, dec) in expected_active.iter().zip(decoded.iter()) {
            assert_eq!(orig.id, dec.id);
            assert_eq!(orig.tokens, dec.tokens);
        }

        // Calling a second time when up-to-date should compile 0 files.
        let compiled_again = compile_ctbnodes_dir_to_ctbnbdl(
            &src_dir,
            &dst_dir,
            default_ctbnbdl_framing_dcs(),
        )?;
        assert_eq!(compiled_again, 0);

        // Compiling in-place (src_dir == dst_dir) should produce the active .ctbnbdl file
        // in `src_dir` and the inactive .ctbnbdl file in `src_dir/system/` without deleting
        // the source .ctbnodes files.
        let src_chunk_file = src_dir.join(format_chunk_filename(SYSTEM_REGION_START));
        assert!(src_chunk_file.is_file());
        let in_place_compiled = compile_ctbnodes_dir_to_ctbnbdl(
            &src_dir,
            &src_dir,
            default_ctbnbdl_framing_dcs(),
        )?;
        assert_eq!(in_place_compiled, 1);
        assert!(
            src_chunk_file.is_file(),
            "Source .ctbnodes file must be preserved when compiling in-place"
        );
        let in_place_bundle = src_dir.join(format_chunk_bundle_filename(SYSTEM_REGION_START));
        assert!(in_place_bundle.is_file());
        assert_eq!(std::fs::read(&in_place_bundle)?, bundle_bytes);

        let history_bundle = src_dir
            .join("system")
            .join(format_chunk_bundle_filename(id1));
        assert!(
            history_bundle.is_file(),
            "Expected inactive history bundle at {}",
            history_bundle.display()
        );
        let decoded_history = decode_nodes_from_ctbnbdl_bytes(&std::fs::read(&history_bundle)?)?;
        assert_eq!(decoded_history.len(), expected_inactive.len());
        for (orig, dec) in expected_inactive.iter().zip(decoded_history.iter()) {
            assert_eq!(orig.id, dec.id);
            assert_eq!(orig.tokens, dec.tokens);
        }

        // In-place second call should compile 0 files.
        let in_place_again = compile_ctbnodes_dir_to_ctbnbdl(
            &src_dir,
            &src_dir,
            default_ctbnbdl_framing_dcs(),
        )?;
        assert_eq!(in_place_again, 0);
        assert!(src_chunk_file.is_file());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_ctbnbdl_bundle_grouping_and_size_split_alignment() -> Result<()> {
        let framing = default_ctbnbdl_framing_dcs();
        // Create 250 nodes (spanning 3 100-node `.ctbnodes` files).
        let nodes: Vec<LedgerNode> = (0..250u128)
            .map(|offset| LedgerNode {
                id: SYSTEM_REGION_START + offset,
                tokens: encode_string_value_tokens("node_payload_text"),
                comment: String::new(),
            })
            .collect();

        // 1. With default limits (1,000,000 nodes / 50 MB), all 250 nodes group into a single bundle.
        let default_bundles = encode_nodes_to_ctbnbdl_bundles_with_limits(
            &nodes,
            framing,
            MAX_NODES_PER_BUNDLE,
            MAX_BYTES_PER_BUNDLE,
        )?;
        assert_eq!(default_bundles.len(), 1);
        assert_eq!(default_bundles[0].0, SYSTEM_REGION_START);
        assert_eq!(
            decode_nodes_from_ctbnbdl_bytes(&default_bundles[0].1)?.len(),
            250
        );

        // 2. With `max_nodes_per_bundle = 100`, if the first bucket (0..100) has to split on size
        // after 60 nodes, the second bundle must contain only the remaining 40 nodes (60..100)
        // so that the third bundle lines back up at node 100 (`SYSTEM_REGION_START + 100`).
        let single_node_bundle = encode_nodes_to_ctbnbdl_bytes(&nodes[..1])?;
        let header_len = CTBNBDL_MAGIC.len() + DCMM_HEADER.len();
        let bytes_per_node = single_node_bundle.len() - header_len;
        let max_bytes_for_60_nodes = header_len + bytes_per_node * 60;

        let split_bundles = encode_nodes_to_ctbnbdl_bundles_with_limits(
            &nodes,
            framing,
            100,
            max_bytes_for_60_nodes,
        )?;
        let start_ids: Vec<u128> =
            split_bundles.iter().map(|(id, _)| *id).collect();
        assert_eq!(
            start_ids,
            vec![
                SYSTEM_REGION_START,       // bucket 0, part 1 (60 nodes: 0..60)
                SYSTEM_REGION_START + 60,  // bucket 0, part 2 (shortened to 40 nodes: 60..100)
                SYSTEM_REGION_START + 100, // bucket 1, part 1 (realigned at 100: 100..160)
                SYSTEM_REGION_START + 160, // bucket 1, part 2 (shortened to 40 nodes: 160..200)
                SYSTEM_REGION_START + 200, // bucket 2 (realigned at 200: 200..250)
            ]
        );
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
    fn test_format_tokens_comment_unicode_display() -> Result<()> {
        let single_props = single_valued_property_set();
        let mut labels = HashMap::new();
        labels.insert(sh(DC_WILDCARD_PATTERN), "Wildcard pattern".to_string());
        labels.insert(sh(DC_BEGIN_ASSOCIATIVITY_GROUP), "Begin associativity group".to_string());
        labels.insert(sh(DC_PATTERN_REPETITION), "Pattern repetition".to_string());
        labels.insert(sh(DC_RANGE), "Range".to_string());
        labels.insert(sh(DC_END_ASSOCIATIVITY_GROUP), "End associativity group".to_string());
        labels.insert(sh(DC_HAS_VALUE), "Has value".to_string());
        labels.insert(DcShorthand::Long(8589934942), "0".to_string());

        let builder =
            LedgerBuilder::from_existing(Vec::new(), &single_props, labels.clone());

        // 1. Literal string in a pattern rule
        let pattern_tokens = vec![
            sh(DC_WILDCARD_PATTERN),
            sh(DC_BEGIN_ASSOCIATIVITY_GROUP),
            sh(DC_PATTERN_REPETITION),
            sh(DC_RANGE),
            DcShorthand::Long(8589934942),
            sh(DC_END_ASSOCIATIVITY_GROUP),
            DcShorthand::Unicode(0x5f), // _
            DcShorthand::Unicode(0x61), // a
            DcShorthand::Unicode(0x72), // r
            DcShorthand::Unicode(0x63), // c
            DcShorthand::Unicode(0x68), // h
            DcShorthand::Unicode(0x69), // i
            DcShorthand::Unicode(0x76), // v
            DcShorthand::Unicode(0x65), // e
            DcShorthand::Unicode(0x2e), // .
            DcShorthand::Unicode(0x74), // t
            DcShorthand::Unicode(0x6f), // o
            DcShorthand::Unicode(0x72), // r
            DcShorthand::Unicode(0x72), // r
            DcShorthand::Unicode(0x65), // e
            DcShorthand::Unicode(0x6e), // n
            DcShorthand::Unicode(0x74), // t
        ];
        let comment = builder.format_tokens_comment(&pattern_tokens);
        assert_eq!(
            comment,
            "Wildcard pattern, Begin associativity group, Pattern repetition, Range, 0, End associativity group, \"_archive.torrent\""
        );

        // 2. Range endpoints with printable ASCII characters must remain individual character names
        let range_tokens = vec![
            sh(DC_RANGE),
            DcShorthand::Unicode(0x41), // A
            DcShorthand::Unicode(0x5a), // Z
        ];
        let range_comment = builder.format_tokens_comment(&range_tokens);
        assert_eq!(
            range_comment,
            "Range, LATIN CAPITAL LETTER A, LATIN CAPITAL LETTER Z"
        );

        // 3. Isolated statement object with Unicode character name
        let stmt_tokens = encode_statement_tokens(
            sh(DC_HAS_VALUE),
            DcShorthand::Long(8589945042),
            DcShorthand::Unicode(0x25), // %
        );
        let stmt_comment = builder.format_tokens_comment(&stmt_tokens);
        assert_eq!(stmt_comment, "Has value (l8589945042, PERCENT SIGN)");

        // 4. Nested statement reference in statement subject
        labels.insert(
            DcShorthand::Long(8589937467),
            "Has a (At, Canonical name)".to_string(),
        );
        let nested_builder =
            LedgerBuilder::from_existing(Vec::new(), &single_props, labels);
        let nested_tokens = encode_statement_tokens(
            sh(DC_HAS_VALUE),
            DcShorthand::Long(8589937467),
            DcShorthand::Unicode(0x25),
        );
        let nested_comment = nested_builder.format_tokens_comment(&nested_tokens);
        assert_eq!(
            nested_comment,
            "Has value (Has a (At, Canonical name), PERCENT SIGN)"
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

    #[crate::ctb_test]
    fn test_arbitrary_base_n_format_param_application_and_dc9() -> Result<()> {
        use crate::dc::DC_FORMAT_PARAM_APPLICATION;
        use crate::dc_def::FormatDetails;
        use crate::syntax::{DatasetRuleResolver, SyntaxRuleResolver};

        // 1. Test arbitrary Natural1 base (e.g. radix=123) in FormatExpr::Apply
        let mut fv_base = sample_dc_defn(354, "A radix in a number system", "Base");
        fv_base.dc_id = 2_228_578;
        fv_base.script = ".Formats:format_variant".to_string();
        fv_base.format = Some(FormatDetails {
            vary_as: Some("radix".to_string()),
            ..FormatDetails::default()
        });

        let mut base123 = sample_dc_defn(999, "Base 123 numeral", "Base123");
        base123.dc_id = 2_229_223;
        base123.script = ".Formats:number".to_string();
        base123.decompositions = vec!["<equiv>f350(radix=123)".to_string()];
        base123.format = Some(FormatDetails::default());

        let scripts = BTreeSet::from([
            ".Formats:format_variant".to_string(),
            ".Formats:number".to_string(),
        ]);
        let (nodes, _) = update_ledger_in_memory(
            Vec::new(),
            &[],
            &[fv_base, base123],
            &scripts,
            None,
        )?;
        let index = crate::seed_graph::SeedGraphIndex::from_nodes(nodes);

        // Verify integer value node 123 was created and referenced inside a `535 f350 258 f354 l<int_id> 259` node
        let expected_int_tokens = encode_integer_value_tokens(123);
        let int_node = index
            .nodes()
            .iter()
            .find(|n| n.tokens == expected_int_tokens)
            .expect("Expected integer value node for 123");
        assert_eq!(index.get_integer(DcShorthand::Long(int_node.id)), Some(123));

        let apply_node = index
            .nodes()
            .iter()
            .find(|n| n.tokens.first() == Some(&sh(DC_FORMAT_PARAM_APPLICATION)))
            .expect("Expected Dc 535 FormatParamApplication value node");
        assert_eq!(
            apply_node.tokens,
            vec![
                sh(DC_FORMAT_PARAM_APPLICATION),
                DcShorthand::Format(350),
                sh(DC_BEGIN_PARAMETER),
                DcShorthand::Format(354),
                DcShorthand::Long(int_node.id),
                sh(crate::dc::DC_END_PARAMETER),
            ]
        );

        let reconstructed = index.reconstruct_format_defns()?;
        let rec_base123 = reconstructed
            .iter()
            .find(|d| d.short_id == Some(999))
            .expect("Reconstructed Base123 format");
        assert_eq!(
            rec_base123.decompositions,
            vec!["<equiv>f350(radix=123)".to_string()]
        );

        // 2. Verify live SeedGraphIndex only uses `.Formats:format_variant` Dcs as targets of Dc 632,
        // and Dc 9 ("Gregorian calendar identifier") is recognized as a format in `.Formats:calendar`.
        let shared = crate::seed_graph::SeedGraphIndex::shared()?;
        for (&(_, prop, val_tok), _) in shared.active_multi_props() {
            if prop == sh(DC_FORMAT_VARIATION_ASPECT) {
                assert!(
                    matches!(val_tok, DcShorthand::Format(_)),
                    "Dc 632 (Format variation aspect) must only take a format_variant Dc, found {val_tok}"
                );
            }
        }

        let resolver = DatasetRuleResolver::load_from_seed_graph()?;
        let dc9_gid = crate::dc::short_to_long_dc(9);
        assert!(resolver.token_satisfies_type(dc9_gid, "format", None));
        assert!(resolver.token_satisfies_type(dc9_gid, "format", Some("calendar")));
        assert!(resolver.token_satisfies_type(dc9_gid, "type", None));
        assert!(resolver.token_satisfies_type(dc9_gid, "type", Some("calendar")));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_append_only_chunk_writer_and_baseline_guards() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let start_id = SYSTEM_REGION_START;
        let base_node_1 = LedgerNode {
            id: start_id,
            tokens: vec![DcShorthand::Short(582), DcShorthand::Short(257), DcShorthand::Short(590)],
            comment: "Is instance of (Enclosing document, Graph statement)".to_string(),
        };
        let base_node_2 = LedgerNode {
            id: start_id.saturating_add(1),
            tokens: vec![DcShorthand::Short(582), DcShorthand::Short(257), DcShorthand::Short(650)],
            comment: "Is instance of (Enclosing document, Document format)".to_string(),
        };

        // Initial write creates the first chunk file via O_CREAT | O_EXCL.
        let written_1 = write_ledger_nodes_to_dir(
            dir.path(),
            std::slice::from_ref(&base_node_1),
        )?;
        assert_eq!(written_1, 1);

        // Appending a second node writes only the delta suffix via O_APPEND.
        let both_nodes = vec![base_node_1.clone(), base_node_2.clone()];
        let written_2 = write_ledger_nodes_to_dir(dir.path(), &both_nodes)?;
        assert_eq!(written_2, 1);

        // Re-writing identical nodes is a no-op.
        let written_noop = write_ledger_nodes_to_dir(dir.path(), &both_nodes)?;
        assert_eq!(written_noop, 0);

        // Attempting to truncate back to 1 node must fail cleanly.
        let rollback_err = write_ledger_nodes_to_dir(
            dir.path(),
            std::slice::from_ref(&base_node_1),
        );
        assert!(
            rollback_err.is_err(),
            "Expected write_ledger_nodes_to_dir to reject rollback/truncation"
        );

        // Attempting to alter tokens or comments of an already-written node must fail.
        let mut mutated_node_1 = base_node_1.clone();
        mutated_node_1.tokens = vec![DcShorthand::Short(582), DcShorthand::Short(257), DcShorthand::Short(1)];
        let mutate_err = write_ledger_nodes_to_dir(
            dir.path(),
            &[mutated_node_1.clone(), base_node_2.clone()],
        );
        assert!(
            mutate_err.is_err(),
            "Expected write_ledger_nodes_to_dir to reject mutation of existing node"
        );

        // Baseline verification passes when current is a superset of baseline, and fails on rollback or token change.
        crate::ctbnbdl::verify_nodes_append_only_against_baseline(
            "test-baseline",
            std::slice::from_ref(&base_node_1),
            &both_nodes,
        )?;
        assert!(
            crate::ctbnbdl::verify_nodes_append_only_against_baseline(
                "test-baseline",
                &both_nodes,
                std::slice::from_ref(&base_node_1),
            )
            .is_err()
        );
        assert!(
            crate::ctbnbdl::verify_nodes_append_only_against_baseline(
                "test-baseline",
                &both_nodes,
                &[mutated_node_1, base_node_2],
            )
            .is_err()
        );

        Ok(())
    }
}
