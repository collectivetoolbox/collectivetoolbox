The following are stable:

- Dc numbers and identity: Once a numeric Dc assignment is published, its semantic meaning will never be intentionally changed.
  - An exception was made for Dcs 286 and 287, which were inadvertently given two conflicting assignments, as documented in the changelog for August 30, 2026.
- The names in README-named-types.csv are stable, as are their associated semantic meanings, though the syntaxes may change or be extended.
- Programmatic identifiers (`@ident("...")` or the `Ident` column in format tables) on entries explicitly marked with `@stable_ident()` are stable and may be referenced by name in syntax rules (`:~` / `:`). Identifiers without `@stable_ident()` are not stable and must not be referenced by name in syntax rules or format specifications.
- No other fields should be assumed to be stable.
