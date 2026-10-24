
### 1. Proposed Root Ontology Dcs to Allocate (Starting at Short Dc 565)

Currently, the highest allocated short Dc is `564` (`1114676`). Below is the minimal set of seed Dcs needed to express the toy graph roots, the CSV columns/directives, and the `:<syntax>` AST in terms of graph statements.

#### A. Core Meta-Model & Revision Roots (from [file-info.md](file:///workspaces/ctoolbox/docs/file-info.md#L234-L251))
In [file-info.md](file:///workspaces/ctoolbox/docs/file-info.md#L234-L237), nodes `1`–`4` are `type`, `relationship type`, `relationship`, and `entity`.
- **Instance-of / typing**: `<TypeOrPredicateDc> <SubjectDc> [<ObjectValueOrDc>]`
  - E.g. in a System node `S`, `HasType(Dc, TypeDc)` or directly using a binary predicate `<PredicateDc> <SubjectDc> <ObjectDc>`.
- **Root Concepts** (~6 Dcs):
  1. **Type / Class** (`1. type: type` in `file-info.md` — or reuse/Clarify relationship with `Dc 275` `* (Object or any subtype)`)
  2. **Relationship type / Predicate** (`2. type: relationship type` — defines a Dc as a binary predicate taking `[subject] [object]`)
  3. **Unary property type** (defines a Dc as a unary property/attribute taking `[subject] [value]`, or unify with `Relationship type` since both link a subject to a target/literal)
  4. **Instance of / Has type** (`subject` is an instance of `type`)
  5. **Subtype of / Implies** (`@implies(...)` — `subject` is a subtype of / entails `object`)
  6. **Supersedes / Replaces statement** (`new_statement_node` replaces `old_statement_node` when a CSV value changes)
  7. **Retracts / Withdraws statement** (`retraction_node` withdraws `old_statement_node` when a CSV property is removed)

#### B. CSV Metadata & Relationship Predicates (~15 Dcs)
Each CSV column/directive compiles into 1-statement System nodes `<PredicateDc> <TargetDc> <Value>`:
- **Naming & Documentation**:
  - `Canonical name` (Column 3 string)
  - `Deprecated flag` (`!` prefix on Column 3, or unary marker)
  - `Description` (Column 10 string)
  - `Rust identifier` (`@ident("...")`)
  - `CLI nickname` (`@nick("...")`)
  - `Informal alias` (bare alias string)
  - `Formal alias` (with qualifier or subtypes for `Correction`, `Control`, `Alternate`, `Figment`, `Abbreviation`)
  - `Annotation` (`@annotation("...")`)
- **Semantic & Lineage Relations** (Note: `<equiv>` and `<approx>` already have `Dc 397` `Equivalent representations` and `Dc 561` `Partially equivalent representations`, and `<ambiguous>` has `Dc 348` `Ambiguous values`!):
  - `Preferred replacement` (`<prefer>...`)
  - `Cross-reference` (`@xref(...)`)
  - `Based on` (`@based_on(...)`)
  - `Format chain` (`@chain(...)` — encoded directly using existing `Dc 300` union, `Dc 301` transformation, `Dc 302` conversion, `Dc 303` transmutation, `Dc 516` intersection, `Dc 535` format parameter application!)
  - `Member of script / category` (Column 8 `Script`)
- **Character & Format Properties**:
  - `Combining class` (Column 4), `Bidi class` (Column 5), `Case partner` (Column 6), `General category` (Column 7)
  - `File extension` (Column 11), `MIME type` (Column 12), `Apple UTI` (Column 13), `Apple OSType` (Column 14), `Magic name` (`@magic_name(...)`), `OS association` (`@os(...)`)

#### C. Syntax (`:<syntax>`) Graph Combinators (~10 Dcs)
Right now `:<syntax>` in CSV uses a small grammar: sequences, alternations (`|`), quantifiers (`?`, `*`, `+`, `{m,n}`), character classes (`[...]`, `[^...]`, ranges `a..b`), self-reference (`~`), rule references (`[310:]`), named constructs (`[number:Natural0]`), and rewrite actions (`: lang.assign($ident, $val)`).
Wait—several of these **already exist** as Dcs:
- Range `a..b` -> `Dc 4` (`Range`)
- Alternation `A | B` -> `Dc 516` (`Type intersection` — "any of multiple types satisfy the description")
- Conjunction `A & B` -> `Dc 300` (`Type union` — "all of multiple types satisfy the description")
- Sequence `A B C` -> `Dc 310`..`Dc 311` (`Begin list`..`End list`)
- Grouping `( ... )` -> `Dc 298`..`Dc 299` (`Begin/End associativity group`)

So to translate `:<syntax>` into graph statements, we only need a handful of syntax-specific Dcs:
1. **Has syntax rule**: Links a Dc (or named construct node) to its syntax expression node.
2. **Closes container** (`@closes(...)`): Links closer Dc (e.g. `Dc 259`) to opener Dc (e.g. `Dc 258`).
3. **Self position (`~`)**: Marker Dc representing the defined Dc's own position in its syntax pattern.
4. **Quantified pattern**: Links a pattern term to `[min, max]` bounds (covering `?` = `0..1`, `*` = `0..∞`, `+` = `1..∞`, `{m,n}` = `m..n`).
5. **Negated match (`[^...]`)**: Matches any single token/frame *except* the target set (with `Dc 255` escape awareness built into the tiny kernel).
6. **Rule reference (`[X:]`)**: Matches a complete instance of target Dc `X`'s syntax rule.
7. **Parameter binding / capture (`$var`) & Rewrite (`: action`)**: Binds a matched sub-span to an identifier (`Dc 270`..`Dc 271`) and rewrites to a canonical Dc expression (e.g., rewriting `Dc 269` into `Dc 268`).

---

### 2. How the Git-Tracked System Block File Works

To keep the generated System block (`8589934592+`) human-readable, diffable in Git, and strictly immutable:
- We store a canonical ledger file in Git (e.g., `src/formats/dcdata/data/system-graph.dcal` or `.csv`/`.dcil` with one System node per line: `<system_node_id> : <dcil_tokens> # human-readable comment`).
- When `./scripts/validate-data-tables` (or the codegen step) runs:
  1. It reads the existing `system-graph` file to load all previously published System nodes (`8589934592..=max_id`) and computes the current effective statement set.
  2. It compiles the current CSVs into the desired set of statements.
  3. **Unchanged statements** keep their existing System node IDs (`8589934592 + k`) byte-for-byte.
  4. **Added statements** are allocated the next sequential System node IDs (`max_id + 1`, ...).
  5. **Modified statements** (e.g., you edited a Dc's `:syntax` or description in CSV) allocate a new System node for the new statement *plus* a `Supersedes` node pointing to the old System node ID.
  6. **Removed statements** allocate a `Retracts` node pointing to the old System node ID.

---

### 3. Recommended Migration Order

1. **Step 1 — Allocate the seed ontology Dcs in CSV**: Agree on the exact ~25–30 short Dcs (`565`+) for the meta-model, CSV predicates, and syntax combinators.
2. **Step 2 — Build the CSV $\to$ System-block compiler & Git ledger**: Have codegen emit the 1-statement-per-node System block (`8589934592+`) from the CSVs, including `:<syntax>` ASTs, with append-only revision tracking.
3. **Step 3 — Build the tiny seed graph loader & index**: A small module (<500 lines) that reads the System block statements, resolves active statements (applying `Supersedes`/`Retracts`), and builds an in-memory index of types, equivalences, container openers/closers, and syntax rules.
4. **Step 4 — Swap `SyntaxRuleResolver` and collapse [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs)**: Replace `syntax_table.generated.rs` and the thousands of lines of hardcoded string-constraint/role logic in [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs) with queries against the indexed graph statements.
