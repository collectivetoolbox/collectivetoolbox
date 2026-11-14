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

//! Abstract syntax tree definitions for StageL 3 (`f656` `.sl3`) and
//! StageR 3 (`f657` `.sr3`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::DcChar;
use crate::DcShorthand;
use malachite::Integer;
use std::fmt;

/// Dialect selector controlling StageL 3 vs StageR 3 source normalization,
/// parsing, validation, and Token Dc serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StageDialect {
    /// StageL 3 (`.sl3` `f656` / `.sli3` `f659`): minimal systems substrate.
    StageL3,
    /// StageR 3 (`.sr3` `f657` / `.sri3` `f660`): strictly total nominal dialect.
    StageR3,
}

/// 1-based source location used in parser and validator diagnostics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Span {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub col: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// Reference to a declared positional `struct` type in StageL 3 (`Dc 2673`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructTypeRef {
    /// Syntactic identifier or local `use ... as` alias (`point`).
    Ident(String),
    /// Direct graph Dc shorthand (`dc/l50`).
    Dc(DcChar),
    /// Canonicalized `(declaring_namespace, original_name)` identity produced by validation.
    Canonical {
        /// Declaring module's namespace path (`None` if declared in an anonymous module).
        namespace: Option<String>,
        /// Original struct name in its declaring module.
        name: String,
    },
}

/// Static value or control type in StageL 3.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StageLType {
    /// Boolean truth value (`b`, `f1424`).
    B,
    /// Unsigned 8-bit integer (`u8`, `f203`).
    U8,
    /// Signed 32-bit integer (`i32`, `f208`).
    I32,
    /// Unsigned 32-bit integer (`u32`, `f207`).
    U32,
    /// Signed 64-bit integer (`i64`, `f210`).
    I64,
    /// Unsigned 64-bit integer (`u64`, `f209`).
    U64,
    /// Unit completion type (`void`, `f1425`).
    Void,
    /// Bottom / non-returning control-transfer type (`never`, `f1426`).
    Never,
    /// Immutable value list (`l/T`, `Dc 2576`).
    List(Box<StageLType>),
    /// Mutable lexical builder buffer (`buf/T`, `Dc 2577`).
    Buf(Box<StageLType>),
    /// Anonymous fixed-slot `n`-ary tuple (`tup: T1 T2 ... ::`, `Dc 2672`).
    Tuple(Vec<StageLType>),
    /// Declared positional or named `struct` type (`name` or `dc/<shorthand>`, `Dc 2673`).
    Struct(StructTypeRef),
    /// Fixed-width 128-bit signed integer (`i128`, `f212`).
    I128,
    /// Fixed-width 128-bit unsigned integer (`u128`, `f211`).
    U128,
    /// Arbitrary-precision signed integer (`int`, `f199`).
    Int,
    /// Arbitrary-precision canonical reduced rational (`rat`, `f345`).
    Rat,
    /// Validated 128-bit Document Character scalar (`dc`, `Dc 640`).
    Dc,
    /// Pure Document Character sequence (`dcs`, `Dc 264`).
    Dcs,
    /// Mixed-mode Document Character + binary string (`s` / `DcMst`, `f484`).
    S,
    /// Structured StageR 3 diagnostic error value (`error`, `Dc 2676`).
    Error,
    /// Immutable bounds-checked streaming extent handle (`view`, `Dc 2677`).
    View,
    /// Immutable value-typed extent stream cursor (`cursor`, `Dc 2678`).
    Cursor,
    /// Parameterized optional value type (`option/T`, `Dc 2674`).
    Option(Box<StageLType>),
    /// Parameterized fallible result type (`result/T`, `Dc 2675`).
    Result(Box<StageLType>),
    /// Declared StageR 3 nominal brand, module-opaque type, or enum (`Dc 2679`).
    Nominal(StructTypeRef),
}

impl StageLType {
    /// Formats this type in canonical StageL/StageR 3 notation (`u32`, `l/u8`, etc.).
    #[must_use]
    pub fn display_name(&self) -> String {
        match self {
            Self::B => String::from("b"),
            Self::U8 => String::from("u8"),
            Self::I32 => String::from("i32"),
            Self::U32 => String::from("u32"),
            Self::I64 => String::from("i64"),
            Self::U64 => String::from("u64"),
            Self::I128 => String::from("i128"),
            Self::U128 => String::from("u128"),
            Self::Int => String::from("int"),
            Self::Rat => String::from("rat"),
            Self::Dc => String::from("dc"),
            Self::Dcs => String::from("dcs"),
            Self::S => String::from("s"),
            Self::Error => String::from("error"),
            Self::View => String::from("view"),
            Self::Cursor => String::from("cursor"),
            Self::Void => String::from("void"),
            Self::Never => String::from("never"),
            Self::List(inner) => format!("l/{}", inner.display_name()),
            Self::Buf(inner) => format!("buf/{}", inner.display_name()),
            Self::Option(inner) => format!("option/{}", inner.display_name()),
            Self::Result(inner) => format!("result/{}", inner.display_name()),
            Self::Tuple(elems) => {
                if elems.is_empty() {
                    String::from("tup: ::")
                } else {
                    let inner: Vec<String> =
                        elems.iter().map(Self::display_name).collect();
                    format!("tup: {} ::", inner.join(" "))
                }
            }
            Self::Struct(StructTypeRef::Ident(name))
            | Self::Nominal(StructTypeRef::Ident(name)) => name.clone(),
            Self::Struct(StructTypeRef::Dc(dc))
            | Self::Nominal(StructTypeRef::Dc(dc)) => {
                format!("dc/{}", format_dc_shorthand(*dc))
            }
            Self::Struct(StructTypeRef::Canonical { name, .. })
            | Self::Nominal(StructTypeRef::Canonical { name, .. }) => {
                name.clone()
            }
        }
    }

    /// Returns `true` for StageL 3 fixed-width integer types (`u8`, `i32`, `u32`,
    /// `i64`, `u64`).
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::U8 | Self::I32 | Self::U32 | Self::I64 | Self::U64
        )
    }

    /// Returns `true` for StageR 3 fixed-width integer types (`u8`, `i32`, `u32`,
    /// `i64`, `u64`, `i128`, `u128`).
    #[must_use]
    pub const fn is_stager_fixed_int(&self) -> bool {
        matches!(
            self,
            Self::U8
                | Self::I32
                | Self::U32
                | Self::I64
                | Self::U64
                | Self::I128
                | Self::U128
        )
    }

    /// Returns `true` for all StageR 3 numeric types (`u8..=u128`, `int`, `rat`).
    #[must_use]
    pub const fn is_stager_numeric(&self) -> bool {
        self.is_stager_fixed_int() || matches!(self, Self::Int | Self::Rat)
    }

    /// Returns `true` for StageL 3 signed integer types (`i32`, `i64`).
    #[must_use]
    pub const fn is_signed_int(&self) -> bool {
        matches!(self, Self::I32 | Self::I64)
    }

    /// Returns `true` for StageR 3 signed numeric types supporting `neg` /
    /// signed operations (`i32`, `i64`, `i128`, `int`, `rat`).
    #[must_use]
    pub const fn is_stager_signed(&self) -> bool {
        matches!(
            self,
            Self::I32 | Self::I64 | Self::I128 | Self::Int | Self::Rat
        )
    }

    /// Returns `true` for unsigned integer types (`u8`, `u32`, `u64`).
    #[must_use]
    pub const fn is_unsigned_int(&self) -> bool {
        matches!(self, Self::U8 | Self::U32 | Self::U64)
    }

    /// Returns `true` for StageR 3 unsigned fixed-width integer types (`u8`, `u32`,
    /// `u64`, `u128`).
    #[must_use]
    pub const fn is_stager_unsigned_int(&self) -> bool {
        matches!(self, Self::U8 | Self::U32 | Self::U64 | Self::U128)
    }

    /// Returns the bit width for fixed-width integer types (`8`, `32`, `64`, `128`).
    #[must_use]
    pub const fn bit_width(&self) -> Option<u32> {
        match self {
            Self::U8 => Some(8),
            Self::I32 | Self::U32 => Some(32),
            Self::I64 | Self::U64 => Some(64),
            Self::I128 | Self::U128 => Some(128),
            _ => None,
        }
    }

    /// Returns `true` if values of this type may be stored as elements of a
    /// `l/T` list or `buf/T` buffer.
    #[must_use]
    pub fn can_be_element(&self) -> bool {
        match self {
            Self::Void | Self::Never | Self::Buf(_) => false,
            Self::List(inner) | Self::Option(inner) => inner.can_be_element(),
            Self::Result(inner) => {
                **inner == Self::Void || inner.can_be_element()
            }
            Self::Tuple(elems) => elems.iter().all(Self::can_be_element),
            Self::B
            | Self::U8
            | Self::I32
            | Self::U32
            | Self::I64
            | Self::U64
            | Self::I128
            | Self::U128
            | Self::Int
            | Self::Rat
            | Self::Dc
            | Self::Dcs
            | Self::S
            | Self::Error
            | Self::View
            | Self::Cursor
            | Self::Struct(_)
            | Self::Nominal(_) => true,
        }
    }

    /// Returns `true` if `self` is a `buf/T` type or syntactically contains one.
    #[must_use]
    pub fn is_or_contains_buf(&self) -> bool {
        match self {
            Self::Buf(_) => true,
            Self::List(inner) | Self::Option(inner) | Self::Result(inner) => {
                inner.is_or_contains_buf()
            }
            Self::Tuple(elems) => elems.iter().any(Self::is_or_contains_buf),
            _ => false,
        }
    }

    /// Returns `true` if `eq` / `ne` are syntactically defined for values of this type.
    #[must_use]
    pub fn supports_eq(&self) -> bool {
        match self {
            Self::B
            | Self::U8
            | Self::I32
            | Self::U32
            | Self::I64
            | Self::U64
            | Self::I128
            | Self::U128
            | Self::Int
            | Self::Rat
            | Self::Dc
            | Self::Dcs
            | Self::S
            | Self::Error
            | Self::Struct(_)
            | Self::Nominal(_) => true,
            Self::List(inner) | Self::Option(inner) => inner.supports_eq(),
            Self::Result(inner) => **inner == Self::Void || inner.supports_eq(),
            Self::Tuple(elems) => elems.iter().all(Self::supports_eq),
            Self::Void
            | Self::Never
            | Self::Buf(_)
            | Self::View
            | Self::Cursor => false,
        }
    }

    /// Returns `true` if `self` is a concrete storable value type (not `Void`,
    /// `Never`, or containing `Buf` when checking runtime storage).
    #[must_use]
    pub fn is_concrete_value(&self) -> bool {
        !matches!(self, Self::Void | Self::Never)
    }
}

impl fmt::Display for StageLType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_name())
    }
}

/// Preserved source comment line or inline comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// Whether this comment appeared at the end of a non-comment line (`246`)
    /// rather than on its own line (`247`).
    pub inline: bool,
    /// Comment text after the leading `: ` marker (empty string if blank `:`).
    pub text: String,
    /// Source location of the comment marker.
    pub span: Span,
}

/// Complete StageL 3 or StageR 3 module AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    /// Language dialect (`StageL3` or `StageR3`).
    pub dialect: StageDialect,
    /// Optional PSR-4 namespace path (`ns path/to/mod.`).
    pub namespace: Option<Vec<String>>,
    /// Explicit namespace imports (`use path/to/mod: ... ::.`).
    pub uses: Vec<UseImport>,
    /// File-header comments preceding declarations.
    pub leading_comments: Vec<Comment>,
    /// Ordered top-level items (declarations, routines, and standalone comments).
    pub items: Vec<TopDecl>,
}

/// Target of an item inside a `use` import statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImportTarget {
    /// Imported by exported programmatic name.
    Name(String),
    /// Imported by assigned Document Character shorthand.
    Dc(DcChar),
}

/// One imported symbol entry inside `use path: item1 item2 as alias ::.`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseImportItem {
    /// Imported symbol name or Dc atom in the source namespace.
    pub target: ImportTarget,
    /// Optional local rename alias (`as alias`).
    pub alias: Option<String>,
    /// Source location.
    pub span: Span,
}

/// Explicit namespace import declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseImport {
    /// Target namespace segments (e.g. `["std", "bytes", "slice"]`).
    pub path: Vec<String>,
    /// Explicit imported items (`item [as alias]`).
    pub items: Vec<UseImportItem>,
    /// Comments immediately preceding this `use` statement.
    pub comments_before: Vec<Comment>,
    /// Source location.
    pub span: Span,
}

/// Module-level immutable binding (`const TYPE/name EXPR.` or
/// `pub [dc/<shorthand>] TYPE/name EXPR.`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopValueDecl {
    /// `true` for `pub`, `false` for private `const`.
    pub is_pub: bool,
    /// `true` when written using inline `pub dc/<shorthand> ...`.
    pub pub_dc_syntax: bool,
    /// Assigned graph identity (from `pub dc/<sh>` or `: @dc <sh>`).
    pub dc_id: Option<DcChar>,
    /// Declared static value type.
    pub ty: StageLType,
    /// Binding name.
    pub name: String,
    /// Initializer expression.
    pub value: Expr,
    /// Comments preceding this declaration.
    pub comments_before: Vec<Comment>,
    /// Source location.
    pub span: Span,
}

/// Module-level immutable positional `struct` declaration
/// (`[pub [dc/<shorthand>]] struct name :: T1 T2 ... .`, `Dc 2217..2218`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionalStructDecl {
    /// `true` for `pub struct`, `false` for module-private `struct`.
    pub is_pub: bool,
    /// `true` when written using inline `pub dc/<shorthand> struct ...`.
    pub pub_dc_syntax: bool,
    /// Assigned graph identity (from `pub dc/<sh>` or `: @dc <sh>`).
    pub dc_id: Option<DcChar>,
    /// Positional struct identifier.
    pub name: String,
    /// Ordered positional slot types (`fields.len() >= 1`).
    pub fields: Vec<StageLType>,
    /// `true` if written in inline `:: T1 T2 ... .` header syntax.
    pub inline_body: bool,
    /// Comments preceding this declaration.
    pub comments_before: Vec<Comment>,
    /// Source location.
    pub span: Span,
}

/// StageR 3 nominal brand or module-opaque type declaration
/// (`[pub [dc/<shorthand>]] (brand | type) BASE_TYPE/name.`, `Dc 2223`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandTypeDecl {
    /// `true` for `pub brand` / `pub type`.
    pub is_pub: bool,
    /// `true` when written using inline `pub dc/<shorthand> ...`.
    pub pub_dc_syntax: bool,
    /// `false` for transparent `brand`, `true` for module-opaque `type`.
    pub is_opaque: bool,
    /// Assigned graph identity (from `pub dc/<sh>` or `: @dc <sh>`).
    pub dc_id: Option<DcChar>,
    /// Underlying representation type.
    pub inner_ty: StageLType,
    /// Declared nominal type identifier.
    pub name: String,
    /// Comments preceding this declaration.
    pub comments_before: Vec<Comment>,
    /// Source location.
    pub span: Span,
}

/// One typed named field inside a [`NamedStructDecl`] (`Dc 2226`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedFieldDecl {
    /// Static field type.
    pub ty: StageLType,
    /// Field identifier.
    pub name: String,
    /// Comments preceding this field line.
    pub comments_before: Vec<Comment>,
    /// Source location.
    pub span: Span,
}

/// StageR 3 module-level immutable named-field `struct` declaration
/// (`Dc 2224..2225`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedStructDecl {
    /// `true` for `pub struct`.
    pub is_pub: bool,
    /// `true` when written using inline `pub dc/<shorthand> struct ...`.
    pub pub_dc_syntax: bool,
    /// Assigned graph identity (from `pub dc/<sh>` or `: @dc <sh>`).
    pub dc_id: Option<DcChar>,
    /// Named struct identifier.
    pub name: String,
    /// Declared named fields (`fields.len() >= 1`).
    pub fields: Vec<NamedFieldDecl>,
    /// `true` if written in inline `:: T1/f1 T2/f2 .` header syntax.
    pub inline_body: bool,
    /// Comments preceding this declaration.
    pub comments_before: Vec<Comment>,
    /// Source location.
    pub span: Span,
}

/// One variant declaration inside an [`EnumDecl`] (`Dc 2232`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantDecl {
    /// Variant identifier.
    pub name: String,
    /// Optional variant payload type (`None` for unit variant).
    pub payload: Option<StageLType>,
    /// Comments preceding this variant line.
    pub comments_before: Vec<Comment>,
    /// Source location.
    pub span: Span,
}

/// StageR 3 module-level closed tagged-union `enum` declaration
/// (`Dc 2230..2231`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    /// `true` for `pub enum`.
    pub is_pub: bool,
    /// `true` when written using inline `pub dc/<shorthand> enum ...`.
    pub pub_dc_syntax: bool,
    /// Assigned graph identity (from `pub dc/<sh>` or `: @dc <sh>`).
    pub dc_id: Option<DcChar>,
    /// Enum identifier.
    pub name: String,
    /// Variant declarations (`0` variants allowed for uninhabited `never` enums).
    pub variants: Vec<EnumVariantDecl>,
    /// `true` if written in inline `:: ... .` header syntax.
    pub inline_body: bool,
    /// Comments preceding this declaration.
    pub comments_before: Vec<Comment>,
    /// Source location.
    pub span: Span,
}

/// Module-level item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopDecl {
    /// Top-level StageL 3 `struct` or `pub struct` positional struct declaration.
    Struct(PositionalStructDecl),
    /// Top-level StageR 3 `brand` or opaque `type` declaration.
    BrandType(BrandTypeDecl),
    /// Top-level StageR 3 named-field `struct` declaration.
    NamedStruct(NamedStructDecl),
    /// Top-level StageR 3 tagged-union `enum` declaration.
    Enum(EnumDecl),
    /// Top-level `const` or `pub` immutable value.
    Value(TopValueDecl),
    /// Top-level `v/...` or `pub v/...` routine.
    Routine(RoutineDecl),
    /// Trailing/standalone comment after the last declaration.
    Comment(Comment),
}

/// Routine formal parameter (`TYPE/name`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    /// Parameter type.
    pub ty: StageLType,
    /// Parameter identifier.
    pub name: String,
    /// Source location.
    pub span: Span,
}

/// Expected outcome of an inline routine `test` case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestExpected {
    /// Expected return value expression.
    Expr(Expr),
    /// Expected deterministic trap (`out fault`).
    Fault(Span),
    /// Expected normal completion for a `void` routine (`absent`).
    Void(Span),
}

/// Single `in ... out ...` test case inside a routine `test` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCase {
    /// Positional argument expressions matching the routine parameter list.
    pub inputs: Vec<Expr>,
    /// Expected result, `fault`, or void completion.
    pub expected: TestExpected,
    /// Source location.
    pub span: Span,
}

/// Inline routine test contract block (`test [label] ...`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestBlock {
    /// Optional human-readable label following `test`.
    pub label: Option<String>,
    /// Ordered test cases inside this block.
    pub cases: Vec<TestCase>,
    /// Source location.
    pub span: Span,
}

/// Routine declaration (`v/RET/name ...`, top-level or nested).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineDecl {
    /// Export visibility (`true` for top-level `pub v/...`).
    pub is_pub: bool,
    /// `true` when declared via inline `pub dc/<shorthand> v/...`.
    pub pub_dc_syntax: bool,
    /// Assigned graph identity (from `pub dc/<sh>` or `: @dc <sh>`).
    pub dc_id: Option<DcChar>,
    /// Declared return type (`void` allowed; `never` allowed for diverging verbs).
    pub return_type: StageLType,
    /// Routine name.
    pub name: String,
    /// Fixed-arity formal parameter list.
    pub params: Vec<Param>,
    /// Attached inline test blocks.
    pub tests: Vec<TestBlock>,
    /// Routine body expression (often `Expr::Block`).
    pub body: Expr,
    /// `true` if written in inline `:: BODY.` header syntax.
    pub inline_body: bool,
    /// Comments immediately preceding this routine.
    pub comments_before: Vec<Comment>,
    /// Source location.
    pub span: Span,
}

/// Typed integer literal in StageL 3 and StageR 3.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypedIntLit {
    /// `u8:<N>`
    U8(u8),
    /// `i32:<N>`
    I32(i32),
    /// `u32:<N>`
    U32(u32),
    /// `i64:<N>`
    I64(i64),
    /// `u64:<N>`
    U64(u64),
    /// `i128:<N>`
    I128(i128),
    /// `u128:<N>`
    U128(u128),
    /// `int:<N>` (arbitrary-precision signed integer)
    Int(Integer),
}

impl TypedIntLit {
    /// Returns the StageL/StageR 3 static type of this literal.
    #[must_use]
    pub fn stage_l_type(&self) -> StageLType {
        match self {
            Self::U8(_) => StageLType::U8,
            Self::I32(_) => StageLType::I32,
            Self::U32(_) => StageLType::U32,
            Self::I64(_) => StageLType::I64,
            Self::U64(_) => StageLType::U64,
            Self::I128(_) => StageLType::I128,
            Self::U128(_) => StageLType::U128,
            Self::Int(_) => StageLType::Int,
        }
    }

    /// Formats this literal in canonical `type:value` syntax.
    #[must_use]
    pub fn display_literal(&self) -> String {
        match self {
            Self::U8(v) => format!("u8:{v}"),
            Self::I32(v) => format!("i32:{v}"),
            Self::U32(v) => format!("u32:{v}"),
            Self::I64(v) => format!("i64:{v}"),
            Self::U64(v) => format!("u64:{v}"),
            Self::I128(v) => format!("i128:{v}"),
            Self::U128(v) => format!("u128:{v}"),
            Self::Int(v) => format!("int:{v}"),
        }
    }
}

/// Canonical built-in diagnostic error tags for StageR 3 `error:<tag>` values
/// (`Dc 2238`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StageRErrorTag {
    /// Fixed-width overflow or shift count >= bit width (`overflow`).
    Overflow,
    /// Division, modulo, or `makeRat` with zero divisor/denominator (`divisionByZero`).
    DivisionByZero,
    /// Out-of-bounds buffer set (`outOfBounds`).
    OutOfBounds,
    /// Mutation or duplicate freeze of a frozen `buf/T` (`frozenBuffer`).
    FrozenBuffer,
    /// Mixed-mode `s` -> `dcs` conversion failed due to binary run (`invalidEncoding`).
    InvalidEncoding,
    /// Surrogate or invalid Document Character in `dcFromU128` (`invalidDc`).
    InvalidDc,
    /// Textual parse failure (`parseFailure`).
    ParseFailure,
    /// Propagating `option/T:none` inside a `result/R` routine (`unexpectedNone`).
    UnexpectedNone,
}

impl StageRErrorTag {
    /// Returns the camelCase keyword suffix for `error:<tag>`.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Overflow => "overflow",
            Self::DivisionByZero => "divisionByZero",
            Self::OutOfBounds => "outOfBounds",
            Self::FrozenBuffer => "frozenBuffer",
            Self::InvalidEncoding => "invalidEncoding",
            Self::InvalidDc => "invalidDc",
            Self::ParseFailure => "parseFailure",
            Self::UnexpectedNone => "unexpectedNone",
        }
    }

    /// Parses a camelCase error tag suffix after `error:`.
    #[must_use]
    pub fn from_keyword(raw: &str) -> Option<Self> {
        match raw {
            "overflow" => Some(Self::Overflow),
            "divisionByZero" => Some(Self::DivisionByZero),
            "outOfBounds" => Some(Self::OutOfBounds),
            "frozenBuffer" => Some(Self::FrozenBuffer),
            "invalidEncoding" => Some(Self::InvalidEncoding),
            "invalidDc" => Some(Self::InvalidDc),
            "parseFailure" => Some(Self::ParseFailure),
            "unexpectedNone" => Some(Self::UnexpectedNone),
            _ => None,
        }
    }

    /// Returns all canonical `error` variant keywords for exhaustive `match`.
    #[must_use]
    pub const fn all_keywords() -> &'static [&'static str] {
        &[
            "overflow",
            "divisionByZero",
            "outOfBounds",
            "frozenBuffer",
            "invalidEncoding",
            "invalidDc",
            "parseFailure",
            "unexpectedNone",
        ]
    }
}

/// Unary built-in operator specific to StageR 3 (`Dc 2559..2572`, `Dc 2610..2613`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StageRUnaryOp {
    /// Total checked negation (`neg`, `Dc 2559`).
    TotalNeg,
    /// Option present predicate (`isSome`, `Dc 2566`).
    IsSome,
    /// Option absent predicate (`isNone`, `Dc 2567`).
    IsNone,
    /// Result success predicate (`isOk`, `Dc 2568`).
    IsOk,
    /// Result error predicate (`isErr`, `Dc 2569`).
    IsErr,
    /// Mixed-mode `s` payload byte length (`byteLen`, `Dc 2572`).
    ByteLen,
    /// Fallible `u128 -> result/dc` validation (`dcFromU128`, `Dc 2610`).
    DcFromU128,
    /// Infallible `dc -> u128` extraction (`u128FromDc`, `Dc 2611`).
    U128FromDc,
    /// Fallible `s -> result/dcs` validation (`dcsFromS`, `Dc 2612`).
    DcsFromS,
    /// Infallible `dcs -> s` widening (`sFromDcs`, `Dc 2613`).
    SFromDcs,
}

impl StageRUnaryOp {
    /// Returns the canonical keyword in `.sr3` notation.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::TotalNeg => "neg",
            Self::IsSome => "isSome",
            Self::IsNone => "isNone",
            Self::IsOk => "isOk",
            Self::IsErr => "isErr",
            Self::ByteLen => "byteLen",
            Self::DcFromU128 => "dcFromU128",
            Self::U128FromDc => "u128FromDc",
            Self::DcsFromS => "dcsFromS",
            Self::SFromDcs => "sFromDcs",
        }
    }
}

/// Binary built-in operator specific to StageR 3 (`Dc 2553..2571`, `Dc 2609`,
/// `Dc 2614..2615`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StageRBinaryOp {
    /// Total checked/exact addition (`add`, `Dc 2553`).
    TotalAdd,
    /// Total checked/exact subtraction (`sub`, `Dc 2554`).
    TotalSub,
    /// Total checked/exact multiplication (`mul`, `Dc 2555`).
    TotalMul,
    /// Total truncating/exact division (`div`, `Dc 2556`).
    TotalDiv,
    /// Total Euclidean modulo (`mod`, `Dc 2557`).
    TotalMod,
    /// Total truncating remainder (`truncRem`, `Dc 2558`).
    TotalTruncRem,
    /// Total checked left shift (`shl`, `Dc 2560`).
    TotalShl,
    /// Total checked right shift (`shr`, `Dc 2561`).
    TotalShr,
    /// Total indexed element access returning `option/Elem` (`get`, `Dc 2563`).
    TotalGet,
    /// Unwrap `option/T` or `result/T` with fallback value (`unwrapOr`, `Dc 2570`).
    UnwrapOr,
    /// Convert `option/T` with `error` into `result/T` (`okOr`, `Dc 2571`).
    OkOr,
    /// Construct canonical reduced `rat` from `(int, int)` (`makeRat`, `Dc 2609`).
    MakeRat,
    /// Split sequence by separator (`explode`, `Dc 2614`).
    Explode,
    /// Join sequences with separator (`implode`, `Dc 2615`).
    Implode,
}

impl StageRBinaryOp {
    /// Returns the canonical keyword in `.sr3` notation.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::TotalAdd => "add",
            Self::TotalSub => "sub",
            Self::TotalMul => "mul",
            Self::TotalDiv => "div",
            Self::TotalMod => "mod",
            Self::TotalTruncRem => "truncRem",
            Self::TotalShl => "shl",
            Self::TotalShr => "shr",
            Self::TotalGet => "get",
            Self::UnwrapOr => "unwrapOr",
            Self::OkOr => "okOr",
            Self::MakeRat => "makeRat",
            Self::Explode => "explode",
            Self::Implode => "implode",
        }
    }
}

/// Ternary built-in operator specific to StageR 3 (`Dc 2564..2565`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StageRTernaryOp {
    /// Total sub-sequence slice returning `option/Seq` (`slice`, `Dc 2564`).
    TotalSlice,
    /// Total checked mutable buffer overwrite returning `result/void` (`bufSet`, `Dc 2565`).
    TotalBufSet,
}

impl StageRTernaryOp {
    /// Returns the canonical keyword in `.sr3` notation.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::TotalSlice => "slice",
            Self::TotalBufSet => "bufSet",
        }
    }
}

/// One `case <variant> [<TYPE>/<binding>] <BODY>` arm inside `Expr::Match` (`Dc 2458`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    /// Matched variant name (`none`/`some`, `ok`/`err`, error tag, or enum variant).
    pub variant: String,
    /// Optional payload binding `(TYPE, name, span)` (`None` for unit variant).
    pub binding: Option<(StageLType, String, Span)>,
    /// Arm body expression.
    pub body: Expr,
    /// Comments preceding this arm.
    pub comments_before: Vec<Comment>,
    /// Source location of `case`.
    pub span: Span,
}

/// Target identifier or resolved Dc reference in `TYPE/name` (`Dc 2202`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolTarget {
    /// Lexical or imported programmatic name.
    Ident(String),
    /// Direct Document Character reference.
    Dc(DcChar),
}

/// Unary core built-in operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreUnaryOp {
    /// Checked negation (`neg`, `Dc 2517`).
    Neg,
    /// Bitwise inversion (`bitNot`, `Dc 2524`).
    BitNot,
    /// Logical negation (`not`, `Dc 2532`).
    Not,
    /// Count leading zero bits (`clz`, `Dc 2529`).
    Clz,
    /// Count trailing zero bits (`ctz`, `Dc 2530`).
    Ctz,
    /// Population bit count (`popcnt`, `Dc 2531`).
    Popcnt,
    /// Sequence/buffer length (`len`, `Dc 2543`).
    Len,
    /// Zero-copy freeze `buf/T -> l/T` (`bufFreeze`, `Dc 2585`).
    BufFreeze,
}

impl CoreUnaryOp {
    /// Canonical camelCase source keyword.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Neg => "neg",
            Self::BitNot => "bitNot",
            Self::Not => "not",
            Self::Clz => "clz",
            Self::Ctz => "ctz",
            Self::Popcnt => "popcnt",
            Self::Len => "len",
            Self::BufFreeze => "bufFreeze",
        }
    }
}

/// Binary core built-in operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreBinaryOp {
    /// Checked add (`add`, `Dc 2512`).
    Add,
    /// Checked sub (`sub`, `Dc 2513`).
    Sub,
    /// Checked mul (`mul`, `Dc 2514`).
    Mul,
    /// Checked truncating div (`div`, `Dc 2515`).
    Div,
    /// Checked truncating remainder (`rem`, `Dc 2516`).
    Rem,
    /// Wrapping add (`wrapAdd`, `Dc 2518`).
    WrapAdd,
    /// Wrapping sub (`wrapSub`, `Dc 2519`).
    WrapSub,
    /// Wrapping mul (`wrapMul`, `Dc 2520`).
    WrapMul,
    /// Bitwise AND (`bitAnd`, `Dc 2521`).
    BitAnd,
    /// Bitwise OR (`bitOr`, `Dc 2522`).
    BitOr,
    /// Bitwise XOR (`bitXor`, `Dc 2523`).
    BitXor,
    /// Shift left (`shl`, `Dc 2525`).
    Shl,
    /// Shift right (`shr`, `Dc 2526`).
    Shr,
    /// Rotate left (`rotl`, `Dc 2527`).
    Rotl,
    /// Rotate right (`rotr`, `Dc 2528`).
    Rotr,
    /// Short-circuiting logical AND (`and`, `Dc 2533`).
    And,
    /// Short-circuiting logical OR (`or`, `Dc 2534`).
    Or,
    /// Structural / value equality (`eq`, `Dc 2535`).
    Eq,
    /// Structural / value inequality (`ne`, `Dc 2536`).
    Ne,
    /// Less than (`lt`, `Dc 2537`).
    Lt,
    /// Less than or equal (`le`, `Dc 2538`).
    Le,
    /// Greater than (`gt`, `Dc 2539`).
    Gt,
    /// Greater than or equal (`ge`, `Dc 2540`).
    Ge,
    /// Sequence element indexing (`get`, `Dc 2544`).
    Get,
    /// Sequence concatenation (`concat`, `Dc 2546`).
    Concat,
    /// Filled buffer allocation (`bufFill`, `Dc 2581`).
    BufFill,
    /// Buffer push append (`bufPush`, `Dc 2582`).
    BufPush,
    /// Buffer bulk list append (`bufExtend`, `Dc 2583`).
    BufExtend,
}

impl CoreBinaryOp {
    /// Canonical camelCase source keyword.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Rem => "rem",
            Self::WrapAdd => "wrapAdd",
            Self::WrapSub => "wrapSub",
            Self::WrapMul => "wrapMul",
            Self::BitAnd => "bitAnd",
            Self::BitOr => "bitOr",
            Self::BitXor => "bitXor",
            Self::Shl => "shl",
            Self::Shr => "shr",
            Self::Rotl => "rotl",
            Self::Rotr => "rotr",
            Self::And => "and",
            Self::Or => "or",
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::Lt => "lt",
            Self::Le => "le",
            Self::Gt => "gt",
            Self::Ge => "ge",
            Self::Get => "get",
            Self::Concat => "concat",
            Self::BufFill => "bufFill",
            Self::BufPush => "bufPush",
            Self::BufExtend => "bufExtend",
        }
    }
}

/// Ternary core built-in operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreTernaryOp {
    /// Sequence sub-slice (`slice`, `Dc 2545`).
    Slice,
    /// Buffer indexed mutation (`bufSet`, `Dc 2584`).
    BufSet,
}

impl CoreTernaryOp {
    /// Canonical camelCase source keyword.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Slice => "slice",
            Self::BufSet => "bufSet",
        }
    }
}

/// Element inside a code block (`do ... ::` or routine body).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockItem {
    /// Expression or statement.
    Expr(Expr),
    /// Preserved comment within the block.
    Comment(Comment),
}

/// StageL 3 expression or local statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// Boolean literal (`true` / `false`).
    Bool {
        /// Boolean value.
        value: bool,
        /// Source location.
        span: Span,
    },
    /// Typed integer literal (`u8:0`, `i32:-1`, etc.).
    Int {
        /// Literal value and width/signedness.
        value: TypedIntLit,
        /// Source location.
        span: Span,
    },
    /// Byte list literal (`b"..."` of type `l/u8`, tokenized as `[data]`).
    Bytes {
        /// Exact payload bytes.
        value: Vec<u8>,
        /// Source location.
        span: Span,
    },
    /// Typed immutable list constructor (`l/T: e1 e2 ::`, `Dc 2578..2579`).
    List {
        /// Element type `T`.
        elem_ty: StageLType,
        /// Element expressions.
        elements: Vec<Expr>,
        /// Source location.
        span: Span,
    },
    /// Anonymous fixed-slot `n`-ary tuple constructor (`tupNew: e1 e2 ::`, `Dc 2215..2216`).
    TupleNew {
        /// Ordered slot expressions.
        elements: Vec<Expr>,
        /// Source location.
        span: Span,
    },
    /// Positional `struct` constructor (`structNew name: e1 e2 ::`, `Dc 2219..2220`).
    StructNew {
        /// Referenced positional `struct` identifier or Dc anchor.
        target: SymbolTarget,
        /// Ordered positional slot expressions (`fields.len() >= 1`).
        fields: Vec<Expr>,
        /// Source location.
        span: Span,
    },
    /// Positional zero-based slot projection (`field EXPR u32:IDX`, `Dc 2547`).
    Field {
        /// Tuple or positional `struct` expression.
        base: Box<Expr>,
        /// Zero-based compile-time slot index.
        index: u32,
        /// Source location.
        span: Span,
    },
    /// Functional positional slot update (`with EXPR u32:IDX NEW_EXPR`, `Dc 2548`).
    With {
        /// Base tuple or positional `struct` expression.
        base: Box<Expr>,
        /// Zero-based compile-time slot index.
        index: u32,
        /// Replacement slot expression.
        value: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Mutable empty buffer constructor (`bufNew T cap`, `Dc 2580`).
    BufNew {
        /// Buffer element type `T`.
        elem_ty: StageLType,
        /// Initial `u64` capacity expression.
        capacity: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Value reference (`TYPE/name` or `/name`, `Dc 2202`).
    ValueRef {
        /// Optional explicit binding type (`Some` for `TYPE/name`, `None` for `/name`).
        ty: Option<StageLType>,
        /// Binding name or Dc reference.
        target: SymbolTarget,
        /// Source location.
        span: Span,
    },
    /// Mutable local declaration (`let TYPE/name [init]`, `Dc 2198`).
    Let {
        /// Variable type.
        ty: StageLType,
        /// Variable identifier.
        name: String,
        /// Optional initial value expression.
        init: Option<Box<Expr>>,
        /// Source location.
        span: Span,
    },
    /// Local constant declaration (`const TYPE/name init`, `Dc 2199`).
    Const {
        /// Optional assigned Dc identity (`None` for locals).
        dc_id: Option<DcChar>,
        /// Constant type.
        ty: StageLType,
        /// Constant identifier.
        name: String,
        /// Initializer expression.
        value: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Mutable local variable assignment (`set TYPE/name expr` or `set /name expr`, `Dc 2201`).
    Set {
        /// Optional explicit variable type (`Some` for `set TYPE/name`, `None` for `set /name`).
        ty: Option<StageLType>,
        /// Target variable identifier.
        name: String,
        /// New value expression.
        value: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Local nested routine declaration (`v/RET/name ...`, `Dc 2203..2204`).
    RoutineDecl(Box<RoutineDecl>),
    /// Direct routine invocation (`name args...`, `Dc 2211..2212`).
    Call {
        /// Called routine identifier or Dc reference.
        target: SymbolTarget,
        /// Positional argument expressions.
        args: Vec<Expr>,
        /// Source location.
        span: Span,
    },
    /// Sequential lexical block (`do ... ::`, `Dc 2213..2214`).
    Block {
        /// Items evaluated sequentially; last expression supplies value.
        stmts: Vec<BlockItem>,
        /// Source location.
        span: Span,
    },
    /// Conditional expression (`if COND THEN ELSE`, `Dc 2448`).
    If {
        /// Boolean condition expression.
        cond: Box<Expr>,
        /// Then-branch expression.
        then_branch: Box<Expr>,
        /// Else-branch expression.
        else_branch: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Pre-test loop (`while COND BODY`, `Dc 2449`).
    While {
        /// Loop condition (`b`).
        cond: Box<Expr>,
        /// Loop body (`void`).
        body: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Immutable sequence iteration (`foreach T/elem LIST BODY`, `Dc 2450`).
    ForEach {
        /// Element binding type.
        elem_ty: StageLType,
        /// Fresh non-shadowing element identifier.
        elem_name: String,
        /// Iterable expression (`l/T`).
        list: Box<Expr>,
        /// Loop body (`void`).
        body: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Break out of nearest enclosing loop (`break`, `Dc 2451`).
    Break {
        /// Source location.
        span: Span,
    },
    /// Continue nearest enclosing loop (`continue`, `Dc 2452`).
    Continue {
        /// Source location.
        span: Span,
    },
    /// Return from enclosing routine (`return [expr]`, `Dc 2453`).
    Return {
        /// Optional returned expression (`None` for `void` routines).
        value: Option<Box<Expr>>,
        /// Source location.
        span: Span,
    },
    /// Unrecoverable trap (`fault`, `Dc 2454`).
    Fault {
        /// Source location.
        span: Span,
    },
    /// Unit no-op expression (`nop`, `Dc 2455`).
    Nop {
        /// Source location.
        span: Span,
    },
    /// Built-in unary operation.
    Unary {
        /// Unary operator.
        op: CoreUnaryOp,
        /// Single operand.
        operand: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Built-in binary operation.
    Binary {
        /// Binary operator.
        op: CoreBinaryOp,
        /// First operand.
        left: Box<Expr>,
        /// Second operand.
        right: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Built-in ternary operation.
    Ternary {
        /// Ternary operator.
        op: CoreTernaryOp,
        /// First operand.
        first: Box<Expr>,
        /// Second operand.
        second: Box<Expr>,
        /// Third operand.
        third: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Explicit numeric conversion (`conv EXPR TARGET_TYPE`, `Dc 2541`).
    Conv {
        /// Value expression to convert.
        expr: Box<Expr>,
        /// Destination numeric type.
        target_ty: StageLType,
        /// Source location.
        span: Span,
    },
    /// Same-width numeric bit reinterpretation (`bitcast EXPR TYPE`, `Dc 2542`).
    Bitcast {
        /// Value expression to reinterpret.
        expr: Box<Expr>,
        /// Destination numeric type of matching bit width.
        target_ty: StageLType,
        /// Source location.
        span: Span,
    },
    /// StageR 3 strict Dcts mixed-mode string literal (`"..."` of type `s`,
    /// `Dc 260..Dc 261`).
    DctsString {
        /// Canonical Dcts string text between quotes (preserves `@...@` escapes).
        raw_dcts: String,
        /// Strictly decoded Document Character sequence.
        value: Vec<DcChar>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 standalone Document Character scalar literal (`dc:<shorthand>`,
    /// encoded as `Dc 260 262 640 263 <atom_dc> 261`).
    DcLit {
        /// Non-surrogate Document Character scalar.
        value: DcChar,
        /// Source location.
        span: Span,
    },
    /// StageR 3 named-field `struct` constructor (`structNew Name: f e ... ::`,
    /// `Dc 2227..2228`).
    NamedStructNew {
        /// Referenced named struct identifier or resolved Dc anchor.
        target: SymbolTarget,
        /// Named field initializers (`(field_name, value_expr, field_span)`).
        fields: Vec<(String, Expr, Span)>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 closed `enum` variant constructor (`Enum:variant [EXPR]`,
    /// `Dc 2233`).
    EnumVariant {
        /// Target enum type reference.
        enum_ty: StructTypeRef,
        /// Variant name.
        variant: String,
        /// Optional variant payload expression.
        payload: Option<Box<Expr>>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 `option/T:none` value (`Dc 2234`).
    OptionNone {
        /// Element type `T`.
        inner_ty: StageLType,
        /// Source location.
        span: Span,
    },
    /// StageR 3 `option/T:some EXPR` value (`Dc 2235`).
    OptionSome {
        /// Element type `T`.
        inner_ty: StageLType,
        /// Present payload expression.
        value: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 `result/T:ok EXPR` value (`Dc 2236`).
    ResultOk {
        /// Success type `T`.
        ok_ty: StageLType,
        /// Success payload expression.
        value: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 `result/T:err EXPR` value (`Dc 2237`).
    ResultErr {
        /// Success type `T`.
        ok_ty: StageLType,
        /// Error expression of type `error`.
        error: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 built-in diagnostic error tag (`error:<tag>`, `Dc 2238`).
    ErrorTag {
        /// Canonical error variant tag.
        tag: StageRErrorTag,
        /// Source location.
        span: Span,
    },
    /// StageR 3 named field projection (`field EXPR fieldName`, `Dc 2549`).
    NamedField {
        /// Named-struct expression.
        base: Box<Expr>,
        /// Field name.
        field: String,
        /// Source location.
        span: Span,
    },
    /// StageR 3 functional named field update (`with EXPR fieldName VALUE`,
    /// `Dc 2550`).
    NamedWith {
        /// Base named-struct expression.
        base: Box<Expr>,
        /// Field name.
        field: String,
        /// Replacement field value expression.
        value: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 nominal wrap (`wrap EXPR TargetType`, `Dc 2551`).
    WrapNominal {
        /// Underlying representation expression.
        expr: Box<Expr>,
        /// Target brand or module-opaque type.
        target_ty: StageLType,
        /// Source location.
        span: Span,
    },
    /// StageR 3 nominal unwrap (`unwrap EXPR`, `Dc 2552`).
    UnwrapNominal {
        /// Nominal brand or module-opaque expression.
        expr: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 shallow exhaustive `match` expression (`Dc 2456..2458`).
    Match {
        /// Scrutinee expression (`enum`, `option/T`, `result/T`, or `error`).
        scrutinee: Box<Expr>,
        /// Ordered explicit `case` arms.
        arms: Vec<MatchArm>,
        /// Optional trailing `else` fallback arm.
        fallback: Option<Box<Expr>>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 trailing `/` (`?`) early-return propagation (`Dc 2459`).
    Propagate {
        /// Fallible sub-expression (`option/T` or `result/T`).
        expr: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 total/string/scalar unary operator.
    StageRUnary {
        /// Operator kind.
        op: StageRUnaryOp,
        /// Operand expression.
        operand: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 total/checked/string binary operator.
    StageRBinary {
        /// Operator kind.
        op: StageRBinaryOp,
        /// First operand expression.
        left: Box<Expr>,
        /// Second operand expression.
        right: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 total ternary operator (`slice`, `bufSet`).
    StageRTernary {
        /// Operator kind.
        op: StageRTernaryOp,
        /// First operand expression.
        first: Box<Expr>,
        /// Second operand expression.
        second: Box<Expr>,
        /// Third operand expression.
        third: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// StageR 3 numeric conversion (`conv EXPR TARGET_TYPE`, `Dc 2562`).
    TotalConv {
        /// Source numeric expression.
        expr: Box<Expr>,
        /// Destination numeric type.
        target_ty: StageLType,
        /// Source location.
        span: Span,
    },
}

impl Expr {
    /// Returns the source span of this expression node.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Bool { span, .. }
            | Self::Int { span, .. }
            | Self::Bytes { span, .. }
            | Self::List { span, .. }
            | Self::TupleNew { span, .. }
            | Self::StructNew { span, .. }
            | Self::Field { span, .. }
            | Self::With { span, .. }
            | Self::BufNew { span, .. }
            | Self::ValueRef { span, .. }
            | Self::Let { span, .. }
            | Self::Const { span, .. }
            | Self::Set { span, .. }
            | Self::Call { span, .. }
            | Self::Block { span, .. }
            | Self::If { span, .. }
            | Self::While { span, .. }
            | Self::ForEach { span, .. }
            | Self::Break { span }
            | Self::Continue { span }
            | Self::Return { span, .. }
            | Self::Fault { span }
            | Self::Nop { span }
            | Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::Ternary { span, .. }
            | Self::Conv { span, .. }
            | Self::Bitcast { span, .. }
            | Self::DctsString { span, .. }
            | Self::DcLit { span, .. }
            | Self::NamedStructNew { span, .. }
            | Self::EnumVariant { span, .. }
            | Self::OptionNone { span, .. }
            | Self::OptionSome { span, .. }
            | Self::ResultOk { span, .. }
            | Self::ResultErr { span, .. }
            | Self::ErrorTag { span, .. }
            | Self::NamedField { span, .. }
            | Self::NamedWith { span, .. }
            | Self::WrapNominal { span, .. }
            | Self::UnwrapNominal { span, .. }
            | Self::Match { span, .. }
            | Self::Propagate { span, .. }
            | Self::StageRUnary { span, .. }
            | Self::StageRBinary { span, .. }
            | Self::StageRTernary { span, .. }
            | Self::TotalConv { span, .. } => *span,
            Self::RoutineDecl(r) => r.span,
        }
    }
}

/// Parses a `DcChar` from a shorthand string (`"256"`, `"f203"`, `"l859001"`).
pub(crate) fn parse_dc_shorthand(raw: &str) -> Result<DcChar> {
    let sh = DcShorthand::parse(raw)
        .with_context(|| format!("Invalid Dc shorthand '{raw}'"))?;
    let dc = match sh {
        DcShorthand::Short(id) => DcChar::from_short(id),
        DcShorthand::Format(fid) => {
            let fid_u32 = u32::try_from(fid).with_context(|| {
                format!("Format shorthand 'f{fid}' exceeds u32 range")
            })?;
            DcChar::from_format(fid_u32)
        }
        DcShorthand::Long(id) => DcChar::from_u128(id),
        DcShorthand::Unicode(cp) => DcChar::from_u128(u128::from(cp)),
        DcShorthand::Local(id) => {
            bail!(
                "Local graph shorthand 'L{id}' is not valid as a persistent Dc anchor"
            );
        }
    };
    Ok(dc)
}

/// Formats a `DcChar` as canonical shorthand (`256`, `f203`, `l859...`).
#[must_use]
pub(crate) fn format_dc_shorthand(dc: DcChar) -> String {
    DcShorthand::from_global_id(dc.to_u128()).to_string()
}
