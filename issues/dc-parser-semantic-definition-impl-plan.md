### Phase 1: System-Block Node Format & 100-Node Chunk Ledger (`assets/data/graph/`)

#### 1.1 File Layout & Chunking
* Store System-block seed graph files in `assets/data/graph/` (e.g., `assets/data/graph/system-00000.sems`, `assets/data/graph/system-00001.sems`, ...), with **at most 100 node declarations per file**.
* Keeping these files in `assets/data/graph/` without `.generated` in their filenames ensures two things:
  1. `build.rs` includes them in the embedded asset bundle (`built/assets/`).
  2. [`scripts/clean-generated`](file:///workspaces/ctoolbox/scripts/clean-generated) will **not** wipe the historical node ledger, preserving permanent Global Graph IDs across edits so `./scripts/validate-data-tables` can append Dc `591` / Dc `592` (`1114703 : <Dc> Supersedes {@ident("Supersedes")}` / `1114704 : <Dc> Retracts {@ident("Retracts")}`) revisions rather than renumbering existing nodes.
* Each non-empty, non-comment line defines one immutable node in the System block (`8589934592..=12884901887`, `l8589934592`+):
  ```text
  l<NodeID>: <DcShorthand tokens...> # <human-readable comment>
  ```
  parsed via [`DcShorthand::parse`](file:///workspaces/ctoolbox/src/storage/minimal/shorthand.rs#L103-L163).

#### 1.2 Node Kinds & Triple Encoding
Every node in `assets/data/graph/` is one of three forms:

1. **Deduplicated Value Nodes** (interned literals, numbers, and `.Semantic:Pattern` expressions):
   * **String literal** framed by Dcs `260 262 264 263 ... 261` (`1114372 : <Dc> Begin literal value {syntax: `:~ [262:] ([^261 255] | (255 (255 | [^255])))* 261`; @ident("BeginLiteral"); @roleDef()}`, `1114374 : <Dc> Begin type definition {syntax: `:~ [script:.EL Types] 263`; @ident("BeginTypeDef"); @roleDef()}`, `1114376 : <Dc> Built-in type identifier: String {@ident("TypeString"); @roleDef(); @roleDef("TypeToken"); <approx>(f80)}`, `1114375 : <Dc> End type definition {syntax: `:262 [script:.EL Types] ~`; @ident("EndTypeDef"); @roleDef()}`, `1114373 : <Dc> End literal value {syntax: `:260 [262:] ([^261 255] | (255 (255 | [^255])))* ~`; @ident("EndLiteral"); @roleDef()}`):
     ```text
     l8589934592: 260 262 264 263 u41 u73 u73 u69 u67 u6e u6d u65 u6e u74 u20 u6f u70 u65 u72 u61 u74 u6f u72 261 # "Assignment operator"
     ```
   * **Integer literal** framed by Dcs `6 f199 ... 7` (`1114118 : <Dc> Begin number {@ident("BeginNumber"); @roleDef()}`, `2228423 : Integer number [number]`, `1114119 : <Dc> End number {@ident("EndNumber"); @roleDef(); @closes(6)}`):
     ```text
     l8589934595: 6 f199 127 7 # 0
     ```
2. **Seed Classification Entity Nodes**:
   * Entities defined in the auxiliary CSV tables that do not have their own Dc/Format IDs—specifically scripts in [`README.scripts.csv`](file:///workspaces/ctoolbox/src/formats/dcdata/data/README.scripts.csv), general categories (`!Cx`, `Lu`, `Nd`, etc.) and bidi classes (`BN`, `L`, `ON`, etc.) in [`shared.rs`](file:///workspaces/ctoolbox/src/formats/dcdata/shared.rs#L25-L195), named syntax constructs (`type`, `value`, `statement`, `number`, `string`, `identifier`, `list`, etc.) in [`README-named-types.csv`](file:///workspaces/ctoolbox/src/formats/dcdata/data/README-named-types.csv), and format categories (`archive`, `number`, `os`, etc.)—receive a System-block entity node typed via Dc `586` (`1114698 : <Dc> Is an instance of {@ident("IsInstanceOf")}`) and named via Dc `584` + Dc `572` + Dc `610` (`1114696 : <Dc> Has a {@ident("HasA")}`, `1114684 : <Dc> Canonical name {@ident("CanonicalName")}`, `1114722 : <Dc> Has value {@ident("HasValue")}`).
3. **3-Token Statement Triples (`<verb> <subject> <object>`)**:
   * **Direct binary relationship** (when the verb directly links two entities):
     * `<closer> @closes(<opener>)` $\rightarrow$ `590 <closer> <opener>` using Dc `590` (`1114702 : <Dc> Is a closing delimiter of syntactic block for {@ident("IsClosingDelimiterFor")}`).
     * `<subject> @implies(<target>)` $\rightarrow$ `587 <subject> <target>` using Dc `587` (`1114699 : <Dc> Is a subtype of {@ident("IsSubtypeOf")}`).
     * `<subject> @based_on(<target>)` $\rightarrow$ `588 <subject> <target>` using Dc `588` (`1114700 : <Dc> Is based on {@ident("IsBasedOn")}`).
     * `<subject> @os(<target>)` $\rightarrow$ `598 <subject> <target>` using Dc `598` (`1114710 : <Dc> Is associated with {@ident("IsAssociatedWith")}`).
     * Deprecated `!Name` $\rightarrow$ `589 <subject> 517` using Dc `589` and Dc `517` (`1114701 : <Dc> Is deprecated {@ident("IsDeprecated")}` and `1114629 : <Dc> Boolean true {@ident("BooleanTrue"); @roleDef(); @roleDef("PrimitiveValueAtom"); @roleDef("DomainValueAtom")}`).
     * Membership in a Script, General Category, or Format Category $\rightarrow$ `586 <subject> <category_node>` using Dc `586` (`1114698 : <Dc> Is an instance of {@ident("IsInstanceOf")}`).
   * **Property with value (two-triple `Has a` + `Has value` pattern)** using Dc `584` and Dc `610` (`1114696 : <Dc> Has a {@ident("HasA")}` and `1114722 : <Dc> Has value {@ident("HasValue")}`):
     ```text
     l8589934593: 584 269 572                 # Has a (269 Assignment operator, 572 Canonical name)
     l8589934594: 610 l8589934593 l8589934592 # Has value (l8589934593, "Assignment operator")
     ```
     This uniformly encodes:
     * `Name` / `Label` / `@title(...)` $\rightarrow$ Dc `572` (`1114684 : <Dc> Canonical name {@ident("CanonicalName")}`)
     * `Description` / `Comments` $\rightarrow$ Dc `573` (`1114685 : <Dc> Description {@ident("Description")}`)
     * `@ident(...)` / `Ident` $\rightarrow$ Dc `574` (`1114686 : <Dc> Programmatic identifier {@ident("ProgrammaticIdentifier")}`)
     * `Nicknames` $\rightarrow$ Dc `575` (`1114687 : <Dc> Nickname {@ident("Nickname")}`)
     * Informal alias $\rightarrow$ Dc `576` (`1114688 : <Dc> Informal alias {@ident("InformalAlias")}`)
     * `[abbr]` / formal alias $\rightarrow$ Dc `577` (`1114689 : <Dc> Formal alias {@ident("FormalAlias")}`)
     * `◌` (Combining class) $\rightarrow$ Dc `580` (`1114692 : <Dc> Unicode canonical combining class {@ident("CanonicalCombiningClass")}`)
     * `⇆` (Bidi class) $\rightarrow$ Dc `581` (`1114693 : <Dc> Unicode bidirectional class {@ident("BidirectionalClass")}`)
     * `Aa` (Case partner) $\rightarrow$ Dc `582` (`1114694 : <Dc> Case mapping partner {@ident("CaseMappingPartner")}`)
     * `Type` (General category) $\rightarrow$ Dc `583` or Dc `608` (`1114695 : <Dc> Unicode general category {@ident("CharacterGeneralCategory")}`, `1114720 : <Dc> Dc general category {@ident("DcGeneralCategory")}`)
     * `>=xref` $\rightarrow$ Dc `593` (`1114705 : <Dc> Cross-reference {@ident("CrossReference")}`)
     * `:<syntax>` $\rightarrow$ Dc `594` (`1114706 : <Dc> Syntax rule {@ident("SyntaxRule")}`)
     * Format pipeline expression $\rightarrow$ Dc `595` (`1114707 : <Dc> Data format or type description {@ident("DataFormatDescription")}`)
     * `Script` $\rightarrow$ Dc `596` or Dc `609` (`1114708 : <Dc> Dc Script {@ident("CharacterScript")}`, `1114721 : <Dc> Unicode script {@ident("UnicodeScript")}`)
     * `Extensions` $\rightarrow$ `f264` (`2228488 : File extension [identifier]`)
     * `MIME` $\rightarrow$ `f636` (`2228860 : Media type identifier [identifier]`)
     * `UTI` $\rightarrow$ `f637` (`2228861 : Uniform Type Identifier [identifier]`)
     * `Apple Type code` $\rightarrow$ `f638` (`2228862 : Apple OSType code [identifier]`)
     * `<prefer>` replacement $\rightarrow$ `584 <subject> 569` (`Has a Replacement`), `610 <stmt> <target>`, and `584 <stmt> 570` (`Has a Preferred`) using Dc `569` and Dc `570` (`1114681 : <Dc> Replacement {@ident("Replacement")}` and `1114682 : <Dc> Preferred {@ident("Preferred")}`).

#### 1.3 Append-Only Revision Algorithm in `./scripts/validate-data-tables`
* Add a `graph_ledger` module in `ctb-formats-dcdata` invoked by [`updater.rs`](file:///workspaces/ctoolbox/src/formats/dcdata/updater.rs):
  1. **Load existing chunks** from `assets/data/graph/system-*.sems`, tracking every node ID, its canonical token list, and which nodes are currently active (not target of `591` `Supersedes` or `592` `Retracts`).
  2. **Compile desired logical statements** from the CSV database in deterministic order.
  3. **Diff desired vs. active**:
     * **Unchanged**: Keep existing node IDs untouched.
     * **Single-valued property modified** (e.g. a Dc's `572` `Canonical name` or `594` `Syntax rule` changed): Keep the `584 <subject> <prop>` (`Has a`) node if it already exists; allocate a new value node (if not already interned), a new `610 <has_a_node> <new_val_node>` (`Has value`) node at `next_id`, and a `591 <new_has_value_node> <old_has_value_node>` (`Supersedes`) node at `next_id + 1`.
     * **Statement added**: Append at `next_id` (rolling over to a new chunk file whenever `line_count == 100`).
     * **Statement removed**: Append a `592 <self_or_retract_node> <old_stmt_node>` (`Retracts`) triple.

---

### Phase 2: Compiling `:<syntax>` Rules into `.Semantic:Pattern` Graph Nodes

Every [`DcSyntaxRule`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/ast.rs#L35-L45) in the CSVs and every named construct in [`README-named-types.csv`](file:///workspaces/ctoolbox/src/formats/dcdata/data/README-named-types.csv) compiles into a `.Semantic:Pattern` value node attached to its subject via Dc `584` (`Has a`) + Dc `594` (`1114706 : <Dc> Syntax rule {@ident("SyntaxRule")}`) + Dc `610` (`Has value`):

| [`ast.rs`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/ast.rs) Construct | `.Semantic:Pattern` Graph Encoding |
| :--- | :--- |
| `SyntaxTerm::SelfRef` (`~`) | Dc `599` (`1114711 : <Dc> Self reference {@ident("SelfReference")}`) |
| `SyntaxTerm::Any` (`*`) | Dc `600` (`1114712 : <Dc> Wildcard pattern {@ident("WildcardPattern")}`) |
| `SyntaxTerm::Char(id)` (`261`, `u20`, `f199`) | Literal token `id` (or escaped with Dc `255` (`1114367 : <Dc> Ignore following Dc {@ident("Escape"); @roleDef()}`) if `id` is itself a pattern operator `599`–`607`) |
| `SyntaxTerm::Range(start, end)` (`127..190`) | `4 <start> <end>` using Dc `4` (`1114116 : <Dc> Range`) |
| `SyntaxPattern::Sequence(elems)` | Framed list `310 594 <elem_1> 312 <elem_2> ... 311` using Dcs `310`, `594`, `312`, `311` (`1114422 : <Dc> Begin list (ordered, contiguous, typed) {syntax: `:~ [number $length]? [type] ([^(311 | 312)]+ (312 [^(311 | 312)]+)*)? 311`; @ident("BeginList"); @roleDef()}`, `1114706 : <Dc> Syntax rule {@ident("SyntaxRule")}`, `1114424 : <Dc> List element separator {@ident("ListElementSeparator"); @roleDef(); @roleDef("StructuralSeparator")}`, `1114423 : <Dc> End list {@ident("EndList"); @roleDef(); @closes(310)}`) |
| `SyntaxPattern::Alternation(branches)` (`A \| B`) | `602 <list_of_branches>` using Dc `602` (`1114714 : <Dc> Pattern alternation {@ident("PatternAlternation")}`) |
| `ConstraintExpr::And(lhs, rhs)` (`A & B`) | `601 <list_of_conjuncts>` using Dc `601` (`1114713 : <Dc> Pattern conjunction {@ident("PatternConjunction")}`) |
| `Quantifier` (`?`, `*`, `+`, `{n}`, `{m..n}`) | `603 <count_or_range> <inner_pattern>` using Dc `603` (`1114715 : <Dc> Pattern repetition {@ident("PatternRepetition")}`), where unbounded upper bound uses Dc `316` (`1114428 : <Dc> Optional value absent {@ident("OptionalAbsent"); @roleDef(); @roleDef("PrimitiveValueAtom"); @roleDef("DomainValueAtom")}`): `?` = `4 0 1`, `*` = `4 0 316`, `+` = `4 1 316` |
| `SyntaxTerm::Negated(inner)` (`[^261 255]`) | `604 <inner_pattern>` using Dc `604` (`1114716 : <Dc> Negated pattern {@ident("NegatedPattern")}`) |
| `[262:]`, `[type]`, `[script:.EL Types]` | `605 <target_entity_or_type>` using Dc `605` (`1114717 : <Dc> Subpattern match {@ident("SubpatternMatch")}`) referencing the Dc, Format, Script entity node, or Named-Type entity node |
| `$ident` capture binding (`[number $length]`) | `606 270 <ident_chars> 271 <inner_pattern>` using Dc `606` and Dcs `270`..`271` (`1114718 : <Dc> Pattern capture {@ident("PatternCapture")}`, `1114382 : <Dc> Begin identifier {syntax: `:~ [type]? [^271]+ 271`; @ident("BeginIdentifier"); @roleDef()}`, `1114383 : <Dc> End identifier {syntax: `:270 [type]? [^271]+ ~`; @ident("EndIdentifier"); @roleDef()}`) |
| `SyntaxAction` (`: lang.assign($ident, $val)`) | `607 <match_pattern> <replacement_expr>` using Dc `607` (`1114719 : <Dc> Pattern replacement {@ident("PatternReplacement")}`) |

---

### Phase 3: Build-Time Translation to Static Graph Tables (`system_graph.generated.rs`)

Instead of parsing CSV strings and regex-like syntax DSL strings into `syntax_table.generated.rs`, [`src/build_support/dc_codegen.rs`](file:///workspaces/ctoolbox/src/build_support/dc_codegen.rs) will read `assets/data/graph/system-*.sems` at build time and emit `src/formats/dcdata/system_graph.generated.rs`:

1. **Resolve Revisions at Build Time**: Replay `591` (`Supersedes`) and `592` (`Retracts`) once during codegen so the generated static tables contain only **active** graph statements (while also exposing a lightweight revision lookup table for provenance inspection if needed).
2. **Precompute Equivalence & Subtype Closures**:
   * Equivalence classes from `<equiv>` / Dc `397` (`1114509 : <Dc> Equivalent representations {syntax: `:~ [list]`; @ident("EquivalentRepresentations"); @roleDef(); @roleDef("EvaluableExpressionStarter")}`) $\rightarrow$ sorted static `(u128, u128)` canonical map.
   * Transitive `586` (`Is an instance of`) and `587` (`Is a subtype of`) reachability $\rightarrow$ static sorted edge slice `SUBTYPE_AND_INSTANCE_EDGES: &[(u128, u128)]`. Checking whether token/type $A$ satisfies type/script/category $B$ becomes a 10-line binary-search reachability check over `SUBTYPE_AND_INSTANCE_EDGES`.
3. **Pre-Decoded Static Pattern Arena**:
   * Every active `.Semantic:Pattern` node (`Dc 599`–`Dc 607`) is emitted into a flat static slice `SYSTEM_PATTERN_NODES: &[CompiledPatternNode]` indexed directly by System-block node offset, with precomputed starter sets and opener/closer pairs from `590` (`IsClosingDelimiterFor`).

---

### Phase 4: Simplifying [`DcDocumentNavigator`](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs)

With `system_graph.generated.rs` in place, we can cut thousands of lines from [`src/formats/dcstring/navigator.rs`](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs) and [`src/formats/dcdata/syntax/resolver.rs`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/resolver.rs):

1. **Unify Shallow Sibling Scanning (`scan_sibling_stub_with_resolver`)**:
   * Only **4 structural primitives** remain in code:
     1. Escape Dc `255` (`1114367 : <Dc> Ignore following Dc {@ident("Escape"); @roleDef()}`) skips the next token.
     2. Base64 binary frame Dc `203`..`204` (`1114315 : <Dc> Begin encapsulated binary data {syntax: `:~ [type]? [number $length]? [checksum]? ([127..190] | 195)* 204`; @ident("StartEncapsulationBinary"); @roleDef()}` .. `1114316 : <Dc> End encapsulated binary data {@ident("EndEncapsulationBinary"); @roleDef(); @closes(203)}`) scans optional header slots and raw base64 digits (`127..190`, `195`) until `204`.
     3. Any token with an active `590` (`IsClosingDelimiterFor`) partner in the graph scans balanced nested frames until its closer (plus any trailing elements after `~` in its `594` `SyntaxRule` pattern, such as `[545:]` after `544`).
     4. Any prefix/infix operator with a `594` (`SyntaxRule`) pattern scans its operands according to its compiled `.Semantic:Pattern` sequence.
2. **Delete Hardcoded Type & Subtype Dispatch (~450 lines in [`navigator.rs:7470-7971`](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L7470-L7971))**:
   * Delete `check_type_compatibility`, `type_statically_satisfies_subtype`, `number_satisfies_subtype`, `check_single_number_subtype`, and `eval_format_number_bounds`.
   * Replace them with:
     * Graph subtype/instance reachability (`graph.satisfies_type(actual_id, expected_id)` using `586` `IsInstanceOf` and `587` `IsSubtypeOf`), and
     * Evaluating the format's `594` (`SyntaxRule`) `.Semantic:Pattern` node (`601` `PatternConjunction` + `4` `Range`) directly on the decoded number or value.
3. **Delete Bespoke Frame Evaluators in Favor of the `.Semantic:Pattern` Matcher**:
   * Consolidate `check_declared_list_length`, `check_equivalent_list_grammar`, `eval_data_construct`, `eval_typed_data_construct`, and redundant per-Dc structural validators into a single pattern matcher over `CompiledPatternNode` (`599`–`607`), using `606` (`PatternCapture`) bindings (`$length`, `$keyType`, `$valType`, `$ident`, `$val`) to enforce length/type constraints generically.
4. **Retire `DatasetRuleResolver` & `syntax_table.generated.rs`**:
   * Replace [`SyntaxRuleResolver`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/resolver.rs#L114-L202) and [` resolver/codegen.rs`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/resolver/codegen.rs) with `SystemGraphIndex` backed by `system_graph.generated.rs`.
