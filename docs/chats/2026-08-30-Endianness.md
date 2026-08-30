I think I should add clippy::host_endian_bytes as a "deny" rule in my Cargo.toml. There are only two uses of to/from native bytes, and if I understand correctly, they'll mean the software breaks or behaves differently on the same input file when compiled under different platforms. Does that sound correct? Are there other Endianness gotchas I should be on the look out for?



Searched for "(to_ne_bytes|from_ne_bytes|native_endian)"
Searched for "(to_ne_bytes|from_ne_bytes|native_endian)"
Viewed bzip.rs:1220-1240
Viewed bzip.rs:1340-1360
Viewed Cargo.toml:195-230
Searched for "(to_be_bytes|from_be_bytes|to_le_bytes|from_le_bytes)"

Adding `clippy::host_endian_bytes = "deny"` to [Cargo.toml](file:///workspaces/ctoolbox/Cargo.toml) is a great idea, especially for a codebase handling file formats, storage, compression, and IPC.

---

### 1. Analysis of the Two Existing Uses

The two instances in the codebase are in [bzip.rs](file:///workspaces/ctoolbox/src/formats/compression/bzip.rs#L1229) and [bzip.rs](file:///workspaces/ctoolbox/src/formats/compression/bzip.rs#L1349):

```rust
// bzip.rs:1229
let v_u32 = u32::from_ne_bytes(v.to_ne_bytes());

// bzip.rs:1349
let v = i32::from_ne_bytes(v_u32.to_ne_bytes());
```

- **What they are doing:** They perform a two's-complement bit-cast between `i32` and `u32` without using the `as` keyword (which is forbidden by your linter).
- **Will they break on different platforms?** In this specific case, **no**, because `.to_ne_bytes()` and `from_ne_bytes()` are called immediately in sequence in the same memory space. The native endianness cancels itself out, yielding identical bit representations on both Little-Endian (x86/ARM) and Big-Endian (s390x/PowerPC) targets.
- **Resolution for the lint:** To satisfy `clippy::host_endian_bytes`, simply change them to `to_le_bytes()` and `from_le_bytes()` (or `to_be_bytes()` and `from_be_bytes()`), which achieves the exact same bitwise reinterpretation deterministically.

---

### 2. Common Endianness Gotchas to Look Out For

When writing cross-platform Rust code dealing with binary data, keep an eye out for these subtleties:

1. **Database / Key-Value Sorting of Blobs**:
   - In SQLite/Turso and key-value stores, storing integer keys as raw byte blobs with `to_le_bytes()` breaks lexicographical sorting. Storing with **`to_be_bytes()`** (big-endian) ensures lexicographical byte order matches numerical integer order for unsigned numbers.
2. **Byte Reinterpretation via Memory Transmutes / Slices**:
   - Using `bytemuck::cast_slice`, `std::mem::transmute`, or constructing integer slices from `&[u8]` directly via raw pointers assumes the in-memory byte order matches the serialized format. This will silently invert byte order on opposite-endian architectures.
3. **Bitwise Shifts vs. Byte Layout**:
   - Rust bitwise operations (`<<`, `>>`, `&`, `|`) operate on the **logical integer value**, independent of platform endianness.
   - However, reconstructing multi-byte integers from a slice like `(b[0] as u32) << 24 | (b[1] as u32) << 16` assumes Big-Endian input, whereas `(b[0] as u32) | (b[1] as u32) << 8` assumes Little-Endian input. Using explicit `u32::from_be_bytes` / `u32::from_le_bytes` is both clearer and less error-prone.
4. **Protocols with Dynamic/Negotiated Endianness (e.g., X11 / Wayland)**:
   - Some protocols (like X11 wire protocol) allow either client or server endianness indicated by a header byte (`'B'` vs `'l'`), requiring dynamic dispatch to `from_be_bytes` vs `from_le_bytes` rather than assuming host or standard network order (`be`).
5. **Floating-Point Values (`f32` / `f64`)**:
   - IEEE 754 float endianness almost always mirrors integer endianness on modern CPUs, but `f32::to_bits()` / `f32::from_bits()` produce integer bit-patterns. Serializing floats across systems should use `f32::to_be_bytes()` / `f32::to_le_bytes()` rather than transmuting pointers.
6. **Bitfield Ordering**:
   - Sub-byte bitfields in hardware headers or binary protocols have platform-dependent packing order if mapped to C struct bitfields directly. Bit-masking on explicit integer types is safer.