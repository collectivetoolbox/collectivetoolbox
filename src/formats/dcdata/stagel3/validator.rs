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

//! Static semantic, totality, definite-assignment, linear-buffer, cycle, and
//! cross-module `use` / PSR-4 validation for StageL 3 (`.sl3`) and StageR 3
//! (`.sr3`) modules.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

mod consts_and_flow;
mod imports_and_decls;
mod ops;
mod stmts_and_calls;
mod types;
mod workspace;

pub use imports_and_decls::{
    validate_module, validate_module_with_registry,
    validate_module_with_registry_and_dc_imports,
    validate_module_with_registry_and_dc_imports_and_options,
};
pub(crate) use types::is_stager_widening_conv;
pub use workspace::{
    parse_and_validate_stager_workspace, parse_and_validate_workspace,
    parse_and_validate_workspace_with_dialect,
};
pub(crate) use workspace::{
    build_stager_import_context, dialect_for_source_path,
    is_stage2_or_sr3_path, top_decl_uses_stager, verify_psr4_path,
};

use super::ast::{
    Module, RoutineDecl, Span, StageLType, StructTypeRef, TopValueDecl,
};
use crate::DcChar;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Source module paired with an optional repository-relative PSR-4 file path.
#[derive(Debug, Clone, Copy)]
pub struct ModuleInput<'a> {
    /// Optional path (e.g. `"std/dctf/codec.sl3"`) checked against `ns`.
    pub source_path: Option<&'a str>,
    /// Raw `.sl3` or `.sr3` source text.
    pub source: &'a str,
}

/// Signature of a validated routine (`v/RET/name PARAM...`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineSig {
    /// Return type (`void` allowed; `never` allowed).
    pub return_type: StageLType,
    /// Ordered parameter types.
    pub params: Vec<StageLType>,
    /// Export visibility.
    pub is_pub: bool,
    /// Optional Dc anchor.
    pub dc_id: Option<DcChar>,
}

/// Unique signature key for a top-level routine within a module (`name` + exact
/// parameter types).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoutineKey {
    /// Routine programmatic identifier in its declaring module.
    pub name: String,
    /// Ordered formal parameter types.
    pub params: Vec<StageLType>,
}

impl RoutineKey {
    /// Builds a [`RoutineKey`] from a [`RoutineDecl`].
    #[must_use]
    pub fn from_decl(r: &RoutineDecl) -> Self {
        Self {
            name: r.name.clone(),
            params: r.params.iter().map(|p| p.ty.clone()).collect(),
        }
    }
}

/// Unique key for a top-level constant value within a module (`name` +
/// [`StageLType`]).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueKey {
    /// Constant programmatic identifier in its declaring module.
    pub name: String,
    /// Declared [`StageLType`].
    pub ty: StageLType,
}

impl ValueKey {
    /// Builds a [`ValueKey`] from a [`TopValueDecl`].
    #[must_use]
    pub fn from_decl(v: &TopValueDecl) -> Self {
        Self {
            name: v.name.clone(),
            ty: v.ty.clone(),
        }
    }
}

/// Statically resolved top-level or imported routine target for an
/// `Expr::Call` site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCallTarget {
    /// `None` if defined in the current module; `Some(ns)` if imported from
    /// `ns`.
    pub namespace: Option<String>,
    /// Original routine identifier in the declaring module.
    pub routine_name: String,
    /// Concrete signature of the selected routine overload.
    pub sig: RoutineSig,
}

impl ResolvedCallTarget {
    /// Returns the [`RoutineKey`] in the declaring module.
    #[must_use]
    pub fn routine_key(&self) -> RoutineKey {
        RoutineKey {
            name: self.routine_name.clone(),
            params: self.sig.params.clone(),
        }
    }
}

/// Statically resolved top-level or imported constant target for an
/// `Expr::ValueRef` site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedValueTarget {
    /// `None` if defined in the current module; `Some(ns)` if imported from
    /// `ns`.
    pub namespace: Option<String>,
    /// Original constant identifier in the declaring module.
    pub original_name: String,
    /// Static constant type.
    pub ty: StageLType,
    /// Optional Dc identity on the target constant.
    pub dc_id: Option<DcChar>,
}

impl ResolvedValueTarget {
    /// Returns the [`ValueKey`] in the declaring module.
    #[must_use]
    pub fn value_key(&self) -> ValueKey {
        ValueKey {
            name: self.original_name.clone(),
            ty: self.ty.clone(),
        }
    }
}

/// Validated positional `struct` definition (`Dc 2217`..`Dc 2218`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDef {
    /// Defining namespace path (`None` if declared in an anonymous module).
    pub namespace: Option<String>,
    /// Original struct identifier in its declaring module.
    pub original_name: String,
    /// Canonicalized ordered slot types (`>= 1` slot).
    pub fields: Vec<StageLType>,
    /// Export visibility.
    pub is_pub: bool,
    /// Optional assigned graph Dc identity.
    pub dc_id: Option<DcChar>,
    /// Source declaration span.
    pub span: Span,
}

/// Validated StageR 3 `brand` or opaque `type` definition (`Dc 2223`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandTypeDef {
    /// Defining namespace path (`None` if declared in an anonymous module).
    pub namespace: Option<String>,
    /// Original identifier in its declaring module.
    pub original_name: String,
    /// `true` for module-opaque `type`, `false` for transparent `brand`.
    pub is_opaque: bool,
    /// Canonicalized underlying representation type.
    pub inner_ty: StageLType,
    /// Export visibility.
    pub is_pub: bool,
    /// Optional assigned graph Dc identity.
    pub dc_id: Option<DcChar>,
    /// Source declaration span.
    pub span: Span,
}

/// Validated StageR 3 named-field `struct` definition (`Dc 2224`..`Dc 2225`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedStructDef {
    /// Defining namespace path (`None` if declared in an anonymous module).
    pub namespace: Option<String>,
    /// Original struct identifier in its declaring module.
    pub original_name: String,
    /// Canonicalized ordered `(field_name, field_type)` pairs (`>= 1` field).
    pub fields: Vec<(String, StageLType)>,
    /// Export visibility.
    pub is_pub: bool,
    /// Optional assigned graph Dc identity.
    pub dc_id: Option<DcChar>,
    /// Source declaration span.
    pub span: Span,
}

/// Canonicalized variant entry in a validated [`EnumDef`] (`Dc 2232`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantDef {
    /// Variant identifier.
    pub name: String,
    /// Canonicalized optional payload type (`None` for unit variant).
    pub payload: Option<StageLType>,
    /// Source declaration span.
    pub span: Span,
}

/// Validated StageR 3 closed `enum` definition (`Dc 2230`..`Dc 2231`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDef {
    /// Defining namespace path (`None` if declared in an anonymous module).
    pub namespace: Option<String>,
    /// Original enum identifier in its declaring module.
    pub original_name: String,
    /// Canonicalized variants (`0` or more).
    pub variants: Vec<EnumVariantDef>,
    /// Export visibility.
    pub is_pub: bool,
    /// Optional assigned graph Dc identity.
    pub dc_id: Option<DcChar>,
    /// Source declaration span.
    pub span: Span,
}

/// Exported items from one compiled namespace module.
#[derive(Debug, Clone, Default)]
pub struct NamespaceExports {
    /// Exported `pub` immutable values by unique constant name.
    pub values: HashMap<String, (StageLType, Option<DcChar>)>,
    /// Exported `pub` routines by name (same arity, pairwise-distinct parameter
    /// type tuples).
    pub routines: HashMap<String, Vec<RoutineSig>>,
    /// Exported `pub` positional structs by exported name.
    pub structs: HashMap<String, StructDef>,
    /// Exported `pub` StageR 3 `brand` / opaque `type` definitions by name.
    pub brand_types: HashMap<String, BrandTypeDef>,
    /// Exported `pub` StageR 3 named-field structs by name.
    pub named_structs: HashMap<String, NamedStructDef>,
    /// Exported `pub` StageR 3 enums by name.
    pub enums: HashMap<String, EnumDef>,
    /// Transitive closure of canonical [`StructDef`]s reachable via this
    /// namespace's exports.
    pub all_structs: BTreeMap<(Option<String>, String), StructDef>,
    /// Transitive closure of canonical [`BrandTypeDef`]s reachable via exports.
    pub all_brand_types: BTreeMap<(Option<String>, String), BrandTypeDef>,
    /// Transitive closure of canonical [`NamedStructDef`]s reachable via
    /// exports.
    pub all_named_structs: BTreeMap<(Option<String>, String), NamedStructDef>,
    /// Transitive closure of canonical [`EnumDef`]s reachable via exports.
    pub all_enums: BTreeMap<(Option<String>, String), EnumDef>,
    /// Assigned Dc shorthand mappings (`dc -> ImportedSymbol`).
    pub by_dc: HashMap<DcChar, ImportedSymbol>,
}

/// Resolved imported symbol entry inside a dependent module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportedSymbol {
    /// Imported `pub` constant value `(namespace_key, original_name, ty, dc)`.
    Value {
        /// Defining namespace path key (`"std/math"`).
        namespace: String,
        /// Original exported constant name.
        original_name: String,
        /// Static value type.
        ty: StageLType,
        /// Optional Dc identity.
        dc_id: Option<DcChar>,
    },
    /// Imported `pub` routine `(namespace_key, original_name, sig)`.
    Routine {
        /// Defining namespace path key.
        namespace: String,
        /// Original exported routine name.
        original_name: String,
        /// Routine signature.
        sig: RoutineSig,
    },
    /// Imported `pub` positional `struct` type.
    Struct {
        /// Defining namespace path key.
        namespace: String,
        /// Original exported struct name.
        original_name: String,
        /// Canonicalized ordered slot types.
        fields: Vec<StageLType>,
        /// Optional Dc identity.
        dc_id: Option<DcChar>,
    },
    /// Imported `pub` StageR 3 `brand` / opaque `type` definition.
    BrandType {
        /// Defining namespace path key.
        namespace: String,
        /// Original exported nominal name.
        original_name: String,
        /// `true` for module-opaque `type`, `false` for transparent `brand`.
        is_opaque: bool,
        /// Canonicalized underlying representation type.
        inner_ty: StageLType,
        /// Optional Dc identity.
        dc_id: Option<DcChar>,
    },
    /// Imported `pub` StageR 3 named-field `struct` type.
    NamedStruct {
        /// Defining namespace path key.
        namespace: String,
        /// Original exported struct name.
        original_name: String,
        /// Canonicalized ordered `(field_name, field_type)` pairs.
        fields: Vec<(String, StageLType)>,
        /// Optional Dc identity.
        dc_id: Option<DcChar>,
    },
    /// Imported `pub` StageR 3 closed `enum` type.
    Enum {
        /// Defining namespace path key.
        namespace: String,
        /// Original exported enum name.
        original_name: String,
        /// Canonicalized enum variants.
        variants: Vec<EnumVariantDef>,
        /// Optional Dc identity.
        dc_id: Option<DcChar>,
    },
}

/// Metadata collected for a validated single module.
#[derive(Debug, Clone)]
pub struct ValidatedModule {
    /// Parsed AST.
    pub module: Module,
    /// Namespace key if `ns` was declared.
    pub namespace_key: Option<String>,
    /// Resolved `use` bindings visible inside the module (`local_name ->
    /// candidates`).
    pub imports: HashMap<String, Vec<ImportedSymbol>>,
    /// Direct Dc call/value/type targets resolved from `use dc/<sh>` or
    /// System-graph linking.
    pub dc_imports: HashMap<DcChar, ImportedSymbol>,
    /// Local or imported struct names mapped to canonical `(namespace,
    /// original_name)`.
    pub local_structs: HashMap<String, (Option<String>, String)>,
    /// Local or imported struct Dcs mapped to canonical `(namespace,
    /// original_name)`.
    pub dc_structs: HashMap<DcChar, (Option<String>, String)>,
    /// Local or imported StageR nominal names (`brand`/`type`/`enum`) mapped to
    /// canonical keys.
    pub local_nominals: HashMap<String, (Option<String>, String)>,
    /// Local or imported StageR nominal Dcs (`brand`/`type`/`enum`) mapped to
    /// canonical keys.
    pub dc_nominals: HashMap<DcChar, (Option<String>, String)>,
    /// All canonical [`StructDef`]s declared or transitively reachable in this
    /// module.
    pub struct_defs: BTreeMap<(Option<String>, String), StructDef>,
    /// All canonical [`BrandTypeDef`]s declared or transitively reachable in
    /// this module.
    pub brand_types: BTreeMap<(Option<String>, String), BrandTypeDef>,
    /// All canonical [`NamedStructDef`]s declared or transitively reachable in
    /// this module.
    pub named_structs: BTreeMap<(Option<String>, String), NamedStructDef>,
    /// All canonical [`EnumDef`]s declared or transitively reachable in this
    /// module.
    pub enums: BTreeMap<(Option<String>, String), EnumDef>,
    /// Tail-position annotations indexed by `Span` (`line`, `col`) of `Call`.
    pub tail_calls: HashSet<(usize, usize)>,
    /// Statically resolved top-level/imported routine call targets indexed by
    /// `Span` (`line`, `col`).
    pub resolved_calls: HashMap<(usize, usize), ResolvedCallTarget>,
    /// Statically resolved top-level/imported constant targets indexed by
    /// `Span` (`line`, `col`).
    pub resolved_values: HashMap<(usize, usize), ResolvedValueTarget>,
    /// Statically resolved positional `struct` targets for [`Expr::StructNew`]
    /// indexed by `Span` (`line`, `col`).
    pub resolved_structs: HashMap<(usize, usize), StructDef>,
    /// Statically resolved named `struct` targets for [`Expr::NamedStructNew`]
    /// indexed by `Span` (`line`, `col`).
    pub resolved_named_structs: HashMap<(usize, usize), NamedStructDef>,
    /// Statically resolved `(enum_key, variant_name)` targets for
    /// [`Expr::EnumVariant`] indexed by `Span` (`line`, `col`).
    pub resolved_enum_variants:
        HashMap<(usize, usize), ((Option<String>, String), String)>,
    /// Statically resolved [`StageLType`] for every [`Expr::ValueRef`] site
    /// indexed by `Span` (`line`, `col`).
    pub value_ref_types: HashMap<(usize, usize), StageLType>,
    /// Canonicalized static [`StageLType`] for every validated [`Expr`] indexed
    /// by `Span` (`line`, `col`).
    pub expr_types: HashMap<(usize, usize), StageLType>,
    /// Module-level exported items.
    pub exports: NamespaceExports,
}

impl ValidatedModule {
    /// Resolves any syntactic struct or nominal type references inside `ty` to
    /// canonical [`StructTypeRef::Canonical`] identities using this module's
    /// symbol tables (with cross-dialect fallback between structs and nominals).
    #[must_use]
    pub fn canonicalize_type(&self, ty: &StageLType) -> StageLType {
        match ty {
            StageLType::B
            | StageLType::U8
            | StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64
            | StageLType::Void
            | StageLType::Never
            | StageLType::I128
            | StageLType::U128
            | StageLType::Int
            | StageLType::Rat
            | StageLType::Dc
            | StageLType::Dcs
            | StageLType::S
            | StageLType::Error
            | StageLType::View
            | StageLType::Cursor => ty.clone(),
            StageLType::List(inner) => {
                StageLType::List(Box::new(self.canonicalize_type(inner)))
            }
            StageLType::Buf(inner) => {
                StageLType::Buf(Box::new(self.canonicalize_type(inner)))
            }
            StageLType::Tuple(elems) => StageLType::Tuple(
                elems.iter().map(|e| self.canonicalize_type(e)).collect(),
            ),
            StageLType::Struct(StructTypeRef::Ident(name)) => {
                if let Some((ns, orig)) = self.local_structs.get(name) {
                    StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else if let Some((ns, orig)) = self.local_nominals.get(name) {
                    StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else {
                    ty.clone()
                }
            }
            StageLType::Struct(StructTypeRef::Dc(dc)) => {
                if let Some((ns, orig)) = self.dc_structs.get(dc) {
                    StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else if let Some((ns, orig)) = self.dc_nominals.get(dc) {
                    StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else {
                    ty.clone()
                }
            }
            StageLType::Struct(StructTypeRef::Canonical { .. }) => ty.clone(),
            StageLType::Option(inner) => {
                StageLType::Option(Box::new(self.canonicalize_type(inner)))
            }
            StageLType::Result(inner) => {
                StageLType::Result(Box::new(self.canonicalize_type(inner)))
            }
            StageLType::Nominal(StructTypeRef::Ident(name)) => {
                if let Some((ns, orig)) = self.local_nominals.get(name) {
                    StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else if let Some((ns, orig)) = self.local_structs.get(name) {
                    StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else {
                    ty.clone()
                }
            }
            StageLType::Nominal(StructTypeRef::Dc(dc)) => {
                if let Some((ns, orig)) = self.dc_nominals.get(dc) {
                    StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else if let Some((ns, orig)) = self.dc_structs.get(dc) {
                    StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else {
                    ty.clone()
                }
            }
            StageLType::Nominal(StructTypeRef::Canonical { .. }) => ty.clone(),
        }
    }

    /// Builds a canonicalized [`RoutineKey`] for `r` declared in this module.
    #[must_use]
    pub fn routine_key(&self, r: &RoutineDecl) -> RoutineKey {
        RoutineKey {
            name: r.name.clone(),
            params: r
                .params
                .iter()
                .map(|p| self.canonicalize_type(&p.ty))
                .collect(),
        }
    }

    /// Builds a canonicalized [`ValueKey`] for `v` declared in this module.
    #[must_use]
    pub fn value_key(&self, v: &TopValueDecl) -> ValueKey {
        ValueKey {
            name: v.name.clone(),
            ty: self.canonicalize_type(&v.ty),
        }
    }
}

/// Validated multi-module workspace ordered topologically (`leaves` first).
#[derive(Debug, Clone)]
pub struct ValidatedWorkspace {
    /// Modules in dependency-safe topological order.
    pub modules: Vec<ValidatedModule>,
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
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Unit tests"
)]
mod tests {
    use super::super::parser::{
        parse_module, parse_sr3_module, scan_module_header_paths,
    };
    use super::*;

    fn validate_sr3(src: &str) -> Result<ValidatedModule> {
        let parsed = parse_sr3_module(src)?;
        validate_module(&parsed)
    }

    #[crate::ctb_test]
    fn test_psr4_paths_and_use_import_validation() {
        let files = [ModuleInput {
            source_path: Some("app/math.sl3"),
            source: "ns demo/wrong.\npub v/u32/id u32/x :: u32/x.\n",
        }];
        let err = parse_and_validate_workspace(&files)
            .expect_err("PSR-4 mismatch should fail");
        assert!(
            err.to_string().contains("PSR-4"),
            "Unexpected error: {err}"
        );

        // Cycle detection A -> B -> A
        let cyclic = [
            ModuleInput {
                source_path: Some("a/one.sl3"),
                source: "ns a/one.\nuse b/two: fnB ::.\npub v/u32/fnA :: fnB.\n",
            },
            ModuleInput {
                source_path: Some("b/two.sl3"),
                source: "ns b/two.\nuse a/one: fnA ::.\npub v/u32/fnB :: fnA.\n",
            },
        ];
        let err = parse_and_validate_workspace(&cyclic)
            .expect_err("Cyclic 'use' graph should fail");
        assert!(
            err.to_string().contains("Cyclic"),
            "Unexpected cycle error: {err}"
        );

        // Importing private symbol fails
        let priv_imp = [
            ModuleInput {
                source_path: Some("lib/secret.sl3"),
                source: "ns lib/secret.\nv/u32/hidden :: u32:1.\n",
            },
            ModuleInput {
                source_path: Some("app/main.sl3"),
                source: "ns app/main.\nuse lib/secret: hidden ::.\npub v/u32/f :: hidden.\n",
            },
        ];
        let err = parse_and_validate_workspace(&priv_imp)
            .expect_err("Importing private symbol should fail");
        assert!(
            err.to_string().contains("hidden"),
            "Unexpected error: {err}"
        );

        // Wildcard import rejected in parser
        let wildcard_src = "ns app/main.\nuse lib/mod: * ::.\n";
        let err = parse_module(wildcard_src)
            .expect_err("Wildcard use import must fail");
        assert!(err.to_string().contains("Wildcard"));

        // scan_module_header_paths verifies raw ns/use extraction
        let (ns, uses) = scan_module_header_paths(
            "ns pkg/sub.\nuse a/b: x y as z ::.\nuse c/d: k ::.\n",
        )
        .expect("scan_module_header_paths");
        assert_eq!(ns, Some(vec!["pkg".to_string(), "sub".to_string()]));
        assert_eq!(
            uses,
            vec![
                vec!["a".to_string(), "b".to_string()],
                vec!["c".to_string(), "d".to_string()],
            ]
        );
    }

    #[crate::ctb_test]
    fn test_const_purity_cycles_and_buf_local_only_rules() {
        // Const cycle
        let cycle_src = r#"
const u32/firstConst add u32/secondConst u32:1.
const u32/secondConst add u32/firstConst u32:2.
"#;
        let err = validate_module(&parse_module(cycle_src).unwrap())
            .expect_err("Const cycle must fail");
        assert!(
            err.to_string().contains("Cycle")
                || err.to_string().contains("Cyclic"),
            "Unexpected error: {err}"
        );

        // Const referencing impure/runtime routine fails
        let impure_const = r#"
v/u32/runtimeFn :: u32:5.
const u32/bad runtimeFn.
"#;
        let err = validate_module(&parse_module(impure_const).unwrap())
            .expect_err("Const calling routine must fail");
        assert!(
            err.to_string().contains("constant")
                || err.to_string().contains("const")
                || err.to_string().contains("pure")
                || err.to_string().contains("Routine"),
            "Unexpected error: {err}"
        );

        // Module-level constant holding buf/u8 fails
        let pub_buf_src = r#"
pub buf/u8/badBuf bufNew u8 u64:4.
"#;
        let err = validate_module(&parse_module(pub_buf_src).unwrap())
            .expect_err("buf in pub const must fail");
        assert!(err.to_string().contains("buf"));

        // Exporting pub routine returning buf/u8 fails
        let ret_buf_src = r#"
pub v/buf/u8/badRet :: bufNew u8 u64:4.
"#;
        let err = validate_module(&parse_module(ret_buf_src).unwrap())
            .expect_err("Exporting pub buf/T return must fail");
        assert!(err.to_string().contains("buf"));

        // Nested list of buf (l/buf/u8) fails
        let nested_buf_src = r#"
v/u32/badNested
    let l/buf/u8/xs l/buf/u8: bufNew u8 u64:1 ::.
    u32:0.
"#;
        let err = validate_module(&parse_module(nested_buf_src).unwrap())
            .expect_err("l/buf/T must fail");
        assert!(err.to_string().contains("buf"));
    }

    #[crate::ctb_test]
    fn test_lexical_non_shadowing_and_definite_assignment() {
        // Re-declaring let in same block fails
        let redecl_src = r#"
v/u32/f
    let u32/x u32:1.
    let u32/x u32:2.
    u32/x.
"#;
        let err = validate_module(&parse_module(redecl_src).unwrap())
            .expect_err("Re-declaring let must fail");
        assert!(
            err.to_string().contains("re-declare")
                || err.to_string().contains("shadow")
        );

        // Shadowing outer variable in nested block fails
        let shadow_src = r#"
v/u32/f u32/p
    if true do
        let u32/p u32:3.
        u32/p.
    :: u32:0.
"#;
        let err = validate_module(&parse_module(shadow_src).unwrap())
            .expect_err("Shadowing param must fail");
        assert!(
            err.to_string().contains("shadow")
                || err.to_string().contains("re-declare")
        );

        // Setting a constant or parameter fails
        let set_param_src = r#"
v/u32/f u32/p
    set u32/p u32:9.
    u32/p.
"#;
        let err = validate_module(&parse_module(set_param_src).unwrap())
            .expect_err("Setting param must fail");
        assert!(
            err.to_string().contains("immutable")
                || err.to_string().contains("let")
        );

        // Reading uninitialized let fails
        let uninit_read = r#"
v/u32/f
    let u32/x.
    u32/x.
"#;
        let err = validate_module(&parse_module(uninit_read).unwrap())
            .expect_err("Reading uninit let must fail");
        assert!(
            err.to_string().contains("definitely")
                || err.to_string().contains("uninitialized")
                || err.to_string().contains("before"),
            "Unexpected error: {err}"
        );

        // Initialized only in one branch fails after 'if'
        let one_branch_src = r#"
v/u32/f b/cond
    let u32/x.
    if b/cond do
        set u32/x u32:1.
        u32:0.
    :: u32:0.
    u32/x.
"#;
        validate_module(&parse_module(one_branch_src).unwrap())
            .expect_err("Partial branch init must fail");

        // Initialized inside while body is still uninit after while
        let while_init_src = r#"
v/u32/f b/cond
    let u32/x.
    while b/cond do
        set u32/x u32:1.
    ::.
    u32/x.
"#;
        validate_module(&parse_module(while_init_src).unwrap())
            .expect_err("While-body init must fail after loop");

        // One branch initializes and the other is Never (fault) -> OK!
        let both_or_never_ok = r#"
v/u32/f b/cond
    let u32/x.
    if b/cond do
        set u32/x u32:42.
    :: do
        fault.
    ::.
    u32/x.
"#;
        validate_module(&parse_module(both_or_never_ok).unwrap())
            .expect("fault else branch guarantees x assigned");

        // Nested verb capture: called BEFORE set fails
        let capture_before_set = r#"
v/u32/f
    let u32/x.
    const v/u32/readX :: u32/x.
    let u32/early readX.
    set u32/x u32:5.
    u32/early.
"#;
        validate_module(&parse_module(capture_before_set).unwrap())
            .expect_err("Calling nested verb before captured let is set must fail");

        // Nested verb capture: uncalled nested verb reading uninitialized var fails
        let capture_never_init = r#"
v/u32/f
    let u32/x.
    const v/u32/readX :: u32/x.
    u32:0.
"#;
        validate_module(&parse_module(capture_never_init).unwrap())
            .expect_err("Capturing uninitialized outer variable fails");

        // Nested verb capture: called AFTER set succeeds!
        let capture_after_set = r#"
v/u32/f
    let u32/x.
    const v/u32/readX :: u32/x.
    set u32/x u32:9.
    readX.
"#;
        validate_module(&parse_module(capture_after_set).unwrap())
            .expect("Calling nested verb after set should validate");

        // Nested verb initializes outer variable via set, then outer reads after calling it -> OK!
        let capture_init_inside_nested = r#"
v/u32/f
    let u32/x.
    const v/void/initX :: set u32/x u32:12.
    initX.
    u32/x.
"#;
        validate_module(&parse_module(capture_init_inside_nested).unwrap())
            .expect("Nested verb initializing captured var should validate");
    }

    #[crate::ctb_test]
    fn test_strict_types_loop_control_and_tail_spans() {
        // No implicit coercions between u8 / u32 / l/u8
        let bad_coerce = r#"
v/u32/f u8/alpha :: add u32:1 u8/alpha.
"#;
        let err = validate_module(&parse_module(bad_coerce).unwrap())
            .expect_err("u8 -> u32 coercion must fail");
        assert!(
            err.to_string().contains("mismatch")
                || err.to_string().contains("matching integer types"),
            "Unexpected error: {err}"
        );

        let bad_list_type = r#"
v/l/u32/f :: l/u8: u8:1 u8:2 ::.
"#;
        let err = validate_module(&parse_module(bad_list_type).unwrap())
            .expect_err("l/u8 vs l/u32 must fail");
        assert!(
            err.to_string().contains("l/u8")
                && err.to_string().contains("l/u32"),
            "Unexpected error: {err}"
        );

        // break outside loop or inside nested verb fails
        let bad_break_nested = r#"
v/u32/f
    while true do
        const v/void/stopper :: break.
        stopper.
    ::.
    u32:0.
"#;
        let err = validate_module(&parse_module(bad_break_nested).unwrap())
            .expect_err("break inside nested verb must fail");
        assert!(
            err.to_string().contains("while")
                || err.to_string().contains("loop")
        );

        // Tail call recording vs non-tail call
        let tail_src = r#"v/u32/helper u32/n :: u32/n.
v/u32/caller u32/n
    let u32/nonTail helper u32/n.
    helper u32/nonTail.
"#;
        let vm = validate_module(&parse_module(tail_src).unwrap()).unwrap();
        assert_eq!(vm.tail_calls.len(), 1);
        assert!(vm.tail_calls.contains(&(4, 5)));
    }

    #[crate::ctb_test]
    fn test_parameter_type_dispatch_and_typed_constants_validation() {
        // 1. Positive: intra-module + cross-module `use` of overloaded routines,
        // single-definition camelCase constants, optional types on `/name` and
        // `set /name`, `nop`, and unambiguous single-definition `never` arg.
        let lib_src = r#"
ns demo/math.

pub u8/u8Unit u8:1.
pub i32/i32Unit i32:1.
pub i64/i64Unit i64:1.

pub v/i32/stepAdd i32/x i32/y :: add /x /y.
pub v/i64/stepAdd i64/x i64/y :: add /x /y.
pub v/l/u8/stepAdd l/u8/prefix u64/take :: slice /prefix u64:0 /take.
pub v/i32/solo i32/x :: /x.
pub v/i32/modStageLInternal i32/x i32/y :: rem /x /y.
"#;
        let caller_src = r#"
ns demo/caller.
use demo/math: u8Unit i32Unit i64Unit stepAdd solo modStageLInternal ::.

v/u8/stepAdd u8/x u8/y :: add /x /y.

pub v/b/checkAll b/abortEarly
    if /abortEarly
        eq solo fault i32:0
        do
            let i32/acc i32:0.
            set /acc modStageLInternal i32:41 i32:42.
            if eq /acc i32:41 nop fault.
            const u8/res8 stepAdd u8:2 /u8Unit.
            const i32/res32 stepAdd i32:40 /i32Unit.
            const i64/res64 stepAdd i64:99 /i64Unit.
            const l/u8/resBytes stepAdd b"abcdef" u64:3.
            and eq /res8 u8:3 and eq /res32 i32:41 and eq /res64 i64:100 eq /resBytes b"abc".
        ::.
"#;
        let ws = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("stage_1_impls/demo/math.sl3"),
                source: lib_src,
            },
            ModuleInput {
                source_path: Some("stage_1_impls/demo/caller.sl3"),
                source: caller_src,
            },
        ])
        .expect("valid overloaded workspace should pass");
        assert_eq!(ws.modules.len(), 2);

        // 1b. `*StageLInternal` import or declaration is rejected in `impls/` or `.sr3`.
        let stage2_bad_import = r#"
ns app/user.
use demo/math: modStageLInternal as safeMod ::.

pub v/i32/run i32/x :: safeMod /x i32:2.
"#;
        let err = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("stage_1_impls/demo/math.sl3"),
                source: lib_src,
            },
            ModuleInput {
                source_path: Some("impls/app/user.sl3"),
                source: stage2_bad_import,
            },
        ])
        .expect_err("*StageLInternal import in impls/ must fail");
        assert!(
            err.to_string().contains("StageL-internal"),
            "Unexpected error: {err}"
        );

        let sr3_bad_decl = r#"
v/i32/helperStageLInternal i32/x :: /x.
"#;
        let err = parse_and_validate_workspace(&[ModuleInput {
            source_path: Some("foo.sr3"),
            source: sr3_bad_decl,
        }])
        .expect_err("*StageLInternal declaration in .sr3 must fail");
        assert!(
            err.to_string().contains("StageL-internal"),
            "Unexpected error: {err}"
        );

        // 2. Return-type-only routine overload is rejected.
        let ret_only_src = r#"
v/i32/foo i32/x :: /x.
v/i64/foo i32/x :: conv /x i64.
"#;
        let err = validate_module(&parse_module(ret_only_src).unwrap())
            .expect_err("return-type-only overload must fail");
        assert!(
            err.to_string()
                .contains("return-type-only overloading is disallowed"),
            "Unexpected error: {err}"
        );

        // 3. Mismatched-arity intra-module routine overload is rejected.
        let arity_mismatch_src = r#"
v/i32/foo i32/x :: /x.
v/i32/foo i32/x i32/y :: add /x /y.
"#;
        let err = parse_module(arity_mismatch_src)
            .and_then(|m| validate_module(&m))
            .expect_err("mismatched-arity overload must fail");
        assert!(
            err.to_string().contains("parameter count")
                || err.to_string().contains("arity"),
            "Unexpected error: {err}"
        );

        // 4. Mismatched-arity across `use` import + local routine is rejected.
        let bad_import_arity = r#"
ns demo/bad_arity.
use demo/math: solo ::.

v/i64/solo i64/x i64/y :: add /x /y.
"#;
        let err = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("stage_1_impls/demo/math.sl3"),
                source: lib_src,
            },
            ModuleInput {
                source_path: Some("stage_1_impls/demo/bad_arity.sl3"),
                source: bad_import_arity,
            },
        ])
        .expect_err("imported vs local arity mismatch must fail");
        assert!(
            err.to_string().contains("parameter count")
                || err.to_string().contains("arity"),
            "Unexpected error: {err}"
        );

        // 5. Ambiguous `never`-argument call across multiple overloads is rejected.
        let ambiguous_never_src = r#"
v/i32/pick i32/x i32/y :: add /x /y.
v/i32/pick u32/x i32/y :: /y.
v/i32/caller :: pick fault i32:1.
"#;
        let err = validate_module(&parse_module(ambiguous_never_src).unwrap())
            .expect_err("ambiguous never argument must fail");
        assert!(
            err.to_string().contains("Ambiguous"),
            "Unexpected error: {err}"
        );

        // 6. Unmatched concrete parameter tuple is rejected.
        let no_match_src = r#"
v/i32/pair i32/x i32/y :: add /x /y.
v/i64/pair i64/x i64/y :: add /x /y.
v/i64/caller :: pair i32:1 i64:2.
"#;
        let err = validate_module(&parse_module(no_match_src).unwrap())
            .expect_err("unmatched overload parameter tuple must fail");
        assert!(
            err.to_string().contains("No matching overload"),
            "Unexpected error: {err}"
        );

        // 7. Duplicate constant name is rejected even when declared with distinct types.
        let dup_const_src = r#"
const i32/K i32:1.
const i64/K i64:2.
"#;
        let err = validate_module(&parse_module(dup_const_src).unwrap())
            .expect_err("duplicate constant name must fail even with distinct types");
        assert!(
            err.to_string().contains("duplicates/shadows")
                || err.to_string().contains("Duplicate"),
            "Unexpected error: {err}"
        );

        // 8. Same identifier for both a constant and a routine is rejected.
        let const_vs_routine_src = r#"
const i32/clash i32:7.
v/i32/clash i32/x :: i32/x.
"#;
        let err = parse_module(const_vs_routine_src)
            .and_then(|m| validate_module(&m))
            .expect_err("constant vs routine name collision must fail");
        assert!(
            err.to_string().contains("constant")
                || err.to_string().contains("duplicates/shadows")
                || err.to_string().contains("Duplicate")
                || err.to_string().contains("conflicts"),
            "Unexpected error: {err}"
        );

        // 9. Nested lexical routine overloading remains prohibited.
        let nested_overload_src = r#"
v/i32/outer i32/x
    const v/i32/inner i32/a :: i32/a.
    const v/i64/inner i64/a :: i64/a.
    inner i32/x.
"#;
        let err = parse_module(nested_overload_src)
            .and_then(|m| validate_module(&m))
            .expect_err("nested lexical routine overloading must fail");
        assert!(
            err.to_string().contains("Duplicate")
                || err.to_string().contains("re-declare"),
            "Unexpected error: {err}"
        );
    }

    #[crate::ctb_test]
    fn test_tuples_and_positional_structs_validation() {
        // 1. Valid: 0/1/3-ary tuples, recursive `l/node`, coinductive `eq`/`ne`, cross-module alias + overloading.
        let lib_src = r#"ns geo/core.
pub struct point :: i32 i32.
pub struct tree :: point l/tree.
pub point/ORIGIN structNew point: i32:0 i32:0 ::.
pub v/i32/norm point/p :: add field /p u32:0 field /p u32:1.
pub v/i32/norm i32/x :: /x.
"#;
        let app_src = r#"ns geo/app.
use geo/core: point as pt tree ORIGIN norm ::.

pub v/b/check pt/a
    const tup: ::/u tupNew: ::.
    const tup: pt b ::/pair tupNew: with /a u32:0 i32:9 true ::.
    const tree/t1 structNew tree: field /pair u32:0 l/tree: :: ::.
    const tree/t2 structNew tree: structNew pt: i32:9 field /a u32:1 :: l/tree: :: ::.
    and eq /u tupNew: :: and eq /t1 /t2 eq norm /ORIGIN norm i32:0.
"#;
        let ws = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("geo/core.sl3"),
                source: lib_src,
            },
            ModuleInput {
                source_path: Some("geo/app.sl3"),
                source: app_src,
            },
        ])
        .expect("valid tuple + recursive struct workspace should validate");
        assert_eq!(ws.modules.len(), 2);

        // 2. Unguarded direct and tuple-mediated cycles must fail.
        for bad_cycle in [
            "struct bad :: bad.",
            "struct a :: tup: i32 bStruct ::.\nstruct bStruct :: a.",
        ] {
            let err = validate_module(&parse_module(bad_cycle).unwrap())
                .expect_err("unguarded struct cycle must fail");
            assert!(
                err.to_string().contains("cycle"),
                "Unexpected error: {err}"
            );
        }

        // 3. Nominal distinction: two structs with identical slots are distinct types.
        let nominal_mismatch = r#"
struct p1 :: i32 i32.
struct p2 :: i32 i32.
v/b/cmp :: eq structNew p1: i32:1 i32:2 :: structNew p2: i32:1 i32:2 ::.
"#;
        assert!(
            validate_module(&parse_module(nominal_mismatch).unwrap()).is_err()
        );

        // 4. Transitive `buf/T` prohibitions (`pub struct`, `l/holder`, `eq`, `const`).
        for bad_buf in [
            "pub struct holder :: buf/u8.",
            "struct holder :: buf/u8.\nv/u64/f :: len l/holder: ::.",
            "struct holder :: buf/u8.\nv/b/f :: eq structNew holder: bufNew u8 u64:0 :: structNew holder: bufNew u8 u64:0 ::.",
            "struct holder :: buf/u8.\nconst holder/H structNew holder: bufNew u8 u64:0 ::.",
        ] {
            assert!(
                validate_module(&parse_module(bad_buf).unwrap()).is_err(),
                "Expected error for: {bad_buf}"
            );
        }

        // 5. Out-of-bounds field index and `with` type mismatch must fail.
        for bad_field in [
            "v/i32/f :: field tupNew: i32:1 i32:2 :: u32:2.",
            "struct pt :: i32 i32.\nv/pt/f :: with structNew pt: i32:1 i32:2 :: u32:1 true.",
        ] {
            assert!(
                validate_module(&parse_module(bad_field).unwrap()).is_err(),
                "Expected error for: {bad_field}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_stager_validator_match_exhaustiveness_and_definite_assignment()
    -> Result<()> {
        // Exhaustive match across option, enum, and all 8 error tags, plus
        // definite assignment along every non-never arm and 0-variant enum.
        let valid = r#"ns demo/matchok.

enum emptyEnum ::.

enum color
    red.
    i32/rgb.

v/never/absurd emptyEnum/e :: match /e ::.

v/i32/allArms color/c option/i32/opt error/e result/void/rv
    let i32/assigned.
    match /c
        case red set /assigned i32:1.
        case rgb i32/val set /assigned /val.
    ::.
    const i32/optVal
        match /opt
            case none i32:0.
            case some i32/x /x.
        ::.
    const i32/rvVal
        match /rv
            case ok i32:0.
            case err error/_ev i32:1.
        ::.
    const i32/errVal
        match /e
            case overflow i32:1.
            case divisionByZero i32:2.
            case outOfBounds i32:3.
            case invalidEncoding i32:4.
            case invalidDc i32:5.
            case unexpectedNone i32:6.
            case frozenBuffer i32:7.
            case parseFailure i32:8.
        ::.
    wrapAdd /assigned wrapAdd /optVal wrapAdd /rvVal /errVal.
"#;
        validate_sr3(valid)?;

        // Definite assignment fails when one arm omits `set`.
        let missing_assign = r#"ns demo/badassign.
v/i32/f option/i32/opt
    let i32/x.
    match /opt
        case none nop.
        case some i32/val set /x /val.
    ::.
    /x.
"#;
        assert!(validate_sr3(missing_assign).is_err());

        // `result/void` `ok` arm must not take a payload binding.
        let bad_res_void_bind = r#"ns demo/badresvoid.
v/i32/f result/void/rv
    match /rv
        case ok i32/x /x.
        case err error/_ev i32:0.
    ::.
"#;
        assert!(validate_sr3(bad_res_void_bind).is_err());

        // Missing error tag, missing enum variant, duplicate arm, and redundant else fail.
        let missing_err_tag = r#"ns demo/misserr.
v/i32/f error/e
    match /e
        case overflow i32:1.
        case divisionByZero i32:2.
    ::.
"#;
        assert!(validate_sr3(missing_err_tag).is_err());

        let dup_arm = r#"ns demo/duparm.
v/i32/f option/i32/opt
    match /opt
        case none i32:0.
        case none i32:1.
        case some i32/x /x.
    ::.
"#;
        assert!(validate_sr3(dup_arm).is_err());

        let redundant_else = r#"ns demo/redelse.
v/i32/f option/i32/opt
    match /opt
        case none i32:0.
        case some i32/x /x.
        else i32:2.
    ::.
"#;
        assert!(validate_sr3(redundant_else).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager_validator_recursive_cycles_and_encapsulation()
    -> Result<()> {
        // Direct struct cycle and mutual brand/struct cycle must fail, even
        // through option/T (since only l/T and buf/T heap-indirect).
        let direct_cycle = r#"ns demo/cycledirect.
struct bad
    option/bad/next.
"#;
        assert!(validate_sr3(direct_cycle).is_err());

        let mutual_cycle = r#"ns demo/cyclemut.
brand bWrap/wrapped.
struct bWrap
    wrapped/inner.
"#;
        assert!(validate_sr3(mutual_cycle).is_err());

        // List-guarded recursion on both named struct and enum is valid!
        let valid_rec = r#"ns demo/recok.
pub struct tree
    i32/val.
    l/tree/children.
pub enum ast
    leaf.
    l/ast/seq.
"#;
        validate_sr3(valid_rec)?;

        // Module-opaque `type` allows wrap/unwrap inside its defining module,
        // rejects external wrap/unwrap across `use`, while `brand` allows both.
        let def_mod = r#"ns demo/opaque.
pub type u64/secretId.
pub brand u64/publicTag.

pub v/secretId/make u64/n :: wrap /n secretId.
pub v/u64/expose secretId/sec :: unwrap /sec.
"#;
        let ok_caller = r#"ns demo/user.
use demo/opaque: secretId publicTag make expose ::.

pub v/result/u64/useBoth u64/x
    const secretId/sec make /x.
    const publicTag/pubVal wrap /x publicTag.
    add expose /sec unwrap /pubVal.
"#;
        parse_and_validate_stager_workspace(&[
            ModuleInput {
                source_path: Some("demo/opaque.sr3"),
                source: def_mod,
            },
            ModuleInput {
                source_path: Some("demo/user.sr3"),
                source: ok_caller,
            },
        ])?;

        let bad_wrap_caller = r#"ns demo/badwrap.
use demo/opaque: secretId ::.
v/secretId/forge u64/x :: wrap /x secretId.
"#;
        assert!(
            parse_and_validate_stager_workspace(&[
                ModuleInput {
                    source_path: Some("demo/opaque.sr3"),
                    source: def_mod,
                },
                ModuleInput {
                    source_path: Some("demo/badwrap.sr3"),
                    source: bad_wrap_caller,
                },
            ])
            .is_err()
        );

        let bad_unwrap_caller = r#"ns demo/badunwrap.
use demo/opaque: secretId make ::.
v/u64/leak u64/x :: unwrap make /x.
"#;
        assert!(
            parse_and_validate_stager_workspace(&[
                ModuleInput {
                    source_path: Some("demo/opaque.sr3"),
                    source: def_mod,
                },
                ModuleInput {
                    source_path: Some("demo/badunwrap.sr3"),
                    source: bad_unwrap_caller,
                },
            ])
            .is_err()
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager_validator_totality_buffers_and_conv_rules() -> Result<()> {
        // `*StageLInternal` is blocked in Stage 2 (`impls/`) .sl3 modules too.
        let stage1_internal = r#"ns demo/internal.
pub v/u32/fStageLInternal :: u32:1.
"#;
        let stage2_caller = r#"ns demo/s2.
use demo/internal: fStageLInternal ::.
pub v/u32/g :: fStageLInternal.
"#;
        assert!(
            parse_and_validate_stager_workspace(&[
                ModuleInput {
                    source_path: Some("stage_1_impls/demo/internal.sl3"),
                    source: stage1_internal,
                },
                ModuleInput {
                    source_path: Some("impls/demo/s2.sl3"),
                    source: stage2_caller,
                },
            ])
            .is_err()
        );
        // Allowed inside Stage 1 (`stage_1_impls/`):
        let stage1_caller = r#"ns demo/s1.
use demo/internal: fStageLInternal ::.
pub v/u32/g :: fStageLInternal.
"#;
        parse_and_validate_stager_workspace(&[
            ModuleInput {
                source_path: Some("stage_1_impls/demo/internal.sl3"),
                source: stage1_internal,
            },
            ModuleInput {
                source_path: Some("stage_1_impls/demo/s1.sl3"),
                source: stage1_caller,
            },
        ])?;

        // `.sr3` rejects `fault` and `unwrapSome`, and requires `result`/`option`
        // for partial arithmetic/indexing plus `conv` narrowing.
        assert!(parse_sr3_module("ns demo/f.\nv/i32/f :: fault.\n").is_err());
        assert!(
            parse_sr3_module(
                "ns demo/u.\nv/i32/f option/i32/x :: unwrapSome /x.\n"
            )
            .is_err()
        );

        let wrong_div_ret = r#"ns demo/p1.
v/i32/bad i32/a i32/b :: div /a /b.
"#;
        assert!(validate_sr3(wrong_div_ret).is_err());

        let wrong_narrow_conv = r#"ns demo/c1.
v/u32/bad i32/x :: conv /x u32.
"#;
        assert!(validate_sr3(wrong_narrow_conv).is_err());

        // Widening `conv` (`u8 -> i32 -> i128 -> int -> rat`) is directly typed!
        let valid_conv = r#"ns demo/c2.
v/rat/widen u8/x :: conv conv conv conv /x i32 i128 int rat.
v/result/i64/narrow rat/r :: conv /r i64.
"#;
        validate_sr3(valid_conv)?;

        // Linear `buf/T`: `bufSet` after `bufFreeze` fails statically, as does
        // placing `buf/T` in a `pub struct` or `l/buf/T`!
        let use_after_freeze = r#"ns demo/b1.
v/result/void/bad :: do
    let buf/i32/mbuf bufNew i32 u64:2.
    const l/i32/frozen bufFreeze /mbuf.
    const u64/_len len /frozen.
    bufSet /mbuf u64:0 i32:9.
::.
"#;
        assert!(validate_sr3(use_after_freeze).is_err());

        let pub_struct_buf = r#"ns demo/b2.
pub struct leaky
    buf/u8/raw.
"#;
        assert!(validate_sr3(pub_struct_buf).is_err());

        let list_of_buf = r#"ns demo/b3.
v/u64/bad l/buf/u8/xs :: len /xs.
"#;
        assert!(validate_sr3(list_of_buf).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_cross_dialect_stagel3_rejects_stager_typed_imports() {
        let sr3_lib = r#"ns demo/srlib.

pub s/GREETING "hello".
pub int/BIG int:42.
pub brand u64/tag.
pub struct rec
    i32/a.
pub enum kind
    one.

pub v/int/idInt int/x :: /x.
pub v/option/i32/maybeNone :: option/i32:none.
"#;

        for bad_consumer in [
            "ns demo/sluser.\nuse demo/srlib: GREETING ::.\nv/u32/f :: u32:0.\n",
            "ns demo/sluser.\nuse demo/srlib: BIG ::.\nv/u32/f :: u32:0.\n",
            "ns demo/sluser.\nuse demo/srlib: idInt ::.\nv/u32/f :: u32:0.\n",
            "ns demo/sluser.\nuse demo/srlib: maybeNone ::.\nv/u32/f :: u32:0.\n",
            "ns demo/sluser.\nuse demo/srlib: tag ::.\nv/u32/f :: u32:0.\n",
            "ns demo/sluser.\nuse demo/srlib: rec ::.\nv/u32/f :: u32:0.\n",
            "ns demo/sluser.\nuse demo/srlib: kind ::.\nv/u32/f :: u32:0.\n",
        ] {
            let res = parse_and_validate_workspace(&[
                ModuleInput {
                    source_path: Some("demo/srlib.sr3"),
                    source: sr3_lib,
                },
                ModuleInput {
                    source_path: Some("demo/sluser.sl3"),
                    source: bad_consumer,
                },
            ]);
            assert!(
                res.is_err(),
                "StageL 3 must reject StageR-typed import in:\n{bad_consumer}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_cross_dialect_stager3_imports_stage1_positional_struct_as_opaque()
    -> Result<()> {
        let stage1_src = r#"ns demo/s1types.

pub struct posPair :: i32 i32.
pub struct internalPairStageLInternal :: i32 i32.

pub v/posPair/mkPair i32/a i32/b :: structNew posPair: /a /b ::.
pub v/i32/sumPair posPair/p :: add field /p u32:0 field /p u32:1.
"#;

        let sr3_valid = r#"ns demo/sruser.
use demo/s1types: posPair mkPair sumPair ::.

pub struct holder
    posPair/item.

pub v/i32/useOpaque i32/a i32/b
    const holder/h structNew holder: item mkPair /a /b ::.
    sumPair field /h item.
"#;
        let ws = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("stage_1_impls/demo/s1types.sl3"),
                source: stage1_src,
            },
            ModuleInput {
                source_path: Some("demo/sruser.sr3"),
                source: sr3_valid,
            },
        ])?;
        assert_eq!(ws.modules.len(), 2);

        // Canonicalizing nominal-parsed `posPair` in `ValidatedModule` resolves
        // to `StageLType::Struct(Canonical { ... })`.
        let sr_vm = &ws.modules[1];
        assert_eq!(
            sr_vm.canonicalize_type(&StageLType::Nominal(StructTypeRef::Ident(
                "posPair".to_owned(),
            ))),
            StageLType::Struct(StructTypeRef::Canonical {
                namespace: Some("demo/s1types".to_owned()),
                name: "posPair".to_owned(),
            })
        );

        // Constructing or projecting an opaque Stage 1 positional struct in `.sr3` fails.
        for bad_sr3 in [
            "ns demo/srbad1.\nuse demo/s1types: internalPairStageLInternal ::.\nv/i32/f :: i32:0.\n",
            "ns demo/srbad2.\nuse demo/s1types: posPair mkPair ::.\nv/i32/f :: field mkPair i32:1 i32:2 u32:0.\n",
            "ns demo/srbad3.\nuse demo/s1types: posPair ::.\nv/posPair/f :: structNew posPair: i32:1 i32:2 ::.\n",
        ] {
            let res = parse_and_validate_workspace(&[
                ModuleInput {
                    source_path: Some("stage_1_impls/demo/s1types.sl3"),
                    source: stage1_src,
                },
                ModuleInput {
                    source_path: Some(if bad_sr3.contains("srbad1") {
                        "demo/srbad1.sr3"
                    } else if bad_sr3.contains("srbad2") {
                        "demo/srbad2.sr3"
                    } else {
                        "demo/srbad3.sr3"
                    }),
                    source: bad_sr3,
                },
            ]);
            assert!(
                res.is_err(),
                "StageR 3 must reject internal/opaque field/ctor usage:\n{bad_sr3}"
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn stager_get_accepts_live_buf_and_rejects_frozen_buf() -> Result<()> {
        let valid_src = r#"ns demo/bufget_ok.
v/result/i32/peekFirst :: do
    let buf/i32/b bufNew i32 u64:2.
    bufPush /b i32:7.
    let option/i32/x get /b u64:0.
    let l/i32/out bufFreeze /b.
    const u64/_n len /out.
    okOr /x error:unexpectedNone.
::.
"#;
        validate_sr3(valid_src)?;

        let frozen_src = r#"ns demo/bufget_frozen.
v/option/i32/peekAfterFreeze :: do
    let buf/i32/b bufNew i32 u64:2.
    bufPush /b i32:7.
    let l/i32/out bufFreeze /b.
    const u64/_n len /out.
    get /b u64:0.
::.
"#;
        let err = validate_sr3(frozen_src)
            .expect_err("get after bufFreeze must fail validation")
            .to_string();
        assert!(
            err.contains("already consumed by 'bufFreeze'"),
            "Unexpected frozen-buf get error: {err}"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn stager_slice_rejects_s_and_accepts_dcs_and_list() -> Result<()> {
        let bad_s_slice = r#"ns demo/slice_s.
v/option/s/bad :: slice "hello" u64:0 u64:2.
"#;
        let err = validate_sr3(bad_s_slice)
            .expect_err("StageR slice must reject 's'")
            .to_string();
        assert!(
            err.contains("requires 'l/T' or 'dcs'"),
            "Unexpected StageR slice error: {err}"
        );

        let valid_src = r#"ns demo/slice_ok.
v/option/dcs/subDcs dcs/xs :: slice /xs u64:0 u64:2.
v/option/l/u8/subList :: slice l/u8: u8:1 u8:2 :: u64:0 u64:1.
"#;
        validate_sr3(valid_src)?;
        Ok(())
    }
}
