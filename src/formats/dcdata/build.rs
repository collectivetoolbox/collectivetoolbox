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

//! Build script regenerating the static Dc syntax table.
//!
//! Compiles the crate's own CSV dataset pipeline (validation, format
//! parsing, and syntax resolution) via `#[path]` includes — the same
//! pattern the kaitai crate's build script uses — so
//! `syntax/resolver/syntax_table.generated.rs` is regenerated from the
//! authoritative CSV dataset in `data/` before the crate itself is
//! compiled. This keeps the generated file reproducible after
//! `scripts/clean-generated` and satisfies the zero-drift regeneration
//! check in `scripts/ci-build-inner`.

#![expect(
    dead_code,
    reason = "The included crate modules carry the runtime API surface \
              (resolver trait methods, Dc constants, embedded-dataset \
              wrappers) that the build script compiles for name resolution \
              but never calls; in the library unit these items are live, \
              so item-level expectations would be unfulfilled there"
)]

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};

// Crate-root re-exports referenced by the included modules, mirroring
// `dcdata.rs`.
pub(crate) use dc_def::DcDefn;
pub(crate) use shared::{SupportLevel, validate_rust_identifier};

/// Mirrors the `dcdata.rs` embedded data accessor, reading from disk.
///
/// The build script must not embed the full `data/` directory, so the
/// embedded directory is replaced by its on-disk source; the returned
/// bytes are identical because the embedded directory is built from
/// these same files.
pub(crate) fn get_dc_data_file(key: &str) -> Option<Vec<u8>> {
    if let Some(droid_rel) = key.strip_prefix("droid/tests/") {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/droid")
            .join(droid_rel);
        return std::fs::read(path).ok();
    }
    if key.starts_with("fixtures/") {
        return ctb_utilities::load_manifest_fixture(
            env!("CARGO_MANIFEST_DIR"),
            key,
        );
    }
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join(key),
    )
    .ok()
}

/// Embedded-only `dcdata.rs` accessor, unavailable in the build context.
///
/// The build script loads the dataset from disk through
/// `validate_all_dc_files_from_disk`, so no embedded categories
/// directory exists here. If a build-time code path ever reaches this
/// shim, dataset loading fails loudly downstream (empty definition
/// lists are rejected) instead of silently indexing nothing.
pub(crate) fn get_dc_categories_dir() -> Option<&'static Dir<'static>> {
    None
}

/// Mirrors the `dcdata.rs` static of the same name. Referenced only by
/// the embedded validation wrappers, which the build script never
/// executes (it uses the from-disk loaders).
pub(crate) static FORMATS_CATEGORIES_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/data/categories/formats");

#[path = "dc.generated.rs"]
pub(crate) mod dc;
#[path = "dc_char.rs"]
pub(crate) mod dc_char;
#[path = "dc_number_minimal.rs"]
pub(crate) mod dc_number_minimal;
#[path = "dc_def.rs"]
pub(crate) mod dc_def;
#[path = "column_spec.rs"]
pub(crate) mod column_spec;
#[path = "report.rs"]
pub(crate) mod report;
#[path = "shared.rs"]
pub(crate) mod shared;
#[path = "layout.rs"]
pub(crate) mod layout;
#[path = "format.rs"]
pub(crate) mod format;
#[path = "updater.rs"]
pub(crate) mod updater;
#[path = "validation.rs"]
pub(crate) mod validation;

/// Synthetic form of the crate's `format_spec` module.
///
/// The crate's `format_spec.rs` itself is not included because child
/// modules of a `#[path]`-loaded file in a build script resolve
/// relative to the build script's directory rather than the file's
/// stem directory; declaring the children inline with explicit paths
/// loads the same files. The re-exports mirror the subset of
/// `format_spec.rs` that the included modules reference.
pub(crate) mod format_spec {
    #[path = "ast.rs"]
    pub mod ast;
    #[path = "dc_stream.rs"]
    pub mod dc_stream;
    #[path = "formatter.rs"]
    pub mod formatter;
    #[path = "parser.rs"]
    pub mod parser;
    #[path = "validator.rs"]
    pub mod validator;

    pub use ast::{FormatExpr, ParamValue};
    pub use parser::parse_format_expr;
    pub use validator::validate_format_expr;
}

/// Synthetic form of the crate's `syntax` module.
///
/// The crate's `syntax.rs` itself is deliberately not included: it
/// declares the generated `syntax_table` module this build script
/// regenerates (which cannot exist yet on a clean build) and hosts the
/// embedded-dataset loader, which relies on crate-root embeds that are
/// unavailable here. The child declarations and re-exports mirror
/// `syntax.rs` for the subset the included modules reference.
pub(crate) mod syntax {
    #[path = "ast.rs"]
    pub mod ast;
    #[path = "data_constraint.rs"]
    pub mod data_constraint;
    #[path = "parser.rs"]
    pub mod parser;
    #[path = "resolver.rs"]
    pub mod resolver;
    #[path = "validator.rs"]
    pub mod validator;

    #[path = "resolver/codegen.rs"]
    pub mod codegen;

    pub use ast::{CharTarget, DcSyntaxRule};
    pub use data_constraint::parse_data_constraint;
    pub use parser::{parse_dc_syntax, parse_target_token};
    pub use resolver::{DatasetRuleResolver, StarterSet};
    pub use validator::{
        FormatValidationRegistry, build_format_validation_registry,
        validate_dc_syntax,
    };
}

fn main() -> Result<()> {
    // Regenerate whenever the dataset inputs or the generated table
    // itself change, including after scripts/clean-generated removes
    // the table.
    println!("cargo:rerun-if-changed=data/categories");
    println!("cargo:rerun-if-changed=data/README-named-types.csv");
    println!("cargo:rerun-if-changed=data/README-decompositions.csv");
    println!("cargo:rerun-if-changed=data/README.scripts.csv");
    println!(
        "cargo:rerun-if-changed=syntax/resolver/syntax_table.generated.rs"
    );

    let repo_root = ctb_build_support::find_repository_root()?;
    syntax::codegen::generate_syntax_table_file(&repo_root)?;
    Ok(())
}
