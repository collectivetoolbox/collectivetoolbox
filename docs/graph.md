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
release their IDs or reservations. The legacy ID region remains reserved.

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