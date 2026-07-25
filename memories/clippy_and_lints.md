# Clippy and Lint Handling Guidelines

## Address Lints Properly
- Do not add `#[allow(...)]` attributes to bypass Clippy lints or warnings (e.g., `clippy::indexing_slicing`, `clippy::string_slice`, `clippy::arithmetic_side_effects`).
- Always fix the root cause of the lint by refactoring code to be safe and idiomatic (e.g. using `.get()`, `.strip_prefix()`, `saturating_add`, etc.).
