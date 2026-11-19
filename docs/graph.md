## Unified Dc Model

In the Dc encoding model, strings, data, characters, graph nodes, documents, programs, libraries, and semantic statements are all the exact same kind of entity—sequences over the full 128-bit Dc space—and can generally be used interchangeably (even if a given sequence isn't always meaningful in every context). Any sequence of Dcs is a runnable, renderable document, even if running it only produces errors as output. Do **not** invent separate traditional-software abstractions like "Unicode-only strings vs code vs content nodes" inside graph storage or tooling: a Dc `[string]` is any arbitrary Dc sequence bracketed by `Dc 260 "Begin literal value"`..`Dc 261 "End literal value"`.

## Keyword Search, Linguistic Words, and Derived Statements

Indexing printable/displayable strings and indexing words will be important for search:

- In the linguistic sense, the graph will contain dedicated nodes for **words** (one node per definition), linked via semantic relationships to their **spellings** (the associated string literal(s)) and their **meanings** (the underlying semantic sense).
- Deterministic **derivation rules** generate **derived statements** (semantic triples) from documents containing words—for example `("Contains", Doc123, StringFoo)`, and when automatic meaning classification or explicit semantic markup is present, `("Contains", Doc123, WordFoo)`.
- The spellings linked to those nodes are then indexed as searchable keywords.

## Future Query & Indexing Notes

- For reverse lookup of node definitions, comparing full definitions in-memory can become expensive when node payloads are large. A future indexed lookup matching by content checksum (computing the checksum of the search target without copying or scanning full node bodies) may be added as an alternative query form once needed.

## Statements and Disagreement

In the graph, statements shouldn't shadow previous statements of the same fact. It would imply that either both statements were true, or there was a disagreement (once the graph becomes semantically richer, it would be able to capture *disagreement* by using that, for instance - it could hold *true but contradictory* statements where two different parties believe something - e.g. if two linguists disagree about a reconstructed reading of an extinct language, the readings could both be stated as fact in the graph, and would be given statements about who the authority is who confirms that claim - then users could query based on which linguist's work they want to start from).

## Dc and Format Block Reservations

New character and format IDs are allocated using the authoritative registries
[README.dc-blocks.csv](../src/formats/dcdata/data-src/README.dc-blocks.csv) and
[README.format-blocks.csv](../src/formats/dcdata/data-src/README.format-blocks.csv).
Both record inclusive first/last long IDs and the last allocated long ID in
`0x` hexadecimal notation. Block starts end in `0`; inclusive ends end in `F`.
The region offsets are multiples of 16, so short IDs have the same alignment.

[README.scripts.csv](../src/formats/dcdata/data-src/README.scripts.csv) controls
the size of future blocks only. Changing an increment never resizes a reservation.
The allocator fills a script's existing blocks above their recorded allocation
high-water marks before reserving another block. Removing category rows does not
release their IDs or reservations. Moving an already-allocated row (`<=` its
block's recorded high-water mark) to another registered script preserves its
immutable ID while keeping future allocations for the new script in that
script's own block. The legacy ID region remains reserved.

To shrink a block manually, edit its last long ID in the registry. Keep its end
aligned to hexadecimal `F`, at or above its last allocated ID and every assigned
ID. Do not lower the allocation high-water mark to reclaim formerly assigned IDs.
A successful shrink releases only the unallocated tail for future blocks.
Manual changes must never alter an assigned ID or its graph statements.

The normal data-table validator checks both registries, including alignment,
non-overlap, script ownership, duplicate IDs, and region limits. Write mode
preflights allocations in both namespaces before saving assignments. Allocation
beyond the current region is an error; adding future regions requires explicitly
extending the layout and allocator rather than spilling into a neighboring region.
Registry reservations are written before category rows, so a later file-write
failure may leave unused reservations but will not release allocated IDs.