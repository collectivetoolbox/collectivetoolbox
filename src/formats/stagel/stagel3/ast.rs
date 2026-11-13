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

//! Abstract syntax tree definitions for StageL 3 (`f656` `.sl3`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_formats_dcdata::DcChar;
use ctb_formats_dcdata::DcShorthand;
use std::fmt;

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

/// Static value or control type in StageL 3.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
}

impl StageLType {
    /// Formats this type in canonical StageL 3 notation (`u32`, `l/u8`, etc.).
    #[must_use]
    pub fn display_name(&self) -> String {
        match self {
            Self::B => String::from("b"),
            Self::U8 => String::from("u8"),
            Self::I32 => String::from("i32"),
            Self::U32 => String::from("u32"),
            Self::I64 => String::from("i64"),
            Self::U64 => String::from("u64"),
            Self::Void => String::from("void"),
            Self::Never => String::from("never"),
            Self::List(inner) => format!("l/{}", inner.display_name()),
            Self::Buf(inner) => format!("buf/{}", inner.display_name()),
        }
    }

    /// Returns `true` for fixed-width integer types (`u8`, `i32`, `u32`, `i64`,
    /// `u64`).
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::U8 | Self::I32 | Self::U32 | Self::I64 | Self::U64
        )
    }

    /// Returns `true` for signed integer types (`i32`, `i64`).
    #[must_use]
    pub const fn is_signed_int(&self) -> bool {
        matches!(self, Self::I32 | Self::I64)
    }

    /// Returns `true` for unsigned integer types (`u8`, `u32`, `u64`).
    #[must_use]
    pub const fn is_unsigned_int(&self) -> bool {
        matches!(self, Self::U8 | Self::U32 | Self::U64)
    }

    /// Returns the bit width for integer types.
    #[must_use]
    pub const fn bit_width(&self) -> Option<u32> {
        match self {
            Self::U8 => Some(8),
            Self::I32 | Self::U32 => Some(32),
            Self::I64 | Self::U64 => Some(64),
            _ => None,
        }
    }

    /// Returns `true` if values of this type may be stored as elements of a
    /// `l/T` list or `buf/T` buffer.
    #[must_use]
    pub fn can_be_element(&self) -> bool {
        match self {
            Self::Void | Self::Never | Self::Buf(_) => false,
            Self::List(inner) => inner.can_be_element(),
            Self::B | Self::U8 | Self::I32 | Self::U32 | Self::I64 | Self::U64 => {
                true
            }
        }
    }

    /// Returns `true` if `self` is a `buf/T` type or contains one.
    #[must_use]
    pub fn is_or_contains_buf(&self) -> bool {
        match self {
            Self::Buf(_) => true,
            Self::List(inner) => inner.is_or_contains_buf(),
            _ => false,
        }
    }

    /// Returns `true` if `eq` / `ne` are defined for values of this type.
    #[must_use]
    pub fn supports_eq(&self) -> bool {
        match self {
            Self::B
            | Self::U8
            | Self::I32
            | Self::U32
            | Self::I64
            | Self::U64 => true,
            Self::List(inner) => inner.supports_eq(),
            Self::Void | Self::Never | Self::Buf(_) => false,
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

/// Complete StageL 3 module AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
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

/// Module-level item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopDecl {
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

/// Typed integer literal in StageL 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

impl TypedIntLit {
    /// Returns the StageL 3 static type of this literal.
    #[must_use]
    pub const fn stage_l_type(self) -> StageLType {
        match self {
            Self::U8(_) => StageLType::U8,
            Self::I32(_) => StageLType::I32,
            Self::U32(_) => StageLType::U32,
            Self::I64(_) => StageLType::I64,
            Self::U64(_) => StageLType::U64,
        }
    }

    /// Formats this literal in canonical `type:value` syntax.
    #[must_use]
    pub fn display_literal(self) -> String {
        match self {
            Self::U8(v) => format!("u8:{v}"),
            Self::I32(v) => format!("i32:{v}"),
            Self::U32(v) => format!("u32:{v}"),
            Self::I64(v) => format!("i64:{v}"),
            Self::U64(v) => format!("u64:{v}"),
        }
    }
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
    /// Mutable empty buffer constructor (`bufNew T cap`, `Dc 2580`).
    BufNew {
        /// Buffer element type `T`.
        elem_ty: StageLType,
        /// Initial `u64` capacity expression.
        capacity: Box<Expr>,
        /// Source location.
        span: Span,
    },
    /// Explicit typed value reference (`TYPE/name`, `Dc 2202`).
    ValueRef {
        /// Declared binding type.
        ty: StageLType,
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
    /// Mutable local variable assignment (`set TYPE/name expr`, `Dc 2201`).
    Set {
        /// Expected variable type.
        ty: StageLType,
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
            | Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::Ternary { span, .. }
            | Self::Conv { span, .. }
            | Self::Bitcast { span, .. } => *span,
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
