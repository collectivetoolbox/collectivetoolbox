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

//! Document Character (Dc) syntactic and semantic role classifications.
//!
//! Replaces hardcoded Dc constants and numeric checks across readers and
//! deserializers with dynamic, database-driven role queries.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_utilities::dc_char::{FORMAT_REGION_END, FORMAT_REGION_START};
use serde::{Deserialize, Serialize};

/// Classification of syntactic framing and semantic roles for Document
/// Characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DcRole {
    // Escaping & Embedding
    /// Escape prefix character (`255`).
    Escape,
    /// Long Document Character embed prefix (`308`).
    LongDcEmbedPrefix,
    /// Binary encapsulation opening delimiter (`203`).
    BinaryEncapsulationOpener,
    /// Binary encapsulation closing delimiter (`204`).
    BinaryEncapsulationCloser,
    /// UTF-8 string encapsulation opening delimiter (`191`).
    Utf8EncapsulationOpener,
    /// UTF-8 string encapsulation closing delimiter (`192`).
    Utf8EncapsulationCloser,

    // Literals, Idents, Numbers
    /// Framed literal value opening delimiter (`260`).
    LiteralOpener,
    /// Framed literal value closing delimiter (`261`).
    LiteralCloser,
    /// Framed identifier opening delimiter (`270`).
    IdentifierOpener,
    /// Framed identifier closing delimiter (`271`).
    IdentifierCloser,
    /// Framed number opening delimiter (`6`).
    NumberOpener,
    /// Framed number closing delimiter (`7`).
    NumberCloser,
    /// Integer format indicator (`FormatId::Integer` `f199` or `8`).
    IntegerFormatIndicator,
    /// Positive numeric sign prefix (`10`).
    PositiveSign,
    /// Negative numeric sign prefix (`11`).
    NegativeSign,
    /// Base64 encapsulation digit (`127..=190`).
    Base64Digit,
    /// Base64 encapsulation padding sentinel (`195`).
    Base64Padding,

    // Collections & Maps
    /// List container opening delimiter (`310`).
    ListOpener,
    /// List container closing delimiter (`311`).
    ListCloser,
    /// List and tuple element separator (`312`).
    ListElementSeparator,
    /// Key-value map opening delimiter with pair containers (`559`).
    KvMapPairsOpener,
    /// Key-value map closing delimiter with pair containers (`560`).
    KvMapPairsCloser,
    /// Legacy key-value map opening delimiter (`313`).
    KvMapLegacyOpener,
    /// Legacy key-value map closing delimiter (`314`).
    KvMapLegacyCloser,
    /// Key-value pair container opening delimiter (`514`).
    KvPairOpener,
    /// Key-value pair container closing delimiter (`515`).
    KvPairCloser,

    // Types & Format Specs
    /// Type definition opening delimiter (`262`).
    TypeDefOpener,
    /// Type definition closing delimiter (`263`).
    TypeDefCloser,
    /// Wildcard / any type indicator (`275`).
    TypeAny,
    /// String type indicator (`264`).
    TypeString,
    /// Number type indicator (`265`).
    TypeNumber,
    /// Routine type indicator (`266`).
    TypeRoutine,
    /// Routine name type indicator (`267`).
    TypeRoutineName,
    /// Type union operator (`300`).
    TypeUnion,
    /// Type intersection operator (`516`).
    TypeIntersection,
    /// Parametric type application operator (`535`).
    TypeParamApplication,
    /// Type grouping opening parenthesis (`298`).
    GroupOpen,
    /// Type grouping closing parenthesis (`299`).
    GroupClose,
    /// General type token (script `.EL Types` or `.Formats`).
    TypeToken,

    // Values & Optionals
    /// Explicit optional present value tag (`315`).
    OptionalPresent,
    /// Explicit optional absent value sentinel (`316`).
    OptionalAbsent,
    /// Boolean true literal (`517`).
    BooleanTrue,
    /// Boolean false literal (`518`).
    BooleanFalse,
    /// Primitive value atom (absent optional or boolean literal).
    PrimitiveValueAtom,
    /// Domain-specific value atom.
    DomainValueAtom,
    /// File path component delimiter (`513`).
    FilePathDelimiter,

    // Representations & Checksums
    /// Equivalent alternative representations opening tag (`397`).
    EquivalentRepresentations,
    /// Partially equivalent representations opening tag (`561`).
    PartiallyEquivalentRepresentations,
    /// Format-specific representation container opening delimiter (`511`).
    FormatRepresentationOpener,
    /// Format-specific representation container closing delimiter (`512`).
    FormatRepresentationCloser,
    /// Checksummed section container opening delimiter (`543`).
    ChecksummedSectionOpener,
    /// Checksummed section container closing delimiter (`544`).
    ChecksummedSectionCloser,
    /// Checksummed section digest tag (`545`).
    SectionDigestTag,
    /// Checksummed section sequence identifier tag (`546`).
    SequenceIdentifierTag,

    // Temporal
    /// Time moment container opening delimiter (`401`).
    TimeMomentOpener,
    /// Time moment container closing delimiter (`402`).
    TimeMomentCloser,
    /// Date magnitude container opening delimiter (`490`).
    DateMagnitudeOpener,
    /// Date magnitude container closing delimiter (`491`).
    DateMagnitudeCloser,
    /// Date duration container opening delimiter (`492`).
    DateDurationOpener,
    /// Date duration container closing delimiter (`493`).
    DateDurationCloser,

    // Scopes, Bindings & Calls
    /// Identifier scope opening delimiter (`541`).
    IdentifierScopeOpener,
    /// Identifier scope closing delimiter (`542`).
    IdentifierScopeCloser,
    /// Assignment operator (`269`).
    AssignOperator,
    /// Bound value reference tag (`540`).
    ReferenceBoundValue,
    /// Named object reference tag (`276`).
    ReferenceNamedObject,
    /// Routine invocation tag (`279`).
    RoutineInvocation,
    /// Stream output invocation tag (`256` `void lang.say(String)`).
    StreamSayInvocation,
    /// Parameter container opening delimiter (`258`).
    ParameterOpener,
    /// Parameter container closing delimiter (`259`).
    ParameterCloser,
    /// Starter token for an evaluable Dc expression.
    EvaluableExpressionStarter,

    // Comments & Separators
    /// Single-line comment opening delimiter (`246`).
    SingleLineCommentOpener,
    /// Single-line comment closing delimiter (`247`).
    SingleLineCommentCloser,
    /// Block comment opening delimiter (`248`).
    BlockCommentOpener,
    /// Block comment closing delimiter (`251`).
    BlockCommentCloser,
    /// Structural separator (list element separator or whitespace delimiter).
    StructuralSeparator,
}

impl DcRole {
    /// Returns true if the given canonical token (in global graph `u128` layout)
    /// directly matches this role.
    #[must_use]
    pub fn matches_canonical(self, token: u128) -> bool {
        let short = DcChar::from_u128(token).to_short().ok();
        match self {
            Self::Escape => short == Some(255),
            Self::LongDcEmbedPrefix => short == Some(308),
            Self::BinaryEncapsulationOpener => short == Some(203),
            Self::BinaryEncapsulationCloser => short == Some(204),
            Self::Utf8EncapsulationOpener => short == Some(191),
            Self::Utf8EncapsulationCloser => short == Some(192),

            Self::LiteralOpener => short == Some(260),
            Self::LiteralCloser => short == Some(261),
            Self::IdentifierOpener => short == Some(270),
            Self::IdentifierCloser => short == Some(271),
            Self::NumberOpener => short == Some(6),
            Self::NumberCloser => short == Some(7),
            Self::IntegerFormatIndicator => {
                short == Some(8)
                    || (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&token)
                        && ctb_utilities::format_id::FormatId::from_dc_id(token)
                            == Some(ctb_utilities::format_id::FormatId::Integer)
            }
            Self::PositiveSign => short == Some(10),
            Self::NegativeSign => short == Some(11),
            Self::Base64Digit => short.is_some_and(|s| (127..=190).contains(&s)),
            Self::Base64Padding => short == Some(195),

            Self::ListOpener => short == Some(310),
            Self::ListCloser => short == Some(311),
            Self::ListElementSeparator => short == Some(312),
            Self::KvMapPairsOpener => short == Some(559),
            Self::KvMapPairsCloser => short == Some(560),
            Self::KvMapLegacyOpener => short == Some(313),
            Self::KvMapLegacyCloser => short == Some(314),
            Self::KvPairOpener => short == Some(514),
            Self::KvPairCloser => short == Some(515),

            Self::TypeDefOpener => short == Some(262),
            Self::TypeDefCloser => short == Some(263),
            Self::TypeAny => short == Some(275),
            Self::TypeString => short == Some(264),
            Self::TypeNumber => short == Some(265),
            Self::TypeRoutine => short == Some(266),
            Self::TypeRoutineName => short == Some(267),
            Self::TypeUnion => short == Some(300),
            Self::TypeIntersection => short == Some(516),
            Self::TypeParamApplication => short == Some(535),
            Self::GroupOpen => short == Some(298),
            Self::GroupClose => short == Some(299),
            Self::TypeToken => {
                (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&token)
                    || matches!(short, Some(264 | 265 | 266 | 267 | 275 | 300 | 513 | 516 | 535))
            }

            Self::OptionalPresent => short == Some(315),
            Self::OptionalAbsent => short == Some(316),
            Self::BooleanTrue => short == Some(517),
            Self::BooleanFalse => short == Some(518),
            Self::PrimitiveValueAtom => matches!(short, Some(316 | 517 | 518)),
            Self::DomainValueAtom => matches!(short, Some(316 | 513 | 517 | 518 | 379..=384 | 482..=489)),
            Self::FilePathDelimiter => short == Some(513),

            Self::EquivalentRepresentations => short == Some(397),
            Self::PartiallyEquivalentRepresentations => short == Some(561),
            Self::FormatRepresentationOpener => short == Some(511),
            Self::FormatRepresentationCloser => short == Some(512),
            Self::ChecksummedSectionOpener => short == Some(543),
            Self::ChecksummedSectionCloser => short == Some(544),
            Self::SectionDigestTag => short == Some(545),
            Self::SequenceIdentifierTag => short == Some(546),

            Self::TimeMomentOpener => short == Some(401),
            Self::TimeMomentCloser => short == Some(402),
            Self::DateMagnitudeOpener => short == Some(490),
            Self::DateMagnitudeCloser => short == Some(491),
            Self::DateDurationOpener => short == Some(492),
            Self::DateDurationCloser => short == Some(493),

            Self::IdentifierScopeOpener => short == Some(541),
            Self::IdentifierScopeCloser => short == Some(542),
            Self::AssignOperator => short == Some(269),
            Self::ReferenceBoundValue => short == Some(540),
            Self::ReferenceNamedObject => short == Some(276),
            Self::RoutineInvocation => short == Some(279),
            Self::StreamSayInvocation => short == Some(256),
            Self::ParameterOpener => short == Some(258),
            Self::ParameterCloser => short == Some(259),
            Self::EvaluableExpressionStarter => {
                matches!(short, Some(540 | 276 | 279 | 256 | 397 | 561 | 511))
            }

            Self::SingleLineCommentOpener => short == Some(246),
            Self::SingleLineCommentCloser => short == Some(247),
            Self::BlockCommentOpener => short == Some(248),
            Self::BlockCommentCloser => short == Some(251),
            Self::StructuralSeparator => {
                matches!(short, Some(312 | 119 | 120 | 121 | 294 | 295))
            }
        }
    }

    /// Returns the Base64 numeral digit value (`0..=63`) or padding sentinel
    /// (`64`) if `token` represents a Base64 encapsulation character.
    #[must_use]
    pub fn base64_digit_value(token: u128) -> Option<u8> {
        let short = DcChar::from_u128(token).to_short().ok()?;
        if short == 195 {
            Some(64)
        } else if (127..=190).contains(&short) {
            u8::try_from(short.saturating_sub(127)).ok()
        } else {
            None
        }
    }
}

