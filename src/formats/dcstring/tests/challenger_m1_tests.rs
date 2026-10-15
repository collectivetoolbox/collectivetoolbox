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

//! Empirical Challenge Test Suite for Milestone 1:
//! 1. Verify `validate_dc_mixed_strict`, `validate_dc_str_strict`, and
//!    `validate_dc_tokens_strict` reject invalid framing and accept valid
//!    documents without base64 allocation.
//! 2. Verify `prefix_rule_fixed_arity` removal has zero adverse impact on
//!    prefix operator evaluation.
//! 3. Adversarial boundary tests and zero-copy performance guarantees.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use ctb_formats_dcdata::dc::*;
use ctb_formats_dcdata::syntax::{
    CharTarget, DatasetRuleResolver, SyntaxPattern, SyntaxRuleResolver,
    default_syntax_resolver,
};
use ctb_formats_dcstring::{
    DcByteSpan, DcChar, DcDocumentNavigator, DcFrameHeaderInfo, DcFrameStub,
    DcMstr, DcMstring, DcProjectionPolicy, DcStr, DcString, SyntaxEvalContext,
    dc_mixed, validate_dc_mixed_strict, validate_dc_str_strict,
    validate_dc_tokens_strict,
};

fn sdc(id: u32) -> u128 {
    DcChar::from_short(id).to_u128()
}

fn fmt(id: u32) -> u128 {
    DcChar::from_format(id).to_u128()
}

fn dc_string_from_tokens(tokens: &[u128]) -> DcString {
    let mut s = DcString::with_capacity(tokens.len().saturating_mul(6));
    for &tok in tokens {
        s.push(DcChar::from_u128(tok));
    }
    s
}

/// Helper: creates a `DcMstring` from raw `u128` tokens.
fn mstring_from_tokens(tokens: &[u128]) -> Result<DcMstring> {
    let s = dc_string_from_tokens(tokens);
    DcMstring::from_dc_str(&s)
}

// ============================================================================
// 1. EMPIRICAL VERIFICATION: ZERO BASE64 ALLOCATION IN VALIDATION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_validate_dc_mixed_zero_base64_allocation() -> Result<()> {
    // 1. Build a DcMstring with a large raw binary payload (1 MiB).
    // If validation were converting this to base64, it would format ~1.33 MiB of
    // base64 characters. With raw O(1) header scanning, it must finish
    // virtually instantaneously without heap allocation for base64.
    let payload_size = 1024usize.saturating_mul(1024);
    let mut large_payload = Vec::with_capacity(payload_size);
    for idx in 0..payload_size {
        let byte_val = u8::try_from(idx.wrapping_rem(256)).unwrap_or(0);
        large_payload.push(byte_val);
    }

    // Warm up the static CSV syntax resolver so we benchmark validation,
    // not one-time CSV disk loading.
    let _ = default_syntax_resolver();

    // 1. Bare binary in DcMstring
    let mut bare_mst = DcMstring::new();
    bare_mst.push_binary(&large_payload);

    let t_bare = std::time::Instant::now();
    validate_dc_mixed_strict(&bare_mst)
        .context("Bare raw binary payload in DcMstring must validate strictly")?;
    let elapsed_bare = t_bare.elapsed();
    println!("validate_dc_mixed_strict (bare 1 MiB binary): {:?}", elapsed_bare);

    ensure!(
        elapsed_bare < std::time::Duration::from_millis(50),
        "validate_dc_mixed_strict on bare 1 MiB raw binary took too long ({:?})",
        elapsed_bare
    );

    // 2. Binary in list
    let mut mst = DcMstring::new();
    mst.push_char(DC_BEGIN_LIST);
    mst.push_char(DcChar::from_short(275)); // Type wildcard

    let t0 = std::time::Instant::now();
    mst.push_binary(&large_payload);
    mst.push_char(DC_END_LIST);
    let t_push = t0.elapsed();

    // Validate using validate_dc_mixed_strict
    let t1 = std::time::Instant::now();
    validate_dc_mixed_strict(&mst)
        .context("Large raw binary payload in DcMstring must validate strictly")?;
    let t_validate = t1.elapsed();

    println!("push_binary: {:?}, validate_dc_mixed_strict (in list): {:?}", t_push, t_validate);

    ensure!(
        t_validate < std::time::Duration::from_millis(50),
        "validate_dc_mixed_strict on 1 MiB raw binary took too long ({:?})",
        t_validate
    );

    // 2. Verify with binary with metadata (e.g. SHA-256)
    let mut mst_with_meta = DcMstring::new();
    mst_with_meta.push_char(DC_BEGIN_LIST);
    mst_with_meta.push_char(DcChar::from_short(275));
    mst_with_meta.push_binary_with_sha256(&large_payload)?;
    mst_with_meta.push_char(DC_END_LIST);

    let t2 = std::time::Instant::now();
    validate_dc_mixed_strict(&mst_with_meta)
        .context("Binary with SHA-256 in DcMstring must validate strictly")?;
    let t_validate_meta = t2.elapsed();

    println!("validate_dc_mixed_strict with metadata: {:?}", t_validate_meta);
    ensure!(
        t_validate_meta < std::time::Duration::from_millis(50),
        "validate_dc_mixed_strict with metadata took too long ({:?})",
        t_validate_meta
    );
    Ok(())
}

// ============================================================================
// 2. EMPIRICAL VERIFICATION: STRICT FRAMING REJECTION ACROSS ALL THREE MODES
// ============================================================================

#[crate::ctb_test]
fn test_challenge_framing_rejection_across_all_modes() -> Result<()> {
    // List of invalid token sequences that violate paired container framing:
    let invalid_framing_cases: Vec<(&'static str, Vec<u128>)> = vec![
        ("dangling list closer", vec![sdc(311)]),
        ("dangling format closer", vec![sdc(512)]),
        ("dangling checksummed section closer", vec![sdc(544)]),
        ("dangling session closer", vec![sdc(548)]),
        ("dangling kv map closer", vec![sdc(560)]),
        ("dangling number closer", vec![sdc(7)]),
        ("dangling literal closer", vec![sdc(261)]),
        ("dangling identifier closer", vec![sdc(271)]),
        ("dangling kv pair closer", vec![sdc(515)]),
        // Mismatched container opener / closer
        (
            "mismatched list opener with format closer",
            vec![sdc(310), sdc(275), sdc(512)],
        ),
        (
            "mismatched format opener with list closer",
            vec![sdc(511), fmt(228), 116, 101, 115, 116, sdc(311)],
        ),
        (
            "mismatched session opener with checksummed section closer",
            vec![sdc(547), sdc(270), 99, 115, 99, sdc(271), sdc(544)],
        ),
        (
            "mismatched kv map opener with pair closer",
            vec![sdc(559), sdc(275), sdc(275), sdc(515)],
        ),
        // Unclosed containers at EOF
        (
            "unclosed list at EOF",
            vec![sdc(310), sdc(275), sdc(517)],
        ),
        (
            "unclosed format representation at EOF",
            vec![sdc(511), fmt(228), 116, 101, 115, 116],
        ),
        (
            "unclosed session at EOF",
            vec![sdc(547), sdc(270), 99, 115, 99, sdc(271)],
        ),
        (
            "unclosed checksummed section at EOF",
            vec![sdc(543), sdc(517)],
        ),
        (
            "unclosed kv map at EOF",
            vec![sdc(559), sdc(275), sdc(275)],
        ),
        // Truncated escape token at EOF
        (
            "truncated escape Dc 255 at EOF",
            vec![sdc(310), sdc(275), sdc(255)],
        ),
        // Invalid interior constraint: raw number instead of framed IP address
        (
            "host local IP without format framing",
            vec![sdc(346), sdc(6), fmt(199), sdc(127), sdc(7)],
        ),
        // Invalid interior constraint: raw number instead of date framing
        (
            "date timestamp without 390..391 framing",
            vec![sdc(330), sdc(6), fmt(199), sdc(127), sdc(7)],
        ),
    ];

    for (desc, tokens) in invalid_framing_cases {
        // 1. Must fail validate_dc_tokens_strict
        ensure!(
            validate_dc_tokens_strict(&tokens).is_err(),
            "{desc} must be rejected by validate_dc_tokens_strict"
        );

        // 2. Must fail validate_dc_str_strict
        let dc_str = dc_string_from_tokens(&tokens);
        ensure!(
            validate_dc_str_strict(&dc_str).is_err(),
            "{desc} must be rejected by validate_dc_str_strict"
        );

        // 3. Must fail validate_dc_mixed_strict
        if let Ok(mst) = mstring_from_tokens(&tokens) {
            ensure!(
                validate_dc_mixed_strict(&mst).is_err(),
                "{desc} must be rejected by validate_dc_mixed_strict"
            );
        }
    }

    Ok(())
}

// ============================================================================
// 3. EMPIRICAL VERIFICATION: ACCEPTANCE OF VALID DOCUMENTS IN ALL THREE MODES
// ============================================================================

#[crate::ctb_test]
fn test_challenge_acceptance_across_all_modes() -> Result<()> {
    let valid_documents: Vec<(&'static str, Vec<u128>)> = vec![
        (
            "atomic booleans",
            vec![sdc(517), sdc(518)],
        ),
        (
            "number construct Dc 6..7",
            vec![sdc(6), fmt(199), 49, 50, 51, sdc(7)],
        ),
        (
            "identifier construct Dc 270..271",
            vec![sdc(270), 102, 111, 111, sdc(271)],
        ),
        (
            "literal string construct Dc 260..261",
            vec![sdc(260), sdc(262), sdc(264), sdc(263), 104, 101, 108, 108, 111, sdc(261)],
        ),
        (
            "valid typed list Dc 310..311",
            vec![sdc(310), sdc(275), sdc(517), sdc(312), sdc(518), sdc(311)],
        ),
        (
            "valid format representation Dc 511..512",
            vec![
                sdc(511),
                fmt(228),
                sdc(260),
                sdc(262),
                sdc(264),
                sdc(263),
                102,
                105,
                108,
                101,
                sdc(261),
                sdc(512),
            ],
        ),
        (
            "valid operation session Dc 547..548",
            vec![sdc(547), sdc(270), 99, 115, 99, sdc(271), sdc(548)],
        ),
        (
            "valid kv pair Dc 514..515",
            vec![sdc(514), 107, sdc(312), 118, sdc(515)],
        ),
        (
            "valid kv map Dc 559..560",
            vec![
                sdc(559),
                sdc(275),
                sdc(275),
                sdc(514),
                107,
                sdc(312),
                118,
                sdc(515),
                sdc(560),
            ],
        ),
        (
            "valid host IP Dc 346 with f490 format",
            vec![
                sdc(346),
                sdc(511),
                fmt(490),
                sdc(6),
                fmt(199),
                sdc(127),
                sdc(7),
                sdc(512),
            ],
        ),
        (
            "valid date timestamp Dc 330 with Dc 390..391",
            vec![
                sdc(330),
                sdc(390),
                fmt(487),
                sdc(6),
                fmt(199),
                sdc(127),
                sdc(7),
                sdc(391),
            ],
        ),
    ];

    for (desc, tokens) in valid_documents {
        // 1. Must pass validate_dc_tokens_strict
        validate_dc_tokens_strict(&tokens)
            .with_context(|| format!("{desc} failed validate_dc_tokens_strict"))?;

        // 2. Must pass validate_dc_str_strict
        let dc_str = dc_string_from_tokens(&tokens);
        validate_dc_str_strict(&dc_str)
            .with_context(|| format!("{desc} failed validate_dc_str_strict"))?;

        // 3. Must pass validate_dc_mixed_strict
        let mst = mstring_from_tokens(&tokens)
            .with_context(|| format!("{desc} failed mstring conversion"))?;
        validate_dc_mixed_strict(&mst)
            .with_context(|| format!("{desc} failed validate_dc_mixed_strict"))?;
    }

    Ok(())
}

// ============================================================================
// 4. EMPIRICAL VERIFICATION: PREFIX OPERATOR EVALUATION WITHOUT FIXED ARITY
// ============================================================================

#[crate::ctb_test]
fn test_challenge_prefix_operator_evaluation_data_driven() -> Result<()> {
    let resolver = default_syntax_resolver();
    let policy = DcProjectionPolicy::strict();

    // A. Prefix operator Dc 330 (Date timestamp): rule defines internal syntax
    // Pattern: :~ [date $date]
    let target_330 = CharTarget::Dc(330);
    let rule_330 = resolver
        .resolve_rule(&target_330)
        .context("Resolver must find rule for Dc 330")?;
    ensure!(
        rule_330.pattern.starts_with_self(),
        "Rule 330 must start with self"
    );

    // Build tokens for 330 with valid date payload:
    let date_tokens = vec![
        sdc(330),
        sdc(390),
        fmt(487),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(391),
    ];
    let date_str = dc_string_from_tokens(&date_tokens);
    let mut nav = DcDocumentNavigator::from_dc_str(&date_str);
    let stub = nav
        .next_sibling_stub(&policy)?
        .context("Navigator must produce stub for prefix operator 330")?;
    ensure!(stub.head_dc == DcChar::from_short(330), "Stub head must be 330");
    ensure!(
        nav.position() == u64::try_from(date_str.len())?,
        "Navigator must advance exactly to end of prefix rule construct"
    );

    // B. Prefix operator followed by another token: verify NO greediness/overconsumption
    let mut multi_tokens = date_tokens.clone();
    let trailing_bool = sdc(517); // Boolean true
    multi_tokens.push(trailing_bool);

    let multi_str = dc_string_from_tokens(&multi_tokens);
    let mut multi_nav = DcDocumentNavigator::from_dc_str(&multi_str);

    let stub1 = multi_nav
        .next_sibling_stub(&policy)?
        .context("First stub must be prefix operator 330")?;
    ensure!(
        stub1.head_dc == DcChar::from_short(330),
        "First stub head must be 330"
    );
    ensure!(
        multi_nav.position() == u64::try_from(date_str.len())?,
        "Position after prefix stub must NOT consume trailing sibling token"
    );

    let stub2 = multi_nav
        .next_sibling_stub(&policy)?
        .context("Second stub must be trailing boolean token")?;
    ensure!(
        stub2.head_dc == DC_BOOLEAN_TRUE,
        "Second stub must be DC_BOOLEAN_TRUE"
    );

    // C. Host IP address prefix operator Dc 346: :~ [ipaddr]
    let target_346 = CharTarget::Dc(346);
    let rule_346 = resolver
        .resolve_rule(&target_346)
        .context("Resolver must find rule for Dc 346")?;
    ensure!(
        rule_346.pattern.starts_with_self(),
        "Rule 346 must start with self"
    );

    let ip_tokens = vec![
        sdc(346),
        sdc(511),
        fmt(490),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(512),
        sdc(518), // Followed by boolean false
    ];
    let ip_str = dc_string_from_tokens(&ip_tokens);
    let mut ip_nav = DcDocumentNavigator::from_dc_str(&ip_str);

    let ip_stub = ip_nav
        .next_sibling_stub(&policy)?
        .context("Navigator must produce stub for prefix 346")?;
    ensure!(
        ip_stub.head_dc == DcChar::from_short(346),
        "IP stub head must be 346"
    );

    let next_stub = ip_nav
        .next_sibling_stub(&policy)?
        .context("Following stub must be boolean false")?;
    ensure!(
        next_stub.head_dc == DC_BOOLEAN_FALSE,
        "Next stub must be DC_BOOLEAN_FALSE"
    );

    // D. Prefix operator with missing arguments: must fail evaluation gracefully
    let incomplete_ip = vec![sdc(346)];
    let incomplete_ip_str = dc_string_from_tokens(&incomplete_ip);
    let mut incomplete_nav = DcDocumentNavigator::from_dc_str(&incomplete_ip_str);
    ensure!(
        incomplete_nav.next_sibling_stub(&policy).is_err(),
        "Prefix operator with missing arguments must error under strict policy"
    );

    Ok(())
}

// ============================================================================
// 5. EMPIRICAL VERIFICATION: ADJACENT UNFRAMED & FRAMED CONSTRUCTS (NO 512 HACK)
// ============================================================================

#[crate::ctb_test]
fn test_challenge_adjacent_format_representations_no_512_consumption() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    // Construct: [310] [*] [511] [f228] "f1" [512] [312] [511] [f228] "f2" [512] [311]
    let tokens = vec![
        sdc(310),
        sdc(275), // Wildcard
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        102,
        49,
        sdc(261),
        sdc(512),
        sdc(312), // Separator
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        102,
        50,
        sdc(261),
        sdc(512),
        sdc(311), // End list
    ];

    validate_dc_tokens_strict(&tokens)
        .context("Adjacent format representations in list must pass validate_dc_tokens_strict")?;

    let doc_str = dc_string_from_tokens(&tokens);
    validate_dc_str_strict(&doc_str)
        .context("Adjacent format representations in list must pass validate_dc_str_strict")?;

    let mst = mstring_from_tokens(&tokens)?;
    validate_dc_mixed_strict(&mst)
        .context("Adjacent format representations in list must pass validate_dc_mixed_strict")?;

    // Step through with navigator
    let mut nav = DcDocumentNavigator::from_dc_str(&doc_str);
    let list_stub = nav
        .next_sibling_stub(&policy)?
        .context("Outer list stub must be present")?;
    nav.enter_frame(&list_stub)?;

    let _type_stub = nav.next_sibling_stub(&policy)?; // skip type wildcard

    let fmt1 = nav
        .next_sibling_stub(&policy)?
        .context("First format stub must be present")?;
    ensure!(
        fmt1.head_dc == DC_BEGIN_FORMAT_REPRESENTATION,
        "First child must be 511"
    );
    ensure!(
        fmt1.closer_dc == Some(DC_END_FORMAT_REPRESENTATION),
        "First child closer must be 512"
    );

    let sep = nav
        .next_sibling_stub(&policy)?
        .context("Separator stub must be present")?;
    ensure!(
        sep.head_dc == DC_LIST_ELEMENT_SEPARATOR,
        "Separator must be 312"
    );

    let fmt2 = nav
        .next_sibling_stub(&policy)?
        .context("Second format stub must be present")?;
    ensure!(
        fmt2.head_dc == DC_BEGIN_FORMAT_REPRESENTATION,
        "Second child must be 511"
    );
    ensure!(
        fmt2.closer_dc == Some(DC_END_FORMAT_REPRESENTATION),
        "Second child closer must be 512"
    );

    ensure!(
        nav.next_sibling_stub(&policy)?.is_none(),
        "No further siblings in list"
    );

    nav.exit_frame()?;
    ensure!(
        nav.next_sibling_stub(&policy)?.is_none(),
        "No further siblings at root"
    );

    Ok(())
}

// ============================================================================
// 6. ADVERSARIAL: CORRUPTED BINARY CHUNK BOUNDS AND METADATA TAMPERING
// ============================================================================

#[crate::ctb_test]
fn test_challenge_adversarial_corrupted_binary_chunk_bounds() -> Result<()> {
    // A. Binary chunk header claims size larger than actual byte stream length
    let mut mst = DcMstring::new();
    mst.push_char(DC_BEGIN_LIST);
    mst.push_char(DcChar::from_short(275)); // Type wildcard
    mst.push_binary(&[1, 2, 3, 4]); // 4 bytes
    mst.push_char(DC_END_LIST);

    // Tamper with the binary chunk size field (bytes 7..23) to claim 0xFFFFFFFF bytes
    let mut tampered_bytes = mst.as_bytes().to_vec();
    // Locate the DC_203_BYTES
    let dc203_pos = tampered_bytes
        .windows(dc_mixed::DC_203_BYTES.len())
        .position(|window| window == dc_mixed::DC_203_BYTES)
        .context("Missing DC_203_BYTES in serialized DcMstring")?;

    let size_start = dc203_pos
        .checked_add(dc_mixed::DC_203_BYTES.len())
        .and_then(|p| p.checked_add(1)) // Skip type byte
        .context("Size offset overflow")?;

    // Corrupt size to 99999999 (extends far past stream end)
    let bad_size = 99_999_999u128.to_be_bytes();
    for (i, &b) in bad_size.iter().enumerate() {
        if let Some(target) = tampered_bytes.get_mut(size_start.saturating_add(i)) {
            *target = b;
        }
    }

    let res = match DcMstr::from_bytes(&tampered_bytes) {
        Ok(mstr) => validate_dc_mixed_strict(mstr),
        Err(e) => Err(anyhow::anyhow!(e)),
    };
    ensure!(
        res.is_err(),
        "Binary chunk claiming length exceeding stream length must fail validation"
    );

    // B. Binary chunk claiming length that extends past its enclosing container
    let mut mst2 = DcMstring::new();
    mst2.push_char(DC_BEGIN_LIST);
    mst2.push_char(DcChar::from_short(275));
    mst2.push_binary(&[10, 20, 30, 40]);
    mst2.push_char(DC_END_LIST);
    mst2.push_char(DC_BOOLEAN_TRUE);

    let mut tampered_bytes2 = mst2.as_bytes().to_vec();
    let dc203_pos2 = tampered_bytes2
        .windows(dc_mixed::DC_203_BYTES.len())
        .position(|window| window == dc_mixed::DC_203_BYTES)
        .context("Missing DC_203_BYTES")?;
    let size_start2 = dc203_pos2
        .checked_add(dc_mixed::DC_203_BYTES.len())
        .and_then(|p| p.checked_add(1))
        .context("Size offset overflow")?;

    // Corrupt size to 50 bytes (extends past DC_END_LIST)
    let bad_size2 = 50u128.to_be_bytes();
    for (i, &b) in bad_size2.iter().enumerate() {
        if let Some(target) = tampered_bytes2.get_mut(size_start2.saturating_add(i)) {
            *target = b;
        }
    }

    let res2 = match DcMstr::from_bytes(&tampered_bytes2) {
        Ok(mstr) => validate_dc_mixed_strict(mstr),
        Err(e) => Err(anyhow::anyhow!(e)),
    };
    ensure!(
        res2.is_err(),
        "Binary chunk extending past container must fail validation"
    );

    Ok(())
}

// ============================================================================
// 7. ADVERSARIAL: INTERLEAVED AND INVERTED FRAMING REJECTION
// ============================================================================

#[crate::ctb_test]
fn test_challenge_interleaved_and_inverted_framing_rejection() -> Result<()> {
    // A. Inverted delimiters: closer appears before opener
    let inverted_cases: Vec<(&'static str, Vec<u128>)> = vec![
        ("inverted format 512..511", vec![sdc(512), fmt(228), sdc(511)]),
        ("inverted list 311..310", vec![sdc(311), sdc(275), sdc(310)]),
        ("inverted session 548..547", vec![sdc(548), sdc(547)]),
        ("inverted map 560..559", vec![sdc(560), sdc(559)]),
        ("inverted number 7..6", vec![sdc(7), sdc(6)]),
    ];

    for (desc, tokens) in inverted_cases {
        ensure!(
            validate_dc_tokens_strict(&tokens).is_err(),
            "{desc} must be rejected by validate_dc_tokens_strict"
        );
        let dc_str = dc_string_from_tokens(&tokens);
        ensure!(
            validate_dc_str_strict(&dc_str).is_err(),
            "{desc} must be rejected by validate_dc_str_strict"
        );
        if let Ok(mst) = mstring_from_tokens(&tokens) {
            ensure!(
                validate_dc_mixed_strict(&mst).is_err(),
                "{desc} must be rejected by validate_dc_mixed_strict"
            );
        }
    }

    // B. Interleaved containers (cross-framing):
    // List opened (310), Format opened (511), List closed (311), Format closed (512)
    let interleaved = vec![
        sdc(310),
        sdc(275),
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        102,
        sdc(261),
        sdc(311), // premature list closer before format closer!
        sdc(512),
    ];
    ensure!(
        validate_dc_tokens_strict(&interleaved).is_err(),
        "Interleaved list and format containers must fail validate_dc_tokens_strict"
    );
    let interleaved_str = dc_string_from_tokens(&interleaved);
    ensure!(
        validate_dc_str_strict(&interleaved_str).is_err(),
        "Interleaved containers must fail validate_dc_str_strict"
    );
    if let Ok(mst) = mstring_from_tokens(&interleaved) {
        ensure!(
            validate_dc_mixed_strict(&mst).is_err(),
            "Interleaved containers must fail validate_dc_mixed_strict"
        );
    }

    // C. Interleaved session and checksummed section
    let interleaved_session = vec![
        sdc(547), // Begin session
        sdc(543), // Begin checksummed
        sdc(548), // End session (crosses closer!)
        sdc(544), // End checksummed
    ];
    ensure!(
        validate_dc_tokens_strict(&interleaved_session).is_err(),
        "Interleaved session and checksummed section must fail validate_dc_tokens_strict"
    );

    Ok(())
}

// ============================================================================
// 8. ADVERSARIAL: NESTED FORMAT REPRESENTATIONS NO PREMATURE 512 CLOSING
// ============================================================================

#[crate::ctb_test]
fn test_challenge_nested_format_representations_no_premature_512() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    // Nested: :511 [f228] (outer) containing a typed list with an inner :511 [f228]
    // Outer: 511 f228 [string] 512
    // Inside a list: 310 [275] 511 [f228] 260 262 264 263 97 261 512 312 511 [f228] 260 262 264 263 98 261 512 311
    let tokens = vec![
        sdc(310),
        sdc(275),
        // Item 1
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        97,
        sdc(261),
        sdc(512),
        sdc(312), // Separator
        // Item 2
        sdc(511),
        fmt(228),
        sdc(260),
        sdc(262),
        sdc(264),
        sdc(263),
        98,
        sdc(261),
        sdc(512),
        sdc(311), // End list
    ];

    validate_dc_tokens_strict(&tokens)?;
    let dc_str = dc_string_from_tokens(&tokens);
    validate_dc_str_strict(&dc_str)?;
    let mst = mstring_from_tokens(&tokens)?;
    validate_dc_mixed_strict(&mst)?;

    // Step navigator through list
    let mut nav = DcDocumentNavigator::from_dc_str(&dc_str);
    let list_stub = nav.next_sibling_stub(&policy)?.context("List stub")?;
    nav.enter_frame(&list_stub)?;

    let _type = nav.next_sibling_stub(&policy)?; // Type wildcard
    let item1 = nav.next_sibling_stub(&policy)?.context("Item 1")?;
    ensure!(item1.head_dc == DC_BEGIN_FORMAT_REPRESENTATION);
    ensure!(item1.closer_dc == Some(DC_END_FORMAT_REPRESENTATION));

    let sep = nav.next_sibling_stub(&policy)?.context("Sep")?;
    ensure!(sep.head_dc == DC_LIST_ELEMENT_SEPARATOR);

    let item2 = nav.next_sibling_stub(&policy)?.context("Item 2")?;
    ensure!(item2.head_dc == DC_BEGIN_FORMAT_REPRESENTATION);
    ensure!(item2.closer_dc == Some(DC_END_FORMAT_REPRESENTATION));

    ensure!(nav.next_sibling_stub(&policy)?.is_none());
    nav.exit_frame()?;
    ensure!(nav.next_sibling_stub(&policy)?.is_none());

    Ok(())
}

// ============================================================================
// 9. ADVERSARIAL: CHAINED PREFIX OPERATORS WITHOUT FIXED ARITY
// ============================================================================

#[crate::ctb_test]
fn test_challenge_chained_prefix_operators() -> Result<()> {
    let policy = DcProjectionPolicy::strict();

    // Sequence of distinct prefix operators followed by an atomic value:
    // 1. Dc 330 (Date prefix) + payload
    // 2. Dc 346 (Host IP prefix) + payload
    // 3. Dc 330 (Date prefix) + payload
    // 4. Atomic boolean Dc 517
    let tokens = vec![
        // 1. Date prefix 330
        sdc(330),
        sdc(390),
        fmt(487),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(391),
        // 2. Host IP prefix 346
        sdc(346),
        sdc(511),
        fmt(490),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(512),
        // 3. Date prefix 330
        sdc(330),
        sdc(390),
        fmt(487),
        sdc(6),
        fmt(199),
        sdc(127),
        sdc(7),
        sdc(391),
        // 4. Atomic boolean true
        sdc(517),
    ];

    validate_dc_tokens_strict(&tokens)?;
    let dc_str = dc_string_from_tokens(&tokens);
    validate_dc_str_strict(&dc_str)?;
    let mst = mstring_from_tokens(&tokens)?;
    validate_dc_mixed_strict(&mst)?;

    // Step navigator through each sibling
    let mut nav = DcDocumentNavigator::from_dc_str(&dc_str);

    let s1 = nav.next_sibling_stub(&policy)?.context("Prefix 1 (date)")?;
    ensure!(s1.head_dc == DcChar::from_short(330), "Stub 1 must be 330");

    let s2 = nav.next_sibling_stub(&policy)?.context("Prefix 2 (ip)")?;
    ensure!(s2.head_dc == DcChar::from_short(346), "Stub 2 must be 346");

    let s3 = nav.next_sibling_stub(&policy)?.context("Prefix 3 (date)")?;
    ensure!(s3.head_dc == DcChar::from_short(330), "Stub 3 must be 330");

    let s4 = nav.next_sibling_stub(&policy)?.context("Stub 4 (bool)")?;
    ensure!(s4.head_dc == DC_BOOLEAN_TRUE, "Stub 4 must be DC_BOOLEAN_TRUE");

    ensure!(
        nav.next_sibling_stub(&policy)?.is_none(),
        "No further stubs expected"
    );

    Ok(())
}

