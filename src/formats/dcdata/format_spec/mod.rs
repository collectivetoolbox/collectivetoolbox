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

//! Format specification domain-specific language (DSL).
//!
//! Provides AST definitions, parsing, formatting, canonical Document Character (Dc)
//! stream encoding/decoding, and semantic validation for layered and composite formats
//! as specified in `docs/file-info.md`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub mod ast;
pub mod dc_stream;
pub mod formatter;
pub mod parser;
pub mod validator;

pub use ast::{
    FormatExpr, FormatOp, MAX_FORMAT_EXPR_DEPTH, MAX_FORMAT_EXPR_NODES,
};
pub use dc_stream::{
    DcToken, decode_dc_stream, decode_dc_stream_string, encode_dc_stream,
    encode_dc_stream_string,
};
pub use formatter::{format_chain_directive, format_expr};
pub use parser::parse_format_expr;
pub use validator::{
    KNOWN_TRANSFORMATION_FORMAT_IDS, REGISTERED_NAMED_TYPES,
    validate_format_expr,
};

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
    use ctb_storage_minimal::shorthand::DcShorthand;

    #[crate::ctb_test]
    fn test_parse_mojibake_expression_and_round_trip() {
        let input = "((f15 > f542) ! f0) > f0";
        let expr = parse_format_expr(input).unwrap();

        assert_eq!(
            expr,
            FormatExpr::Convert(
                Box::new(FormatExpr::Transmute(
                    Box::new(FormatExpr::Convert(
                        Box::new(FormatExpr::Dc(DcShorthand::Format(15))),
                        Box::new(FormatExpr::Dc(DcShorthand::Format(542))),
                    )),
                    Box::new(FormatExpr::Dc(DcShorthand::Format(0))),
                )),
                Box::new(FormatExpr::Dc(DcShorthand::Format(0))),
            )
        );

        // Verify text formatter
        let formatted = format_expr(&expr);
        assert_eq!(formatted, "((f15 > f542) ! f0) > f0");

        // Verify Dc token stream encoding
        let dc_string = encode_dc_stream_string(&expr);
        assert_eq!(dc_string, "302 303 302 f15 f542 f0 f0");

        // Verify Dc stream decoding
        let decoded = decode_dc_stream_string(&dc_string).unwrap();
        assert_eq!(decoded, expr);
        // Confirm the parser flexibly supports other valid syntaxes
        let long_dc_string = "298 302 298 303 298 302 f15 f542 299 f0 299 f0 299";
        let decoded = decode_dc_stream_string(&long_dc_string).unwrap();
        assert_eq!(decoded, expr);
        let manually_written_dc_string = "302 303 302 f15 f542 299 f0 299 f0";
        let decoded = decode_dc_stream_string(&manually_written_dc_string).unwrap();
        assert_eq!(decoded, expr);
        // Doesn't actually need terminators for transmutation/conversion as they only take two parameters... only & or | would need them
        let manually_written_dc_string = "302 303 302 f15 f542 f0 f0";
        let decoded = decode_dc_stream_string(&manually_written_dc_string).unwrap();
        assert_eq!(decoded, expr);
    }

    #[crate::ctb_test]
    fn test_variable_arity_disambiguation_and_dual_syntax() {
        // User example: 1 & ((2 | 3 | 4) > 5)
        let infix = "1 & ((2 | 3 | 4) > 5)";
        let expr = parse_format_expr(infix).unwrap();

        assert_eq!(
            expr,
            FormatExpr::Union(vec![
                FormatExpr::Dc(DcShorthand::Short(1)),
                FormatExpr::Convert(
                    Box::new(FormatExpr::Intersection(vec![
                        FormatExpr::Dc(DcShorthand::Short(2)),
                        FormatExpr::Dc(DcShorthand::Short(3)),
                        FormatExpr::Dc(DcShorthand::Short(4)),
                    ])),
                    Box::new(FormatExpr::Dc(DcShorthand::Short(5))),
                ),
            ])
        );

        // Verify succinct prefix encoding emits 299 only when required to disambiguate
        let dc_string = encode_dc_stream_string(&expr);
        assert_eq!(dc_string, "300 1 302 516 2 3 4 299 5");

        // Verify decoding numeric prefix stream
        let decoded = decode_dc_stream_string(&dc_string).unwrap();
        assert_eq!(decoded, expr);

        // Verify decoding symbolic prefix stream
        let decoded_sym = decode_dc_stream_string("& 1 > | 2 3 4 ) 5").unwrap();
        assert_eq!(decoded_sym, expr);

        // Verify parse_format_expr supports prefix notation directly
        let expr_from_prefix = parse_format_expr("& 1 > | 2 3 4 ) 5").unwrap();
        assert_eq!(expr_from_prefix, expr);

        // Also test variable-arity at the tail without disambiguator: 5 > (2 | 3 | 4)
        let tail_infix = "5 > (2 | 3 | 4)";
        let tail_expr = parse_format_expr(tail_infix).unwrap();
        let tail_dc_string = encode_dc_stream_string(&tail_expr);
        assert_eq!(tail_dc_string, "302 5 516 2 3 4");

        let tail_from_prefix = parse_format_expr("> 5 | 2 3 4").unwrap();
        assert_eq!(tail_from_prefix, tail_expr);
    }

    #[crate::ctb_test]
    fn test_chain_directive_parsing_and_formatting() {
        let directive = "@chain(((f15 > f542) ! f0) > f0)";
        let expr = parse_format_expr(directive).unwrap();
        assert_eq!(
            format_chain_directive(&expr),
            "@chain(((f15 > f542) ! f0) > f0)"
        );
    }

    #[crate::ctb_test]
    fn test_left_associative_operational_operators() {
        let input = "f10 > f20 > f30";
        let expr = parse_format_expr(input).unwrap();
        assert_eq!(
            expr,
            FormatExpr::Convert(
                Box::new(FormatExpr::Convert(
                    Box::new(FormatExpr::Dc(DcShorthand::Format(10))),
                    Box::new(FormatExpr::Dc(DcShorthand::Format(20))),
                )),
                Box::new(FormatExpr::Dc(DcShorthand::Format(30))),
            )
        );
        assert_eq!(format_expr(&expr), "f10 > f20 > f30");
    }

    #[crate::ctb_test]
    fn test_strict_disambiguation_rules() {
        // Mixing & and | without parentheses is rejected
        assert!(parse_format_expr("f10 & f20 | f30").is_err());
        assert!(parse_format_expr("(f10 & f20) | f30").is_ok());
        assert!(parse_format_expr("f10 & (f20 | f30)").is_ok());

        // Mixing & or | with > or ! or : without parentheses is rejected
        assert!(parse_format_expr("f10 & f20 > f30").is_err());
        assert!(parse_format_expr("f10 > f20 & f30").is_err());
        assert!(parse_format_expr("f10 & (f20 > f30)").is_ok());
        assert!(parse_format_expr("(f10 > f20) & f30").is_ok());

        // Homogeneous chaining of & and | is permitted
        assert!(parse_format_expr("f10 & f20 & f30").is_ok());
        assert!(parse_format_expr("f10 | f20 | f30").is_ok());
    }

    #[crate::ctb_test]
    fn test_validator_registered_named_types_and_shorthands() {
        // Valid registered named type
        let valid_named = parse_format_expr("f0 > string").unwrap();
        assert!(validate_format_expr(&valid_named).is_ok());

        // Unregistered identifier / nickname is rejected
        let invalid_nick = parse_format_expr("f0 > utf8").unwrap();
        assert!(validate_format_expr(&invalid_nick).is_err());

        // Valid transformation target
        let valid_trans = parse_format_expr("f10 : f323").unwrap();
        assert!(validate_format_expr(&valid_trans).is_ok());

        // Invalid transformation target (f0 is UTF-8 encoding, not a registered transformation)
        let invalid_trans = parse_format_expr("f10 : f0").unwrap();
        assert!(validate_format_expr(&invalid_trans).is_err());
    }

    #[crate::ctb_test]
    fn test_stream_decode_round_trip_for_composite_trees() {
        let input = "f10 & (f20 | (f30 > f40))";
        let expr = parse_format_expr(input).unwrap();
        let dc_tokens = encode_dc_stream_string(&expr);
        let decoded = decode_dc_stream_string(&dc_tokens).unwrap();
        assert_eq!(decoded, expr);
        assert_eq!(format_expr(&decoded), input);
    }
}
