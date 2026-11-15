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

//! Deterministic reference evaluator and inline `test` runner for validated
//! StageL 3 (`f656` `.sl3`) and StageR 3 (`.sr3`) modules and workspaces.
//!
//! Enforces strict left-to-right operand evaluation (short-circuiting `if`,
//! `and`, and `or`), checked/wrapping/total integer arithmetic, exact `int`
//! and `rat` bounds, mutable local `buf/T` freeze tracking, shared mutable
//! capture cells for nested verbs, bounded compute quotas, and constant-stack
//! tail-call trampolining.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

mod ops;
mod vm;

pub use vm::StageLWorkspaceVm;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use malachite::base::num::logic::traits::SignificantBits;
use malachite::{Integer, Rational};

use super::ast::{
    StageDialect, StageLType, StageRErrorTag, StructTypeRef, TypedIntLit,
};
use super::parser::parse_module_with_dialect;
use super::validator::{
    ModuleInput, ValidatedWorkspace, parse_and_validate_workspace_with_dialect,
    validate_module,
};
use crate::DcChar;

/// Configuration limits for StageL 3 and StageR 3 execution.
#[derive(Debug, Clone, Copy)]
pub struct EvalConfig {
    /// Maximum execution steps allowed before trapping with quota exhaustion.
    pub compute_quota: u64,
    /// Maximum non-tail call depth (tail calls execute in O(1) stack).
    pub max_call_depth: usize,
    /// Maximum significant bit-width permitted for arbitrary-precision `int`
    /// and `rat`.
    pub max_bigint_bits: u64,
    /// Maximum element count permitted for `l/T`, `buf/T`, `s`, or `dcs`.
    pub max_seq_elems: usize,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            compute_quota: 2_000_000,
            max_call_depth: 1_024,
            max_bigint_bits: 1_048_576,
            max_seq_elems: 1_048_576,
        }
    }
}

/// Mutable backing storage for a StageL 3 `buf/T` handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufStorage {
    /// Stored element values.
    pub elems: Vec<Value>,
    /// Set to `true` by `bufFreeze`; subsequent mutations trap.
    pub frozen: bool,
}

/// Runtime value in StageL 3 and StageR 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Boolean (`b`).
    Bool(bool),
    /// Unsigned 8-bit integer (`u8`).
    U8(u8),
    /// Signed 32-bit integer (`i32`).
    I32(i32),
    /// Unsigned 32-bit integer (`u32`).
    U32(u32),
    /// Signed 64-bit integer (`i64`).
    I64(i64),
    /// Unsigned 64-bit integer (`u64`).
    U64(u64),
    /// Signed 128-bit integer (`i128`, StageR 3).
    I128(i128),
    /// Unsigned 128-bit integer (`u128`, StageR 3).
    U128(u128),
    /// Arbitrary-precision signed integer (`int`, StageR 3).
    Int(Integer),
    /// Exact arbitrary-precision rational (`rat`, StageR 3).
    Rat(Rational),
    /// Standalone Document Character scalar (`dc`, StageR 3).
    Dc(DcChar),
    /// Pure Document Character sequence (`dcs`, StageR 3).
    Dcs(Rc<Vec<DcChar>>),
    /// Mixed-mode Document Character + binary string (`s`, StageR 3).
    S(Rc<Vec<DcChar>>),
    /// Structured camelCase semantic error tag (`error`, StageR 3).
    Error(StageRErrorTag),
    /// Tagged optional value (`option/T`, StageR 3).
    Option(Option<Box<Value>>),
    /// Tagged fallible result (`result/T`, StageR 3).
    Result(std::result::Result<Box<Value>, StageRErrorTag>),
    /// Unit completion value (`void`).
    Void,
    /// Immutable value-semantics list (`l/T`).
    List(Rc<Vec<Value>>),
    /// Mutable local builder buffer handle (`buf/T`).
    Buf(Rc<RefCell<BufStorage>>),
    /// Immutable structural `n`-ary tuple (`tup: T0 ... ::`).
    Tuple(Rc<Vec<Value>>),
    /// Immutable nominal positional `struct` (`struct <name> :: T0 ... .`).
    Struct {
        /// Canonical `(namespace, original_name)` identity of the positional
        /// `struct`.
        type_key: (Option<String>, String),
        /// Positional field values.
        fields: Rc<Vec<Value>>,
    },
    /// Immutable StageR 3 named-field `struct` instance.
    NamedStruct {
        /// Canonical `(namespace, original_name)` identity of the named
        /// `struct`.
        type_key: (Option<String>, String),
        /// Field values keyed by field name.
        fields: Rc<BTreeMap<String, Value>>,
    },
    /// Wrapped StageR 3 `brand` or opaque `type` instance.
    NominalWrap {
        /// Canonical `(namespace, original_name)` identity of the brand or
        /// opaque type.
        type_key: (Option<String>, String),
        /// Wrapped underlying value.
        inner: Box<Value>,
    },
    /// StageR 3 closed `enum` variant instance.
    EnumVariant {
        /// Canonical `(namespace, original_name)` identity of the enum type.
        type_key: (Option<String>, String),
        /// Variant constructor name.
        variant: String,
        /// Optional variant payload value.
        payload: Option<Box<Value>>,
    },
}

impl Value {
    pub(super) fn from_typed_int(
        lit: &TypedIntLit,
        max_bigint_bits: u64,
    ) -> std::result::Result<Self, String> {
        match lit {
            TypedIntLit::U8(v) => Ok(Self::U8(*v)),
            TypedIntLit::I32(v) => Ok(Self::I32(*v)),
            TypedIntLit::U32(v) => Ok(Self::U32(*v)),
            TypedIntLit::I64(v) => Ok(Self::I64(*v)),
            TypedIntLit::U64(v) => Ok(Self::U64(*v)),
            TypedIntLit::I128(v) => Ok(Self::I128(*v)),
            TypedIntLit::U128(v) => Ok(Self::U128(*v)),
            TypedIntLit::Int(v) => {
                if v.significant_bits() > max_bigint_bits {
                    return Err(format!(
                        "arbitrary-precision int literal exceeds max_bigint_bits ({max_bigint_bits})"
                    ));
                }
                Ok(Self::Int(v.clone()))
            }
        }
    }

    pub(super) fn as_u64_index(&self, context_msg: &str) -> Result<usize> {
        let Self::U64(n) = self else {
            bail!("{context_msg}: expected u64 index/count, found {self:?}");
        };
        usize::try_from(*n).with_context(|| {
            format!("{context_msg}: u64 index {n} exceeds host usize range")
        })
    }

    pub(super) fn matches_stagel_type(&self, ty: &StageLType) -> bool {
        match (self, ty) {
            (Self::Bool(_), StageLType::B)
            | (Self::U8(_), StageLType::U8)
            | (Self::I32(_), StageLType::I32)
            | (Self::U32(_), StageLType::U32)
            | (Self::I64(_), StageLType::I64)
            | (Self::U64(_), StageLType::U64)
            | (Self::I128(_), StageLType::I128)
            | (Self::U128(_), StageLType::U128)
            | (Self::Int(_), StageLType::Int)
            | (Self::Rat(_), StageLType::Rat)
            | (Self::Dc(_), StageLType::Dc)
            | (Self::Dcs(_), StageLType::Dcs)
            | (Self::S(_), StageLType::S)
            | (Self::Error(_), StageLType::Error)
            | (Self::Void, StageLType::Void) => true,
            (Self::Option(None), StageLType::Option(_)) => true,
            (Self::Option(Some(inner)), StageLType::Option(inner_ty)) => {
                inner.matches_stagel_type(inner_ty)
            }
            (Self::Result(Err(_)), StageLType::Result(_)) => true,
            (Self::Result(Ok(inner)), StageLType::Result(inner_ty)) => {
                inner.matches_stagel_type(inner_ty)
            }
            (Self::List(elems), StageLType::List(elem_ty)) => elems
                .first()
                .is_none_or(|first| first.matches_stagel_type(elem_ty)),
            (Self::Buf(buf), StageLType::Buf(elem_ty)) => buf
                .borrow()
                .elems
                .first()
                .is_none_or(|first| first.matches_stagel_type(elem_ty)),
            (Self::Tuple(elems), StageLType::Tuple(slot_tys)) => {
                elems.len() == slot_tys.len()
                    && elems
                        .iter()
                        .zip(slot_tys)
                        .all(|(v, t)| v.matches_stagel_type(t))
            }
            (
                Self::Struct { type_key, .. }
                | Self::NamedStruct { type_key, .. },
                StageLType::Struct(StructTypeRef::Canonical {
                    namespace,
                    name,
                }),
            )
            | (
                Self::NominalWrap { type_key, .. }
                | Self::EnumVariant { type_key, .. },
                StageLType::Nominal(StructTypeRef::Canonical {
                    namespace,
                    name,
                }),
            ) => &type_key.0 == namespace && &type_key.1 == name,
            _ => false,
        }
    }
}

/// Result of running a single StageL 3 / StageR 3 routine invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutineOutcome {
    /// Returned normally with a concrete value (or `Value::Void`).
    Returned(Value),
    /// Deterministically trapped (`fault`, arithmetic overflow, bounds, quota).
    Trapped(String),
}

/// Outcome for one inline `test` case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCaseReport {
    /// Namespace of the tested routine (if any).
    pub namespace: Option<String>,
    /// Routine name.
    pub routine_name: String,
    /// Optional `test` block label.
    pub block_label: Option<String>,
    /// 0-based index of the case within the test block.
    pub case_index: usize,
    /// Whether this test case passed.
    pub passed: bool,
    /// Diagnostic detail on failure (or concise note on pass).
    pub detail: String,
}

/// Aggregated report across all inline `test` cases executed.
#[derive(Debug, Clone, Default)]
pub struct TestSuiteReport {
    /// Individual test case outcomes.
    pub cases: Vec<TestCaseReport>,
    /// Number of passed cases.
    pub passed_count: usize,
    /// Number of failed cases.
    pub failed_count: usize,
}

impl TestSuiteReport {
    /// Returns an error summarizing any failed inline tests, or the total
    /// passing count when all tests passed.
    pub fn ensure_all_passed(&self) -> Result<usize> {
        if self.failed_count > 0 {
            let failures: Vec<String> = self
                .cases
                .iter()
                .filter(|c| !c.passed)
                .map(|c| {
                    // Reason for fallback: Anonymous modules display as '<module>'.
                    let ns = c.namespace.as_deref().unwrap_or("<module>");
                    // Reason for fallback: Unlabeled test blocks display as 'unlabeled'.
                    let label = c.block_label.as_deref().unwrap_or("unlabeled");
                    format!(
                        "{ns}::{} [{label} #{}]: {}",
                        c.routine_name, c.case_index, c.detail
                    )
                })
                .collect();
            bail!(
                "{} StageL 3 / StageR 3 inline test case(s) failed:\n{}",
                self.failed_count,
                failures.join("\n")
            );
        }
        Ok(self.passed_count)
    }
}

/// Parses, validates, and runs all inline `test` blocks of a single module
/// under the requested `dialect` and [`EvalConfig`] limits.
pub fn run_module_inline_tests_with_dialect(
    source: &str,
    dialect: StageDialect,
    config: EvalConfig,
) -> Result<TestSuiteReport> {
    let parsed = parse_module_with_dialect(source, dialect)?;
    let validated = validate_module(&parsed)?;
    let ws = ValidatedWorkspace {
        modules: vec![validated],
    };
    run_validated_workspace_inline_tests_with_config(&ws, config)
}

/// Parses, validates, and runs all inline `test` blocks of a single `.sl3`
/// source module.
pub fn run_module_inline_tests(source: &str) -> Result<TestSuiteReport> {
    run_module_inline_tests_with_dialect(
        source,
        StageDialect::StageL3,
        EvalConfig::default(),
    )
}

/// Parses, validates, and runs all inline `test` blocks of a single `.sl3`
/// source module with custom compute quota and depth configuration.
pub fn run_module_inline_tests_with_config(
    source: &str,
    config: EvalConfig,
) -> Result<TestSuiteReport> {
    run_module_inline_tests_with_dialect(source, StageDialect::StageL3, config)
}

/// Parses, validates, and runs all inline `test` blocks inside a standalone
/// StageR 3 `.sr3` module.
pub fn run_sr3_module_inline_tests(source: &str) -> Result<TestSuiteReport> {
    run_module_inline_tests_with_dialect(
        source,
        StageDialect::StageR3,
        EvalConfig::default(),
    )
}

/// Parses, validates, and runs all inline `test` blocks inside a standalone
/// StageR 3 `.sr3` module using explicit [`EvalConfig`] limits.
pub fn run_sr3_module_inline_tests_with_config(
    source: &str,
    config: EvalConfig,
) -> Result<TestSuiteReport> {
    run_module_inline_tests_with_dialect(source, StageDialect::StageR3, config)
}

/// Parses, validates, and runs all inline `test` blocks across a multi-module
/// workspace using `default_dialect` when a module has no explicit `.sl3`/`.sr3`
/// path suffix and explicit [`EvalConfig`] limits.
pub fn run_workspace_inline_tests_with_dialect(
    inputs: &[ModuleInput<'_>],
    default_dialect: StageDialect,
    config: EvalConfig,
) -> Result<TestSuiteReport> {
    let ws = parse_and_validate_workspace_with_dialect(inputs, default_dialect)?;
    run_validated_workspace_inline_tests_with_config(&ws, config)
}

/// Parses, validates (with PSR-4 `ns` and `use` DAG), and runs all inline
/// `test` blocks across a multi-module workspace.
pub fn run_workspace_inline_tests(
    inputs: &[ModuleInput<'_>],
) -> Result<TestSuiteReport> {
    run_workspace_inline_tests_with_dialect(
        inputs,
        StageDialect::StageL3,
        EvalConfig::default(),
    )
}

/// Parses, validates, and runs all inline `test` blocks across a StageR 3
/// `.sr3` workspace.
pub fn run_sr3_workspace_inline_tests(
    inputs: &[ModuleInput<'_>],
) -> Result<TestSuiteReport> {
    run_workspace_inline_tests_with_dialect(
        inputs,
        StageDialect::StageR3,
        EvalConfig::default(),
    )
}

/// Parses, validates, and runs all inline `test` blocks across a StageR 3
/// `.sr3` workspace using explicit [`EvalConfig`] limits.
pub fn run_sr3_workspace_inline_tests_with_config(
    inputs: &[ModuleInput<'_>],
    config: EvalConfig,
) -> Result<TestSuiteReport> {
    run_workspace_inline_tests_with_dialect(
        inputs,
        StageDialect::StageR3,
        config,
    )
}

/// Runs all inline `test` blocks across an already-validated multi-module
/// [`ValidatedWorkspace`].
pub fn run_validated_workspace_inline_tests(
    ws: &ValidatedWorkspace,
) -> Result<TestSuiteReport> {
    run_validated_workspace_inline_tests_with_config(ws, EvalConfig::default())
}

/// Runs all inline `test` blocks across an already-validated multi-module
/// [`ValidatedWorkspace`] using explicit [`EvalConfig`] limits.
pub fn run_validated_workspace_inline_tests_with_config(
    ws: &ValidatedWorkspace,
    config: EvalConfig,
) -> Result<TestSuiteReport> {
    let vm = StageLWorkspaceVm::from_validated_workspace(ws, config)?;
    vm.run_all_tests(ws)
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
    use super::super::ast::{Span, StageRErrorTag, TernaryOp};
    use super::super::validator::{ModuleInput, parse_and_validate_workspace};
    use super::ops::eval_ternary;
    use super::vm::EvalSignal;

    #[crate::ctb_test]
    fn test_eval_arithmetic_bitwise_conv_bitcast_and_traps() {
        let src = r#"
v/u32/testArith
    test checked
        in.
        out u32:42.
    let u32/prod mul u32:6 u32:7.
    u32/prod.

v/u32/trapAddOverflow
    test overflow
        in.
        out fault.
    add u32:4294967295 u32:1.

v/i32/trapNegMin
    test negMin
        in.
        out fault.
    neg i32:-2147483648.

v/i32/trapDivMinMinusOne
    test divOverflow
        in.
        out fault.
    div i32:-2147483648 i32:-1.

v/i32/trapDivZero
    test divZero
        in.
        out fault.
    div i32:5 i32:0.

v/i32/trapRemMinMinusOne
    test remOverflow
        in.
        out fault.
    rem i32:-2147483648 i32:-1.

v/u32/testWrappingAndBitops
    test wraps
        in.
        out u32:80.
    let u32/wrapped wrapAdd u32:4294967295 u32:3.
    let u32/wrapped2 wrapSub u32/wrapped u32:5.
    : w = 2, w2 = u32::MAX - 2 = 0xFFFFFFFD -> popcnt = 31
    let u32/pop popcnt u32/wrapped2.
    : ctz(8) = 3, clz(16) = 27
    let u32/tz ctz u32:8.
    let u32/lz clz u32:16.
    : rotl(1, 4) = 16, rotr(16, 3) = 2
    let u32/rotLeft rotl u32:1 u32:4.
    let u32/rotRight rotr u32/rotLeft u32:3.
    : 31 + 3 + 27 + 16 + 2 + 1 = 80
    add add u32/pop u32/tz add u32/lz add u32/rotLeft add u32/rotRight u32:1.

v/i32/testSignedShiftAndBitcast
    test shifted
        in.
        out i32:-4.
    let i32/negVal i32:-16.
    : arithmetic right shift by 2 -> -4
    let i32/shiftedVal shr i32/negVal u32:2.
    : Verify bitcast round-trip preserves bit pattern
    let u32/bits bitcast i32/shiftedVal u32.
    bitcast u32/bits i32.

v/i32/trapShiftWidth
    test shiftWidth
        in.
        out fault.
    shl i32:1 u32:32.

v/u8/testConvSuccessAndTrap u32/n
    test ok
        in u32:200
        out u8:200.
    test hi
        in u32:256
        out fault.
    conv u32/n u8.
"#;
        let report = run_module_inline_tests(src).unwrap();
        let passed = report.ensure_all_passed().unwrap();
        assert_eq!(passed, 11);
    }

    #[crate::ctb_test]
    fn test_eval_buffers_captures_and_foreach_snapshots() {
        let src = r#"
v/l/u8/buildAndFreeze u8/fillByte
    test ok
        in u8:65
        out b"AAA!".
    let buf/u8/buf1 bufFill u8/fillByte u64:4.
    bufSet buf/u8/buf1 u64:3 u8:33.
    bufFreeze buf/u8/buf1.

v/u64/trapPostFreeze
    test frozenWrite
        in.
        out fault.
    let buf/u8/buf2 bufNew u8 u64:2.
    let l/u8/frozen bufFreeze buf/u8/buf2.
    bufSet buf/u8/buf2 u64:0 u8:99.
    len l/u8/frozen.

v/u32/sharedCaptureMutation
    test shared
        in.
        out u32:15.
    let u32/total u32:0.
    const v/void/addToTotal u32/delta :: set u32/total add u32/total u32/delta.
    addToTotal u32:4.
    addToTotal u32:11.
    u32/total.

v/u32/foreachSnapshot
    test nestedLoops
        in.
        out u32:20.
    let l/u32/nums l/u32: u32:1 u32:2 u32:3 ::.
    let u32/sum u32:0.
    let buf/u32/dynBuf bufNew u32 u64:0.
    bufPush buf/u32/dynBuf u32:4.
    bufExtend buf/u32/dynBuf l/u32/nums.
    : dyn has [4, 1, 2, 3], frozen snapshot has sum 10
    let l/u32/snap bufFreeze buf/u32/dynBuf.
    let buf/u32/otherBuf bufNew u32 u64:0.
    foreach u32/itemX l/u32/snap do
        : Mutating another buffer while iterating snap does not disturb snap
        bufPush buf/u32/otherBuf u32:99.
        set u32/sum add u32/sum u32/itemX.
    ::.
    : Check snap again equals 10 -> total 20
    foreach u32/itemY l/u32/snap do
        set u32/sum add u32/sum u32/itemY.
    ::.
    u32/sum.
"#;
        let report = run_module_inline_tests(src).unwrap();
        let passed = report.ensure_all_passed().unwrap();
        assert_eq!(passed, 4);
    }

    #[crate::ctb_test]
    fn test_cross_module_use_alias_dc_import_and_tail_call_trampoline() {
        let inputs = [
            ModuleInput {
                source_path: Some("math/parity.sl3"),
                source: r#"ns math/parity.

pub u32/zeroConst u32:0.
pub dc/257 u32/oneConst u32:1.

pub v/b/isEven u32/n
    if eq u32/n u32/zeroConst true isOdd sub u32/n u32/oneConst.

pub v/b/isOdd u32/n
    if eq u32/n u32/zeroConst false isEven sub u32/n u32/oneConst.

pub dc/256 v/u32/incByOne u32/argVal :: add u32/argVal u32/oneConst.

: @dc l9000100
pub v/u32/doubleIt u32/argVal :: mul u32/argVal u32:2.
"#,
            },
            ModuleInput {
                source_path: Some("app/runner.sl3"),
                source: r#"ns app/runner.
use math/parity: isEven doubleIt as timesTwo dc/256 dc/257 ::.

pub v/b/checkDeepTail
    test even50k
        in.
        out true.
    isEven u32:50000.

pub v/u32/useImports u32/inputVal
    test imported
        in u32:6
        out u32:13.
    : Uses aliased 'timesTwo' AND direct Dc routine call 'dc/256' (incByOne)
    dc/256 timesTwo u32/inputVal.
"#,
            },
        ];

        let ws = parse_and_validate_workspace(&inputs)
            .expect("workspace parse and validate");

        // Verify 50,000 mutual tail calls run cleanly with max_call_depth = 32!
        let vm = StageLWorkspaceVm::from_validated_workspace(
            &ws,
            EvalConfig {
                compute_quota: 5_000_000,
                max_call_depth: 32,
                ..EvalConfig::default()
            },
        )
        .expect("construct workspace VM");
        let report = vm.run_all_tests(&ws).expect("workspace inline tests");
        let passed = report.ensure_all_passed().unwrap();
        assert_eq!(passed, 2);

        // Also verify compute quota exhaustion produces a clean Fault (caught by 'out fault')
        let quota_src = r#"
v/void/spin :: spin.
v/u32/testQuota
    test hitsQuota
        in.
        out fault.
    spin.
    u32:0.
"#;
        let qreport = run_module_inline_tests_with_config(
            quota_src,
            EvalConfig {
                compute_quota: 250,
                max_call_depth: 32,
                ..EvalConfig::default()
            },
        )
        .unwrap();
        assert_eq!(qreport.ensure_all_passed().unwrap(), 1);
    }

    #[crate::ctb_test]
    fn test_tuples_and_recursive_structs_eval() {
        let inputs = [
            ModuleInput {
                source_path: Some("geo/types.sl3"),
                source: r#"ns geo/types.

pub struct point :: i32 i32.
pub struct tree :: point l/tree.

pub point/ORIGIN structNew point: i32:0 i32:0 ::.

pub v/i32/sumTree tree/node
    test leaf
        in structNew tree: structNew point: i32:3 i32:4 :: l/tree: :: ::
        out i32:7.
    let i32/acc add field field /node u32:0 u32:0 field field /node u32:0 u32:1.
    foreach tree/child field /node u32:1 do
        set /acc add /acc sumTree /child.
    ::.
    /acc.
"#,
            },
            ModuleInput {
                source_path: Some("geo/runner.sl3"),
                source: r#"ns geo/runner.
use geo/types: point as pt tree ORIGIN sumTree ::.

pub v/b/verifyAll
    test fullCheck
        in.
        out true.
    const tup: ::/u1 tupNew: ::.
    const tup: ::/u2 tupNew: ::.
    const tup: i32 ::/one tupNew: i32:42 ::.
    const tup: i32 b l/u8 ::/tri tupNew: i32:10 false b"ab" ::.
    const tup: i32 b l/u8 ::/triUpdated with /tri u32:1 true.
    : Original 'tri' must remain unchanged (value semantics)
    if field /tri u32:1 do return false. :: nop.
    if not field /triUpdated u32:1 do return false. :: nop.

    const pt/p1 with /ORIGIN u32:0 i32:5.
    const pt/p2 with /p1 u32:1 i32:8.
    if ne field /ORIGIN u32:0 i32:0 do return false. :: nop.

    const tree/leafA structNew tree: /p1 l/tree: :: ::.
    const tree/leafB structNew tree: /p2 l/tree: :: ::.
    const tree/root1 structNew tree: /ORIGIN l/tree: /leafA /leafB :: ::.
    const tree/root2 structNew tree: structNew pt: i32:0 i32:0 :: l/tree: structNew tree: structNew pt: i32:5 i32:0 :: l/tree: :: :: structNew tree: structNew pt: i32:5 i32:8 :: l/tree: :: :: :: ::.
    and eq /u1 /u2 and eq field /one u32:0 i32:42 and eq /root1 /root2 eq sumTree /root1 i32:18.
"#,
            },
        ];

        let report = run_workspace_inline_tests(&inputs).expect("eval workspace");
        assert_eq!(report.ensure_all_passed().unwrap(), 2);
    }

    #[crate::ctb_test]
    fn test_stager_eval_i128_u128_int_rat_and_shifts() -> Result<()> {
        let src = r#"ns eval/nums.

pub v/b/checkWideLimits
    test checks i128 u128 overflow and mod truncRem MIN -1
        in.
        out true
    const i128/min i128:-170141183460469231731687303715884105728.
    const b/ok1 eq neg /min result/i128:err error:overflow.
    const b/ok2 eq div /min i128:-1 result/i128:err error:overflow.
    const b/ok3 eq mod /min i128:-1 result/i128:ok i128:0.
    const b/ok4 eq truncRem /min i128:-1 result/i128:ok i128:0.
    const b/ok5 eq sub u128:0 u128:1 result/u128:err error:overflow.
    const b/ok6 eq shl u32:1 u32:31 result/u32:ok u32:2147483648.
    const b/ok7 eq shl u32:1 u32:32 result/u32:err error:overflow.
    const b/ok8 eq shr u128:8 u32:127 result/u128:ok u128:0.
    const b/ok9 eq shr u128:8 u32:128 result/u128:err error:overflow.
    and /ok1 and /ok2 and /ok3 and /ok4 and /ok5 and /ok6 and /ok7 and /ok8 /ok9.

pub v/result/b/checkBigIntAndRat
    test checks arbitrary int and canonical rational arithmetic
        in.
        out result/b:ok true
    const int/big mul int:100000000000000000000000000000 int:200000000000000000000000000000.
    const b/okInt and
        eq /big int:20000000000000000000000000000000000000000000000000000000000
        eq div int:5 int:0 result/int:err error:divisionByZero.
    const rat/r1 makeRat? int:6 int:-4.
    const rat/r2 makeRat? int:-3 int:2.
    const rat/r3 add makeRat? int:1 int:3 makeRat? int:1 int:6.
    const rat/half makeRat? int:1 int:2.
    const b/badDen eq makeRat int:1 int:0 result/rat:err error:divisionByZero.
    result/b:ok and /okInt and eq /r1 /r2 and eq /r3 /half /badDen.
"#;
        let report = run_sr3_module_inline_tests(src)?;
        assert_eq!(report.ensure_all_passed()?, 2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager_eval_euclidean_mod_vs_trunc_rem_and_exact_conv()
    -> Result<()> {
        let src = r#"ns eval/modconv.

pub v/result/b/checkModAndTruncRem
    test checks euclidean mod vs truncRem across signed operands
        in.
        out result/b:ok true
    const i32/m1 mod? i32:-7 i32:3.
    const i32/m2 mod? i32:-7 i32:-3.
    const i32/t1 truncRem? i32:-7 i32:3.
    const i32/t2 truncRem? i32:-7 i32:-3.
    const i32/m0 mod? i32:5 i32:-1.
    const i32/t0 truncRem? i32:5 i32:-1.
    const b/okMod and eq /m1 i32:2 and eq /m2 i32:2 eq /m0 i32:0.
    const b/okRem and eq /t1 i32:-1 and eq /t2 i32:-1 eq /t0 i32:0.
    result/b:ok and /okMod /okRem.

pub v/result/b/checkExactConv
    test checks exact value-preserving conv rules
        in.
        out result/b:ok true
    const rat/exactIntRat makeRat? int:6 int:2.
    const rat/nonIntRat makeRat? int:5 int:2.
    const int/fromRat conv? /exactIntRat int.
    const b/okExact eq /fromRat int:3.
    const b/failNonInt eq conv /nonIntRat int result/int:err error:overflow.
    const b/okNarrow eq conv int:255 u8 result/u8:ok u8:255.
    const b/failRange eq conv int:256 u8 result/u8:err error:overflow.
    const rat/wide conv int:-42 rat.
    const b/okWide eq /wide makeRat? int:-42 int:1.
    result/b:ok and /okExact and /failNonInt and /okNarrow and /failRange /okWide.
"#;
        let report = run_sr3_module_inline_tests(src)?;
        assert_eq!(report.ensure_all_passed()?, 2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager_eval_dc_dcs_s_explode_implode_get_slice_and_bufset()
    -> Result<()> {
        let src = r#"ns eval/strings.

pub v/result/b/checkDcAndStrings
    test checks dcFromU128 dcsFromS sFromDcs byteLen and explode implode
        in.
        out result/b:ok true
    const dc/cu100 dcFromU128? u128:256.
    const dc/c256 dcFromU128? u128:1114368.
    const b/okDc and eq /cu100 dc:u100 and eq /c256 dc:256 eq u128FromDc /c256 u128:1114368.
    const b/badSurrogate eq dcFromU128 u128:55296 result/dc:err error:invalidDc.

    const dcs/pure dcsFromS? "hi @256@".
    const b/okLen and eq len /pure u64:4 eq get /pure u64:3 option/dc:some dc:256.
    const b/okOob and eq get /pure u64:9 option/dc:none eq slice /pure u64:3 u64:1 option/dcs:none.
    const dcs/sl slice? /pure u64:1 u64:3.
    const s/back sFromDcs /sl.
    const b/okRound eq /back "i ".

    : Embedded valid Base64 binary run Dc 203..204 (21-byte header + 3 payload bytes = 24 bytes)
    const s/withBin "pre:@203@@127@@127@@127@@127@@204@:post".
    const b/binRejected eq dcsFromS /withBin result/dcs:err error:invalidEncoding.
    const u64/blen byteLen /withBin.
    const l/s/parts explode ":" /withBin.
    const s/rejoined implode ":" /parts.
    const l/s/chars explode "" "a@256@@203@@127@@127@@127@@127@@204@".
    const b/okBinSplit and eq /rejoined /withBin and eq len /parts u64:3 and eq len /chars u64:3 eq get /parts u64:1 option/s:some "@203@@127@@127@@127@@127@@204@".

    const l/i32/ints l/i32: i32:10 i32:20 i32:30 ::.
    const l/l/i32/iParts explode l/i32: i32:20 :: /ints.
    const b/okIntList and eq implode l/i32: i32:20 :: /iParts /ints eq len /iParts u64:2.

    let buf/i32/mbuf bufFill i32:0 u64:2.
    const b/okSet0 eq bufSet /mbuf u64:0 i32:11 result/void:ok nop.
    const b/okMatchSet
        match bufSet /mbuf u64:1 i32:99
            case ok true.
            case err error/_ev false.
        ::.
    const b/badSet eq bufSet /mbuf u64:5 i32:1 result/void:err error:outOfBounds.
    const l/i32/frozen bufFreeze /mbuf.
    const b/okFrozen and eq get /frozen u64:0 option/i32:some i32:11 eq get /frozen u64:1 option/i32:some i32:99.

    result/b:ok and /okDc and /badSurrogate and /okLen and /okOob and /okRound and /binRejected and gt /blen u64:24 and /okBinSplit and /okIntList and /okSet0 and /okMatchSet and /badSet /okFrozen.
"#;
        let report = run_sr3_module_inline_tests(src)?;
        assert_eq!(report.ensure_all_passed()?, 1);

        // Also test runtime `err error:frozenBuffer` directly on a frozen Value::Buf handle.
        let frozen_buf = Value::Buf(std::rc::Rc::new(std::cell::RefCell::new(
            crate::stagel3::eval::BufStorage {
                elems: vec![Value::I32(1)],
                frozen: true,
            },
        )));
        let sig = eval_ternary(
            TernaryOp::TotalBufSet,
            frozen_buf,
            Value::U64(0),
            Value::I32(7),
            Span { line: 1, col: 1 },
        )?;
        assert!(matches!(
            sig,
            EvalSignal::Value(Value::Result(Err(StageRErrorTag::FrozenBuffer)))
        ));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager_eval_propagate_match_tail_trampoline_and_resource_traps()
    -> Result<()> {
        let src = r#"ns eval/control.

enum step
    int/done.
    int/next.

v/option/i32/optPropagate option/i32/x
    const i32/unw /x?.
    option/i32:some wrapAdd /unw i32:1.

v/result/i32/optToRes option/i32/x
    const i32/unw /x/.
    result/i32:ok /unw.

v/int/countRec step/st int/acc
    match /st
        case done int/fin /fin.
        case next int/n if le /n int:0
            countRec step:done /acc int:0
            countRec step:next sub /n int:1 add /acc /n.
    ::.

pub v/int/tailMatchSum int/n
    test deep match tail call with tight max_call_depth
        in int:500
        out int:125250
    countRec step:next /n int:0.

pub v/b/checkPropagation
    test verifies option and result propagation conversions
        in.
        out true
    const b/o1 eq optPropagate option/i32:none option/i32:none.
    const b/o2 eq optToRes option/i32:none result/i32:err error:unexpectedNone.
    const b/o3 eq optToRes option/i32:some i32:7 result/i32:ok i32:7.
    and /o1 and /o2 /o3.
"#;
        let report = run_sr3_module_inline_tests_with_config(
            src,
            EvalConfig {
                compute_quota: 2_000_000,
                max_call_depth: 16,
                ..EvalConfig::default()
            },
        )?;
        assert_eq!(report.ensure_all_passed()?, 2);

        // Verify `max_bigint_bits` and `max_seq_elems` cleanly trap (`EvalSignal::Fault`).
        let big_trap = r#"ns eval/bigtrap.
pub v/int/boom
    test bigint trap
        in.
        out int:0
    mul int:1000000000000 int:1000000000000.
"#;
        let big_rep = run_sr3_module_inline_tests_with_config(
            big_trap,
            EvalConfig {
                max_bigint_bits: 32,
                ..EvalConfig::default()
            },
        )?;
        let big_err = big_rep.ensure_all_passed().unwrap_err().to_string();
        assert!(big_err.contains("max_bigint_bits"), "got: {big_err}");

        let seq_trap = r#"ns eval/seqtrap.
pub v/u64/boom
    test seq trap
        in.
        out u64:0
    let buf/u8/mbuf bufFill u8:1 u64:16.
    len bufFreeze /mbuf.
"#;
        let seq_rep = run_sr3_workspace_inline_tests_with_config(
            &[ModuleInput {
                source_path: Some("eval/seqtrap.sr3"),
                source: seq_trap,
            }],
            EvalConfig {
                max_seq_elems: 4,
                ..EvalConfig::default()
            },
        )?;
        let seq_err = seq_rep.ensure_all_passed().unwrap_err().to_string();
        assert!(
            seq_err.contains("max_seq_elems") || seq_err.contains("capacity"),
            "got: {seq_err}"
        );
        let _ = StageDialect::StageR3;
        Ok(())
    }

    #[crate::ctb_test]
    fn stager_eval_get_on_buf_in_and_out_of_bounds() -> Result<()> {
        let src = r#"ns eval/bufget.
pub v/tup: i32 b ::/getBufAndFreeze
    test verifies StageR get on live buf/i32 before bufFreeze
        in.
        out tupNew: i32:42 true ::
    let buf/i32/b bufNew i32 u64:2.
    bufPush /b i32:42.
    const option/i32/hit get /b u64:0.
    const option/i32/miss get /b u64:5.
    const l/i32/out bufFreeze /b.
    const u64/_n len /out.
    tupNew: unwrapOr /hit i32:0 eq /miss option/i32:none ::.
"#;
        let report = run_sr3_module_inline_tests(src)?;
        assert_eq!(report.ensure_all_passed()?, 1);
        Ok(())
    }
}
