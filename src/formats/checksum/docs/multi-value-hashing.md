# Multi-Value Hashing Specification for Fowler-Noll-Vo (FNV)

This document describes one application protocol for multi-value hashing using
the Fowler-Noll-Vo (FNV) family of algorithms. RFC 9923 Section 4 ("Hashing
Multiple Values Together") motivates component framing and describes the
mathematical properties used here, but does not standardize a framing format.

---

## 1. Context and Problem Statement

When hashing a sequence of multiple discrete values $(X_1, X_2, \dots, X_n)$,
naive concatenation without framing creates collision vulnerabilities across
differing component boundaries. For example:
- Sequence $A = [\text{"12"}, \text{"345"}]$
- Sequence $B = [\text{"123"}, \text{"45"}]$

Both sequences evaluate to the identical byte stream `"12345"` under simple
concatenation ($X_1 \parallel X_2$), generating identical hash digests despite
representing structurally distinct data.

RFC 9923 discusses two paradigms for multi-value hashing:
1. **Sequential Chaining via `offset_basis`** (with length-preservation framing).
2. **Hierarchical / Parallel Tree Hashing** (chunk-level digest concatenation).

Neither of these paradigms requires or depends on any reference C code; they
are specified here in clean, portable mathematical and protocol terms.

---

## 2. Paradigm 1: Sequential Chaining via `offset_basis`

### 2.1 Mathematical Identity

The fundamental operational property of FNV-1a is that hashing the
concatenation of two byte sequences $X$ and $Y$ using standard basis $B_0$:

$$\text{Digest} = \text{FNV-1a}(B_0, X \parallel Y)$$

is mathematically identical to evaluating $\text{FNV-1a}$ on $X$ using $B_0$,
and then evaluating $\text{FNV-1a}$ on $Y$ using the intermediate hash state of
$X$ as the initial `offset_basis`:

$$H_X = \text{FNV-1a}(B_0, X)$$
$$\text{Digest} = \text{FNV-1a}(H_X, Y)$$

This identity holds across all bit widths (32, 64, 128, 256, 512, 1024) and all
standard variants (FNV-0, FNV-1, FNV-1a).

### 2.2 Boundary Disambiguation Framing

To eliminate boundary ambiguity introduced by concatenation, implementations
of this application protocol structure the data stream using one of the two
framing schemes below. This does not make FNV collision resistant; it only
ensures that distinct component boundaries produce distinct framed inputs.

#### Scheme A: Length-Prefixed Framing (This Protocol)

Each variable-length component $X_i$ is preceded by its length $L_i$ in bytes,
encoded as a 4-octet (32-bit) unsigned integer in big-endian network byte order.
Components longer than $2^{32}-1$ bytes cannot be represented by this protocol:

```
+--------------------+-----------------------+--------------------+-----------------------+
| Length L_1 (4B BE) | Component X_1 (L_1 B) | Length L_2 (4B BE) | Component X_2 (L_2 B) | ...
+--------------------+-----------------------+--------------------+-----------------------+
```

1. Initialize $\text{State}_0 = \text{offset\_basis}$.
2. For each component $X_i$ with length $L_i$:
   - Encode $L_i$ as 4 octets: $[(L_i \gg 24), (L_i \gg 16), (L_i \gg 8), (L_i \ \& \ 0\text{xFF})]$.
   - $\text{State}' = \text{FNV-1a-Update}(\text{State}, \text{EncodedLength})$.
   - $\text{State} = \text{FNV-1a-Update}(\text{State}', X_i)$.
3. Return $\text{State}$.

Because $L_i$ is explicitly bound to each element, $[\text{"12"}, \text{"345"}]$
hashes as `[0,0,0,2, '1', '2', 0,0,0,3, '3', '4', '5']`, whereas
$[\text{"123"}, \text{"45"}]$ hashes as `[0,0,0,3, '1', '2', '3', 0,0,0,2, '4', '5']`.
The inputs are provably distinct.

#### Scheme B: Null-Terminated / Sentinel Framing

Applicable only when component values are guaranteed not to contain a reserved
sentinel byte (such as ASCII strings without interior null bytes `0x00`):

```
+-------------------+-------------+-------------------+-------------+
| Component X_1     | 0x00 (1B)   | Component X_2     | 0x00 (1B)   | ...
+-------------------+-------------+-------------------+-------------+
```

Each component is fed into the hasher followed by a single sentinel octet `0x00`.

### 2.3 Partial Precomputation Optimization

For applications where an initial component is constant or quasi-constant
(such as an IPv6 source address, tenant identifier, or schema version) while
subsequent components vary frequently:

1. **Precompute Once:**
   $$\text{Basis}_{\text{tenant}} = \text{FNV-1a}(\text{StandardBasis}, \text{TenantID})$$
2. **Execute Per Request:**
   $$\text{Digest}_{\text{req}} = \text{FNV-1a}(\text{Basis}_{\text{tenant}}, \text{RequestData})$$

This eliminates redundant processing of the prefix bytes across requests.

---

## 3. Paradigm 2: Hierarchical / Parallel Tree Hashing

For large inputs of length $L$ on systems with parallel execution units,
sequential hashing latency is bounded by $O(L)$. RFC 9923 Section 4 discusses a
two-level parallel composition that can reduce elapsed time to
$O(L/k + k \cdot B)$; it does not define a general tree-hashing standard.

### 3.1 Tree Structure Specification

1. **Chunk Partitioning:**
   - Partition input stream $M$ of length $L$ into $k$ contiguous chunks
     $C_1, C_2, \dots, C_k$, each of size $m = \lceil L / k \rceil$ bytes.
2. **Stage 1 (Parallel Leaf Hashing):**
   - For each chunk $C_i$ concurrently:
     $$D_i = \text{FNV-1a}(\text{StandardBasis}, C_i)$$
   - Each $D_i$ produces an output digest of $B = \text{Width} / 8$ bytes.
     This protocol uses the RFC 9923 Section 2.3 little-endian byte-vector
     representation.
3. **Stage 2 (Root Digest Hashing):**
   - Concatenate all leaf digests in sequential chunk order:
     $$M_{\text{root}} = D_1 \parallel D_2 \parallel \dots \parallel D_k \quad (k \times B \text{ octets})$$
   - Compute final digest:
     $$D_{\text{final}} = \text{FNV-1a}(\text{StandardBasis}, M_{\text{root}})$$

### 3.2 Tree Hashing Complexity & Trade-Off

| Metric | Sequential FNV-1a | Tree FNV-1a ($k$ chunks) |
|---|---|---|
| Total Operations | $2L$ ops | $2(L + k \cdot B)$ ops |
| Wall-Clock Elapsed Time | $O(L)$ | $O(L/k + k \cdot B)$ |
| Memory Overhead | $O(1)$ | $O(k \cdot B)$ |

Tree hashing performs $k \cdot B$ additional bytes of computation, but reduces
critical-path latency whenever $L/k + k \cdot B < L$.

---

## 4. Byte Serialization and Interoperability

To ensure interoperability between independent implementations:
1. **Integer Digest Representation:**
   - 32-bit and 64-bit word values must format big-endian (most significant byte
     first) when emitted as hex strings.
   - In accordance with RFC 9923 Section 2.3, when serialized to binary storage
     or persistent communication arrays, words are stored little-endian
     (`bytes[0]` is least significant octet).
2. **Chaining Basis Passing:**
   - When passing an intermediate digest $D$ as `offset_basis` into subsequent
     stages, $D$ must be passed as the exact untruncated numerical state of that
     hash width.
