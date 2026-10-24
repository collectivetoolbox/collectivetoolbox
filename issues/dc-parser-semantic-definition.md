# Graph-Defined Dc Ontology & Pattern Matcher Design

## 1. Core Design Principles

### A. Strings, Binary Data, and Values as Distinct Graph Nodes
Strings, binary data, and primitive values are emitted as their own distinct graph nodes in the System block (`8589934592..=12884901887`), which other statements then reference by node ID (as illustrated in [file-info.md](file:///workspaces/ctoolbox/docs/file-info.md#L232-L251)):
- A statement assigning a property value to an entity uses two 3-token triples (`Dc 584` `Has a` followed by `Has value` referencing the `Has a` statement node and the value node):
  ```text
  l8589934592: 260 262 264 263 u41 u73 ... 261 # "Assignment operator"
  l8589934593: 584 269 572                     # Has a (Assignment operator, Canonical name)
  l8589934594: <HasValue> l8589934593 l8589934592 # Has value (l8589934593, "Assignment operator")
  ```
- Because `l8589934592` is a first-class graph node distinct from the semantic entity `269`, statements about the string itself (such as lexical breakdown or word meanings: `l8589934592` $\to$ `"assignment (sense 1, noun)"`, `"operator (sense 1, noun)"`) can be attached directly to the string node without conflating the English phrase with the semantic concept `269`.
- Furthermore, because `l8589934593` reifies the specific property possession (`269`'s `Canonical name`), qualifiers (such as `570` `Preferred` on a `569` `Replacement`) or future revisions (`591` `Supersedes`) can attach directly to `l8589934593` or `l8589934594` while keeping every statement a uniform 3-token triple (`<verb> <subject> <object>`).

### B. Enum-Like Domain Values as Semantic Entity Nodes vs. Direct String Properties
When compiling the CSV database into graph statements:
- **Enum-like classifications become semantic entity nodes**: Values that would be enum variants in Rust—such as Dc scripts (`README.scripts.csv`, e.g., `.Symbols`), Unicode scripts, Unicode and Dc General Categories (`!Cx`, `Po`, `Ll`, etc.), Unicode Bidirectional Classes (`BN`, `L`, `R`, etc.), named syntax construct categories (`README-named-types.csv`), and decomposition tags (`README-decompositions.csv`)—are generated as their own semantic entity nodes, distinct from the string nodes holding their names:
  1. `<node:str_symbols>` = `".Symbols"` (string literal node)
  2. `<node:script_symbols>` $\to$ `Is an instance of` (`586`) $\to$ `Dc Script` (`596`)
  3. `<node:has_name>` = `Has a` (`584`) `<node:script_symbols>` `Canonical name` (`572`), and `Has value` `<node:has_name>` `<node:str_symbols>`
  4. `<node:has_script>` = `Has a` (`584`) `21` (`Number sign`) `Dc Script` (`596`), and `Has value` `<node:has_script>` `<node:script_symbols>`
- **Simple text properties link directly to string nodes**: Properties such as `Canonical name`, `Description`, `Programmatic identifier`, `Nickname`, `Informal alias`, and `Annotation` do not introduce an intermediate semantic entity node; their `Has value` statement links directly to the underlying string literal node.

### C. Verb + Noun Decomposition of Relationships and Attributes
Rather than proliferating monolithic predicate Dcs that bake a verb and a specific attribute noun together (such as `"Has preferred replacement"` or `"Has Apple Uniform Type Identifier"`), statements are composed from:
1. **A small set of `.Semantic:Relationship` verbs** using clear directional phrasing (`Has a`, `Has value`, `Is equal to`, `Is an instance of`, `Is a subtype of`, `Is based on`, `Is deprecated`, `Is associated with`, `Is a closing delimiter of syntactic block for`, `Supersedes`, `Retracts`).
2. **`.Semantic` and `.Formats:identifier` nouns** (`Replacement`, `Preferred`, `Decomposition`, `Canonical name`, `Description`, `Programmatic identifier`, `Nickname`, `Informal alias`, `Formal alias`, `Annotation`, `Magic signature`, `Unicode canonical combining class`, `Unicode bidirectional class`, `Case mapping partner`, `Unicode general category`, `Dc general category`, `Unicode script`, `Dc Script`, `Cross-reference`, `Syntax rule`, `Data format or type description`, `Operating system`, plus format nouns `f264` `File extension`, `MediaType`, `UniformTypeIdentifier`, `AppleOsType`).
3. **Composable compound relationships**: A compound concept can either be expressed directly in statements or defined via `Decomposition` (e.g., decomposing a preferred replacement relationship into `Has a` + `Preferred` + `Replacement`).

### D. Generalized Regex-Like Pattern Matcher for Dcs (`.Semantic:Pattern`)
The pattern-matching Dcs in `.Semantic:Pattern` are defined generally so they serve as a reusable regex/structural-pattern-matching toolkit over Dc streams anywhere in the system—not exclusively for `:<syntax>` definitions of Dcs:
- **Reused `.Semantic` and `.EL` primitives**:
  - Inclusive range (`[start] 4 [end]`): `Dc 4` (`Range`)
    - *Historic DCE Wiki Compatibility ([representing_ranges_in_dce.txt](file:///workspaces/ctoolbox/old/wiki/ember-web-site-only-relevant/data/pages/representing_ranges_in_dce.txt), [dce_calendar_format.txt](file:///workspaces/ctoolbox/old/wiki/ember-web-site-only-relevant/data/pages/dce_calendar_format.txt))*: In DCE, `Dc 4` is strictly a binary infix operator `[start value] 4 [end value]` (`{base string #1}@4@{base string #2}`). Using `[start] 4 [end]` for character/token ranges (`a..b`) and repetition count bounds (`m..n`, `0..1`) aligns with the historic wiki syntax, provided open-ended upper bounds (`*`, `+`, `{m..}`) always supply an explicit `[end value]` token (such as `u221e` `∞` or `Dc 316` `Optional value absent`) rather than leaving `Dc 4` without a right-hand operand.
  - Grouping (`( ... )`): `Dc 298` / `Dc 299` (`Begin associativity group` / `End associativity group`)
  - Ordered sequence (`A B C`): `Dc 310` / `Dc 311` (`Begin list` / `End list`)
- **9 General Pattern-Matching Dcs in `.Semantic:Pattern`** (`Dc 275`, `Dc 300`, and `Dc 516` are kept strictly for `.EL` / `.Semantic` type expressions rather than overloaded for pattern matching):
  1. `Self reference` (`@ident("SelfReference")`): Placeholder referring to the enclosing subject entity or the pattern rule currently being defined or matched (`~` in `:<syntax>`, or recursive self-reference in a pattern).
  2. `Wildcard pattern` (`@ident("WildcardPattern")`): Matches any single Dc token or balanced syntactic frame (`.` / wildcard in patterns).
  3. `Pattern conjunction` (`@ident("PatternConjunction")`): Matches a Dc sequence only if all of the specified component patterns match (`A & B`).
  4. `Pattern alternation` (`@ident("PatternAlternation")`): Matches a Dc sequence if any of the specified alternative patterns match (`A | B`).
  5. `Pattern repetition` (`@ident("PatternRepetition")`): Quantifies a pattern element with an exact repetition count or a `Dc 4` `[start] 4 [end]` count range.
  6. `Negated pattern` (`@ident("NegatedPattern")`): Matches any single Dc token or balanced syntactic frame that does not match the specified target pattern or character set (`[^...]`).
  7. `Subpattern match` (`@ident("SubpatternMatch")`): Matches a sequence of Dcs against a referenced pattern or the syntax rule of a referenced entity or type (`[X:]`), rather than matching the referenced identifier literally.
  8. `Pattern capture` (`@ident("PatternCapture")`): Matches a pattern element and binds the matched Dc sub-sequence to an identifier (`Dc 270`..`Dc 271`) for backreference, extraction, or substitution (`[type $var]`).
  9. `Pattern replacement` (`@ident("PatternReplacement")`): Pairs a match pattern with a replacement or target Dc expression instantiated from its captured bindings (`s/pattern/replacement/` or `: lang.assign($ident, $val)`).

---

## 2. Seed Ontology Dcs Added to the CSVs (Assigned via `validate-data-tables`)

### A. `.Semantic` Nouns & Qualifiers (`src/formats/dcdata/data/categories/semantic.csv`)
- `Dc 565` (`1114677`): `Type` (`@ident("EntityType")`, `@roleDef("TypeToken")`)
- `Dc 566` (`1114678`): `Relationship type` (`@ident("RelationshipType")`, `@roleDef("TypeToken")`)
- `Dc 567` (`1114679`): `Relationship` (`@ident("Relationship")`, `@roleDef("TypeToken")`)
- `Dc 568` (`1114680`): `Entity` (`@ident("Entity")`, `@roleDef("TypeToken")`)
- `Dc 569` (`1114681`): `Replacement` (`@ident("Replacement")`)
- `Dc 570` (`1114682`): `Preferred` (`@ident("Preferred")`)
- `Dc 571` (`1114683`): `Decomposition` (`@ident("Decomposition")`)
- `Dc 572` (`1114684`): `Canonical name` (`@ident("CanonicalName")`)
- `Dc 573` (`1114685`): `Description` (`@ident("Description")`)
- `Dc 574` (`1114686`): `Programmatic identifier` (`@ident("ProgrammaticIdentifier")`)
- `Dc 575` (`1114687`): `Nickname` (`@ident("Nickname")`)
- `Dc 576` (`1114688`): `Informal alias` (`@ident("InformalAlias")`)
- `Dc 577` (`1114689`): `Formal alias` (`@ident("FormalAlias")`)
- `Dc 578` (`1114690`): `Annotation` (`@ident("Annotation")`)
- `Dc 579` (`1114691`): `Magic signature` (`@ident("MagicSignature")`)
- `Dc 580` (`1114692`): `Unicode canonical combining class` (`@ident("CanonicalCombiningClass")`)
- `Dc 581` (`1114693`): `Unicode bidirectional class` (`@ident("BidirectionalClass")`)
- `Dc 582` (`1114694`): `Case mapping partner` (`@ident("CaseMappingPartner")`)
- `Dc 583` (`1114695`): `Unicode general category` (`@ident("CharacterGeneralCategory")`)
- `Dc 608` (`1114720`): `Dc general category` (`@ident("DcGeneralCategory")`)
- `Dc 609` (`1114721`): `Unicode script` (`@ident("UnicodeScript")`)
- `Dc 596` (`1114708`): `Dc Script` (`@ident("CharacterScript")`)
- `Dc 593` (`1114705`): `Cross-reference` (`@ident("CrossReference")`)
- `Dc 594` (`1114706`): `Syntax rule` (`@ident("SyntaxRule")`)
- `Dc 595` (`1114707`): `Data format or type description` (`@ident("DataFormatDescription")`)
- `Dc 597` (`1114709`): `Operating system` (`@ident("OperatingSystem")`)

### B. `.Semantic:Relationship` Verbs (`src/formats/dcdata/data/categories/semantic.csv`)
- `Dc 584` (`1114696`): `Has a` (`@ident("HasA")`)
- `Dc 585` (`1114697`): `Is equal to` (`@ident("IsEqualTo")`)
- `Dc 586` (`1114698`): `Is an instance of` (`@ident("IsInstanceOf")`)
- `Dc 587` (`1114699`): `Is a subtype of` (`@ident("IsSubtypeOf")`)
- `Dc 588` (`1114700`): `Is based on` (`@ident("IsBasedOn")`)
- `Dc 589` (`1114701`): `Is deprecated` (`@ident("IsDeprecated")`)
- `Dc 590` (`1114702`): `Is a closing delimiter of syntactic block for` (`@ident("IsClosingDelimiterFor")`)
- `Dc 591` (`1114703`): `Supersedes` (`@ident("Supersedes")`)
- `Dc 592` (`1114704`): `Retracts` (`@ident("Retracts")`)
- `Dc 598` (`1114710`): `Is associated with` (`@ident("IsAssociatedWith")`)
- `Has value` (`@ident("HasValue")` — pending ID assignment)

### C. `.Semantic:Pattern` Pattern-Matching Primitives (`src/formats/dcdata/data/categories/semantic.csv`)
- `Dc 599` (`1114711`): `Self reference` (`@ident("SelfReference")`)
- `Dc 600` (`1114712`): `Wildcard pattern` (`@ident("WildcardPattern")`)
- `Dc 601` (`1114713`): `Pattern conjunction` (`@ident("PatternConjunction")`)
- `Dc 602` (`1114714`): `Pattern alternation` (`@ident("PatternAlternation")`)
- `Dc 603` (`1114715`): `Pattern repetition` (`@ident("PatternRepetition")`)
- `Dc 604` (`1114716`): `Negated pattern` (`@ident("NegatedPattern")`)
- `Dc 605` (`1114717`): `Subpattern match` (`@ident("SubpatternMatch")`)
- `Dc 606` (`1114718`): `Pattern capture` (`@ident("PatternCapture")`)
- `Dc 607` (`1114719`): `Pattern replacement` (`@ident("PatternReplacement")`)

### D. `.Formats:identifier` Format Nouns (`src/formats/dcdata/data/categories/formats/identifier.csv`)
- Existing `f264` (`2228488`): `FileExtension` (`File extension`)
- `f636` (`2228860`): `MediaType` (`Media type identifier`)
- `f637` (`2228861`): `UniformTypeIdentifier` (`Uniform Type Identifier`)
- `f638` (`2228862`): `AppleOsType` (`Apple OSType code`)

---

## 3. Git-Tracked System Block Ledger & Migration Order

### A. Immutable System Block Ledger (`8589934592..=12884901887`)
- Stored in Git in a human-readable, line-oriented format (one 1-statement System node per line).
- When the CSV-to-graph generator runs:
  1. **Literal & Enum Entity Nodes**: String literals, numbers, scripts (`.Symbols`, `.Semantic`, etc.), Unicode General Categories, Bidi classes, named syntax types, and decomposition tags are interned as System nodes.
  2. **Unchanged statements** retain their existing System node IDs (`8589934592 + k`).
  3. **Added statements** receive the next sequential System node IDs (`max_id + 1`, ...).
  4. **Modified statements** allocate a new System node for the updated statement plus a `Supersedes` statement node referencing the prior System node ID.
  5. **Removed statements** allocate a `Retracts` statement node referencing the prior System node ID.

### B. Next Implementation Steps
1. **Assign IDs via `./scripts/validate-data-tables`**: Run the validator/updater to assign permanent `Dc` and `Short` IDs to the new rows in `semantic.csv` and `identifier.csv`.
2. **Build the CSV $\to$ System-block compiler & Git ledger**: Emit the interned string/value nodes, enum entity nodes (scripts, categories, bidi classes, etc.), and 1-statement-per-node System block (`8589934592+`) from the CSVs, including `:<syntax>` pattern ASTs and append-only `Supersedes`/`Retracts` revision tracking.
3. **Build the lightweight seed graph loader & index**: Resolve active statements and index types, equivalences, container openers/closers, and pattern rules.
4. **Replace `SyntaxRuleResolver` and simplify [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs)**: Replace `syntax_table.generated.rs` and the procedural role/constraint tables in [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs) with queries against the indexed graph statements.
