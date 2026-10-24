### 1. Current Architectural Gaps & Open Design Decisions

Before diving into the design review and recommended strategies, here are the concrete gaps between the current System-block ledger implementation in [graph_ledger.rs](file:///workspaces/ctoolbox/src/formats/dcdata/graph_ledger.rs) and the multi-statement / derived-statement model you are picturing:

1. **No intra-document statement chaining or inline `[value]` decoding in `graph_ledger.rs` yet**:
   - [`consume_atom_dc_with`](file:///workspaces/ctoolbox/src/build_support/ctbnbdl.rs#L318-L340) currently rejects `616` (`1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`) when parsing `[atom_dc]` operands (returning `None` at line 335), and [`LedgerBuilder`](file:///workspaces/ctoolbox/src/formats/dcdata/graph_ledger.rs#L467-L490) always allocates a separate global node ID for every `584` (`1114696 : <Dc> Has a {syntax: `:~ [atom_dc] [atom_dc]`}`) statement, every `610` (`1114722 : <Dc> Has value {syntax: `:~ [atom_dc] [atom_dc]`}`) statement, and every string/integer/pattern value.
2. **No derived-statement index schema or sub-node addressing contract**:
   - In [graph_impl.rs](file:///workspaces/ctoolbox/src/storage/models/graph_impl.rs#L38-L53), storage only has the raw `nodes` table (`id`, `graph_id`, `type`, `data`, `checksum`, `timestamp`). There is not yet a derived-statements index table or a canonical way to address a statement derived from inside a multi-statement document or an ETL pair.
3. **No indexer version / transform pinning contract**:
   - As you noted, if derived statements are identified or cached without recording the semantics of the extractor that produced them, upgrading the indexer engine creates reproducibility drift.

---

### 2. Review of the Current Node Encoding (Why 14,000+ Nodes Make Sense—and Where It Stops Scaling)

In [`compile_defn_statements`](file:///workspaces/ctoolbox/src/formats/dcdata/graph_ledger.rs#L1648-L2017) and [`LedgerBuilder::apply_desired_state`](file:///workspaces/ctoolbox/src/formats/dcdata/graph_ledger.rs#L1100-L1447), roughly 1,270 CSV definitions (Dcs + Formats) plus ~130 seed entities (bidi classes, general categories, scripts, named syntax constructs) expand into **14,000+ System-block nodes** because the ledger uses **maximal reification**:

- Every single-valued or multi-valued property assertion (such as `572` — `1114684 : <Dc> Canonical name`, `573` — `1114685 : <Dc> Description`, `574` — `1114686 : <Dc> Programmatic identifier`, `594` — `1114706 : <Dc> Syntax rule`, etc.) allocates **up to 3 separate nodes**:
  1. A deduplicated value node (e.g., the string or syntax pattern token stream),
  2. A relationship-instance node `584 <subj> <prop>` (`1114696 : <Dc> Has a {syntax: `:~ [atom_dc] [atom_dc]`}`),
  3. A value-binding node `610 l<has_a_id> l<val_id>` (`1114722 : <Dc> Has value {syntax: `:~ [atom_dc] [atom_dc]`}`).
- A qualified property like `<prefer>` adds a fourth node: `584 l<has_a_id> 570` (`1114696 : <Dc> Has a {syntax: `:~ [atom_dc] [atom_dc]`}`, `1114682 : <Dc> Preferred`).

#### Verdict on the System Ledger Encoding
For the **core ontology ledger** (`assets/data/graph/*.ctbnodes`), this 1-assertion-per-node encoding is **genuinely well-designed**:
- **Surgical revision history**: When a single Dc's description or syntax rule changes in a future commit, [`LedgerBuilder::apply_desired_state`](file:///workspaces/ctoolbox/src/formats/dcdata/graph_ledger.rs#L1183-L1204) only has to append a new `610` (`1114722 : <Dc> Has value {syntax: `:~ [atom_dc] [atom_dc]`}`) node and a `591` (`1114703 : <Dc> Supersedes {syntax: `:~ [atom_dc] [atom_dc]`}`) node targeting the old `has_val_id`. The `584` (`1114696 : <Dc> Has a {syntax: `:~ [atom_dc] [atom_dc]`}`) relationship identity remains stable, and every other property of that Dc is untouched.
- **Zero-ambiguity addressing**: Every relationship instance is already a first-class node ID (`l85899...`), so any qualifier, `591` (`1114703 : <Dc> Supersedes {syntax: `:~ [atom_dc] [atom_dc]`}`), or `592` (`1114704 : <Dc> Retracts {syntax: `:~ [atom_dc] [atom_dc]`}`) can point to it directly as an `[atom_dc]`.

#### Where Maximal Node Allocation Should *Not* Be Used
You put your finger on the exact boundary: **not every value is an entity, and not every statement deserves its own top-level node**.
- **Scalar values (dates, timestamps, numbers, short strings)**: Allocating a node ID for every revision timestamp or order date means a 100,000-row spreadsheet or a busy revision log burns ratusan of thousands of node IDs and requires multi-node lookups just to compare two dates.
- **Cohesive multi-statement documents**: When a user or tool writes a document containing 20 statements, splitting that document into 40+ tiny nodes destroys document locality and makes signing, archiving, or deleting the document as a unit much harder.

---

### 3. Solving Puzzle (a): Intra-Document Statements & Lightweight Inline Values (Like Dates)

Look closely at the syntax rule you already defined in [README-named-types.csv](file:///workspaces/ctoolbox/src/formats/dcdata/data/README-named-types.csv#L22-L25) for `atom_dc` and `value`:
- `atom_dc` is `:([^255 616] | (255 (255 | 616)) | 616:)` (`1114367 : <Dc> Ignore following Dc`, `1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`).
- `value` is `:([string] | [number] | [boolean] | [data] | [date] | [duration] | [datemagnitude] | [list] | [key_value] | 511: | 276: | 540: | 279:)` (`1114388 : <Dc> Reference named object {syntax: `:~ ([identifier] | ([type] [identifier]))`}`, `1114652 : <Dc> Reference bound value {syntax: `:~ [identifier]`}`, `1114391 : <Dc> Routine invocation {syntax: `:~ [identifier]`}`).

This existing design in your grammar actually solves **both** inline dates/scalars and intra-document references without needing new top-level nodes! Here is how:

#### A1. Inline Scalars (Dates, Numbers, Strings) via `616` (`1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`)
Because every relationship verb (`584` `1114696 : <Dc> Has a {syntax: `:~ [atom_dc] [atom_dc]`}`, `610` `1114722 : <Dc> Has value {syntax: `:~ [atom_dc] [atom_dc]`}`, etc.) takes `[atom_dc] [atom_dc]`, the second operand does **not** have to be a `l<node_id>` token!
- If the value is a date (`[date]`, framed by `390`..`391` — `1114502 : <Dc> Begin time moment description {syntax: `:~ [formats:date] [^391]+ 391`}`, `1114503 : <Dc> End time moment description`), a number (`[number]`, framed by `6`..`7` — `1114118 : <Dc> Begin number`, `1114119 : <Dc> End number`), or a literal string (`[string]`, framed by `260`..`261` — `1114372 : <Dc> Begin literal value {syntax: `:~ [262:] ([^261 255] | (255 (255 | [^255])))* 261`}`, `1114373 : <Dc> End literal value {syntax: `:260 [262:] ([^261 255] | (255 (255 | [^255])))* ~`}`), you can write it **inline** inside the statement using `616` (`1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`):
  ```text
  610 <rel_target> 616 390 <date_format_and_tokens...> 391
  ```
- **How the Indexer handles inline values**: When the indexer parses `616 [date]`, it does **not** allocate a node ID. Instead, it normalizes the date (e.g., to an `i128` nanosecond epoch or chronological sort key) and stores it in a **typed scalar value column** (`val_int` / `val_timestamp` / `val_text`) in the statement index table. That gives you $O(\log N)$ B-tree range queries (`WHERE predicate = <date_prop> AND val_timestamp BETWEEN t1 AND t2`) with **zero node-ID overhead**.

#### A2. Intra-Document Chaining ("Node 123 has name, and that relationship has value 456" in the same document)
Inside a multi-statement document, why did `610` (`1114722 : <Dc> Has value {syntax: `:~ [atom_dc] [atom_dc]`}`) lack an ID to reference? Because `584 123 572` (`1114696 : <Dc> Has a {syntax: `:~ [atom_dc] [atom_dc]`}`, `1114684 : <Dc> Canonical name`) was written as an anonymous statement.
You have two clean ways to connect them inside a single document using your existing Dcs:

1. **Document-Scoped Identifier Binding (`269` + `270`..`271` + `616 540`)**:
   - Your grammar already has `269` (`1114381 : <Dc> Assignment operator {syntax: `:[identifier $ident] ~ [value $val] : lang.assign($ident, $val)`}`), `270`..`271` (`1114382 : <Dc> Begin identifier {syntax: `:~ [type]? [^271]+ 271`}`, `1114383 : <Dc> End identifier {syntax: `:270 [type]? [^271]+ ~`}`), and `540` (`1114652 : <Dc> Reference bound value {syntax: `:~ [identifier]`}`), which is already a valid `[value]` inside `616` (`1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`)!
   - If a relationship statement within a document is bound to a local identifier `$r1` (or if `[identifier]` labels the statement instance within the document's lexical scope), then any subsequent statement in the *same* document can refer to it as `616 540 270 <r1> 271`:
     - Statement 1 binds `r1 := (584 123 572)`
     - Statement 2 asserts `610 (616 540 r1) 456` — and a third statement in the same document can also qualify `r1` (`584 (616 540 r1) 570` — `1114682 : <Dc> Preferred`)!
2. **Or Compound / Nested Expression in Subject Position (`616 298 ... 299`)**:
   - When a relationship only needs a single `610` (`1114722 : <Dc> Has value {syntax: `:~ [atom_dc] [atom_dc]`}`) and no other qualifiers, allowing a grouped relationship expression `298 584 <subj> <prop> 299` (`1114410 : <Dc> Begin associativity group`, `1114411 : <Dc> End associativity group`, `1114696 : <Dc> Has a {syntax: `:~ [atom_dc] [atom_dc]`}`) as a `[value]` under `616` (`1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`) lets you write a single self-contained sentence:
     ```text
     610 616 298 584 l123 572 299 l456
     ```
     ("Has value [Evaluate as atom: (Has a l123, Canonical name)] l456").
   - When the indexer reads that nested expression, it emits both the `Has a` relationship instance and its `Has value` binding in one step, without needing even a local variable name unless multiple qualifiers refer to the same relationship instance.

---

### 4. Solving Puzzle (b) & the Excel + Schema ETL Puzzle: Addressing Derived Statements Across Documents

Now consider your harder question:
> What if Document B wants to refer to a statement derived from Document A (or derived from the combination of Excel Data Node `X` and Schema Node `S`), when those derived statements don't have their own node IDs, and character offsets alone fail because multiple statements can be derived at the same offset?

#### Key Architectural Distinction: **Asserted Nodes** vs. **Derived Index Rows**
The cleanest way to solve this is to separate **Storage Node Identity (`u128`)** from **Derived Statement Identity (`StatementKey`)**:

1. **A derived statement is identified by its *Derivation Provenance Tuple*, not a bare character offset.**
   Every statement in the index is produced by a deterministic extractor/evaluator from one or more source nodes. Its unique structural address is:
   $$\text{StatementId} = \bigl(\text{PrimarySourceNode},\; \text{TransformOrSchemaNode},\; \text{AnchorKey},\; \text{Predicate},\; \text{Ordinal}\bigr)$$
   Let's see what each component does in your two examples:
   - **Case 1: Direct multi-statement Dc document (`Node D`, no external schema)**:
     - `PrimarySourceNode` = `D`
     - `TransformOrSchemaNode` = `0` (native Dc document extractor)
     - `AnchorKey` = either the explicit local identifier (`270`..`271` — `1114382 : <Dc> Begin identifier {syntax: `:~ [type]? [^271]+ 271`}`, `1114383 : <Dc> End identifier {syntax: `:270 [type]? [^271]+ ~`}`) if the author named the statement inside `D`, **or** a semantic anchor `(statement_ast_path)` if unnamed.
     - Why is an AST path or explicit local identifier better than a raw byte/character offset? Because if multiple statements are emitted at one macro/expression site, each emitted statement has a distinct `(ast_path, emission_ordinal)` or `(local_ident, predicate)`. Even better: if an author wants a statement inside a multi-statement document to be ** stably referenceable by *other* documents** (for `591` `1114703 : <Dc> Supersedes {syntax: `:~ [atom_dc] [atom_dc]`}` or `592` `1114704 : <Dc> Retracts {syntax: `:~ [atom_dc] [atom_dc]`}`), giving it a local identifier `(Node D, Ident "rel_name")` (similar to an HTML fragment `#id` on a URL: `l<NodeD>#<ident>`) makes it immune to editing earlier bytes in a future revision!
   - **Case 2: Excel Data Node `X` + Schema/ETL Document Node `S`**:
     - `PrimarySourceNode` = `l<X>` (the immutable Excel data node)
     - `TransformOrSchemaNode` = `l<S>` (the schema/mapping node that says "Node `X` is Excel, Col A = Primary Key, Col B = Customer Name, Col C = Order Date")
     - `AnchorKey` = `(sheet_id, row_primary_key_or_row_index, column_id)` (e.g., Row key `"ORD-9921"`, Column `"C"`).
     - Notice what happens here! The combination `(l<X>, l<S>, "ORD-9921")` deterministically identifies the virtual entity for that row, and `(l<X>, l<S>, "ORD-9921", Col_C)` deterministically identifies the "Order Date" statement for that row—**without allocating 100,000 global nodes in the `nodes` table**!
     - If Document `C` later wants to override or annotate a single cell's derived statement (e.g., "Row `ORD-9921` in `(X, S)` actually has Order Date `2026-10-25`"), Document `C` can reference the **composite derivation expression** via `616` (`1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`)—for example, a routine/lookup expression `279` (`1114391 : <Dc> Routine invocation {syntax: `:~ [identifier]`}`) or path expression `(X, S, "ORD-9921", Col_C)`—or, if an entity in an ETL table is "promoted" when someone edits/annotates it, only that *one* annotated override statement becomes a new node in the local graph!

#### How the Relational / Columnar Statement Index Should Look in Turso/SQLite
Instead of forcing the `nodes` table to hold every derived triple, keep `nodes` strictly for **authored/published documents & blobs**, and maintain a **Derived Statement Index** (rebuildable at any time from `nodes`):

| Column | Purpose |
| :--- | :--- |
| `stmt_hash` (`BLOB(16)` / `u128`) | Deterministic content/address hash of `(source_node_id, schema_node_id, anchor_key, verb_dc, ordinal)` |
| `source_node_id` (`u128`) | The document or data node (`X`) from which this statement was derived |
| `schema_node_id` (`u128`) | `0` for self-describing Dc documents, or `S` when derived via a separate schema/ETL node |
| `anchor_key` (`BLOB`) | Sub-document identifier (`270`..`271`), row primary key, or AST address within `source_node_id` |
| `verb_dc` (`u128`) | Canonical predicate (`584`, `586`, `610`, or resolved property Dc like `572` for fast direct lookup) |
| `subject_key` (`BLOB`) | Either a `u128` node ID (`l123`) or a virtual entity key `Hash(X, S, row_key)` |
| `object_kind` (`u8`) | `0 = NodeId`, `1 = InlineInt`, `2 = InlineTimestampNs`, `3 = InlineString`, `4 = InlineBlob` |
| `object_node_id` (`u128 NULL`) | Populated when the object is another node/Dc |
| `object_scalar` (`INT / TEXT / BLOB`) | Populated when the object is an inline `616` (`1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`) value (e.g. a date in nanoseconds or a string) |
| `extractor_version` (`u32`) | Pinned semantic version of the extractor/rule that produced this row |

Look at the advantages of this table:
- **Fast property queries**: The indexer can materialize both the low-level reified pair (`584` `1114696 : <Dc> Has a {syntax: `:~ [atom_dc] [atom_dc]`}` + `610` `1114722 : <Dc> Has value {syntax: `:~ [atom_dc] [atom_dc]`}`) *and* a flattened `(subject, property_dc, value, reified_stmt_hash)` index row so querying `Canonical name (572)` or `Order Date` is a single index seek instead of a self-join!
- **Instant deletion / retraction cleanup**: When a local node `X` or schema `S` is deleted or distrusted (per [mvp-roadmap.md](file:///workspaces/ctoolbox/docs/mvp-roadmap.md#L68-L74)), `DELETE FROM derived_statements WHERE source_node_id = ? OR schema_node_id = ?` immediately purges every derived statement from the index.

---

### 5. Indexer Version Reproducibility & Sharding Strategies

#### Strategy 1: How to Solve Indexer Version Non-Reproducibility
You raised a subtle and critical point:
> *The version of the indexer engine would be a source of non-reproducibility of what statements are derived at a given node ID and offset.*

There are three sources of drift in an indexer, and each has a clean mitigation:
1. **Separate "Semantic Extraction Rules" from "Index Storage Layout"**:
   - An upgrade that merely adds a B-tree index, changes SQL tables, or speeds up full-text tokenization changes *how* statements are searched, not *which* semantic statements exist.
   - The actual **derivation rules** (how a Dc document or an Excel+Schema pair maps to `(subject, verb, object)` statements) should be treated as a **versioned, deterministic pure function**:
     $$\text{derive\_statements}(\text{NodeBytes},\; \text{SchemaBytes},\; \text{ExtractorSpecVersion}) \rightarrow \text{Vec}<\text{DerivedStatement}>$$
2. **Pin the Extractor/Grammar Version in the Schema or Document Envelope**:
   - When a schema node `S` describes how to ETL Excel node `X`, `S` uses declarative Dc constructs (column mappings, format Dcs like `f199`, etc.) rather than ambient hardcoded Rust heuristics.
   - For built-in format decoders (like the Excel parser), the indexer records `extractor_version` in the index chunk header. If a cross-document reference targets a derived statement, it addresses it by **semantic key** `(Node X, Schema S, RowKey "ORD-9921", ColumnProperty)`—**never** by "statement #4,312 emitted by indexer v1.2". Even if indexer v1.3 emits an *additional* metadata statement per row, `(Node X, Schema S, RowKey "ORD-9921", ColumnProperty)` still resolves to the exact same statement!
3. **Content-Addressable Statement Hashes (`stmt_hash`)**:
   - If the identity of a derived statement is `BLAKE3(source_node_id || schema_node_id || semantic_anchor_key || predicate_dc)`, then its ID depends **only** on the semantic coordinates of the assertion, never on the order in which the indexer visited the file or how many other statements the new indexer version extracted nearby.

#### Strategy 2: How to Shard the Indexer for Larger Teams
Your note in [outline.md](file:///workspaces/ctoolbox/docs/outline.md#L28) actually sketches a very strong foundation for distributed/team indexing. Combining that with your `global-graph-layout.csv` and scope/trust model suggests a **Two-Tier Sharded Index Architecture**:

1. **Shard Extraction by Source Document Range ("Map" phase)**:
   - Because deriving statements from `(Node D)` (or `(Data Node X, Schema Node S)`) is a **pure, bounded-capability function** (per [mvp-roadmap.md](file:///workspaces/ctoolbox/docs/mvp-roadmap.md#L337-L340)), any worker or peer can independently index a chunk of nodes (e.g., a `.ctbnbdl` bundle or a range of team graph IDs) without coordinating with other workers.
2. **Shard Query Indices by `(Scope/Graph, SubjectBucket)` and `(Scope/Graph, Predicate)` ("Reduce" / LSM Chunk phase)**:
   - Once statements are extracted, queries almost always enter via either:
     - **Entity lookup**: "Tell me everything about `subject_id`" -> shard by `subject_key % N` (or by node ID block).
     - **Predicate / Inverted lookup**: "Find all entities where `predicate = 572 (Canonical name)` and `object_scalar = 'foo'`", or "Find all revisions where `timestamp BETWEEN t1 AND t2`" -> shard by `predicate_dc` (each predicate or predicate group gets its own append-mostly LSM/chunked index file, just like your 1M-entry chunk idea in [outline.md](file:///workspaces/ctoolbox/docs/outline.md#L28)).
3. **Keep Trust/Scope Filtering as a Lightweight Bitmap/Join Layer Over Shared Statement Chunks**:
   - If 10 team members share the same Excel node `X` and Schema node `S`, the 100,000 derived statements from `(X, S)` only need to be extracted and stored in a **single shared immutable index artifact** keyed by `(hash(X), hash(S), extractor_version)`.
   - Each user's workspace scope simply maintains a lightweight **Trusted Node / Revision Set** (`trusted_source_nodes`). At query time, a derived index chunk is active in your view if and only if both its `source_node_id` (`X`) and `schema_node_id` (`S`) are active and trusted in your current scope!

---

### Summary of Recommended Next Design Steps

1. **Keep the 1-statement-per-node encoding for `assets/data/graph/*.ctbnodes`** (the System ontology ledger), where surgical per-property `591` (`1114703 : <Dc> Supersedes {syntax: `:~ [atom_dc] [atom_dc]`}`) and `592` (`1114704 : <Dc> Retracts {syntax: `:~ [atom_dc] [atom_dc]`}`) history on individual Dc definitions is worth the ~11× node expansion.
2. **Enable `616` (`1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`) in statement decoders** so user documents and revision logs can embed inline `[date]` (`390`..`391` — `1114502 : <Dc> Begin time moment description {syntax: `:~ [formats:date] [^391]+ 391`}`, `1114503 : <Dc> End time moment description`), `[number]` (`6`..`7` — `1114118 : <Dc> Begin number`, `1114119 : <Dc> End number`), and `[string]` (`260`..`261` — `1114372 : <Dc> Begin literal value {syntax: `:~ [262:] ([^261 255] | (255 (255 | [^255])))* 261`}`, `1114373 : <Dc> End literal value {syntax: `:260 [262:] ([^261 255] | (255 (255 | [^255])))* ~`}`) values directly in `[atom_dc]` positions without allocating a node per date/scalar.
3. **Use `616` (`1114728 : <Dc> Evaluate value as atom {syntax: `:~ [value]`}`) + local `[identifier]` bindings (`270`..`271` — `1114382 : <Dc> Begin identifier {syntax: `:~ [type]? [^271]+ 271`}`, `1114383 : <Dc> End identifier {syntax: `:270 [type]? [^271]+ ~`}` / `540` — `1114652 : <Dc> Reference bound value {syntax: `:~ [identifier]`}`) or nested `298`..`299` (`1114410 : <Dc> Begin associativity group`, `1114411 : <Dc> End associativity group`) groups** for intra-document statement chaining.
4. **Identify derived statements in the index by semantic provenance coordinates `(source_node_id, schema_node_id, semantic_anchor_key, predicate_dc)`** rather than raw byte offsets or sequential emission counters, solving both the multi-statement-at-one-offset problem and the Excel + Schema ETL problem while keeping statement identity resilient to indexer engine upgrades.