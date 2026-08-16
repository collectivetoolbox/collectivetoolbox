//! Utilities for Unicode, including:
//! - Character descriptions, annotations, aliases, and meanings
//! - Conversion of scalars to surrogates and vice versa
//! - UCS-2 encoding and decoding from scalars

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod character_description;
pub(crate) mod data;

pub use character_description::describe;

// Re-export all Unicode scalar/surrogate and UCS-2 helpers inlined for rustdoc.
#[doc(inline)]
pub use ctb_utilities::circular_dep_unicode::*;